//! SQLite user store.

#![allow(
    clippy::cast_possible_wrap,
    clippy::cast_sign_loss,
    clippy::missing_errors_doc
)]
//!
//! Manages users (username, argon2id password hash, role) and refresh tokens
//! (opaque SHA-256 hash, expiry, revocation).
//!
//! All plaintext passwords and tokens are handled in the caller — this module
//! stores only hashes, never plaintext.

use crate::auth::hash_password;
use crate::error::ApiError;
use control_protocol::Role;
use rusqlite::{params, Connection, OptionalExtension};
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

            CREATE TABLE IF NOT EXISTS access_sessions (
                jti        TEXT PRIMARY KEY,
                user_id    INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
                session_id INTEGER NOT NULL REFERENCES refresh_tokens(id) ON DELETE CASCADE,
                expires_at INTEGER NOT NULL,
                revoked    INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL DEFAULT (unixepoch())
            );

            CREATE TABLE IF NOT EXISTS mix_assignments (
                mix_index INTEGER PRIMARY KEY,
                user_id INTEGER NOT NULL UNIQUE REFERENCES users(id) ON DELETE CASCADE,
                assigned_at INTEGER NOT NULL DEFAULT (unixepoch())
            );

            CREATE TABLE IF NOT EXISTS migrations (
                id INTEGER PRIMARY KEY,
                name TEXT NOT NULL UNIQUE,
                applied_at INTEGER NOT NULL DEFAULT (unixepoch())
            );
            ",
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;

        // M001: add must_change_password column (idempotent — ignore duplicate column error)
        match conn.execute(
            "ALTER TABLE users ADD COLUMN must_change_password INTEGER NOT NULL DEFAULT 0",
            [],
        ) {
            Ok(_) => {}
            Err(e) if e.to_string().contains("duplicate column name") => {}
            Err(e) => return Err(ApiError::Internal(e.to_string())),
        }

        Ok(())
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

    /// Look up a user by username. Returns `(id, pw_hash, role, must_change_password)`.
    ///
    /// # Errors
    /// Returns `ApiError::Unauthorized` if user not found.
    pub fn find_user(&self, username: &str) -> Result<(i64, String, Role, bool), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.query_row(
            "SELECT id, pw_hash, role, must_change_password FROM users WHERE username = ?1",
            params![username],
            |row| {
                let id: i64 = row.get(0)?;
                let pw_hash: String = row.get(1)?;
                let role_str: String = row.get(2)?;
                let mcp: i64 = row.get(3)?;
                Ok((id, pw_hash, role_str, mcp))
            },
        )
        .map_err(|_| ApiError::Unauthorized("invalid credentials"))
        .and_then(|(id, hash, role_str, mcp)| {
            let role = str_to_role(&role_str)?;
            Ok((id, hash, role, mcp != 0))
        })
    }

    /// Bootstrap the `soundtech` user with role ENGINEER.
    ///
    /// Idempotent: does nothing if the user already exists.
    /// `password` is hashed with Argon2id; plaintext is never stored or logged.
    ///
    /// # Errors
    /// Returns `ApiError::Internal` on hashing or DB failure.
    pub fn bootstrap_soundtech(&self, password: &str) -> Result<(), ApiError> {
        // Check existence without revealing the password in any error path.
        let exists = {
            let conn = self
                .conn
                .lock()
                .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
            conn.query_row(
                "SELECT COUNT(*) FROM users WHERE username = 'soundtech'",
                [],
                |row| row.get::<_, i64>(0),
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?
                > 0
        };
        if exists {
            return Ok(());
        }
        let pw_hash = hash_password(password)?;
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        match conn.execute(
            "INSERT INTO users (username, pw_hash, role, must_change_password) VALUES ('soundtech', ?1, 'ENGINEER', 1)",
            params![pw_hash],
        ) {
            Ok(_) => {}
            Err(e) if e.to_string().contains("UNIQUE") => {
                // TOCTOU race: another process inserted soundtech between our COUNT check and
                // this INSERT. Both paths result in soundtech existing — treat as idempotent.
            }
            Err(e) => return Err(ApiError::Internal(e.to_string())),
        }
        Ok(())
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

    /// Return newly created refresh-token session ID.
    pub fn store_refresh_token_with_id(
        &self,
        user_id: i64,
        token_hash: &str,
        expires_at: u64,
        family: &str,
    ) -> Result<i64, ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.execute("INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family) VALUES (?1, ?2, ?3, ?4)", params![user_id, token_hash, expires_at.cast_signed(), family]).map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok(conn.last_insert_rowid())
    }

    /// Return refresh session ID by token hash.
    pub fn refresh_token_session_id(&self, token_hash: &str) -> Result<i64, ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.query_row(
            "SELECT id FROM refresh_tokens WHERE token_hash = ?1",
            params![token_hash],
            |row| row.get(0),
        )
        .map_err(|_| ApiError::Unauthorized("refresh token not found"))
    }

    /// Persist access JWT mapping.
    pub fn store_access_session(
        &self,
        jti: &str,
        user_id: i64,
        session_id: i64,
        expires_at: u64,
    ) -> Result<(), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.execute("INSERT INTO access_sessions (jti, user_id, session_id, expires_at) VALUES (?1, ?2, ?3, ?4)", params![jti, user_id, session_id, expires_at.cast_signed()]).map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Atomically create refresh session and its access-token mapping.
    pub fn create_session_with_access(
        &self,
        user_id: i64,
        token_hash: &str,
        refresh_expires_at: u64,
        family: &str,
        jti: &str,
        access_expires_at: u64,
    ) -> Result<i64, ApiError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let tx = conn
            .transaction()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute("INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family) VALUES (?1, ?2, ?3, ?4)", params![user_id, token_hash, refresh_expires_at.cast_signed(), family]).map_err(|e| ApiError::Internal(e.to_string()))?;
        let session_id = tx.last_insert_rowid();
        tx.execute("INSERT INTO access_sessions (jti, user_id, session_id, expires_at) VALUES (?1, ?2, ?3, ?4)", params![jti, user_id, session_id, access_expires_at.cast_signed()]).map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.commit().map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok(session_id)
    }

    /// Remove newly-created login session when JWT signing fails.
    pub fn cleanup_session_after_signing_failure(
        &self,
        token_hash: &str,
        jti: &str,
    ) -> Result<(), ApiError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let tx = conn
            .transaction()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute("DELETE FROM access_sessions WHERE jti = ?1", params![jti])
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute(
            "DELETE FROM refresh_tokens WHERE token_hash = ?1",
            params![token_hash],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.commit().map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Restore old refresh token and remove replacement after JWT signing fails.
    pub fn discard_refresh_after_signing_failure(
        &self,
        new_token_hash: &str,
        jti: &str,
    ) -> Result<(), ApiError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let tx = conn
            .transaction()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute("DELETE FROM access_sessions WHERE jti = ?1", params![jti])
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute(
            "DELETE FROM refresh_tokens WHERE token_hash = ?1",
            params![new_token_hash],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        // Never reactivate old state here. A concurrent logout, replay
        // detection, admin revoke, or user deletion may have revoked it after
        // rotation; rollback must fail closed rather than resurrect access.
        tx.commit().map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Check access JWT mapping, user existence, expiry, and revocation.
    pub fn is_access_session_active(
        &self,
        jti: &str,
        user_id: i64,
        session_id: i64,
        now_unix: u64,
    ) -> Result<bool, ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.query_row("SELECT EXISTS(SELECT 1 FROM access_sessions a JOIN refresh_tokens r ON r.id = a.session_id JOIN users u ON u.id = a.user_id WHERE a.jti = ?1 AND a.user_id = ?2 AND a.session_id = ?3 AND a.revoked = 0 AND a.expires_at > ?4 AND r.revoked = 0 AND r.expires_at > ?4)", params![jti, user_id, session_id, now_unix.cast_signed()], |row| row.get(0)).map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Revoke access mappings associated with refresh sessions.
    pub fn revoke_access_for_user(&self, user_id: i64) -> Result<(), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.execute(
            "UPDATE access_sessions SET revoked = 1 WHERE user_id = ?1",
            params![user_id],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Revoke access mapping by JWT ID.
    pub fn revoke_access_jti(&self, jti: &str) -> Result<(), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.execute(
            "UPDATE access_sessions SET revoked = 1 WHERE jti = ?1",
            params![jti],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok(())
    }

    /// Look up refresh-token owner before rotation without mutating token state.
    ///
    /// This intentionally ignores revoked/expired state so rotation can handle
    /// replay detection and revoke the entire token family.
    pub fn refresh_token_owner(&self, token_hash: &str) -> Result<i64, ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.query_row(
            "SELECT user_id FROM refresh_tokens WHERE token_hash = ?1",
            params![token_hash],
            |row| row.get(0),
        )
        .map_err(|_| ApiError::Unauthorized("invalid refresh token"))
    }

    /// Revoke refresh-token family after replay detection.
    ///
    /// # Errors
    /// Returns `ApiError::Unauthorized` when token is unknown; database failures are internal errors.
    pub fn revoke_refresh_family(&self, token_hash: &str) -> Result<(), ApiError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let tx = conn
            .transaction()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let family: String = tx
            .query_row(
                "SELECT family FROM refresh_tokens WHERE token_hash = ?1",
                params![token_hash],
                |row| row.get(0),
            )
            .map_err(|_| ApiError::Unauthorized("invalid refresh token"))?;
        tx.execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE family = ?1",
            params![family],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute(
            "UPDATE access_sessions SET revoked = 1 WHERE session_id IN (SELECT id FROM refresh_tokens WHERE family = ?1)",
            params![family],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.commit().map_err(|e| ApiError::Internal(e.to_string()))
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
        new_token_hash: &str,
        new_expires_at: u64,
        now_unix: u64,
    ) -> Result<(i64, String), ApiError> {
        self.rotate_refresh_token_with_access(
            token_hash,
            new_token_hash,
            new_expires_at,
            now_unix,
            None,
            None,
        )
    }

    /// Rotate refresh token and persist access mapping in same transaction.
    pub fn rotate_refresh_token_with_access(
        &self,
        token_hash: &str,
        new_token_hash: &str,
        new_expires_at: u64,
        now_unix: u64,
        jti: Option<&str>,
        access_expires_at: Option<u64>,
    ) -> Result<(i64, String), ApiError> {
        self.rotate_refresh_token_with_access_id(
            token_hash,
            new_token_hash,
            new_expires_at,
            now_unix,
            jti,
            access_expires_at,
        )
        .map(|(user_id, family, _)| (user_id, family))
    }

    /// Rotate refresh token and return replacement session ID.
    pub fn rotate_refresh_token_with_access_id(
        &self,
        token_hash: &str,
        new_token_hash: &str,
        new_expires_at: u64,
        now_unix: u64,
        jti: Option<&str>,
        access_expires_at: Option<u64>,
    ) -> Result<(i64, String, i64), ApiError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let tx = conn
            .transaction()
            .map_err(|e| ApiError::Internal(e.to_string()))?;

        let result: rusqlite::Result<(i64, i64, i64, i64, String)> = tx.query_row(
            "SELECT id, user_id, revoked, expires_at, family FROM refresh_tokens WHERE token_hash = ?1",
            params![token_hash],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?, row.get(4)?)),
        );
        let (tok_id, user_id, revoked, expires, family) =
            result.map_err(|_| ApiError::Unauthorized("refresh token not found"))?;

        if revoked != 0 {
            tx.execute(
                "UPDATE refresh_tokens SET revoked = 1 WHERE family = ?1",
                params![family],
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;
            tx.execute(
                "UPDATE access_sessions SET revoked = 1 WHERE session_id IN (SELECT id FROM refresh_tokens WHERE family = ?1)",
                params![family],
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;
            tx.commit().map_err(|e| ApiError::Internal(e.to_string()))?;
            return Err(ApiError::Unauthorized(
                "refresh token reuse detected — family revoked",
            ));
        }
        if now_unix.cast_signed() >= expires {
            return Err(ApiError::Unauthorized("refresh token expired"));
        }

        tx.execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE id = ?1 AND revoked = 0",
            params![tok_id],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute(
            "UPDATE access_sessions SET revoked = 1 WHERE session_id = ?1",
            params![tok_id],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute(
            "INSERT INTO refresh_tokens (user_id, token_hash, expires_at, family) VALUES (?1, ?2, ?3, ?4)",
            params![user_id, new_token_hash, new_expires_at.cast_signed(), family],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        // Capture refresh_tokens.id before inserting access_sessions. SQLite's
        // last_insert_rowid() would otherwise return the mapping row ID.
        let new_session_id = tx.last_insert_rowid();
        if let (Some(jti), Some(access_expires_at)) = (jti, access_expires_at) {
            tx.execute(
                "INSERT INTO access_sessions (jti, user_id, session_id, expires_at) VALUES (?1, ?2, ?3, ?4)",
                params![jti, user_id, new_session_id, access_expires_at.cast_signed()],
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        }
        tx.commit().map_err(|e| ApiError::Internal(e.to_string()))?;
        Ok((user_id, family, new_session_id))
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

    /// Assign a mix slot to a user.
    ///
    /// # Errors
    /// Returns `ApiError::BadRequest` for an invalid mix slot or conflicting assignment.
    pub fn assign_mix(&self, mix_index: usize, user_id: i64) -> Result<(), ApiError> {
        if mix_index >= mix_engine::MAX_MIXES || user_id <= 0 {
            return Err(ApiError::BadRequest("invalid mix assignment".to_owned()));
        }
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let existing: Option<i64> = conn
            .query_row(
                "SELECT mix_index FROM mix_assignments WHERE user_id = ?1",
                params![user_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        if existing.is_some() {
            return Err(ApiError::BadRequest("user already owns a mix".to_owned()));
        }
        conn.execute(
            "INSERT INTO mix_assignments (mix_index, user_id) VALUES (?1, ?2)",
            params![mix_index as i64, user_id],
        )
        .map_err(|e| {
            let message = e.to_string();
            if message.contains("FOREIGN KEY") {
                ApiError::NotFound("user not found".to_owned())
            } else if message.contains("UNIQUE") || message.contains("PRIMARY KEY") {
                ApiError::BadRequest("mix already assigned".to_owned())
            } else {
                ApiError::Internal(message)
            }
        })?;
        Ok(())
    }

    /// Remove assignment from a mix slot.
    ///
    /// # Errors
    /// Returns `ApiError::NotFound` when no assignment exists.
    pub fn unassign_mix(&self, mix_index: usize) -> Result<(), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let count = conn
            .execute(
                "DELETE FROM mix_assignments WHERE mix_index = ?1",
                params![mix_index as i64],
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        if count == 0 {
            return Err(ApiError::NotFound("mix assignment not found".to_owned()));
        }
        Ok(())
    }

    /// List all mix assignments with usernames.
    pub fn list_mix_assignments(&self) -> Result<Vec<(usize, i64, String)>, ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let mut stmt = conn.prepare("SELECT m.mix_index, m.user_id, u.username FROM mix_assignments m JOIN users u ON u.id = m.user_id ORDER BY m.mix_index")
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, i64>(0)? as usize, row.get(1)?, row.get(2)?))
            })
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Return assigned user for a mix slot.
    pub fn get_mix_assignment(&self, mix_index: usize) -> Result<Option<i64>, ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.query_row(
            "SELECT user_id FROM mix_assignments WHERE mix_index = ?1",
            params![mix_index as i64],
            |row| row.get(0),
        )
        .optional()
        .map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Test-only fault injection for fail-closed authorization tests.
    ///
    /// Debug-only so destructive test plumbing cannot ship in release builds.
    #[cfg(debug_assertions)]
    #[doc(hidden)]
    pub fn drop_mix_assignments_table_for_test(&self) -> Result<(), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.execute_batch("DROP TABLE mix_assignments")
            .map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Return mix slot assigned to a user.
    pub fn get_user_assigned_mix(&self, user_id: i64) -> Result<Option<usize>, ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        conn.query_row(
            "SELECT mix_index FROM mix_assignments WHERE user_id = ?1",
            params![user_id],
            |row| row.get::<_, i64>(0),
        )
        .optional()
        .map(|v| v.map(|i| i as usize))
        .map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Revoke all refresh and access sessions for user atomically.
    pub fn revoke_all_for_user(&self, user_id: i64) -> Result<(), ApiError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let tx = conn
            .transaction()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute(
            "UPDATE refresh_tokens SET revoked = 1 WHERE user_id = ?1",
            params![user_id],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute(
            "UPDATE access_sessions SET revoked = 1 WHERE user_id = ?1",
            params![user_id],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.commit().map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Revoke all sessions and delete user atomically.
    pub fn delete_user_with_sessions(&self, user_id: i64) -> Result<(), ApiError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let tx = conn
            .transaction()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let affected = tx
            .execute("DELETE FROM users WHERE id = ?1", params![user_id])
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        if affected == 0 {
            return Err(ApiError::NotFound(format!("user {user_id} not found")));
        }
        tx.commit().map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// Revoke refresh and access session atomically.
    pub fn revoke_session_by_id(&self, session_id: i64) -> Result<(), ApiError> {
        let mut conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let tx = conn
            .transaction()
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let affected = tx
            .execute(
                "UPDATE refresh_tokens SET revoked = 1 WHERE id = ?1",
                params![session_id],
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        tx.execute(
            "UPDATE access_sessions SET revoked = 1 WHERE session_id = ?1",
            params![session_id],
        )
        .map_err(|e| ApiError::Internal(e.to_string()))?;
        if affected == 0 {
            return Err(ApiError::NotFound(format!(
                "session {session_id} not found"
            )));
        }
        tx.commit().map_err(|e| ApiError::Internal(e.to_string()))
    }

    /// List all users ordered by ID.
    ///
    /// # Errors
    /// Returns `ApiError::Internal` on DB error.
    pub fn list_users(&self) -> Result<Vec<(i64, String, Role)>, ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let mut stmt = conn
            .prepare("SELECT id, username, role FROM users ORDER BY id")
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map([], |row| {
                let id: i64 = row.get(0)?;
                let username: String = row.get(1)?;
                let role_str: String = row.get(2)?;
                Ok((id, username, role_str))
            })
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let mut users = Vec::new();
        for row in rows {
            let (id, username, role_str) = row.map_err(|e| ApiError::Internal(e.to_string()))?;
            let role = str_to_role(&role_str)?;
            users.push((id, username, role));
        }
        Ok(users)
    }

    /// Delete a user by ID.
    ///
    /// # Errors
    /// Returns `ApiError::NotFound` if no user with that ID exists.
    /// Returns `ApiError::Internal` on DB error.
    pub fn delete_user(&self, user_id: i64) -> Result<(), ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let affected = conn
            .execute("DELETE FROM users WHERE id = ?1", params![user_id])
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        if affected == 0 {
            return Err(ApiError::NotFound(format!("user {user_id} not found")));
        }
        Ok(())
    }

    /// List active (non-revoked, non-expired) refresh token sessions.
    ///
    /// # Errors
    /// Returns `ApiError::Internal` on DB error.
    pub fn list_active_sessions(&self, now_unix: u64) -> Result<Vec<(i64, i64, u64)>, ApiError> {
        let conn = self
            .conn
            .lock()
            .map_err(|_| ApiError::Internal("db lock poisoned".to_owned()))?;
        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, expires_at FROM refresh_tokens \
                 WHERE revoked = 0 AND expires_at > ?1 ORDER BY id",
            )
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let rows = stmt
            .query_map(params![now_unix.cast_signed()], |row| {
                let id: i64 = row.get(0)?;
                let user_id: i64 = row.get(1)?;
                let expires_at: i64 = row.get(2)?;
                Ok((id, user_id, expires_at))
            })
            .map_err(|e| ApiError::Internal(e.to_string()))?;
        let mut sessions = Vec::new();
        for row in rows {
            let (id, user_id, expires_at) = row.map_err(|e| ApiError::Internal(e.to_string()))?;
            sessions.push((id, user_id, expires_at.cast_unsigned()));
        }
        Ok(sessions)
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
        let (id, hash, role, _) = db.find_user("alice").unwrap();
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
        let (user_id, _, _, _) = db.find_user("carol").unwrap();
        let now = 1_000_000_u64;
        db.store_refresh_token(user_id, "tokenhash_a", now + 1000, "fam-1")
            .unwrap();
        let (uid, fam) = db
            .rotate_refresh_token("tokenhash_a", "tokenhash_a_new", now + 1000, now)
            .unwrap();
        assert_eq!(uid, user_id);
        assert_eq!(fam, "fam-1");
    }

    #[test]
    fn refresh_token_reuse_revokes_family_and_access() {
        let db = setup();
        db.create_user("dave", "hash", Role::Musician).unwrap();
        let (user_id, _, _, _) = db.find_user("dave").unwrap();
        let now = 1_000_000_u64;
        let old_session_id = db
            .create_session_with_access(
                user_id,
                "tokenhash_b",
                now + 1000,
                "fam-2",
                "access-jti-old",
                now + 1000,
            )
            .unwrap();
        let (_, _, new_session_id) = db
            .rotate_refresh_token_with_access_id(
                "tokenhash_b",
                "tokenhash_b_new",
                now + 1000,
                now,
                Some("access-jti-new"),
                Some(now + 1000),
            )
            .unwrap();
        assert!(!db
            .is_access_session_active("access-jti-old", user_id, old_session_id, now)
            .unwrap());
        assert!(db
            .is_access_session_active("access-jti-new", user_id, new_session_id, now)
            .unwrap());

        // Reuse — should fail, revoke family, and invalidate all linked access.
        let err = db
            .rotate_refresh_token("tokenhash_b", "unused", now + 1000, now)
            .unwrap_err();
        assert!(matches!(err, ApiError::Unauthorized(_)));
        assert!(!db
            .is_access_session_active("access-jti-new", user_id, new_session_id, now)
            .unwrap());
    }

    #[test]
    fn revoke_all_for_user_makes_tokens_invalid() {
        let db = setup();
        db.create_user("eve", "hash", Role::Admin).unwrap();
        let (user_id, _, _, _) = db.find_user("eve").unwrap();
        let now = 1_000_000_u64;
        db.store_refresh_token(user_id, "tokenhash_c", now + 1000, "fam-3")
            .unwrap();
        db.revoke_all_for_user(user_id).unwrap();
        let err = db
            .rotate_refresh_token("tokenhash_c", "unused", now + 1000, now)
            .unwrap_err();
        assert!(matches!(err, ApiError::Unauthorized(_)));
    }

    #[test]
    fn list_users_empty_and_populated() {
        let db = setup();
        let users = db.list_users().unwrap();
        assert_eq!(users.len(), 0);
        db.create_user("alice", "hash1", Role::Admin).unwrap();
        db.create_user("bob", "hash2", Role::Engineer).unwrap();
        let users = db.list_users().unwrap();
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].1, "alice");
        assert_eq!(users[1].1, "bob");
    }

    #[test]
    fn delete_user_ok_and_not_found() {
        let db = setup();
        db.create_user("carol", "hash", Role::Musician).unwrap();
        let (user_id, _, _, _) = db.find_user("carol").unwrap();
        db.delete_user(user_id).unwrap();
        let err = db.delete_user(user_id).unwrap_err();
        assert!(matches!(err, ApiError::NotFound(_)));
    }

    #[test]
    fn list_and_revoke_sessions() {
        let db = setup();
        db.create_user("dave", "hash", Role::Musician).unwrap();
        let (user_id, _, _, _) = db.find_user("dave").unwrap();
        let now = 1_000_000_u64;
        db.store_refresh_token(user_id, "tokenhash_s1", now + 1000, "fam-s1")
            .unwrap();
        let sessions = db.list_active_sessions(now).unwrap();
        assert_eq!(sessions.len(), 1);
        let (session_id, sid_user, _) = sessions[0];
        assert_eq!(sid_user, user_id);
        db.revoke_session_by_id(session_id).unwrap();
        let sessions = db.list_active_sessions(now).unwrap();
        assert_eq!(sessions.len(), 0);
    }

    #[test]
    fn bootstrap_creates_soundtech() {
        let db = setup();
        db.bootstrap_soundtech("testpassword123").unwrap();
        let (_, _, role, must_change) = db.find_user("soundtech").unwrap();
        assert_eq!(role, Role::Engineer);
        assert!(
            must_change,
            "must_change_password must be true for bootstrapped soundtech"
        );
    }

    #[test]
    fn bootstrap_is_idempotent() {
        let db = setup();
        db.bootstrap_soundtech("first-password").unwrap();
        let (_, first_hash, _, _) = db.find_user("soundtech").unwrap();
        // Second call with different password must succeed and NOT overwrite
        db.bootstrap_soundtech("second-password").unwrap();
        let (_, second_hash, _, _) = db.find_user("soundtech").unwrap();
        assert_eq!(
            first_hash, second_hash,
            "hash must not change on second bootstrap call"
        );
    }

    #[test]
    fn bootstrap_does_not_overwrite_existing_user() {
        let db = setup();
        let known_hash = "$argon2id$v=19$m=19456,t=2,p=1$known$salt";
        db.create_user("soundtech", known_hash, Role::Engineer)
            .unwrap();
        // bootstrap with different password must not touch existing user
        db.bootstrap_soundtech("other-password").unwrap();
        let (_, hash, _, _) = db.find_user("soundtech").unwrap();
        assert_eq!(
            hash, known_hash,
            "existing user hash must not be overwritten"
        );
    }

    #[test]
    fn must_change_password_false_for_regular_users() {
        let db = setup();
        db.create_user(
            "alice",
            "$argon2id$v=19$m=19456,t=2,p=1$fakesalt$fakehash",
            Role::Engineer,
        )
        .unwrap();
        let (_, _, _, must_change) = db.find_user("alice").unwrap();
        assert!(
            !must_change,
            "regular users must have must_change_password = false"
        );
    }
}
