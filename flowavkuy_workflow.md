# AVKuy.com Download Workflow

## Overview

AVKuy (www2.avkuy.com / av-kuy.com) เป็นเว็บที่มี Cloudflare protection และใช้ iframe player ฝังตัวจาก av-kuy.com เพื่อเล่นวิดีโอ ระบบใช้กลยุทธ์ 3 ชั้นเพื่อดึง video URL ออกมา:

1. **Tauri WebView** (primary) - ใช้ WKWebView จริงข้าม Cloudflare ได้ดีกว่า
2. **Chrome Detector** (fallback) - ใช้ headless Chrome CDP
3. **HTTP + Packer Unpack** (last resort) - ใช้ reqwest ดึง HTML แล้ว unpack JavaScript packer

---

## Flow Diagram

```
User paste URL (e.g. https://www2.avkuy.com/avsubthai-adn-750/)
        |
        v
[URL Detection] -- avkuy.com / av-kuy.com ตรงกับไหม?
        |
        +-- NO --> ไป parser อื่น (rongyok, hsck, njav, etc.)
        |
        +-- YES --> [Strategy 1: Tauri WebView]
                        |
                        v
                   [Phase A: Main Page Extraction]
                        |
                        v
                   +---------------------------+
                   | Create WebView Window     |
                   | (off-screen, 1024x768)    |
                   | Load avkuy.com URL        |
                   +---------------------------+
                        |
                        v
                   [Wait for Cloudflare]
                   (ตรวจจับ "Just a moment" / challenge-running)
                   (poll 20 รอบ, 2.5s/รอบ = ~50s max)
                        |
                        v
                   [Extract Metadata via JS]
                   - title: og:title / h1 / document.title
                   - poster: og:image / twitter:image / video[poster]
                   - iframe: iframe[src*='av-kuy.com/v/']
                        |
                        v
                   +-----+------+
                   | iframe URL |--- YES --> [Phase B: Player Page]
                   |  พบ?       |                |
                   +-----+------+                v
                         | NO               +---------------------------+
                         |                  | Open iframe URL in new    |
                         v                  | WebView window            |
                   [Strategy 2: Chrome]     +---------------------------+
                                              |
                                              v
                                         [Setup PerformanceObserver]
                                         (monitor .m3u8 / .mp4 requests)
                                              |
                                              v
                                         [Click + Poll Loop]
                                         (click play buttons, video, poster)
                                         (poll 14 รอบ, ~2.6s/รอบ)
                                              |
                                              v
                                         +----------+
                                         | Video URL |--- YES --> return
                                         | .m3u8/.mp4|              |
                                         |  พบ?      |              v
                                         +-----+-----+     [Build UnifiedSeriesInfo]
                                               | NO         - title
                                               v            - poster_url
                                         [window close]     - episode_urls[1] = video_url
                                                            - source: "avkuy"
                                                            - cookies
```

---

## Phase A: Main Page via WebView

**File**: `src-tauri/src/lib.rs` - `fetch_avkuy_via_webview()`

### Step 1: Create Hidden WebView Window

```rust
// สร้าง window ที่มองไม่เห็น (position -32000,-32000)
let window = WebviewWindowBuilder::new(app_handle, "avkuy-helper", WebviewUrl::External(url))
    .inner_size(1024.0, 768.0)
    .position(-32000.0, -32000.0)
    .visible(true)
    .decorations(false)
    .build();
```

### Step 2: Wait & Poll for Metadata

JavaScript injection ทำงาน 2 อย่าง:
- **ตรวจ Cloudflare**: ดู `document.title` มี "just a moment" หรือ `#challenge-running` อยู่
- **ดึง metadata**: เมื่อ Cloudflare ผ่านแล้ว ดึง title, poster, iframe URL

การสื่อสารระหว่าง WebView -> Rust ใช้ **location.hash side-channel**:
- `#ak_status=cf` = กำลังรอ Cloudflare
- `#ak_meta={json}` = metadata พร้อมแล้ว

```javascript
// ตัวอย่าง payload ที่ส่งผ่าน hash
{
  "title": "AVSUBTHAI ADN-750 ...",
  "poster": "https://avkuy.com/wp-content/uploads/...",
  "iframe": "https://av-kuy.com/v/xxxxx"
}
```

### Step 3: Timeout

