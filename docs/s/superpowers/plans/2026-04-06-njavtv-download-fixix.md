# NjavTV Video Download Fix Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Enable video downloading from njavtv.com by detecting the actual video URL using headless Chrome browser, intercepting network requests, and then downloading with FFplay.

**Architecture:** Launch headless Chrome (via `headless_chrome` crate), navigate to the NjavTV video page, inject JS to intercept XHR/fetch requests to find m3u8/master playlist URLs, Click the play button, and extract the m3u8 URL, and pass to FFmpeg for download. This NjavTV uses Cloudflare-protected blob URLs, so simple HTTP requests fail — we need a real browser-based detection.

**Tech Stack:** Rust/headless_chrome (crate) + tokio + ffplay (bundled)

---

## Problem analysis

เว็บไซต์์์นjavtv.com ใช้ **Cloudflare protection + blob URLs**:
- Cloudflare blocks simple HTTP GET requests, returns challenge page or requiring JavaScript execution
- Video player uses HLS.js library that creates blob URLs (`blob:https://...`)
- Blob URLs can't be accessed externally — only via browser internals

### Current approach (from `chrome_detector.rs`)
The 6-layer detection strategy tries to find m3u8 URLs:
1. PerformanceObserver (network requests)
2. `performance.getEntriesByType('resource')` (existing entries)
3. Video tag inspection (`<video src>`)
4. HTML regex search
5. Page config extraction

**Problem:** All 5 methods **fail** because:
- Cloudflare returns challenge page → no video element exists
- Blob URLs are created by HLS.js, so performance API only sees the HLS source, not the m3u8 directly
- No network requests appear in performance entries for blob URLs

### Root cause
The PerformanceObserver intercepts **resource load** events, not XHR responses
- Video tag `src` attribute is set to `blob:` URL
- HTML never contains raw m3u8 URL — it it embedded in JS/iframe

## solution

**Use Chrome DevTools Protocol (CDP) to intercept network traffic at**
- CDP's `Network.enable` captures actual HTTP requests (XHR/fetch)
- Intercept responses to read m3u8 URLs
- Blob URL resolution via fetching the actual HLS playlist
- Alternative: inject custom XHR/fetch interceptors before page JS loads

## File structure

```
src-tauri/src/
├── chrome_detector.rs      # MODIFY: add CDP network interception
├── njavtv_parser.rs        # modify: (minor) use CDP intercepted URLs)
├── lib.rs                # modify: (minor) wire up)
└── downloader.rs           # no changes needed (already handles m3u8)
```

---

### task 1: CDP Network Interception  chrome_detector.rs

**Files:**
- modify: `src-tauri/src/chrome_detector.rs`

**What:** Add Chrome DevTools Protocol (CDP) `Network.enable` to capture actual HTTP traffic (XHR/fetch) including responses). This is the raw m3u8 URLs that Cloudflare protection and blob URLs.

- [ ] **Step 1: Add `enable_cdp_network` method to `ChromeVideoDetector`**

Add a `use headless_chrome::protocol::cdp::Network;` to imports list.

```rust
impl ChromeVideoDetector {
    // ... existing fields ...
    /// Store captured m3u8 URLs from CDP network interception
    captured_m3u8_urls: Mutex<Vec<String>>,
}
```

- [ ] **Step 2: Initialize captured_m3u8_urls in `ChromeVideoDetector::new()`**

In `fn new()`, add `captured_m3u8_urls: Mutex::new(Vec::new())` field.

- [ ] **Step 3: Enable CDP network after tab creation in `detect_video_url`**

In `fn detect_video_url`, after creating tab:
```rust
// Enable CDP Network domain to intercept requests
let network = headless_chrome::protocol::cdp::Network;
let _ = tab.call_method(network.enable(None)).map_err(|e| format!("CDP Network.enable failed: {}", e))?;
```

- [ ] **Step 4: After page load, collect captured m3u8 URLs**

In `detect_video_url`, after trying all methods, add fallback to CDP:
```rust
// Check CDP-captured m3u8 URLs
{
    let urls = self.captured_m3u8_urls.lock().unwrap();
    if let Some(url) = urls.iter().find(|u| u.contains(".m3u8")) {
        eprintln!("[ChromeDetector] Found m3u8 via CDP: {}", url);
        return Some(url.clone());
    }
}
```

- [ ] **Step 5: Update `NjavTV` path in `chrome_detector.rs` to check CDP URLs first**

