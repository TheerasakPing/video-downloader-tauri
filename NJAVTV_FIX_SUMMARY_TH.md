# แก้ไขปัญหา NjavTV ค้างนาน

## ❌ ปัญหาเดิม

เมื่อวาง URL NjavTV แอปจะค้างอยู่ที่:
```
[INFO] Detected njavtv.com — using Chrome detector...
```
เป็นเวลานาน **4-5 นาที** เนื่องจาก:

### สาเหตุ
`NjavtvParser::get_series_info()` พยายาม fetch หน้าเว็บด้วย HTTP reqwest
- Cloudflare block request → timeout 30 วินาที
- แล้วค่อยไปใช้ Chrome detector อีก 20-30 วินาที
- **รวมทั้งหมด: 50-60+ วินาที** (หรือค้างถ้า timeout ไม่ทำงาน)

## ✅ การแก้ไข

### 1. ข้าม HTTP Fetch โดยสิ้นเชิง
**ไฟล์:** `src-tauri/src/njavtv_parser.rs`

**เดิม:**
```rust
let html = self.try_fetch(series_url).await.unwrap_or_default();  // ← ค้าง 30s!
if html.contains("_cf_chl") || ... {
    // ใช้ Chrome detection
}
```

**ใหม่:**
```rust
// ข้าม HTTP fetch — NjavTV เป็น Cloudflare เสมอ
// ไปใช้ Chrome detection โดยตรง
let title = self.extract_title_from_url(series_url);
return Ok(NjavtvSeriesInfo { ... });
```

**ผลลัพธ์:** ประหยัดเวลา 30 วินาที

### 2. ลด Chrome Detection Attempts
**ไฟล์:** `src-tauri/src/chrome_detector.rs`

**เดิม:** 15 ครั้ง × 2 วินาที = **30 วินาที**
**ใหม่:** 10 ครั้ง × 2 วินาที = **20 วินาที**

จากการทดสอบ พบว่า usually ได้ผลลัพธ์ใน 2-3 ครั้งแรก (4-6 วินาที)

## 📊 ผลลัพธ์

### ก่อนแก้ไข
```
[16:55:28] Detected njavtv.com
[16:59:34] ... ยังค้างอยู่ (4 นาที+) ❌
```

### หลังแก้ไข
```
[16:55:28] Detected njavtv.com — using Chrome detector...
[16:55:28] Launching Chrome to bypass Cloudflare...
[16:55:28] Detecting video from: https://njavtv.com/...
[16:55:34] Found video URL: https://surrit.com/... ✅
```

**เวลารวม: ~6 วินาที** (จาก 4+ นาที)

## 🧪 การทดสอบ

```bash
# ทดสอบความเร็ว
node test_njavtv_speed.mjs

# Expected output:
# ✓ Found m3u8 on attempt 1-3 (4-8s)
# ✅ SUCCESS! Detection completed in 6.0 seconds
```

## 📝 Timeline ที่ถูกต้อง

```
0.0s  → วาง URL
0.0s  → ตรวจจับว่าเป็น NjavTV
0.0s  → ข้าม HTTP fetch (ทันที)
0.5s  → เปิด Chrome
2.5s  → รอหน้าโหลด + player init
4.0s  → พบ window.hls.url (attempt 1)
6.0s  → ✅ เสร็จสิ้น!
```

## 🔧 ไฟล์ที่แก้ไข

1. `src-tauri/src/njavtv_parser.rs` - ข้าม HTTP fetch
2. `src-tauri/src/chrome_detector.rs` - ลด attempts เป็น 10 ครั้ง

## ⚠️ หมายเหตุ

### ทำไมไม่ใช้ HTTP fetch เลย?
- NjavTV ใช้ Cloudflare **ทุกครั้ง**
- ไม่มีทางได้ HTML ที่ถูกต้องจาก reqwest
- ต้องใช้ Chrome เท่านั้น

### แล้วทำไมไม่ใช้ Chrome fetch ตรงๆ?
- เราใช้ Chrome แล้ว! แต่ใช้ `window.hls.url` แทน
- NjavTV ตั้งค่า `window.hls` object เมื่อหน้าโหลด
- ไม่ต้องรอคลิกหรือทำอะไร แค่รอ 2-3 วินาที

### ถ้า detection ล้มเหลว?
- ลองอีกครั้ง (อาจเป็นปัญหา network ชั่วคราว)
- ตรวจสอบ Internet
- ดู Log ว่าติดขั้นตอนไหน
