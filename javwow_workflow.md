# JavWow Download Workflow

## เป้าหมาย

- รองรับ URL หน้าวิดีโอของ `javwow.com` (slug-based URL)
- Auto detect URL จาก clipboard/ช่องกรอก
- แสดง `title` และ `thumbnail`
- แยกไฟล์ดาวน์โหลดลงโฟลเดอร์ `javwow/`
- ดึง stream จริงจาก player (onlysubthai.com) ที่มี packed JS obfuscation
- **ไม่ต้องใช้ Chrome detector** (ประหยัด RAM ~200-500MB)

## ตัวอย่าง URL ทดสอบ

| ประเภท | URL |
|--------|-----|
| หน้าวิดีโอ | `https://javwow.com/start-257/` |
| iframe player | `https://onlysubthai.com/v/AiPmxvRLnWQNvKW?sid=4752&t=hls` |
| HLS stream | `https://googleapisthemes14tf5f58.dednaja.com/hls/.../xfgdYshjhYhj=!sdsHsyG` |

## Flow Diagram

```
User paste URL (https://javwow.com/start-257/)
        |
        v
[URL Detection] -- javwow.com ตรงไหม?
        |
        +-- NO --> ไป parser อื่น
        |
        +-- YES --> [Phase 0: HTTP Fetch Metadata]
                        |
                        v
                   +------------------------------+
                   | reqwest GET javwow.com       |
                   | (ดึง og:title, og:image,     |
                   |  iframe onlysubthai.com)     |
                   +------------------------------+
                        |
                        v
                   +-----+------+
                   | Cloudflare  |--- YES (403) --> [Phase A: WebView Main Page]
                   |  ผ่าน?       |                      |
                   +-----+------+                      v
                         | NO               +--------------------------+
                         v                  | Create WKWebView window   |
                   [ได้ embed URL?]          | Load javwow.com URL       |
                         |                  | Wait for Cloudflare       |
                         v                  | Extract metadata + cookies|
                   [Phase B: Packer Unpack]  +--------------------------+
                                                      |
                                                      v
                                                 [Extract via JS + location.hash]
                                                 - title: og:title / h1
                                                 - poster: og:image
                                                 - iframe: onlysubthai.com/v/ URL
                                                 - cookies: document.cookie
                                                      |
                                                      v
                                                 [ได้ embed URL + cookies?]
                                                      |
                                                      v
                   +----------------------------------+
                   |
                   v
             [Phase B: Packer Unpack with cookies]
                   |
                   v
              reqwest GET onlysubthai.com/v/xxx + cookies
                   |
                   v
              Unpack packed JS --> JWPlayer config
                   |
                   v
              file: "https://googleapisthemes14tf5f58.dednaja.com/hls/..."
                   |
                   v
              +--------+--------+
              | Stream URL พบ?  |--- YES --> [Download HLS]
              +--------+--------+                  |
                       | NO                        v
                       v                      [FFmpeg attempt]
              [Chrome Detector fallback]           |
              (last resort only)              [PNG prefix error]
                                                    |
                                                    v
                                              [Manual HLS Download]
                                              (strip PNG prefix)
                                                    |
                                                    v
                                              Save to javwow/ folder
```

## ขั้นตอนที่ 0: HTTP Fetch Metadata

**Code**: `src-tauri/src/javwow_parser.rs` -> `get_series_info()`

ลอง HTTP request ไป javwow.com ก่อน (อาจผ่าน Cloudflare ได้ถ้า session เดิมยังใช้ได้):

```
// ตรวจสอบ Cloudflare จาก HTTP response
// - 403 status -> "Cloudflare blocked (403)"
// - cf-browser-verification / challenge-platform -> challenge page
```

ถ้าผ่าน -> ดึง metadata จาก HTML:
```
title:  og:title || <title> || h1
poster: og:image || twitter:image || img.wp-post-image
embed:  iframe[src*="onlysubthai.com"] || regex match in raw HTML
```

ถ้า embed_url พบจาก HTTP -> ข้ามไป Phase B เลย (ไม่ต้องเปิด WebView)

## ขั้นตอนที่ 1: WebView Main Page Extraction

**Code**: `src-tauri/src/lib.rs` -> `fetch_via_webview()` Phase A

เปิด javwow.com ใน WKWebView รอ Cloudflare ผ่าน แล้วดึง metadata:

```javascript
// JS injection ดึง metadata + cookies
{
    title: document.querySelector("meta[property='og:title']")?.content || document.title,
    poster: document.querySelector("meta[property='og:image']")?.content || '',
    iframe: "https://onlysubthai.com/v/xxxxx",  // จาก iframe[src] ที่มี /v/
    cookies: document.cookie                // << สำคัญมาก
}
```

**Side-channel**: `window.location.hash = 'ak_meta=' + encodeURIComponent(JSON.stringify(payload))`

### iframe Detection Logic

```javascript
// วิธีที่ 1: ตรวจจาก <iframe> element
const frames = document.querySelectorAll('iframe');
for (let i = 0; i < frames.length; i++) {
    const src = frames[i].src || frames[i].dataset.src || frames[i].dataset.lazySrc || '';
    if (src.includes('/v/')) {           // จับทุก iframe ที่มี /v/ (player URL pattern)
        iframe = src.startsWith('//') ? 'https:' + src : src;
        break;
    }
}

// วิธีที่ 2: Regex fallback ใน raw HTML
const m = html.match(/(https?:\/\/[^"'\s<>]*\/v\/[^"'\s<>]*)/i);
```

## ขั้นตอนที่ 2: Packer Unpack (ไม่ต้องใช้ Chrome)

**Code**: `src-tauri/src/lib.rs` -> `fetch_avkuy_stream_from_player_page()`

ใช้ cookies จาก WebView ยิง HTTP request ไป onlysubthai.com:

```
reqwest GET https://onlysubthai.com/v/xxxxx?sid=4752&t=hls
  - Cookie: PHPSESSID=xxx; __dtsu=xxx; ...  (จาก WebView)
  - Referer: https://javwow.com/...
  -> ได้ HTML ที่มี packed JavaScript
```

### ตัวอย่าง HTML จาก onlysubthai.com

```html
<script src="https://content.jwplatform.com/libraries/SAHhwvZq.js"></script>
<div id="jw_player"></div>
<script>
  [packed JS with JWPlayer config - Base-62 encoded, radix 62, count 84]
</script>
```

### Unpack Algorithm

1. Regex หา packed JS pattern: `function(p,a,c,k,e,d){...}('payload',radix,count,'symtab'.split('|'),digits,{})`
2. Base-N decode (radix 62) แปลง index -> keyword
3. Replace keywords ใน payload
4. Extract `file` และ `image` จาก unpacked JS

### ตัวอย่าง Unpacked JS

```javascript
$(document).ready(function(){
    const playerInstance = jwplayer("jw_player").setup({
        playlist:[{
            sources:[{
                label:'START',
                type:'hls',
                file:'https://googleapisthemes14tf5f58.dednaja.com/hls/.../xfgdYshjhYhj=!sdsHsyG'
            }],
            image:"/uploads/banners/START-257.jpg",
        }],
        ...
    });
});
```

## ขั้นตอนที่ 3: HLS Download (PNG Prefix Stripping)

**Code**: `src-tauri/src/downloader.rs` -> `download_hls_manual()` + `strip_png_prefix()`

CDN (dednaja.com) ใส่ PNG header ด้านหน้า segment เป็น anti-scraping (เหมือน kuylive.com ของ avkuy):

```
[68 bytes PNG 1x1] [2 bytes header] [MPEG-TS video data]
89 50 4E 47 ... IEND ... 0x82 0x20 ... 47 40 11 ... (h264 1280x720)
```

**Fix**: `strip_png_prefix()` หา IEND marker -> skip -> หา MPEG-TS sync byte `0x47` -> return data ที่เหลือ

### FFmpeg Fallback Flow

```
FFmpeg direct HLS -> ล้มเหลว (PNG prefix detected as png_pipe format)
    |
    v
Manual HLS download:
1. Fetch master playlist (content-based #EXTM3U detection)
2. Parse segment URLs
3. Download each segment via HTTP
4. strip_png_prefix() on each segment
5. Concatenate all segments
6. FFmpeg remux to MP4
```

## Flow เปรียบเทียบ

### ก่อน (ใช้ Chrome):
```
Chrome detector -> Cloudflare bypass -> Navigate to onlysubthai.com -> Poll for .m3u8
~500MB RAM                                            ~2-3 นาที + มักล้มเหลว
```
ปัญหา: Chrome detector ไปที่ onlysubthai.com แล้วไม่พบ .m3u8 เพราะ video URL ถูกซ่อนใน packed JS ไม่ได้โหลดเป็น network request ตรงๆ

