use headless_chrome::{Browser, LaunchOptions};
use std::time::Duration;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};

/// Auto-detect video URLs using headless Chrome browser
pub struct ChromeVideoDetector {
    browser: Option<Arc<Browser>>,
}

impl ChromeVideoDetector {
    pub fn new() -> Result<Self, String> {
        Ok(Self { browser: None })
    }

    /// Initialize headless Chrome browser
    fn init_browser(&mut self) -> Result<Arc<Browser>, String> {
        if let Some(ref browser) = self.browser {
            return Ok(Arc::clone(browser));
        }

        eprintln!("[ChromeDetector] Launching headless Chrome...");

        let mut args = Vec::new();
        args.push(std::ffi::OsStr::new("--no-sandbox"));
        args.push(std::ffi::OsStr::new("--disable-dev-shm-usage"));
        args.push(std::ffi::OsStr::new("--disable-gpu"));
        args.push(std::ffi::OsStr::new("--disable-software-rasterizer"));
        args.push(std::ffi::OsStr::new("--disable-extensions"));
        args.push(std::ffi::OsStr::new("--window-size=1920,1080"));
        args.push(std::ffi::OsStr::new("--autoplay-policy=no-user-gesture-required"));

        let options = LaunchOptions {
            headless: true,
            sandbox: false,
            args: args,
            ..Default::default()
        };

        let browser = Browser::new(options)
            .map_err(|e| format!("Failed to launch Chrome: {}", e))?;

        let browser = Arc::new(browser);
        self.browser = Some(Arc::clone(&browser));

        eprintln!("[ChromeDetector] Chrome launched successfully");
        Ok(browser)
    }

    /// Helper to emit progress events
    fn emit_progress(&self, app_handle: Option<&AppHandle>, message: &str, progress: u32) {
        if let Some(app) = app_handle {
            let _ = app.emit("detection-progress", serde_json::json!({
                "message": message,
                "progress": progress
            }));
        }
        eprintln!("[ChromeDetector] {} ({}%)", message, progress);
    }

