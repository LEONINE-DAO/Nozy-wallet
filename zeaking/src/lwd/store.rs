//! SQLite persistence for compact blocks and optional Orchard witness rows (Nozy prove shape).

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{ZeakingError, ZeakingResult};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS sync_meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS compact_blocks (
    height INTEGER PRIMARY KEY NOT NULL,
    block_hash BLOB,
    data BLOB NOT NULL
);
CREATE TABLE IF NOT EXISTS orchard_witness (
    cmx_hex TEXT PRIMARY KEY NOT NULL,
    anchor_hex TEXT NOT NULL,
    position INTEGER NOT NULL,
    auth_path_json TEXT NOT NULL,
    block_height INTEGER NOT NULL,
    updated_at INTEGER NOT NULL
);
"#;

/// Thread-safe SQLite store for compact blocks (and witness ingest for spends).
pub struct LwdCompactStore {
    conn: Mutex<Connection>,
}

impl LwdCompactStore {
    pub fn open(path: &Path) -> ZeakingResult<Self> {
        let conn = Connection::open(path).map_err(|e| ZeakingError::Storage(e.to_string()))?;
        conn.execute_batch(SCHEMA)
            .map_err(|e| ZeakingError::Storage(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn put_compact_block(
        &self,
        height: u64,
        block_hash: Option<&[u8]>,
        data: &[u8],
    ) -> ZeakingResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| ZeakingError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO compact_blocks (height, block_hash, data) VALUES (?1, ?2, ?3)
             ON CONFLICT(height) DO UPDATE SET block_hash = excluded.block_hash, data = excluded.data",
            params![height as i64, block_hash, data],
        )
        .map_err(|e| ZeakingError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn max_compact_height(&self) -> ZeakingResult<Option<u64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| ZeakingError::Storage(e.to_string()))?;
        let v: Option<i64> = conn
            .query_row("SELECT MAX(height) FROM compact_blocks", [], |r| r.get(0))
            .optional()
            .map_err(|e| ZeakingError::Storage(e.to_string()))?;
        Ok(v.map(|h| h as u64))
    }

    pub fn set_meta(&self, key: &str, value: &str) -> ZeakingResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| ZeakingError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO sync_meta (key, value) VALUES (?1, ?2) ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| ZeakingError::Storage(e.to_string()))?;
        Ok(())
    }
}