### ตอนนี้ (ไม่ใช้ Chrome):
```
WebView (เบา) -> Packer unpack (HTTP) -> Download
  ~50MB RAM      ~5 วินาที              Manual HLS
```
WebView เปิด javwow.com เพื่อเอา cookies และ embed URL เท่านั้น -> ไม่ต้อง render video
จากนั้นใช้ HTTP request ดึง player HTML -> unpack JS -> ได้ stream URL ตรงๆ

Chrome detector ยังคงเป็น fallback แต่จะใช้เฉพาะเมื่อทั้ง HTTP และ WebView ล้มเหลว

## Key Code Locations

| Component | File | Function |
|-----------|------|----------|
| WebView extractor (ทั่วไป) | `src-tauri/src/lib.rs` | `fetch_via_webview()` |
| Cookie parser | `src-tauri/src/lib.rs` | `parse_cookie_string()` |
| Packer unpacker | `src-tauri/src/lib.rs` | `unpack_packer_script()` |
| Player HTML extractor | `src-tauri/src/lib.rs` | `extract_avkuy_stream_from_player_html()` |
| HTTP player fetcher | `src-tauri/src/lib.rs` | `fetch_avkuy_stream_from_player_page()` |
| JavWow parser | `src-tauri/src/javwow_parser.rs` | `JavwowParser` |
| HLS detection | `src-tauri/src/downloader.rs` | content-based `#EXTM3U` probe |
| PNG prefix strip | `src-tauri/src/downloader.rs` | `strip_png_prefix()` |
| Chrome fallback | `src-tauri/src/chrome_detector.rs` | javwow section (last resort) |

## ตัวอย่าง Log (Flow ใหม่ ไม่มี Chrome)

```
[INFO] Detected javwow -- using WebView extractor...
[JavWow] HTTP fetch failed: Cloudflare blocked (403), using URL-based info
[INFO] Waiting Cloudflare... (1/20)
...
[INFO] Found javwow embed URL via WebView: https://onlysubthai.com/v/AiPmxvRLnWQNvKW?sid=4752&t=hls
[INFO] Trying javwow player HTML extraction (WebView cookies)...
[INFO] Found javwow stream via player HTML: https://googleapisthemes14tf5f58.dednaja.com/hls/.../xfgdYshjhYhj=!sdsHsyG
[SUCCESS] Auto-loaded: [START-257] (1 episodes)
[INFO] Starting download of 1 episodes...
[Downloader] FFmpeg failed (PNG prefix), retrying manual segment download...
[ManualHLS] Downloading 1037 segments
[SUCCESS] Episode 1 downloaded
[SUCCESS] Merged to: /Volumes/Media/_VDO_download/javwow/START-257.mp4
```

## จุดที่ต้องระวัง

- CDN segments มี PNG prefix ต้อง strip ก่อน concat (เหมือน avkuy)
- Stream URL ไม่มี `.m3u8` extension -> ต้องใช้ content-based detection (`#EXTM3U`)
- Cookies จาก WebView อาจไม่ครบ -> Chrome detector เป็น fallback
- Stream token อาจหมดอายุ -> ควร download ทันทีหลัง extract
- onlysubthai.com ใช้ packed JS (radix 62, count 84) -> `unpack_packer_script()` รองรับแล้ว
- Domain CDN อาจเปลี่ยน (googleapisthemes14tf5f58, dednaja.com) -> ต้องตรวจจาก packed JS

## การทดสอบที่ผ่าน

| วันที่ | URL | ผลลัพธ์ |
|--------|-----|---------|
| 2026-04-13 | `https://javwow.com/start-257/` | START-257.mp4, 1195MB, h264 1280x720, 144min, aac 44100Hz |

## ความเชื่อมโยงกับ avkuy.com

ทั้ง avkuy.com และ javwow.com ใช้ infrastructure เดียวกัน:
- CDN token suffix เดียวกัน: `xfgdYshjhYhj=!sdsHsyG`
- PNG prefix anti-scraping เหมือนกันบน HLS segments
- Packed JS format เดียวกัน (JWPlayer config)
- WebView + packer unpack pattern เดียวกัน

ดูรายละเอียดเพิ่มเติมได้ที่ `avkuy_workflow.md`
