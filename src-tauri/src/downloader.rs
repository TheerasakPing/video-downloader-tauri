use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Command, Stdio};
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
pub struct MergeProgress {
    pub percentage: f64,
    pub current_time: f64,
    pub total_duration: f64,
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

/// Get FFmpeg command - tries bundled sidecar first, then Resources folder, then system
pub fn get_ffmpeg_command() -> Command {
    // Try sidecar binary first (externalBin puts binaries next to the executable)
    let sidecar_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| {
            #[cfg(target_os = "windows")]
            return p.join("ffmpeg.exe");
            #[cfg(not(target_os = "windows"))]
            return p.join("ffmpeg");
        }))
        .filter(|p| p.exists());

    if let Some(path) = sidecar_path {
        return Command::new(path);
    }

    // Try bundled FFmpeg in Resources folder (legacy location)
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

/// Get FFprobe command - tries bundled sidecar first, then Resources folder, then system
fn get_ffprobe_command() -> Command {
    // Try sidecar binary first (externalBin puts binaries next to the executable)
    let sidecar_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| {
            #[cfg(target_os = "windows")]
            return p.join("ffprobe.exe");
            #[cfg(not(target_os = "windows"))]
            return p.join("ffprobe");
        }))
        .filter(|p| p.exists());

    if let Some(path) = sidecar_path {
        return Command::new(path);
    }

    // Try bundled FFprobe in Resources folder (legacy location)
    #[cfg(target_os = "macos")]
    let ffprobe_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("..").join("Resources").join("ffprobe")))
        .filter(|p| p.exists());

    #[cfg(target_os = "linux")]
    let ffprobe_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("resources").join("ffprobe")))
        .filter(|p| p.exists());

    #[cfg(target_os = "windows")]
    let ffprobe_path = std::env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|p| p.join("resources").join("ffprobe.exe")))
        .filter(|p| p.exists());

    if let Some(path) = ffprobe_path {
        Command::new(path)
    } else {
        // Fall back to system ffprobe
        Command::new("ffprobe")
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

/// Validate a video file by checking if it exists and is a valid video
/// Returns true if the file appears valid
fn validate_video_file(path: &str) -> bool {
    let path = std::path::Path::new(path);
    if !path.exists() {
        return false;
    }

    // Check file size - valid MP4 files should be at least 1KB
    if let Ok(metadata) = std::fs::metadata(path) {
        let size = metadata.len();
        if size < 1024 {
            // File is too small to be a valid video
            return false;
        }
    } else {
        return false;
    }

    // Use ffprobe to validate the video file structure
    // This catches files with missing moov atoms (incomplete downloads)
    let path_str = if let Some(s) = path.to_str() {
        s.to_string()
    } else {
        path.to_string_lossy().to_string()
    };
    let output = get_ffprobe_command()
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", &path_str])
        .output();

    match output {
        Ok(result) => {
            // Check if ffprobe succeeded and returned a valid duration
            if result.status.success() {
                let stdout = String::from_utf8_lossy(&result.stdout);
                // Parse the duration - if it's a positive number, the file is valid
                stdout.trim().parse::<f64>().map_or(false, |d| d > 0.0)
            } else {
                // ffprobe failed - file is likely corrupted
                false
            }
        }
        Err(_) => false
    }
}

/// Merge videos using FFmpeg
pub fn merge_videos(video_files: Vec<String>, output_path: &str) -> Result<(), String> {
    merge_videos_with_progress(video_files, output_path, None)
}

/// Get video duration using ffprobe
fn get_video_duration(path: &str) -> Option<f64> {
    let output = get_ffprobe_command()
        .args(["-v", "error", "-show_entries", "format=duration", "-of", "default=noprint_wrappers=1:nokey=1", path])
        .output()
        .ok()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        stdout.trim().parse::<f64>().ok()
    } else {
        None
    }
}

