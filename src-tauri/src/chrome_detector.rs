use headless_chrome::{Browser, LaunchOptions};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Auto-detect video URLs using headless Chrome browser
pub struct ChromeVideoDetector {
    browser: Option<Arc<Browser>>,
    /// Store cookies from last detection for authentication
    last_cookies: Vec<(String, String)>,
    /// Last detected page title (best effort)
    last_title: Option<String>,
    /// Last detected poster/thumbnail URL (best effort)
    last_poster_url: Option<String>,
}

impl ChromeVideoDetector {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            browser: None,
            last_cookies: Vec::new(),
            last_title: None,
            last_poster_url: None,
        })
    }

    fn extract_cookies(&mut self, tab: &headless_chrome::Tab) {
        if let Ok(cookies) = tab.get_cookies() {
            let pairs: Vec<(String, String)> = cookies
                .into_iter()
                .map(|c| (c.name, c.value))
                .collect();
            if !pairs.is_empty() {
                eprintln!(
                    "[ChromeDetector] Extracted {} cookies: {:?}",
                    pairs.len(),
                    pairs.iter().map(|(n, _)| n).collect::<Vec<_>>()
                );
                self.last_cookies = pairs;
            }
        }
    }

    /// Get cookies from last detection
    pub fn get_last_cookies(&self) -> &[(String, String)] {
        &self.last_cookies
    }

    /// Get last captured page title
    pub fn get_last_title(&self) -> Option<&str> {
        self.last_title.as_deref()
    }

    /// Get last captured poster/thumbnail URL
    pub fn get_last_poster_url(&self) -> Option<&str> {
        self.last_poster_url.as_deref()
    }

    /// Capture title/poster metadata from the currently loaded page.
    /// This is best-effort and never fails the detection flow.
    fn capture_page_metadata(&mut self, tab: &headless_chrome::Tab) {
        let metadata_script = r#"
            (function() {
                const get = (sel, attr = "content") => {
                    const el = document.querySelector(sel);
                    if (!el) return "";
                    if (attr === "text") return (el.textContent || "").trim();
                    return (el.getAttribute(attr) || "").trim();
                };

                const title =
                    get("meta[property='og:title']") ||
                    get("meta[name='twitter:title']") ||
                    get("h1", "text") ||
                    (document.title || "").trim();

                const poster =
                    get("meta[property='og:image']") ||
                    get("meta[name='twitter:image']") ||
                    get("video", "poster") ||
                    get("img[src*='cover']", "src") ||
                    get("img[src*='poster']", "src") ||
                    get("img", "src");

                return { title, poster };
            })()
        "#;

        if let Ok(result) = tab.evaluate(metadata_script, false) {
            if let Some(value) = result.value {
                if let Ok(meta) = serde_json::from_value::<serde_json::Value>(value) {
                    if let Some(title) = meta.get("title").and_then(|v| v.as_str()) {
                        let clean = title.trim();
                        if !clean.is_empty() {
                            self.last_title = Some(clean.to_string());
                        }
                    }
                    if let Some(poster) = meta.get("poster").and_then(|v| v.as_str()) {
                        let clean = poster.trim();
                        if !clean.is_empty() {
                            self.last_poster_url = Some(clean.to_string());
                        }
                    }
                }
            }
        }
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
        // Position window off-screen to hide from user while keeping full browser capabilities
        args.push(std::ffi::OsStr::new("--window-position=-32000,-32000"));
        args.push(std::ffi::OsStr::new(
            "--autoplay-policy=no-user-gesture-required",
        ));
        // Anti-bot-detection: hide automation indicators from Cloudflare
        args.push(std::ffi::OsStr::new(
            "--disable-blink-features=AutomationControlled",
        ));
        args.push(std::ffi::OsStr::new("--disable-infobars"));
        args.push(std::ffi::OsStr::new("--excludeSwitches=enable-automation"));
        // Match Python script's User-Agent exactly to ensure same behavior
        args.push(std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"));

        let options = LaunchOptions {
            headless: false,
            sandbox: false,
            args: args,
            ..Default::default()
        };

        let browser =
            Browser::new(options).map_err(|e| format!("Failed to launch Chrome: {}", e))?;

        let browser = Arc::new(browser);
        self.browser = Some(Arc::clone(&browser));

        eprintln!("[ChromeDetector] Chrome launched successfully");
        Ok(browser)
    }

    /// Helper to emit progress events
    fn emit_progress(&self, app_handle: Option<&AppHandle>, message: &str, progress: u32) {
        if let Some(app) = app_handle {
            let _ = app.emit(
                "detection-progress",
                serde_json::json!({
                    "message": message,
                    "progress": progress
                }),
            );
        }
        eprintln!("[ChromeDetector] {} ({}%)", message, progress);
    }

    /// Auto-detect video URL from a webpage
    pub fn detect_video_url(
        &mut self,
        url: &str,
        app_handle: Option<&AppHandle>,
    ) -> Result<Option<String>, String> {
        // Reset metadata cache for this detection run
        self.last_title = None;
        self.last_poster_url = None;

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

        // Inject PerformanceObserver logic
        let observer_script = r#"
            (function() {
                // Increase buffer size to capture all requests (default is often 150-250)
                if (performance.setResourceTimingBufferSize) {
                    performance.setResourceTimingBufferSize(5000);
                }

                if (!window.__FOUND_URLS) {
                    window.__FOUND_URLS = [];

                    const isValidCandidate = (name) => {
                        return name.includes('.m3u8') ||
                               name.includes('.mp4') ||
                               name.includes('video') ||
                               name.includes('master');
                    };

                    // Check existing entries first
                    performance.getEntriesByType('resource').forEach(e => {
                        if (isValidCandidate(e.name)) {
                            console.log("Found candidate via getEntries:", e.name);
                            window.__FOUND_URLS.push(e.name);
                        }
                    });

                    // Observe new entries
                    try {
                        const observer = new PerformanceObserver((list) => {
                            list.getEntries().forEach((entry) => {
                                if (isValidCandidate(entry.name)) {
                                    console.log("Found candidate via Observer:", entry.name);
                                    window.__FOUND_URLS.push(entry.name);
                                }
                            });
                        });
                        observer.observe({ entryTypes: ['resource'] });
                        console.log("PerformanceObserver attached with buffer size 5000");
                    } catch(e) {
                        console.error("Observer failed:", e);
                    }
                }
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
                eprintln!(
                    "[ChromeDetector] Failed to create tab: {}, restarting browser...",
                    e
                );
                self.cleanup();
                let browser = self.init_browser()?;
                browser
                    .new_tab()
                    .map_err(|e| format!("Failed to create tab after restart: {}", e))?
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
        self.extract_cookies(&tab);
        self.capture_page_metadata(&tab);

        // NjavTV special handling: Poll for HLS.js instance with m3u8 URL
        // Key insight: NjavTV sets window.hls.url during page init (no clicking needed).
        // PerformanceObserver fails (0 entries), HTML has no m3u8, clicking causes page navigation.
        let is_njavtv = url.contains("njavtv.com");
        if is_njavtv {
            eprintln!("[ChromeDetector] NjavTV detected - polling for HLS.js instance");
            self.emit_progress(
                app_handle,
                "NjavTV detected - waiting for video player...",
                45,
            );

            // Wait for initial page load to stabilize
            std::thread::sleep(Duration::from_secs(2));

            // PRIMARY: Poll window.hls.url in a retry loop.
            // DO NOT click - it causes page navigation and loses the video player
            let hls_url_check = r#"
                (function() {
                    // 1. Direct window.hls check
                    if (window.hls && window.hls.url) return window.hls.url;
                    // 2. Check video elements for attached HLS instances
                    const videos = document.querySelectorAll('video');
                    for (let v of videos) {
                        if (v._hls && v._hls.url) return v._hls.url;
                    }
                    return null;
                })()
            "#;

            // 10 attempts × 2s = 20s total (usually finds within 2-3 attempts)
            for attempt in 1..=10 {
                let progress = 45 + (attempt * 3);
                self.emit_progress(
                    app_handle,
                    &format!("Waiting for video player... ({}/10)", attempt),
                    progress as u32,
                );

                // Wait between attempts
                std::thread::sleep(Duration::from_secs(2));

                // Check window.hls
                if let Ok(result) = tab.evaluate(hls_url_check, false) {
                    if let Some(value) = result.value {
                        // Handle both string URL and null
                        if value.is_string() {
                            if let Ok(found_url) = serde_json::from_value::<String>(value) {
                                if !found_url.is_empty() {
                                    eprintln!("[ChromeDetector] NjavTV: Found m3u8 via window.hls.url (attempt {}): {}", attempt, found_url);
                                    self.extract_cookies(&tab);
                                    self.emit_progress(app_handle, "Found video URL via HLS.js!", 100);
                                    return Ok(Some(found_url));
                                }
                            }
                        }
                        // value is null - continue
                    }
                }

                // Also check PerformanceObserver as fallback
                if let Some(found_url) = self.check_for_video_url(&tab) {
                    eprintln!("[ChromeDetector] NjavTV: Found URL via Observer (attempt {}): {}", attempt, found_url);
                    self.extract_cookies(&tab);
                    self.emit_progress(app_handle, "Found video URL via Network!", 100);
                    return Ok(Some(found_url));
                }

                eprintln!("[ChromeDetector] NjavTV attempt {} - no URL yet, continuing...", attempt);
            }

            eprintln!(
                "[ChromeDetector] NjavTV specific methods failed, trying general detection..."
            );
        }

        // njav.org special handling: Resolve iframe chain and navigate to video player
        // njav.org → iframe (missav.guide) → redirect → javxx.com SPA → iframe → surrit.store
        // The video is 2 iframes deep; generic iframe scan doesn't recurse enough.
        let is_njav = url.contains("njav.org");
        if is_njav {
            eprintln!("[ChromeDetector] njav.org detected - resolving iframe chain");
            self.emit_progress(
                app_handle,
                "njav.org detected - resolving video page...",
                45,
            );

            // Wait for njav.org page to fully load (Cloudflare challenge may need time)
            std::thread::sleep(Duration::from_secs(3));

            // Step 1: Extract iframe src from njav.org page (e.g., missav.guide/snos-034)
            let iframe_extract_script = r#"
                (function() {
                    // Try id="advanced_iframe" (WordPress Advanced iFrame plugin)
                    let iframe = document.getElementById('advanced_iframe');
                    if (iframe && iframe.src && iframe.src.startsWith('http')) return iframe.src;

                    // Fallback: any iframe with known video domains
                    const iframes = document.querySelectorAll('iframe[src]');
                    for (let i of iframes) {
                        if (i.src.includes('missav') || i.src.includes('javxx') || i.src.includes('surrit')) {
                            return i.src;
                        }
                    }
                    return null;
                })()
            "#;

            let iframe_url = if let Ok(result) = tab.evaluate(iframe_extract_script, false) {
                if let Some(value) = result.value {
                    if value.is_string() {
                        serde_json::from_value::<String>(value).ok()
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            };

            if let Some(ref src) = iframe_url {
                eprintln!("[ChromeDetector] njav.org: Found iframe URL: {}", src);
                self.emit_progress(app_handle, "Navigating to video page...", 55);

                // Step 2: Navigate to iframe URL (missav.guide → redirects to javxx.com)
                if let Ok(_) = tab.navigate_to(src) {
                    let _ = tab.wait_until_navigated();
                    self.capture_page_metadata(&tab);
                    eprintln!("[ChromeDetector] njav.org: Navigated to iframe page");

                    // Wait for javxx.com SPA to render (Cloudflare + React)
                    std::thread::sleep(Duration::from_secs(5));

                    // Inject observer on the new page
                    let _ = tab.evaluate(observer_script, false);

                    // Step 3: Check if this page has the video directly
                    if let Some(found_url) = self.check_for_video_url(&tab) {
                        eprintln!("[ChromeDetector] njav.org: Found video URL on iframe page: {}", found_url);
                        self.extract_cookies(&tab);
                        self.emit_progress(app_handle, "Found video URL!", 100);
                        return Ok(Some(found_url));
                    }

                    // Step 4: Look for surrit.store iframe (second level)
                    eprintln!("[ChromeDetector] njav.org: Looking for nested iframe (surrit.store)...");
                    self.emit_progress(app_handle, "Scanning nested iframe...", 70);

                    let surrit_extract_script = r#"
                        (function() {
                            const iframes = document.querySelectorAll('iframe[src]');
                            for (let i of iframes) {
                                if (i.src.includes('surrit.store') || i.src.includes('wowstream')) {
                                    return i.src;
                                }
                            }
                            return null;
                        })()
                    "#;

                    let surrit_url = if let Ok(result) = tab.evaluate(surrit_extract_script, false) {
                        if let Some(value) = result.value {
                            if value.is_string() {
                                serde_json::from_value::<String>(value).ok()
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    if let Some(ref surrit) = surrit_url {
                        eprintln!("[ChromeDetector] njav.org: Found surrit.store iframe: {}", surrit);
                        self.emit_progress(app_handle, "Loading video player...", 80);

                        // Navigate directly to surrit.store
                        if let Ok(_) = tab.navigate_to(surrit) {
                            let _ = tab.wait_until_navigated();
                            self.capture_page_metadata(&tab);

                            // Inject observer on surrit.store
                            let _ = tab.evaluate(observer_script, false);
                            std::thread::sleep(Duration::from_secs(2));

                            // Click play to trigger video load
                            let _ = tab.evaluate(click_script, false);
                            std::thread::sleep(Duration::from_secs(3));

                            // Check for m3u8 URL
                            if let Some(found_url) = self.check_for_video_url(&tab) {
                                eprintln!("[ChromeDetector] njav.org: Found m3u8 from surrit.store: {}", found_url);
                                self.extract_cookies(&tab);
                                self.emit_progress(app_handle, "Found video URL!", 100);
                                return Ok(Some(found_url));
                            }

                            // Retry: click again and wait longer
                            for attempt in 1..=5 {
                                eprintln!("[ChromeDetector] njav.org: Retry attempt {} on surrit.store", attempt);
                                self.emit_progress(
                                    app_handle,
                                    &format!("Retrying video detection... ({}/5)", attempt),
                                    80 + (attempt * 3) as u32,
                                );
                                let _ = tab.evaluate(click_script, false);
                                std::thread::sleep(Duration::from_secs(3));

                                if let Some(found_url) = self.check_for_video_url(&tab) {
                                    eprintln!("[ChromeDetector] njav.org: Found m3u8 (attempt {}): {}", attempt, found_url);
                                    self.extract_cookies(&tab);
                                    self.emit_progress(app_handle, "Found video URL!", 100);
                                    return Ok(Some(found_url));
                                }
                            }
                        }
                    } else {
                        eprintln!("[ChromeDetector] njav.org: No surrit.store iframe found, trying general detection on current page");

                        // No surrit iframe but we're on javxx.com - try clicking and polling
                        for attempt in 1..=6 {
                            let _ = tab.evaluate(click_script, false);
                            std::thread::sleep(Duration::from_secs(3));

                            if let Some(found_url) = self.check_for_video_url(&tab) {
                                eprintln!("[ChromeDetector] njav.org: Found URL on javxx page (attempt {}): {}", attempt, found_url);
                                self.extract_cookies(&tab);
                                self.emit_progress(app_handle, "Found video URL!", 100);
                                return Ok(Some(found_url));
                            }
                        }
                    }
                }
            } else {
                eprintln!("[ChromeDetector] njav.org: No iframe found on page, trying general detection");
            }

            // If njav-specific methods failed, fall through to generic detection
            eprintln!("[ChromeDetector] njav.org specific methods failed, trying general detection...");
        }

        // avkuy.com special handling: Cloudflare + embedded av-kuy player iframe
        let is_avkuy = url.contains("avkuy.com") || url.contains("av-kuy.com");
        if is_avkuy {
            eprintln!("[ChromeDetector] avkuy detected - resolving iframe player");
            self.emit_progress(
                app_handle,
                "avkuy detected - bypassing Cloudflare...",
                45,
            );

            let cloudflare_resolved = if self.is_cloudflare_page(&tab) {
                self.wait_for_cloudflare(&tab, app_handle, 60)
            } else {
                true
            };

            if cloudflare_resolved {
                // Cloudflare flow can occasionally land on homepage/search.
                // Navigate back to the requested URL before extracting iframe/player.
                let current_url = tab.get_url();
                let requested_tail = url
                    .split('/')
                    .filter(|s| !s.is_empty())
                    .next_back()
                    .unwrap_or("");
                let lost_target_page = !requested_tail.is_empty() && !current_url.contains(requested_tail);
                if lost_target_page && tab.navigate_to(url).is_ok() {
                    let _ = tab.wait_until_navigated();
                    std::thread::sleep(Duration::from_secs(2));
                }

                // Capture title/poster from the main page first
                self.capture_page_metadata(&tab);
                let _ = tab.evaluate(observer_script, false);
                std::thread::sleep(Duration::from_secs(2));

                if let Some(found_url) = self.check_for_video_url(&tab) {
                    eprintln!("[ChromeDetector] avkuy: Found URL on main page: {}", found_url);
                    self.extract_cookies(&tab);
                    self.emit_progress(app_handle, "Found video URL!", 100);
                    return Ok(Some(found_url));
                }

                // Try interacting on the main post page first.
                // The embedded iframe sometimes loads stream requests that are visible
                // from the top-level page without direct iframe navigation.
                for attempt in 1..=8 {
                    self.emit_progress(
                        app_handle,
                        &format!("Trying avkuy main player... ({}/8)", attempt),
                        52 + (attempt * 2) as u32,
                    );
                    let _ = tab.evaluate(click_script, false);
                    std::thread::sleep(Duration::from_secs(3));

                    if let Some(found_url) = self.check_for_video_url(&tab) {
                        eprintln!(
                            "[ChromeDetector] avkuy: Found URL on main page (attempt {}): {}",
                            attempt, found_url
                        );
                        self.extract_cookies(&tab);
                        self.emit_progress(app_handle, "Found video URL!", 100);
                        return Ok(Some(found_url));
                    }
                }

                let iframe_extract = r#"
                    (function() {
                        const normalize = (src) => {
                            if (!src) return '';
                            if (src.startsWith('//')) return 'https:' + src;
                            if (src.startsWith('http')) return src;
                            return '';
                        };

                        const iframes = document.querySelectorAll('iframe');
                        for (const frame of iframes) {
                            const src =
                                frame.src ||
                                frame.getAttribute('data-src') ||
                                frame.getAttribute('data-lazy-src') ||
                                '';
                            const u = normalize(src);
                            if (!u) continue;
                            if (u.includes('av-kuy.com/v/') || u.includes('/v/')) {
                                return u;
                            }
                        }
                        // Fallback: some themes keep player URL inside script/html
                        const html = document.documentElement.outerHTML;
                        const m = html.match(/(https?:\/\/[^"'\s<>]*av-kuy\.com\/v\/[^"'\s<>]*)/i);
                        if (m) return m[1];
                        return null;
                    })()
                "#;

                // Player iframe can be injected late by theme scripts/ads.
                let mut iframe_url: Option<String> = None;
                for attempt in 1..=10 {
                    if let Ok(result) = tab.evaluate(iframe_extract, false) {
                        if let Some(value) = result.value {
                            if value.is_string() {
                                if let Ok(found) = serde_json::from_value::<String>(value) {
                                    if !found.is_empty() {
                                        iframe_url = Some(found);
                                        break;
                                    }
                                }
                            }
                        }
                    }

                    self.emit_progress(
                        app_handle,
                        &format!("Waiting for avkuy player iframe... ({}/10)", attempt),
                        58 + (attempt * 2) as u32,
                    );
                    std::thread::sleep(Duration::from_secs(2));
                }

                if let Some(ref iframe_src) = iframe_url {
                    eprintln!("[ChromeDetector] avkuy: Found iframe URL: {}", iframe_src);
                    self.emit_progress(app_handle, "Loading embedded player...", 60);

                    if tab.navigate_to(iframe_src).is_ok() {
                        let _ = tab.wait_until_navigated();
                        self.extract_cookies(&tab);
                        if self.is_cloudflare_page(&tab) {
                            self.emit_progress(
                                app_handle,
                                "Solving player Cloudflare challenge...",
                                62,
                            );
                            let solved = self.wait_for_cloudflare(&tab, app_handle, 45);
                            if solved {
                                // Re-open iframe URL after challenge resolution to ensure player scripts run
                                let _ = tab.navigate_to(iframe_src);
                                let _ = tab.wait_until_navigated();
                            }
                        }
                        self.capture_page_metadata(&tab);
                        let _ = tab.evaluate(observer_script, false);
                        std::thread::sleep(Duration::from_secs(2));

                        if let Some(found_url) = self.check_for_video_url(&tab) {
                            eprintln!("[ChromeDetector] avkuy: Found URL in iframe: {}", found_url);
                            self.extract_cookies(&tab);
                            self.emit_progress(app_handle, "Found video URL!", 100);
                            return Ok(Some(found_url));
                        }

                        for attempt in 1..=10 {
                            self.emit_progress(
                                app_handle,
                                &format!("Waiting for avkuy video... ({}/10)", attempt),
                                60 + (attempt * 3) as u32,
                            );
                            let _ = tab.evaluate(click_script, false);
                            std::thread::sleep(Duration::from_secs(3));

                            if let Some(found_url) = self.check_for_video_url(&tab) {
                                eprintln!(
                                    "[ChromeDetector] avkuy: Found URL in iframe (attempt {}): {}",
                                    attempt, found_url
                                );
                                self.extract_cookies(&tab);
                                self.emit_progress(app_handle, "Found video URL!", 100);
                                return Ok(Some(found_url));
                            }
                        }
                    }
                }
            }

            // Reset to original page before generic fallback, so we don't stay on
            // the iframe challenge page.
            let _ = tab.navigate_to(url);
            let _ = tab.wait_until_navigated();
            let _ = tab.evaluate(observer_script, false);
            eprintln!("[ChromeDetector] avkuy specific methods failed, trying general detection...");
        }

        // javwow.com special handling: Resolve onlysubthai.com iframe for HLS stream
        // javwow.com → Cloudflare challenge → page with iframe → onlysubthai.com → HLS .m3u8
        let is_javwow = url.contains("javwow.com");
        if is_javwow {
            eprintln!("[ChromeDetector] javwow.com detected - resolving video page");
            self.emit_progress(
                app_handle,
                "javwow.com detected - bypassing Cloudflare...",
                45,
            );

            // Phase 1: Handle Cloudflare challenge (wait up to 30s for auto-resolution)
            let cloudflare_resolved = if self.is_cloudflare_page(&tab) {
                eprintln!("[ChromeDetector] javwow.com: Cloudflare challenge detected");
                self.wait_for_cloudflare(&tab, app_handle, 30)
            } else {
                eprintln!("[ChromeDetector] javwow.com: No Cloudflare challenge, page loaded directly");
                true
            };

            // Inject comprehensive stealth scripts (helps with post-challenge checks)
            let stealth_script = r#"
                (function() {
                    Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
                    Object.defineProperty(navigator, 'languages', { get: () => ['th', 'en-US', 'en'] });
                    Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
                    // Override permissions to avoid detection
                    if (navigator.permissions) {
                        const origQuery = navigator.permissions.query;
                        navigator.permissions.query = (params) => (
                            params.name === 'notifications' ?
                                Promise.resolve({ state: Notification.permission }) :
                                origQuery(params)
                        );
                    }
                    return true;
                })()
            "#;
            let _ = tab.evaluate(stealth_script, false);
            let _ = tab.evaluate(observer_script, false);

            // Phase 2: If Cloudflare was resolved, extract iframe and find video
            if cloudflare_resolved {
                // Small wait for page DOM to settle after Cloudflare redirect
                std::thread::sleep(Duration::from_secs(2));

                // Check if page already has m3u8
                if let Some(found_url) = self.check_for_video_url(&tab) {
                    eprintln!("[ChromeDetector] javwow.com: Found video URL on main page: {}", found_url);
                    self.extract_cookies(&tab);
                    self.emit_progress(app_handle, "Found video URL!", 100);
                    return Ok(Some(found_url));
                }

                // Extract onlysubthai.com embed URL from iframe, og:video, or regex
                let embed_extract = r#"
                    (function() {
                        // Method 1: iframe with onlysubthai
                        const iframes = document.querySelectorAll('iframe');
                        for (let i of iframes) {
                            const src = i.src || i.dataset.src || i.dataset.lazySrc || '';
                            if (src.includes('onlysubthai') || src.includes('subthai')) {
                                if (src.startsWith('//')) return 'https:' + src;
                                return src;
                            }
                        }
                        // Method 2: og:video meta tag
                        const ogVideo = document.querySelector('meta[property="og:video"]');
                        if (ogVideo && ogVideo.content) return ogVideo.content;
                        const ogVideoUrl = document.querySelector('meta[property="og:video:url"]');
                        if (ogVideoUrl && ogVideoUrl.content) return ogVideoUrl.content;
                        // Method 3: Regex in raw HTML
                        const html = document.documentElement.outerHTML;
                        const match = html.match(/(https?:\/\/[^"'\s<>]*onlysubthai[^"'\s<>]*)/);
                        if (match) return match[1];
                        // Method 4: Look for embedURL in JSON-LD or application/ld+json
                        const ldScripts = document.querySelectorAll('script[type="application/ld+json"]');
                        for (let s of ldScripts) {
                            try {
                                const data = JSON.parse(s.textContent);
                                if (data.embedUrl) return data.embedUrl;
                                if (data.contentUrl) return data.contentUrl;
                            } catch(e) {}
                        }
                        return null;
                    })()
                "#;

                let embed_url = if let Ok(result) = tab.evaluate(embed_extract, false) {
                    if let Some(value) = result.value {
                        if value.is_string() {
                            serde_json::from_value::<String>(value).ok()
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                if let Some(ref src) = embed_url {
                    eprintln!("[ChromeDetector] javwow.com: Found embed URL: {}", src);
                    self.emit_progress(app_handle, "Loading video player...", 60);

                    if let Ok(_) = tab.navigate_to(src) {
                        let _ = tab.wait_until_navigated();
                        self.capture_page_metadata(&tab);
                        std::thread::sleep(Duration::from_secs(3));
                        let _ = tab.evaluate(observer_script, false);

                        // Quick check
                        if let Some(found_url) = self.check_for_video_url(&tab) {
                            eprintln!("[ChromeDetector] javwow.com: Found m3u8 on onlysubthai: {}", found_url);
                            self.extract_cookies(&tab);
                            self.emit_progress(app_handle, "Found video URL!", 100);
                            return Ok(Some(found_url));
                        }

                        // Click + poll for m3u8 on onlysubthai.com (no Cloudflare there)
                        for attempt in 1..=8 {
                            eprintln!("[ChromeDetector] javwow.com: Polling onlysubthai attempt {}", attempt);
                            self.emit_progress(
                                app_handle,
                                &format!("Waiting for video... ({}/8)", attempt),
                                60 + (attempt * 4) as u32,
                            );
                            let _ = tab.evaluate(click_script, false);
                            std::thread::sleep(Duration::from_secs(3));

                            if let Some(found_url) = self.check_for_video_url(&tab) {
                                eprintln!("[ChromeDetector] javwow.com: Found m3u8 (attempt {}): {}", attempt, found_url);
                                self.extract_cookies(&tab);
                                self.emit_progress(app_handle, "Found video URL!", 100);
                                return Ok(Some(found_url));
                            }
                        }
                    }
                } else {
                    eprintln!("[ChromeDetector] javwow.com: No embed URL found on page");
                }
            } else {
                eprintln!("[ChromeDetector] javwow.com: Could not bypass Cloudflare after 30s");
            }

            // Fall through to generic detection as last resort
            eprintln!("[ChromeDetector] javwow.com specific methods failed, trying general detection...");
        }

        // Inject PerformanceObserver immediately after load
        let _ = tab.evaluate(observer_script, false);

        // 1. Initial wait for basic page load
        self.emit_progress(app_handle, "Waiting for dynamic content...", 50);
        std::thread::sleep(Duration::from_secs(5));

        // POLLING LOOP: Try to find video URL multiple times
        for attempt in 1..=8 {
            // Increased attempts
            let progress = 50 + (attempt * 5);

            let _ = tab.evaluate(click_script, false);

            // Wait after clicking
            self.emit_progress(
                app_handle,
                &format!("Attempt {}/8: Waiting for video load...", attempt),
                progress as u32 + 4,
            );
            std::thread::sleep(Duration::from_secs(4)); // Increased wait time

            // C. Check again after click
            if let Some(url) = self.check_for_video_url(&tab) {
                self.emit_progress(app_handle, "Video URL found!", 100);
                return Ok(Some(url));
            }
        }

        // D. Recursive Iframe Check (if main page failed)
        self.emit_progress(
            app_handle,
            "Main page scan finished. Checking internal iframes...",
            95,
        );

        // DEBUG: Dump main page HTML
        let dump_main_script = "document.documentElement.outerHTML";
        if let Ok(result) = tab.evaluate(dump_main_script, false) {
            if let Some(value) = result.value {
                if let Ok(html) = serde_json::from_value::<String>(value) {
                    let _ = std::fs::write("page_dump.html", html);
                    eprintln!("[ChromeDetector] Dumped main page HTML to page_dump.html");
                }
            }
        }

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
                        eprintln!(
                            "[ChromeDetector] Found {} internal iframes to scan",
                            iframe_urls.len()
                        );

                        for (i, iframe_url) in iframe_urls.iter().enumerate() {
                            self.emit_progress(
                                app_handle,
                                &format!(
                                    "Scanning internal iframe {}/{}...",
                                    i + 1,
                                    iframe_urls.len()
                                ),
                                95,
                            );
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
                                    self.emit_progress(
                                        app_handle,
                                        "Video URL found in iframe!",
                                        100,
                                    );
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
                                    self.emit_progress(
                                        app_handle,
                                        "Video URL found in iframe!",
                                        100,
                                    );
                                    return Ok(Some(url));
                                }

                                // DEBUG: Dump iframe HTML to file to analyze why regex failed
                                let dump_script = "document.documentElement.outerHTML";
                                if let Ok(result) = tab.evaluate(dump_script, false) {
                                    if let Some(value) = result.value {
                                        if let Ok(html) = serde_json::from_value::<String>(value) {
                                            let _ = std::fs::write("iframe_dump.html", html);
                                            eprintln!("[ChromeDetector] Dumped iframe HTML to iframe_dump.html");
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        self.extract_cookies(&tab);
        self.emit_progress(app_handle, "No video URL found after all attempts", 100);
        Ok(None)
    }

    /// Helper to check for video URLs using multiple methods
    fn check_for_video_url(&self, tab: &headless_chrome::Tab) -> Option<String> {
        // Method 0: Check our injected global variable (PerformanceObserver results)
        let observer_check_code = r#"
            (function() {
                if (window.__FOUND_URLS && window.__FOUND_URLS.length > 0) {
                    const isPreview = (u) => u.includes('fourhoi.com') || u.includes('growcdnssedge.com') || u.includes('preview.mp4');
                    
                    // Filter and prioritize
                    const m3u8s = window.__FOUND_URLS.filter(u => u.includes('.m3u8') && !isPreview(u));
                    
                    // Prioritize surrit.com
                    const surrit = m3u8s.find(u => u.includes('surrit.com'));
                    if (surrit) return JSON.stringify([surrit]);
                    
                    // Prioritize 720p or higher if named
                    const highRes = m3u8s.find(u => u.includes('720p') || u.includes('1080p') || u.includes('original'));
                    if (highRes) return JSON.stringify([highRes]);

                    if (m3u8s.length > 0) return JSON.stringify([m3u8s[m3u8s.length-1]]);
                    
                    const mp4s = window.__FOUND_URLS.filter(u => u.includes('.mp4') && !isPreview(u));
                    if (mp4s.length > 0) return JSON.stringify([mp4s[0]]);

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
                            eprintln!(
                                "[ChromeDetector] Found URL via PerformanceObserver: {}",
                                url
                            );
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
                        if url.contains(".m3u8") || url.contains(".mp4") || url.starts_with("blob:")
                        {
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

        // Method 3: Robust Regex Search (Python Parity)
        // Matches https://...m3u8 inside quotes or whitespace boundaries
        let html_search_code = r#"
            (function() {
                const html = document.documentElement.outerHTML;

                // Pattern 1: URL inside quotes (common in JS vars)
                // Looks for "http...m3u8..." or 'http...m3u8...'
                let match = html.match(/["'](https?:[^"']+\.m3u8[^"']*)["']/);
                if (match) return match[1];

                // Pattern 2: URL inside quotes (escaped slashes)
                // Looks for "http:\/\/...m3u8..."
                match = html.match(/["'](https?:\\\/\\\/[^"']+\.m3u8[^"']*)["']/);
                if (match) return match[1].replace(/\\\//g, '/');

                // Pattern 3: URL without quotes (less common but possible in some attributes)
                match = html.match(/(https?:\/\/[^\s<>"']+\.m3u8[^\s<>"']*)/);
                if (match) return match[1];

                return null;
            })()
        "#;

        if let Ok(result) = tab.evaluate(html_search_code, false) {
            if let Some(value) = result.value {
                if let Ok(url) = serde_json::from_value::<String>(value) {
                    if !url.is_empty() {
                        let clean_url = url
                            .replace("\\/", "/")
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
                    if let (Some(asset), Some(media_id)) =
                        (config["asset"].as_str(), config["mediaId"].as_str())
                    {
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

    /// Extract video ID from NjavTV URL
    /// e.g., https://njavtv.com/dm13/th/cus-1267 -> cus-1267
    #[allow(dead_code)]
    fn extract_njavtv_video_id(url: &str) -> Option<String> {
        // Handle URLs like /dm13/th/cus-1267 or /cus-1267
        if let Some(pos) = url.rfind('/') {
            let part = &url[pos + 1..];
            if !part.is_empty() && !part.contains('.') {
                // Make sure it's not a domain
                if !part.contains("njavtv.com") {
                    return Some(part.to_string());
                }
            }
        }
        // Also try to match pattern like cus-1267, abp-123, etc.
        let patterns = ["cus-", "abp-", "ipx-", "ssni-"];
        for pattern in &patterns {
            if let Some(idx) = url.to_lowercase().find(pattern) {
                let start = idx;
                let end = url[start..]
                    .find('/')
                    .map(|p| start + p)
                    .unwrap_or(url.len());
                let video_id = &url[start..end];
                if !video_id.is_empty() && video_id.len() > 3 {
                    return Some(video_id.to_string());
                }
            }
        }
        None
    }

    /// Extract m3u8 URL from playlist API response
    #[allow(dead_code)]
    fn extract_m3u8_from_playlist(response: &serde_json::Value) -> Option<String> {
        // Try different response structures

        // Structure 1: response.data.items[].file
        if let Some(items) = response["data"]["items"].as_array() {
            for item in items {
                if let Some(file) = item["file"].as_str() {
                    if file.contains(".m3u8") {
                        return Some(file.to_string());
                    }
                }
            }
        }

        // Structure 2: response.data.file
        if let Some(file) = response["data"]["file"].as_str() {
            if file.contains(".m3u8") {
                return Some(file.to_string());
            }
        }

        // Structure 3: response.data[].sources[].file
        if let Some(data) = response["data"].as_array() {
            for item in data {
                if let Some(sources) = item["sources"].as_array() {
                    for source in sources {
                        if let Some(file) = source["file"].as_str() {
                            if file.contains(".m3u8") {
                                return Some(file.to_string());
                            }
                        }
                    }
                }
            }
        }

        // Structure 4: response.items[].file
        if let Some(items) = response["items"].as_array() {
            for item in items {
                if let Some(file) = item["file"].as_str() {
                    if file.contains(".m3u8") {
                        return Some(file.to_string());
                    }
                }
            }
        }

        // Structure 5: response.file
        if let Some(file) = response["file"].as_str() {
            if file.contains(".m3u8") {
                return Some(file.to_string());
            }
        }

        None
    }

    /// Check if current page is a Cloudflare challenge page
    fn is_cloudflare_page(&self, tab: &headless_chrome::Tab) -> bool {
        let check_script = r#"
            (function() {
                const title = (document.title || '').toLowerCase();
                const bodyText = document.body ? (document.body.innerText || '').toLowerCase() : '';
                const hasCfErrorImage = document.querySelector('img[alt="error"]') !== null;
                const hasInlineCfScript = Array.from(document.querySelectorAll('script'))
                    .some(s => (s.textContent || '').includes('__CF$cv$params'));
                return title.includes('just a moment') ||
                       title.includes('attention required') ||
                       title.includes('checking your browser') ||
                       (hasCfErrorImage && hasInlineCfScript) ||
                       bodyText.includes('verify you are human') ||
                       bodyText.includes('checking your browser before accessing') ||
                       document.getElementById('challenge-running') !== null ||
                       document.getElementById('challenge-form') !== null ||
                       document.querySelector('form#challenge-form') !== null ||
                       document.querySelector('.cf-turnstile') !== null ||
                       document.querySelector('iframe[src*="challenges.cloudflare.com"]') !== null;
            })()
        "#;

        if let Ok(result) = tab.evaluate(check_script, false) {
            if let Some(value) = result.value {
                if let Ok(is_cf) = serde_json::from_value::<bool>(value) {
                    return is_cf;
                }
            }
        }
        false
    }

    /// Wait for Cloudflare challenge to auto-resolve, returns true if resolved
    fn wait_for_cloudflare(
        &self,
        tab: &headless_chrome::Tab,
        app_handle: Option<&AppHandle>,
        max_wait_secs: u64,
    ) -> bool {
        let start = std::time::Instant::now();
        let mut waited = 0u64;

        // First, try clicking any Turnstile checkbox
        let click_turnstile = r#"
            (function() {
                // Click Turnstile iframe
                document.querySelectorAll('iframe[src*="challenges.cloudflare.com"]').forEach(f => {
                    try { f.click(); } catch(e) {}
                });
                // Click visible checkboxes
                document.querySelectorAll('input[type="checkbox"]').forEach(cb => {
                    try { if (cb.offsetParent !== null) cb.click(); } catch(e) {}
                });
                // Click challenge-local buttons only (avoid unrelated site buttons/forms)
                const scopes = [
                    document.getElementById('challenge-form'),
                    document.querySelector('[id*=\"challenge\"]'),
                    document.querySelector('.cf-challenge'),
                    document.querySelector('.cf-turnstile')
                ].filter(Boolean);
                scopes.forEach(scope => {
                    scope.querySelectorAll('button, input[type=\"submit\"]').forEach(btn => {
                        try { if (btn.offsetParent !== null) btn.click(); } catch(e) {}
                    });
                });
                // Fallback center click (often where Cloudflare checkbox sits)
                try {
                    const el = document.elementFromPoint(window.innerWidth / 2, window.innerHeight / 2);
                    if (el) el.click();
                } catch(e) {}
                return true;
            })()
        "#;
        let _ = tab.evaluate(click_turnstile, false);

        while start.elapsed().as_secs() < max_wait_secs {
            if !self.is_cloudflare_page(tab) {
                eprintln!(
                    "[ChromeDetector] Cloudflare resolved after {}s",
                    start.elapsed().as_secs()
                );
                // Small extra wait for page to fully load after challenge
                std::thread::sleep(Duration::from_millis(500));
                return true;
            }

            waited += 2;
            self.emit_progress(
                app_handle,
                &format!("Solving Cloudflare challenge... ({}s)", waited),
                47,
            );
            std::thread::sleep(Duration::from_secs(2));

            // Re-try clicking on each iteration
            let _ = tab.evaluate(click_turnstile, false);
        }

        eprintln!(
            "[ChromeDetector] Cloudflare NOT resolved after {}s",
            max_wait_secs
        );
        false
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

#[cfg(test)]
mod live_tests {
    use super::ChromeVideoDetector;

    #[test]
    #[ignore = "Live network/browser test for avkuy Cloudflare flow"]
    fn live_detect_avkuy_url() {
        let mut detector = ChromeVideoDetector::new().expect("detector init failed");
        let url = "https://www2.avkuy.com/avsubthai-adn-750/";
        let detected = detector
            .detect_video_url(url, None)
            .expect("detection should not error");

        assert!(
            detected
                .as_deref()
                .map(|u| u.contains(".m3u8") || u.contains(".mp4"))
                .unwrap_or(false),
            "Expected m3u8/mp4 URL, got: {:?}",
            detected
        );
    }
}