    /// Auto-detect video URL from a webpage
    pub fn detect_video_url(&mut self, url: &str, app_handle: Option<&AppHandle>) -> Result<Option<String>, String> {
        // Javascript to click play buttons, iframes, and center screen
        let click_script = r#"
            (function() {
                console.log("Triggering aggressive click...");

                // 1. Click all potential play buttons
                const selectors = [
                    "button.jw-display-icon-container",
                    ".jw-display-icon-display",
                    ".jw-display-container",
                    "button[aria-label='Play']",
                    "button.play-button",
                    ".video-play-button",
                    "button.vjs-big-play-button",
                    ".plyr__control--overlaid",
                    "div[class*='play']",
                    "button[class*='play']",
                    "a[class*='play']",
                    "img[src*='play']",
                    ".jw-video",
                    ".jw-poster" // Explicitly target poster
                ];

                for (let selector of selectors) {
                    const elements = document.querySelectorAll(selector);
                    for (let el of elements) {
                        if (el && el.offsetParent !== null) { // Check if visible
                            try { el.click(); } catch(e) {}
                        }
                    }
                }

                // 2. Click POSTER images (User specific request)
                const posters = document.querySelectorAll('.jw-poster, [class*="poster"], img[src*="poster"]');
                for (let poster of posters) {
                    try { poster.click(); } catch(e) {}
                }

                // 3. Click all IFRAMES (to activate embedded players)
                const iframes = document.querySelectorAll('iframe');
                for (let iframe of iframes) {
                    try {
                         // Try to click the iframe element itself
                         iframe.click();
                         // Try to focus it
                         iframe.focus();
                    } catch(e) {}
                }

                // 4. Click VIDEO tags directly
                const videos = document.querySelectorAll('video');
                for (let video of videos) {
                    try {
                        video.play();
                        video.click();
                    } catch(e) {}
                }

                // 5. Center screen click (fallback for overlays)
                try {
                    const x = window.innerWidth / 2;
                    const y = window.innerHeight / 2;
                    const el = document.elementFromPoint(x, y);
                    if (el) el.click();
                } catch(e) {}

                return true;
            })()
        "#;

        self.emit_progress(app_handle, "Starting video detection...", 0);

        self.emit_progress(app_handle, "Launching Chrome browser...", 10);

        // Try to initialize browser, retry once if fails
        let browser = match self.init_browser() {
            Ok(b) => b,
            Err(e) => {
                eprintln!("[ChromeDetector] Initial launch failed: {}, retrying...", e);
                self.cleanup();
                self.init_browser()?
            }
        };

        // Create a new tab - with retry logic for "connection closed" errors
        let tab = match browser.new_tab() {
            Ok(t) => t,
            Err(e) => {
                eprintln!("[ChromeDetector] Failed to create tab: {}, restarting browser...", e);
                self.cleanup();
                let browser = self.init_browser()?;
                browser.new_tab().map_err(|e| format!("Failed to create tab after restart: {}", e))?
            }
        };

        // Enable network tracking
        self.emit_progress(app_handle, "Enabling network monitoring...", 20);
        // let _ = tab.enable_fetch(None, None); // CAUSES HANGS if not handled!

        // Attempt to inject script on new document creation to catch early requests
        // using CDP command Page.addScriptToEvaluateOnNewDocument
        // let observer_script_source = r#"
        //    (function() {
        // ... (commented out to remove warning)
        //    })()
        // "#;

        // Try to use Page.addScriptToEvaluateOnNewDocument
        // Note: call_method might return error if not supported, we ignore it
        // We need to construct the params
        // use headless_chrome::protocol::cdp::types::Event;
        // use headless_chrome::protocol::cdp::Page;

        // This is a best-effort attempt. If it fails to compile or run, we fall back to evaluation after load.
        // We'll wrap it in a block to avoid import issues if possible, or just use the evaluation after load as primary.
        // Given we are editing the file, let's stick to the safe approach first: remove enable_fetch.
        // And relying on the script injection we already have (but verify its placement).

        self.emit_progress(app_handle, "Navigating to URL...", 30);

        // Navigate to the page
        tab.navigate_to(url)
            .map_err(|e| format!("Failed to navigate: {}", e))?;

        // Wait for page to load
        self.emit_progress(app_handle, "Waiting for page to load...", 40);
        tab.wait_until_navigated()
            .map_err(|e| format!("Navigation failed: {}", e))?;

        // Inject PerformanceObserver immediately after load
        let observer_script = r#"
            (function() {
                if (!window.__FOUND_URLS) {
                    window.__FOUND_URLS = [];
                    // Check existing entries first
                    performance.getEntriesByType('resource').forEach(e => {
                        if (e.name.includes('.m3u8') || e.name.includes('.mp4') || e.name.includes('video')) {
                            window.__FOUND_URLS.push(e.name);
                        }
                    });

                    // Observe new entries
                    try {
                        const observer = new PerformanceObserver((list) => {
                            list.getEntries().forEach((entry) => {
                                if (entry.name.includes('.m3u8') || entry.name.includes('.mp4') || entry.name.includes('video')) {
                                    window.__FOUND_URLS.push(entry.name);
                                }
                            });
                        });
                        observer.observe({ entryTypes: ['resource'] });
                        console.log("PerformanceObserver attached");
                    } catch(e) {
                        console.error("Observer failed:", e);
                    }
                }
            })()
        "#;
        let _ = tab.evaluate(observer_script, false);

        // 1. Initial wait for basic page load
        self.emit_progress(app_handle, "Waiting for dynamic content...", 50);
        std::thread::sleep(Duration::from_secs(5));

        // POLLING LOOP: Try to find video URL multiple times
        for attempt in 1..=8 { // Increased attempts
            let progress = 50 + (attempt * 5);

            let _ = tab.evaluate(click_script, false);

            // Wait after clicking
            self.emit_progress(app_handle, &format!("Attempt {}/8: Waiting for video load...", attempt), progress as u32 + 4);
            std::thread::sleep(Duration::from_secs(4)); // Increased wait time

            // C. Check again after click
            if let Some(url) = self.check_for_video_url(&tab) {
                self.emit_progress(app_handle, "Video URL found!", 100);
                return Ok(Some(url));
            }
        }

        // D. Recursive Iframe Check (if main page failed)
        self.emit_progress(app_handle, "Main page scan finished. Checking internal iframes...", 95);

        // Extract iframe URLs from the page
        let iframe_script = r#"
            (function() {
                const urls = [];
                document.querySelectorAll('iframe').forEach(i => {
                    if (i.src && i.src.startsWith('http')) urls.push(i.src);
                    else if (i.dataset.src && i.dataset.src.startsWith('http')) urls.push(i.dataset.src);
                    else if (i.dataset.lazySrc && i.dataset.lazySrc.startsWith('http')) urls.push(i.dataset.lazySrc);
                });
                return JSON.stringify(urls);
            })()
        "#;

        if let Ok(result) = tab.evaluate(iframe_script, false) {
             if let Some(json_str) = result.value {
                 if let Ok(iframe_urls) = serde_json::from_value::<Vec<String>>(json_str) {
                     if !iframe_urls.is_empty() {
                         eprintln!("[ChromeDetector] Found {} internal iframes to scan", iframe_urls.len());

                         for (i, iframe_url) in iframe_urls.iter().enumerate() {
                             self.emit_progress(app_handle, &format!("Scanning internal iframe {}/{}...", i+1, iframe_urls.len()), 95);
                             eprintln!("[ChromeDetector] Scanning internal iframe: {}", iframe_url);

                             // Navigate to iframe URL
                             if let Ok(_) = tab.navigate_to(iframe_url) {
                                 let _ = tab.wait_until_navigated();

                                 // Inject observer again for the new page
                                 let _ = tab.evaluate(observer_script, false);

                                 // Wait a bit
                                 std::thread::sleep(Duration::from_secs(3));

                                 // Quick scan (don't do full 8 attempts, just quick check + click)

                                 // 1. Check immediately
                                 if let Some(url) = self.check_for_video_url(&tab) {
                                     self.emit_progress(app_handle, "Video URL found in iframe!", 100);
                                     return Ok(Some(url));
                                 }

                                 // 2. Click (using duplicated script to avoid scope issues)
                                 let click_script_iframe = r#"
                                    (function() {
                                        console.log("Triggering aggressive click in iframe...");
                                        const selectors = [
                                            "button.jw-display-icon-container", ".jw-display-icon-display", ".jw-display-container",
                                            "button[aria-label='Play']", "button.play-button", ".video-play-button",
                                            "button.vjs-big-play-button", ".plyr__control--overlaid",
                                            "div[class*='play']", "button[class*='play']", "a[class*='play']", "img[src*='play']",
                                            ".jw-video", ".jw-poster"
                                        ];
                                        for (let selector of selectors) {
                                            document.querySelectorAll(selector).forEach(el => {
                                                try { el.click(); } catch(e) {}
                                            });
                                        }
                                        document.querySelectorAll('video').forEach(v => { try { v.play(); v.click(); } catch(e) {} });
                                        return true;
                                    })()
                                "#;
                                 let _ = tab.evaluate(click_script_iframe, false);
                                 std::thread::sleep(Duration::from_secs(3));

                                 // 3. Check again
                                 if let Some(url) = self.check_for_video_url(&tab) {
                                     self.emit_progress(app_handle, "Video URL found in iframe!", 100);
                                     return Ok(Some(url));
                                 }
                             }
                         }
                     }
                 }
             }
        }

        self.emit_progress(app_handle, "No video URL found after all attempts", 100);
        Ok(None)
    }

