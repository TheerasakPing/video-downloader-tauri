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

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityOption {
    pub resolution: String,
    pub bandwidth: u64,
    pub label: String,
    pub stream_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QualityInfo {
    pub qualities: Vec<QualityOption>,
    pub default_index: usize,
}

/// Parse HLS master playlist to extract quality variants.
/// Returns variants sorted by bandwidth (highest first).
pub fn parse_master_playlist(master_text: &str, master_url: &str) -> Vec<QualityOption> {
    let base_url = master_url.rfind('/').map(|i| &master_url[..=i]).unwrap_or(master_url);
    let mut variants: Vec<QualityOption> = Vec::new();
    let mut current_bandwidth: i64 = -1;
    let mut current_resolution: Option<String> = None;

    for line in master_text.lines() {
        let line = line.trim();
        if line.starts_with("#EXT-X-STREAM-INF:") {
            current_bandwidth = line.split(',')
                .find_map(|part| {
                    part.trim().strip_prefix("BANDWIDTH=")
                        .and_then(|v| v.parse::<i64>().ok())
                })
                .unwrap_or(0);
            current_resolution = line.split(',')
                .find_map(|part| {
                    part.trim().strip_prefix("RESOLUTION=")
                        .map(|v| v.to_string())
                });
        } else if !line.is_empty() && !line.starts_with('#') && current_bandwidth >= 0 {
            let sub_url = if line.starts_with("http") {
                line.to_string()
            } else {
                format!("{}/{}", base_url.trim_end_matches('/'), line.trim_start_matches('/'))
            };
            let resolution = current_resolution.clone().unwrap_or_else(|| "unknown".to_string());
            let bandwidth = current_bandwidth as u64;
            let height = resolution.split('x').nth(1).unwrap_or("?");
            let label = format!("{}p ({:.1} Mbps)", height, bandwidth as f64 / 1_000_000.0);
            variants.push(QualityOption {
                resolution,
                bandwidth,
                label,
                stream_url: sub_url,
            });
            current_bandwidth = -1;
            current_resolution = None;
        }
    }

    variants.sort_by(|a, b| b.bandwidth.cmp(&a.bandwidth));
    variants
}

/// Retry wrapper for fallible async operations.
async fn retry_request<F, Fut, T>(
    max_retries: u32,
    retry_delay_ms: u64,
    f: F,
) -> Result<T, String>
where
    F: Fn() -> Fut,
    Fut: std::future::Future<Output = Result<T, String>>,
{
    let mut last_error = String::new();
    for attempt in 0..=max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) => {
                last_error = e;
                if attempt < max_retries {
                    tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms)).await;
                }
            }
        }
    }
    Err(last_error)
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

    pub async fn download_episode(
        &self,
        episode: i32,
        video_url: &str,
        hls_key_info: Option<crate::titan_parser::HlsKeyInfo>,
        referer: Option<&str>,
        cookies: &[(String, String)],
        app_handle: &AppHandle,
        download_state: Option<Arc<DownloadState>>,
        preferred_quality: Option<String>,
    ) -> DownloadResult {
        let file_path = self.get_episode_filename(episode);
        let file_path_str = file_path.to_string_lossy().to_string();

        // Titan encrypted HLS: download + decrypt manually
        if let Some(key_info) = hls_key_info {
            return self.download_titan_hls(episode, video_url, &key_info, &file_path_str, app_handle, download_state).await;
        }

        // Standard HLS stream (m3u8 without encryption metadata)
        if video_url.contains(".m3u8") {
            if !check_ffmpeg() {
                return DownloadResult {
                    episode,
                    success: false,
                    file_path: None,
                    error: Some("FFmpeg is required for .m3u8 downloads but was not found".to_string()),
                };
            }
            // Pre-resolve master playlist if quality is specified
            let effective_url = if let Some(ref quality) = preferred_quality {
                let pre_client = Client::builder()
                    .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36")
                    .build().unwrap_or_else(|_| Client::new());
                if let Ok(resp) = pre_client.get(video_url).send().await {
                    if let Ok(text) = resp.text().await {
                        if text.contains("#EXT-X-STREAM-INF") {
                            let variants = parse_master_playlist(&text, video_url);
                            let resolved = variants.iter()
                                .find(|v| v.resolution == *quality)
                                .map(|v| v.stream_url.clone())
                                .unwrap_or_else(|| video_url.to_string());
                            resolved
                        } else {
                            video_url.to_string()
                        }
                    } else {
                        video_url.to_string()
                    }
                } else {
                    video_url.to_string()
                }
            } else {
                video_url.to_string()
            };

            let result = self.download_hls_stream(episode, &effective_url, &file_path_str, referer, cookies, app_handle, download_state.clone()).await;

            // If FFmpeg failed due to obfuscated segment extensions (.jpeg, .html, etc.),
            // fall back to manual segment download via reqwest + FFmpeg concat
            if !result.success {
                if let Some(ref err) = result.error {
                    let err_lower = err.to_lowercase();
                    if err_lower.contains("empty segment")
                        || err_lower.contains("allowed_segment_extensions")
                        || err_lower.contains("allowed_extensions")
                        || err_lower.contains("mismatches")
                        || err_lower.contains("invalid data found")
                    {
                        let _ = app_handle.emit("log-info", "FFmpeg HLS demuxer rejected segments — retrying with manual segment download...".to_string());
                        eprintln!("[Downloader] Retrying with manual segment download for: {}", effective_url);
                        return self.download_hls_manual(episode, &effective_url, &file_path_str, referer, cookies, app_handle, download_state, preferred_quality).await;
                    }
                }
            }

            return result;
        }

        // Direct file download (MP4, etc) using Reqwest
        self.download_direct_file(episode, video_url, &file_path, app_handle, download_state).await
    }

    /// Download and decrypt AES-128-CBC encrypted HLS stream (Titan/357ms.com pattern).
    ///
    /// Strategy:
    /// 1. Fetch variant M3U8 (H.264 v1) from `hls_base_url/v1/index.m3u8`
    /// 2. Download each .ts segment using reqwest with proper Referer
    /// 3. Decrypt each segment with AES-128-CBC (key from API)
    /// 4. Concatenate decrypted segments into .ts file
    /// 5. Use FFmpeg to convert the .ts → .mp4
    async fn download_titan_hls(
        &self,
        episode: i32,
        _stream_url: &str,
        key_info: &crate::titan_parser::HlsKeyInfo,
        output_path: &str,
        app_handle: &AppHandle,
        download_state: Option<Arc<DownloadState>>,
    ) -> DownloadResult {
        use aes::cipher::{BlockDecryptMut, KeyIvInit, block_padding::NoPadding};
        use base64::Engine as _;

        type Aes128CbcDec = cbc::Decryptor<aes::Aes128>;

        let referer = Self::derive_referer_from_hls_url(&key_info.hls_base_url);

        // Build reqwest client with Referer
        let client = Client::builder()
            .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36")
            .build()
            .unwrap_or_default();

        // ── Decode key + IV ─────────────────────────────────────────────
        let key_bytes = match base64::engine::general_purpose::STANDARD.decode(&key_info.key_b64) {
            Ok(b) if b.len() == 16 => b,
            _ => {
                return DownloadResult {
                    episode,
                    success: false,
                    file_path: None,
                    error: Some("Invalid HLS key (must be 16 bytes after base64 decode)".to_string()),
                };
            }
        };
        let iv_bytes = match hex::decode(&key_info.iv_hex) {
            Ok(b) if b.len() == 16 => b,
            _ => {
                return DownloadResult {
                    episode,
                    success: false,
                    file_path: None,
                    error: Some(format!("Invalid HLS IV (must be 32 hex chars): {}", key_info.iv_hex)),
                };
            }
        };
        let key_arr: [u8; 16] = key_bytes.try_into().unwrap();
        let iv_arr:  [u8; 16] = iv_bytes.try_into().unwrap();

        // ── Fetch H.264 variant playlist ────────────────────────────────
        let v1_m3u8_url = format!("{}/v1/index.m3u8", key_info.hls_base_url);
        eprintln!("[Titan] Fetching variant playlist: {}", v1_m3u8_url);
        let _ = app_handle.emit("log-info", format!("[EP {}] Fetching playlist...", episode));

        let v1_content = match client.get(&v1_m3u8_url)
            .header("Referer", &referer)
            .send().await
            .and_then(|r| Ok(r))
        {
            Ok(resp) => match resp.text().await {
                Ok(t) => t,
                Err(e) => return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("Failed to read variant M3U8: {}", e)),
                },
            },
            Err(e) => return DownloadResult {
                episode, success: false, file_path: None,
                error: Some(format!("Failed to fetch variant M3U8: {}", e)),
            },
        };

        // Extract segment names (skip # comment lines)
        let seg_base = format!("{}/v1/", key_info.hls_base_url);
        let segments: Vec<String> = v1_content.lines()
            .map(|l| l.trim())
            .filter(|l| !l.is_empty() && !l.starts_with('#'))
            .map(|seg| {
                if seg.starts_with("http") {
                    seg.to_string()
                } else {
                    format!("{}{}", seg_base, seg)
                }
            })
            .collect();

        if segments.is_empty() {
            return DownloadResult {
                episode, success: false, file_path: None,
                error: Some("No segments found in variant M3U8".to_string()),
            };
        }
        eprintln!("[Titan] EP {} - {} segments to download", episode, segments.len());
        let _ = app_handle.emit("log-info", format!("[EP {}] Downloading {} segments...", episode, segments.len()));

        // ── Download + decrypt segments ──────────────────────────────────
        let ts_path = output_path.replace(".mp4", ".tmp.ts");
        let mut ts_file = match std::fs::File::create(&ts_path) {
            Ok(f) => f,
            Err(e) => return DownloadResult {
                episode, success: false, file_path: None,
                error: Some(format!("Cannot create temp TS file: {}", e)),
            },
        };

        let total_segments = segments.len();
        let start_time = std::time::Instant::now();
        let mut total_bytes: u64 = 0;
        for (i, seg_url) in segments.iter().enumerate() {
            // Check cancellation
            if let Some(ref ds) = download_state {
                if ds.is_cancelled.load(Ordering::Relaxed) {
                    let _ = std::fs::remove_file(&ts_path);
                    return DownloadResult {
                        episode, success: false, file_path: None,
                        error: Some("Download cancelled".to_string()),
                    };
                }
            }

            eprintln!("[Titan] EP {} seg {}/{}", episode, i + 1, total_segments);
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                total_bytes as f64 / elapsed
            } else {
                0.0
            };
            let _ = app_handle.emit("download-progress", crate::downloader::DownloadProgress {
                episode,
                downloaded: (i as u64 * 100) / total_segments as u64,
                total: 100,
                speed,
                percentage: (i as f64 / total_segments as f64) * 100.0,
            });

            let encrypted = match client.get(seg_url)
                .header("Referer", &referer)
                .send().await
            {
                Ok(resp) => match resp.bytes().await {
                    Ok(b) => {
                        total_bytes += b.len() as u64;
                        b
                    },
                    Err(e) => return DownloadResult {
                        episode, success: false, file_path: None,
                        error: Some(format!("Failed to read segment {}: {}", i, e)),
                    },
                },
                Err(e) => return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("Failed to download segment {}: {}", i, e)),
                },
            };

            if encrypted.is_empty() {
                continue;
            }

            // AES-128-CBC decrypt (no padding removal — TS segments are block-aligned)
            let mut buf = encrypted.to_vec();
            // Pad to 16-byte boundary if necessary
            let rem = buf.len() % 16;
            if rem != 0 {
                buf.resize(buf.len() + (16 - rem), 0);
            }

            let decryptor = Aes128CbcDec::new(&key_arr.into(), &iv_arr.into());
            match decryptor.decrypt_padded_mut::<NoPadding>(&mut buf) {
                Ok(decrypted) => {
                    if let Err(e) = ts_file.write_all(decrypted) {
                        return DownloadResult {
                            episode, success: false, file_path: None,
                            error: Some(format!("Write error seg {}: {}", i, e)),
                        };
                    }
                }
                Err(e) => return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("Decrypt error seg {}: {:?}", i, e)),
                },
            }
        }
        drop(ts_file); // flush + close

        let _ = app_handle.emit("log-info", format!("[EP {}] Segments downloaded, converting to MP4...", episode));

        // ── FFmpeg: TS → MP4 ────────────────────────────────────────────
        if !check_ffmpeg() {
            return DownloadResult {
                episode, success: false, file_path: None,
                error: Some("FFmpeg not found for TS→MP4 conversion".to_string()),
            };
        }

        let mut cmd = get_ffmpeg_command();
        cmd.args([
            "-y",
            "-i", &ts_path,
            "-c", "copy",
            "-movflags", "+faststart",
            output_path,
        ]);
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = std::fs::remove_file(&ts_path);
                return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("FFmpeg spawn error: {}", e)),
                };
            }
        };

        let status = child.wait().unwrap_or_else(|_| std::process::ExitStatus::default());
        let _ = std::fs::remove_file(&ts_path); // cleanup temp

        if status.success() {
            let elapsed = start_time.elapsed().as_secs_f64();
            let final_speed = if elapsed > 0.0 {
                total_bytes as f64 / elapsed
            } else {
                0.0
            };
            let _ = app_handle.emit("download-progress", crate::downloader::DownloadProgress {
                episode,
                downloaded: total_segments as u64,
                total: total_segments as u64,
                speed: final_speed,
                percentage: 100.0,
            });
            let _ = app_handle.emit("log-info", format!("[EP {}] Download complete!", episode));
            eprintln!("[Titan] EP {} -> SUCCESS: {}", episode, output_path);
            DownloadResult {
                episode,
                success: true,
                file_path: Some(output_path.to_string()),
                error: None,
            }
        } else {
            DownloadResult {
                episode, success: false, file_path: None,
                error: Some(format!("FFmpeg conversion failed (exit: {:?})", status.code())),
            }
        }
    }

    /// Auto-derive a Referer origin URL from an HLS stream URL.
    ///
    /// Rules:
    /// - Strip the scheme+host, remove `hls.` prefix if present, add `www.` if no subdomain
    /// - Always returns an `https://` origin (no path)
    ///
    /// Examples:
    ///   `https://hls.357ms.com/series_360/ep_1/master.m3u8` → `https://www.357ms.com`
    ///   `https://cdn.rongyok.com/...`                        → `https://www.rongyok.com`
    ///   `https://stream.example.com/...`                     → `https://www.example.com`
    fn derive_referer_from_hls_url(hls_url: &str) -> String {
        // Extract host from URL
        let host = hls_url
            .trim_start_matches("https://")
            .trim_start_matches("http://")
            .split('/')
            .next()
            .unwrap_or("");

        if host.is_empty() {
            return "https://www.example.com".to_string();
        }

        // Remove port if present (host:port)
        let host = host.split(':').next().unwrap_or(host);

        // Count dots to check if there's a subdomain
        let dot_count = host.chars().filter(|&c| c == '.').count();

        let referer_host = if dot_count >= 2 {
            // Has subdomain — replace leftmost part with "www"
            // e.g. hls.357ms.com → www.357ms.com
            let rest = host.splitn(2, '.').nth(1).unwrap_or(host);
            format!("www.{}", rest)
        } else {
            // No subdomain — prepend www
            format!("www.{}", host)
        };

        format!("https://{}", referer_host)
    }

    // Helper for downloading HLS streams with FFmpeg
    async fn download_hls_stream(
        &self,
        episode: i32,
        video_url: &str,
        file_path: &str,
        referer: Option<&str>,
        cookies: &[(String, String)],
        app_handle: &AppHandle,
        download_state: Option<Arc<DownloadState>>,
    ) -> DownloadResult {
        let mut cmd = get_ffmpeg_command();

        let effective_referer = referer
            .map(|s| s.to_string())
            .unwrap_or_else(|| Self::derive_referer_from_hls_url(video_url));
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

        let mut headers = format!(
            "User-Agent: {}\r\nReferer: {}\r\n",
            ua, effective_referer
        );

        if !cookies.is_empty() {
            let cookie_header: String = cookies.iter()
                .map(|(k, v)| format!("{}={}", k, v))
                .collect::<Vec<_>>()
                .join("; ");
            headers.push_str(&format!("Cookie: {}\r\n", cookie_header));
        }

        cmd.args([
            "-y",
            "-headers", &headers,
            "-rw_timeout", "30000000", // 30s timeout to prevent hanging
            "-allowed_extensions", "ALL", // Allow all playlist extensions
            "-allowed_segment_extensions", "ALL", // FFmpeg 8.x: separate check for segment extensions (.jpeg, etc.)
            "-i", video_url,
            "-c", "copy",
            // Network resilience flags
            "-reconnect", "1",
            "-reconnect_at_eof", "1",
            "-reconnect_streamed", "1",
            "-reconnect_delay_max", "2",
            file_path,
        ]);


        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::null());

        let cmd_str = format!("{:?} {:?}", cmd.get_program(), cmd.get_args());
        let _ = app_handle.emit("log-info", format!("Running FFmpeg: {}", cmd_str));
        eprintln!("[Downloader] Running FFmpeg: {}", cmd_str);

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => return DownloadResult {
                episode,
                success: false,
                file_path: None,
                error: Some(format!("Failed to start FFmpeg: {}", e)),
            },
        };

        let stderr = child.stderr.take().unwrap();
        let reader = BufReader::new(stderr);
        let mut last_emit = std::time::Instant::now();

        // Capture last error line
        let mut last_error_line = String::new();

        let mut total_duration: f64 = 0.0;

        // Processing loop
        for line in reader.lines() {
            // Check cancellation
            if let Some(ref state) = download_state {
                if state.is_cancelled.load(Ordering::SeqCst) {
                    let _ = child.kill();
                    let _ = fs::remove_file(file_path);
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
                // Also capture "mismatches" lines (FFmpeg 8.x format consistency check)
                if line.contains("Error") || line.contains("Failed") || line.contains("Invalid") || line.contains("mismatches") {
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
                        file_path: Some(file_path.to_string()),
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
    }

    /// Manual HLS segment downloader — bypasses FFmpeg's HLS demuxer entirely.
    /// Downloads each segment via reqwest (with proper Referer/UA headers),
    /// writes them concatenated into a temp .ts file, then uses FFmpeg to convert
    /// the raw MPEG-TS data into the final .mp4 output.
    ///
    /// This is needed for servers that disguise .ts segments as .jpeg/.html/etc.
    /// to prevent standard download tools from recognizing them.
    async fn download_hls_manual(
        &self,
        episode: i32,
        master_url: &str,
        output_path: &str,
        referer: Option<&str>,
        cookies: &[(String, String)],
        app_handle: &AppHandle,
        download_state: Option<Arc<DownloadState>>,
        preferred_quality: Option<String>,
    ) -> DownloadResult {
        let _ = app_handle.emit("log-info", format!("Manual HLS: fetching master playlist..."));
        eprintln!("[ManualHLS] Fetching master playlist: {}", master_url);

        let effective_referer = referer
            .map(|s| s.to_string())
            .unwrap_or_else(|| Self::derive_referer_from_hls_url(master_url));
        let ua = "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36";

        // Build HTTP client for segment downloads
        let seg_client = Client::builder()
            .user_agent(ua)
            .default_headers({
                let mut headers = reqwest::header::HeaderMap::new();
                headers.insert("Referer", effective_referer.parse().unwrap());
                if !cookies.is_empty() {
                    let cookie_header = cookies
                        .iter()
                        .map(|(k, v)| format!("{}={}", k, v))
                        .collect::<Vec<_>>()
                        .join("; ");
                    if let Ok(cookie_val) = cookie_header.parse() {
                        headers.insert("Cookie", cookie_val);
                    }
                }
                headers
            })
            .build()
            .unwrap_or_else(|_| Client::new());

        // 1. Fetch master playlist
        let master_text = match seg_client.get(master_url).send().await {
            Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
            Ok(resp) => {
                return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("Master playlist returned HTTP {}", resp.status())),
                };
            }
            Err(e) => {
                return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("Failed to fetch master playlist: {}", e)),
                };
            }
        };

        // 2. Parse master playlist and select quality variant
        let variants = parse_master_playlist(&master_text, master_url);

        let sub_playlist_url = if let Some(ref quality) = preferred_quality {
            variants.iter()
                .find(|v| v.resolution == *quality)
                .map(|v| v.stream_url.clone())
                .unwrap_or_else(|| {
                    // Fallback: nearest lower quality
                    let target = quality.split('x').nth(1)
                        .and_then(|h| h.parse::<u32>().ok())
                        .unwrap_or(0);
                    variants.iter()
                        .filter(|v| {
                            v.resolution.split('x').nth(1)
                                .and_then(|h| h.parse::<u32>().ok())
                                .unwrap_or(0) <= target
                        })
                        .map(|v| v.stream_url.clone())
                        .next()
                        .unwrap_or_else(|| variants.first().map(|v| v.stream_url.clone()).unwrap_or_else(|| master_url.to_string()))
                })
        } else {
            variants.first().map(|v| v.stream_url.clone()).unwrap_or_else(|| master_url.to_string())
        };
        let _ = app_handle.emit("log-info", format!("Manual HLS: using sub-playlist: {}", sub_playlist_url));
        eprintln!("[ManualHLS] Sub-playlist URL: {}", sub_playlist_url);

        // 3. Fetch sub-playlist and extract segment URLs
        let sub_text = match seg_client.get(&sub_playlist_url).send().await {
            Ok(resp) if resp.status().is_success() => resp.text().await.unwrap_or_default(),
            Ok(resp) => {
                return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("Sub-playlist returned HTTP {}", resp.status())),
                };
            }
            Err(e) => {
                return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("Failed to fetch sub-playlist: {}", e)),
                };
            }
        };

        let sub_base_url = sub_playlist_url.rfind('/').map(|i| &sub_playlist_url[..=i]).unwrap_or(&sub_playlist_url);
        
        // Better segment extraction: only lines following #EXTINF
        let mut segment_urls = Vec::new();
        let mut lines_iter = sub_text.lines().peekable();
        while let Some(line) = lines_iter.next() {
            let line = line.trim();
            if line.starts_with("#EXTINF:") {
                // Find next non-empty, non-comment line
                while let Some(next_line) = lines_iter.peek() {
                    let next_line = next_line.trim();
                    if next_line.is_empty() {
                        lines_iter.next();
                        continue;
                    }
                    if !next_line.starts_with('#') {
                        let seg_url = if next_line.starts_with("http") {
                            next_line.to_string()
                        } else {
                            format!("{}/{}", sub_base_url.trim_end_matches('/'), next_line.trim_start_matches('/'))
                        };
                        segment_urls.push(seg_url);
                        lines_iter.next();
                        break;
                    }
                    // If we hit another tag before a URL, this EXTINF was invalid or empty
                    if next_line.starts_with('#') {
                        break;
                    }
                }
            }
        }

        if segment_urls.is_empty() {
            return DownloadResult {
                episode, success: false, file_path: None,
                error: Some("No segments found in HLS playlist".to_string()),
            };
        }

        let total_segments = segment_urls.len();
        let _ = app_handle.emit("log-info", format!("Manual HLS: found {} segments", total_segments));
        eprintln!("[ManualHLS] Downloading {} segments", total_segments);

        // 4. Create temp .ts file and download segments into it
        let ts_temp_path = format!("{}.ts", output_path.trim_end_matches(".mp4"));
        let mut ts_file = match File::create(&ts_temp_path) {
            Ok(f) => f,
            Err(e) => {
                return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("Failed to create temp .ts file: {}", e)),
                };
            }
        };

        let start_time = std::time::Instant::now();
        let mut total_bytes: u64 = 0;

        for (i, seg_url) in segment_urls.iter().enumerate() {
            // Check cancellation
            if let Some(ref state) = download_state {
                if state.is_cancelled.load(Ordering::SeqCst) {
                    let _ = fs::remove_file(&ts_temp_path);
                    return DownloadResult {
                        episode, success: false, file_path: None,
                        error: Some("Download cancelled".to_string()),
                    };
                }
            }

            // Download segment with retry
            let seg_url_clone = seg_url.clone();
            let segment_data = retry_request(3, 2000, || {
                let client = seg_client.clone();
                let url = seg_url_clone.clone();
                async move {
                    let resp = client.get(&url).send().await.map_err(|e| e.to_string())?;
                    if resp.status().is_success() {
                        resp.bytes().await.map_err(|e| e.to_string()).map(|b| b.to_vec())
                    } else {
                        Err(format!("HTTP {}", resp.status()))
                    }
                }
            }).await;

            let data = match segment_data {
                Ok(data) => data,
                Err(e) => {
                    let _ = app_handle.emit("log-warning", format!("Segment {} failed after retries: {}", seg_url, e));
                    continue;
                }
            };

            {
                let seg_size = data.len() as u64;
                total_bytes += seg_size;
                if let Err(e) = ts_file.write_all(&data) {
                    let _ = fs::remove_file(&ts_temp_path);
                    return DownloadResult {
                        episode, success: false, file_path: None,
                        error: Some(format!("Failed to write segment: {}", e)),
                    };
                }
            }

            // Emit progress with speed
            let elapsed = start_time.elapsed().as_secs_f64();
            let speed = if elapsed > 0.0 {
                total_bytes as f64 / elapsed
            } else {
                0.0
            };
            let percentage = ((i + 1) as f64 / total_segments as f64) * 90.0; // Reserve last 10% for FFmpeg
            if let Some(ref _state) = download_state {
                let progress = DownloadProgress {
                    episode,
                    downloaded: (i + 1) as u64,
                    total: total_segments as u64,
                    speed,
                    percentage,
                };
                let _ = app_handle.emit("download-progress", progress);
            }
        }

        // Flush and close the .ts file
        if let Err(e) = ts_file.flush() {
            let _ = fs::remove_file(&ts_temp_path);
            return DownloadResult {
                episode, success: false, file_path: None,
                error: Some(format!("Failed to flush .ts file: {}", e)),
            };
        }
        drop(ts_file);

        let _ = app_handle.emit("log-info", "Manual HLS: segments downloaded, converting to MP4...".to_string());

        // 5. Convert .ts → .mp4 with FFmpeg (just remux, no re-encoding)
        let mut cmd = get_ffmpeg_command();
        cmd.args([
            "-y",
            "-i", &ts_temp_path,
            "-c", "copy",
            output_path,
        ]);
        cmd.stderr(Stdio::piped());
        cmd.stdout(Stdio::null());

        let mut child = match cmd.spawn() {
            Ok(c) => c,
            Err(e) => {
                let _ = fs::remove_file(&ts_temp_path);
                return DownloadResult {
                    episode, success: false, file_path: None,
                    error: Some(format!("Failed to start FFmpeg for .ts conversion: {}", e)),
                };
            }
        };

        let status = child.wait().unwrap_or_else(|_| std::process::ExitStatus::default());

        // Clean up temp .ts file
        let _ = fs::remove_file(&ts_temp_path);

        if status.success() {
            // Final progress
            if let Some(ref _state) = download_state {
                let elapsed = start_time.elapsed().as_secs_f64();
                let final_speed = if elapsed > 0.0 {
                    total_bytes as f64 / elapsed
                } else {
                    0.0
                };
                let progress = DownloadProgress {
                    episode,
                    downloaded: total_segments as u64,
                    total: total_segments as u64,
                    speed: final_speed,
                    percentage: 100.0,
                };
                let _ = app_handle.emit("download-progress", progress);
            }

            DownloadResult {
                episode,
                success: true,
                file_path: Some(output_path.to_string()),
                error: None,
            }
        } else {
            DownloadResult {
                episode, success: false, file_path: None,
                error: Some("FFmpeg failed to convert .ts to .mp4".to_string()),
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
