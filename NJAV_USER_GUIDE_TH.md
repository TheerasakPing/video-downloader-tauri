# วิธีดาวน์โหลดวิดีโอจาก NJAV.org

## ✅ ใช้งานได้จริงแล้ว!

แอปนี้รองรับการดาวน์โหลดวิดีโอจาก **njav.org** เรียบร้อยแล้ว

## วิธีใช้งาน

### 1. เปิดแอป
```bash
npm run tauri dev
```

### 2. วาง URL
วางลิงก์ NJAV ลงในช่อง URL ตัวอย่าง:
```
https://njav.org/snos-034/
```

### 3. กด "Fetch Series"
- แอปจะตรวจจับว่าเป็นเว็บ NJAV.org
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
1. ตรวจจับ NJAV.org URL
   ↓
2. ข้าม Cloudflare (ใช้ javxx.com แทน)
   ↓
3. เปิด Chrome แบบ headless
   ↓
4. หา surrit.store iframe
   ↓
5. หา m3u8 URL จาก network requests
   ↓
6. ดึง cookies สำหรับ authentication
   ↓
7. ดาวน์โหลดผ่าน FFmpeg
   ↓
8. ถ้า FFmpeg ไม่สำเร็จ (เพราะ .jpeg segments)
   ↓
9. ใช้ Manual Download (reqwest + cookies)
   ↓
10. แปลงเป็น MP4
    ↓
✓ เสร็จสิ้น!
```

## โฟลเดอร์แยกอัตโนมัติ

เมื่อเปิด **"Group by Source"** ใน Settings:
- ไฟล์จะถูกบันทึกในโฟลเดอร์ `njav/`
- ตัวอย่าง: `{output_dir}/njav/SNOS-034.mp4`

## การตั้งค่า Auto-Detect URL

แอปมีระบบ Auto-Detect URL หลายแบบ:

### 1. **Auto-Paste on Startup**
- เมื่อเปิดแอป จะวาง URL จาก clipboard อัตโนมัติ

### 2. **Auto-Fetch on Focus**
- เมื่อหน้าต่างได้รับ focus จะดึง URL จาก clipboard และ fetch อัตโนมัติ

### 3. **Batch Mode**
- วางหลาย URL พร้อมกัน (คั่นด้วย newline หรือ space)
- แอปจะสลับเป็น Batch Mode อัตโนมัติ
- ดาวน์โหลดทีละวิดีโอตามคิว

### 4. **Auto-Capture Mode**
- เปิดปุ่ม Clipboard Monitor
- ตรวจจับ URL ใหม่ใน clipboard ทุก 1 วินาที
- เพิ่มเข้าคิวดาวน์โหลดอัตโนมัติ

## หมายเหตุ

### ทำไมต้องใช้ Chrome?
- NJAV.org ใช้ Cloudflare ป้องกันการเข้าถึงโดยตรง
- ต้องมี cookies ที่ถูกต้อง
- วิดีโอถูกซ่อนอยู่ใน nested iframes (njav.org → missav.guide → javxx.com → surrit.store)

### ทำไม FFmpeg ล้มเหลว?
- NJAV ใช้ไฟล์ `.jpeg` เป็น video segments (เพื่อหลบการตรวจจับ)
- FFmpeg 8.x เข้มงวดเรื่อง extension
- **ไม่เป็นไร!** แอปจะสลับไปใช้ Manual Download อัตโนมัติ

### Manual Download คืออะไร?
- ดาวน์โหลดทีละ segment ด้วย reqwest
- ใช้ cookies เดียวกับ Chrome
- รวมไฟล์แล้วแปลงเป็น MP4
- ช้ากว่า FFmpeg เล็กน้อย แต่เสถียรกว่า

## ตัวอย่าง Log ที่ถูกต้อง

```
[Info] Detected njav.org — using Chrome detector...
[Info] Launching Chrome to detect video from njav.org iframes...
[Info] Detecting video from: https://njav.org/snos-034/
[njav] Starting detection for: https://njav.org/snos-034/
[njav] Found video iframe (attempt 3): https://surrit.store/...
[njav] Found m3u8 URL: https://wowstream.com/.../playlist.m3u8
[Info] Found video URL: https://wowstream.com/.../playlist.m3u8
[Downloader] Starting FFmpeg download...
[FFmpeg Error] detected format mpegts extension none mismatches...
[Info] FFmpeg HLS demuxer rejected segments — retrying with manual segment download...
[ManualHLS] Fetching master playlist...
[ManualHLS] Downloaded 150/1500 segments
[ManualHLS] Converting to MP4...
✓ Download complete!
```

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

## ไฟล์ที่ดาวน์โหลด

ไฟล์จะถูกบันทึกใน:
```
{output_dir}/njav/SNOS-034.mp4
```

(ถ้าเปิด "Group by Source" จะได้โฟลเดอร์ `njav/`)

## ตัวอย่าง URL ที่ใช้งานได้

- `https://njav.org/snos-034/`
- `https://njav.org/abc-123/`
- `https://njav.org/xyz-789-uncensored/`

## สนับสนุน

มีปัญหาหรือข้อเสนอแนะ? แจ้งได้เลย!
