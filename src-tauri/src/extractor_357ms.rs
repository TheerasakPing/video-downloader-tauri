use anyhow::{anyhow, Context, Result};
use url::Url;
use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::protocol::cdp::Network;
use headless_chrome::protocol::cdp::types::Event;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use std::thread;

pub struct Extractor357ms {
    browser: Browser,
}

#[derive(Debug, Clone)]
pub struct VideoInfo {
    pub m3u8_url: Option<String>,
    pub key_hex: Option<String>,
    pub headers: HashMap<String, String>,
}

impl Extractor357ms {
    pub fn new() -> Result<Self> {
        let options = LaunchOptions {
            headless: true,
            args: vec![
                std::ffi::OsStr::new("--no-sandbox"),
                std::ffi::OsStr::new("--disable-gpu"),
                std::ffi::OsStr::new("--disable-dev-shm-usage"),
                std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
                std::ffi::OsStr::new("--autoplay-policy=no-user-gesture-required"),
            ],
            ..Default::default()
        };

        let browser = Browser::new(options)
            .map_err(|e| anyhow!("Failed to launch browser: {}", e))?;
        
        Ok(Self { browser })
    }

    pub fn extract_video_info(&self, url: &str) -> Result<VideoInfo> {
        eprintln!("[357ms-Rust] Extracting: {}", url);
        let tab = self.browser.new_tab()
            .map_err(|e| anyhow!("Failed to create tab: {}", e))?;

        // Enable network tracking
        tab.call_method(Network::Enable { 
            max_total_buffer_size: None, 
            max_resource_buffer_size: None, 
            max_post_data_size: None,
            enable_durable_messages: None,
            report_direct_socket_traffic: None,
        }).context("Failed to enable network")?;

        // Shared state for captured data
        let mut initial_headers = HashMap::new();
        initial_headers.insert("User-Agent".to_string(), "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36".to_string());
        initial_headers.insert("Referer".to_string(), url.to_string());
        // Parse origin from URL for the Origin header
        if let Ok(u) = Url::parse(url) {
            if let Some(host) = u.host_str() {
                initial_headers.insert("Origin".to_string(), format!("{}://{}", u.scheme(), host));
            }
        }

        let captured_info = Arc::new(Mutex::new(VideoInfo {
            m3u8_url: None,
            key_hex: None,
            headers: initial_headers,
        }));
        
        // Potential key requests to check
        let candidate_keys = Arc::new(Mutex::new(Vec::new()));

        let info_clone = captured_info.clone();
        let candidates_clone = candidate_keys.clone();

        // Register event listener
        tab.add_event_listener(Arc::new(move |event: &Event| {
            if let Event::NetworkResponseReceived(params) = event {
                let url = &params.params.response.url;
                let mime = &params.params.response.mime_type;
                let status = params.params.response.status;

                // 1. Capture m3u8
                if url.contains(".m3u8") {
                    eprintln!("[357ms-Rust] Found m3u8: {}", url);
                    let mut info = info_clone.lock().unwrap();
                    if info.m3u8_url.is_none() {
                        info.m3u8_url = Some(url.clone());
                        // Headers are already initialized with UA and Referer
                    }
                }

                // 2. Identify potential key candidates
                // 357ms keys are often binary (application/octet-stream) or text/plain
                // They are NOT images, css, js, etc.
                if status == 200 {
                    let is_resource = url.ends_with(".png") || url.ends_with(".jpg") || 
                                      url.ends_with(".css") || url.ends_with(".js") ||
                                      url.ends_with(".svg") || url.ends_with(".woff");
                    
                    if !is_resource && (mime.contains("octet") || mime.contains("text") || mime.contains("json") || url.contains("key") || url.contains("config")) {
                        candidates_clone.lock().unwrap().push(params.params.request_id.clone());
                    }
                }
            }
        }))?;

        // Navigate
        tab.navigate_to(url)?;
        // Wait for network idle or just a fixed time
        // tab.wait_until_navigated()?; // sometimes hangs on some sites
        
        // Polling loop
        let start = Instant::now();
        let timeout = Duration::from_secs(20);
        
        while start.elapsed() < timeout {
            thread::sleep(Duration::from_secs(1));
            
            // Check if we found m3u8 and key
            {
                let info = captured_info.lock().unwrap();
                if info.m3u8_url.is_some() && info.key_hex.is_some() {
                    eprintln!("[357ms-Rust] Success: Found m3u8 and key");
                    return Ok(info.clone());
                }
            }
            
            // Check candidates for key
            let candidates: Vec<String> = {
                let mut c = candidate_keys.lock().unwrap();
                let v = c.clone();
                c.clear(); // Clear so we don't re-check
                v
            };

            for request_id in candidates {
                // Try to get body
                // NOTE: This might fail if request is not ready. 
                // We use mapped result to ignore errors efficiently.
                let result = tab.call_method(Network::GetResponseBody { request_id: request_id.clone() });
                
                if let Ok(body_response) = result {
                    // Check if it's the key (16 bytes)
                    let body_bytes = if body_response.base_64_encoded {
                        use base64::{Engine as _, engine::general_purpose};
                        general_purpose::STANDARD.decode(&body_response.body).unwrap_or_default()
                    } else {
                        body_response.body.as_bytes().to_vec()
                    };

                    if body_bytes.len() == 16 {
                        // Manual hex encoding since hex crate is not in dependencies
                        let hex_string: String = body_bytes.iter().map(|b| format!("{:02x}", b)).collect();
                        eprintln!("[357ms-Rust] Found KEY (16 bytes) in request {}", request_id);
                        let mut info = captured_info.lock().unwrap();
                        info.key_hex = Some(hex_string);
                    }
                }
            }
        }

        // Final check
        let info = captured_info.lock().unwrap().clone();
        if info.m3u8_url.is_some() {
            eprintln!("[357ms-Rust] Found m3u8 but NO key (or timeout). Returning partial info.");
            return Ok(info);
        }

        Err(anyhow!("Timeout: Failed to find m3u8 URL"))
    }
}
