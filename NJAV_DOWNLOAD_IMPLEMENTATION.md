# 🎬 NJAV.ORG Download Support - สรุปการแก้ไข

## 🔧 สิ่งที่แก้ไข

### 1. Backend (Rust)

#### `src-tauri/src/njav_parser.rs`
- เพิ่ม field `javxx_url` ใน `NjavSeriesInfo` สำหรับเก็บ URL ของ javxx.com โดยตรง
- Parser จะสร้าง javxx.com URL จาก video code (เช่น `snos-034` → `https://javxx.com/v/snos-034`)
- URL นี้ใช้ bypass Cloudflare protection ของ njav.org

#### `src-tauri/src/lib.rs`
- เพิ่ม `debug_log_njav()` helper function สำหรับเขียน debug log ไปยัง `/tmp/njav-chrome-debug.log`
- ปรับปรุง `fetch_series` สำหรับ njav.org:
  - **Strategy ใหม่**: ลองหลาย URL ตามลำดับ
    1. `javxx.com/v/{code}` (ก่อน - bypass Cloudflare)
    2. `njav.org/{code}` (fallback)
    3. Original URL (fallback สุดท้าย)
  - ถ้า URL แรกไม่สำเร็จ จะลอง URL ถัดไปอัตโนมัติ
  - เพิ่ม debug logging ทุกขั้นตอน

#### `src-tauri/src/chrome_detector.rs`
- เพิ่ม method `extract_m3u8_from_html()` - ดึง m3u8 URL จาก HTML source โดยตรง (fallback สุดท้าย)
- เพิ่ม method `extract_m3u8_rust_regex()` - ใช้ Rust regex ค้นหา m3u8 ใน HTML
- ปรับปรุง njav detection flow:
  - **Phase 1**: หา video iframe บน javxx.com SPA (poll สูงสุด 20 ครั้ง)
  - **Phase 2**: ตรวจสอบ PerformanceObserver สำหรับ m3u8 URL
  - **Phase 3**: Navigate ไปที่ surrit iframe URL โดยตรง
  - **Phase 4**: ดึง m3u8 จาก HTML source ของ surrit page
- เพิ่ม diagnostic logging ทุก 5 attempts
- เพิ่ม HTML dump ไปยัง `/tmp/njav_surrit_dump.html` และ `/tmp/njav_html_analysis.html` สำหรับ debug

### 2. Frontend (React)

#### `src/App.tsx`
- เปลี่ยน default URL เป็นค่าว่าง (ให้ผู้ใช้ paste หรือ auto-detect เอง)
- เพิ่ม placeholder แสดงตัวอย่าง URL ของทุก domain ที่รองรับ
- เพิ่ม **"🔍 Auto Detect URL"** button - เปิด Chrome เพื่อตรวจจับ video URL โดยตรง
- เพิ่ม `handleAutoDetectUrl()` handler - เรียก Tauri command `auto_detect_video_url`
- เพิ่ม state `autoDetectUrl` สำหรับแสดง loading ขณะ detect

### 3. การแยกโฟลเดอร์ตาม Source

เมื่อเปิดใช้งาน **Group by Website** ใน Settings:
```
~/Downloads/rongyok/
├── rongyok/      ← ไฟล์จาก rongyok.com
├── titan/        ← ไฟล์จาก 51cg1.com
├── baanjeen/     ← ไฟล์จาก baanjeen
├── hsck/         ← ไฟล์จาก hsck123.com
├── njavtv/       ← ไฟล์จาก njavtv.com
└── njav/         ← ไฟล์จาก njav.org  ⭐ ใหม่
```

## 🧪 วิธีทดสอบ

### วิธีที่ 1: ใช้ Tauri Dev Mode
```bash
cd /Volumes/Data/_workspace/_git/video-downloader-tauri
npm run tauri dev
```

