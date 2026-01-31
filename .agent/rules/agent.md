---
trigger: always_on
---

# 🤖 Antigravity User Rules v2.0

> กฎสำหรับ Antigravity IDE — **ต้องทำตามทุกครั้งโดยไม่มีข้อยกเว้น**
> 
> ✨ **อัปเดตใหม่**: รองรับ Claude Skills + MCP Servers

---

## 📚 1. AUTO-READ Knowledge Base (บังคับ)

> ⚠️ **กฎนี้ต้องทำ AUTO ทุกครั้งที่เริ่มตอบคำถามใหม่**

### ก่อนตอบคำถามหรือทำงานใดๆ ให้ทำสิ่งนี้ก่อนเสมอ:

1. **อ่าน index.md ทันที** (ไม่ต้องถาม user)
   ```
   C:\Users\chawa\.gemini\antigravity\.agent\knowledge\index.md
   ```

2. **Smart Reading** - อ่านเพิ่มตามประเภทงาน:

| งานเกี่ยวกับ | อ่านเพิ่ม |
|-------------|----------|
| TikTok Uploader | `projects/tiktok-uploader/context.md` + `rules.md` |
| PSI Engine | `projects/psi-engine/context.md` + `rules.md` |
| TitanMirror | `projects/titan-mirror/context.md` + `rules.md` |
| แก้ Bug/ปัญหา | ค้นหาใน `solutions.md` + อ่าน `lessons.md` |
| เขียน Code ใหม่ | `memory/patterns.md` + `memory/snippets.md` |
| Architecture | `memory/decisions.md` |

3. **สรุปก่อนทำงาน** - บอกสั้นๆ ว่าอ่านอะไรบ้าง

---

## 💾 2. AUTO-SAVE Knowledge (บังคับ)

> ⚠️ **กฎนี้ต้องทำ AUTO ทุกครั้งหลังแก้ปัญหาสำเร็จ**

### หลังทำงานสำเร็จ ให้บันทึก AUTO (ไม่ต้องถาม user):

| ถ้าเจอ... | บันทึกลง... | บังคับ? |
|----------|------------|--------|
| วิธีแก้ปัญหาใหม่ | `knowledge/solutions.md` | ✅ ต้องทำ |
| บทเรียน/ข้อผิดพลาด | `knowledge/lessons.md` | ✅ ต้องทำ |
| Pattern ที่ใช้บ่อย | `memory/patterns.md` | ถ้าเจอ |
| Snippet ที่ใช้ซ้ำได้ | `memory/snippets.md` | ถ้าเจอ |
| Decision สำคัญ | `memory/decisions.md` | ถ้าเจอ |

### Format การบันทึก:
- ใช้ template ที่มีอยู่ในแต่ละไฟล์
- เพิ่ม metadata (date, project, tags)
- อัพเดท index.md ถ้าจำเป็น

---

## 🎯 3. Skills Integration (ใหม่!)

> ✨ Claude Skills = ชุดความรู้เฉพาะทางที่ทำให้ Claude ทำงานได้ดีขึ้น

### 3.1 Skills ที่มีอยู่

Claude มี Skills ในระบบที่สามารถใช้งานได้:

| Skill | ใช้สำหรับ | Location |
|-------|----------|----------|
| **docx** | สร้าง/แก้ไข Word documents | `/mnt/skills/public/docx/` |
| **pptx** | สร้าง/แก้ไข PowerPoint | `/mnt/skills/public/pptx/` |
| **xlsx** | สร้าง/แก้ไข Excel | `/mnt/skills/public/xlsx/` |
| **pdf** | จัดการ PDF files | `/mnt/skills/public/pdf/` |
| **frontend-design** | สร้าง UI/UX ที่สวยงาม | `/mnt/skills/public/frontend-design/` |
| **product-self-knowledge** | ความรู้เกี่ยว Claude products | `/mnt/skills/public/product-self-knowledge/` |

### 3.2 เมื่อไหร่ต้องอ่าน Skill?

| งานที่ต้องทำ | Skill ที่ต้องอ่าน |
|-------------|------------------|
| สร้าง/แก้ Word document | `view /mnt/skills/public/docx/SKILL.md` |
| สร้าง/แก้ PowerPoint | `view /mnt/skills/public/pptx/SKILL.md` |
| สร้าง/แก้ Excel | `view /mnt/skills/public/xlsx/SKILL.md` |
| จัดการ PDF | `view /mnt/skills/public/pdf/SKILL.md` |
| สร้าง web UI/component | `view /mnt/skills/public/frontend-design/SKILL.md` |