/// Parse FFmpeg progress output to extract time in seconds
fn parse_ffmpeg_time(line: &str) -> Option<f64> {
    // FFmpeg outputs progress in format: time=00:01:23.45
    if let Some(time_start) = line.find("time=") {
        let time_str = &line[time_start + 5..];
        if let Some(end) = time_str.find(' ') {
            let time_part = &time_str[..end];
            // Parse HH:MM:SS.ms format
            let parts: Vec<&str> = time_part.split(':').collect();
            if parts.len() == 3 {
                let hours: f64 = parts[0].parse().unwrap_or(0.0);
                let minutes: f64 = parts[1].parse().unwrap_or(0.0);
                let seconds: f64 = parts[2].parse().unwrap_or(0.0);
                return Some(hours * 3600.0 + minutes * 60.0 + seconds);
            }
        }
    }
    None
}

/// Merge videos using FFmpeg with progress reporting
pub fn merge_videos_with_progress(video_files: Vec<String>, output_path: &str, app_handle: Option<&AppHandle>) -> Result<(), String> {
    if video_files.is_empty() {
        return Err("No videos to merge".to_string());
    }

    // Filter out invalid files
    let valid_files: Vec<&String> = video_files.iter()
        .filter(|f| validate_video_file(f))
        .collect();

    if valid_files.is_empty() {
        return Err("No valid video files to merge - all files appear incomplete or corrupted".to_string());
    }

    if valid_files.len() != video_files.len() {
        eprintln!("Warning: {} out of {} files were invalid and skipped", video_files.len() - valid_files.len(), video_files.len());
        if let Some(app) = app_handle {
            let _ = app.emit("log-info", format!("Warning: {} files skipped (corrupted/incomplete)", video_files.len() - valid_files.len()));
        }
    }

    // For single valid file, just copy it
    if valid_files.len() == 1 {
        if let Some(app) = app_handle {
            let _ = app.emit("merge-progress", MergeProgress {
                percentage: 100.0,
                current_time: 0.0,
                total_duration: 0.0,
            });
        }
        let source = std::path::Path::new(valid_files[0]);
        let dest = std::path::Path::new(output_path);
        std::fs::copy(source, dest)
            .map_err(|e| format!("Failed to copy file: {}", e))?;
        return Ok(());
    }

    // Calculate total duration for progress calculation
    let total_duration: f64 = valid_files.iter()
        .filter_map(|f| get_video_duration(f))
        .sum();

    if let Some(app) = app_handle {
        let _ = app.emit("log-info", format!("Total video duration: {:.1}s ({} files)", total_duration, valid_files.len()));
    }

    // Create concat list file for demuxer (much faster than filter - no re-encoding)
    let list_path = std::env::temp_dir().join("ffmpeg_concat_list.txt");
    let mut list_content = String::new();
    for file in valid_files.iter() {
        let abs_path = std::fs::canonicalize(file)
            .map_err(|e| format!("Cannot find file {}: {}", file, e))?;
        // Escape single quotes for FFmpeg concat list format
        let escaped_path = abs_path.to_string_lossy().replace("'", "'\\''");
        list_content.push_str(&format!("file '{}'\n", escaped_path));
    }
    std::fs::write(&list_path, &list_content)
        .map_err(|e| format!("Failed to create concat list: {}", e))?;

    // Build command using concat demuxer (stream copy - no re-encoding = FAST)
    let mut cmd = get_ffmpeg_command();
    cmd.args([
        "-y",
        "-f", "concat",
        "-safe", "0",
        "-i", list_path.to_str().unwrap_or(""),
        "-c", "copy",  // Stream copy - no re-encoding!
        output_path,
    ]);

    // Spawn process and read stderr for progress
    cmd.stderr(Stdio::piped());
    cmd.stdout(Stdio::null());

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

    let stderr = child.stderr.take()
        .ok_or_else(|| "Failed to capture FFmpeg stderr".to_string())?;

    let reader = BufReader::new(stderr);
    let mut last_emit = std::time::Instant::now();

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(current_time) = parse_ffmpeg_time(&line) {
                let percentage = if total_duration > 0.0 {
                    (current_time / total_duration * 100.0).min(100.0)
                } else {
                    0.0
                };

                // Emit progress every 500ms to reduce overhead
                if last_emit.elapsed().as_millis() >= 500 {
                    if let Some(app) = app_handle {
                        let _ = app.emit("merge-progress", MergeProgress {
                            percentage,
                            current_time,
                            total_duration,
                        });
                    }
                    last_emit = std::time::Instant::now();
                }
            }
        }
    }

    // Clean up temp file
    std::fs::remove_file(&list_path).ok();

    let status = child.wait()
        .map_err(|e| format!("FFmpeg process error: {}", e))?;

    if status.success() {
        // Emit 100% completion
        if let Some(app) = app_handle {
            let _ = app.emit("merge-progress", MergeProgress {
                percentage: 100.0,
                current_time: total_duration,
                total_duration,
            });
        }
        Ok(())
    } else {
        // If concat demuxer fails (incompatible formats), fall back to concat filter with re-encoding
        if let Some(app) = app_handle {
            let _ = app.emit("log-info", "Stream copy failed, trying re-encode method...".to_string());
        }
        merge_videos_reencode(video_files, output_path, app_handle)
    }
}

