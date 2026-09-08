//! SQLite user store.
//!
//! Manages users (username, argon2id password hash, role) and refresh tokens
//! (opaque SHA-256 hash, expiry, revocation).
//!
//! All plaintext passwords and tokens are handled in the caller — this module
//! stores only hashes, never plaintext.

use crate::error::ApiError;
use control_protocol::Role;
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

/// Thread-safe SQLite connection wrapper.
#[derive(Clone, Debug)]
pub struct Db {
    conn: Arc<Mutex<Connection>>,
}

impl Db {
    /// Open (or create) the SQLite database at `path`.
    ///
    /// Runs schema migrations on every open.
    ///
    /// # Errors
    /// Returns `ApiError::Internal` on connection or migration failure.
    pub fn open(path: &str) -> Result<Self, ApiError> {
        let conn = Connection::open(path).map_err(|e| ApiError::Internal(e.to_string()))?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.migrate()?;
        Ok(db)
    }

    /// Open an in-memory database (testing only).
    ///
    /// # Errors
    /// Returns `ApiError::Internal` on failure.
    pub fn open_in_memory() -> Result<Self, ApiError> {
        let conn = Connection::open_in_memory().map_err(|e| ApiError::Internal(e.to_string()))?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.migrate()?;
        Ok(db)
    }

    fn migrate(&self) -> Result<(), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.execute_batch(
            "
            PRAGMA journal_mode=WAL;
            PRAGMA foreign_keys=ON;

            CREATE TABLE IF NOT EXISTS users (
                id       INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                pw_hash  TEXT NOT NULL,
                role     TEXT NOT NULL CHECK(role IN ('ADMIN','ENGINEER','MUSICIAN')),
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            );

            CREATE TABLE IF NOT EXISTS refresh_tokens (
                id         INTEGER PRIMARY KEY AUTOINCREMENT,
                user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                token_hash TEXT NOT NULL UNIQUE,
                expires_at INTEGER NOT NULL,
                revoked    INTEGER NOT NULL DEFAULT 0,
                family     TEXT NOT NULL,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            );
            ",
        )
        .map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Create a user. `pw_hash` must be an Argon2id PHC string.
    ///
    /// # Errors
    /// Returns `ApiError::Internal` on DB error or `ApiError::BadRequest` on duplicate.
    pub fn create_user(&self, username: &str, pw_hash: &str, role: Role) -> Result<(), ApiError> {
        let role_str = role_to_str(role);
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.execute(
            "INSERT INTO users (username, pw_hash, role) VALUES (?1, ?2, ?3)",
            params![username, pw_hash, role_str],
        )
        .map_err(|e| {
            if e.to_string().contains("UNIQUE") {
                ApiError::BadRequest(format!("user '{username}' already exists"))
            } else {
                ApiError::Internal(e.to_string())
            }
        })?;
        Ok(())
    }

    /// Look up a user by username. Returns `(id, pw_hash, role)`.
    ///
    /// # Errors
    /// Returns `ApiError::Unauthorized` if user not found.
    pub fn find_user(&self, username: &str) -> Result<(i64, String, Role), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.query_row(
            "SELECT id, pw_hash, role FROM users WHERE username = ?1",
            params![username],
            |row| {
                let id: i64 = row.get(0)?;
                let pw_hash: String = row.get(1)?;
                let role_str: String = row.get(2)?;
                Ok((id, pw_hash, role_str))
            },
        )
        .map_err(|_| ApiError::Unauthorized("invalid credentials"))
        .and_then(|(id, hash, role_str)| {
            let role = str_to_role(&role_str)?;
            Ok((id, hash, role))
        })
    }

    /// Store a refresh token hash for a user.
    ///
    /// `token_hash` must be a hex-encoded SHA-256 of the raw opaque token.
    /// `expires_at` is Unix seconds.
    ///
    /// # Errors
    /// Returns `ApiError::Internal` on DB error.
    pub fn store_refresh_token(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: u64,
        family: &str,
    ) -> Result<(), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.execute(
            "INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family) VALUES (?1, ?2, ?3, ?4)",
            params![user_id, token_hash, expires_at.cast_signed(), family],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Validate and rotate a refresh token.
    ///
    /// Returns `(user_id, family)` if valid. Marks the old token revoked.
    /// If the token is already revoked (reuse), revokes the entire family.
    ///
    /// # Errors
    /// Returns `ApiError::Unauthorized` if token not found, revoked, or expired.
    pub fn rotate_refresh_token(
        &self,
        token_hash: &str,
        now_unix: u64,
    ) -> Result<(i64, String), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;

        let result: rusqlite::Result<(i64, i64, i64, String)> = conn.query_row(
            "SELECT id, user_id, revoked, family FROM refresh_tokens WHERE token_hash = ?1",
            params![token_hash],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
        );

        let (tok_id, user_id, revoked, family) =
            result.map_err(|_| ApiError::Unauthorized("refresh token not found"))?;

        if revoked != 0 {
            // Reuse detected: revoke entire family
            conn.execute(
                "UPDATE refresh_tokens SET revoked = 1 WHERE family = ?1",
                params![family],
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;
            return Err(ApiError::Unauthorized(
                "refresh token reuse detected — family revoked",
            ));
        }

        // Check expiry
        let expires: i64 = conn
            .query_row(
                "SELECT expires_at FROM refresh_tokens WHERE id = ?1",
                params![tok_id],
                |r| r.get(0),
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        if (now_unix.cast_signed()) > expires {
            return Err(ApiError::Unauthorized("refresh token expired"));
        }

        // Mark used
        conn.execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE id = ?1",
            params![tok_id],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;

        Ok((user_id, family))
    }

    /// Find a user by numeric ID. Returns `(username, role)`.
    ///
    /// # Errors
    /// Returns `ApiError::Unauthorized` if user not found.
    pub fn find_user_by_id(&self, user_id: i64) -> Result<(String, Role), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.query_row(
            "SELECT username, role FROM users WHERE id = ?1",
            params![user_id],
            |row| {
                let username: String = row.get(0)?;
                let role_str: String = row.get(1)?;
                Ok((username, role_str))
            },
        )
        .map_err(|_| ApiError::Unauthorized("user not found"))
        .and_then(|(username, role_str)| {
            let role = str_to_role(&role_str)?;
            Ok((username, role))
        })
    }

    /// Revoke all refresh tokens for a user (logout).
    ///
    /// # Errors
    /// Returns `ApiError::Internal` on DB error.
    pub fn revoke_all_for_user(&self, user_id: i64) -> Result<(), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE user_id = ?1",
            params![user_id],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok(())
    }
}

