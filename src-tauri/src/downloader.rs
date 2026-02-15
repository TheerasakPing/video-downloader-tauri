use crate::utils::sanitize_filename;
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
use std::collections::HashMap;
use crate::python_interface::extract_357ms_video;
use crate::extractor_357ms;

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
        let expanded_dir = crate::utils::expand_path(output_dir);

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


    // Helper to handle 357ms encryption with captured key
    async fn download_encrypted_m3u8(
        &self,
        episode: i32,
        m3u8_url: &str,
        output_path: &PathBuf,
        headers: &HashMap<String, String>,
        key_hex: &str,
        app_handle: &AppHandle,
    ) -> DownloadResult {
        
        let _ = app_handle.emit("log-info", "Starting encrypted download workflow...");

        // 1. Download m3u8 content
        let mut request = self.client.get(m3u8_url);
        for (k, v) in headers {
            request = request.header(k, v);
        }
        
        let m3u8_content = match request.send().await {
            Ok(res) => {
                if !res.status().is_success() {
                    return DownloadResult {
                        episode,
                        success: false,
                        file_path: None,
                        error: Some(format!("Failed to fetch m3u8: {}", res.status())),
                    };
                }
                match res.text().await {
                    Ok(t) => t,
                    Err(e) => return DownloadResult {
                        episode,
                        success: false,
                        file_path: None,
                        error: Some(format!("Failed to read m3u8 text: {}", e)),
                    }
                }
            },
            Err(e) => return DownloadResult {
                episode,
                success: false,
                file_path: None,
                error: Some(format!("Failed to download m3u8: {}", e)),
            }
        };

        // 2. Decode key (Manual hex decode to avoid adding crate)
        let mut key_bytes = Vec::new();
        let mut chars = key_hex.chars();
        while let (Some(h), Some(l)) = (chars.next(), chars.next()) {
            let h = h.to_digit(16);
            let l = l.to_digit(16);
            if let (Some(h), Some(l)) = (h, l) {
                key_bytes.push((h << 4 | l) as u8);
            } else {
                 return DownloadResult {
                    episode,
                    success: false,
                    file_path: None,
                    error: Some("Invalid hex char".to_string()),
                };
            }
        }

        // 3. Create temp dir for this download
        let temp_dir = std::env::temp_dir().join("rongyok_cache").join(format!("ep_{}_{}", episode, std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_micros()));
        if let Err(e) = std::fs::create_dir_all(&temp_dir) {
            return DownloadResult {
                episode,
                success: false,
                file_path: None,
                error: Some(format!("Failed to create temp dir: {}", e)),
            };
        }

        let local_key_path = temp_dir.join("video.key");
        let local_m3u8_path = temp_dir.join("video.m3u8");

        // 4. Write key file
        if let Err(e) = std::fs::write(&local_key_path, &key_bytes) {
             return DownloadResult {
                episode,
                success: false,
                file_path: None,
                error: Some(format!("Failed to output key file: {}", e)),
            };
        }

        // 5. Download segments and rewrite m3u8
        let mut simple_base = m3u8_url.to_string();
        if let Ok(u) = url::Url::parse(m3u8_url) {
             if let Some(_) = u.path_segments() {
                // remove last segment
                let mut u2 = u.clone();
                u2.path_segments_mut().unwrap().pop();
                simple_base = u2.to_string();
                if !simple_base.ends_with('/') {
                    simple_base.push('/');
                }
             }
        }

        let mut new_lines = Vec::new();
        let mut segment_download_tasks = Vec::new();
        let mut segment_count = 0;
        
        // Report start of segment downloading
        let _ = app_handle.emit("log-info", "Downloading encrypted segments...");
        
        for line in m3u8_content.lines() {
            if line.starts_with("#EXT-X-KEY") {
                 if let Some(start) = line.find("URI=\"") {
                     if let Some(end) = line[start+5..].find('"') {
                         let before = &line[..start+5];
                         let after = &line[start+5+end..];
                         // Rewritten to local video.key
                         let new_line = format!("{}video.key{}", before, after); 
                         new_lines.push(new_line);
                     } else {
                         new_lines.push(line.to_string());
                     }
                 } else {
                     new_lines.push(line.to_string());
                 }
            } else if !line.starts_with('#') && !line.trim().is_empty() {
                // This is a segment URL
                let segment_url = if !line.starts_with("http") {
                    format!("{}{}", simple_base, line)
                } else {
                    line.to_string()
                };

                let segment_filename = format!("segment_{:03}.ts", segment_count);
                let segment_path = temp_dir.join(&segment_filename);
                new_lines.push(segment_filename.clone());
                segment_count += 1;

                // Create download task
                let client = self.client.clone();
                let headers_clone = headers.clone();
                let url = segment_url.clone();
                
                segment_download_tasks.push(async move {
                    // Retry logic (3 times)
                    for i in 0..3 {
                        let mut req = client.get(&url);
                        for (k, v) in &headers_clone {
                            req = req.header(k, v);
                        }
                        
                        match req.send().await {
                            Ok(resp) => {
                                if resp.status().is_success() {
                                    match resp.bytes().await {
                                        Ok(bytes) => {
                                            match tokio::fs::write(&segment_path, &bytes).await {
                                                Ok(_) => return Ok(()),
                                                Err(e) => {
                                                    if i == 2 { return Err(format!("Write error: {}", e)); }
                                                }
                                            }
                                        },
                                        Err(e) => {
                                             if i == 2 { return Err(format!("Bytes error: {}", e)); }
                                        }
                                    }
                                } else {
                                    if i == 2 { return Err(format!("HTTP {}", resp.status())); }
                                }
                            },
                            Err(e) => {
                                if i == 2 { return Err(format!("Rewest error: {}", e)); }
                            }
                        }
                        // Wait before retry
                        tokio::time::sleep(Duration::from_millis(500)).await;
                    }
                    Err(format!("Failed to download {}", url))
                });

            } else {
                new_lines.push(line.to_string());
            }
        }
        
        // Execute downloads concurrently
        let stream = futures_util::stream::iter(segment_download_tasks)
            .buffer_unordered(10); // 10 concurrent downloads
        
        let results: Vec<_> = stream.collect().await;
        
        // Check for failures
        let mut fail_count = 0;
        for res in results {
            if let Err(e) = res {
                let _ = app_handle.emit("log-error", format!("Segment download failed: {}", e));
                fail_count += 1;
            }
        }
        
        if fail_count > 0 {
             return DownloadResult {
                episode,
                success: false,
                file_path: None,
                error: Some(format!("Failed to download {} segments", fail_count)),
            };
        }
        
        let new_m3u8_content = new_lines.join("\n");
        if let Err(e) = std::fs::write(&local_m3u8_path, &new_m3u8_content) {
             return DownloadResult {
                episode,
                success: false,
                file_path: None,
                error: Some(format!("Failed to write modified m3u8: {}", e)),
            };
        }

        // 6. Run FFmpeg
        let mut ffmpeg_cmd = get_ffmpeg_command();
        
        ffmpeg_cmd.arg("-y");
        ffmpeg_cmd.args(&["-allowed_extensions", "ALL"]);
        // We still need crypto, file, data for local playback
        ffmpeg_cmd.args(&["-protocol_whitelist", "file,crypto,data"]);
        ffmpeg_cmd.arg("-i").arg(&local_m3u8_path);
        ffmpeg_cmd.args(&["-c", "copy"]);
        ffmpeg_cmd.args(&["-bsf:a", "aac_adtstoasc"]);
        ffmpeg_cmd.arg(output_path);

         let _ = app_handle.emit("log-info", format!("Running FFmpeg (Encrypted) on: {:?}", local_m3u8_path));

        #[cfg(target_os = "windows")]
        use std::os::windows::process::CommandExt;
        #[cfg(target_os = "windows")]
        const CREATE_NO_WINDOW: u32 = 0x08000000;
        #[cfg(target_os = "windows")]
        ffmpeg_cmd.creation_flags(CREATE_NO_WINDOW);

        let child = match ffmpeg_cmd.stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped())
            .spawn() {
                Ok(c) => c,
                Err(e) => return DownloadResult {
                    episode,
                    success: false,
                    file_path: None,
                    error: Some(format!("Failed to spawn FFmpeg: {}", e)),
                }
            };

        // Wait for FFmpeg 
        let result = self.wait_for_ffmpeg(child, episode, app_handle.clone(), None, output_path.to_string_lossy().to_string()).await;
        
        // Cleanup temp dir
        let _ = std::fs::remove_dir_all(&temp_dir);
        
        result
    }

    pub async fn download_episode(
        &self,
        episode: i32,
        video_url: &str,
        app_handle: &AppHandle,
        download_state: Option<Arc<DownloadState>>,
    ) -> DownloadResult {
        let file_path = self.get_episode_filename(episode);
        let file_path_str = file_path.to_string_lossy().to_string();

        let mut final_url = video_url.to_string();
        let mut custom_headers = None;

        // Check for 357ms
        if video_url.contains("357ms.com") || video_url.contains("357") {
             let _ = app_handle.emit("log-info", format!("Detecting 357ms video (Rust Native) for URL: {}", video_url));
             
             let url_clone = video_url.to_string();
             let extraction_task = tokio::task::spawn_blocking(move || {
                 match crate::extractor_357ms::Extractor357ms::new() {
                     Ok(extractor) => extractor.extract_video_info(&url_clone),
                     Err(e) => Err(anyhow::anyhow!("Failed to init extractor: {}", e))
                 }
             });

             match extraction_task.await {
                 Ok(Ok(info)) => {
                     if let Some(m3u8) = info.m3u8_url {
                         final_url = m3u8;
                         custom_headers = Some(info.headers.clone());
                         let _ = app_handle.emit("log-info", format!("357ms Rust extraction success: {}", final_url));

                         if let Some(k_hex) = info.key_hex {
                            let _ = app_handle.emit("log-info", format!("Got 357ms key (hex): {}", k_hex));
                            // Use special handling for encrypted 357ms
                            return self.download_encrypted_m3u8(
                                episode,
                                &final_url,
                                &file_path,
                                &info.headers,
                                &k_hex,
                                app_handle
                            ).await;
                         }
                     } else {
                         return DownloadResult {
                            episode,
                            success: false,
                            file_path: None,
                            error: Some("357ms extraction returned no m3u8 URL".to_string()),
                        };
                     }
                 },
                 Ok(Err(e)) => {
                      let _ = app_handle.emit("log-error", format!("357ms Rust extraction failed: {}", e));
                      return DownloadResult {
                        episode,
                        success: false,
                        file_path: None,
                        error: Some(format!("357ms extraction failed: {}", e)),
                    };
                 },
                 Err(e) => {
                      return DownloadResult {
                        episode,
                        success: false,
                        file_path: None,
                        error: Some(format!("Task join error: {}", e)),
                    };
                 }
             }
        }

        // Check if it's an HLS stream (m3u8) - require FFmpeg
        if final_url.contains(".m3u8") {
            if !check_ffmpeg() {
                return DownloadResult {
                    episode,
                    success: false,
                    file_path: None,
                    error: Some("FFmpeg is required for .m3u8 downloads but was not found".to_string()),
                };
            }
            return self.download_hls_stream(episode, &final_url, &file_path_str, app_handle, download_state, custom_headers).await;
        }

        // Direct file download (MP4, etc) using Reqwest
        self.download_direct_file(episode, &final_url, &file_path, app_handle, download_state).await
    }

    // Helper for downloading HLS streams with FFmpeg
    async fn download_hls_stream(
        &self,
        episode: i32,
        video_url: &str,
        file_path: &str,
        app_handle: &AppHandle,
        download_state: Option<Arc<DownloadState>>,
        custom_headers: Option<HashMap<String, String>>,
    ) -> DownloadResult {
        let mut cmd = get_ffmpeg_command();

        // Add headers to mimic a browser + generic referer
        let mut headers_str = "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string();
        let mut user_agent = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string();

        if let Some(h) = custom_headers {
            headers_str = String::new();
            for (k, v) in h {
                if k.eq_ignore_ascii_case("user-agent") {
                    user_agent = v.clone();
                }
                headers_str.push_str(&format!("{}: {}\r\n", k, v));
            }
        }

        cmd.args([
            "-y",
            "-user_agent", &user_agent, // Explicitly set UA
            "-headers", &headers_str,
            "-rw_timeout", "30000000", // 30s timeout to prevent hanging
            "-i", video_url,
            "-c", "copy",
            // Network resilience flags
            "-reconnect", "1",
            "-reconnect_at_eof", "1",
            "-reconnect_streamed", "1",
            "-reconnect_delay_max", "2",
            // "-bsf:a", "aac_adtstoasc", // Only if needed
            file_path,
        ]);

        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::null());

        let cmd_str = format!("{:?} {:?}", cmd.get_program(), cmd.get_args());
        let _ = app_handle.emit("log-info", format!("Running FFmpeg: {}", cmd_str));
        eprintln!("[Downloader] Running FFmpeg: {}", cmd_str);

        let child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return DownloadResult {
                episode,
                success: false,
                file_path: None,
                error: Some(format!("Failed to start FFmpeg: {}", e)),
            },
        };

        self.wait_for_ffmpeg(child, episode, app_handle.clone(), download_state, file_path.to_string()).await
    }

    // Helper to monitor FFmpeg process
    async fn wait_for_ffmpeg(
        &self,
        mut child: std::process::Child,
        episode: i32,
        app_handle: AppHandle,
        download_state: Option<Arc<DownloadState>>,
        file_path_str: String,
    ) -> DownloadResult {
        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);
        // Capture last error line
        
        // We need to read lines in a non-blocking way or spawn a thread.
        // Since we are in async function, blocking on reader.lines() is bad if it blocks.
        // But BufReader on ChildStderr blocks.
        // We should run this in spawn_blocking?
        // Or use tokio::process::Command?
        // The current implementation uses std::process::Command.
        // So we MUST use spawn_blocking for the waiting loop.
        
        let file_path_clone = file_path_str.clone();
        
        let result = tokio::task::spawn_blocking(move || {
            let mut reader = reader;
            let mut total_duration = 0.0;
            let mut last_emit = std::time::Instant::now();
            let mut last_error_line = String::new();
            
            for line in reader.lines() {
                // Check cancellation
                if let Some(ref state) = download_state {
                    if state.is_cancelled.load(Ordering::SeqCst) {
                        let _ = child.kill();
                        let _ = fs::remove_file(&file_path_clone);
                        return DownloadResult {
                            episode,
                            success: false,
                            file_path: None,
                            error: Some("Download cancelled".to_string()),
                        };
                    }
                }

                if let Ok(line) = line {
                    // Parse Duration if not yet found
                    if total_duration == 0.0 && line.contains("Duration:") {
                        if let Some(start) = line.find("Duration: ") {
                            let content = &line[start + 10..]; // Skip "Duration: "
                            if let Some(end) = content.find(',') {
                                let time_str = &content[..end];
                                let parts: Vec<&str> = time_str.split(':').collect();
                                if parts.len() == 3 {
                                    let h: f64 = parts[0].parse().unwrap_or(0.0);
                                    let m: f64 = parts[1].parse().unwrap_or(0.0);
                                    let s: f64 = parts[2].parse().unwrap_or(0.0);
                                    total_duration = h * 3600.0 + m * 60.0 + s;
                                    let _ = app_handle.emit("log-info", format!("Total Duration parsed: {}s", total_duration));
                                }
                            }
                        }
                    }

                    // Log the first few lines to see what's happening
                    if last_emit.elapsed().as_secs() < 5 {
                         let _ = app_handle.emit("log-info", format!("FFmpeg output: {}", line));
                         eprintln!("[FFmpeg Output] {}", line);
                    }
                    
                    // Keep track of potential error messages
                    if line.contains("Error") || line.contains("Failed") || line.contains("Invalid") {
                        last_error_line = line.clone();
                        let _ = app_handle.emit("log-info", format!("FFmpeg error found: {}", line));
                        eprintln!("[FFmpeg Error] {}", line);
                    }

                    if let Some(stats) = parse_ffmpeg_stats(&line) {
                        if last_emit.elapsed().as_millis() >= 500 {
                            let speed_bps = stats.bitrate_kbps * 1024.0 / 8.0; // Convert kbps to Bytes/s roughly
                            
                            let percentage = if total_duration > 0.0 {
                                (stats.time_seconds / total_duration * 100.0).min(99.9)
                            } else {
                                0.0
                            };

                            let progress = DownloadProgress {
                                episode,
                                downloaded: stats.size_bytes,
                                total: 0, // Unknown for HLS usually
                                speed: speed_bps,
                                percentage, 
                            };
                            let _ = app_handle.emit("download-progress", progress);
                            last_emit = std::time::Instant::now();
                        }
                    }
                }
            }

            match child.wait() {
                Ok(status) => {
                    if status.success() {
                        DownloadResult {
                            episode,
                            success: true,
                            file_path: Some(file_path_clone),
                            error: None,
                        }
                    } else {
                        let err_msg = if !last_error_line.is_empty() {
                            format!("FFmpeg exited with error: {}", last_error_line)
                        } else {
                            "FFmpeg exited with error (unknown cause)".to_string()
                        };

                        DownloadResult {
                            episode,
                            success: false,
                            file_path: None,
                            error: Some(err_msg),
                        }
                    }
                },
                Err(e) => DownloadResult {
                    episode,
                    success: false,
                    file_path: None,
                    error: Some(format!("Failed to wait for FFmpeg: {}", e)),
                }
            }
        }).await;
        
        match result {
            Ok(res) => res,
            Err(e) => DownloadResult {
                episode,
                success: false,
                file_path: None,
                error: Some(format!("Join error: {}", e)),
            }
        }
    }

    // Existing logic moved to helper method
    async fn download_direct_file(
        &self,
        episode: i32,
        video_url: &str,
        file_path: &PathBuf,
        app_handle: &AppHandle,
        download_state: Option<Arc<DownloadState>>,
    ) -> DownloadResult {
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
        let file = match if start_byte > 0 {
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
        
        // Use BufWriter to reduce syscalls and improve disk I/O performance
        // This makes a HUGE difference for high-speed downloads
        let mut writer = std::io::BufWriter::with_capacity(1024 * 1024, file); // 1MB buffer

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
                    if let Err(e) = writer.write_all(&chunk) {
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
        
        // Flush buffer at the end
        if let Err(e) = writer.flush() {
             return DownloadResult {
                episode,
                success: false,
                file_path: None,
                error: Some(format!("Failed to flush file: {}", e)),
            };
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
            {
                let path = p.join("ffmpeg-x86_64-pc-windows-msvc.exe");
                if path.exists() {
                    return path;
                }
                return p.join("ffmpeg.exe");
            }
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
#[allow(dead_code)]
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
struct FFmpegStats {
    time_seconds: f64,
    size_bytes: u64,
    bitrate_kbps: f64,
}

/// Parse FFmpeg progress output to extract stats
fn parse_ffmpeg_stats(line: &str) -> Option<FFmpegStats> {
    // Example: size=    9728KiB time=00:00:29.56 bitrate=2695.9kbits/s speed= 8.4x
    
    let mut stats = FFmpegStats {
        time_seconds: 0.0,
        size_bytes: 0,
        bitrate_kbps: 0.0,
    };
    let mut found = false;

    // Parse time
    if let Some(time_start) = line.find("time=") {
        let time_str = &line[time_start + 5..];
        if let Some(end) = time_str.find(' ') {
            let time_part = &time_str[..end];
            let parts: Vec<&str> = time_part.split(':').collect();
            if parts.len() == 3 {
                let hours: f64 = parts[0].parse().unwrap_or(0.0);
                let minutes: f64 = parts[1].parse().unwrap_or(0.0);
                let seconds: f64 = parts[2].parse().unwrap_or(0.0);
                stats.time_seconds = hours * 3600.0 + minutes * 60.0 + seconds;
                found = true;
            }
        }
    }

    // Parse size
    if let Some(size_start) = line.find("size=") {
        let size_str = &line[size_start + 5..];
        if let Some(end) = size_str.find(' ') {
            let val_str = size_str[..end].trim();
            // Check unit
            if let Some(num_str) = val_str.strip_suffix("KiB") {
                if let Ok(val) = num_str.trim().parse::<f64>() {
                    stats.size_bytes = (val * 1024.0) as u64;
                    found = true;
                }
            } else if let Some(num_str) = val_str.strip_suffix("MiB") {
                if let Ok(val) = num_str.trim().parse::<f64>() {
                    stats.size_bytes = (val * 1024.0 * 1024.0) as u64;
                    found = true;
                }
            } else if let Some(num_str) = val_str.strip_suffix("kB") {
                 if let Ok(val) = num_str.trim().parse::<f64>() {
                    stats.size_bytes = (val * 1000.0) as u64;
                    found = true;
                }
            } else if let Some(num_str) = val_str.strip_suffix("MB") {
                 if let Ok(val) = num_str.trim().parse::<f64>() {
                    stats.size_bytes = (val * 1000.0 * 1000.0) as u64;
                    found = true;
                }
            } else if let Ok(val) = val_str.parse::<u64>() {
                 // Assume bytes if no unit? FFmpeg usually has unit.
                 stats.size_bytes = val;
                 found = true;
            }
        }
    }

    // Parse bitrate (kbits/s)
    if let Some(br_start) = line.find("bitrate=") {
        let br_str = &line[br_start + 8..];
        if let Some(end) = br_str.find("kbits/s") {
             if let Ok(val) = br_str[..end].trim().parse::<f64>() {
                 stats.bitrate_kbps = val;
             }
        }
    }

    if found {
        Some(stats)
    } else {
        None
    }
}

/// Helper for legacy time parsing if needed (kept for compatibility or remove/rename)
fn parse_ffmpeg_time_legacy(line: &str) -> Option<f64> {
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
            if let Some(stats) = parse_ffmpeg_stats(&line) {
                let current_time = stats.time_seconds;
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
            if let Some(current_time) = parse_ffmpeg_time_legacy(&line) {
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

// Sanitize filename helper moved to utils.rs