Poll สูงสุด 20 รอบ (รอบละ ~2.5 วินาที) = **~50 วินาที** สำหรับ Cloudflare

---

## Phase B: Player Page via WebView

**File**: `src-tauri/src/lib.rs` - `fetch_avkuy_via_webview()` (ต่อ)

### Step 1: Close Main Window, Open Player Window

```rust
let _ = window.close();  // ปิด main page window
// เปิดใหม่ด้วย iframe URL เช่น https://av-kuy.com/v/xxxxx
let window2 = WebviewWindowBuilder::new(app_handle, "avkuy-helper", WebviewUrl::External(iframe_url))
    .build();
```

### Step 2: Monitor Network Requests

ใช้ `PerformanceObserver` ติดตาม resource requests:
```javascript
// ตั้ง observer จับ .m3u8 / .mp4 requests
const o = new PerformanceObserver((list) => {
    list.getEntries().forEach(e => {
        if (e.name.includes('.m3u8') || e.name.includes('.mp4'))
            window.__FOUND_URLS.push(e.name);
    });
});
o.observe({ entryTypes: ['resource'] });
```

### Step 3: Click + Poll

- **Click JS**: คลิกทุก element ที่เป็น `button, video, .jw-poster, .jw-display-icon-container`
- **Check JS**: อ่าน `window.__FOUND_URLS` หรือ regex หา `.m3u8` ใน HTML
- **Communication**: `#ak_video={encoded_url}`
- **Poll**: 14 รอบ x ~2.6s = **~36 วินาที** max

### Step 4: Return Result

```rust
AvkuyWebviewResult {
    title: Some("AVSUBTHAI ADN-750 ..."),
    poster_url: Some("https://..."),
    iframe_url: Some("https://av-kuy.com/v/xxxxx"),
    video_url: Some("https://cdn.xxx/xxx.m3u8"),  // << สิ่งที่ต้องการ
}
```

---

## Strategy 2: Chrome Detector Fallback

**File**: `src-tauri/src/chrome_detector.rs` - `detect_video_url()`

ถ้า WebView หา video URL ไม่ได้ จะใช้ Chrome Detector:

### Flow

```
Chrome Detector (headless Chrome CDP)
        |
        v
[Navigate to URL / iframe URL]
        |
        v
[Wait for Cloudflare] (wait_for_cloudflare, max 60s)
        |
        v
[Cloudflare ผ่านแล้ว - ตรวจสอบว่าไม่หลุดไป homepage]
        |
        v
+-------+--------+
| Video URL บน   |--- YES --> return URL
| main page?      |
+-------+--------+
        | NO
        v
[Click + Poll 8 รอบ บน main page]
        |
        v
+-------+--------+
| พบ?             |--- YES --> return URL
+-------+--------+
        | NO
        v
[Extract iframe URL จาก DOM]
(ดู iframe[src*='av-kuy.com/v/'] หรือ regex ใน HTML)
        |
        v
+-------+--------+
| iframe พบ?      |--- NO --> general detection fallback
+-------+--------+
        | YES
        v
[Navigate to iframe URL]
        |
        v
[Handle Cloudflare on iframe page]
(บางครั้ง iframe URL ก็มี Cloudflare ของตัวเอง)
        |
        v
[Setup observer + Click + Poll 10 รอบ]
        |
        v
+-------+--------+
| .m3u8 / .mp4?   |--- YES --> return URL
+-------+--------+
        | NO
        v
[General detection fallback]
```

---

## Strategy 3: HTTP + Packer Unpack (Last Resort)

**File**: `src-tauri/src/lib.rs` - `fetch_avkuy_stream_from_player_page()`

ใช้เมื่อทั้ง WebView และ Chrome detector หา video URL ไม่ได้ แต่ iframe URL มีอยู่:

### Flow

```
[มี iframe URL] เช่น https://av-kuy.com/v/xxxxx
        |
        v
[reqwest GET iframe URL]
- User-Agent: Chrome
- Referer: avkuy.com post URL
- Cookie: ใช้ cookie จาก Chrome detector
        |
        v
[Unpack JavaScript Packer]
- หา obfuscated packer pattern: function(p,a,c,k,e,d){...}
- ถอดรหัส base-N encoded strings
        |
        v
[Extract from unpacked JS]
- file: "https://cdn.xxx/stream.m3u8"
- image: "https://cdn.xxx/poster.jpg"
        |
        v
return (stream_url, poster_url)
```

