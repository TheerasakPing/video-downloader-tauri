use std::sync::Arc;
use serde::{Deserialize, Serialize};
use chrono::Timelike;
use tauri::Emitter;

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
    config: Arc<std::sync::Mutex<ScheduleConfig>>,
    app_handle: tauri::AppHandle,
) {
    tokio::spawn(async move {
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
