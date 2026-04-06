// Direct test of Chrome detector for NjavTV
use headless_chrome::{Browser, LaunchOptions};
use std::time::Duration;

fn main() -> Result<(), String> {
    let test_url = "https://njavtv.com/th/dass-812-uncensored-leak";
    println!("[Test] Testing NjavTV Chrome detector...");
    
    // Launch Chrome
    let browser = Browser::new(LaunchOptions {
        headless: true,
        sandbox: false,
        args: vec![
            std::ffi::OsStr::new("--no-sandbox"),
            std::ffi::OsStr::new("--disable-dev-shm-usage"),
            std::ffi::OsStr::new("--disable-gpu"),
            std::ffi::OsStr::new("--window-size=1920,1080"),
            std::ffi::OsStr::new("--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"),
        ],
        ..Default::default()
    }).map_err(|e| format!("Failed to launch Chrome: {}", e))?;
    
    let tab = browser.new_tab().map_err(|e| format!("Failed to create tab: {}", e))?;
    
    println!("[Test] Navigating to: {}", test_url);
    tab.navigate_to(test_url).map_err(|e| format!("Navigation failed: {}", e))?;
    tab.wait_until_navigated().map_err(|e| format!("Navigation failed: {}", e))?;
    
    // Wait for page to load
    std::thread::sleep(Duration::from_secs(5));
    
    // Check window.hls
    println!("[Test] Checking window.hls...");
    let hls_check = r#"
        (function() {
            if (window.hls && window.hls.url) {
                return JSON.stringify({
                    url: window.hls.url,
                    hasConfig: !!window.hls.config,
                    currentLevel: window.hls.currentLevel,
                    levels: window.hls.levels?.length
                });
            }
            return JSON.stringify({ exists: false });
        })()
    "#;
    
    if let Ok(result) = tab.evaluate(hls_check, false) {
        if let Some(value) = result.value {
            println!("[Test] window.hls result: {}", value);
            
            if let Ok(hls_info) = serde_json::from_value::<serde_json::Value>(value) {
                if let Some(url) = hls_info["url"].as_str() {
                    println!("[✓] Found m3u8 URL: {}", url);
                    
                    // Extract cookies
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
                                println!("[Test] Extracted {} cookies:", cookies.len());
                                for (name, value) in &cookies {
                                    println!("  {}={}", name, &value[..value.len().min(30)]);
                                }
                            }
                        }
                    }
                    
                    println!("\n[Test] SUCCESS! NjavTV detection works correctly.");
                    return Ok(());
                }
            }
        }
    }
    
    println!("[✗] Failed to find window.hls.url");
    Err("window.hls.url not found".to_string())
}
