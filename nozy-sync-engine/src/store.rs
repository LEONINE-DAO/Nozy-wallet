//! SQLite persistence for compact blocks served over CompactTxStreamer.

use std::path::Path;
use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::error::{IndexerError, IndexerResult};

const SCHEMA: &str = r#"
CREATE TABLE IF NOT EXISTS meta (
    key TEXT PRIMARY KEY NOT NULL,
    value TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS compact_blocks (
    height INTEGER PRIMARY KEY NOT NULL,
    block_hash BLOB NOT NULL,
    data BLOB NOT NULL
);
"#;

pub struct IndexerStore {
    conn: Mutex<Connection>,
}

impl IndexerStore {
    pub fn open(path: &Path) -> IndexerResult<Self> {
        if let Some(parent) = path.parent() {
            if !parent.as_os_str().is_empty() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let conn = Connection::open(path).map_err(|e| IndexerError::Storage(e.to_string()))?;
        conn.execute_batch(SCHEMA)
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }

    pub fn put_compact_block(
        &self,
        height: u64,
        block_hash: &[u8],
        data: &[u8],
    ) -> IndexerResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO compact_blocks (height, block_hash, data) VALUES (?1, ?2, ?3)
             ON CONFLICT(height) DO UPDATE SET block_hash = excluded.block_hash, data = excluded.data",
            params![height as i64, block_hash, data],
        )
        .map_err(|e| IndexerError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn get_compact_block(&self, height: u64) -> IndexerResult<Option<Vec<u8>>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        let row: Option<Vec<u8>> = conn
            .query_row(
                "SELECT data FROM compact_blocks WHERE height = ?1",
                params![height as i64],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        Ok(row)
    }

    pub fn get_block_hash(&self, height: u64) -> IndexerResult<Option<Vec<u8>>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        let row: Option<Vec<u8>> = conn
            .query_row(
                "SELECT block_hash FROM compact_blocks WHERE height = ?1",
                params![height as i64],
                |r| r.get(0),
            )
            .optional()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        Ok(row)
    }

    pub fn max_height(&self) -> IndexerResult<Option<u64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        let v: Option<i64> = conn
            .query_row("SELECT MAX(height) FROM compact_blocks", [], |r| {
                r.get::<_, Option<i64>>(0)
            })
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        Ok(v.map(|h| h as u64))
    }

    pub fn min_height(&self) -> IndexerResult<Option<u64>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        let v: Option<i64> = conn
            .query_row("SELECT MIN(height) FROM compact_blocks", [], |r| {
                r.get::<_, Option<i64>>(0)
            })
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        Ok(v.map(|h| h as u64))
    }

    pub fn prune_above(&self, tip: u64) -> IndexerResult<u64> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        let deleted = conn
            .execute(
                "DELETE FROM compact_blocks WHERE height > ?1",
                params![tip as i64],
            )
            .map_err(|e| IndexerError::Storage(e.to_string()))? as u64;
        Ok(deleted)
    }

    pub fn set_meta(&self, key: &str, value: &str) -> IndexerResult<()> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        conn.execute(
            "INSERT INTO meta (key, value) VALUES (?1, ?2)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value",
            params![key, value],
        )
        .map_err(|e| IndexerError::Storage(e.to_string()))?;
        Ok(())
    }

    pub fn get_meta(&self, key: &str) -> IndexerResult<Option<String>> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        let row: Option<String> = conn
            .query_row("SELECT value FROM meta WHERE key = ?1", params![key], |r| {
                r.get(0)
            })
            .optional()
            .map_err(|e| IndexerError::Storage(e.to_string()))?;
        Ok(row)
    }
}
