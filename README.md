# 🎬 Rongyok Video Downloader

> 📥 โปรแกรมดาวน์โหลดวิดีโอจากเว็บ Streaming ชั้นนำ (Rongyok, BaanJeen, 357ms) แบบครบวงจร

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)
![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?logo=typescript)
![Rust](https://img.shields.io/badge/Rust-1.75-orange?logo=rust)
![Python](https://img.shields.io/badge/Python-3.8+-yellow?logo=python)
![License](https://img.shields.io/badge/License-MIT-green)

---

## ✨ คุณสมบัติเด่น

| ฟีเจอร์                   | รายละเอียด                                   |
| ------------------------- | -------------------------------------------- |
| 🌐 **Multi-Site Support** | รองรับ Rongyok, BaanJeen, 357ms, และ Direct URL |
| 🧠 **Hybrid Parsing**     | ระบบตรวจจับวิดีโออัจฉริยะ (Static + Chrome Detector) |
| 🚀 **Smart Queue**        | จัดการคิวอัตโนมัติ (Sequential Download)     |
| 📋 **Auto Capture**       | ตรวจจับลิงก์จาก Clipboard อัตโนมัติ          |
| 🎯 **ดาวน์โหลดหลายตอน**   | เลือกตอนที่ต้องการดาวน์โหลดได้อิสระ          |
| ⚡ **ดาวน์โหลดพร้อมกัน**  | รองรับดาวน์โหลดหลายไฟล์พร้อมกัน              |
| 🔀 **รวมไฟล์อัตโนมัติ**   | ใช้ FFmpeg รวมตอนเป็นไฟล์เดียว               |
| 📊 **กราฟความเร็ว**       | แสดงความเร็วดาวน์โหลดแบบ Real-time           |
| 🎨 **UI สวยงาม**          | 5 ธีมสี (Violet, Blue, Emerald, Amber, Rose) |
| 🌍 **Multi-language**     | รองรับภาษาไทยและอังกฤษ (TH/EN)               |
| 🖼️ **Mini Mode**          | โหมดหน้าต่างเล็กสำหรับ Monitor ดาวน์โหลด     |
| 💾 **Resume Download**    | ดาวน์โหลดต่อจากที่ค้างไว้ได้                 |

---

## 📸 ภาพหน้าจอ

```
┌─────────────────────────────────────────────────────────┐
│  🎬 Rongyok Downloader          [Download][Files][⚙️]  │
├─────────────────────────────────────────────────────────┤
│                                                         │
│  🔗 Series URL: [https://rongyok.com/watch/...]        │
│  📂 Output: [~/Downloads/rongyok]          [Browse]    │
│                                                         │
│  ┌─────────────────────────────────────────────────┐   │
│  │ 📺 ชื่อซีรี่ส์                    62 Episodes  │   │
│  │ ID: 12345                        URLs cached ✓ │   │
│  └─────────────────────────────────────────────────┘   │
│                                                         │
│  Episodes (62/62 selected)         [Select All]        │
│  ┌──┬──┬──┬──┬──┬──┬──┬──┬──┬──┐                      │
│  │1 │2 │3 │4 │5 │6 │7 │8 │9 │10│ ...                  │
│  └──┴──┴──┴──┴──┴──┴──┴──┴──┴──┘                      │
│                                                         │
│  📈 Speed: 5.2 MB/s | Avg: 4.8 MB/s | Peak: 8.1 MB/s  │
│  ████████████████░░░░░░░░░░ 65% Episode 40            │
│                                                         │
│  [🚀 Download (62)]  [⏸️ Pause]  [❌ Cancel]           │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

---

## 🛠️ ติดตั้ง

### 📋 ความต้องการ

- **Node.js** 18+
- **Rust** 1.70+
- **Python** 3.8+ (สำหรับบางเว็บไซต์)
- **FFmpeg** (ไม่บังคับ - สำหรับรวมไฟล์)

### 🍎 macOS

```bash
# ติดตั้ง FFmpeg (ถ้าต้องการรวมไฟล์)
brew install ffmpeg python

# ดาวน์โหลดและติดตั้ง .dmg จาก Releases
```

### 🪟 Windows

```bash
# ติดตั้ง FFmpeg (ถ้าต้องการรวมไฟล์)
winget install FFmpeg Python.Python.3

# ดาวน์โหลดและรัน .msi หรือ .exe จาก Releases
```

### 🐧 Linux

```bash
# Ubuntu/Debian
sudo apt install ffmpeg python3

# ดาวน์โหลด .deb, .rpm หรือ .AppImage จาก Releases
```

---

## 🚀 การใช้งาน

### 1️⃣ วาง URL

คัดลอก URL จากเว็บที่รองรับ แล้ววางในช่อง URL

```
รูปแบบที่รองรับ:
✅ https://rongyok.com/watch/?series_id=...
✅ https://baanjeen.com/...
✅ https://357ms.com/...
✅ Direct URL (.mp4, .m3u8)
```

### 2️⃣ กด Fetch

โปรแกรมจะดึงข้อมูลซีรี่ส์และ URL ของทุกตอน

### 3️⃣ เลือกตอน

- คลิกเลือกตอนที่ต้องการ
- หรือกด **Select All** เพื่อเลือกทั้งหมด

### 4️⃣ กด Download

โปรแกรมจะเริ่มดาวน์โหลดและแสดงความคืบหน้า

---

## 📁 โครงสร้างโปรเจค

```
rongyok-downloader/
├── 📂 src/                    # React Frontend
│   ├── 📂 components/         # UI Components
│   ├── 📂 hooks/              # Custom Hooks
│   ├── App.tsx                # Main App
│   └── main.tsx               # Entry Point
│
├── 📂 src-tauri/              # Rust Backend
│   ├── 📂 src/
│   │   ├── lib.rs             # Tauri Commands & App State
│   │   ├── main.rs            # Entry Point
│   │   ├── downloader.rs      # Download Logic (Async/Stream)
│   │   ├── parser.rs          # Rongyok Parser
│   │   ├── baanjeen_parser.rs # BaanJeen Parser
│   │   ├── titan_parser.rs    # Titan Parser
│   │   ├── chrome_detector.rs # Hybrid Detection Logic
│   │   ├── python_interface.rs# Python Bridge (357ms)
│   │   └── utils.rs           # Helpers
│   ├── Cargo.toml             # Rust Dependencies
│   └── tauri.conf.json        # Tauri Config
│
├── 📂 scripts/                # Utility Scripts
│   ├── 357ms_extractor.py
│   └── web_video_extractor.py
│
├── package.json
└── vite.config.ts
```

---

## 🔧 เทคโนโลยี

### Frontend

- ⚛️ **React 19** - UI Framework
- 📘 **TypeScript** - Type Safety
- 🎨 **Tailwind CSS** - Styling
- 📊 **Lucide Icons** - Icons

### Backend

- 🦀 **Rust** - System Programming
- 🖥️ **Tauri 2.0** - Desktop Framework
- 🔗 **Reqwest** - HTTP Client
- 🕸️ **Scraper** - HTML Parser
- 🎬 **FFmpeg** - Video Processing

---

## 📄 License

MIT License - ใช้งานได้อิสระ ⚖️

---

## 🤝 Contributing

ยินดีรับ Pull Requests!

1. Fork โปรเจค
2. สร้าง Branch (`git checkout -b feature/amazing`)
3. Commit (`git commit -m 'Add amazing feature'`)
4. Push (`git push origin feature/amazing`)
5. เปิด Pull Request

---

## 💖 Credits

- สร้างด้วย ❤️ โดยใช้ [Tauri](https://tauri.app)
- ไอคอนจาก [Lucide](https://lucide.dev)
- UI Components จาก [Tailwind CSS](https://tailwindcss.com)

---

<div align="center">

**⭐ ถ้าชอบโปรเจคนี้ อย่าลืมกด Star ด้วยนะ! ⭐**

Made with 🦀 Rust + ⚛️ React + 💜 Tauri

</div>
