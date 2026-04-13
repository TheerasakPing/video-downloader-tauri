use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Mutex;

// --- Data types ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NotificationEntry {
    pub id: i64,
    pub category: String,
    pub title: String,
    pub message: String,
    pub read: bool,
    pub action_type: Option<String>,
    pub action_data: Option<String>,
    pub created_at: String,
}

// --- Database manager ---

pub struct NotificationDb {
    conn: Mutex<Connection>,
}

impl NotificationDb {
    pub fn new(app_data_dir: &Path) -> Result<Self, String> {
        let db_path = app_data_dir.join("notifications.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open notifications DB: {}", e))?;

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
            "CREATE TABLE IF NOT EXISTS notification_log (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                category TEXT NOT NULL,
                title TEXT NOT NULL,
                message TEXT NOT NULL,
                read INTEGER NOT NULL DEFAULT 0,
                action_type TEXT,
                action_data TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_notification_read ON notification_log(read);
            CREATE INDEX IF NOT EXISTS idx_notification_created ON notification_log(created_at DESC);"
        ).map_err(|e| format!("Schema init failed: {}", e))?;

        Ok(())
    }

    pub fn log_notification(
        &self,
        category: &str,
        title: &str,
        message: &str,
        action_type: Option<&str>,
        action_data: Option<&str>,
    ) -> Result<i64, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "INSERT INTO notification_log (category, title, message, action_type, action_data)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![category, title, message, action_type, action_data],
        )
        .map_err(|e| format!("Failed to log notification: {}", e))?;

        Ok(conn.last_insert_rowid())
    }

    pub fn get_notifications(&self, limit: i32, unread_only: bool) -> Result<Vec<NotificationEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let sql = if unread_only {
            "SELECT id, category, title, message, read, action_type, action_data, created_at
             FROM notification_log WHERE read = 0 ORDER BY created_at DESC LIMIT ?1"
        } else {
            "SELECT id, category, title, message, read, action_type, action_data, created_at
             FROM notification_log ORDER BY created_at DESC LIMIT ?1"
        };

        let mut stmt = conn.prepare(sql).map_err(|e| format!("Failed to prepare query: {}", e))?;

        let notifications: Vec<NotificationEntry> = stmt
            .query_map(params![limit], |row| {
                Ok(NotificationEntry {
                    id: row.get(0)?,
                    category: row.get(1)?,
                    title: row.get(2)?,
                    message: row.get(3)?,
                    read: row.get::<_, i32>(4)? != 0,
                    action_type: row.get(5)?,
                    action_data: row.get(6)?,
                    created_at: row.get(7)?,
                })
            })
            .map_err(|e| format!("Failed to query notifications: {}", e))?
            .filter_map(|n| n.ok())
            .collect();

        Ok(notifications)
    }

    pub fn mark_read(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute(
            "UPDATE notification_log SET read = 1 WHERE id = ?1",
            params![id],
        )
        .map_err(|e| format!("Failed to mark notification as read: {}", e))?;

        Ok(())
    }

    pub fn mark_all_read(&self) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("UPDATE notification_log SET read = 1 WHERE read = 0", [])
            .map_err(|e| format!("Failed to mark all as read: {}", e))?;

        Ok(())
    }

    pub fn get_unread_count(&self) -> Result<i32, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let count: i32 = conn
            .query_row("SELECT COUNT(*) FROM notification_log WHERE read = 0", [], |row| {
                row.get(0)
            })
            .map_err(|e| format!("Failed to get unread count: {}", e))?;

        Ok(count)
    }

    pub fn clear_old(&self, days: i32) -> Result<usize, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let affected = conn
            .execute(
                "DELETE FROM notification_log WHERE created_at < datetime('now', '-' || ?1 || ' days')",
                params![days],
            )
            .map_err(|e| format!("Failed to clear old notifications: {}", e))?;

        Ok(affected)
    }

    #[allow(dead_code)]
    pub fn delete_notification(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM notification_log WHERE id = ?1", params![id])
            .map_err(|e| format!("Failed to delete notification: {}", e))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> NotificationDb {
        // Use unique temp directory for each test to avoid locking
        let unique_id = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let dir = std::env::temp_dir().join(format!("vdt_test_notifications_{}", unique_id));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).ok();
        NotificationDb::new(&dir).expect("Failed to create test DB")
    }

    #[test]
    fn test_log_and_get_notifications() {
        let db = test_db();
        let id = db.log_notification("download", "Test", "Test message", None, None).unwrap();
        assert!(id > 0);

        let notifs = db.get_notifications(10, false).unwrap();
        // Account for potential internal notifications
        assert!(notifs.len() >= 1);
        assert_eq!(notifs[notifs.len() - 1].title, "Test");
    }

    #[test]
    fn test_mark_read() {
        let db = test_db();
        let id = db.log_notification("test", "Title", "Msg", None, None).unwrap();
        db.mark_read(id).unwrap();

        let count = db.get_unread_count().unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_log_with_action_data() {
        let db = test_db();
        let id = db.log_notification(
            "download",
            "Download Complete",
            "Episode 1 finished",
            Some("open_file"),
            Some("/path/to/file.mp4"),
        ).unwrap();
        assert!(id > 0);

        let notifs = db.get_notifications(10, false).unwrap();
        let last = notifs.iter().find(|n| n.id == id).unwrap();
        assert_eq!(last.action_type, Some("open_file".to_string()));
        assert_eq!(last.action_data, Some("/path/to/file.mp4".to_string()));
    }

    #[test]
    fn test_get_notifications_unread_only() {
        let db = test_db();
        let id1 = db.log_notification("cat1", "Unread", "msg1", None, None).unwrap();
        let _id2 = db.log_notification("cat2", "To Read", "msg2", None, None).unwrap();
        db.mark_read(id1).unwrap();

        let unread = db.get_notifications(10, true).unwrap();
        assert!(unread.iter().all(|n| !n.read));
        assert!(unread.iter().any(|n| n.title == "To Read"));
        assert!(!unread.iter().any(|n| n.title == "Unread"));
    }

    #[test]
    fn test_mark_all_read() {
        let db = test_db();
        db.log_notification("a", "First", "m1", None, None).unwrap();
        db.log_notification("b", "Second", "m2", None, None).unwrap();
        db.log_notification("c", "Third", "m3", None, None).unwrap();

        assert_eq!(db.get_unread_count().unwrap(), 3);
        db.mark_all_read().unwrap();
        assert_eq!(db.get_unread_count().unwrap(), 0);

        let all = db.get_notifications(10, false).unwrap();
        assert!(all.iter().all(|n| n.read));
    }

    #[test]
    fn test_delete_notification() {
        let db = test_db();
        let id = db.log_notification("del", "Delete Me", "msg", None, None).unwrap();
        assert!(db.get_notifications(10, false).unwrap().iter().any(|n| n.id == id));

        db.delete_notification(id).unwrap();
        assert!(!db.get_notifications(10, false).unwrap().iter().any(|n| n.id == id));
    }

    #[test]
    fn test_unread_count_accuracy() {
        let db = test_db();
        assert_eq!(db.get_unread_count().unwrap(), 0);

        db.log_notification("a", "A", "m", None, None).unwrap();
        db.log_notification("b", "B", "m", None, None).unwrap();
        assert_eq!(db.get_unread_count().unwrap(), 2);

        let id = db.log_notification("c", "C", "m", None, None).unwrap();
        assert_eq!(db.get_unread_count().unwrap(), 3);

        db.mark_read(id).unwrap();
        assert_eq!(db.get_unread_count().unwrap(), 2);
    }

    #[test]
    fn test_notifications_returns_all() {
        let db = test_db();
        let id1 = db.log_notification("a", "First", "m", None, None).unwrap();
        let _id2 = db.log_notification("b", "Second", "m", None, None).unwrap();
        let id3 = db.log_notification("c", "Third", "m", None, None).unwrap();

        let notifs = db.get_notifications(10, false).unwrap();
        let ids: Vec<i64> = notifs.iter().map(|n| n.id).collect();
        assert!(ids.contains(&id1));
        assert!(ids.contains(&_id2));
        assert!(ids.contains(&id3));
    }

    #[test]
    fn test_get_notifications_limit() {
        let db = test_db();
        for i in 0..5 {
            db.log_notification("cat", &format!("Notif {}", i), "msg", None, None).unwrap();
        }

        let limited = db.get_notifications(2, false).unwrap();
        assert_eq!(limited.len(), 2);
    }
}
