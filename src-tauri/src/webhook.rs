// Webhook Notification Module
// Handles sending webhook notifications to Discord, Line Notify, and custom endpoints

use serde::{Deserialize, Serialize};
use tauri::AppHandle;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WebhookConfig {
    pub enabled: bool,
    pub url: String,
    pub webhook_type: String,
    pub secret: Option<String>,
    pub events: Vec<String>,
}

impl Default for WebhookConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            url: String::new(),
            webhook_type: "discord".to_string(),
            secret: None,
            events: vec!["download_complete".to_string()],
        }
    }
}

/// Send webhook notification based on configuration
pub async fn send_webhook(
    config: &WebhookConfig,
    event: &str,
    title: &str,
    message: &str,
) -> Result<(), String> {
    if !config.enabled || config.url.is_empty() {
        return Ok(());
    }

    // Check if this event is enabled
    if !config.events.contains(&event.to_string()) {
        return Ok(());
    }

    let client = reqwest::Client::new();

    match config.webhook_type.as_str() {
        "discord" => {
            let payload = serde_json::json!({
                "embeds": [{
                    "title": title,
                    "description": message,
                    "color": match event {
                        "download_complete" => 5763719,  // green
                        "download_failed" => 15548997,   // red
                        "new_episode" => 3447003,        // orange
                        _ => 5814783,                     // blue
                    },
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "footer": {
                        "text": "Video Downloader Tauri"
                    }
                }]
            });
            client
                .post(&config.url)
                .json(&payload)
                .send()
                .await
                .map_err(|e| format!("Discord webhook failed: {}", e))?;
        }
        "line" => {
            let token = config.secret.as_deref().unwrap_or("");
            let msg = format!("{}\n{}", title, message);
            let client = reqwest::Client::builder()
                .build()
                .map_err(|e| format!("Failed to build client: {}", e))?;

            client
                .post("https://notify-api.line.me/api/notify")
                .header("Authorization", format!("Bearer {}", token))
                .form(&[("message", msg)])
                .send()
                .await
                .map_err(|e| format!("Line notify failed: {}", e))?;
        }
        _ => {
            // Custom POST
            let payload = serde_json::json!({
                "event": event,
                "title": title,
                "message": message,
                "timestamp": chrono::Utc::now().to_rfc3339()
            });
            let mut req = client.post(&config.url).json(&payload);
            if let Some(ref secret) = config.secret {
                req = req.header("Authorization", format!("Bearer {}", secret));
            }
            req.send()
                .await
                .map_err(|e| format!("Webhook failed: {}", e))?;
        }
    }
    Ok(())
}

/// Send a test webhook notification
pub async fn send_test_webhook(config: &WebhookConfig) -> Result<String, String> {
    if !config.enabled || config.url.is_empty() {
        return Err("Webhook is not enabled or URL is empty".to_string());
    }

    let test_message = match config.webhook_type.as_str() {
        "discord" => "Test notification from Video Downloader Tauri! Your webhook is working correctly.",
        "line" => "Test notification from Video Downloader Tauri!",
        _ => "Test notification from Video Downloader Tauri!",
    };

    send_webhook(
        config,
        "test",
        "Test Notification",
        test_message,
    ).await?;

    Ok("Test notification sent successfully!".to_string())
}

/// Check for new episodes in library series
pub async fn check_new_episodes(
    library_db: &crate::library::LibraryDb,
    notification_db: &crate::notifications::NotificationDb,
    webhook_config: &WebhookConfig,
    _app_handle: &AppHandle,
) -> Result<Vec<String>, String> {
    let entries = library_db.get_library(None)?;
    let new_episodes = Vec::new();

    for entry in entries {
        if let Some(ref source_url) = entry.source_url {
            // Only check series that have source URLs (can be re-fetched)
            if source_url.is_empty() || source_url.contains(".m3u8") || source_url.contains(".mp4") {
                continue;
            }

            // Log notification for new episodes
            let msg = format!(
                "Checking {} for new episodes (current: {})",
                entry.title, entry.total_episodes
            );

            // For now, we'll just log this - actual refetching would need the full fetch_series logic
            // This is a simplified version that logs the check
            let _ = notification_db.log_notification(
                "auto_check",
                "Episode Check",
                &msg,
                Some("open_library"),
                Some(&entry.id.to_string()),
            );
        }
    }

    // Send webhook if any new episodes found
    if !new_episodes.is_empty() && webhook_config.enabled {
        let summary = format!("Found new episodes for {} series", new_episodes.len());
        let details = new_episodes.join("\n");
        let _ = send_webhook(webhook_config, "new_episode", &summary, &details).await;
    }

    Ok(new_episodes)
}

// --- Tauri Commands ---

