//! SQLite-backed scene store with immutable revision history.

use crate::{Scene, SceneConfig, SceneError, SceneStoreSnapshot, SCHEMA_VERSION};
use rusqlite::{params, Connection, OptionalExtension};
use std::collections::HashSet;
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
    /// Snapshot export failed validation.
    #[error("invalid scene snapshot: {0}")]
    InvalidSnapshot(String),
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

    fn load_active_revision(conn: &Connection, scene_id: &str) -> Result<Scene, StoreError> {
        let (active_revision, payload): (i64, Option<String>) = conn
            .query_row(
                "SELECT s.active_revision, sr.payload FROM scenes s LEFT JOIN scene_revisions sr ON sr.scene_id = s.id AND sr.revision = s.active_revision WHERE s.id = ?1",
                params![scene_id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(scene_id.to_owned()),
                other => StoreError::Db(other),
            })?;
        if active_revision <= 0 {
            return Err(StoreError::InvalidSnapshot(
                "active scene revision must be positive".to_owned(),
            ));
        }
        let payload = payload.ok_or_else(|| {
            StoreError::InvalidSnapshot("active scene revision has no persisted payload".to_owned())
        })?;
        let scene =
            crate::decode(&payload).map_err(|e| StoreError::CorruptPayload(e.to_string()))?;
        if scene.id != scene_id {
            return Err(StoreError::InvalidSnapshot(
                "active scene payload ID does not match scene ID".to_owned(),
            ));
        }
        if scene.revision
            != u64::try_from(active_revision).map_err(|_| {
                StoreError::InvalidSnapshot(
                    "active scene revision exceeds unsigned range".to_owned(),
                )
            })?
        {
            return Err(StoreError::InvalidSnapshot(
                "active scene payload revision does not match active revision".to_owned(),
            ));
        }
        Ok(scene)
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
                active_revision: u64::try_from(rev).map_err(|_| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Integer,
                        Box::new(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "negative active revision",
                        )),
                    )
                })?,
                created_at: row.get(3)?,
                updated_at: row.get(4)?,
            })
        })?;
        let summaries: Result<Vec<_>, _> = rows.collect();
        let summaries = summaries?;
        drop(stmt);
        for summary in &summaries {
            Self::load_active_revision(&conn, &summary.id)?;
        }
        Ok(summaries)
    }

    /// Export current durable scenes and active pointer as bounded JSON.
    /// Transient runtime state is not represented in the snapshot.
    ///
    /// # Errors
    /// Returns an error when the store is unavailable or contains corrupt data.
    pub fn export_snapshot(&self) -> Result<String, StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        let tx = conn.unchecked_transaction()?;
        let mut stmt = tx.prepare("SELECT s.id FROM scenes s ORDER BY s.created_at ASC")?;
        let rows = stmt.query_map([], |row| row.get::<_, String>(0))?;
        let scene_ids: Result<Vec<_>, _> = rows.collect();
        let scene_ids = scene_ids?;
        drop(stmt);
        let mut scenes = Vec::new();
        let mut estimated_bytes = 32usize;
        for scene_id in scene_ids {
            let scene = Self::load_active_revision(&tx, &scene_id)?;
            let scene_bytes = serde_json::to_vec(&scene)
                .map_err(|e| StoreError::InvalidSnapshot(e.to_string()))?
                .len();
            let scene_bytes_with_separator = scene_bytes
                .checked_add(1)
                .ok_or(StoreError::Validation(SceneError::PayloadTooLarge))?;
            estimated_bytes = estimated_bytes
                .checked_add(scene_bytes_with_separator)
                .ok_or(StoreError::Validation(SceneError::PayloadTooLarge))?;
            if estimated_bytes > crate::MAX_PAYLOAD_BYTES {
                return Err(StoreError::Validation(SceneError::PayloadTooLarge));
            }
            scenes.push(scene);
        }
        let active_scene_id: Option<String> = tx
            .query_row(
                "SELECT scene_id FROM active_scene WHERE key = 'active'",
                [],
                |row| row.get(0),
            )
            .optional()?;
        if let Some(ref active_id) = active_scene_id {
            if active_id.is_empty() || active_id.len() > crate::MAX_TEXT_BYTES {
                return Err(StoreError::InvalidSnapshot(
                    "active scene ID exceeds bounds".to_owned(),
                ));
            }
            let exported_ids: HashSet<&str> =
                scenes.iter().map(|scene| scene.id.as_str()).collect();
            if !exported_ids.contains(active_id.as_str()) {
                return Err(StoreError::InvalidSnapshot(
                    "active scene ID does not reference an exported scene".to_owned(),
                ));
            }
        }
        let json = serde_json::to_string(&SceneStoreSnapshot {
            version: SCHEMA_VERSION,
            scenes,
            active_scene_id,
        })
        .map_err(|e| StoreError::InvalidSnapshot(e.to_string()))?;
        if json.len() > crate::MAX_PAYLOAD_BYTES {
            return Err(StoreError::Validation(SceneError::PayloadTooLarge));
        }
        tx.commit()?;
        Ok(json)
    }

    /// Replace durable scenes and active pointer from a versioned JSON snapshot.
    ///
    /// Validation runs before transaction mutation. Replacement is atomic.
    ///
    /// # Errors
    /// Returns an error for malformed, unsupported, or inconsistent input.
    pub fn restore_snapshot(&self, json: &str) -> Result<(), StoreError> {
        if json.len() > crate::MAX_PAYLOAD_BYTES {
            return Err(StoreError::Validation(SceneError::PayloadTooLarge));
        }
        let snapshot: SceneStoreSnapshot =
            serde_json::from_str(json).map_err(|e| StoreError::InvalidSnapshot(e.to_string()))?;
        if snapshot.version != SCHEMA_VERSION {
            return Err(StoreError::InvalidSnapshot(format!(
                "unsupported snapshot version {}",
                snapshot.version
            )));
        }
        let mut ids = HashSet::new();
        for scene in &snapshot.scenes {
            if !ids.insert(scene.id.as_str()) {
                return Err(StoreError::InvalidSnapshot("duplicate scene ID".to_owned()));
            }
            crate::validate(scene).map_err(|e| StoreError::InvalidSnapshot(e.to_string()))?;
        }
        if let Some(active_id) = snapshot.active_scene_id.as_deref() {
            if !ids.contains(active_id) {
                return Err(StoreError::InvalidSnapshot(
                    "active scene ID does not reference a scene".to_owned(),
                ));
            }
        }
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        let tx = conn.unchecked_transaction()?;
        tx.execute("DELETE FROM active_scene WHERE key = 'active'", [])?;
        tx.execute(
            "DELETE FROM scene_revisions WHERE scene_id IN (SELECT id FROM scenes)",
            [],
        )?;
        tx.execute("DELETE FROM scenes WHERE 1 = 1", [])?;
        let now = now_secs();
        for scene in &snapshot.scenes {
            let revision = i64::try_from(scene.revision).map_err(|_| {
                StoreError::InvalidSnapshot("revision exceeds SQLite range".to_owned())
            })?;
            let payload =
                crate::encode(scene).map_err(|e| StoreError::InvalidSnapshot(e.to_string()))?;
            tx.execute("INSERT INTO scenes (id, name, active_revision, created_at, updated_at) VALUES (?1, ?2, ?3, ?4, ?4)", params![scene.id, scene.name, revision, now])?;
            tx.execute("INSERT INTO scene_revisions (scene_id, revision, payload, created_at) VALUES (?1, ?2, ?3, ?4)", params![scene.id, revision, payload, now])?;
        }
        if let Some(active_id) = snapshot.active_scene_id {
            tx.execute(
                "INSERT INTO active_scene (key, scene_id) VALUES ('active', ?1)",
                params![active_id],
            )?;
        }
        tx.commit()?;
        Ok(())
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

    /// Duplicate current revision of existing scene as new scene.
    /// Source scene and active-scene pointer remain unchanged.
    ///
    /// # Errors
    ///
    /// Returns an error when the source is missing or corrupt, the name is
    /// invalid, or the database transaction cannot be completed.
    pub fn duplicate_scene(&self, id: &str, name: &str) -> Result<Scene, StoreError> {
        if name.is_empty() || name.len() > crate::MAX_SCENE_NAME_BYTES {
            return Err(StoreError::Validation(SceneError::InvalidName {
                max: crate::MAX_SCENE_NAME_BYTES,
            }));
        }
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;
        let tx = conn.unchecked_transaction()?;
        let source = Self::load_active_revision(&tx, id)?;
        let copy = Scene {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_owned(),
            schema_version: SCHEMA_VERSION,
            revision: 1,
            config: source.config,
        };
        let encoded = crate::encode(&copy)?;
        let now = now_secs();
        tx.execute(
            "INSERT INTO scenes (id, name, active_revision, created_at, updated_at) VALUES (?1, ?2, 1, ?3, ?3)",
            params![copy.id, copy.name, now],
        )?;
        tx.execute(
            "INSERT INTO scene_revisions (scene_id, revision, payload, created_at) VALUES (?1, 1, ?2, ?3)",
            params![copy.id, encoded, now],
        )?;
        tx.commit()?;
        Ok(copy)
    }

    /// Save a new immutable revision for an existing scene.
    ///
    /// # Errors
    /// Returns `StoreError::NotFound` if `id` does not exist,
    /// `StoreError::Validation` if config is invalid,
    /// or `StoreError::InvalidSnapshot` when stored revision data is invalid,
    /// or `StoreError::Db` / `StoreError::LockPoisoned` on failure.
    pub fn save_scene(&self, id: &str, config: SceneConfig) -> Result<Scene, StoreError> {
        let conn = self.conn.lock().map_err(|_| StoreError::LockPoisoned)?;
        conn.execute_batch("PRAGMA foreign_keys=ON;")?;

        let (name, cur_rev): (String, i64) = conn
            .query_row(
                "SELECT name, active_revision FROM scenes WHERE id = ?1",
                params![id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => StoreError::NotFound(id.to_owned()),
                other => StoreError::Db(other),
            })?;
        let active = Self::load_active_revision(&conn, id)?;
        if active.revision
            != u64::try_from(cur_rev).map_err(|_| {
                StoreError::InvalidSnapshot(
                    "active scene revision exceeds unsigned range".to_owned(),
                )
            })?
        {
            return Err(StoreError::InvalidSnapshot(
                "active scene revision does not match loaded revision".to_owned(),
            ));
        }

        let max_rev: i64 = conn.query_row(
            "SELECT COALESCE(MAX(revision), 0) FROM scene_revisions WHERE scene_id = ?1",
            params![id],
            |row| row.get(0),
        )?;
        let new_rev = max_rev.checked_add(1).ok_or_else(|| {
            StoreError::InvalidSnapshot("scene revision exceeds SQLite range".to_owned())
        })?;

        let scene = Scene {
            id: id.to_owned(),
            name: name.clone(),
            schema_version: SCHEMA_VERSION,
            revision: u64::try_from(new_rev).map_err(|_| {
                StoreError::InvalidSnapshot("scene revision exceeds unsigned range".to_owned())
            })?,
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
        Self::load_active_revision(&conn, id)
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

        Self::load_active_revision(&conn, id)?;
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
                Self::load_active_revision(&conn, scene_id)?;
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
        let active_scene_id: Option<String> = conn
            .query_row(
                "SELECT scene_id FROM active_scene WHERE key = 'active'",
                [],
                |row| row.get(0),
            )
            .optional()?;

        match active_scene_id {
            None => Ok(None),
            Some(scene_id) => Self::load_active_revision(&conn, &scene_id).map(Some),
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
    fn duplicate_copies_config_with_new_id_and_does_not_activate() {
        let store = SceneStore::open_in_memory().unwrap();
        let source = store.create_scene("Source", empty_config()).unwrap();
        store.set_active_scene(Some(&source.id)).unwrap();
        let copy = store.duplicate_scene(&source.id, "Copy").unwrap();
        assert_ne!(copy.id, source.id);
        assert_eq!(copy.revision, 1);
        assert_eq!(copy.config, source.config);
        assert_eq!(store.get_active_scene().unwrap().unwrap().id, source.id);
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
    fn invalid_active_revision_fails_closed_for_reads_and_writes() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        let conn = store.conn.lock().unwrap();
        conn.execute(
            "UPDATE scenes SET active_revision = 0 WHERE id = ?1",
            params![scene.id],
        )
        .unwrap();
        drop(conn);

        assert!(matches!(
            store.list_scenes(),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.duplicate_scene(&scene.id, "Copy"),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.save_scene(&scene.id, empty_config()),
            Err(StoreError::InvalidSnapshot(_))
        ));

        let conn = store.conn.lock().unwrap();
        conn.execute(
            "UPDATE scenes SET active_revision = 2 WHERE id = ?1",
            params![scene.id],
        )
        .unwrap();
        drop(conn);
        assert!(matches!(
            store.save_scene(&scene.id, empty_config()),
            Err(StoreError::InvalidSnapshot(_))
        ));
    }

    #[test]
    fn active_payload_validation_fails_closed_for_invalid_payload() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        store.corrupt_for_test(&scene.id);

        assert!(matches!(
            store.list_scenes(),
            Err(StoreError::CorruptPayload(_))
        ));
        assert!(matches!(
            store.duplicate_scene(&scene.id, "Copy"),
            Err(StoreError::CorruptPayload(_))
        ));
        assert!(matches!(
            store.save_scene(&scene.id, empty_config()),
            Err(StoreError::CorruptPayload(_))
        ));
        assert!(matches!(
            store.set_active_scene(Some(&scene.id)),
            Err(StoreError::CorruptPayload(_))
        ));
        assert!(matches!(
            store.delete_scene(&scene.id),
            Err(StoreError::CorruptPayload(_))
        ));
    }

    #[test]
    fn active_payload_validation_fails_closed_for_mismatched_identity() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        let mut payload_scene = scene.clone();
        payload_scene.id = "different-id".to_owned();
        let payload = crate::encode(&payload_scene).unwrap();
        let conn = store.conn.lock().unwrap();
        conn.execute(
            "UPDATE scene_revisions SET payload = ?1 WHERE scene_id = ?2 AND revision = 1",
            params![payload, scene.id],
        )
        .unwrap();
        drop(conn);

        assert!(matches!(
            store.list_scenes(),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.duplicate_scene(&scene.id, "Copy"),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.save_scene(&scene.id, empty_config()),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.get_scene(&scene.id),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.set_active_scene(Some(&scene.id)),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.delete_scene(&scene.id),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.export_snapshot(),
            Err(StoreError::InvalidSnapshot(_))
        ));
    }

    #[test]
    fn active_payload_validation_fails_closed_for_mismatched_revision_on_reads() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        let mut payload_scene = scene.clone();
        payload_scene.revision = 2;
        let payload = crate::encode(&payload_scene).unwrap();
        let conn = store.conn.lock().unwrap();
        conn.execute(
            "UPDATE scene_revisions SET payload = ?1 WHERE scene_id = ?2 AND revision = 1",
            params![payload, scene.id],
        )
        .unwrap();
        drop(conn);
        assert!(matches!(
            store.get_scene(&scene.id),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.set_active_scene(Some(&scene.id)),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.delete_scene(&scene.id),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert!(matches!(
            store.export_snapshot(),
            Err(StoreError::InvalidSnapshot(_))
        ));
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

    #[test]
    fn restore_snapshot_replaces_store_atomically() {
        let store = SceneStore::open_in_memory().unwrap();
        let old = store.create_scene("Old", empty_config()).unwrap();
        let source = SceneStore::open_in_memory().unwrap();
        let fresh = source.create_scene("Fresh", empty_config()).unwrap();
        source.set_active_scene(Some(&fresh.id)).unwrap();
        let snapshot = source.export_snapshot().unwrap();
        store.restore_snapshot(&snapshot).unwrap();
        assert!(matches!(
            store.get_scene(&old.id),
            Err(StoreError::NotFound(_))
        ));
        assert_eq!(store.get_scene(&fresh.id).unwrap().name, "Fresh");
        assert_eq!(store.get_active_scene().unwrap().unwrap().id, fresh.id);
    }

    #[test]
    fn restore_snapshot_rejects_invalid_active_pointer_without_mutation() {
        let store = SceneStore::open_in_memory().unwrap();
        let old = store.create_scene("Old", empty_config()).unwrap();
        let invalid =
            serde_json::json!({"version": SCHEMA_VERSION, "scenes": [], "active_scene_id": old.id})
                .to_string();
        assert!(matches!(
            store.restore_snapshot(&invalid),
            Err(StoreError::InvalidSnapshot(_))
        ));
        assert_eq!(store.get_scene(&old.id).unwrap().name, "Old");
    }

    #[test]
    fn export_empty_store_contains_no_active_scene() {
        let store = SceneStore::open_in_memory().unwrap();
        let snapshot: crate::SceneStoreSnapshot =
            serde_json::from_str(&store.export_snapshot().unwrap()).unwrap();
        assert_eq!(snapshot.version, SCHEMA_VERSION);
        assert!(snapshot.scenes.is_empty());
        assert_eq!(snapshot.active_scene_id, None);
    }

    #[test]
    fn export_includes_current_revisions_and_active_pointer() {
        let store = SceneStore::open_in_memory().unwrap();
        let first = store.create_scene("First", empty_config()).unwrap();
        store.save_scene(&first.id, empty_config()).unwrap();
        let second = store.create_scene("Second", empty_config()).unwrap();
        store.set_active_scene(Some(&second.id)).unwrap();
        let snapshot: crate::SceneStoreSnapshot =
            serde_json::from_str(&store.export_snapshot().unwrap()).unwrap();
        assert_eq!(snapshot.scenes.len(), 2);
        let first_export = snapshot
            .scenes
            .iter()
            .find(|scene| scene.id == first.id)
            .unwrap();
        assert_eq!(first_export.revision, 2);
        let second_export = snapshot
            .scenes
            .iter()
            .find(|scene| scene.id == second.id)
            .unwrap();
        assert_eq!(second_export.revision, 1);
        assert_eq!(snapshot.active_scene_id, Some(second.id));
    }

    #[test]
    fn export_rejects_corrupt_payload() {
        let store = SceneStore::open_in_memory().unwrap();
        let scene = store.create_scene("Show", empty_config()).unwrap();
        store.corrupt_for_test(&scene.id);
        let err = store.export_snapshot().unwrap_err();
        assert!(matches!(err, StoreError::CorruptPayload(_)));
    }
}