### Packer Unpack Algorithm

```
Input:  obfuscated packer call with packed_string, radix, count, dictionary
        |
        v
[Base-N decode] แปลง key เป็น keyword ด้วย radix-based encoding
        |
        v
[Replace keywords] แทนที่ keys ด้วย decoded words
        |
        v
Output: unpacked JavaScript containing 'file': 'https://....m3u8'
```

---

## Download Phase

**File**: `src-tauri/src/lib.rs` - `start_download()`

เมื่อได้ `UnifiedSeriesInfo` แล้ว:

```
[UnifiedSeriesInfo]
  - title: "AVSUBTHAI ADN-750 ..."
  - poster_url: "https://..."
  - episode_urls: { 1: "https://cdn.xxx/stream.m3u8" }
  - source: "avkuy"
  - cookies: [(cf_clearance, xxx), ...]
        |
        v
[User clicks Download]
        |
        v
[Create output directory]
  - group_by_source=true --> {output_dir}/avkuy/
  - group_by_source=false --> {output_dir}/
        |
        v
[HLS Download (.m3u8)]
  - ใช้ cookies สำหรับ CDN authentication
  - referer = source_url (avkuy post URL)
  - Decrypt AES-128 segments (ถ้ามี HLS key)
  - Merge TS segments into final .mp4
        |
        v
[Progress events emitted to frontend]
  - download-progress: { episode, percent, speed }
  - download-complete: { episode, path }
```

---

## URL Detection Logic

```rust
// ใน get_series_info() และ auto_detect_video_url()
let is_avkuy = settings.avkuy_domain.split(',')
    .map(|d| d.trim())
    .any(|d| !d.is_empty() && url.contains(d))
    || url.contains("avkuy.com")
    || url.contains("av-kuy.com");
```

Default domain: `www2.avkuy.com` (configurable ใน Settings)

---

## Key Code Locations

| Component | File | Function/Section |
|-----------|------|-------------------|
| WebView extractor | `src-tauri/src/lib.rs:225` | `fetch_avkuy_via_webview()` |
| Packer unpacker | `src-tauri/src/lib.rs:134` | `unpack_packer_script()` |
| Player HTML extractor | `src-tauri/src/lib.rs:164` | `extract_avkuy_stream_from_player_html()` |
| HTTP player fetcher | `src-tauri/src/lib.rs:183` | `fetch_avkuy_stream_from_player_page()` |
| Chrome fallback | `src-tauri/src/chrome_detector.rs:606` | avkuy section in `detect_video_url()` |
| Main entry point | `src-tauri/src/lib.rs:921` | avkuy section in `get_series_info()` |
| Auto-detect | `src-tauri/src/lib.rs:1107` | avkuy check in `auto_detect_video_url()` |
| Download handler | `src-tauri/src/lib.rs:1128` | `start_download()` |
| Domain config | `src-tauri/src/lib.rs:507` | `DomainSettings.avkuy_domain` |
| TypeScript type | `src/types/index.ts:59` | `avkuyDomain?: string` |

---

## Error Scenarios

| Scenario | Cause | Result |
|----------|-------|--------|
| Cloudflare timeout (50s) | ไม่ผ่าน challenge | WebView returns empty, fallback to Chrome |
| iframe URL ไม่พบ | หน้าเว็บเปลี่ยนโครงสร้าง | WebView returns no iframe, Chrome tries harder |
| Player page ไม่มี video | dead link / region blocked | Error: "Could not find video URL on avkuy page" |
| Packer ถอดไม่ได้ | JS obfuscation เปลี่ยนรูปแบบ | Returns None, strategy 3 ข้าม |
| Cloudflare redirect to homepage | Challenge ผ่านแต่ redirect ผิด | Chrome detector navigate back to original URL |

---

## Debugging

### WebView Debug Log

```
~/Library/Application Support/com.rongyok.downloader/webview_debug.log
```

### Chrome Detector Stderr

```
[ChromeDetector] avkuy detected - resolving iframe player
[ChromeDetector] avkuy: Found URL on main page: https://...
```

### Frontend Log Events

```
"Detected avkuy - using WebView extractor..."
"Found AVKuy video via WebView: https://..."
"WebView not enough, falling back to Chrome detector..."
"Trying AVKuy player HTML extraction..."
"Found AVKuy stream via player HTML: https://..."
```
