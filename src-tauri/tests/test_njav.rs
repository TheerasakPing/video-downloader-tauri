use std::time::Duration;
use headless_chrome::{Browser, LaunchOptions};
use headless_chrome::protocol::cdp::Page::{AddScriptToEvaluateOnNewDocument, Navigate};
use std::ffi::OsStr;

/// Full E2E test: javxx.com → extract surrit.store URL → navigate with Referer → m3u8
///
/// Key insight: surrit.store's player script (script-CuQ7tx8E.js) checks the Referer header.
/// Without a javxx.com Referer, the script redirects to javxx.com/th after 1 second.
/// With the Referer, it initializes the HLS player and loads the video.
#[tokio::test]
async fn test_njav_full_m3u8_extraction() {
    let mut args = Vec::new();
    args.push(OsStr::new("--no-sandbox"));
    args.push(OsStr::new("--disable-dev-shm-usage"));
    args.push(OsStr::new("--disable-gpu"));
    args.push(OsStr::new("--disable-blink-features=AutomationControlled"));
    args.push(OsStr::new("--disable-infobars"));
    args.push(OsStr::new("--disable-web-security"));
    args.push(OsStr::new("--headless=new"));
    args.push(OsStr::new("--autoplay-policy=no-user-gesture-required"));
    args.push(OsStr::new("--window-size=1920,1080"));
    args.push(OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36"));

    let options = LaunchOptions {
        headless: false,
        sandbox: false,
        args: args,
        ignore_default_args: vec![OsStr::new("--enable-automation")],
        ..Default::default()
    };

    let browser = Browser::new(options).expect("Failed to launch Chrome");
    let tab = browser.new_tab().expect("Failed to create tab");

    // Step 1: Navigate to javxx.com and extract surrit.store iframe URL from DOM
    println!("[test] Step 1: Navigating to javxx.com to extract surrit.store URL...");
    let _ = tab.navigate_to("https://javxx.com/en/v/snos-034");
    let _ = tab.wait_until_navigated();
    std::thread::sleep(Duration::from_secs(3));

    let iframe_extract_script = r#"
        (function() {
            var iframes = document.querySelectorAll('iframe[src]');
            for (var i = 0; i < iframes.length; i++) {
                if (iframes[i].src.includes('surrit.store') || iframes[i].src.includes('wowstream')) {
                    return iframes[i].src;
                }
            }
            return null;
        })()
    "#;

    let mut surrit_url: Option<String> = None;
    for attempt in 1..=10 {
        if let Ok(result) = tab.evaluate(iframe_extract_script, false) {
            if let Some(value) = result.value {
                if value.is_string() {
                    if let Ok(url) = serde_json::from_value::<String>(value) {
                        if !url.is_empty() {
                            println!("[test] Found surrit.store URL (attempt {}): {}", attempt, url);
                            surrit_url = Some(url);
                            break;
                        }
                    }
                }
            }
        }
        println!("[test] Attempt {} - no surrit iframe in DOM yet...", attempt);
        std::thread::sleep(Duration::from_secs(2));
    }

    let surrit_url = surrit_url.expect("No surrit.store iframe URL found in DOM");

    // Step 2: Inject PerformanceObserver and m3u8 capture BEFORE navigation
    println!("[test] Step 2: Setting up m3u8 capture...");
    let capture_script = r#"
        if (performance.setResourceTimingBufferSize) performance.setResourceTimingBufferSize(5000);
        window.__FOUND_M3U8 = null;
        try {
            var observer = new PerformanceObserver(function(list) {
                list.getEntries().forEach(function(entry) {
                    if (entry.name.includes('.m3u8') || entry.name.includes('master')) {
                        window.__FOUND_M3U8 = entry.name;
                    }
                });
            });
            observer.observe({ entryTypes: ['resource'] });
        } catch(e) {}

        // Also intercept fetch/XHR for m3u8 URLs
        var _origFetch = window.fetch;
        window.fetch = function() {
            var url = typeof arguments[0] === 'string' ? arguments[0] :
                      (arguments[0] && arguments[0].url) ? arguments[0].url : '';
            if (url.indexOf('.m3u8') !== -1 || url.indexOf('master') !== -1) {
                window.__FOUND_M3U8 = url;
            }
            return _origFetch.apply(this, arguments);
        };

        var _origXHROpen = XMLHttpRequest.prototype.open;
        XMLHttpRequest.prototype.open = function(method, url) {
            if (url && (url.indexOf('.m3u8') !== -1 || url.indexOf('master') !== -1)) {
                window.__FOUND_M3U8 = url;
            }
            return _origXHROpen.apply(this, arguments);
        };
    "#;

    let _ = tab.call_method(AddScriptToEvaluateOnNewDocument {
        source: capture_script.to_string(),
        world_name: None,
        include_command_line_api: None,
        run_immediately: None,
    });

    // Step 3: Navigate to surrit.store WITH Referer header
    // The player script checks the Referer to verify it's embedded from javxx.com
    println!("[test] Step 3: Navigating to surrit.store with javxx.com Referer...");
    let nav_result = tab.call_method(Navigate {
        url: surrit_url.clone(),
        referrer: Some("https://javxx.com/".to_string()),
        transition_Type: None,
        frame_id: None,
        referrer_policy: None,
    });
    println!("[test] Navigate result: {:?}", nav_result);

    std::thread::sleep(Duration::from_secs(5));

    // Check current URL
    let current_url = tab.evaluate("window.location.href", false)
        .ok()
        .and_then(|r| r.value)
        .and_then(|v| serde_json::from_value::<String>(v).ok())
        .unwrap_or_default();
    println!("[test] Current URL: {}", current_url);

    // Step 4: Click play
    println!("[test] Step 4: Clicking play...");
    let click_script = r#"
        (function() {
            var selectors = ['.plyr__control--overlaid', 'button[aria-label="Play"]',
                            'button[data-plyr="play"]', 'button.plyr__control',
                            '.plyr__controls button', 'video'];
            var clicked = [];
            for (var i = 0; i < selectors.length; i++) {
                var els = document.querySelectorAll(selectors[i]);
                for (var j = 0; j < els.length; j++) {
                    try { els[j].click(); clicked.push(selectors[i]); } catch(e) {}
                }
            }
            document.querySelectorAll('video').forEach(function(v) {
                try { v.play(); clicked.push('video.play'); } catch(e) {}
            });
            try {
                var el = document.elementFromPoint(window.innerWidth / 2, window.innerHeight / 2);
                if (el) { el.click(); clicked.push('center:' + el.tagName); }
            } catch(e) {}

            // Check existing resources
            if (!window.__FOUND_M3U8) {
                performance.getEntriesByType('resource').forEach(function(e) {
                    if (e.name.includes('.m3u8') || e.name.includes('master')) {
                        window.__FOUND_M3U8 = e.name;
                    }
                });
            }

            return JSON.stringify({
                clicked: clicked,
                foundM3u8: window.__FOUND_M3U8,
                videoCount: document.querySelectorAll('video').length,
                plyrCount: document.querySelectorAll('.plyr').length,
                hlsUrl: (window.hls && window.hls.url) ? window.hls.url : null,
                resourceCount: performance.getEntriesByType('resource').length,
                url: window.location.href,
                title: document.title
            });
        })()
    "#;

    match tab.evaluate(click_script, false) {
        Ok(result) => {
            if let Some(value) = result.value {
                println!("[test] Click result: {}", value);
            }
        }
        Err(e) => println!("[test] Click failed: {}", e),
    }

    // Step 5: Poll for m3u8
    println!("[test] Step 5: Polling for m3u8...");
    let mut found_m3u8: Option<String> = None;

    for attempt in 1..=20 {
        std::thread::sleep(Duration::from_secs(2));

        let poll_script = r#"
            (function() {
                // Click play
                document.querySelectorAll('.plyr__control--overlaid, video').forEach(function(el) {
                    try { el.click(); } catch(e) {}
                });
                document.querySelectorAll('video').forEach(function(v) {
                    try { v.play(); } catch(e) {}
                });

                // Check HLS.js
                var hlsUrl = null;
                if (window.hls && window.hls.url) hlsUrl = window.hls.url;
                var videos = document.querySelectorAll('video');
                for (var i = 0; i < videos.length; i++) {
                    if (videos[i]._hls && videos[i]._hls.url) hlsUrl = videos[i]._hls.url;
                    if (videos[i].src && videos[i].src.includes('m3u8')) hlsUrl = videos[i].src;
                }

                // Check PerformanceObserver/fetch captures
                if (!window.__FOUND_M3U8) {
                    var entries = performance.getEntriesByType('resource');
                    for (var i = 0; i < entries.length; i++) {
                        if (entries[i].name.includes('.m3u8') || entries[i].name.includes('master')) {
                            window.__FOUND_M3U8 = entries[i].name;
                        }
                    }
                }

                // Check HTML for m3u8
                var htmlMatch = document.documentElement.outerHTML.match(/["'](https?:[^"']+\.m3u8[^"']*)["']/);
                var htmlM3u8 = htmlMatch ? htmlMatch[1] : null;

                return JSON.stringify({
                    hlsUrl: hlsUrl,
                    foundM3u8: window.__FOUND_M3U8,
                    htmlM3u8: htmlM3u8,
                    videoSrc: videos.length > 0 ? videos[0].src : null,
                    videoCurrentSrc: videos.length > 0 ? videos[0].currentSrc : null,
                    resourceCount: performance.getEntriesByType('resource').length,
                    url: window.location.href,
                    title: document.title
                });
            })()
        "#;

        match tab.evaluate(poll_script, false) {
            Ok(result) => {
                if let Some(value) = result.value {
                    if let Some(s) = value.as_str() {
                        if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(s) {
                            for key in &["hlsUrl", "foundM3u8", "htmlM3u8", "videoSrc", "videoCurrentSrc"] {
                                if let Some(url) = parsed[*key].as_str() {
                                    if url.contains(".m3u8") && url.starts_with("http") {
                                        println!("[test] Found m3u8 via {} (attempt {}): {}", key, attempt, url);
                                        found_m3u8 = Some(url.to_string());
                                        break;
                                    }
                                }
                            }

                            if found_m3u8.is_some() { break; }

                            if attempt <= 3 || attempt % 5 == 0 {
                                println!("[test] Attempt {} - {}", attempt, s);
                            }
                        }
                    }
                }
            }
            Err(e) => println!("[test] Poll failed (attempt {}): {}", attempt, e),
        }
    }

    assert!(found_m3u8.is_some(), "Should find m3u8 URL from surrit.store");
    let m3u8 = found_m3u8.unwrap();
    assert!(m3u8.contains(".m3u8"), "URL should be m3u8: {}", m3u8);
    println!("[test] SUCCESS - Final m3u8 URL: {}", m3u8);
}

/// Test: verify javxx.com is accessible via HTTP
#[tokio::test]
async fn test_javxx_http_accessible() {
    let client = reqwest::Client::builder()
        .user_agent("Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/131.0.0.0 Safari/537.36")
        .timeout(Duration::from_secs(15))
        .build()
        .unwrap();

    let resp = client.get("https://javxx.com/en/v/snos-034").send().await.unwrap();
    assert_eq!(resp.status(), reqwest::StatusCode::OK);
    let html = resp.text().await.unwrap();
    assert!(html.contains("WatchPlayer"), "javxx.com should contain WatchPlayer");
    println!("javxx.com HTTP test passed");
}