### 3.3 Skill Reading Priority

```
ลำดับการอ่าน:
1. อ่าน Antigravity Knowledge Base (index.md + project context)
2. อ่าน Skill ที่เกี่ยวข้อง (ถ้างานต้องใช้)
3. เริ่มทำงาน
```

**ตัวอย่าง:**
```
User: สร้าง PowerPoint สรุปโครงการ TikTok Uploader

AI ต้องทำ:
1. view index.md → รู้ว่า TikTok Uploader คืออะไร
2. view projects/tiktok-uploader/context.md → รายละเอียดโครงการ
3. view /mnt/skills/public/pptx/SKILL.md → วิธีสร้าง PowerPoint ที่ดี
4. เริ่มสร้าง presentation
```

---

## 🔌 4. MCP Servers Integration (ใหม่!)

> 🌐 MCP (Model Context Protocol) = ให้ Claude เชื่อมต่อกับ external tools/services

### 4.1 MCP Servers ที่อาจมี

Claude อาจมี MCP servers เชื่อมต่ออยู่ (ขึ้นกับการตั้งค่าของ user):

| MCP Server | หน้าที่ | ใช้เมื่อ |
|------------|--------|---------|
| **Filesystem** | อ่าน/เขียนไฟล์ | จัดการไฟล์ในเครื่อง |
| **GitHub** | จัดการ repos | Clone, push, PR |
| **Google Drive** | เข้าถึง Drive files | หาเอกสารใน Drive |
| **Slack** | ส่งข้อความ/อ่านแชท | ติดต่อทีม |
| **Database** | Query databases | ดึงข้อมูลจาก DB |
| **Browser** | เปิด/ควบคุม browser | Automation, scraping |

### 4.2 MCP Usage Rules

| ✅ ควรใช้ | ❌ ไม่ควรใช้ |
|----------|-------------|
| หาไฟล์ที่ user อัพโหลด | สำหรับงานที่ทำได้ด้วย bash |
| ดึงข้อมูลจาก external services | เมื่อมี built-in tool ทำได้ |
| Automate tasks ที่ซับซ้อน | งานง่ายๆ ที่ไม่จำเป็น |

### 4.3 MCP + Knowledge Base

เมื่อใช้ MCP ร่วมกับ Knowledge Base:

```
1. อ่าน Knowledge Base เพื่อรู้ context
2. ใช้ MCP เพื่อดึงข้อมูลเพิ่ม (ถ้าจำเป็น)
3. รวมข้อมูลทั้งหมดแล้วตอบ user
```

**ตัวอย่าง:**
```
User: หาไฟล์ TikTok Uploader config ล่าสุด

AI ต้องทำ:
1. view index.md → รู้ว่า config อยู่ที่ไหน
2. ใช้ MCP Filesystem หรือ bash → หาไฟล์
3. view file → อ่านเนื้อหา
4. ตอบ user
```

---

## 🔍 5. Search Methods (Simplified)

> ⚠️ **ใช้ bash tools ของ Claude แทนการอธิบายยืดยาว**

### 5.1 Tools ที่ใช้บ่อย

| Tool | ใช้สำหรับ | ตัวอย่าง |
|------|----------|---------|
| `view` | อ่านไฟล์/โฟลเดอร์ | `view /path/to/file.md` |
| `bash_tool` | Run commands | `bash_tool ls -la` |
| `str_replace` | แก้ไขไฟล์ | `str_replace old_text new_text` |
| `create_file` | สร้างไฟล์ใหม่ | `create_file /path/to/new.md` |

### 5.2 Search Best Practices

| สถานการณ์ | วิธีที่แนะนำ |
|-----------|-------------|
| หาไฟล์ | `bash: find . -name "*.js"` |
| ค้นหา text | `bash: grep -r "keyword" .` |
| ดูโครงสร้าง | `view /path/to/directory` |
| อ่านบางส่วน | `view` with line range |

### 5.3 Encoding Notes

- ✅ **bash tools รองรับ UTF-8** → ใช้ได้กับไฟล์ไทย/Emoji
- ✅ **view tool แสดงผลได้ดี** → แนะนำสำหรับไฟล์ที่มีภาษาไทย

---

## 📁 6. File & Folder Structure