    /// Helper to check for video URLs using multiple methods
    fn check_for_video_url(&self, tab: &headless_chrome::Tab) -> Option<String> {
        // Method 0: Check our injected global variable (PerformanceObserver results)
        let observer_check_code = r#"
            (function() {
                if (window.__FOUND_URLS && window.__FOUND_URLS.length > 0) {
                    // Return the most recent m3u8 if possible
                    const m3u8s = window.__FOUND_URLS.filter(u => u.includes('.m3u8'));
                    if (m3u8s.length > 0) return JSON.stringify([m3u8s[m3u8s.length-1]]);
                    return JSON.stringify(window.__FOUND_URLS);
                }
                return "[]";
            })()
        "#;

        if let Ok(result) = tab.evaluate(observer_check_code, false) {
            if let Some(json_str) = result.value {
                if let Ok(urls) = serde_json::from_value::<Vec<String>>(json_str) {
                    for url in urls {
                        if url.contains(".m3u8") || url.contains(".mp4") {
                            eprintln!("[ChromeDetector] Found URL via PerformanceObserver: {}", url);
                            return Some(url);
                        }
                    }
                }
            }
        }

        // Method 1: Check Performance Entries (Network requests) - Legacy check
        let perf_code = r#"
            (function() {
                const entries = performance.getEntriesByType('resource');
                const videoUrls = entries
                    .filter(e =>
                        e.name.includes('.m3u8') ||
                        e.name.includes('master.m3u8') ||
                        e.name.includes('.mp4') ||
                        e.name.includes('video') // Generic 'video' check for some XHRs
                    )
                    .map(e => e.name);
                return JSON.stringify(videoUrls);
            })()
        "#;

        if let Ok(result) = tab.evaluate(perf_code, false) {
            if let Some(json_str) = result.value {
                if let Ok(urls) = serde_json::from_value::<Vec<String>>(json_str) {
                    for url in urls {
                        // Prioritize m3u8
                        if url.contains(".m3u8") {
                            eprintln!("[ChromeDetector] Found .m3u8 URL via Network: {}", url);
                            return Some(url);
                        }
                        // Accept mp4 if no m3u8 found (will be checked in loop)
                        if url.contains(".mp4") {
                             eprintln!("[ChromeDetector] Found .mp4 URL via Network: {}", url);
                             return Some(url);
                        }
                    }
                }
            }
        }

        // Method 2: Check Video Tags
        let video_tag_code = r#"
            (function() {
                const urls = [];
                document.querySelectorAll('video').forEach(v => {
                    if (v.src) urls.push(v.src);
                    if (v.currentSrc) urls.push(v.currentSrc);
                    v.querySelectorAll('source').forEach(s => {
                        if (s.src) urls.push(s.src);
                    });
                });
                return JSON.stringify(urls);
            })()
        "#;

        if let Ok(result) = tab.evaluate(video_tag_code, false) {
            if let Some(json_str) = result.value {
                if let Ok(urls) = serde_json::from_value::<Vec<String>>(json_str) {
                    for url in urls {
                        if url.contains(".m3u8") || url.contains(".mp4") || url.starts_with("blob:") {
                             eprintln!("[ChromeDetector] Found URL via Video Tag: {}", url);
                             // If it's a blob, we might need to do more, but returning it is better than nothing
                             // ideally we'd need to intercept the fetch for the blob content
                             if !url.starts_with("blob:") {
                                 return Some(url);
                             }
                        }
                    }
                }
            }
        }

        // Method 3: Brute force regex search in HTML
        let html_search_code = r#"
            (function() {
                const html = document.documentElement.outerHTML;
                // Search for m3u8
                let match = html.match(/https?:\\\\/\\\\/[^\"'\\s<>]+\\\\.m3u8[^\"'\\s<>]*/);
                if (match) return match[0];

                // Search for mp4
                match = html.match(/https?:\\\\/\\\\/[^\"'\\s<>]+\\\\.mp4[^\"'\\s<>]*/);
                if (match) return match[0];

                return null;
            })()
        "#;

        if let Ok(result) = tab.evaluate(html_search_code, false) {
            if let Some(value) = result.value {
                if let Ok(url) = serde_json::from_value::<String>(value) {
                    if !url.is_empty() {
                         let clean_url = url.replace("\\/", "/")
                            .replace("\\u0026", "&")
                            .replace("&amp;", "&");
                        eprintln!("[ChromeDetector] Found URL via Regex: {}", clean_url);
                        return Some(clean_url);
                    }
                }
            }
        }

        // Method 4: Search for "asset" and "medias" config in HTML (Python script fallback)
        let config_search_code = r#"
            (function() {
                const html = document.documentElement.outerHTML;
                const assetMatch = html.match(/"asset"\s*:\s*"([^"]+)"/);
                const mediaMatch = html.match(/"medias"\s*:\s*\{[^}]*"original"\s*:\s*"([^"]+)"/);

                if (assetMatch && mediaMatch) {
                    const asset = assetMatch[1];
                    const mediaId = mediaMatch[1];
                    return JSON.stringify({
                        asset: asset,
                        mediaId: mediaId
                    });
                }
                return null;
            })()
        "#;

        if let Ok(result) = tab.evaluate(config_search_code, false) {
            if let Some(value) = result.value {
                if let Ok(config) = serde_json::from_value::<serde_json::Value>(value) {
                    if let (Some(asset), Some(media_id)) = (config["asset"].as_str(), config["mediaId"].as_str()) {
                        // Construct the master.m3u8 URL which is the most common
                        let url = format!("https://{}/hls/{}/master.m3u8", asset, media_id);
                        eprintln!("[ChromeDetector] Constructed URL from page config: {}", url);
                        return Some(url);
                    }
                }
            }
        }

        None
    }

    /// Cleanup browser instance
    pub fn cleanup(&mut self) {
        if self.browser.is_some() {
            eprintln!("[ChromeDetector] Closing browser...");
            self.browser = None;
        }
    }
}

impl Drop for ChromeVideoDetector {
    fn drop(&mut self) {
        self.cleanup();
    }
}
