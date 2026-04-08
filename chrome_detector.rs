use headless_chrome::{Browser, LaunchOptions};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};

/// Debug log to file (for GUI apps where stderr isn't captured)
fn debug_log(msg: &str) {
    use std::io::Write;
    if let Ok(mut f) = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open("/tmp/njav-chrome-debug.log")
    {
        let _ = writeln!(f, "{}", msg);
    }
}

/// Auto-detect video URLs using headless Chrome browser
pub struct ChromeVideoDetector {
    browser: Option<Arc<Browser>>,
    /// Store cookies from last detection for authentication
    last_cookies: Vec<(String, String)>,
}

impl ChromeVideoDetector {
    pub fn new() -> Result<Self, String> {
        Ok(Self {
            browser: None,
            last_cookies: Vec::new(),
        })
    }

    fn extract_cookies(&mut self, tab: &headless_chrome::Tab) {
        let cookie_script = r#"
            (function() {
                return document.cookie.split(';').map(c => {
                    const eq = c.indexOf('=');
                    const name = c.substring(0, eq).trim();
                    const value = c.substring(eq + 1).trim();
                    return [name, value];
                });
            })()
        "#;

        if let Ok(result) = tab.evaluate(cookie_script, false) {
            if let Some(value) = result.value {
                if let Ok(cookies) = serde_json::from_value::<Vec<(String, String)>>(value) {
                    if !cookies.is_empty() {
                        eprintln!(
                            "[ChromeDetector] Extracted {} cookies: {:?}",
                            cookies.len(),
                            cookies.iter().map(|(n, _)| n).collect::<Vec<_>>()
                        );
                        self.last_cookies = cookies;
                    }
                }
            }
        }
    }