#[tauri::command]
pub fn cmd_get_webhook_config(
    state: tauri::State<'_, crate::AppState>,
) -> Result<WebhookConfig, String> {
    let cfg = state.webhook_config.lock().unwrap();
    Ok(cfg.clone())
}

#[tauri::command]
pub fn cmd_save_webhook_config(
    state: tauri::State<'_, crate::AppState>,
    config: WebhookConfig,
) -> Result<(), String> {
    let mut cfg = state.webhook_config.lock().unwrap();
    *cfg = config;
    Ok(())
}

#[tauri::command]
pub async fn cmd_test_webhook(
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    let config = state.webhook_config.lock().unwrap().clone();
    send_test_webhook(&config).await
}

#[tauri::command]
pub async fn cmd_check_new_episodes(
    state: tauri::State<'_, crate::AppState>,
    app_handle: AppHandle,
) -> Result<Vec<String>, String> {
    let config = state.webhook_config.lock().unwrap().clone();
    check_new_episodes(
        &state.library_db,
        &state.notification_db,
        &config,
        &app_handle,
    ).await
}

#[tauri::command]
pub async fn cmd_send_test_notification(
    state: tauri::State<'_, crate::AppState>,
) -> Result<String, String> {
    let config = state.webhook_config.lock().unwrap().clone();

    send_webhook(
        &config,
        "test",
        "Test Notification",
        "This is a test notification from Video Downloader Tauri!",
    ).await?;

    Ok("Test notification sent successfully!".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Config Defaults ─────────────────────────────

    #[test]
    fn test_default_config() {
        let config = WebhookConfig::default();
        assert!(!config.enabled);
        assert!(config.url.is_empty());
        assert_eq!(config.webhook_type, "discord");
        assert!(config.secret.is_none());
        assert_eq!(config.events, vec!["download_complete"]);
    }

    #[test]
    fn test_config_serialization_roundtrip() {
        let mut config = WebhookConfig::default();
        config.enabled = true;
        config.url = "https://discord.com/webhook/test".to_string();
        config.events = vec!["download_complete".into(), "download_failed".into()];

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WebhookConfig = serde_json::from_str(&json).unwrap();

        assert!(deserialized.enabled);
        assert_eq!(deserialized.url, "https://discord.com/webhook/test");
        assert_eq!(deserialized.events.len(), 2);
    }

    #[test]
    fn test_config_camel_case_serialization() {
        let config = WebhookConfig {
            enabled: true,
            url: "https://example.com".to_string(),
            webhook_type: "custom".to_string(),
            secret: Some("mysecret".to_string()),
            events: vec!["download_complete".to_string()],
        };
        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("webhookType"));
        assert!(json.contains("download_complete"));
    }

    // ─── send_webhook guard clauses (no HTTP needed) ─

    #[tokio::test]
    async fn test_send_webhook_disabled_returns_ok() {
        let config = WebhookConfig {
            enabled: false,
            ..Default::default()
        };
        let result = send_webhook(&config, "download_complete", "Title", "Message").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_webhook_empty_url_returns_ok() {
        let config = WebhookConfig {
            enabled: true,
            url: String::new(),
            ..Default::default()
        };
        let result = send_webhook(&config, "download_complete", "Title", "Message").await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_send_webhook_unsubscribed_event_returns_ok() {
        let config = WebhookConfig {
            enabled: true,
            url: "https://example.com/webhook".to_string(),
            events: vec!["download_complete".to_string()],
            ..Default::default()
        };
        let result = send_webhook(&config, "download_failed", "Title", "Message").await;
        assert!(result.is_ok());
    }

    // ─── send_test_webhook guard clauses ─────────────

    #[tokio::test]
    async fn test_send_test_webhook_disabled_returns_error() {
        let config = WebhookConfig {
            enabled: false,
            ..Default::default()
        };
        let result = send_test_webhook(&config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not enabled"));
    }

    #[tokio::test]
    async fn test_send_test_webhook_empty_url_returns_error() {
        let config = WebhookConfig {
            enabled: true,
            url: String::new(),
            ..Default::default()
        };
        let result = send_test_webhook(&config).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("URL is empty"));
    }

    // ─── Event filtering ─────────────────────────────

    #[test]
    fn test_event_filtering_with_multiple_events() {
        let config = WebhookConfig {
            enabled: true,
            url: "https://example.com".to_string(),
            events: vec![
                "download_complete".to_string(),
                "download_failed".to_string(),
                "new_episode".to_string(),
            ],
            ..Default::default()
        };
        assert!(config.events.contains(&"download_complete".to_string()));
        assert!(config.events.contains(&"download_failed".to_string()));
        assert!(config.events.contains(&"new_episode".to_string()));
        assert!(!config.events.contains(&"test".to_string()));
    }
}
