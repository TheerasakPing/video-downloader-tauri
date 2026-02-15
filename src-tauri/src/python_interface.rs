use headless_chrome::{Browser, LaunchOptions, Tab};
use std::collections::HashMap;
use std::time::Duration;
use crate::UnifiedSeriesInfo;
use serde::Deserialize;

// Helper to initialize browser with specific flags
fn init_browser() -> Result<Browser, String> {
    eprintln!("[357ms] Launching headless Chrome...");

    let mut args = Vec::new();
    args.push(std::ffi::OsStr::new("--no-sandbox"));
    args.push(std::ffi::OsStr::new("--disable-dev-shm-usage"));
    args.push(std::ffi::OsStr::new("--disable-gpu"));
    args.push(std::ffi::OsStr::new("--disable-software-rasterizer"));
    args.push(std::ffi::OsStr::new("--disable-extensions"));
    args.push(std::ffi::OsStr::new("--window-size=390,844")); // Mobile viewport size as in Python script
    args.push(std::ffi::OsStr::new("--autoplay-policy=no-user-gesture-required"));
    args.push(std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1"));

    let options = LaunchOptions {
        headless: true,
        sandbox: false,
        args: args,
        ..Default::default()
    };

    Browser::new(options)
        .map_err(|e| format!("Failed to launch Chrome: {}", e))
}

pub fn fetch_357ms_series(url: &str) -> Result<UnifiedSeriesInfo, String> {
    let browser = init_browser()?;
    let tab = browser.new_tab().map_err(|e| format!("Failed to create tab: {}", e))?;

    eprintln!("[357ms] Navigating to series: {}", url);
    tab.navigate_to(url).map_err(|e| format!("Failed to navigate: {}", e))?;
    tab.wait_until_navigated().map_err(|e| format!("Navigation failed: {}", e))?;

    // Wait for content (Python script waited 5s)
    std::thread::sleep(Duration::from_secs(5));

    // Script to extract series info
    let extract_script = r#"
        (function() {
            const info = {
                title: "Unknown Series",
                cover_url: null,
                episodes: []
            };

            // Title
            let h1 = document.querySelector("h1");
            if (h1) info.title = h1.innerText.trim();
            else if (window.seriesTitle) info.title = window.seriesTitle;

            // Cover URL - Try multiple sources
            // 1. OG Image (Most reliable)
            const ogImage = document.querySelector('meta[property="og:image"]');
            if (ogImage && ogImage.content) {
                info.cover_url = ogImage.content;
            }

            // 2. Common poster/cover classes if OG failed
            if (!info.cover_url) {
                const imgSelectors = [
                    ".series-cover img", 
                    ".poster img", 
                    ".cover img", 
                    "img.cover", 
                    "img.poster",
                    ".movie-poster img",
                    "div[class*='poster'] img",
                    "div[class*='cover'] img"
                ];
                
                for (const sel of imgSelectors) {
                    const img = document.querySelector(sel);
                    if (img && img.src) {
                        info.cover_url = img.src;
                        break;
                    }
                }
            }

            // Episodes
            const items = document.querySelectorAll("a.ep-card");
            const seen = new Set();
            
            items.forEach(item => {
                let href = item.getAttribute("href");
                if (!href) return;
                
                if (!href.startsWith("http")) {
                    href = new URL(href, document.location.href).href;
                }

                let epText = "0";
                const epNoEl = item.querySelector(".ep-number");
                if (epNoEl) {
                    epText = epNoEl.innerText.replace("EP.", "").trim();
                } else {
                    epText = item.getAttribute("data-ep") || "0";
                }

                let epNum = parseInt(epText);
                if (isNaN(epNum)) epNum = 0;

                // Dedup by URL
                if (!seen.has(href)) {
                    seen.add(href);
                    info.episodes.push({
                        number: epNum,
                        title: epText,
                        url: href
                    });
                }
            });

            // Sort
            info.episodes.sort((a, b) => a.number - b.number);

            return JSON.stringify(info);
        })()
    "#;

    let remote_object = tab.evaluate(extract_script, false)
        .map_err(|e| format!("Failed to evaluate script: {}", e))?;
    
    let result_json = remote_object.value
        .ok_or("No value returned from script")?;
    
    #[derive(Deserialize)]
    struct JsEpisode {
        number: i32,
        #[allow(dead_code)]
        title: String,
        url: String,
    }

    #[derive(Deserialize)]
    struct JsSeriesInfo {
        title: String,
        cover_url: Option<String>,
        episodes: Vec<JsEpisode>,
    }

    // Handle both direct object (if supported) and stringified JSON
    let info: JsSeriesInfo = if let Some(json_str) = result_json.as_str() {
        serde_json::from_str(json_str)
            .map_err(|e| format!("Failed to parse JSON string: {} (Content: {})", e, json_str))?
    } else {
        serde_json::from_value(result_json)
            .map_err(|e| format!("Failed to parse JSON object: {}", e))?
    };

    let mut episode_urls = HashMap::new();
    for ep in info.episodes {
        episode_urls.insert(ep.number, ep.url);
    }

    eprintln!("[357ms] Found {} episodes for {}", episode_urls.len(), info.title);

    Ok(UnifiedSeriesInfo {
        series_id: 0,
        title: info.title,
        total_episodes: episode_urls.len() as i32,
        poster_url: info.cover_url,
        episode_urls,
        source: "357ms".to_string(),
    })
}

pub fn extract_357ms_video(url: &str) -> Result<(String, HashMap<String, String>), String> {
    let browser = init_browser()?;
    let tab = browser.new_tab().map_err(|e| format!("Failed to create tab: {}", e))?;

    eprintln!("[357ms] Extracting video for: {}", url);

    // 1. Setup PerformanceObserver to catch m3u8 requests
    // Using a simpler approach than chrome_detector: just evaluate script to start observing immediately
    // Ideally we should inject this on new document, but headless_chrome makes that hard synchronously.
    // We will navigate, then inject, then wait/reload if needed? 
    // Actually, `ChromeVideoDetector` shows we can just navigate then inject.

    tab.navigate_to(url).map_err(|e| format!("Failed to navigate: {}", e))?;
    tab.wait_until_navigated().map_err(|e| format!("Navigation failed: {}", e))?;

    let observer_script = r#"
        (function() {
            if (performance.setResourceTimingBufferSize) {
                performance.setResourceTimingBufferSize(5000);
            }
            window.__FOUND_URLS = [];
            const observer = new PerformanceObserver((list) => {
                list.getEntries().forEach((entry) => {
                    if (entry.name.includes('.m3u8')) {
                        window.__FOUND_URLS.push(entry.name);
                    }
                });
            });
            observer.observe({ entryTypes: ['resource'] });
            
            // Also check existing
            performance.getEntriesByType('resource').forEach(e => {
                if (e.name.includes('.m3u8')) window.__FOUND_URLS.push(e.name);
            });
        })()
    "#;
    let _ = tab.evaluate(observer_script, false);

    // Wait for JS to load (iframe to load)
    std::thread::sleep(Duration::from_secs(5));

    // Try to find m3u8 in captured requests
    let check_urls_script = "JSON.stringify(window.__FOUND_URLS || [])";
    if let Ok(res) = tab.evaluate(check_urls_script, false) {
        if let Some(val) = res.value {
            // Handle stringified JSON array
            let urls: Option<Vec<String>> = if let Some(json_str) = val.as_str() {
                serde_json::from_str(json_str).ok()
            } else {
                serde_json::from_value(val).ok()
            };

            if let Some(urls) = urls {
                if let Some(m3u8) = urls.last() {
                    eprintln!("[357ms] Found m3u8 via network: {}", m3u8);
                    return Ok((m3u8.clone(), get_headers(&tab)));
                }
            }
        }
    }

    // 2. If not found, look for iframes (baiwarp) and navigate into them
    let iframe_script = r#"
        (function() {
            const iframes = document.querySelectorAll('iframe');
            for (let f of iframes) {
                let src = f.src || f.getAttribute('data-lazy-src');
                if (src && (src.includes('baiwarp') || src.includes('embed'))) {
                    return src;
                }
            }
            return null;
        })()
    "#;

    if let Ok(res) = tab.evaluate(iframe_script, false) {
        if let Some(val) = res.value {
            if let Some(iframe_src) = val.as_str() {
                let mut target = iframe_src.to_string();
                if target.starts_with("//") {
                    target = format!("https:{}", target);
                }
                eprintln!("[357ms] Found iframe, navigating to: {}", target);
                
                tab.navigate_to(&target).map_err(|e| format!("Failed to navigate to iframe: {}", e))?;
                tab.wait_until_navigated().map_err(|e| format!("Iframe nav failed: {}", e))?;
                std::thread::sleep(Duration::from_secs(5));

                // Check for playerConfig
                let config_script = r#"
                    (function() {
                        if (window.playerConfig && window.playerConfig.medias && window.playerConfig.medias.original && window.playerConfig.asset) {
                            return `https://${window.playerConfig.asset}/${window.playerConfig.medias.original}/video.m3u8`;
                        }
                        return null;
                    })()
                "#;
                if let Ok(res) = tab.evaluate(config_script, false) {
                    if let Some(val) = res.value {
                         if let Some(m3u8) = val.as_str() {
                             eprintln!("[357ms] Found m3u8 via playerConfig: {}", m3u8);
                             return Ok((m3u8.to_string(), get_headers(&tab)));
                         }
                    }
                }

                // Fallback regex on iframe content
                let regex_script = r#"
                    (function() {
                        const html = document.documentElement.outerHTML;
                        const match = html.match(/["'](https?:[^"']+\.m3u8[^"']*)["']/);
                        return match ? match[1].replace(/\\\//g, '/') : null;
                    })()
                "#;
                if let Ok(res) = tab.evaluate(regex_script, false) {
                    if let Some(val) = res.value {
                         if let Some(m3u8) = val.as_str() {
                             eprintln!("[357ms] Found m3u8 via regex: {}", m3u8);
                             return Ok((m3u8.to_string(), get_headers(&tab)));
                         }
                    }
                }
            }
        }
    }

    // 3. Fallback regex on main page
    let regex_script = r#"
        (function() {
            const html = document.documentElement.outerHTML;
            const match = html.match(/["'](https?:[^"']+\.m3u8[^"']*)["']/);
            return match ? match[1].replace(/\\\//g, '/') : null;
        })()
    "#;
    if let Ok(res) = tab.evaluate(regex_script, false) {
        if let Some(val) = res.value {
                if let Some(m3u8) = val.as_str() {
                    eprintln!("[357ms] Found m3u8 via regex on main page: {}", m3u8);
                    return Ok((m3u8.to_string(), get_headers(&tab)));
                }
        }
    }

    Err("Could not find video URL".to_string())
}

fn get_headers(tab: &Tab) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    
    // Get User-Agent
    let ua_script = "navigator.userAgent";
    if let Ok(res) = tab.evaluate(ua_script, false) {
        if let Some(val) = res.value {
            if let Some(ua) = val.as_str() {
                headers.insert("User-Agent".to_string(), ua.to_string());
            }
        }
    }

    // Get Cookies
    if let Ok(cookies) = tab.get_cookies() {
        let cookie_str = cookies.iter()
            .map(|c| format!("{}={}", c.name, c.value))
            .collect::<Vec<_>>()
            .join("; ");
        if !cookie_str.is_empty() {
             headers.insert("Cookie".to_string(), cookie_str);
        }
    }
    
    // Referer (Current URL)
    let url = tab.get_url();
    headers.insert("Referer".to_string(), url.clone());
    
    // Origin
    if let Ok(parsed) = url::Url::parse(&url) {
        let origin = format!("{}://{}", parsed.scheme(), parsed.domain().unwrap_or(""));
        headers.insert("Origin".to_string(), origin);
    }

    headers
}