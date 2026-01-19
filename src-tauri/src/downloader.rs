use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::time::{sleep, Duration};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub episode: i32,
    pub downloaded: u64,
    pub total: u64,
    pub speed: f64,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadResult {
    pub episode: i32,
    pub success: bool,
    pub file_path: Option<String>,
    pub error: Option<String>,
}

#[derive(Clone)]
pub struct DownloadConfig {
    pub speed_limit_kbps: i32,  // 0 = unlimited
    pub file_naming: String,    // "ep_001", "episode_1", "title_ep1"
    pub series_title: String,
}

impl Default for DownloadConfig {
    fn default() -> Self {
        Self {
            speed_limit_kbps: 0,
            file_naming: "ep_001".to_string(),
            series_title: "".to_string(),
        }
    }
}

/// Shared state for tracking download pause/cancel status
pub struct DownloadState {
    pub is_paused: Arc<AtomicBool>,
    pub is_cancelled: Arc<AtomicBool>,
}

impl DownloadState {
    pub fn new() -> Self {
        Self {
            is_paused: Arc::new(AtomicBool::new(false)),
            is_cancelled: Arc::new(AtomicBool::new(false)),
        }
    }
}

pub struct VideoDownloader {
    client: Client,
    output_dir: PathBuf,
    config: DownloadConfig,
}

impl VideoDownloader {
    pub fn with_config(output_dir: &str, config: DownloadConfig) -> Self {
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
            .build()
            .expect("Failed to create HTTP client");

        // Expand ~ to home directory
        let expanded_dir = if output_dir.starts_with("~/") {
            if let Some(home) = dirs::home_dir() {
                home.join(&output_dir[2..])
            } else {
                PathBuf::from(output_dir)
            }
        } else {
            PathBuf::from(output_dir)
        };

        fs::create_dir_all(&expanded_dir).ok();

        Self {
            client,
            output_dir: expanded_dir,
            config,
        }
    }

    pub fn get_episode_filename(&self, episode: i32) -> PathBuf {
        let filename = match self.config.file_naming.as_str() {
            "episode_1" => format!("episode_{}.mp4", episode),
            "title_ep1" => {
                let title = sanitize_filename(&self.config.series_title);
                if title.is_empty() {
                    format!("EP{}.mp4", episode)
                } else {
                    format!("{}_EP{}.mp4", title, episode)
                }
            }
            _ => format!("ep_{:03}.mp4", episode), // Default: ep_001
        };
        self.output_dir.join(filename)
    }

