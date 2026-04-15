# 🎬 Video Downloader Tauri

> 📥 โปรแกรมดาวน์โหลดวิดีโอแบบครบวงจร — สถาปัตยกรรม **Microservice-oriented** รองรับ 8 เว็บไซต์ พร้อมระบบ Headless Chrome Bypass, AES-128 HLS Decryption, Scheduled Downloads, Webhook Notifications และ Library Management

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)
![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?logo=typescript)
![Rust](https://img.shields.io/badge/Rust-1.75-orange?logo=rust)
![SQLite](https://img.shields.io/badge/SQLite-WAL-003B57?logo=sqlite)
![License](https://img.shields.io/badge/License-MIT-green)

---

## 📑 Table of Contents

- [เว็บไซต์ที่รองรับ (Supported Sites)](#-เว็บไซต์ที่รองรับ-supported-sites)
- [สถาปัตยกรรมระบบ (System Architecture)](#-สถาปัตยกรรมระบบ-system-architecture)
- [รายละเอียดลอจิกทุก Parser](#-รายละเอียดลอจิกทุก-parser-deep-dive)
- [Headless Chrome Bypass Engine](#-headless-chrome-bypass-engine-chrome_detectorrs)
- [Download Engine](#-download-engine-downloaderrs)
- [เวิร์กโฟลว์การดาวน์โหลด 6 ขั้นตอน](#-เวิร์กโฟลว์การดาวน์โหลดสมบูรณ์-6-stage-lifecycle)
- [ระบบฐานข้อมูล](#-ระบบฐานข้อมูล-database-layer)
- [ระบบเสริม](#-ระบบเสริม-supporting-systems)
- [ฟีเจอร์ UI/Frontend](#-ฟีเจอร์-uifrontend)
- [ติดตั้งและใช้งาน](#️-ติดตั้งและใช้งาน)
- [โครงสร้างโปรเจค](#-โครงสร้างโปรเจค)

---

## 🌐 เว็บไซต์ที่รองรับ (Supported Sites)

ระบบถูกออกแบบให้ทุก Parser เป็นโมดูลอิสระ (`*_parser.rs`) แต่ละตัวมี Engine การดึงข้อมูลที่ปรับแต่งเฉพาะเว็บ:

| # | เว็บไซต์ | ไฟล์ Parser | ประเภทเว็บ | วิธีดึงวิดีโอ | Anti-Bot Bypass |
|---|----------|-------------|-----------|--------------|-----------------|
| 1 | **Rongyok** / Thongyok | `parser.rs` | ซีรีส์เกาหลี/จีน/ฝรั่ง (หลายตอน) | HTTP Scraper → regex m3u8 | ❌ ไม่มี |
| 2 | **Baanjeen** (บ้านจีน.com) | `baanjeen_parser.rs` | ซีรีส์จีน (หลายตอน) | HTTP Scraper → iframe → JS src | ❌ ไม่มี |
| 3 | **TitanMirror** (357ms.com / 51cg) | `titan_parser.rs` | วิดีโอเฉพาะทาง (Series + Archive) | HTTP + XOR-decoded API → AES-128 HLS | ❌ ไม่มี |
| 4 | **HSCK** (hsck123.com / cctv12306.com) | `hsck_parser.rs` | วิดีโอเดี่ยว | HTTP Scraper → `<img id="video_img">` | ❌ ไม่มี |
| 5 | **Jav18tv** (18jav.tv) | `jav18tv_parser.rs` | วิดีโอเดี่ยว (Cloudflare) | HTTP + Cloudflare detection → fallback Chrome | ✅ Cloudflare |
| 6 | **Javwow** (javwow.com) | `javwow_parser.rs` | วิดีโอเดี่ยว (Cloudflare + packed JS) | HTTP → onlysubthai.com iframe → JWPlayer unpack | ✅ Cloudflare |
| 7 | **Njav.org** | `njav_parser.rs` | วิดีโอเดี่ยว (WordPress Advanced iFrame) | HTTP → missav.guide → 302 redirect → javxx.com → Chrome | ✅ Cloudflare + SPA |
| 8 | **NjavTV** (njavtv.com) | `njavtv_parser.rs` | วิดีโอเดี่ยว (Cloudflare เต็มรูปแบบ) | Chrome detector เท่านั้น (ข้าม HTTP ไปเลย) | ✅ Cloudflare ตลอด |

**ไอคอนสถานะ:** 🟢 = Static HTML (ไม่ต้อง bypass) | 🔴 = ต้องใช้ Chrome Detector หรือ Cloudflare Bypass

---

## 🧠 สถาปัตยกรรมระบบ (System Architecture)

```
┌────────────────────────────────────────────────────────────────────┐
│                         React Frontend (UI)                        │
│   ┌──────────┐ ┌───────────┐ ┌──────────┐ ┌────────────────────┐  │
│   │ URL Input│ │ Series    │ │ Download │ │ Library / Settings │  │
│   │ + Paste  │ │ Selector  │ │ Progress │ │ Scheduler / Queue  │  │
│   └────┬─────┘ └─────┬─────┘ └────┬─────┘ └────────┬───────────┘  │
│        │              │            │                │              │
│────────┼──────────────┼────────────┼────────────────┼──────────────│
│        │      Tauri IPC (invoke / emit / channels)  │              │
│────────┼──────────────┼────────────┼────────────────┼──────────────│
│                                                                    │
│                    Rust Backend (lib.rs — 2,800+ LOC)              │
│                                                                    │
│  ┌──────────────────────────────────────────────────────────┐      │
│  │               Domain Router (lib.rs)                      │      │
│  │  URL → domain matching → เลือก parser ที่ตรง              │      │
│  │  is_titan_url() / is_hsck_url() / is_baanjeen_url() ...  │      │
│  └────┬──────┬──────┬──────┬──────┬──────┬──────┬──────┬────┘      │
│       │      │      │      │      │      │      │      │           │
│  ┌────▼──┐┌──▼───┐┌─▼──┐┌─▼──┐┌──▼──┐┌──▼──┐┌──▼──┐┌──▼──┐      │
│  │Rongyok││Baan  ││Titan││HSCK││18JAV││Javwow││Njav ││NjavTV│     │
│  │Parser ││Jeen  ││Parse││Pars││Parse││Parser││Parse││Parse│      │
│  └───┬───┘└──┬───┘└──┬──┘└──┬─┘└──┬──┘└──┬──┘└──┬──┘└──┬──┘      │
│      │       │       │      │     │      │      │      │           │
│      └───────┴───────┼──────┴─────┴──────┴──────┴──────┘           │
│                      │                                              │
│              ┌───────▼──────────┐     ┌──────────────────┐          │
│              │  Chrome Detector │     │   Video          │          │
│              │  (headless_chrome)│◄───│   Downloader     │          │
│              │  1,521 LOC       │     │   (1,979 LOC)    │          │
│              └──────────────────┘     └────────┬─────────┘          │
│                                                │                    │
│  ┌──────────────┐ ┌──────────────┐ ┌───────────▼──────┐            │
│  │  Queue DB    │ │ Schedule DB  │ │  FFmpeg Process   │            │
│  │  (SQLite WAL)│ │ (SQLite WAL) │ │  (Background)     │            │
│  └──────────────┘ └──────────────┘ └──────────────────┘            │
│                                                                    │
│  ┌──────────────┐ ┌──────────────┐ ┌──────────────────┐            │
│  │  Library DB  │ │ Notification │ │  Webhook          │            │
│  │  (SQLite WAL)│ │ DB (SQLite)  │ │  (Discord/Line)   │            │
│  └──────────────┘ └──────────────┘ └──────────────────┘            │
└────────────────────────────────────────────────────────────────────┘
```

### Core Components

| Component | ไฟล์ | LOC | หน้าที่หลัก |
|-----------|------|-----|------------|
| **Domain Router** | `lib.rs` | ~2,800 | จุดเข้าหลัก, IPC commands, domain matching, proxy config |
| **Download Engine** | `downloader.rs` | 1,979 | FFmpeg HLS, Manual HLS, Titan AES decryption, Direct download |
| **Chrome Detector** | `chrome_detector.rs` | 1,521 | Headless Chrome, anti-bot bypass, network interception |
| **Library Manager** | `library.rs` | 1,731 | Series/episode persistence, watch progress, tags, stats |
| **Scheduler** | `scheduler.rs` | 806 | Cron-like scheduling, bandwidth limiting, background tasks |
| **Rongyok Parser** | `parser.rs` | 443 | Multi-episode series extraction |
| **Titan Parser** | `titan_parser.rs` | 609 | XOR API decode, AES-128 HLS keys, concurrent fetch |
| **Baanjeen Parser** | `baanjeen_parser.rs` | 500 | Iframe/JS video URL mining |
| **HSCK Parser** | `hsck_parser.rs` | 395 | `<img>` tag-based URL extraction |
| **Javwow Parser** | `javwow_parser.rs` | 565 | Cloudflare bypass, onlysubthai.com embed |
| **Jav18tv Parser** | `jav18tv_parser.rs` | 491 | Hybrid HTTP/Chrome detection |
| **Njav Parser** | `njav_parser.rs` | 282 | WordPress Advanced iFrame, redirect chain resolution |
| **NjavTV Parser** | `njavtv_parser.rs` | 270 | Pure Chrome detection (skips HTTP entirely) |
| **Queue DB** | `queue_db.rs` | 429 | Download queue persistence, status tracking |
| **Webhook** | `webhook.rs` | 357 | Discord/LINE/Custom webhook notifications |
| **Notifications** | `notifications.rs` | 310 | In-app notification log |
| **Backup** | `backup.rs` | ~400 | Configuration backup/restore |

---

## 🔍 รายละเอียดลอจิกทุก Parser (Deep Dive)

### 1️⃣ Rongyok Parser (`parser.rs`)

**เป้าหมาย:** ดึงข้อมูลซีรีส์จาก Rongyok/Thongyok (ดูซีรีส์เกาหลี จีน ฝรั่ง มีหลายตอน)

**URL Formats ที่รองรับ:**
- `https://rongyok.com/watch/?series_id=1004`
- `https://thongyok.com/series/1004/`

**ขั้นตอนการทำงาน:**

```
1. parse_series_url(url) → ดึง series_id จาก URL
   ├─ Pattern 1: regex `series_id=(\d+)` (query string)
   └─ Pattern 2: regex `/series/(\d+)` (path-based)

2. get_series_info(series_id) → HTTP GET หน้าซีรีส์
   ├─ Headers: Accept, Accept-Language, Referer
   ├─ ใช้ proxy config (ถ้ามี)
   └─ รองรับทั้ง rongyok.com และ thongyok.com

3. HTML Extraction (ใช้ `scraper` crate):
   ├─ Title: <title> tag → ตัด suffix " - ตอนที่ X" และชื่อเว็บ
   ├─ Poster: <meta property="og:image"> → content attribute
   ├─ Total Episodes: regex จากปุ่ม EP ใน HTML
   └─ Episode URLs: extract_all_episode_urls() → regex หา .m3u8 URLs
      ├─ Pattern: href ที่มี /watch/ + episode suffix
      └─ Fallback: ค้นหา m3u8 URL จาก HTML โดยตรง

4. Search & Browse:
   ├─ search(): GET `?s={query}&page={page}` → parse listing
   └─ browse(): GET category page → parse listing
```

**Regex สำคัญ:**
```rust
// URL extraction
r"series_id=(\d+)"
r"/series/(\d+)"
// Title cleanup
r"\s*-\s*ตอนที่\s*\d+.*$"
r"\s*(Thongyok|Rongyok).*$"
```

---

### 2️⃣ Baanjeen Parser (`baanjeen_parser.rs`)

**เป้าหมาย:** ดึงวิดีโอจากเว็บ บ้านจีน.com (ใช้ Punycode domain: `xn--82c7abb4jua0l.com`)

**URL Detection:**
```rust
fn is_baanjeen_url(url: &str, domain: &str) -> bool {
    url.contains(domain)
        || url.contains("xn--82c7abb4jua0l.com")    // Punycode
        || url.contains("บ้านจีน.com")                 // Unicode domain
}
```

**ขั้นตอนการทำงาน (Multi-Method Search):**

```
1. HTTP GET หน้าซีรีส์ → ได้ HTML

2. Extraction Phase (Synchronous block — ไม่มี await):
   ├─ Title: extract_title()
   │   ├─ og:title meta tag
   │   ├─ <title> tag → ตัด " - บ้านจีน" suffix
   │   ├─ <h1> tag
   │   └─ Regex fallback: r#""title"\s*:\s*"([^"]+)""#
   │
   ├─ Poster: extract_poster()
   │   ├─ og:image
   │   ├─ twitter:image
   │   └─ img.wp-post-image / img.attachment-full
   │
   ├─ Iframe Sources: <iframe> tags → src / data-lazy-src
   │
   ├─ Episode Links: a[href*='ตอนที่'], a[href*='ep-'], a[href*='episode']
   │
   └─ Script Sources: <script src="..."> tags (สำหรับ JS video URL mining)

3. Video URL Discovery (3 ขั้นตอน ลำดับ priority):
   ├─ Step 1: ค้นหา m3u8/mp4 URLs จาก iframe sources
   │   └─ ถ้า iframe src มี .m3u8 หรือ .mp4 → ใช้เลย
   │
   ├─ Step 2: ค้นหาจาก HTML หลัก (main page)
   │   └─ Regex: r#"['\"](https?://[^'\"].*?\.m3u8[^'\"]*)['\"#]"
   │
   └─ Step 3: Fetch external JS files แล้วค้นหา URL ข้างใน
       ├─ HTTP GET แต่ละ <script src="...">
       └─ ค้นหา m3u8/mp4 URL ใน JS content

4. Episode Mapping:
   ├─ ถ้ามี episode links → map แต่ละ link เป็น episode number
   └─ ถ้าไม่มี → ถือเป็น single episode (total_episodes = 1)
```

**จุดเด่นทางเทคนิค:**
- ค้นหา video URL ใน **3 ชั้น**: iframe → HTML → external JS
- Extract document ใน synchronous block แยกจาก async code (ป้องกัน `!Send` issue)

---

### 3️⃣ TitanMirror Parser (`titan_parser.rs`)

**เป้าหมาย:** ดึงวิดีโอจาก 357ms.com และ domain ที่เกี่ยวข้อง (51cg, 51cm, titan51.net) — มีระบบ **XOR-encoded API** และ **AES-128-CBC HLS encryption**

**URL Detection (Multi-domain):**
```rust
fn is_titan_url(url: &str, domains: &str) -> bool {
    if url.contains("51cg") || url.contains("357ms") { return true; }
    // ตรวจ comma-separated custom domains
    domains.split(',').map(|d| d.trim()).any(|d| url.contains(d))
}
```

**URL Types ที่รองรับ:**
- Series listing: `https://www.357ms.com/series/360`
- Episode watch: `https://www.357ms.com/watch/26037`
- Archive single: `https://www.357ms.com/archives/252482`

**ขั้นตอนการทำงาน:**

```
1. HTTP GET หน้าเว็บ → ดึง final URL (หลัง redirect)

2. HTML Extraction (Synchronous block):
   ├─ Title: <title> tag → ตัด " | " suffix
   ├─ Poster:
   │   ├─ og:image meta tag
   │   └─ Fallback: regex `window.seriesCover\s*=\s*['"]([^'"]+)`
   │       (สำหรับ watch pages ที่ไม่มี og:image)
   ├─ Episode Links: a[href*='/watch/'] → deduplicate
   │   └─ EP number: regex `EP[.\s]*(\d+)` จาก link text
   └─ Series ID:
       ├─ จาก URL: regex `/series/(\d+)` หรือ `/archives/(\d+)`
       └─ จาก HTML: regex `href=['"]*/series/(\d+)['"]`

3. Video URL Resolution (3 แนวทาง):

   ─── แนวทาง A: Archive Pages ───
   ├─ Strategy 1: JSON "url" field ใน dplayer data-config
   │   regex: r#""url"\s*:\s*"(https?:[^"]*?\.m3u8[^"]*?)""#
   │   → unescape \/ → / (JSON escaped slashes)
   ├─ Strategy 2: Quoted m3u8 URL (fallback)
   │   regex: r#"[\"'](https?://[^\"']*?\.m3u8[^\"']*?)[\"']"#
   └─ Strategy 3: Broad m3u8 anywhere
       regex: r#"(https?:[^\s\"'<>]*?\.m3u8[^\s\"'<>]*)"#

   ─── แนวทาง B: Series Pages (มี episode links) ───
   ├─ XOR-decoded API (per episode, concurrent 5 at a time):
   │   ├─ API URL: {base}/api/v1/hls/config/{series_id}/{ep_num}
   │   ├─ Response: XOR-encoded text
   │   │   ├─ First 2 bytes = decimal key (e.g., "70")
   │   │   └─ Rest = char XOR key → decoded JSON
   │   ├─ Decoded JSON:
   │   │   {
   │   │     "status": "success",
   │   │     "data": {
   │   │       "stream_url": "https://hls.357ms.com/series_360/ep_1/master.m3u8",
   │   │       "hls_key_b64": "base64_encoded_AES_key",
   │   │       "hls_iv": "hex_IV_32chars"
   │   │     }
   │   │   }
   │   └─ Store HlsKeyInfo { key_b64, iv_hex, hls_base_url } per episode
   │
   └─ Fallback (API fail): Pattern URL
       `https://hls.357ms.com/series_{id}/ep_{num}/master.m3u8`

   ─── แนวทาง C: Direct watch page (ไม่มี ep-card links) ───
   └─ Regex fallback: ค้นหา m3u8 URL จาก HTML

4. Poster Fetch:
   └─ fetch_image_as_data_url() → HTTP GET → base64 data URL
      (ฝัง poster เป็น data:image/jpeg;base64,... ส่งไป frontend)
```

**🔐 XOR Decode Logic (สำคัญ):**
```rust
fn xor_decode(raw: &str) -> Option<String> {
    // raw = "70" + XOR-encoded payload
    let key: u8 = raw[..2].parse()?;        // e.g., key = 70
    let decoded: String = raw[2..]
        .chars()
        .map(|c| char::from_u32((c as u32) ^ (key as u32)).unwrap_or(c))
        .collect();
    Some(decoded) // → valid JSON string
}
```

**🔑 AES-128-CBC HLS Key Info:**
```rust
pub struct HlsKeyInfo {
    pub key_b64: String,        // Base64-encoded AES-128 key (16 bytes)
    pub iv_hex: String,         // IV as hex string (32 hex chars = 16 bytes)
    pub hls_base_url: String,   // Base URL for relative segment paths
}
```

---

### 4️⃣ HSCK Parser (`hsck_parser.rs`)

**เป้าหมาย:** ดึงวิดีโอจาก hsck123.com (วิดีโอเดี่ยว ไม่มี series)

**URL Structure:**
- List page: `/?type=TYPE&p=PAGE`
- Video page: `/view/?id=XXXXXXXX`

**เทคนิคพิเศษ — ใช้ `<img>` tag แทน `<video>`:**
```html
<!-- เว็บนี้ใช้ src เป็น video URL และ alt เป็น poster URL (สลับกัน!) -->
<img id="video_img"
     src="https://t0.97img.com/b1000415/a.m3u8"    ← Video URL อยู่ใน src
     alt="https://aliyun.cctv05.com/i/b1000415.jpg"> ← Poster อยู่ใน alt
```

**ขั้นตอนการทำงาน:**

```
1. parse_video_id(url) → regex `[?&]id=([a-zA-Z0-9]+)` → video ID

2. HTTP GET video page พร้อม headers:
   ├─ Accept-Language: zh-CN,zh;q=0.9,en;q=0.8
   └─ (เว็บจีน ต้องใช้ Chinese Accept-Language)

3. extract_video_data(html):
   ├─ Primary: <img id="video_img">
   │   ├─ src → video URL (M3U8 stream)
   │   └─ alt → poster URL
   │
   └─ Fallback: regex `https?://[^\s\"'<>]+\.m3u8[^\s\"'<>]*`
      (ค้นหา m3u8 URL จาก HTML ตรงๆ)

4. Title: <title> tag → ตัด suffix ` - 黄色仓库 - hsck123.com`
   └─ regex: r"\s*-\s*黄色仓库.*$"

5. get_series_info() → wrap as single episode
   └─ total_episodes = 1, episode_urls = { 1: video_url }
```

**Lazy Static Selectors (Performance):**
```rust
lazy_static! {
    static ref IMG_SELECTOR: Selector = Selector::parse("img#video_img");
    static ref M3U8_RE: Regex = Regex::new(r#"https?://[^\s"'<>]+\.m3u8[^\s"'<>]*"#);
    static ref TITLE_CLEANUP_RE: Regex = Regex::new(r"\s*-\s*黄色仓库.*$");
}
```

**Categories:**
| ID | Label |
|----|-------|
| `latest` | Latest |
| `hot` | Hot |
| `chinese` | Chinese |
| `japanese` | Japanese |
| `korean` | Korean |

---

### 5️⃣ Jav18tv Parser (`jav18tv_parser.rs`)

**เป้าหมาย:** ดึงวิดีโอจาก 18jav.tv — เว็บ WordPress มี Cloudflare protection

**Strategy: Hybrid (HTTP first → Chrome fallback)**

```
1. try_fetch_html(url):
   ├─ HTTP GET พร้อม full browser headers:
   │   User-Agent, Sec-Fetch-Dest, Sec-Fetch-Mode, etc.
   │
   ├─ Cloudflare Detection:
   │   ├─ HTTP 403 → blocked
   │   ├─ "cf-browser-verification" in HTML → challenge
   │   ├─ "challenge-platform" in HTML → challenge
   │   ├─ "Just a moment" в HTML → challenge
   │   └─ "Checking your browser" in HTML → challenge
   │
   └─ ถ้าผ่าน Cloudflare → parse HTML ปกติ

2. parse_html() (เมื่อ HTTP สำเร็จ):
   ├─ Title Priority:
   │   ├─ 1. og:title
   │   ├─ 2. <title> → ตัด " - " suffix (filter "18jav"/"18 JAV")
   │   ├─ 3. h1 / h1.entry-title / .video-title
   │   └─ 4. [class*='title'] (CSS wildcard)
   │
   ├─ Poster Priority:
   │   ├─ 1. og:image
   │   ├─ 2. twitter:image
   │   ├─ 3. video[poster]
   │   └─ 4. img.wp-post-image / img[src*='thumb']
   │
   ├─ Embed URL:
   │   ├─ 1. <iframe> with src containing /embed/, /v/, player, m3u8, .mp4, stream, video
   │   ├─ 2. og:video / og:video:url / og:video:secure_url meta tags
   │   └─ 3. Regex: r#"(https?://[^"'\s<>]*(?:embed|player|video)[^"'\s<>]*)"#
   │
   └─ Direct Video URL:
       ├─ 1. var hlsUrl = "..." (18jav.tv specific JS variable)
       ├─ 2. m3u8 URL (generic, filter .css/.js)
       ├─ 3. og:video meta tag (mp4)
       └─ 4. mp4 URL (filter preview/css/js)

3. Fallback (Cloudflare blocked):
   └─ Return URL-based info → Chrome detector จัดการ
      title = slugfrom URL → UPPERCASE
```

---

### 6️⃣ Javwow Parser (`javwow_parser.rs`)

**เป้าหมาย:** ดึงวิดีโอจาก javwow.com — ระบบ Cloudflare + onlysubthai.com player + packed JWPlayer JS

**Architecture ที่ซับซ้อน:**
```
javwow.com (Cloudflare)
    └─ <iframe src="onlysubthai.com/v/{videoId}?sid=5917&t=hls">
         └─ Packed JS (JWPlayer) → unpack → HLS stream URL
              └─ CDN: dednaja.com (HLS with PNG prefix anti-scraping)
```

**ขั้นตอนการทำงาน:**

```
1. try_fetch_html():
   ├─ HTTP GET พร้อม Thai Accept-Language (th,en-US;q=0.9)
   └─ Cloudflare challenge detection (เหมือน Jav18tv)

2. เมื่อได้ HTML:
   ├─ Title: og:title → <title> (ตัด "JavWow" suffix) → h1.entry-title
   ├─ Poster: og:image → twitter:image → img.wp-post-image
   │
   └─ Embed URL (3 methods):
       ├─ Method 1: <iframe> ที่มี "onlysubthai.com" ใน src/data-src/data-lazy-src
       │   └─ Handle "//" prefix → "https://"
       ├─ Method 2: og:video meta tag ที่มี "onlysubthai" หรือ "subthai"
       └─ Method 3: Regex `(https?://[^"'\s<>]*onlysubthai[^"'\s<>]*)`

3. lib.rs จะใช้ embed URL ที่ได้:
   ├─ HTTP fetch onlysubthai.com player page → ได้ packed JS
   ├─ Unpack JWPlayer packed JS → ดึง HLS stream URL
   └─ Manual HLS download with PNG prefix stripping
      (เหมือน avkuy.com — segments ฝัง PNG header ข้างหน้า)

4. Chrome detector เป็น fallback สุดท้ายเท่านั้น
```

**Categories:**
| ID | Label |
|----|-------|
| `latest` | Latest |
| `censored` | Censored |
| `uncensored` | Uncensored |
| `subthai` | Sub-Thai |

---

### 7️⃣ Njav Parser (`njav_parser.rs`)

**เป้าหมาย:** ดึงวิดีโอจาก njav.org — WordPress + Advanced iFrame plugin → redirect chain

**Redirect Chain:**
```
njav.org/snos-034/
    └─ <iframe id="advanced_iframe" src="https://missav.guide/snos-034">
         └─ HTTP 302 → https://javxx.com/th/v/snos-034-uncensored-leaked
              └─ React SPA → <iframe src="surrit.store/...">
                   └─ HLS video player
```

**ขั้นตอนการทำงาน:**

```
1. extract_video_code(url):
   └─ URL path → last segment: "https://njav.org/snos-034/" → "snos-034"

2. try_fetch(url) → HTTP GET หน้า njav.org

3. Phase 1 — Synchronous extraction (ไม่มี await):
   ├─ Title: og:title → <title> → h1 → .entry-title → regex JSON
   ├─ Poster: og:image → img[src*='uploads']
   └─ extract_advanced_iframe_src(html):
       ├─ Primary: <iframe id="advanced_iframe"> → src attribute
       └─ Fallback: <iframe> ที่มี missav/javxx/surrit ใน src

4. Phase 2 — Async redirect resolution (หลัง drop document):
   └─ resolve_redirect(iframe_src):
       ├─ HTTP GET missav.guide URL → follow redirects
       ├─ ดึง final URL (javxx.com)
       └─ Accept ถ้า domain = javxx.com / missav / surrit
          (return anyway as fallback — domain อาจเปลี่ยน)

5. ส่ง resolved URL ไป Chrome detector:
   └─ Chrome detector เปิด javxx.com (React SPA)
       → render JS → หา surrit.store iframe → ดัก m3u8 traffic
```

**จุดเด่น:** Pre-resolve iframe chain ฝั่ง server เพื่อให้ Chrome detector ได้ URL ที่ถูกต้องทันที (ไม่ต้องให้ Chrome navigate ผ่าน njav.org)

---

### 8️⃣ NjavTV Parser (`njavtv_parser.rs`)

**เป้าหมาย:** ดึงวิดีโอจาก njavtv.com — **Cloudflare protected ตลอด ← ข้าม HTTP ไปเลย**

**URL Structure:** `/dm18/th/cus-376` (video page)

**ขั้นตอนการทำงาน:**

```
1. get_series_info():
   ├─ ⚠️ SKIP HTTP fetch เลย (always Cloudflare)
   │   "ข้ามการ fetch ทาง HTTP เพราะจะ hang 30 วินาทีก่อน timeout"
   │
   ├─ Title: extract_title_from_url()
   │   └─ URL slug → capitalize each word
   │      "cus-376" → "Cus 376"
   │
   └─ Return NjavtvSeriesInfo {
         total_episodes: 1,
         direct_page_url: Some(series_url),   // ส่งให้ Chrome detector
         episode_page_urls: { 1: series_url },
       }

2. Chrome detector takes over:
   └─ เปิดหน้าใน headless Chrome → bypass Cloudflare challenge
       → ดัก network traffic → หา HLS/MP4 URL
```

**เมื่อ HTML ใช้ได้ (rare — parse_html fallback):**
```
├─ Title: <title> → og:title → h1 → regex JSON
├─ Poster: og:image
└─ Episodes: a.episode-link, .episode-list a, a[href*='/th/']
   └─ EP number: regex `(?:ep|episode|e)[-_]?(\d+)`
```

---

## 🛡️ Headless Chrome Bypass Engine (`chrome_detector.rs`)

### Anti-Bot Configuration

Chrome เปิดด้วย flags เพื่อ bypass Cloudflare และ anti-bot systems:

```rust
LaunchOptions {
    headless: false,                                    // ⚠️ ใช้ headed mode (ซ่อน window off-screen)
    sandbox: false,
    args: [
        "--no-sandbox",
        "--disable-dev-shm-usage",
        "--disable-gpu",
        "--window-size=1920,1080",
        "--window-position=-32000,-32000",              // 📍 ซ่อน window ออกนอกจอ
        "--autoplay-policy=no-user-gesture-required",   // 🔊 Auto-play video
        "--disable-blink-features=AutomationControlled", // 🛡️ ซ่อน automation flag
        "--disable-infobars",
        "--excludeSwitches=enable-automation",
        "--user-agent=Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) Chrome/120.0.0.0",
    ],
}
```

### Detection Flow

```
1. init_browser() → Launch Chrome (reuse existing instance)

2. detect_video_url(url):
   ├─ Open new tab → navigate to URL
   │
   ├─ Inject PerformanceObserver JS:
   │   └─ Monitor network requests for .m3u8 / .mp4 URLs
   │      window.__detectedVideoUrls = [];
   │      new PerformanceObserver((list) => {
   │          entries.forEach(e => {
   │              if (e.name.match(/\.m3u8|\.mp4/))
   │                  window.__detectedVideoUrls.push(e.name);
   │          });
   │      }).observe({ entryTypes: ['resource'] });
   │
   ├─ Click Script (Aggressive):
   │   ├─ Click play buttons (JWPlayer, VideoJS, Plyr selectors)
   │   ├─ Click poster images (.jw-poster, [class*="poster"])
   │   ├─ Click all iframes (activate embedded players)
   │   ├─ video.play() + video.click()
   │   └─ Center-screen click (fallback for overlays)
   │
   ├─ Wait & poll for detected URLs (timeout: ~60s)
   │   └─ Check window.__detectedVideoUrls every 2s
   │
   ├─ Capture metadata (best-effort):
   │   ├─ Title: og:title → twitter:title → h1 → document.title
   │   └─ Poster: og:image → twitter:image → video[poster] → img[src*='cover']
   │
   └─ Extract cookies (for authenticated stream access)
       └─ tab.get_cookies() → Vec<(name, value)>
```

### Cookie Passthrough

```
Chrome detector → cookies → VideoDownloader HTTP client
                           → FFmpeg -headers "Cookie: cf_clearance=...; ..."
```

---

## 📥 Download Engine (`downloader.rs`)

### Download Strategy Selection

```rust
fn download_episode(video_url, hls_key_info, ...) {
    if hls_key_info.is_some() {
        // 1. Titan AES-128-CBC encrypted HLS
        download_titan_hls()
    }
    else if video_url.contains(".m3u8") {
        // 2. Standard HLS via FFmpeg
        download_hls_stream()
        // → on failure (obfuscated extensions) →
        download_hls_manual()   // 3. Manual segment download
    }
    else if probe_url_is_hls() {
        // 4. Content-based HLS detection (no .m3u8 extension)
        download_hls_stream()   // → fallback → download_hls_manual()
    }
    else {
        // 5. Direct MP4/binary download
        download_direct()
    }
}
```

### 1. Titan AES-128-CBC HLS (`download_titan_hls`)

```
Input: master_url + HlsKeyInfo { key_b64, iv_hex, hls_base_url }

1. Fetch master playlist → parse bandwidth variants → select quality
2. Fetch variant playlist → parse segment list (.ts files)
3. Download each segment via HTTP
4. Decrypt: AES-128-CBC(segment_data, base64_decode(key_b64), hex_decode(iv_hex))
5. Write decrypted segments → concatenated .ts file
6. FFmpeg: convert .ts → .mp4
7. Cleanup temp .ts file
```

### 2. Standard FFmpeg HLS (`download_hls_stream`)

```
1. Pre-resolve master playlist → select quality variant
2. Execute FFmpeg:
   ffmpeg -i "{m3u8_url}"
          -headers "Referer: {referer}\r\nCookie: {cookies}\r\n"
          -c copy -bsf:a aac_adtstoasc
          "{output.mp4}"
3. Parse FFmpeg stderr:
   ├─ Duration: regex → total_duration
   ├─ Progress: parse_ffmpeg_stats → percentage/speed
   └─ Errors: track last_error_line
4. Emit download-progress events → frontend realtime UI
5. Cancellation: check is_cancelled.load(SeqCst) → kill child process
```

### 3. Manual HLS Segment Download (`download_hls_manual`)

**ใช้เมื่อ:** FFmpeg reject segments ที่มี extension แปลก (`.jpeg`, `.html`) หรือ format mismatches

```
1. Fetch master playlist → select variant
2. Fetch variant playlist → parse segment URLs
3. Download segments via reqwest (ไม่ใช้ FFmpeg HLS demuxer):
   ├─ Concurrent HTTP GETs with proper headers
   ├─ PNG prefix stripping: ลบ PNG header ที่ฝังข้างหน้า TS data
   │   (anti-scraping technique ของบาง CDN)
   └─ Write to temp .ts file (concatenated)
4. FFmpeg: .ts → .mp4 (remux only, no re-encode)
```

### 4. Direct Download (`download_direct`)

```
1. HTTP GET → streaming response
2. Read chunks → write to file
3. Track progress: downloaded/total bytes → emit events
4. Retry on failure: exponential backoff
```

### Quality Selection Logic

```rust
fn parse_master_playlist(master_text, master_url) -> Vec<QualityOption> {
    // Parse #EXT-X-STREAM-INF lines:
    //   BANDWIDTH=5000000,RESOLUTION=1920x1080 → "1080p (5.0 Mbps)"
    //   BANDWIDTH=2500000,RESOLUTION=1280x720  → "720p (2.5 Mbps)"
    // Sort by bandwidth descending
    // User selects preferred_quality → match by resolution
}
```

### Connection Pooling

```rust
Client::builder()
    .pool_max_idle_per_host(10)          // Keep 10 idle connections per host
    .pool_idle_timeout(Duration::from_secs(90))  // 90s idle timeout
    .user_agent("Mozilla/5.0 ...")
    .build()
```

### Error Recovery

```rust
// Automatic fallback chain
download_hls_stream()
    → if error contains "empty segment" / "allowed_extensions" / "mismatches" / "invalid data"
    → download_hls_manual()  // Manual segment-by-segment download
```

---

## 🔄 เวิร์กโฟลว์การดาวน์โหลดสมบูรณ์ (6-Stage Lifecycle)

### Stage 1️⃣ — Input (ตรวจจับลิงก์)

```
User → วาง URL ในช่อง Input (หรือ Auto Capture จาก Clipboard)
     → Frontend ส่ง IPC invoke("fetch_series", { url }) ไป Rust backend
     → lib.rs: Domain Router วิเคราะห์ URL → เลือก Parser
         ├─ is_titan_url()   → TitanParser
         ├─ is_hsck_url()    → HsckParser
         ├─ is_baanjeen_url()→ BaanJeenParser
         ├─ is_jav18tv_url() → Jav18tvParser
         ├─ is_javwow_url()  → JavwowParser
         ├─ is_njav_url()    → NjavParser
         ├─ is_njavtv_url()  → NjavtvParser
         └─ default          → RongyokParser
```

### Stage 2️⃣ — Metadata Fetch (เข้าถึงแหล่งข้อมูล)

```
Parser.get_series_info(url):
  ├─ เว็บปกติ (Rongyok, Baanjeen, Titan, HSCK):
  │   └─ HTTP GET → scraper parse → m3u8/mp4 URLs
  │
  ├─ เว็บ Cloudflare (Jav18tv, Javwow):
  │   └─ try HTTP → ถ้าถูก block → return partial info
  │      → lib.rs จะใช้ Chrome detector เสริม
  │
  ├─ เว็บ iframe chain (Njav):
  │   └─ HTTP → resolve iframe → redirect → return resolved URL
  │      → Chrome detector จัดการ javxx.com SPA
  │
  └─ เว็บ pure Cloudflare (NjavTV):
      └─ SKIP HTTP → return URL directly
         → Chrome detector จัดการทั้งหมด

Result → UnifiedSeriesInfo {
    title, poster_url, total_episodes,
    episode_urls: HashMap<episode_num, video_url>,
    needs_chrome_detection: bool
}
```

### Stage 3️⃣ — Selection & Queuing

```
Frontend แสดงรายชื่อตอน → User เลือก (Select All / เฉพาะบางตอน)
  → Quality selection (parse_master_playlist → show resolution options)
  → กดยืนยัน → Push เข้า SQLite Queue (queue_db.rs)
     ├─ Status: "pending"
     ├─ Priority: 0 (default)
     └─ series_info: JSON metadata
```

### Stage 4️⃣ — Execution (ดาวน์โหลด)

```
Download Scheduler:
  ├─ ดึง pending items จาก queue
  ├─ Token-based concurrency (Semaphore)
  ├─ Per-episode:
  │   ├─ Select download strategy (FFmpeg/Manual/Direct/Titan)
  │   ├─ Download with progress tracking
  │   │   └─ emit("download-progress", { episode, downloaded, total, speed, percentage })
  │   ├─ Cancellation check: download_state.is_cancelled.load(SeqCst)
  │   └─ Retry on failure: exponential backoff
  └─ Update queue status: "downloading" → "completed" / "failed"
```

### Stage 5️⃣ — Post-Processing (FFmpeg)

```
เมื่อ segments ย่อยโหลดครบ:
  ├─ Titan HLS: AES-128-CBC decrypt → concatenate → FFmpeg remux .ts → .mp4
  ├─ Standard HLS: FFmpeg -c copy → .mp4 (no re-encode)
  ├─ Manual HLS: strip PNG prefix → concatenate → FFmpeg remux → .mp4
  └─ Direct: ไม่ต้อง post-process

FFmpeg Progress Tracking:
  ├─ Parse stderr: Duration, time=, bitrate=, size=
  └─ Calculate: (current_time / total_duration) * 100 → percentage
```

### Stage 6️⃣ — Completion

```
เมื่อดาวน์โหลดสำเร็จ:
  ├─ Update Library DB (library.rs):
  │   └─ save_series() + update_episode_status("completed", file_path)
  │
  ├─ Log Notification (notifications.rs):
  │   └─ log_notification("download", "Download Complete", message)
  │
  ├─ Send Webhook (webhook.rs):
  │   ├─ Discord: Embed with color-coded status
  │   ├─ LINE Notify: Bearer token + message
  │   └─ Custom: POST JSON with event/title/message
  │
  ├─ Update Queue DB:
  │   └─ update_status("completed", series_info, None)
  │
  └─ Emit frontend event:
      └─ emit("download-complete", { episode, file_path })
```

---

## 🗄️ ระบบฐานข้อมูล (Database Layer)

ทุก DB ใช้ **SQLite WAL mode** เพื่อ concurrent read performance:
```sql
PRAGMA journal_mode=WAL;
PRAGMA foreign_keys=ON;
```

### Queue DB (`queue_db.rs`)

```sql
CREATE TABLE download_queue (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    url TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',  -- pending/downloading/completed/failed
    series_info TEXT,                         -- JSON metadata
    error TEXT,
    priority INTEGER NOT NULL DEFAULT 0,     -- Higher = processed first
    created_at TEXT NOT NULL DEFAULT (datetime('now')),
    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
);
CREATE INDEX idx_queue_status ON download_queue(status);
CREATE INDEX idx_queue_priority ON download_queue(priority DESC, created_at);
```

**Operations:** `add_item`, `get_pending`, `update_status`, `remove_item`, `clear_completed`, `get_stats`

---

### Library DB (`library.rs`)

```sql
-- Schema Version 4 (with migrations)
CREATE TABLE library (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    parser_series_id INTEGER NOT NULL DEFAULT 0,
    title TEXT NOT NULL,
    source TEXT NOT NULL,               -- "rongyok"/"titan"/"hsck"/etc.
    source_url TEXT NOT NULL DEFAULT '',
    poster_path TEXT,                    -- Local file path for poster image
    total_episodes INTEGER DEFAULT 1,
    date_added TEXT NOT NULL,
    last_downloaded TEXT,
    metadata TEXT,
    favorite INTEGER NOT NULL DEFAULT 0,
    description TEXT,
    rating REAL,
    year INTEGER,
    genre TEXT,
    duration TEXT,
    UNIQUE(source, source_url)
);

CREATE TABLE library_episodes (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    library_id INTEGER NOT NULL REFERENCES library(id) ON DELETE CASCADE,
    episode_number INTEGER NOT NULL,
    video_url TEXT,
    file_path TEXT,                      -- Downloaded file location
    quality TEXT,
    file_size INTEGER,
    status TEXT DEFAULT 'pending',       -- pending/downloading/completed
    watched INTEGER NOT NULL DEFAULT 0,
    watched_at TEXT,
    UNIQUE(library_id, episode_number)
);

CREATE TABLE library_tags ( id INTEGER PRIMARY KEY, name TEXT NOT NULL UNIQUE );
CREATE TABLE library_tag_map ( library_id INTEGER, tag_id INTEGER, PRIMARY KEY(library_id, tag_id) );
CREATE TABLE watch_progress (
    library_id INTEGER REFERENCES library(id) ON DELETE CASCADE,
    episode_number INTEGER,
    position_seconds REAL DEFAULT 0,
    duration_seconds REAL DEFAULT 0,
    updated_at TEXT DEFAULT (datetime('now')),
    PRIMARY KEY(library_id, episode_number)
);
```

**Schema Migrations:**
| Version | Changes |
|---------|---------|
| 1 | Base schema (library + library_episodes) |
| 2 | Add `favorite` column, `library_tags`, `library_tag_map` tables |
| 3 | Add `watched`/`watched_at` columns, `watch_progress` table |
| 4 | Add `description`, `rating`, `year`, `genre`, `duration` columns |

---

### Schedule DB (`scheduler.rs`)

```sql
CREATE TABLE download_schedule (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    url TEXT NOT NULL,
    output_dir TEXT NOT NULL,
    cron_expression TEXT NOT NULL,  -- "hourly"/"daily HH:MM"/"weekly N HH:MM"
    enabled INTEGER DEFAULT 1,
    last_run TEXT,
    next_run TEXT,
    created_at TEXT DEFAULT (datetime('now'))
);
```

**Cron Expressions:**
| Expression | ตัวอย่าง | ความหมาย |
|-----------|---------|----------|
| `hourly` | `hourly` | ทุกชั่วโมง ที่ :00 |
| `daily HH:MM` | `daily 08:00` | ทุกวัน เวลา 08:00 |
| `weekly N HH:MM` | `weekly 1 09:00` | ทุกวันจันทร์ (N=1) เวลา 09:00 |

**Bandwidth Limiting (ScheduleConfig):**
```rust
ScheduleConfig {
    enabled: bool,
    active_start: Option<String>,    // "22:00" — เริ่มเวลาที่กำหนด
    active_end: Option<String>,      // "06:00" — จบเวลาที่กำหนด
    speed_during_active: u64,        // KB/s (0 = unlimited)
    speed_outside_active: u64,       // KB/s outside active window
    auto_pause: bool,
    auto_resume: bool,
}
// รองรับ งวดข้ามเที่ยงคืน (22:00 → 06:00)
```

---

### Notification DB (`notifications.rs`)

```sql
CREATE TABLE notification_log (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    category TEXT NOT NULL,          -- "download"/"auto_check"/"test"
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    read INTEGER DEFAULT 0,
    action_type TEXT,                -- "open_file"/"open_library"/null
    action_data TEXT,                -- file path or library ID
    created_at TEXT DEFAULT (datetime('now'))
);
```

---

## 🔌 ระบบเสริม (Supporting Systems)

### Webhook Notifications (`webhook.rs`)

| Type | Endpoint | Format |
|------|----------|--------|
| **Discord** | Webhook URL | Embeds with color-coded status (🟢 Complete / 🔴 Failed / 🟠 New Episode) |
| **LINE Notify** | `https://notify-api.line.me/api/notify` | Bearer token + form message |
| **Custom** | Any URL | POST JSON `{ event, title, message, timestamp }` |

**Supported Events:**
- `download_complete` — ดาวน์โหลดสำเร็จ
- `download_failed` — ดาวน์โหลดล้มเหลว
- `new_episode` — ตรวจพบตอนใหม่
- `test` — ทดสอบ webhook

### Auto Episode Check (`check_new_episodes`)

```
1. Query library entries ที่มี source_url
2. Filter: ข้าม URL ที่เป็น .m3u8 / .mp4 โดยตรง
3. ตรวจสอบ current total_episodes vs ข้อมูลใหม่
4. Log ผลลัพธ์ใน notification_db
5. ส่ง webhook ถ้าพบตอนใหม่
```

### Proxy Support (`proxy.rs`)

```rust
ProxyConfig {
    enabled: bool,
    url: String,         // "http://proxy:8080" หรือ "socks5://..."
    username: Option<String>,
    password: Option<String>,
}
// build_client() → reqwest::Client ที่ตั้ง proxy (ถ้าเปิดใช้)
```

### Backup System (`backup.rs`)

- Export/Import configuration as JSON
- ครอบคลุม: proxy settings, domain configs, webhook config, schedule config

---

## ✨ ฟีเจอร์ UI/Frontend

| ฟีเจอร์ | รายละเอียด |
|---------|-----------|
| 🚀 **Smart Queue** | จัดการคิวอัตโนมัติแบบ Parallel พร้อม Priority |
| 📋 **Auto Capture** | ตรวจจับลิงก์จาก Clipboard อัตโนมัติ |
| 📊 **Real-time Graph** | กราฟความเร็วดาวน์โหลดแบบ Real-time ด้วย custom hooks |
| 🎨 **5 Theme Colors** | Violet, Blue, Emerald, Amber, Rose + Full Dark Mode |
| 🌍 **Multi-language** | ไทย (TH) และ อังกฤษ (EN) |
| ⌨️ **Keyboard Shortcuts** | Ctrl+V (Paste), Ctrl+D (Download), Space (Pause), Esc (Cancel) |
| 🖼️ **Mini Mode** | หน้าต่างเล็กลอยหน้าจอ สำหรับ Monitor สถานะดาวน์โหลด |
| 📚 **Library Management** | จัดการคลังซีรีส์ + Tags + Favorites + Watch Progress |
| ⏰ **Download Scheduler** | ตั้งเวลาดาวน์โหลด (hourly/daily/weekly) + Bandwidth Limiting |
| 🔔 **Notifications** | In-app notification center + Discord/LINE webhooks |
| 💾 **Backup/Restore** | Export/Import ทุก configuration เป็น JSON |

---

## 🛠️ ติดตั้งและใช้งาน

### 📋 ความต้องการระบบ

| Dependency | Version | หน้าที่ |
|-----------|---------|--------|
| **Node.js** | 18+ | Frontend build tooling |
| **Rust** | 1.70+ | Backend compilation |
| **FFmpeg** | Latest | HLS merging, video conversion (**ต้องอยู่ใน system PATH**) |
| **Chrome / Chromium** | Latest | Headless bypass สำหรับเว็บ Cloudflare |

### Clone และติดตั้ง

```bash
git clone https://github.com/your-username/video-downloader-tauri.git
cd video-downloader-tauri
npm install
```

### รันโหมดพัฒนา

```bash
npm run tauri dev
```

### Build Production

```bash
npm run tauri build
```

---

## 📁 โครงสร้างโปรเจค

```
video-downloader-tauri/
├── 📂 src/                          # React Frontend
│   ├── 📂 components/               # UI Elements
│   ├── 📂 hooks/                    # Custom Hooks (useDownload, useAppState, useLibrary)
│   ├── 📂 features/                 # Feature modules (Download, Library, Settings)
│   └── 📂 types/                    # TypeScript interfaces
│
├── 📂 src-tauri/                    # Rust Backend
│   ├── 📂 src/
│   │   ├── lib.rs                   # 📍 Entry point + Domain Router + IPC Commands (~2,800 LOC)
│   │   ├── main.rs                  # Tauri main entry (minimal)
│   │   │
│   │   ├── ── Parsers ──────────────
│   │   ├── parser.rs                # 🟢 Rongyok — Series (multi-episode, HTTP scraper)
│   │   ├── baanjeen_parser.rs       # 🟢 BaanJeen — Series (iframe + JS mining)
│   │   ├── titan_parser.rs          # 🔴 TitanMirror — XOR API + AES-128 HLS
│   │   ├── hsck_parser.rs           # 🟢 HSCK — Single video (<img> tag extraction)
│   │   ├── jav18tv_parser.rs        # 🔴 18JAV — Hybrid (HTTP + Chrome fallback)
│   │   ├── javwow_parser.rs         # 🔴 JavWow — Cloudflare + packed JS + PNG strip
│   │   ├── njav_parser.rs           # 🔴 Njav.org — iframe chain resolution + Chrome
│   │   ├── njavtv_parser.rs         # 🔴 NjavTV — Pure Chrome detection
│   │   │
│   │   ├── ── Core Engine ──────────
│   │   ├── downloader.rs            # 📥 Download engine (HLS/Manual/Titan/Direct, 1,979 LOC)
│   │   ├── chrome_detector.rs       # 🛡️ Headless Chrome bypass (1,521 LOC)
│   │   │
│   │   ├── ── Data Layer ───────────
│   │   ├── queue_db.rs              # 📋 Download queue (SQLite WAL)
│   │   ├── library.rs               # 📚 Library + Episodes + Tags + Watch Progress (1,731 LOC)
│   │   ├── scheduler.rs             # ⏰ Scheduled downloads + bandwidth limiting (806 LOC)
│   │   ├── notifications.rs         # 🔔 In-app notification log
│   │   │
│   │   ├── ── Supporting ───────────
│   │   ├── webhook.rs               # 🔌 Discord/LINE/Custom webhooks
│   │   ├── proxy.rs                 # 🌐 HTTP/SOCKS proxy configuration
│   │   ├── backup.rs                # 💾 Config backup/restore
│   │   └── utils.rs                 # 🔧 Path expansion, filename sanitization
│   │
│   ├── Cargo.toml                   # Rust dependencies
│   └── tauri.conf.json              # Tauri configuration
│
├── OPTIMIZATION_ROADMAP.md          # Performance optimization plan (4 phases)
└── README.md                        # 📖 This file
```

---

## 📄 License & 🤝 Contributing

MIT License — ใช้งานได้อิสระ ยินดีรับ Pull Requests เพื่อปรับแต่ง Parser หรือเพิ่มเว็บไซต์ใหม่!

**วิธีเพิ่ม Parser ใหม่:**
1. สร้างไฟล์ `{name}_parser.rs` ตาม pattern ที่มี (implement `get_series_info`, `search`, `browse`, `list_categories`)
2. เพิ่ม `is_{name}_url()` function สำหรับ domain matching
3. เพิ่ม routing logic ใน `lib.rs` Domain Router
4. เพิ่ม domain settings ใน frontend Settings panel

<div align="center">

**Total Codebase: ~15,000+ LOC (Rust Backend) + React Frontend**

Made with 🦀 Rust + ⚛️ React + 💜 Tauri

</div>
