# เว็บไซต์ที่รองรับและเทคนิคการดึงวิดีโอ

## ตารางสรุป

| เว็บไซต์ | เทคนิคการดึงวิดีโอ | ต้องเปิด Chrome? | หมายเหตุ |
|---------|-------------------|-----------------|----------|
| **NjavTV** | WebView + HLS.js Polling | ✅ ใช่ | ตรวจจับ `window.hls.url` จาก HLS.js player |
| **Njav.org** | Iframe Chain + Chrome Detector | ✅ ใช่ | WordPress → missav.guide → javxx.com → surrit.store |
| **Avkuy** | Cloudflare Bypass + Iframe | ✅ ใช่ | ต้องผ่าน Cloudflare challenge ก่อน |
| **Javwow** | Cloudflare + Iframe Resolution | ✅ ใช่ | ดึง onlysubthai.com embed URL |
| **Rongyok** | HTTP Parsing | ❌ ไม่ | ดึง Discord CDN URLs จาก HTML/JSON |
| **Thongyok** | HTTP Parsing + Episode Fetch | ❌ ไม่ | ดึง Discord CDN URLs จากหน้าแต่ละตอน |
| **Baanjeen** | HTTP Parsing | ❌ ไม่ | ดึง video URLs จาก HTML |
| **HSCK** | HTTP Parsing | ❌ ไม่ | ดึง video URLs จาก HTML |
| **Titan** | HTTP Parsing | ❌ ไม่ | ดึง video URLs จาก HTML |

---

## รายละเอียดแต่ละเว็บไซต์

### 1. NjavTV (njavtv.com)
**เทคนิค:** WebView + HLS.js Polling  
**ต้องใช้ Chrome:** ✅ ใช่

**วิธีการทำงาน:**
- เว็บไซต์ป้องกันด้วย Cloudflare ตลอดเวลา
- ข้าม HTTP fetch ไปใช้ Chrome detector โดยตรง
- Poll `window.hls.url` จาก HLS.js player instance (10 ครั้ง × 2 วินาที)
- **ไม่ต้องคลิก** - HLS.js โหลด URL อัตโนมัติ
- Fallback: PerformanceObserver สำหรับ network requests

**ไฟล์ที่เกี่ยวข้อง:**
- `src-tauri/src/njavtv_parser.rs` - Parser หลัก
- `src-tauri/src/chrome_detector.rs:364-434` - NjavTV detection logic

---

### 2. Njav.org
**เทคนิค:** Iframe Chain Resolution + Chrome Detector  
**ต้องใช้ Chrome:** ✅ ใช่

**วิธีการทำงาน:**
1. **Phase 1:** ดึง iframe URL จาก njav.org (WordPress Advanced iFrame plugin)
2. **Phase 2:** Resolve redirect chain: `missav.guide` → `javxx.com`
3. **Phase 3:** Navigate ไปยัง javxx.com (React SPA)
4. **Phase 4:** ค้นหา surrit.store iframe (nested iframe)
5. **Phase 5:** Navigate ไปยัง surrit.store และคลิกเพื่อโหลดวิดีโอ

**ความท้าทาย:**
- Iframe ซ้อน 2 ชั้น (javxx → surrit)
- React SPA ต้องรอ render (5 วินาที)
- ต้องคลิกหลายครั้งเพื่อ trigger video load

**ไฟล์ที่เกี่ยวข้อง:**
- `src-tauri/src/njav_parser.rs` - Pre-resolution ของ iframe chain
- `src-tauri/src/chrome_detector.rs:439-604` - Njav detection logic

---

### 3. Avkuy (avkuy.com, av-kuy.com)
**เทคนิค:** Cloudflare Bypass + Iframe Extraction  
**ต้องใช้ Chrome:** ✅ ใช่

**วิธีการทำงาน:**
1. **Phase 1:** รอ Cloudflare challenge auto-resolve (60 วินาที)
2. **Phase 2:** ลองดึงวิดีโอจากหน้าหลักก่อน (8 ครั้ง)
3. **Phase 3:** ดึง av-kuy.com/v/ iframe URL (poll 10 ครั้ง × 2 วินาที)
4. **Phase 4:** Navigate ไปยัง iframe และผ่าน Cloudflare อีกครั้ง (45 วินาที)
5. **Phase 5:** คลิกและ poll หา m3u8 URL (10 ครั้ง × 3 วินาที)

**ความท้าทาย:**
- Cloudflare challenge 2 ชั้น (main page + iframe)
- Player iframe โหลดช้าจาก theme scripts/ads
- ต้อง re-navigate หลัง Cloudflare resolve

**ไฟล์ที่เกี่ยวข้อง:**
- `src-tauri/src/chrome_detector.rs:607-786` - Avkuy detection logic

---

### 4. Javwow (javwow.com)
**เทคนิค:** Cloudflare Bypass + Iframe Resolution  
**ต้องใช้ Chrome:** ✅ ใช่

**วิธีการทำงาน:**
1. **Phase 1:** รอ Cloudflare challenge auto-resolve (30 วินาที)
2. **Phase 2:** ดึง onlysubthai.com embed URL จาก iframe/og:video/JSON-LD
3. **Phase 3:** Navigate ไปยัง onlysubthai.com
4. **Phase 4:** คลิกและ poll หา m3u8 URL (8 ครั้ง × 3 วินาที)

**ความท้าทาย:**
- Cloudflare challenge บนหน้าหลัก
- onlysubthai.com ไม่มี Cloudflare (ง่ายกว่า avkuy)