    pub async fn download_episode(
        &self,
        episode: i32,
        video_url: &str,
        app_handle: &AppHandle,
        download_state: Option<Arc<DownloadState>>,
    ) -> DownloadResult {
        let file_path = self.get_episode_filename(episode);

        // Check for existing partial download
        let mut start_byte: u64 = 0;
        if file_path.exists() {
            if let Ok(metadata) = fs::metadata(&file_path) {
                start_byte = metadata.len();
            }
        }

        // Build request with Range header for resume
        let mut request = self.client.get(video_url);
        if start_byte > 0 {
            request = request.header("Range", format!("bytes={}-", start_byte));
        }

        let response = match request.send().await {
            Ok(r) => r,
            Err(e) => {
                return DownloadResult {
                    episode,
                    success: false,
                    file_path: None,
                    error: Some(format!("Request failed: {}", e)),
                };
            }
        };

        // Get content length
        let total_size = if start_byte > 0 {
            response
                .content_length()
                .map(|cl| cl + start_byte)
                .unwrap_or(0)
        } else {
            response.content_length().unwrap_or(0)
        };

        // Check if server returned 416 (Range Not Satisfiable) - file is complete
        if response.status().as_u16() == 416 {
            return DownloadResult {
                episode,
                success: true,
                file_path: Some(file_path.to_string_lossy().to_string()),
                error: None,
            };
        }

        // Open file for writing
        let mut file = match if start_byte > 0 {
            fs::OpenOptions::new().append(true).open(&file_path)
        } else {
            File::create(&file_path)
        } {
            Ok(f) => f,
            Err(e) => {
                return DownloadResult {
                    episode,
                    success: false,
                    file_path: None,
                    error: Some(format!("Failed to create file: {}", e)),
                };
            }
        };

        let mut downloaded = start_byte;
        let mut stream = response.bytes_stream();
        let start_time = std::time::Instant::now();
        let mut last_emit = std::time::Instant::now();

        // Speed limiting variables
        let speed_limit_bytes = if self.config.speed_limit_kbps > 0 {
            (self.config.speed_limit_kbps as u64) * 1024
        } else {
            0
        };
        let mut interval_downloaded: u64 = 0;
        let mut interval_start = std::time::Instant::now();

        while let Some(chunk_result) = stream.next().await {
            // Check for pause request
            if let Some(ref state) = download_state {
                if state.is_cancelled.load(Ordering::SeqCst) {
                    // Clean up partial file on cancel
                    let _ = fs::remove_file(&file_path);
                    return DownloadResult {
                        episode,
                        success: false,
                        file_path: None,
                        error: Some("Download cancelled".to_string()),
                    };
                }

                while state.is_paused.load(Ordering::SeqCst) {
                    // Check for cancel while paused
                    if state.is_cancelled.load(Ordering::SeqCst) {
                        let _ = fs::remove_file(&file_path);
                        return DownloadResult {
                            episode,
                            success: false,
                            file_path: None,
                            error: Some("Download cancelled".to_string()),
                        };
                    }
                    sleep(Duration::from_millis(100)).await;
                }
            }

            match chunk_result {
                Ok(chunk) => {
                    if let Err(e) = file.write_all(&chunk) {
                        return DownloadResult {
                            episode,
                            success: false,
                            file_path: None,
                            error: Some(format!("Write failed: {}", e)),
                        };
                    }

                    downloaded += chunk.len() as u64;
                    interval_downloaded += chunk.len() as u64;

                    // Speed limiting
                    if speed_limit_bytes > 0 {
                        let interval_elapsed = interval_start.elapsed().as_secs_f64();
                        if interval_elapsed > 0.0 {
                            let current_speed = interval_downloaded as f64 / interval_elapsed;
                            if current_speed > speed_limit_bytes as f64 {
                                // Calculate how long we should wait to achieve the target speed
                                let target_time = interval_downloaded as f64 / speed_limit_bytes as f64;
                                let sleep_time = target_time - interval_elapsed;
                                if sleep_time > 0.0 {
                                    sleep(Duration::from_secs_f64(sleep_time)).await;
                                }
                            }
                        }
                        // Reset interval every second
                        if interval_start.elapsed().as_secs() >= 1 {
                            interval_downloaded = 0;
                            interval_start = std::time::Instant::now();
                        }
                    }

                    // Emit progress every 100ms
                    if last_emit.elapsed().as_millis() >= 100 {
                        let elapsed = start_time.elapsed().as_secs_f64();
                        let speed = if elapsed > 0.0 {
                            (downloaded - start_byte) as f64 / elapsed
                        } else {
                            0.0
                        };

                        let percentage = if total_size > 0 {
                            (downloaded as f64 / total_size as f64) * 100.0
                        } else {
                            0.0
                        };

                        let progress = DownloadProgress {
                            episode,
                            downloaded,
                            total: total_size,
                            speed,
                            percentage,
                        };

                        let _ = app_handle.emit("download-progress", progress);
                        last_emit = std::time::Instant::now();
                    }
                }
                Err(e) => {
                    return DownloadResult {
                        episode,
                        success: false,
                        file_path: Some(file_path.to_string_lossy().to_string()),
                        error: Some(format!("Download stream error: {}", e)),
                    };
                }
            }
        }

        DownloadResult {
            episode,
            success: true,
            file_path: Some(file_path.to_string_lossy().to_string()),
            error: None,
        }
    }
}

