use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
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

pub struct VideoDownloader {
    client: Client,
    output_dir: PathBuf,
    config: DownloadConfig,
}

impl VideoDownloader {
    pub fn new(output_dir: &str) -> Self {
        Self::with_config(output_dir, DownloadConfig::default())
    }

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

/// Check if FFmpeg is available
pub fn check_ffmpeg() -> bool {
    Command::new("ffmpeg")
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

    let mut concat_content = String::new();
    for file in &video_files {
        concat_content.push_str(&format!("file '{}'\n", file));
    }

    fs::write(&concat_file, &concat_content).map_err(|e| format!("Failed to create concat file: {}", e))?;

    // Run FFmpeg
    let output = Command::new("ffmpeg")
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
        Ok(())
    } else {
        Err(format!(
            "FFmpeg failed: {}",
            String::from_utf8_lossy(&output.stderr)
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
