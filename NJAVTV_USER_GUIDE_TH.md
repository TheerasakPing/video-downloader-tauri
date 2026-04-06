# วิธีดาวน์โหลดวิดีโอจาก NjavTV

## ✅ ใช้งานได้จริงแล้ว!

แอปนี้สามารถดาวน์โหลดวิดีโอจาก njavtv.com ได้เรียบร้อยแล้ว

## วิธีใช้งาน

### 1. เปิดแอป
```bash
npm run tauri dev
```

### 2. วาง URL
วางลิงก์ NjavTV ลงในช่อง URL ตัวอย่าง:
```
https://njavtv.com/th/dass-812-uncensored-leak
```

### 3. กด "Fetch Series"
- แอปจะตรวจจับว่าเป็นเว็บ NjavTV
- เปิด Chrome แบบ headless เพื่อหาวิดีโอ
- ดึง m3u8 URL และ cookies

### 4. เลือก Episode
- เลือกตอนที่ต้องการดาวน์โหลด
- หรือเลือกทั้งหมด

### 5. กด "Download"
- แอปจะเริ่มดาวน์โหลด
- ดูความคืบหน้าใน Log

## กระบวนการทำงาน

```
1. ตรวจจับ NjavTV URL
   ↓
2. เปิด Chrome แบบ headless
   ↓
3. หา m3u8 URL จาก window.hls.url
   ↓
4. ดึง cookies สำหรับ authentication
   ↓
5. ดาวน์โหลดผ่าน FFmpeg
   ↓
6. ถ้า FFmpeg ไม่สำเร็จ (เพราะ .jpeg segments)
   ↓
7. ใช้ Manual Download (reqwest + cookies)
   ↓
8. แปลงเป็น MP4
   ↓
✓ เสร็จสิ้น!
```

## หมายเหตุ

### ทำไมต้องใช้ Chrome?
- NjavTV ใช้ Cloudflare ป้องกันการเข้าถึงโดยตรง
- ต้องมี cookies ที่ถูกต้อง
- video URL ถูกซ่อนอยู่ใน JavaScript

### ทำไม FFmpeg ล้มเหลว?
- NjavTV ใช้ไฟล์ `.jpeg` เป็น video segments
- FFmpeg 8.x เข้มงวดเรื่อง extension
- **ไม่เป็นไร!** แอปจะสลับไปใช้ Manual Download อัตโนมัติ

### Manual Download คืออะไร?
- ดาวน์โหลดทีละ segment ด้วย reqwest
- ใช้ cookies เดียวกับ Chrome
- รวมไฟล์แล้วแปลงเป็น MP4
- ช้ากว่า FFmpeg เล็กน้อย แต่เสถียรกว่า

## การแก้ไขปัญหา

### ❌ "Could not find video URL"
**สาเหตุ:**
- หน้าเว็บโหลดไม่สมบูรณ์
- Internet ช้า
- Cloudflare challenge ยากเกินไป

**วิธีแก้:**
- ลองอีกครั้ง
- ตรวจสอบ Internet
- ดู Log ว่าติดขั้นตอนไหน

### ❌ Download ค้างที่ 0%
**สาเหตุ:**
- Cookie หมดอายุ (พบน้อย)
- CDN มีปัญหา
- Timeout 30 วินาที

**วิธีแก้:**
- ลองอีกครั้ง
- รอซักครู่แล้วลองใหม่

### ❌ FFmpeg Error
**นี่คือเรื่องปกติ!**
- FFmpeg จะล้มเหลวเพราะ `.jpeg` segments
- แอปจะสลับไป Manual Download อัตโนมัติ
- ดู Log จะมีข้อความ "[ManualHLS]"

## ตัวอย่าง Log ที่ถูกต้อง

```
[Info] Detected njavtv.com — using Chrome detector...
[Info] Launching Chrome to bypass Cloudflare...
[Info] Detecting video from: https://njavtv.com/th/dass-812...
[ChromeDetector] NjavTV detected - polling for HLS.js instance
[ChromeDetector] NjavTV: Found m3u8 via window.hls.url (attempt 2)
[Info] Found video URL: https://surrit.com/.../playlist.m3u8
[Downloader] Starting FFmpeg download...
[FFmpeg Error] detected format mpegts extension none mismatches...
[Info] FFmpeg HLS demuxer rejected segments — retrying with manual segment download...
[ManualHLS] Fetching master playlist...
[ManualHLS] Sub-playlist URL: https://surrit.com/.../720p/video.m3u8
[ManualHLS] Downloaded 150/1500 segments
[ManualHLS] Converting to MP4...
✓ Download complete!
```

## ไฟล์ที่ดาวน์โหลด

ไฟล์จะถูกบันทึกใน:
```
{output_dir}/njavtv/dass-812-uncensored-leak.mp4
```

(ถ้าเปิด "Group by Source" จะได้โฟลเดอร์ `njavtv/`)

## สนับสนุน

มีปัญหาหรือข้อเสนอแนะ? แจ้งได้เลย!