/// Get FFmpeg command - tries bundled first, then system
pub fn get_ffmpeg_command() -> Command {
    // Try bundled FFmpeg first (from resources)
    #[cfg(target_os = "macos")]
    let ffmpeg_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("..").join("Resources").join("ffmpeg")))
        .filter(|p| p.exists());

    #[cfg(target_os = "linux")]
    let ffmpeg_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("resources").join("ffmpeg")))
        .filter(|p| p.exists());

    #[cfg(target_os = "windows")]
    let ffmpeg_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("resources").join("ffmpeg.exe")))
        .filter(|p| p.exists());

    if let Some(path) = ffmpeg_path {
        Command::new(path)
    } else {
        // Fall back to system ffmpeg
        Command::new("ffmpeg")
    }
}

/// Check if FFmpeg is available
pub fn check_ffmpeg() -> bool {
    get_ffmpeg_command()
        .arg("-version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Merge videos using FFmpeg
pub fn merge_videos(video_files: Vec<String>, output_path: &str) -> Result<(), String> {
    if video_files.len() < 2 {
        return Err("Need at least 2 videos to merge".to_string());
    }

    // Create concat file
    let output_dir = std::path::Path::new(output_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));
    let concat_file = output_dir.join("concat_list.txt");

    // Build concat content with properly escaped paths
    // FFmpeg concat demuxer requires absolute paths or paths relative to concat file
    let mut concat_content = String::new();
    for file in &video_files {
        // Use absolute paths and escape single quotes in file paths
        let abs_path = std::fs::canonicalize(file)
            .map_err(|e| format!("Cannot find file {}: {}", file, e))?;
        let path_str = abs_path.to_string_lossy();
        // Escape single quotes by doubling them (FFmpeg requirement)
        let escaped = path_str.replace('\'', "''");
        concat_content.push_str(&format!("file '{}'\n", escaped));
    }

    fs::write(&concat_file, &concat_content)
        .map_err(|e| format!("Failed to create concat file: {}", e))?;

    // First try with -c copy (fast, no re-encoding)
    let output = get_ffmpeg_command()
        .args([
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            concat_file.to_str().unwrap(),
            "-c",
            "copy",
            output_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

    // Clean up concat file
    fs::remove_file(&concat_file).ok();

    if output.status.success() {
        return Ok(());
    }

    // If copy failed, try re-encoding (slower but more compatible)
    let stderr = String::from_utf8_lossy(&output.stderr);
    println!("FFmpeg copy failed: {}, trying re-encoding...", stderr);

    // Re-create concat file for re-encoding attempt
    fs::write(&concat_file, &concat_content)
        .map_err(|e| format!("Failed to recreate concat file: {}", e))?;

    let reencode_output = get_ffmpeg_command()
        .args([
            "-y",
            "-f",
            "concat",
            "-safe",
            "0",
            "-i",
            concat_file.to_str().unwrap(),
            "-c",
            "libx264",
            "-preset",
            "fast",
            "-crf",
            "22",
            output_path,
        ])
        .output()
        .map_err(|e| format!("Failed to run FFmpeg (reencode): {}", e))?;

    // Clean up concat file
    fs::remove_file(&concat_file).ok();

    if reencode_output.status.success() {
        Ok(())
    } else {
        let reencode_stderr = String::from_utf8_lossy(&reencode_output.stderr);
        Err(format!(
            "FFmpeg failed (both copy and re-encoding): {}",
            reencode_stderr
        ))
    }
}

/// Sanitize filename - handle UTF-8 properly
pub fn sanitize_filename(name: &str) -> String {
    let re = regex::Regex::new(r#"[<>:"/\\|?*]"#).unwrap();
    let clean = re.replace_all(name, "");
    let clean = clean.trim();

    // Use chars() to properly handle UTF-8 instead of byte slicing
    let chars: Vec<char> = clean.chars().collect();
    if chars.len() > 50 {
        chars[..50].iter().collect()
    } else {
        clean.to_string()
    }
}