```
C:\Users\chawa\.gemini\antigravity\.agent\

📁 knowledge/           ← ความรู้ทั่วไป
   📄 index.md          ← อ่านก่อนเสมอ!
   📄 solutions.md      ← วิธีแก้ปัญหา
   📄 lessons.md        ← บทเรียน
   📄 problems.md       ← โจทย์ฝึก

📁 memory/              ← Memory Banks
   📄 patterns.md       ← Design patterns
   📄 snippets.md       ← Code templates
   📄 decisions.md      ← Architecture decisions

📁 projects/            ← Project Context
   📁 tiktok-uploader/
   📁 psi-engine/
   📁 titan-mirror/

📁 workflows/           ← Commands
   📄 startup.md        ← /startup
   📄 solve-all.md      ← /solve-all
```

---

## 🎨 7. Code Style Rules

### General

| หัวข้อ | กฎ |
|--------|-----|
| ภาษา | **TypeScript** เมื่อเป็นไปได้ |
| Style | **Functional programming** > OOP |
| ตัวแปร | **camelCase** |
| Constants | **UPPER_SNAKE_CASE** |
| Indentation | **2 spaces** |
| Strings | **Single quotes** |

### Functions

| ✅ ทำ | ❌ ห้าม |
|------|--------|
| Function ทำหน้าที่เดียว | Function > 50 บรรทัด |
| ตั้งชื่อ verb+noun | ชื่อคลุมเครือ |
| Parameters ≤ 3 | Parameters > 5 |

### Comments

| Type | ใช้เมื่อ |
|------|--------|
| `// TODO:` | งานค้าง |
| `// FIXME:` | Bugs ที่รู้ |
| `// NOTE:` | หมายเหตุสำคัญ |
| JSDoc | Public functions |

---

## 🔒 8. Security Rules

### Secrets & Credentials

| ✅ ทำ | ❌ ห้าม |
|------|--------|
| ใช้ `.env` + `.gitignore` | Hardcode secrets |
| Environment variables | Commit credentials |
| Secrets manager (prod) | API keys in code |

### Input Validation

| ✅ ทำ | ❌ ห้าม |
|------|--------|
| Validate ทุก input | Trust user input |
| Zod/Yup validation | ใช้ `eval()` |
| Sanitize before display | `innerHTML` with user input |
| Parameterized queries | SQL string concat |

---

## ⚡ 9. Performance Rules

### Frontend

| ✅ ทำ | ❌ ห้าม |
|------|--------|
| Lazy loading | Load ทุกอย่างพร้อมกัน |
| Code splitting | Bundle ไฟล์เดียว |
| useMemo/useCallback | Re-render ไม่จำเป็น |
| Virtualize long lists | Render 1000+ items |

### Images

| ✅ ทำ | ❌ ห้าม |
|------|--------|
| WebP/AVIF | PNG/JPEG uncompressed |
| Lazy load | Load ทุกภาพ |
| Responsive images | ภาพขนาดเดียว |
| Compress | Upload 5MB+ |

### API

| ✅ ทำ | ❌ ห้าม |
|------|--------|
| Cache responses | Call API ซ้ำ |
| Pagination | Return ทุกอย่าง |
| Debounce search | Fire every keystroke |
| Field selection | Over-fetch |

### Performance Targets

| Metric | Target |
|--------|--------|
| FCP | < 1.8s |
| LCP | < 2.5s |
| TTI | < 3.8s |
| CLS | < 0.1 |
| API Response | < 200ms |

---

## ⚠️ 10. Error Handling

### General Rules

| ✅ ทำ | ❌ ห้าม |
|------|--------|
| try-catch เหมาะสม | Swallow errors |
| Log with context | แสดง internal error |
| Custom error classes | Generic Error ทุกที่ |
| Meaningful messages | Return stack trace |

### Error Response Format

```typescript
{
  success: false,
  error: {
    code: "VALIDATION_ERROR",
    message: "อีเมลไม่ถูกต้อง",
    details: { field: "email" }  // optional
  }
}
```

---

## 🤖 11. AI Response Style

### ภาษา

| สถานการณ์ | ตอบเป็น |
|-----------|--------|
| User พิมพ์ไทย | **ภาษาไทย** |
| User พิมพ์อังกฤษ | **ภาษาอังกฤษ** |
| Technical terms | **ไม่แปล** |

### รูปแบบการตอบ