    /// Get cookies from last detection
    pub fn get_last_cookies(&self) -> &[(String, String)] {
        &self.last_cookies
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
        args.push(std::ffi::OsStr::new(
            "--autoplay-policy=no-user-gesture-required",
        ));
        // Anti-bot detection: prevent Cloudflare from detecting headless Chrome
        args.push(std::ffi::OsStr::new("--disable-blink-features=AutomationControlled"));
        args.push(std::ffi::OsStr::new("--disable-infobars"));
        // Disable web security to allow cross-origin iframe network monitoring
        // This lets PerformanceObserver capture m3u8 requests from surrit.store iframe
        args.push(std::ffi::OsStr::new("--disable-web-security"));
        // Use NEW headless mode (--headless=new) which has the same TLS fingerprint as real Chrome.
        // The old --headless mode has a distinct JA3 fingerprint that Cloudflare blocks with 403.
        args.push(std::ffi::OsStr::new("--headless=new"));
        // Match a recent Chrome User-Agent
        args.push(std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"));

        let options = LaunchOptions {
            headless: false, // Don't let crate add --headless (old mode); we use --headless=new manually
            sandbox: false,
            args: args,
            ignore_default_args: vec![
                std::ffi::OsStr::new("--enable-automation"), // Biggest Cloudflare detection signal
            ],
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
        // Javascript to click play buttons, iframes, and center screen
        let click_script = r##"
            (function() {
                console.log("Triggering aggressive click...");

                // 1. Click all potential play buttons and verification buttons
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
                    ".jw-poster",
                    // Confirmation buttons
                    ".btn-confirm",
                    "button.confirm",
                    "button.yes",
                    ".age-gate-button",
                    "#confirm-button",
                    ".enter-button",
                    ".warning-button"
                ];

                for (let selector of selectors) {
                    try {
                        const elements = document.querySelectorAll(selector);
                        for (let el of elements) {
                            if (el && el.offsetParent !== null) { // Check if visible
                                el.click();
                            }
                        }
                    } catch(e) {}
                }
                
                // Aggressive text-based clicking for confirmation dialogs
                try {
                    const allElements = document.querySelectorAll('button, a, div, span');
                    const keywords = ['confirm', 'agree', 'yes', 'enter', 'เข้าชม', 'ตกลง', 'ยอมรับ', '18+', 'over 18'];
                    for (let el of allElements) {
                        if (el.children.length > 0) continue; // Only target leaf elements with text
                        const txt = el.innerText.toLowerCase().trim();
                        if (keywords.some(k => txt.includes(k))) {
                            el.click();
                        }
                    }
                } catch(e) {}

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
        "##;

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
                        const n = name.toLowerCase();
                        return n.includes('.m3u8') ||
                               n.includes('.mp4') ||
                               n.includes('playlist') ||
                               n.includes('master') ||
                               n.includes('surrit.store') ||
                               n.includes('wowstream.com') ||
                               (n.includes('video') && !n.includes('google-analytics') && !n.includes('ads'));
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

        // Inject stealth script BEFORE navigation to hide headless Chrome fingerprints
        // This overrides navigator.webdriver and patches other detection vectors
        let stealth_script = r#"
            Object.defineProperty(navigator, 'webdriver', { get: () => undefined });
            Object.defineProperty(navigator, 'plugins', { get: () => [1, 2, 3, 4, 5] });
            Object.defineProperty(navigator, 'languages', { get: () => ['en-US', 'en'] });
            window.chrome = { runtime: {} };
        "#;
        let _ = tab.evaluate(stealth_script, false);

        // Navigate to the page
        tab.navigate_to(url)
            .map_err(|e| format!("Failed to navigate: {}", e))?;

        // Wait for page to load
        self.emit_progress(app_handle, "Waiting for page to load...", 40);
        tab.wait_until_navigated()
            .map_err(|e| format!("Navigation failed: {}", e))?;

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

        // njav.org / javxx.com handling:
        // javxx.com/v/{code} → React SPA → surrit.store iframe → wowstream CDN m3u8
        let is_njav = url.contains("njav.org") || url.contains("missav.guide") || url.contains("javxx.com");
        if is_njav {
            debug_log(&format!("[njav] Starting detection for: {}", url));
            eprintln!("[ChromeDetector] njav/javxx detected - resolving video chain");
            self.emit_progress(app_handle, "Resolving video page...", 45);

            // Wait for page to fully load (React SPA)
            if url.contains("javxx.com") {
                self.emit_progress(app_handle, "Loading javxx.com player...", 50);
                std::thread::sleep(Duration::from_secs(3));
            } else {
                // njav.org or missav.guide - wait for redirect
                self.emit_progress(app_handle, "Waiting for redirect...", 50);
                std::thread::sleep(Duration::from_secs(5));
                // Check if we got redirected
                if let Ok(result) = tab.evaluate("window.location.href", false) {
                    if let Some(val) = result.value {
                        if let Ok(current) = serde_json::from_value::<String>(val) {
                            debug_log(&format!("[njav] After redirect: {}", current));
                        }
                    }
                }
            }

            // Inject PerformanceObserver
            let _ = tab.evaluate(observer_script, false);

            // === PHASE 1: Find video iframe on javxx.com SPA ===
            let video_iframe_script = r#"
                (function() {
                    const iframes = document.querySelectorAll('iframe[src]');
                    for (let i of iframes) {
                        const s = i.src.toLowerCase();
                        if (s.includes('surrit') || s.includes('wowstream') || s.includes('player') || s.includes('video') || s.includes('embed')) {
                            return i.src;
                        }
                    }
                    for (let i of iframes) {
                        if (i.offsetWidth > 300 && i.offsetHeight > 200) return i.src;
                    }
                    return null;
                })()
            "#;

            let mut surrit_url: Option<String> = None;

            // Poll up to 20 attempts for SPA to render the iframe (React can be slow)
            for attempt in 1..=20 {
                let progress = 50 + (attempt * 2);
                self.emit_progress(
                    app_handle,
                    &format!("Waiting for video player... ({}/20)", attempt),
                    progress.min(90) as u32,
                );

                // Click to dismiss overlays
                let _ = tab.evaluate(click_script, false);
                std::thread::sleep(Duration::from_secs(2));

                if let Ok(result) = tab.evaluate(video_iframe_script, false) {
                    if let Some(value) = result.value {
                        if value.is_string() {
                            if let Ok(found) = serde_json::from_value::<String>(value) {
                                if !found.is_empty() {
                                    debug_log(&format!("[njav] Found video iframe (attempt {}): {}", attempt, found));
                                    surrit_url = Some(found);
                                    break;
                                }
                            }
                        }
                    }
                }

                // Diagnostic logging every 5 attempts
                if attempt % 5 == 0 {
                    let log_iframes = r#"JSON.stringify(Array.from(document.querySelectorAll('iframe')).map(i => ({src: i.src, width: i.offsetWidth, height: i.offsetHeight})))"#;
                    if let Ok(res) = tab.evaluate(log_iframes, false) {
                        debug_log(&format!("[njav] Attempt {} - All iframes: {:?}", attempt, res.value));
                    }
                    // Dump page title and URL
                    let info_script = r#"JSON.stringify({url: window.location.href, title: document.title, iframes: document.querySelectorAll('iframe').length})"#;
                    if let Ok(res) = tab.evaluate(info_script, false) {
                        debug_log(&format!("[njav] Attempt {} - Page info: {:?}", attempt, res.value));
                    }
                }
            }

            // === PHASE 2: Extract m3u8 from iframe ===
            if let Some(ref surrit) = surrit_url {
                debug_log(&format!("[njav] Found iframe: {}", surrit));
                self.emit_progress(app_handle, "Loading video player...", 75);

                // Try clicking on the main page first
                let _ = tab.evaluate(click_script, false);
                std::thread::sleep(Duration::from_secs(2));

                // Check PerformanceObserver on main page first
                for attempt in 1..=5 {
                    std::thread::sleep(Duration::from_secs(2));
                    if let Some(found_url) = self.check_for_video_url(&tab) {
                        debug_log(&format!("[njav] Found m3u8 via PerformanceObserver (attempt {}): {}", attempt, found_url));
                        self.extract_cookies(&tab);
                        self.emit_progress(app_handle, "Found video URL!", 100);
                        return Ok(Some(found_url));
                    }
                }

                // === PHASE 3: Navigate directly to surrit iframe URL ===
                debug_log(&format!("[njav] Navigating directly to: {}", surrit));
                if let Ok(_) = tab.navigate_to(surrit) {
                    let _ = tab.wait_until_navigated();
                    debug_log(&format!("[njav] Navigated to: {:?}", tab.get_url()));

                    // Re-inject observer on the surrit page
                    let _ = tab.evaluate(observer_script, false);
                    std::thread::sleep(Duration::from_secs(3));

                    // Click play on surrit page
                    let _ = tab.evaluate(click_script, false);
                    std::thread::sleep(Duration::from_secs(3));

                    // Check for m3u8 URL
                    if let Some(found_url) = self.check_for_video_url(&tab) {
                        debug_log(&format!("[njav] Found m3u8 from surrit page: {}", found_url));
                        self.extract_cookies(&tab);
                        self.emit_progress(app_handle, "Found video URL!", 100);
                        return Ok(Some(found_url));
                    }

                    // Retry with more clicks
                    for attempt in 1..=8 {
                        debug_log(&format!("[njav] surrit click attempt {}/8", attempt));
                        let _ = tab.evaluate(click_script, false);
                        std::thread::sleep(Duration::from_secs(3));

                        if let Some(found_url) = self.check_for_video_url(&tab) {
                            debug_log(&format!("[njav] Found m3u8 (surrit retry {}): {}", attempt, found_url));
                            self.extract_cookies(&tab);
                            self.emit_progress(app_handle, "Found video URL!", 100);
                            return Ok(Some(found_url));
                        }

                        // Dump HTML to analyze
                        if attempt == 4 {
                            let dump_script = "document.documentElement.outerHTML.substring(0, 5000)";
                            if let Ok(res) = tab.evaluate(dump_script, false) {
                                if let Some(val) = res.value {
                                    if let Ok(html) = serde_json::from_value::<String>(val) {
                                        let _ = std::fs::write("/tmp/njav_surrit_dump.html", &html);
                                        debug_log(&format!("[njav] Dumped surrit HTML ({} chars)", html.len()));
                                    }
                                }
                            }
                        }
                    }

                    // === PHASE 4: Extract m3u8 from HTML source of surrit page ===
                    debug_log("[njav] Extracting m3u8 from surrit page HTML...");
                    if let Some(m3u8) = self.extract_m3u8_from_html(&tab) {
                        debug_log(&format!("[njav] Found m3u8 in HTML source: {}", m3u8));
                        self.extract_cookies(&tab);
                        self.emit_progress(app_handle, "Found video URL from HTML!", 100);
                        return Ok(Some(m3u8));
                    }
                }
            } else {
                debug_log("[njav] No video iframe found after polling");
                // Try generic detection as fallback
            }

            // Fall through to generic detection
            eprintln!("[ChromeDetector] njav specific methods failed, trying general detection...");
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
    /// Extract m3u8 URL directly from page HTML source.
    fn extract_m3u8_from_html(&self, tab: &headless_chrome::Tab) -> Option<String> {
        let html_m3u8_script = r#"
            (function() {
                const html = document.documentElement.outerHTML;
                const results = [];
                const re1 = /https?:[\/\\]{0,2}[^"'\\\s]+\.m3u8[^"'\\\s]*/gi;
                let m;
                while ((m = re1.exec(html)) !== null) results.push(m[0].replace(/\\\//g, '/'));
                const re2 = /"file"\s*:\s*"([^"]*\.m3u8[^"]*)"/gi;
                while ((m = re2.exec(html)) !== null) results.push(m[1].replace(/\\\//g, '/'));
                const re3 = /src\s*=\s*["']([^"']*\.m3u8[^"']*)["']/gi;
                while ((m = re3.exec(html)) !== null) results.push(m[1]);
                const re5 = /"videoUrl"\s*:\s*"([^"]+)"/gi;
                while ((m = re5.exec(html)) !== null) results.push(m[1].replace(/\\\//g, '/'));
                return JSON.stringify([...new Set(results)]);
            })()
        "#;

        if let Ok(result) = tab.evaluate(html_m3u8_script, false) {
            if let Some(val) = result.value {
                if let Ok(urls) = serde_json::from_value::<Vec<String>>(val) {
                    for url in urls {
                        if !url.is_empty() && url.contains("m3u8") && !url.contains("google") && !url.contains("analytics") {
                            let clean = url.replace("\\/", "/").replace("\\u0026", "&").replace("&amp;", "&");
                            eprintln!("[ChromeDetector] Found m3u8 in HTML source: {}", clean);
                            return Some(clean);
                        }
                    }
                }
            }
        }

        // Dump HTML for analysis
        if let Ok(result) = tab.evaluate("document.documentElement.outerHTML", false) {
            if let Some(val) = result.value {
                if let Ok(html) = serde_json::from_value::<String>(val) {
                    let _ = std::fs::write("/tmp/njav_html_analysis.html", &html);
                    debug_log(&format!("[njav] Dumped full HTML ({} chars)", html.len()));
                    if let Some(m3u8) = Self::extract_m3u8_rust_regex(&html) {
                        debug_log(&format!("[njav] Found m3u8 via Rust regex: {}", m3u8));
                        return Some(m3u8);
                    }
                }
            }
        }
        None
    }

    /// Rust-side regex to find m3u8 URLs in HTML
    fn extract_m3u8_rust_regex(html: &str) -> Option<String> {
        let patterns = [
            r"https?://[^\s\"'<>]+\.m3u8[^\s\"'<>]*",
            r"https?://[^\s\"'<>]+m3u8[^\s\"'<>]*",
        ];
        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(mat) = re.find(html) {
                    let url = mat.as_str();
                    if !url.contains("google") && !url.contains("analytics") && !url.contains("gstatic") {
                        return Some(url.to_string());
                    }
                }
            }
        }
        None
    }


    /// Extract m3u8 URL directly from page HTML source.
    /// This is the most reliable fallback when PerformanceObserver fails
    /// (e.g., cross-origin iframe, obfuscated requests, etc.)
    fn extract_m3u8_from_html(&self, tab: &headless_chrome::Tab) -> Option<String> {
        // Multi-pattern m3u8 extraction from HTML
        let html_m3u8_script = r#"
            (function() {
                const html = document.documentElement.outerHTML;
                const results = [];

                // Pattern 1: Direct m3u8 URL (handles escaped slashes)
                const re1 = /https?:[\/\\]{0,2}[^"'\\\s]+\.m3u8[^"'\\\s]*/gi;
                let m;
                while ((m = re1.exec(html)) !== null) {
                    results.push(m[0].replace(/\\\//g, '/'));
                }

                // Pattern 2: "file" property with m3u8
                const re2 = /"file"\s*:\s*"([^"]*\.m3u8[^"]*)"/gi;
                while ((m = re2.exec(html)) !== null) {
                    results.push(m[1].replace(/\\\//g, '/'));
                }

                // Pattern 3: src with m3u8
                const re3 = /src\s*=\s*["']([^"']*\.m3u8[^"']*)["']/gi;
                while ((m = re3.exec(html)) !== null) {
                    results.push(m[1]);
                }

                // Pattern 4: url with m3u8
                const re4 = /url:\s*["']([^"']*\.m3u8[^"']*)["']/gi;
                while ((m = re4.exec(html)) !== null) {
                    results.push(m[1]);
                }

                // Pattern 5: videoUrl from player config
                const re5 = /"videoUrl"\s*:\s*"([^"]+)"/gi;
                while ((m = re5.exec(html)) !== null) {
                    results.push(m[1].replace(/\\\//g, '/'));
                }

                // Deduplicate and filter
                const unique = [...new Set(results)];
                return JSON.stringify(unique);
            })()
        "#;

        if let Ok(result) = tab.evaluate(html_m3u8_script, false) {
            if let Some(val) = result.value {
                if let Ok(urls) = serde_json::from_value::<Vec<String>>(val) {
                    for url in urls {
                        if !url.is_empty()
                            && url.contains("m3u8")
                            && !url.contains("google")
                            && !url.contains("analytics")
                        {
                            let clean = url
                                .replace("\\/", "/")
                                .replace("\\u0026", "&")
                                .replace("&amp;", "&");
                            eprintln!("[ChromeDetector] Found m3u8 in HTML source: {}", clean);
                            return Some(clean);
                        }
                    }
                }
            }
        }

        // Also dump the full HTML for analysis
        let dump_script = "document.documentElement.outerHTML";
        if let Ok(result) = tab.evaluate(dump_script, false) {
            if let Some(val) = result.value {
                if let Ok(html) = serde_json::from_value::<String>(val) {
                    let _ = std::fs::write("/tmp/njav_html_analysis.html", &html);
                    debug_log(&format!("[njav] Dumped full HTML for analysis ({} chars)", html.len()));

                    // Also try Rust-side regex as a last resort
                    if let Some(m3u8) = Self::extract_m3u8_rust_regex(&html) {
                        debug_log(&format!("[njav] Found m3u8 via Rust regex: {}", m3u8));
                        return Some(m3u8);
                    }
                }
            }
        }

        None
    }

    /// Rust-side regex to find m3u8 URLs in HTML
    fn extract_m3u8_rust_regex(html: &str) -> Option<String> {
        let patterns = [
            r"https?://[^\s\"'<>]+\.m3u8[^\s\"'<>]*",
            r"https?://[^\s\"'<>]+m3u8[^\s\"'<>]*",
        ];

        for pattern in &patterns {
            if let Ok(re) = regex::Regex::new(pattern) {
                if let Some(mat) = re.find(html) {
                    let url = mat.as_str();
                    if !url.contains("google") && !url.contains("analytics") && !url.contains("gstatic") {
                        return Some(url.to_string());
                    }
                }
            }
        }
        None
    }


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