fn role_to_str(role: Role) -> &'static str {
    match role {
        Role::Admin => "ADMIN",
        Role::Engineer => "ENGINEER",
        Role::Musician => "MUSICIAN",
    }
}

fn str_to_role(s: &str) -> Result<Role, ApiError> {
    match s {
        "ADMIN" => Ok(Role::Admin),
        "ENGINEER" => Ok(Role::Engineer),
        "MUSICIAN" => Ok(Role::Musician),
        _ => Err(ApiError::Internal(format!("unknown role in db: {s}"))),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn setup() -> Db {
        Db::open_in_memory().unwrap()
    }

    #[test]
    fn create_and_find_user() {
        let db = setup();
        db.create_user(
            "alice",
            "$argon2id$v=19$m=19456,t=2,p=1$fakesalt$fakehash",
            Role::Engineer,
        )
        .unwrap();
        let (id, hash, role) = db.find_user("alice").unwrap();
        assert!(id > 0);
        assert!(hash.starts_with("$argon2id$"));
        assert_eq!(role, Role::Engineer);
    }

    #[test]
    fn duplicate_user_is_bad_request() {
        let db = setup();
        db.create_user("bob", "hash1", Role::Musician).unwrap();
        let err = db.create_user("bob", "hash2", Role::Musician).unwrap_err();
        assert!(matches!(err, ApiError::BadRequest(_)));
    }

    #[test]
    fn unknown_user_returns_unauthorized() {
        let db = setup();
        assert!(matches!(
            db.find_user("nobody"),
            Err(ApiError::Unauthorized(_))
        ));
    }

    #[test]
    fn refresh_token_rotate_happy_path() {
        let db = setup();
        db.create_user("carol", "hash", Role::Musician).unwrap();
        let (user_id, _, _) = db.find_user("carol").unwrap();
        let now = 1_000_000_u64;
        db.store_refresh_token(user_id, "tokenhash_a", now + 1000, "fam-1")
            .unwrap();
        let (uid, fam) = db.rotate_refresh_token("tokenhash_a", now).unwrap();
        assert_eq!(uid, user_id);
        assert_eq!(fam, "fam-1");
    }

    #[test]
    fn refresh_token_reuse_revokes_family() {
        let db = setup();
        db.create_user("dave", "hash", Role::Musician).unwrap();
        let (user_id, _, _) = db.find_user("dave").unwrap();
        let now = 1_000_000_u64;
        db.store_refresh_token(user_id, "tokenhash_b", now + 1000, "fam-2")
            .unwrap();
        // First rotation — ok
        db.rotate_refresh_token("tokenhash_b", now).unwrap();
        // Reuse — should fail and revoke family
        let err = db.rotate_refresh_token("tokenhash_b", now).unwrap_err();
        assert!(matches!(err, ApiError::Unauthorized(_)));
    }

    #[test]
    fn revoke_all_for_user_makes_tokens_invalid() {
        let db = setup();
        db.create_user("eve", "hash", Role::Admin).unwrap();
        let (user_id, _, _) = db.find_user("eve").unwrap();
        let now = 1_000_000_u64;
        db.store_refresh_token(user_id, "tokenhash_c", now + 1000, "fam-3")
            .unwrap();
        db.revoke_all_for_user(user_id).unwrap();
        let err = db.rotate_refresh_token("tokenhash_c", now).unwrap_err();
        assert!(matches!(err, ApiError::Unauthorized(_)));
    }
}
