// Download Scheduler Module
// Handles scheduled downloads with cron-like expressions

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::Emitter;

// --- Existing schedule config for bandwidth limiting ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleConfig {
    pub enabled: bool,
    pub active_start: Option<String>,   // "HH:MM" local time
    pub active_end: Option<String>,     // "HH:MM" local time
    pub speed_during_active: u64,       // KB/s, 0 = unlimited
    pub speed_outside_active: u64,      // KB/s, 0 = unlimited
    pub auto_pause: bool,
    pub auto_resume: bool,
}

impl Default for ScheduleConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            active_start: None,
            active_end: None,
            speed_during_active: 0,
            speed_outside_active: 0,
            auto_pause: false,
            auto_resume: false,
        }
    }
}

fn parse_time(time_str: &str) -> Option<(u32, u32)> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() == 2 {
        let h: u32 = parts[0].parse().ok()?;
        let m: u32 = parts[1].parse().ok()?;
        if h < 24 && m < 60 {
            return Some((h, m));
        }
    }
    None
}

pub fn is_active_now(config: &ScheduleConfig) -> bool {
    let start = config.active_start.as_deref().and_then(parse_time);
    let end = config.active_end.as_deref().and_then(parse_time);
    let (start_h, start_m) = match start { Some(t) => t, None => return true };
    let (end_h, end_m) = match end { Some(t) => t, None => return true };

    // Use chrono::Local with Timelike trait
    use chrono::Timelike;
    let now = chrono::Local::now();
    let now_mins = now.hour() * 60 + now.minute();
    let start_mins = start_h * 60 + start_m;
    let end_mins = end_h * 60 + end_m;

    if start_mins > end_mins {
        // Crosses midnight
        now_mins >= start_mins || now_mins < end_mins
    } else {
        now_mins >= start_mins && now_mins < end_mins
    }
}

/// Start a background scheduler task that emits schedule status events.
pub fn start_scheduler(
    config: std::sync::Arc<Mutex<ScheduleConfig>>,
    app_handle: tauri::AppHandle,
) {
    tauri::async_runtime::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            interval.tick().await;
            let cfg = config.lock().unwrap().clone();
            if !cfg.enabled { continue; }

            let active = is_active_now(&cfg);
            let speed_limit = if active { cfg.speed_during_active } else { cfg.speed_outside_active };

            let _ = app_handle.emit("schedule-status", serde_json::json!({
                "active": active,
                "currentSpeedLimit": speed_limit,
            }));
        }
    });
}