/// Fallback merge using re-encoding (slower but handles incompatible formats)
fn merge_videos_reencode(video_files: Vec<String>, output_path: &str, app_handle: Option<&AppHandle>) -> Result<(), String> {
    let valid_files: Vec<&String> = video_files.iter()
        .filter(|f| validate_video_file(f))
        .collect();

    if valid_files.is_empty() {
        return Err("No valid video files to merge".to_string());
    }

    let total_duration: f64 = valid_files.iter()
        .filter_map(|f| get_video_duration(f))
        .sum();

    let mut inputs = Vec::new();
    for file in valid_files.iter() {
        let abs_path = std::fs::canonicalize(file)
            .map_err(|e| format!("Cannot find file {}: {}", file, e))?;
        inputs.extend(["-i".to_string(), abs_path.to_string_lossy().to_string()]);
    }

    // Build concat filter
    let filter_parts: Vec<String> = (0..valid_files.len())
        .map(|i| format!("[{}:v][{}:a]", i, i))
        .collect();
    let filter_inputs = format!(
        "{}concat=n={}:v=1:a=1[outv][outa]",
        filter_parts.join(""),
        valid_files.len()
    );

    let mut cmd = get_ffmpeg_command();
    cmd.args(["-y"]);
    cmd.args(&inputs);
    cmd.args([
        "-filter_complex", &filter_inputs,
        "-map", "[outv]",
        "-map", "[outa]",
        "-c:v", "libx264",
        "-preset", "ultrafast",  // Use ultrafast for speed
        "-crf", "23",
        "-c:a", "aac",
        "-b:a", "128k",
        output_path,
    ]);

    cmd.stderr(Stdio::piped());
    cmd.stdout(Stdio::null());

    let mut child = cmd.spawn()
        .map_err(|e| format!("Failed to run FFmpeg: {}", e))?;

    let stderr = child.stderr.take()
        .ok_or_else(|| "Failed to capture FFmpeg stderr".to_string())?;

    let reader = BufReader::new(stderr);
    let mut last_emit = std::time::Instant::now();

    for line in reader.lines() {
        if let Ok(line) = line {
            if let Some(current_time) = parse_ffmpeg_time(&line) {
                let percentage = if total_duration > 0.0 {
                    (current_time / total_duration * 100.0).min(100.0)
                } else {
                    0.0
                };

                if last_emit.elapsed().as_millis() >= 500 {
                    if let Some(app) = app_handle {
                        let _ = app.emit("merge-progress", MergeProgress {
                            percentage,
                            current_time,
                            total_duration,
                        });
                    }
                    last_emit = std::time::Instant::now();
                }
            }
        }
    }

    let status = child.wait()
        .map_err(|e| format!("FFmpeg process error: {}", e))?;

    if status.success() {
        if let Some(app) = app_handle {
            let _ = app.emit("merge-progress", MergeProgress {
                percentage: 100.0,
                current_time: total_duration,
                total_duration,
            });
        }
        Ok(())
    } else {
        Err(format!(
            "FFmpeg merge failed. The following files may be corrupted/incomplete: {:?}",
            valid_files.iter().map(|s| std::path::Path::new(s).file_name().and_then(|n| n.to_str()).unwrap_or("")).collect::<Vec<_>>()
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