| ✅ ทำ | ❌ ห้าม |
|------|--------|
| กระชับ ตรงประเด็น | ยาวเกินจำเป็น |
| Emoji เหมาะสม | Emoji มากเกินไป |
| Tables, bullets | Paragraph ยาว |
| Code blocks + syntax | Code ไม่มี highlight |

### พฤติกรรม

| ✅ ทำ | ❌ ห้าม |
|------|--------|
| ไม่แน่ใจ → ถาม | สมมติเอง |
| อธิบายเหตุผล | ทำเกินที่ขอ |
| บอก trade-offs | ซ่อนข้อจำกัด |

---

## ⚡ 12. Commands

| Command | Action |
|---------|--------|
| `/startup` | อ่าน knowledge ตาม Smart Reading |
| `/solve-all` | แก้โจทย์ใน problems.md + บันทึก memory |

---

## 📋 13. Workflow Checklist

### ก่อนเริ่มงาน:
- [ ] อ่าน `index.md` แล้ว?
- [ ] เกี่ยวกับ project ไหน? → อ่าน context
- [ ] ต้องใช้ Skill? → อ่าน SKILL.md
- [ ] ต้องใช้ MCP? → เตรียม MCP tools
- [ ] มี solution เดิม? → ค้นหา solutions.md

### ขณะทำงาน:
- [ ] ทำตาม Skill guidelines (ถ้ามี)
- [ ] ใช้ MCP เมื่อจำเป็น
- [ ] Follow code style rules
- [ ] Test ก่อน deliver

### หลังจบงาน:
- [ ] แก้ปัญหาอะไร? → บันทึก solutions.md
- [ ] เรียนรู้อะไร? → บันทึก lessons.md
- [ ] เจอ pattern ใหม่? → บันทึก patterns.md
- [ ] ย้ายไฟล์ไป `/mnt/user-data/outputs/`
- [ ] แชร์ไฟล์กับ user ด้วย `present_files`

---

## 🎯 Key Changes Summary

### ✂️ สิ่งที่ตัดออก:

1. **Search Methods ยืดยาว (9 วิธี)** → ตัดเหลือแค่ tools หลักของ Claude
   - เหตุผล: Claude มี bash_tool + view ที่ทำงานได้ดีแล้ว ไม่ต้องอธิบาย PowerShell/grep ละเอียด
   
2. **PowerShell/CMD specific commands** → ใช้ bash_tool แทน
   - เหตุผล: bash_tool รองรับ UTF-8 และใช้งานง่ายกว่า
   
3. **Folder Structure แยกตาม Web Projects** → ใช้โครงสร้างเดิม
   - เหตุผล: ไม่จำเป็นต้องกำหนดเกินไป ให้ยืดหยุ่นตาม project

4. **Documentation Rules แยกเป็น section** → รวมเข้า Best Practices
   - เหตุผล: ไม่ซ้ำซ้อน

### ➕ สิ่งที่เพิ่มเข้ามา:

1. **Section 3: Skills Integration** ✨
   - รายชื่อ Skills ที่มี
   - เมื่อไหร่ต้องอ่าน Skill
   - ลำดับการอ่าน (Knowledge Base → Skill → ทำงาน)

2. **Section 4: MCP Servers Integration** 🔌
   - MCP servers ที่อาจมี
   - MCP usage rules
   - การใช้ร่วมกับ Knowledge Base

3. **Workflow Checklist แบบใหม่** 📋
   - เพิ่มขั้นตอนเช็ค Skill + MCP
   - เพิ่ม present_files และ outputs directory

4. **Key Changes Summary** 🎯
   - สรุปสิ่งที่เปลี่ยนแปลง
   - อธิบายเหตุผล

---

## 💡 Best Practices

1. **ไฟล์เล็ก** → ใช้ `view` อ่านเลย
2. **ไฟล์ใหญ่** → ใช้ `bash: grep` ค้นหาก่อน
3. **Documents** → อ่าน Skill ก่อนสร้าง
4. **Source code** → ใช้ `view` + line range
5. **MCP tasks** → อ่าน Knowledge Base ก่อน

---

> 💡 **เป้าหมาย**: ทำให้ Antigravity ทำงานร่วมกับ Claude Skills + MCP ได้อย่างลงตัว!
>
> ⚠️ **สำคัญ**: กฎเหล่านี้ต้องทำ **อัตโนมัติ** ไม่ต้องรอ user สั่ง!