// --- Download Schedule Database ---

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduleEntry {
    pub id: i64,
    pub name: String,
    pub url: String,
    pub output_dir: String,
    pub cron_expression: String,
    pub enabled: bool,
    pub last_run: Option<String>,
    pub next_run: Option<String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateScheduleRequest {
    pub name: String,
    pub url: String,
    pub output_dir: String,
    pub cron_expression: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateScheduleRequest {
    pub id: i64,
    pub name: Option<String>,
    pub url: Option<String>,
    pub output_dir: Option<String>,
    pub cron_expression: Option<String>,
}

/// Database manager for download schedules
pub struct ScheduleDb {
    conn: Mutex<Connection>,
}

impl ScheduleDb {
    pub fn new(app_data_dir: &std::path::Path) -> Result<Self, String> {
        let db_path = app_data_dir.join("schedule.db");
        let conn = Connection::open(&db_path)
            .map_err(|e| format!("Failed to open schedule DB: {}", e))?;

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
            "CREATE TABLE IF NOT EXISTS download_schedule (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                url TEXT NOT NULL,
                output_dir TEXT NOT NULL,
                cron_expression TEXT NOT NULL,
                enabled INTEGER NOT NULL DEFAULT 1,
                last_run TEXT,
                next_run TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            );
            CREATE INDEX IF NOT EXISTS idx_schedule_enabled ON download_schedule(enabled);
            CREATE INDEX IF NOT EXISTS idx_schedule_next_run ON download_schedule(next_run);"
        ).map_err(|e| format!("Schema init failed: {}", e))?;

        Ok(())
    }

    pub fn create_schedule(&self, req: CreateScheduleRequest) -> Result<ScheduleEntry, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let next_run = self.calculate_next_run(&req.cron_expression, &conn)?;

        conn.execute(
            "INSERT INTO download_schedule (name, url, output_dir, cron_expression, enabled, next_run)
             VALUES (?1, ?2, ?3, ?4, 1, ?5)",
            params![req.name, req.url, req.output_dir, req.cron_expression, next_run],
        ).map_err(|e| format!("Create schedule failed: {}", e))?;

        let id = conn.last_insert_rowid();
        Self::query_schedule_by_id(&conn, id)
    }

    pub fn get_schedules(&self) -> Result<Vec<ScheduleEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn.prepare(
            "SELECT id, name, url, output_dir, cron_expression, enabled, last_run, next_run, created_at
             FROM download_schedule
             ORDER BY created_at DESC"
        ).map_err(|e| e.to_string())?;

        let schedules = stmt.query_map([], |row| {
            Ok(ScheduleEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                output_dir: row.get(3)?,
                cron_expression: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                last_run: row.get(6)?,
                next_run: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

        Ok(schedules)
    }

    #[allow(dead_code)]
    fn get_schedule_by_id(&self, id: i64) -> Result<ScheduleEntry, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;
        Self::query_schedule_by_id(&conn, id)
    }

    fn query_schedule_by_id(conn: &Connection, id: i64) -> Result<ScheduleEntry, String> {
        conn.query_row(
            "SELECT id, name, url, output_dir, cron_expression, enabled, last_run, next_run, created_at
             FROM download_schedule WHERE id = ?1",
            params![id],
            |row| {
                Ok(ScheduleEntry {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    url: row.get(2)?,
                    output_dir: row.get(3)?,
                    cron_expression: row.get(4)?,
                    enabled: row.get::<_, i32>(5)? != 0,
                    last_run: row.get(6)?,
                    next_run: row.get(7)?,
                    created_at: row.get(8)?,
                })
            },
        ).map_err(|e| format!("Schedule not found: {}", e))
    }

    pub fn update_schedule(&self, req: UpdateScheduleRequest) -> Result<ScheduleEntry, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // Build dynamic update query
        let mut updates = Vec::new();
        let mut params: Vec<String> = Vec::new();
        let mut param_refs: Vec<&dyn rusqlite::ToSql> = Vec::new();

        if let Some(ref name) = req.name {
            updates.push("name = ?");
            params.push(name.clone());
        }
        if let Some(ref url) = req.url {
            updates.push("url = ?");
            params.push(url.clone());
        }
        if let Some(ref output_dir) = req.output_dir {
            updates.push("output_dir = ?");
            params.push(output_dir.clone());
        }
        if let Some(ref cron) = req.cron_expression {
            updates.push("cron_expression = ?");
            params.push(cron.clone());
            // Recalculate next run when cron changes
            let next_run = self.calculate_next_run(cron, &conn)?;
            updates.push("next_run = ?");
            params.push(next_run);
        }

        if updates.is_empty() {
            return Self::query_schedule_by_id(&conn, req.id);
        }

        for p in &params {
            param_refs.push(p);
        }
        param_refs.push(&req.id);

        let sql = format!("UPDATE download_schedule SET {} WHERE id = ?", updates.join(", "));

        conn.execute(&sql, param_refs.as_slice())
            .map_err(|e| format!("Update schedule failed: {}", e))?;

        Self::query_schedule_by_id(&conn, req.id)
    }

    pub fn toggle_schedule(&self, id: i64) -> Result<bool, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let current: bool = conn.query_row(
            "SELECT enabled FROM download_schedule WHERE id = ?1",
            params![id],
            |row| row.get::<_, i32>(0).map(|v| v != 0),
        ).map_err(|e| format!("Toggle schedule failed: {}", e))?;

        let new_val = !current;
        conn.execute(
            "UPDATE download_schedule SET enabled = ?1 WHERE id = ?2",
            params![new_val as i32, id],
        ).map_err(|e| format!("Toggle schedule failed: {}", e))?;

        Ok(new_val)
    }

    pub fn delete_schedule(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        conn.execute("DELETE FROM download_schedule WHERE id = ?1", params![id])
            .map_err(|e| format!("Delete schedule failed: {}", e))?;

        Ok(())
    }

    pub fn get_due_schedules(&self) -> Result<Vec<ScheduleEntry>, String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        let mut stmt = conn.prepare(
            "SELECT id, name, url, output_dir, cron_expression, enabled, last_run, next_run, created_at
             FROM download_schedule
             WHERE enabled = 1 AND next_run IS NOT NULL AND datetime(next_run) <= datetime('now')
             ORDER BY next_run"
        ).map_err(|e| e.to_string())?;

        let schedules = stmt.query_map([], |row| {
            Ok(ScheduleEntry {
                id: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                output_dir: row.get(3)?,
                cron_expression: row.get(4)?,
                enabled: row.get::<_, i32>(5)? != 0,
                last_run: row.get(6)?,
                next_run: row.get(7)?,
                created_at: row.get(8)?,
            })
        }).map_err(|e| e.to_string())?
        .filter_map(|r| r.ok())
        .collect();

        Ok(schedules)
    }

    pub fn mark_schedule_run(&self, id: i64) -> Result<(), String> {
        let conn = self.conn.lock().map_err(|e| e.to_string())?;

        // Get the cron expression first
        let cron_expr: String = conn.query_row(
            "SELECT cron_expression FROM download_schedule WHERE id = ?1",
            params![id],
            |row| row.get(0),
        ).map_err(|e| format!("Failed to get cron expression: {}", e))?;

        let now = chrono::Utc::now().to_rfc3339();
        let next_run = self.calculate_next_run(&cron_expr, &conn)?;

        conn.execute(
            "UPDATE download_schedule SET last_run = ?1, next_run = ?2 WHERE id = ?3",
            params![now, next_run, id],
        ).map_err(|e| format!("Mark schedule run failed: {}", e))?;

        Ok(())
    }

    /// Calculate next run time from a simplified cron expression
    /// Uses SQLite datetime functions for reliable time calculations
    /// Supports: "daily HH:MM", "weekly N HH:MM" (N=0-6, 0=Sunday), "hourly"
    fn calculate_next_run(&self, cron_expr: &str, conn: &Connection) -> Result<String, String> {
        let expr = cron_expr.trim().to_lowercase();

        let sql = if expr == "hourly" {
            // Next hour at :00
            "SELECT datetime(strftime('%Y-%m-%d %H:00:00', 'now', '+1 hour'))"
        } else if expr.starts_with("daily ") {
            // "daily HH:MM"
            let time_str = expr.strip_prefix("daily ")
                .ok_or_else(|| "Invalid daily format".to_string())?;
            let (hour, minute) = self.parse_hhmm(time_str)?;
            let time = format!("{:02}:{:02}", hour, minute);
            // Get next occurrence of this time (today if in future, tomorrow if passed)
            return Ok(conn.query_row(
                "SELECT CASE
                    WHEN datetime('now', 'localtime') < datetime(date('now', 'localtime') || ' ' || ?1, 'localtime')
                    THEN datetime(date('now', 'localtime') || ' ' || ?1, 'localtime')
                    ELSE datetime(date('now', '+1 day', 'localtime') || ' ' || ?1, 'localtime')
                END",
                [&time],
                |row| row.get(0),
            ).map_err(|e| format!("Calculate next daily run failed: {}", e))?);
        } else if expr.starts_with("weekly ") {
            // "weekly N HH:MM" where N=0-6 (Sunday=0)
            let parts: Vec<&str> = expr.strip_prefix("weekly ")
                .ok_or_else(|| "Invalid weekly format".to_string())?
                .split_whitespace()
                .collect();
            if parts.len() != 2 {
                return Err("Invalid weekly format, expected 'weekly N HH:MM'".to_string());
            }
            let day_of_week: i32 = parts[0].parse()
                .map_err(|_| "Invalid day of week".to_string())?;
            if day_of_week < 0 || day_of_week > 6 {
                return Err("Day of week must be 0-6".to_string());
            }
            let (hour, minute) = self.parse_hhmm(parts[1])?;
            let time = format!("{:02}:{:02}", hour, minute);
            // SQLite: 0=Sunday, 1=Monday, etc. - same as our format
            // Use parameterized values to prevent SQL injection
            return Ok(conn.query_row(
                "SELECT datetime(date('now', 'localtime',
                    CASE
                        WHEN cast(strftime('%w', 'now', 'localtime') as integer) < ?1
                        THEN (?1 - cast(strftime('%w', 'now', 'localtime') as integer)) || ' days'
                        ELSE (7 + ?1 - cast(strftime('%w', 'now', 'localtime') as integer)) || ' days'
                    END
                ) || ' ' || ?2, 'localtime')",
                rusqlite::params![day_of_week, &time],
                |row| row.get(0),
            ).map_err(|e| format!("Calculate next weekly run failed: {}", e))?);
        } else {
            return Err(format!("Unsupported cron expression: {}", cron_expr));
        };

        conn.query_row(sql, [], |row| row.get(0))
            .map_err(|e| format!("Calculate next run failed: {}", e))
    }

    fn parse_hhmm(&self, time_str: &str) -> Result<(u32, u32), String> {
        let parts: Vec<&str> = time_str.split(':').collect();
        if parts.len() != 2 {
            return Err("Time must be HH:MM".to_string());
        }
        let hour: u32 = parts[0].parse().map_err(|_| "Invalid hour".to_string())?;
        let minute: u32 = parts[1].parse().map_err(|_| "Invalid minute".to_string())?;
        if hour > 23 || minute > 59 {
            return Err("Invalid time values".to_string());
        }
        Ok((hour, minute))
    }
}

