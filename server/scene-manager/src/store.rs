//! SQLite-backed scene store with immutable revision history.

use crate::{Scene, SceneConfig, SceneError, SCHEMA_VERSION};
use rusqlite::{params, Connection, OptionalExtension};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| i64::try_from(d.as_secs()).unwrap_or(i64::MAX))
}

/// Summary of a stored scene without full config payload.
#[derive(Debug, Clone)]
pub struct SceneSummary {
    /// Stable scene ID.
    pub id: String,
    /// Display name.
    pub name: String,
    /// Current active revision.
    pub active_revision: u64,
    /// Unix timestamp of creation.
    pub created_at: i64,
    /// Unix timestamp of last update.
    pub updated_at: i64,
}

/// Store errors.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// Underlying SQLite error.
    #[error("db error: {0}")]
    Db(#[from] rusqlite::Error),
    /// Mutex was poisoned.
    #[error("lock poisoned")]
    LockPoisoned,
    /// Scene validation failed.
    #[error("validation error: {0}")]
    Validation(#[from] SceneError),
    /// Scene not found.
    #[error("scene not found: {0}")]
    NotFound(String),
    /// Scene is the active scene; cannot delete.
    #[error("scene is active")]
    IsActive,
    /// Stored payload could not be decoded.
    #[error("corrupt payload: {0}")]
    CorruptPayload(String),
}

/// SQLite-backed scene store.
pub struct SceneStore {
    conn: Arc<Mutex<Connection>>,
}

impl SceneStore {
    fn new(conn: Connection) -> Result<Self, StoreError> {
        let store = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        store.migrate()?;
        Ok(store)
    }

    /// Open or create a file-backed store.
    ///
    /// # Errors
    /// Returns `StoreError::Db` on connection or migration failure.
    pub fn open(path: &str) -> Result<Self, StoreError> {
        let conn = Connection::open(path)?;
        Self::new(conn)
    }

    /// Open a transient in-memory store.
    ///
    /// # Errors
    /// Returns `StoreError::Db` on connection or migration failure.
    pub fn open_in_memory() -> Result<Self, StoreError> {
        let conn = Connection::open_in_memory()?;
        Self::new(conn)
    }

    fn migrate(&self) -> Result<(), StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        conn.execute_batch("PRAGMA journal_mode=WAL;")?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS scenes (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                active_revision INTEGER NOT NULL DEFAULT 0,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            );
            CREATE TABLE IF NOT EXISTS scene_revisions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                scene_id TEXT NOT NULL REFERENCES scenes(id) ON DELETE CASCADE,
                revision INTEGER NOT NULL,
                payload TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                UNIQUE(scene_id, revision)
            );
            CREATE TABLE IF NOT EXISTS active_scene (
                key TEXT PRIMARY KEY DEFAULT 'active',
                scene_id TEXT REFERENCES scenes(id)
            );",
        )?;
        Ok(())
    }

    /// List all scenes ordered by creation time.
    ///
    /// # Errors
    /// Returns `StoreError::Db` or `StoreError::LockPoisoned`.
    pub fn list_scenes(&self) -> Result<Vec<SceneSummary>, StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        let mut stmt = conn.prepare(
            "SELECT id, name, active_revision, created_at, updated_at FROM scenes ORDER BY created_at ASC",
        )?;
        let rows = stmt.query_map([], |row| {
            let rev: i64 = row.get(2)?;
            Ok(SceneSummary {
                id: row.get(0)?,
                name: row.get(1)?,
                active_revision: u64::try_from(rev).unwrap_or(0),
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        let mut out = Vec::new();
        for r in rows {
            out.push(r?);
        }
        Ok(out)
    }

    /// Create a new scene with revision 1.
    ///
    /// # Errors
    /// Returns `StoreError::Validation` if the name or config is invalid,
    /// or `StoreError::Db` / `StoreError::LockPoisoned` on persistence failure.
    pub fn create_scene(&self, name: &str, config: SceneConfig) -> Result<Scene, StoreError> {
        let name_bytes = name.len();
        if name_bytes == 0 || name_bytes > crate::MAX_SCENE_NAME_BYTES {
            return Err(StoreError::Validation(SceneError::InvalidName {
                max: crate::MAX_SCENE_NAME_BYTES,
            }));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let scene = Scene {
            id: id.clone(),
            name: name.to_owned(),
            schema_version: SCHEMA_VERSION,
            revision: 1,
            config,
        };
        crate::validate(&scene)?;
        let payload = crate::encode(&scene)?;
        let now = now_secs();

        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO scenes (id, name, active_revision, created_at, updated_at) VALUES (?1, ?2, 1, ?3, ?3)",
            params![id, scene.name, now],
        )?;
        tx.execute(
            "INSERT INTO scene_revisions (scene_id, revision, payload, created_at) VALUES (?1, 1, ?2, ?3)",
            params![id, payload, now],
        )?;
        tx.commit()?;
        Ok(scene)
    }

    /// Save a new immutable revision for an existing scene.
    ///
    /// # Errors
    /// Returns `StoreError::NotFound` if `id` does not exist,
    /// `StoreError::Validation` if config is invalid,
    /// or `StoreError::Db` / `StoreError::LockPoisoned` on failure.
    pub fn save_scene(&self, id: &str, config: SceneConfig) -> Result<Scene, StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let (name, _cur_rev): (String, i64) = conn
            .query_row(
                "SELECT name, active_revision FROM scenes WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(id.to_owned()),
                other => StoreError::Db(other),
            })?;

        let max_rev: i64 = conn.query_row(
            "SELECT COALESCE(MAX(revision), 0) FROM scene_revisions WHERE scene_id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        let new_rev = max_rev + 1;

        let scene = Scene {
            id: id.to_owned(),
            name: name.clone(),
            schema_version: SCHEMA_VERSION,
            revision: u64::try_from(new_rev).unwrap_or(u64::MAX),
            config,
        };
        crate::validate(&scene)?;
        let payload = crate::encode(&scene)?;
        let now = now_secs();

        let tx = conn.unchecked_transaction()?;
        tx.execute(
            "INSERT INTO scene_revisions (scene_id, revision, payload, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![id, new_rev, payload, now],
        )?;
        tx.execute(
            "UPDATE scenes SET active_revision = ?1, updated_at = ?2 WHERE id = ?3",
            params![new_rev, now, id],
        )?;
        tx.commit()?;
        Ok(scene)
    }

    /// Fetch a scene by ID (latest active revision).
    ///
    /// # Errors
    /// Returns `StoreError::NotFound` if the scene does not exist,
    /// `StoreError::CorruptPayload` if stored JSON is invalid,
    /// or `StoreError::Db` / `StoreError::LockPoisoned` on failure.
    pub fn get_scene(&self, id: &str) -> Result<Scene, StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        let payload: String = conn
            .query_row(
                "SELECT sr.payload FROM scene_revisions sr
                 JOIN scenes s ON s.id = sr.scene_id
                 WHERE sr.scene_id = ?1 AND sr.revision = s.active_revision",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(id.to_owned()),
                other => StoreError::Db(other),
            })?;

        crate::decode(&payload).map_err(|e| StoreError::CorruptPayload(e.to_string()))
    }

    /// Delete a scene. Fails if it is currently the active scene.
    ///
    /// # Errors
    /// Returns `StoreError::IsActive` if the scene is active,
    /// `StoreError::NotFound` if it does not exist,
    /// or `StoreError::Db` / `StoreError::LockPoisoned` on failure.
    pub fn delete_scene(&self, id: &str) -> Result<(), StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let active: Option<String> = conn
            .query_row(
                "SELECT scene_id FROM active_scene WHERE key = 'active'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        if let Some(ref active_id) = active {
            if active_id == id {
                return Err(StoreError::IsActive);
            }
        }

        let affected = conn.execute("DELETE FROM scenes WHERE id = ?1", params![id])?;
        if affected == 0 {
            return Err(StoreError::NotFound(id.to_owned()));
        }
        Ok(())
    }

    /// Set or clear the active scene.
    ///
    /// # Errors
    /// Returns `StoreError::NotFound` if `Some(id)` does not exist,
    /// or `StoreError::Db` / `StoreError::LockPoisoned` on failure.
    pub fn set_active_scene(&self, id: Option<&str>) -> Result<(), StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        match id {
            None => {
                conn.execute("DELETE FROM active_scene WHERE key = 'active'", [])?;
            }
            Some(scene_id) => {
                let exists: bool = conn
                    .query_row(
                        "SELECT 1 FROM scenes WHERE id = ?1",
                        params![scene_id],
                        |_| Ok(true),
                    )
                    .optional()?
                    .unwrap_or(false);
                if !exists {
                    return Err(StoreError::NotFound(scene_id.to_owned()));
                }
                conn.execute(
                    "INSERT OR REPLACE INTO active_scene (key, scene_id) VALUES ('active', ?1)",
                    params![scene_id],
                )?;
            }
        }
        Ok(())
    }

    /// Get the currently active scene, if any.
    ///
    /// # Errors
    /// Returns `StoreError::CorruptPayload` if stored JSON is invalid,
    /// or `StoreError::Db` / `StoreError::LockPoisoned` on failure.
    pub fn get_active_scene(&self) -> Result<Option<Scene>, StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        let result = conn
            .query_row(
                "SELECT sr.payload FROM active_scene a
                 JOIN scenes s ON s.id = a.scene_id
                 JOIN scene_revisions sr ON sr.scene_id = s.id AND sr.revision = s.active_revision
                 WHERE a.key = 'active'",
                [],
                |row| row.get::<_, String>(0),
            )
            .optional()?;

        match result {
            None => Ok(None),
            Some(payload) => {
                let scene = crate::decode(&payload)
                    .map_err(|e| StoreError::CorruptPayload(e.to_string()))?;
                Ok(Some(scene))
            }
        }
    }

    /// Test helper: corrupt the latest revision payload for a scene.
    ///
    /// # Panics
    /// Panics if the lock is poisoned or the update fails.
    #[cfg(test)]
    pub fn corrupt_for_test(&self, scene_id: &str) {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE scene_revisions SET payload='bad json' WHERE scene_id=?",
            [scene_id],
        )
        .unwrap();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::SceneConfig;

    fn empty_config() -> SceneConfig {
        SceneConfig {
            channels: vec![],
            mixes: vec![],
        }
    }

    #[test]
    fn fresh_migration_idempotent() {
        let store = SceneStore::open_in_memory().unwrap();
        store.migrate().unwrap();
    }

    #[test]
    fn create_and_get_round_trip() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("MyShow", empty_config()).unwrap();
        assert_eq!(scene.name, "MyShow");
        assert_eq!(scene.revision, 1);
        let fetched = store.get_scene(&scene.id).unwrap();
        assert_eq!(fetched.name, "MyShow");
        assert_eq!(fetched.revision, 1);
    }

    #[test]
    fn save_creates_new_revision() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        let s2 = store.save_scene(&scene.id, empty_config()).unwrap();
        assert_eq!(s2.revision, 2);
        let s3 = store.save_scene(&scene.id, empty_config()).unwrap();
        assert_eq!(s3.revision, 3);
    }

    #[test]
    fn prior_revision_preserved_in_db() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        store.save_scene(&scene.id, empty_config()).unwrap();
        store.save_scene(&scene.id, empty_config()).unwrap();
        let conn = store.conn.lock().unwrap();
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM scene_revisions WHERE scene_id = ?1",
                params![scene.id],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(count, 3);
    }

    #[test]
    fn delete_non_active_ok() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        store.delete_scene(&scene.id).unwrap();
        let err = store.get_scene(&scene.id).unwrap_err();
        assert!(matches!(err, StoreError::NotFound(_)));
    }

    #[test]
    fn delete_active_fails() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        store.set_active_scene(Some(&scene.id)).unwrap();
        let err = store.delete_scene(&scene.id).unwrap_err();
        assert!(matches!(err, StoreError::IsActive));
    }

    #[test]
    fn get_nonexistent_not_found() {
        let store = SceneStore::open_in_memory().unwrap();
        let err = store.get_scene("no-such-id").unwrap_err();
        assert!(matches!(err, StoreError::NotFound(_)));
    }

    #[test]
    fn list_scenes_empty_then_populated() {
        let store = SceneStore::open_in_memory().unwrap();
        let list = store.list_scenes().unwrap();
        assert_eq!(list.len(), 0);
        store.create_scene("Show", empty_config()).unwrap();
        let list = store.list_scenes().unwrap();
        assert_eq!(list.len(), 1);
    }

    #[test]
    fn set_and_get_active_scene() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        store.set_active_scene(Some(&scene.id)).unwrap();
        let active = store.get_active_scene().unwrap();
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, scene.id);
    }

    #[test]
    fn file_store_persists_across_reopen() {
        let dir = std::env::temp_dir().join(format!("open_iem_scene_{}", uuid::Uuid::new_v4()));
        std::fs::create_dir(&dir).unwrap();
        let path = dir.join("scenes.db");
        let path_str = path.to_str().unwrap().to_owned();
        let scene = {
            let store = SceneStore::open(&path_str).unwrap();
            store
                .create_scene("Persistent show", empty_config())
                .unwrap()
        };

        {
            let reopened = SceneStore::open(&path_str).unwrap();
            let fetched = reopened.get_scene(&scene.id).unwrap();
            assert_eq!(fetched.id, scene.id);
            assert_eq!(fetched.name, "Persistent show");
            assert_eq!(fetched.revision, 1);
            assert_eq!(reopened.list_scenes().unwrap().len(), 1);
        }

        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn corrupt_payload_error() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        store.corrupt_for_test(&scene.id);
        let err = store.get_scene(&scene.id).unwrap_err();
        assert!(matches!(err, StoreError::CorruptPayload(_)));
    }
}
