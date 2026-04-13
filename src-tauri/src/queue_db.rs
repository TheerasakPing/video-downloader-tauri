use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

// --- Data types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersistentQueueItem {
    pub id: i64,
    pub url: String,
    pub status: String,
    pub series_info: Option<String>,
    pub error: Option<String>,
    pub priority: i32,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueueStats {
    pub pending: i32,
    pub downloading: i32,
    pub completed: i32,
    pub failed: i32,
}

// --- Database manager ---

pub struct QueueDb {
    conn: Mutex<Connection>,
}

impl QueueDb {
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        let db_path = app_data_dir.join("download_queue.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open queue DB: {}", e))?;

        // Enable WAL mode for better concurrent read performance
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .map_err(|e| format!("Failed to set WAL mode: {}", e))?;

        let db = Self {
            conn: Mutex::new(conn),
        };
        db.init_schema()?;
        Ok(db)
    }

    fn init_schema(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS download_queue (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL,
                status TEXT NOT NULL DEFAULT 'pending',
                series_info TEXT,
                error TEXT,
                priority INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_queue_status ON download_queue(status);
            CREATE INDEX IF NOT EXISTS idx_queue_priority ON download_queue(priority DESC, created_at);"
        ).map_err(|e| format!("Schema init failed: {}", e))?;

        Ok(())
    }

    pub fn add_item(&self, url: &str) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "INSERT INTO download_queue (url, status, priority) VALUES (?1, 'pending', 0)",
            params![url],
        ).map_err(|e| format!("Add queue item failed: {}", e))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn get_all(&self) -> Result<Vec<PersistentQueueItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, url, status, series_info, error, priority, created_at, updated_at
             FROM download_queue
             ORDER BY priority DESC, created_at"
        ).map_err(|e| e.to_string())?;

        let items: Vec<PersistentQueueItem> = stmt.query_map([], |row| {
            Ok(PersistentQueueItem {
                id: row.get(0)?,
                url: row.get(1)?,
                status: row.get(2)?,
                series_info: row.get(3)?,
                error: row.get(4)?,
                priority: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

        Ok(items)
    }

    pub fn get_pending(&self) -> Result<Vec<PersistentQueueItem>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let mut stmt = conn.prepare(
            "SELECT id, url, status, series_info, error, priority, created_at, updated_at
             FROM download_queue
             WHERE status = 'pending'
             ORDER BY priority DESC, created_at"
        ).map_err(|e| e.to_string())?;

        let items: Vec<PersistentQueueItem> = stmt.query_map([], |row| {
            Ok(PersistentQueueItem {
                id: row.get(0)?,
                url: row.get(1)?,
                status: row.get(2)?,
                series_info: row.get(3)?,
                error: row.get(4)?,
                priority: row.get(5)?,
                created_at: row.get(6)?,
                updated_at: row.get(7)?,
            })
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

        Ok(items)
    }

    pub fn update_status(
        &self,
        id: i64,
        status: &str,
        series_info: Option<&str>,
        error: Option<&str>,
    ) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "UPDATE download_queue SET status = ?1, series_info = ?2, error = ?3, updated_at = datetime('now')
             WHERE id = ?4",
            params![status, series_info, error, id],
        ).map_err(|e| format!("Update queue item failed: {}", e))?;
        Ok(())
    }

    pub fn remove_item(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        conn.execute(
            "DELETE FROM download_queue WHERE id = ?1",
            params![id],
        ).map_err(|e| format!("Remove queue item failed: {}", e))?;
        Ok(())
    }

    pub fn clear_completed(&self) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        let rows = conn.execute(
            "DELETE FROM download_queue WHERE status IN ('completed', 'failed')",
            [],
        ).map_err(|e| format!("Clear completed failed: {}", e))?;
        Ok(rows)
    }

    pub fn get_stats(&self) -> Result<QueueStats, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let pending: i32 = conn.query_row(
            "SELECT COUNT(*) FROM download_queue WHERE status = 'pending'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let downloading: i32 = conn.query_row(
            "SELECT COUNT(*) FROM download_queue WHERE status = 'downloading'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let completed: i32 = conn.query_row(
            "SELECT COUNT(*) FROM download_queue WHERE status = 'completed'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        let failed: i32 = conn.query_row(
            "SELECT COUNT(*) FROM download_queue WHERE status = 'failed'",
            [],
            |row| row.get(0),
        ).unwrap_or(0);

        Ok(QueueStats {
            pending,
            downloading,
            completed,
            failed,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_db() -> QueueDb {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("vdt_test_queue_{}_{}", ts, counter));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).ok();
        QueueDb::new(&dir).expect("Failed to create test QueueDb")
    }

    // ─── Schema & Init ───────────────────────────────

    #[test]
    fn test_db_creates_successfully() {
        let _db = test_db();
    }

    #[test]
    fn test_add_and_get_single_item() {
        let db = test_db();
        let id = db.add_item("https://example.com/video1.m3u8").unwrap();

        let items = db.get_all().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].id, id);
        assert_eq!(items[0].url, "https://example.com/video1.m3u8");
        assert_eq!(items[0].status, "pending");
        assert_eq!(items[0].priority, 0);
        assert!(items[0].error.is_none());
        assert!(items[0].series_info.is_none());
    }

    // ─── Add & Get ───────────────────────────────────

    #[test]
    fn test_add_multiple_items() {
        let db = test_db();
        db.add_item("https://example.com/v1.m3u8").unwrap();
        db.add_item("https://example.com/v2.m3u8").unwrap();
        db.add_item("https://example.com/v3.m3u8").unwrap();

        let items = db.get_all().unwrap();
        assert_eq!(items.len(), 3);
    }

    #[test]
    fn test_get_pending_returns_only_pending() {
        let db = test_db();
        let id1 = db.add_item("https://example.com/v1.m3u8").unwrap();
        let id2 = db.add_item("https://example.com/v2.m3u8").unwrap();
        let _id3 = db.add_item("https://example.com/v3.m3u8").unwrap();

        // Transition id1 to downloading, id2 to completed
        db.update_status(id1, "downloading", None, None).unwrap();
        db.update_status(id2, "completed", None, None).unwrap();

        let pending = db.get_pending().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, _id3);
    }

    // ─── Status Updates ──────────────────────────────

    #[test]
    fn test_update_status_to_downloading() {
        let db = test_db();
        let id = db.add_item("https://example.com/v1.m3u8").unwrap();

        db.update_status(id, "downloading", None, None).unwrap();

        let items = db.get_all().unwrap();
        assert_eq!(items[0].status, "downloading");
    }

    #[test]
    fn test_update_status_with_series_info_and_error() {
        let db = test_db();
        let id = db.add_item("https://example.com/v1.m3u8").unwrap();

        db.update_status(id, "failed", Some("Series: Test"), Some("Network timeout")).unwrap();

        let items = db.get_all().unwrap();
        assert_eq!(items[0].status, "failed");
        assert_eq!(items[0].series_info.as_deref(), Some("Series: Test"));
        assert_eq!(items[0].error.as_deref(), Some("Network timeout"));
    }

    #[test]
    fn test_full_status_lifecycle() {
        let db = test_db();
        let id = db.add_item("https://example.com/v1.m3u8").unwrap();

        // pending → downloading → completed
        db.update_status(id, "downloading", None, None).unwrap();
        db.update_status(id, "completed", Some("Series X"), None).unwrap();

        let items = db.get_all().unwrap();
        assert_eq!(items[0].status, "completed");
        assert_eq!(items[0].series_info.as_deref(), Some("Series X"));

        // Stats should reflect
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.pending, 0);
    }

    // ─── Remove & Clear ──────────────────────────────

    #[test]
    fn test_remove_item() {
        let db = test_db();
        let id1 = db.add_item("https://example.com/v1.m3u8").unwrap();
        let _id2 = db.add_item("https://example.com/v2.m3u8").unwrap();

        db.remove_item(id1).unwrap();

        let items = db.get_all().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].url, "https://example.com/v2.m3u8");
    }

    #[test]
    fn test_clear_completed_removes_completed_and_failed() {
        let db = test_db();
        let id1 = db.add_item("https://example.com/v1.m3u8").unwrap();
        let id2 = db.add_item("https://example.com/v2.m3u8").unwrap();
        let _id3 = db.add_item("https://example.com/v3.m3u8").unwrap();

        db.update_status(id1, "completed", None, None).unwrap();
        db.update_status(id2, "failed", None, Some("Error")).unwrap();

        let cleared = db.clear_completed().unwrap();
        assert_eq!(cleared, 2);

        // Only pending item should remain
        let items = db.get_all().unwrap();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].status, "pending");
    }

    #[test]
    fn test_clear_completed_nothing_to_clear() {
        let db = test_db();
        db.add_item("https://example.com/v1.m3u8").unwrap();

        let cleared = db.clear_completed().unwrap();
        assert_eq!(cleared, 0);

        let items = db.get_all().unwrap();
        assert_eq!(items.len(), 1);
    }

    // ─── Stats ───────────────────────────────────────

    #[test]
    fn test_stats_empty_queue() {
        let db = test_db();
        let stats = db.get_stats().unwrap();
        assert_eq!(stats.pending, 0);
        assert_eq!(stats.downloading, 0);
        assert_eq!(stats.completed, 0);
        assert_eq!(stats.failed, 0);
    }

    #[test]
    fn test_stats_mixed_statuses() {
        let db = test_db();
        let id1 = db.add_item("https://example.com/v1.m3u8").unwrap();
        let id2 = db.add_item("https://example.com/v2.m3u8").unwrap();
        let id3 = db.add_item("https://example.com/v3.m3u8").unwrap();
        let _id4 = db.add_item("https://example.com/v4.m3u8").unwrap();

        db.update_status(id1, "downloading", None, None).unwrap();
        db.update_status(id2, "completed", None, None).unwrap();
        db.update_status(id3, "failed", None, None).unwrap();
        // _id4 stays pending

        let stats = db.get_stats().unwrap();
        assert_eq!(stats.pending, 1);
        assert_eq!(stats.downloading, 1);
        assert_eq!(stats.completed, 1);
        assert_eq!(stats.failed, 1);
    }

    // ─── Priority Ordering ───────────────────────────

    #[test]
    fn test_items_ordered_by_created_at_by_default() {
        let db = test_db();
        db.add_item("https://example.com/first.m3u8").unwrap();
        db.add_item("https://example.com/second.m3u8").unwrap();
        db.add_item("https://example.com/third.m3u8").unwrap();

        let items = db.get_all().unwrap();
        assert_eq!(items[0].url, "https://example.com/first.m3u8");
        assert_eq!(items[1].url, "https://example.com/second.m3u8");
        assert_eq!(items[2].url, "https://example.com/third.m3u8");
    }

    // ─── Edge Cases ──────────────────────────────────

    #[test]
    fn test_remove_nonexistent_item_no_error() {
        let db = test_db();
        // Should not panic or error, just delete 0 rows
        db.remove_item(99999).unwrap();
    }

    #[test]
    fn test_update_nonexistent_item_no_error() {
        let db = test_db();
        db.update_status(99999, "completed", None, None).unwrap();
    }
}