// --- Tauri Commands ---

#[tauri::command]
pub fn cmd_create_schedule(
    req: CreateScheduleRequest,
    state: tauri::State<'_, crate::AppState>,
) -> Result<ScheduleEntry, String> {
    state.schedule_db.create_schedule(req)
}

#[tauri::command]
pub fn cmd_get_schedules(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<ScheduleEntry>, String> {
    state.schedule_db.get_schedules()
}

#[tauri::command]
pub fn cmd_update_schedule(
    req: UpdateScheduleRequest,
    state: tauri::State<'_, crate::AppState>,
) -> Result<ScheduleEntry, String> {
    state.schedule_db.update_schedule(req)
}

#[tauri::command]
pub fn cmd_toggle_schedule(
    id: i64,
    state: tauri::State<'_, crate::AppState>,
) -> Result<bool, String> {
    state.schedule_db.toggle_schedule(id)
}

#[tauri::command]
pub fn cmd_delete_schedule(
    id: i64,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    state.schedule_db.delete_schedule(id)
}

#[tauri::command]
pub fn cmd_get_due_schedules(
    state: tauri::State<'_, crate::AppState>,
) -> Result<Vec<ScheduleEntry>, String> {
    state.schedule_db.get_due_schedules()
}

#[tauri::command]
pub fn cmd_mark_schedule_run(
    id: i64,
    state: tauri::State<'_, crate::AppState>,
) -> Result<(), String> {
    state.schedule_db.mark_schedule_run(id)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_db() -> ScheduleDb {
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = std::env::temp_dir().join(format!("vdt_test_schedule_{}_{}", ts, counter));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).ok();
        ScheduleDb::new(&dir).expect("Failed to create test ScheduleDb")
    }

    fn make_request(name: &str, url: &str, cron: &str) -> CreateScheduleRequest {
        CreateScheduleRequest {
            name: name.to_string(),
            url: url.to_string(),
            output_dir: "/tmp/downloads".to_string(),
            cron_expression: cron.to_string(),
        }
    }

    // ─── ScheduleConfig Defaults ──────────────────────

    #[test]
    fn test_default_schedule_config() {
        let config = ScheduleConfig::default();
        assert!(!config.enabled);
        assert!(config.active_start.is_none());
        assert!(config.active_end.is_none());
        assert_eq!(config.speed_during_active, 0);
        assert_eq!(config.speed_outside_active, 0);
        assert!(!config.auto_pause);
        assert!(!config.auto_resume);
    }

    #[test]
    fn test_schedule_config_serialization() {
        let config = ScheduleConfig {
            enabled: true,
            active_start: Some("22:00".to_string()),
            active_end: Some("06:00".to_string()),
            speed_during_active: 1024,
            speed_outside_active: 0,
            auto_pause: true,
            auto_resume: true,
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("activeStart"));
        assert!(json.contains("speedDuringActive"));
        let back: ScheduleConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(back.active_start.as_deref(), Some("22:00"));
    }

    // ─── parse_time ──────────────────────────────────

    #[test]
    fn test_parse_time_valid() {
        assert_eq!(parse_time("00:00"), Some((0, 0)));
        assert_eq!(parse_time("12:30"), Some((12, 30)));
        assert_eq!(parse_time("23:59"), Some((23, 59)));
    }

    #[test]
    fn test_parse_time_invalid() {
        assert_eq!(parse_time("24:00"), None);
        assert_eq!(parse_time("12:60"), None);
        assert_eq!(parse_time("25:99"), None);
    }

    #[test]
    fn test_parse_time_malformed() {
        assert_eq!(parse_time(""), None);
        assert_eq!(parse_time("12"), None);
        assert_eq!(parse_time("12:30:45"), None);
        assert_eq!(parse_time("ab:cd"), None);
    }

    // ─── is_active_now ───────────────────────────────

    #[test]
    fn test_is_active_now_no_start_returns_true() {
        let config = ScheduleConfig {
            active_start: None,
            active_end: Some("23:59".to_string()),
            ..Default::default()
        };
        assert!(is_active_now(&config));
    }

    #[test]
    fn test_is_active_now_no_end_returns_true() {
        let config = ScheduleConfig {
            active_start: Some("00:00".to_string()),
            active_end: None,
            ..Default::default()
        };
        assert!(is_active_now(&config));
    }

    // ─── ScheduleDb CRUD ─────────────────────────────

    #[test]
    fn test_db_creates_successfully() {
        let _db = test_db();
    }

    #[test]
    fn test_create_schedule_daily() {
        let db = test_db();
        let entry = db.create_schedule(make_request("Daily test", "https://example.com", "daily 08:00")).unwrap();

        assert!(entry.id > 0);
        assert_eq!(entry.name, "Daily test");
        assert_eq!(entry.url, "https://example.com");
        assert_eq!(entry.cron_expression, "daily 08:00");
        assert!(entry.enabled);
        assert!(entry.next_run.is_some());
        assert!(entry.last_run.is_none());
    }

    #[test]
    fn test_create_schedule_hourly() {
        let db = test_db();
        let entry = db.create_schedule(make_request("Hourly", "https://example.com", "hourly")).unwrap();
        assert!(entry.next_run.is_some());
        assert!(entry.next_run.unwrap().contains(":00:00"));
    }

    #[test]
    fn test_create_schedule_weekly() {
        let db = test_db();
        let entry = db.create_schedule(make_request("Weekly", "https://example.com", "weekly 1 09:00")).unwrap();
        assert!(entry.next_run.is_some());
    }

    #[test]
    fn test_create_schedule_invalid_cron() {
        let db = test_db();
        let result = db.create_schedule(make_request("Bad", "https://example.com", "invalid cron"));
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Unsupported cron"));
    }

    #[test]
    fn test_get_schedules_empty() {
        let db = test_db();
        let schedules = db.get_schedules().unwrap();
        assert!(schedules.is_empty());
    }

    #[test]
    fn test_get_schedules_returns_all() {
        let db = test_db();
        db.create_schedule(make_request("First", "https://a.com", "hourly")).unwrap();
        db.create_schedule(make_request("Second", "https://b.com", "daily 10:00")).unwrap();

        let schedules = db.get_schedules().unwrap();
        assert_eq!(schedules.len(), 2);
    }

    #[test]
    fn test_update_schedule_name() {
        let db = test_db();
        let entry = db.create_schedule(make_request("Original", "https://a.com", "hourly")).unwrap();

        let updated = db.update_schedule(UpdateScheduleRequest {
            id: entry.id,
            name: Some("Updated".to_string()),
            url: None,
            output_dir: None,
            cron_expression: None,
        }).unwrap();

        assert_eq!(updated.name, "Updated");
        assert_eq!(updated.url, "https://a.com"); // unchanged
    }

    #[test]
    fn test_update_schedule_cron_recalculates_next_run() {
        let db = test_db();
        let entry = db.create_schedule(make_request("Test", "https://a.com", "daily 08:00")).unwrap();

        let updated = db.update_schedule(UpdateScheduleRequest {
            id: entry.id,
            name: None,
            url: None,
            output_dir: None,
            cron_expression: Some("daily 20:00".to_string()),
        }).unwrap();

        assert_eq!(updated.cron_expression, "daily 20:00");
        // next_run should be recalculated (may or may not differ depending on current time)
        assert!(updated.next_run.is_some());
    }

    #[test]
    fn test_update_no_fields_returns_unchanged() {
        let db = test_db();
        let entry = db.create_schedule(make_request("Test", "https://a.com", "hourly")).unwrap();

        let result = db.update_schedule(UpdateScheduleRequest {
            id: entry.id,
            name: None,
            url: None,
            output_dir: None,
            cron_expression: None,
        }).unwrap();

        assert_eq!(result.name, "Test");
    }

    // ─── Toggle ──────────────────────────────────────

    #[test]
    fn test_toggle_schedule() {
        let db = test_db();
        let entry = db.create_schedule(make_request("Test", "https://a.com", "hourly")).unwrap();
        assert!(entry.enabled);

        let new_state = db.toggle_schedule(entry.id).unwrap();
        assert!(!new_state);

        let back = db.toggle_schedule(entry.id).unwrap();
        assert!(back);
    }

    // ─── Delete ──────────────────────────────────────

    #[test]
    fn test_delete_schedule() {
        let db = test_db();
        let entry = db.create_schedule(make_request("Delete me", "https://a.com", "hourly")).unwrap();

        db.delete_schedule(entry.id).unwrap();

        let schedules = db.get_schedules().unwrap();
        assert!(schedules.is_empty());
    }

    // ─── Due Schedules ───────────────────────────────

    #[test]
    fn test_get_due_schedules_empty() {
        let db = test_db();
        let due = db.get_due_schedules().unwrap();
        assert!(due.is_empty());
    }

    #[test]
    fn test_mark_schedule_run() {
        let db = test_db();
        let entry = db.create_schedule(make_request("Test", "https://a.com", "hourly")).unwrap();

        db.mark_schedule_run(entry.id).unwrap();

        // Verify last_run was set by checking the schedule
        let schedules = db.get_schedules().unwrap();
        assert!(schedules[0].last_run.is_some());
        // next_run should be recalculated
        assert!(schedules[0].next_run.is_some());
    }

    // ─── parse_hhmm ──────────────────────────────────

    #[test]
    fn test_parse_hhmm_valid() {
        let db = test_db();
        assert_eq!(db.parse_hhmm("00:00").unwrap(), (0, 0));
        assert_eq!(db.parse_hhmm("12:30").unwrap(), (12, 30));
        assert_eq!(db.parse_hhmm("23:59").unwrap(), (23, 59));
    }

    #[test]
    fn test_parse_hhmm_invalid() {
        let db = test_db();
        assert!(db.parse_hhmm("24:00").is_err());
        assert!(db.parse_hhmm("12:60").is_err());
        assert!(db.parse_hhmm("25:99").is_err());
    }

    #[test]
    fn test_parse_hhmm_malformed() {
        let db = test_db();
        assert!(db.parse_hhmm("abc").is_err());
        assert!(db.parse_hhmm("12").is_err());
        assert!(db.parse_hhmm("").is_err());
    }

    // ─── Edge Cases ──────────────────────────────────

    #[test]
    fn test_delete_nonexistent_no_error() {
        let db = test_db();
        db.delete_schedule(99999).unwrap();
    }

    #[test]
    fn test_toggle_nonexistent_returns_error() {
        let db = test_db();
        assert!(db.toggle_schedule(99999).is_err());
    }
}
