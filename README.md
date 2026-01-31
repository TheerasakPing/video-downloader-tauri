# 🎬 Rongyok Video Downloader

> 📥 โปรแกรมดาวน์โหลดวิดีโอจาก rongyok.com แบบครบวงจร

![Tauri](https://img.shields.io/badge/Tauri-2.0-blue?logo=tauri)
![React](https://img.shields.io/badge/React-19-61DAFB?logo=react)
![TypeScript](https://img.shields.io/badge/TypeScript-5.0-3178C6?logo=typescript)
![Rust](https://img.shields.io/badge/Rust-1.75-orange?logo=rust)
![License](https://img.shields.io/badge/License-MIT-green)

---

## ✨ คุณสมบัติเด่น

| ฟีเจอร์                   | รายละเอียด                                   |
| ------------------------- | -------------------------------------------- |
| 🚀 **Smart Queue**        | จัดการคิวอัตโนมัติ (Sequential Download)     |
| 📋 **Auto Capture**       | ตรวจจับลิงก์จาก Clipboard อัตโนมัติ          |
| 🎯 **ดาวน์โหลดหลายตอน**   | เลือกตอนที่ต้องการดาวน์โหลดได้อิสระ          |
| ⚡ **ดาวน์โหลดพร้อมกัน**  | รองรับดาวน์โหลดหลายไฟล์พร้อมกัน              |
| 🔀 **รวมไฟล์อัตโนมัติ**   | ใช้ FFmpeg รวมตอนเป็นไฟล์เดียว               |
| 📊 **กราฟความเร็ว**       | แสดงความเร็วดาวน์โหลดแบบ Real-time           |
| 🎨 **UI สวยงาม**          | 5 ธีมสี (Violet, Blue, Emerald, Amber, Rose) |
| 🌍 **Multi-language**     | รองรับภาษาไทยและอังกฤษ (TH/EN)               |
| ⌨️ **Keyboard Shortcuts** | คีย์ลัด (Ctrl+V, Ctrl+D, Space, etc.)        |
| 🖼️ **Mini Mode**          | โหมดหน้าต่างเล็กสำหรับ Monitor ดาวน์โหลด     |
| 🖱️ **Drag & Drop**        | ลาก URL มาวางเพื่อเริ่มดาวน์โหลด             |
| 📱 **Responsive**         | ใช้งานได้ทุกขนาดหน้าจอ                       |
| 💾 **Resume Download**    | ดาวน์โหลดต่อจากที่ค้างไว้ได้                 |
| 🚀 **ข้ามแพลตฟอร์ม**      | รองรับ Windows, macOS, Linux                 |

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
- **FFmpeg** (ไม่บังคับ - สำหรับรวมไฟล์)

### 🍎 macOS

```bash
# ติดตั้ง FFmpeg (ถ้าต้องการรวมไฟล์)
brew install ffmpeg

# ดาวน์โหลดและติดตั้ง .dmg จาก Releases
```

### 🪟 Windows

```bash
# ติดตั้ง FFmpeg (ถ้าต้องการรวมไฟล์)
winget install FFmpeg

# ดาวน์โหลดและรัน .msi หรือ .exe จาก Releases
```

### 🐧 Linux

```bash
# Ubuntu/Debian
sudo apt install ffmpeg

# Fedora
sudo dnf install ffmpeg

# ดาวน์โหลด .deb, .rpm หรือ .AppImage จาก Releases
```

---

## 🚀 การใช้งาน

### 1️⃣ วาง URL

คัดลอก URL จาก rongyok.com แล้ววางในช่อง URL

```
รูปแบบที่รองรับ:
✅ https://rongyok.com/watch/?series_id=12345
✅ https://rongyok.com/series/12345/ชื่อซีรี่ส์
```

### 2️⃣ กด Fetch

โปรแกรมจะดึงข้อมูลซีรี่ส์และ URL ของทุกตอน

### 3️⃣ เลือกตอน

- คลิกเลือกตอนที่ต้องการ
- หรือกด **Select All** เพื่อเลือกทั้งหมด

### 4️⃣ กด Download

โปรแกรมจะเริ่มดาวน์โหลดและแสดงความคืบหน้า

### 5️⃣ รวมไฟล์ (อัตโนมัติ)

ถ้าติ๊ก ✅ "Merge videos after download" โปรแกรมจะรวมไฟล์อัตโนมัติ

---

## ⚙️ ตั้งค่า

| ตัวเลือก                    | คำอธิบาย                                      |
| --------------------------- | --------------------------------------------- |
| 📂 **Output Directory**     | โฟลเดอร์สำหรับบันทึกไฟล์                      |
| 🔢 **Concurrent Downloads** | จำนวนไฟล์ที่ดาวน์โหลดพร้อมกัน (1-5)           |
| 🐌 **Speed Limit**          | จำกัดความเร็ว (0 = ไม่จำกัด)                  |
| 📝 **File Naming**          | รูปแบบชื่อไฟล์ (ep_001, episode_1, title_ep1) |
| 🔀 **Auto Merge**           | รวมไฟล์อัตโนมัติหลังดาวน์โหลด                 |
| 🌙 **Theme**                | ธีม Dark / Light / System                     |
| 🔔 **Notifications**        | แจ้งเตือนเมื่อดาวน์โหลดเสร็จ                  |
| 🔊 **Sound**                | เสียงแจ้งเตือน                                |

---

## 🏗️ พัฒนาเอง

### Clone และติดตั้ง

```bash
git clone https://github.com/your-username/rongyok-downloader.git
cd rongyok-downloader

# ติดตั้ง dependencies
npm install
```

### รันโหมดพัฒนา

```bash
npm run tauri dev
```

### Build สำหรับ Production

```bash
npm run tauri build
```

---

## 📁 โครงสร้างโปรเจค

```
rongyok-downloader/
├── 📂 src/                    # React Frontend
│   ├── 📂 components/         # UI Components
│   │   ├── Button.tsx
│   │   ├── EpisodeSelector.tsx
│   │   ├── FileBrowser.tsx
│   │   ├── ProgressBar.tsx
│   │   ├── SeriesCard.tsx
│   │   ├── SettingsPanel.tsx
│   │   └── SpeedGraph.tsx
│   ├── 📂 hooks/              # Custom Hooks
│   │   ├── useHistory.ts
│   │   ├── useLogger.ts
│   │   ├── useSettings.ts
│   │   └── useSpeedGraph.ts
│   ├── 📂 types/              # TypeScript Types
│   ├── App.tsx                # Main App
│   ├── index.css              # Styles + Animations
│   └── main.tsx               # Entry Point
│
├── 📂 src-tauri/              # Rust Backend
│   ├── 📂 src/
│   │   ├── lib.rs             # Tauri Commands
│   │   ├── downloader.rs      # Download Logic
│   │   ├── parser.rs          # URL Parser
│   │   └── main.rs            # Entry Point
│   ├── Cargo.toml             # Rust Dependencies
│   └── tauri.conf.json        # Tauri Config
│
├── 📂 .github/workflows/      # CI/CD
│   ├── ci.yml                 # Test & Build
│   └── release.yml            # Release All Platforms
│
├── package.json
├── tsconfig.json
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