In the NjavTV section, modify:
```rust
if is_njavtv {
    // ... existing code ...
    // Check CDP-captured URLs first
    let urls = self.captured_m3_8_urls.lock().unwrap();
    if let Some(url) = urls.iter().find(|u| u.contains(".m3u8")) {
        eprintln!("[ChromeDetector] NjavTV: Found m3u8 via CDP: {}", url);
        self.emit_progress(app_handle, "Found video URL via CDP!", 100);
        return Some(url.clone());
    }
    // ... rest of existing NjavTV detection ...
```

- [ ] **Step 6: Clear captured URLs between detections**

Add `pub fn clear_captured_urls(&mut self)`:
```rust
pub fn clear_captured_urls(&mut self) {
    let mut urls = self.captured_m3u8_urls.lock().unwrap();
    urls.clear();
    *self.captured_m3u8_urls = Mutex::new(urls);
}
```

- [ ] **Step 7: Test with actual NjavTV URL**

Run: `cargo test -p src-tauri -- --nocapture`

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/chrome_detector.rs
git commit -m "feat(njavtv): add CDP network interception for reliable m3u8 extraction from Cloudflare-protected blob URLs"
```

---

### task 2: update njavtv_parser.rs to prefer CDP URLs

**files:**
- modify: `src-tauri/src/njavtv_parser.rs`

**what:** Minor cleanup — `njavtv_parser.rs` doesn't need to handle any video URL extraction (that's Chrome detector's job). But it parser just returns page URLs and Chrome detector does handle the actual m3u8 detection.

- [ ] **Step 1: review `njavtv_parser.rs` and verify no video URL logic is needed**

- [ ] **step 2: remove any unused video extraction methods (if any)**

- [ ] **step 3: commit**

```bash
git add src-tauri/src/njavtv_parser.rs
git commit -m "refactor(njavtv): simplify parser to only return page URLs"
```

---

### task 3: integration test with live URL

**files:**
- no file changes

**what:** Verify the full pipeline works with `https://njavtv.com/th/dass-812-uncensored-leak`

- [ ] **Step 1: build and run app**

```bash
cd src-tauri && cargo test -p src-tauri -- --nocapture 2>&1 | head -20
npm run tauri dev
```

- [ ] **step 2: paste URL in app, click Fetch, and Download**

- [ ] **step 3: verify m3u8 URL is detected and video downloads**

- [ ] **step 4: if issues, debug and iterate**

---

### task 4: handle edge cases and robustness

**files:**
- modify: `src-tauri/src/chrome_detector.rs`

**what:** Add timeout, retry, and cleanup for robustness

- [ ] **Step 1: add retry logic for CDP network enable (retry up to 3 times)**

- [ ] **Step 2: add timeout for m3u8 URL collection (max 30s)**

- [ ] **Step 3: add cleanup for Drop implementation**

- [ ] **Step 4: test again with live URL**

- [ ] **Step 5: commit**

```bash
git add src-tauri/src/chrome_detector.rs src-tauri/src/njavtv_parser.rs
git commit -m "fix(njavtv): add retry, timeout, and cleanup for reliable Cloudflare bypass"
```

---

### task 5: verify with actual URL (ทดสอด812)

**files:**
- no file changes

**what:** Final verification that download works end-to-end

- [ ] **Step 1: test with `https://njavtv.com/th/dass-812-uncensored-leak`**

- [ ] **step 2: verify video file is valid**

- [ ] **step 3: final commit**

---

## Key technical decisions

1. **CDP Network over PerformanceObserver:** CDP operates at a lower level (intercepts requests only, not responses), but It directly captures responses (which contain m3u8 URLs) from XHR/fetch. PerformanceObserver captures resource loads only (the response body).
2. **Why CDP + existing fallbacks:** CDP gives us actual network traffic visibility, PerformanceObserver only sees resource timing. When the site creates blob URLs, the m3u8 URL is in an XHR response, not the the resource timing. So CDP catches both request + response.
 the **When to fallback:** If CDP fails (older Chrome versions), we still have PerformanceObserver + video tag + regex search as fallbacks.
 PerformanceObserver alone is usually sufficient because blob URLs have become so common.

3. **Blob URL handling:** We don't try to resolve blob URLs. Instead, we capture the actual m3u8 URL from network traffic and then use FFmpeg directly. This avoids needing a serve blob content through the JS.
 which would be extremely fragile.

4. **Thread safety:** CDP network interception runs in the Chrome process (main thread). It doesn't block the page rendering. The The `captured_m3u8_urls` is cleared between detections to avoid memory leaks.
