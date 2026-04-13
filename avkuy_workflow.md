# AVKuy Download Workflow

## เป้าหมาย

- รองรับ URL หน้าโพสต์ของ `www2.avkuy.com`
- Auto detect URL จาก clipboard/ช่องกรอก
- แสดง `title` และ `thumbnail`
- แยกไฟล์ดาวน์โหลดลงโฟลเดอร์ `avkuy/`
- ดึง stream จริงจาก player ที่มี Cloudflare/iframe protection
- **ไม่ต้องใช้ Chrome detector** (ประหยัด RAM ~200-500MB)

## ตัวอย่าง URL ทดสอบ

| ประเภท | URL |
|--------|-----|
| หน้าโพสต์ | `https://www2.avkuy.com/avsubthai-adn-750/` |
| iframe player | `https://av-kuy.com/v/LQ76ROcLox2aZSg?sid=17081&t=hls` |
| HLS stream | `https://cache11224510.kuylive.com/hls/.../xfgdYshjhYhj=!sdsHsyG` |

## Flow Diagram

```
User paste URL (https://www2.avkuy.com/avsubthai-adn-750/)
        |
        v
[URL Detection] -- avkuy.com / av-kuy.com ตรงไหม?
        |
        +-- NO --> ไป parser อื่น
        |
        +-- YES --> [Phase A: WebView Main Page]
                        |
                        v
                   +------------------------------+
                   | Create WKWebView window      |
                   | (off-screen, hidden)         |
                   | Load avkuy.com URL           |
                   +------------------------------+
                        |
                        v
                   [Wait for Cloudflare] (poll 20รอบ, ~50s max)
                        |
                        v
                   [Extract via JS + location.hash]
                   - title: og:title / h1
                   - poster: og:image
                   - iframe: av-kuy.com/v/ URL
                   - cookies: document.cookie
                        |
                        v
                   +-----+------+
                   | iframe URL |--- YES --> [Phase B: WebView Player Page]
                   |  พบ?       |                |
                   +-----+------+                v
                         | NO               +--------------------------+
                         v                  | Open player URL in new   |
                   [Skip to packer]         | WebView window           |
                                            +--------------------------+
                                                 |
                                                 v
                                            [Click + Poll 14รอบ]
                                            (PerformanceObserver for .m3u8/.mp4)
                                            [Extract player cookies via document.cookie]
                                                 |
                                                 v
                                            [Close window]
                        |
                        v
                   [Packer Unpack with WebView cookies]
                        |
                        v
                   reqwest GET player URL + cookies
                        |
                        v
                   Unpack packed JS → JWPlayer config
                        |
                        v
                   file: "https://cache11224510.kuylive.com/hls/..."
                        |
                        v
                   +--------+--------+
                   | Stream URL พบ?  |--- YES --> [Download HLS]
                   +--------+--------+                  |
                            | NO                        v
                            v                      [Content-based HLS detection]
                   [Chrome Detector fallback]        (probe URL for #EXTM3U)
                   (last resort only)                      |
                                                           v
                                                      FFmpeg download
                                                      (strip PNG prefix from segments)
                                                           |
                                                           v
                                                      Save to avkuy/ folder
```

## ขั้นตอนที่ 1: WebView Main Page Extraction

**Code**: `src-tauri/src/lib.rs` → `fetch_avkuy_via_webview()` Phase A

เปิด avkuy.com ใน WKWebView รอ Cloudflare ผ่าน แล้วดึง metadata:

```javascript
// JS injection ดึง metadata + cookies
{
    title: document.querySelector("meta[property='og:title']")?.content || document.title,
    poster: document.querySelector("meta[property='og:image']")?.content || '',
    iframe: "https://av-kuy.com/v/xxxxx",  // จาก iframe[src]
    cookies: document.cookie                // << สำคัญมาก
}
```

**Side-channel**: `window.location.hash = 'ak_meta=' + encodeURIComponent(JSON.stringify(payload))`

## ขั้นตอนที่ 2: WebView Player Page (Phase B)

**Code**: `src-tauri/src/lib.rs` → `fetch_avkuy_via_webview()` Phase B

เปิด iframe URL (av-kuy.com) ใน WebView window ใหม่:

```javascript
// ตั้ง PerformanceObserver จับ .m3u8/.mp4 requests
const o = new PerformanceObserver((list) => {
    list.getEntries().forEach(e => {
        if (e.name.includes('.m3u8') || e.name.includes('.mp4'))
            window.__FOUND_URLS.push(e.name);
    });
});

// Click + Poll 14 รอบ
// สุดท้ายดึง cookies:
window.location.hash = 'ak_cookies=' + encodeURIComponent(document.cookie);
```

## ขั้นตอนที่ 3: Packer Unpack (ไม่ต้องใช้ Chrome)

**Code**: `src-tauri/src/lib.rs` → `fetch_avkuy_stream_from_player_page()`

ใช้ cookies จาก WebView ยิง HTTP request ไป player page:

```
reqwest GET https://av-kuy.com/v/xxxxx
  - Cookie: cf_clearance=xxx; PHPSESSID=xxx  (จาก WebView)
  - Referer: https://www2.avkuy.com/...
  → ได้ HTML ที่มี packed JavaScript
```

Unpack algorithm:
1. Regex หา `eval(function(p,a,c,k,e,d){...}('payload',radix,count,'symtab'.split('|'),...)`
2. Base-N decode (radix 10-62) แปลง index → keyword
3. Replace keywords ใน payload
4. Extract `file` และ `image` จาก unpacked JS

## ขั้นตอนที่ 4: HLS Download (PNG Prefix Stripping)

**Code**: `src-tauri/src/downloader.rs` → `download_hls_manual()` + `strip_png_prefix()`

CDN (kuylive.com) ใส่ PNG header ด้านหน้า segment เป็น anti-scraping:

```
[68 bytes PNG 1x1] [2 bytes header] [MPEG-TS video data]
89 50 4E 47 ... IEND ... 0x82 0x20 ... 47 40 11 ... (h264 1280x720)
```

**Fix**: `strip_png_prefix()` หา IEND marker → skip → หา MPEG-TS sync byte `0x47` → return data ที่เหลือ

## Flow เปรียบเทียบ

### ก่อน (ใช้ Chrome):
```
WebView → Chrome detector (หนัก) → Packer unpack → Download
          ~200-500MB RAM          ~2-3 นาที
```

### ตอนนี้ (ไม่ใช้ Chrome):
```
WebView (เบา) → Packer unpack → Download
  ~50MB RAM      ~5 วินาที
```

Chrome detector ยังคงเป็น fallback แต่จะใช้เฉพาะเมื่อ WebView cookies ไม่พอ

## Key Code Locations

| Component | File | Function |
|-----------|------|----------|
| WebView extractor | `src-tauri/src/lib.rs` | `fetch_avkuy_via_webview()` |
| Cookie parser | `src-tauri/src/lib.rs` | `parse_cookie_string()` |
| Packer unpacker | `src-tauri/src/lib.rs` | `unpack_packer_script()` |
| Player HTML extractor | `src-tauri/src/lib.rs` | `extract_avkuy_stream_from_player_html()` |
| HTTP player fetcher | `src-tauri/src/lib.rs` | `fetch_avkuy_stream_from_player_page()` |
| HLS detection | `src-tauri/src/downloader.rs` | content-based `#EXTM3U` probe |
| PNG prefix strip | `src-tauri/src/downloader.rs` | `strip_png_prefix()` |
| Chrome fallback | `src-tauri/src/chrome_detector.rs` | avkuy section (last resort) |

## ตัวอย่าง Log (Flow ใหม่ ไม่มี Chrome)

```
[INFO] Detected avkuy — using WebView extractor...
[INFO] Waiting AVKuy Cloudflare... (1/20)
...
[INFO] AVKuy player scan 1/14...
...
[INFO] Trying AVKuy player HTML extraction (WebView cookies)...
[INFO] Found AVKuy stream via player HTML: https://cache11224510.kuylive.com/hls/...
[SUCCESS] Auto-loaded: [title] (1 episodes)
[INFO] Starting download of 1 episodes...
[SUCCESS] Episode 1 downloaded
[SUCCESS] Merged to: /Volumes/Media/_VDO_download/avkuy/[title].mp4
```

## จุดที่ต้องระวัง

- CDN segments มี PNG prefix ต้อง strip ก่อน concat
- Stream URL ไม่มี `.m3u8` extension → ต้องใช้ content-based detection
- Cookies จาก WebView อาจไม่ครบ → Chrome detector เป็น fallback
- Stream token อาจหมดอายุ → ควร download ทันทีหลัง extract