**ไฟล์ที่เกี่ยวข้อง:**
- `src-tauri/src/javwow_parser.rs` - Parser หลัก
- `src-tauri/src/chrome_detector.rs:788-936` - Javwow detection logic

---

### 5. Rongyok (rongyok.com)
**เทคนิค:** HTTP Parsing (Discord CDN)  
**ต้องใช้ Chrome:** ❌ ไม่ต้อง

**วิธีการทำงาน:**
- ดึง HTML จาก series page ด้วย HTTP request
- Parse Discord CDN URLs จาก JavaScript/JSON
- รองรับหลายรูปแบบ:
  - `seriesData` JSON object
  - Discord CDN with numeric filename (1.mp4, 2.mp4)
  - Discord CDN with EP prefix (EP01.mp4, EP02.mp4)
  - Generic `video_url` in JSON

**ข้อดี:**
- ไม่ต้องใช้ Chrome (เร็วกว่า)
- ไม่มี Cloudflare
- รองรับ search และ browse

**ไฟล์ที่เกี่ยวข้อง:**
- `src-tauri/src/parser.rs` - Rongyok parser implementation

---

### 6. Thongyok (thongyok.com)
**เทคนิค:** HTTP Parsing + Episode Page Fetch  
**ต้องใช้ Chrome:** ❌ ไม่ต้อง

**วิธีการทำงาน:**
- ดึง series page เพื่อหา total episodes
- Fetch แต่ละ episode page: `https://thongyok.com/watch/{series_id}/{ep}`
- Parse Discord CDN URLs จากแต่ละหน้า

**ความแตกต่างจาก Rongyok:**
- Video URLs อยู่ในหน้าแต่ละตอน (ไม่ใช่ series page)
- ต้อง fetch หลาย requests (1 request ต่อ 1 ตอน)

**ไฟล์ที่เกี่ยวข้อง:**
- `src-tauri/src/parser.rs` - ใช้ RongyokParser ร่วมกัน

---

### 7. Baanjeen, HSCK, Titan
**เทคนิค:** HTTP Parsing  
**ต้องใช้ Chrome:** ❌ ไม่ต้อง

**วิธีการทำงาน:**
- ดึง HTML ด้วย HTTP request
- Parse video URLs จาก HTML/JavaScript
- แต่ละเว็บมี parser แยกกัน

**ไฟล์ที่เกี่ยวข้อง:**
- `src-tauri/src/baanjeen_parser.rs`
- `src-tauri/src/hsck_parser.rs`
- `src-tauri/src/titan_parser.rs`

---

## Chrome Detector Architecture

### PerformanceObserver Pattern
Chrome detector ใช้ `PerformanceObserver` API เพื่อดักจับ network requests:

```javascript
// Inject PerformanceObserver
window.__FOUND_URLS = [];
const observer = new PerformanceObserver((list) => {
    list.getEntries().forEach((entry) => {
        if (entry.name.includes('.m3u8') || entry.name.includes('.mp4')) {
            window.__FOUND_URLS.push(entry.name);
        }
    });
});
observer.observe({ entryTypes: ['resource'] });
```

### Click Strategy
Chrome detector ใช้ aggressive clicking เพื่อ trigger video load:

```javascript
// Click all potential play buttons
const selectors = [
    "button.jw-display-icon-container",
    ".jw-display-icon-display",
    "button[aria-label='Play']",
    ".jw-poster",
    "video"
];
```

### Cloudflare Bypass
สำหรับเว็บที่มี Cloudflare:
- รอ auto-resolve (30-60 วินาที)
- คลิก Turnstile checkbox
- ตรวจสอบ challenge page indicators
- Re-navigate หลัง resolve

---

## สรุป

### ต้องใช้ Chrome (4 เว็บ)
1. **NjavTV** - HLS.js polling
2. **Njav.org** - Iframe chain (2 ชั้น)
3. **Avkuy** - Cloudflare (2 ชั้น)
4. **Javwow** - Cloudflare + iframe

### ไม่ต้องใช้ Chrome (5+ เว็บ)
1. **Rongyok** - Discord CDN parsing
2. **Thongyok** - Discord CDN parsing
3. **Baanjeen** - HTTP parsing
4. **HSCK** - HTTP parsing
5. **Titan** - HTTP parsing

### เทคนิคหลัก
- **WebView + Polling:** NjavTV (window.hls.url)
- **Iframe Resolution:** Njav, Avkuy, Javwow
- **Cloudflare Bypass:** Avkuy, Javwow
- **HTTP Parsing:** Rongyok, Thongyok, Baanjeen, HSCK, Titan

---

## การพัฒนาในอนาคต

### แนวทางลด Chrome Dependency
1. **WebView Extraction Pattern:** ใช้ Tauri WebView แทน Chrome
   - ใช้ได้กับ Cloudflare sites (avkuy, javwow)
   - ไม่ต้องติดตั้ง Chrome แยก
   - ดูตัวอย่างใน memory: `webview-extraction-pattern.md`

2. **Packer Unpack:** สำหรับ JavaScript obfuscation
   - ใช้ร่วมกับ WebView pattern
   - Decode packed JavaScript

3. **API Reverse Engineering:** หา API endpoints โดยตรง
   - ลด overhead จาก browser automation
   - เร็วกว่าและเสถียรกว่า

---

**อัปเดตล่าสุด:** 2026-04-14  
**เวอร์ชัน:** 1.0