1. วาง URL: `https://njav.org/snos-034/`
2. กดปุ่ม **🔍** (Auto Detect) หรือ **Search**
3. รอ Chrome ตรวจจับ video URL (อาจใช้ 10-30 วินาที)
4. ถ้าเจอ URL จะแสดง Series Card พร้อมปุ่ม Download
5. กด **Download** เพื่อดาวน์โหลด

### วิธีที่ 2: ใช้ Auto Capture
1. เปิด **Clipboard Monitor** (ปุ่ม AUTO)
2. คัดลอก URL `https://njav.org/snos-034/`
3. โปรแกรมจะตรวจจับและเพิ่มเข้า Queue อัตโนมัติ

### วิธีที่ 3: ใช้ Smart Queue (Batch Mode)
1. วางหลาย URL พร้อมกัน
2. โปรแกรมจะเข้า Batch Mode อัตโนมัติ
3. กด Play เพื่อเริ่มดาวน์โหลดทีละรายการ

## 📋 Debug Log Files

หากมีปัญหา ตรวจสอบ log files เหล่านี้:
```bash
# Chrome detection log (njav.org เท่านั้น)
cat /tmp/njav-chrome-debug.log

# Surrit page HTML dump (สำหรับวิเคราะห์)
cat /tmp/njav_surrit_dump.html

# Full HTML analysis
cat /tmp/njav_html_analysis.html

# Chrome detector debug
cat /tmp/njav-chrome-debug.log
```

## 🔍 Flow การทำงาน

```
ผู้ใช้วาง URL: https://njav.org/snos-034/
    ↓
NjavParser.parse()
    → แยก video code: "snos-034"
    → สร้าง javxx.com URL: https://javxx.com/v/snos-034
    ↓
lib.rs: fetch_series()
    → ลอง javxx.com URL ก่อน (bypass Cloudflare)
    → Chrome detector → Detect m3u8
    ↓
Chrome Detector:
    Phase 1: หา iframe บน javxx.com SPA
    Phase 2: ตรวจสอบ PerformanceObserver
    Phase 3: Navigate ไป surrit URL โดยตรง
    Phase 4: ดึง m3u8 จาก HTML source
    ↓
ได้ m3u8 URL → UnifiedSeriesInfo
    ↓
Downloader.download_episode()
    → HLS stream (.m3u8) → FFmpeg download
    → ถ้า FFmpeg ล้มเหลว → Manual segment download
    ↓
บันทึกไฟล์: ~/Downloads/rongyok/njav/SNOS-034_EP1.mp4
```

## ⚠️ ข้อควรทราบ

1. **ต้องติดตั้ง Google Chrome** - Chrome detector ใช้ headless Chrome เพื่อ bypass Cloudflare
2. **ใช้เวลานาน** - การตรวจจับอาจใช้เวลา 15-60 วินาที ขึ้นอยู่กับความเร็วอินเทอร์เน็ต
3. **Region blocked** - บางวิดีโออาจไม่สามารถเข้าถึงได้จากประเทศไทย
4. **FFmpeg ต้องการสำหรับ HLS** - การดาวน์โหลด m3u8 stream ต้องการ FFmpeg

## 📦 Build

```bash
# Frontend
npm run build

# Rust check
cargo check --manifest-path src-tauri/Cargo.toml

# Full Tauri build
npm run tauri build
```

## ✅ Status

- [x] NjavParser รองรับ njav.org
- [x] javxx.com URL generation (bypass Cloudflare)
- [x] Multi-URL fallback strategy ใน lib.rs
- [x] Chrome detector ปรับปรุงสำหรับ njav/missav/javxx
- [x] m3u8 extraction จาก HTML source
- [x] Auto Detect URL button ใน UI
- [x] แยกโฟลเดอร์ njav/ ใน Group by Source
- [x] Placeholder อัปเดตแสดงทุก domain
- [x] Rust compilation ✅
- [x] Frontend build ✅
