# NjavTV Video Download Implementation

## Overview
The application can download videos from njavtv.com using a multi-stage detection and download process.

## Download Flow

### Stage 1: URL Detection
```
User Input: https://njavtv.com/th/dass-812-uncensored-leak
                ↓
    NjavtvParser::is_njavtv_url() ✓
                ↓
    NjavtvParser::get_series_info()
                ↓
    Returns NjavtvSeriesInfo with:
    - title: extracted from page
    - total_episodes: 1 (single video page)
    - direct_page_url: the input URL
```

### Stage 2: Chrome Video Detection
```
    ChromeVideoDetector::detect_video_url()
                ↓
    Detect "njavtv.com" in URL → Special handling mode
                ↓
    Navigate to page with headless Chrome
                ↓
    Wait 3 seconds for page load
                ↓
    Poll window.hls.url (up to 15 attempts, 2s each)
                ↓
    Found: https://surrit.com/{uuid}/playlist.m3u8
                ↓
    Extract cookies from browser
                ↓
    Return (m3u8_url, cookies)
```

### Stage 3: Video Download
```
    Downloader::download_episode()
                ↓
    Detect .m3u8 URL → HLS download mode
                ↓
    Try FFmpeg with cookies:
    - Pass cookies via Cookie header
    - Use Referer: https://njavtv.com/
    - Use browser User-Agent
    
    If FFmpeg fails with "mismatches" or "Invalid data":
                ↓
    Fallback to Manual HLS Download:
    - Fetch master playlist with reqwest (with cookies)
    - Parse playlist to find best quality
    - Extract segment URLs (.jpeg files)
    - Download each segment with reqwest (with cookies)
    - Concatenate segments into .ts file
    - Convert to .mp4 with FFmpeg
```

## Key Technical Details

### Why Manual HLS Fallback is Needed
NjavTV uses `.jpeg` file extensions for HLS segments to:
1. Bypass CDN caching restrictions
2. Prevent naive download tools from recognizing video content
3. Obfuscate the video stream

FFmpeg 8.x has strict format checking and rejects `.jpeg` files as MPEG-TS segments, even with `-allowed_segment_extensions ALL`. The error message contains "mismatches" which triggers the fallback.

### Cookie Handling
Cookies are critical for CDN authentication:
- `cf_clearance`: Cloudflare clearance token
- `missav_session`: Session identifier
- `XSRF-TOKEN`: CSRF token
- Various other tracking/auth cookies

These are extracted from Chrome after page load and passed to FFmpeg/reqwest via the Cookie header.

### Referer and User-Agent
- **Referer**: `https://njavtv.com/` (prevents hotlink protection)
- **User-Agent**: Chrome 120 on macOS (matches browser fingerprint)

## Testing

### Manual Test
```bash
# 1. Build the app
cargo build --release --manifest-path src-tauri/Cargo.toml

# 2. Run the app
npm run tauri dev

# 3. Paste URL in the UI
https://njavtv.com/th/dass-812-uncensored-leak

# 4. Click "Fetch Series"
# 5. Select episode
# 6. Click "Download"
# 7. Monitor logs for:
#    - "[ChromeDetector] NjavTV detected"
#    - "[ChromeDetector] NjavTV: Found m3u8 via window.hls.url"
#    - "[ManualHLS] Fetching master playlist" (if FFmpeg fallback triggers)
```

### Automated Test
```bash
node test_chrome_detector.mjs
```

Expected output:
```
✓ Found m3u8 URL: https://surrit.com/{uuid}/playlist.m3u8
```

## Troubleshooting

### "Could not find video URL"
- Page may require login
- Region-blocked content
- Cloudflare challenge too complex
- Check logs for Chrome detection progress

### "FFmpeg exited with error"
- Normal: FFmpeg will fail on `.jpeg` segments
- Should automatically fallback to manual download
- Check logs for "[ManualHLS]" messages

### Download stalls at 0%
- Cookie expiration (rare, cookies are fresh from Chrome)
- Network timeout (30s timeout is set)
- CDN issues (try again later)

## Files Involved
- `src-tauri/src/njavtv_parser.rs`: URL detection and page parsing
- `src-tauri/src/chrome_detector.rs`: Headless Chrome video detection
- `src-tauri/src/downloader.rs`: Download engine with HLS fallback
- `src-tauri/src/lib.rs`: Orchestration and IPC handlers
