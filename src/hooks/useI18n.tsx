import { useState, useCallback, createContext, useContext, ReactNode } from "react";

type Language = "en" | "th";

interface Translations {
  [key: string]: {
    en: string;
    th: string;
  };
}

const translations: Translations = {
  // App
  "app.title": { en: "Rongyok", th: "Rongyok" },
  "app.subtitle": { en: "Video Downloader", th: "ดาวน์โหลดวิดีโอ" },

  // Tabs
  "tab.download": { en: "Download", th: "ดาวน์โหลด" },
  "tab.files": { en: "Files", th: "ไฟล์" },
  "tab.history": { en: "History", th: "ประวัติ" },
  "tab.settings": { en: "Settings", th: "ตั้งค่า" },
  "tab.logs": { en: "Logs", th: "บันทึก" },

  // Download
  "download.url": { en: "Series URL", th: "URL ซีรีส์" },
  "download.urlPlaceholder": { en: "https://rongyok.com/watch/?series_id=XXX", th: "https://rongyok.com/watch/?series_id=XXX" },
  "download.outputDir": { en: "Output Directory", th: "โฟลเดอร์บันทึก" },
  "download.fetch": { en: "Fetch", th: "ดึงข้อมูล" },
  "download.paste": { en: "Paste", th: "วาง" },
  "download.browse": { en: "Browse", th: "เรียกดู" },
  "download.open": { en: "Open", th: "เปิด" },
  "download.start": { en: "Download", th: "ดาวน์โหลด" },
  "download.pause": { en: "Pause", th: "หยุดชั่วคราว" },
  "download.resume": { en: "Resume", th: "ดำเนินต่อ" },
  "download.cancel": { en: "Cancel", th: "ยกเลิก" },
  "download.resumePrevious": { en: "Resume Previous", th: "ดำเนินต่อ" },
  "download.autoMerge": { en: "Merge videos after download", th: "รวมวิดีโอหลังดาวน์โหลด" },
  "download.dragDrop": { en: "Drag & drop URL here", th: "ลาก URL มาวางที่นี่" },
  "download.dropHere": { en: "Drop to add URL", th: "ปล่อยเพื่อเพิ่ม URL" },

  // Episodes
  "episodes.title": { en: "Episodes", th: "ตอน" },
  "episodes.selectAll": { en: "Select All", th: "เลือกทั้งหมด" },
  "episodes.deselectAll": { en: "Deselect All", th: "ยกเลิกเลือก" },
  "episodes.selected": { en: "selected", th: "ที่เลือก" },

  // Settings
  "settings.download": { en: "Download Settings", th: "ตั้งค่าดาวน์โหลด" },
  "settings.concurrent": { en: "Concurrent Downloads", th: "ดาวน์โหลดพร้อมกัน" },
  "settings.concurrentDesc": { en: "Number of episodes to download at once", th: "จำนวนตอนที่ดาวน์โหลดพร้อมกัน" },
  "settings.speedLimit": { en: "Speed Limit", th: "จำกัดความเร็ว" },
  "settings.speedLimitDesc": { en: "0 = Unlimited", th: "0 = ไม่จำกัด" },
  "settings.fileNaming": { en: "File Naming", th: "การตั้งชื่อไฟล์" },
  "settings.fileNamingDesc": { en: "Format for episode files", th: "รูปแบบชื่อไฟล์" },
  "settings.autoMerge": { en: "Auto Merge", th: "รวมอัตโนมัติ" },
  "settings.autoMergeDesc": { en: "Merge videos after download", th: "รวมวิดีโอหลังดาวน์โหลด" },
  "settings.deleteAfterMerge": { en: "Delete After Merge", th: "ลบหลังรวม" },
  "settings.deleteAfterMergeDesc": { en: "Remove episode files after merging", th: "ลบไฟล์ตอนหลังรวม" },

  "settings.notifications": { en: "Notifications", th: "การแจ้งเตือน" },
  "settings.systemNotifications": { en: "System Notifications", th: "แจ้งเตือนระบบ" },
  "settings.systemNotificationsDesc": { en: "Show notification when download completes", th: "แสดงการแจ้งเตือนเมื่อดาวน์โหลดเสร็จ" },
  "settings.sound": { en: "Sound Alert", th: "เสียงแจ้งเตือน" },
  "settings.soundDesc": { en: "Play sound when done", th: "เล่นเสียงเมื่อเสร็จ" },

  "settings.appearance": { en: "Appearance", th: "หน้าตา" },
  "settings.theme.light": { en: "Light", th: "สว่าง" },
  "settings.theme.dark": { en: "Dark", th: "มืด" },
  "settings.theme.system": { en: "System", th: "ระบบ" },

  "settings.language": { en: "Language", th: "ภาษา" },
  "settings.languageDesc": { en: "Interface language", th: "ภาษาอินเทอร์เฟซ" },

  "settings.updates": { en: "Updates", th: "อัปเดต" },
  "settings.checkUpdates": { en: "Check for Updates", th: "ตรวจสอบอัปเดต" },
  "settings.currentVersion": { en: "Current version", th: "เวอร์ชันปัจจุบัน" },
  "settings.checking": { en: "Checking...", th: "กำลังตรวจสอบ..." },
  "settings.checkNow": { en: "Check Now", th: "ตรวจสอบ" },

  "settings.presets": { en: "Download Presets", th: "พรีเซ็ตดาวน์โหลด" },
  "settings.presetsDesc": { en: "Quick settings presets", th: "ตั้งค่าด่วน" },

  "settings.schedule": { en: "Speed Schedule", th: "ตารางความเร็ว" },
  "settings.scheduleDesc": { en: "Limit speed by time", th: "จำกัดความเร็วตามเวลา" },

  "settings.shortcuts": { en: "Keyboard Shortcuts", th: "ปุ่มลัด" },
  "settings.shortcutsDesc": { en: "Available shortcuts", th: "ปุ่มลัดที่ใช้ได้" },

  "settings.actions": { en: "Actions", th: "การดำเนินการ" },
  "settings.openFolder": { en: "Open Output Folder", th: "เปิดโฟลเดอร์" },
  "settings.reset": { en: "Reset Settings", th: "รีเซ็ตตั้งค่า" },

  // Mini mode
  "mini.title": { en: "Mini Mode", th: "โหมดเล็ก" },
  "mini.expand": { en: "Expand", th: "ขยาย" },

  // Files
  "files.title": { en: "Downloaded Files", th: "ไฟล์ที่ดาวน์โหลด" },
  "files.empty": { en: "No files found", th: "ไม่พบไฟล์" },
  "files.refresh": { en: "Refresh", th: "รีเฟรช" },
  "files.delete": { en: "Delete", th: "ลบ" },
  "files.play": { en: "Play", th: "เล่น" },

  // History
  "history.title": { en: "Download History", th: "ประวัติดาวน์โหลด" },
  "history.empty": { en: "No history", th: "ไม่มีประวัติ" },
  "history.clear": { en: "Clear History", th: "ล้างประวัติ" },
  "history.stats": { en: "Statistics", th: "สถิติ" },

  // Logs
  "logs.title": { en: "Application Logs", th: "บันทึกแอป" },
  "logs.clear": { en: "Clear Logs", th: "ล้างบันทึก" },

  // Status
  "status.ready": { en: "Ready", th: "พร้อม" },
  "status.fetching": { en: "Fetching...", th: "กำลังดึงข้อมูล..." },
  "status.downloading": { en: "Downloading...", th: "กำลังดาวน์โหลด..." },
  "status.paused": { en: "Paused", th: "หยุดชั่วคราว" },
  "status.completed": { en: "Completed", th: "เสร็จสิ้น" },
  "status.failed": { en: "Failed", th: "ล้มเหลว" },
  "status.merging": { en: "Merging...", th: "กำลังรวม..." },

  // Common
  "common.save": { en: "Save", th: "บันทึก" },
  "common.cancel": { en: "Cancel", th: "ยกเลิก" },
  "common.delete": { en: "Delete", th: "ลบ" },
  "common.edit": { en: "Edit", th: "แก้ไข" },
  "common.add": { en: "Add", th: "เพิ่ม" },
  "common.close": { en: "Close", th: "ปิด" },
  "common.confirm": { en: "Confirm", th: "ยืนยัน" },
  "common.loading": { en: "Loading...", th: "กำลังโหลด..." },
  "common.error": { en: "Error", th: "ข้อผิดพลาด" },
  "common.success": { en: "Success", th: "สำเร็จ" },
  "common.warning": { en: "Warning", th: "คำเตือน" },
  "common.info": { en: "Info", th: "ข้อมูล" },
};

interface I18nContextValue {
  language: Language;
  setLanguage: (lang: Language) => void;
  t: (key: string, params?: Record<string, string | number>) => string;
}

const I18nContext = createContext<I18nContextValue | null>(null);

export function I18nProvider({ children }: { children: ReactNode }) {
  const [language, setLanguageState] = useState<Language>(() => {
    try {
      const saved = localStorage.getItem("rongyok-language");
      return (saved as Language) || "en";
    } catch {
      return "en";
    }
  });

  const setLanguage = useCallback((lang: Language) => {
    setLanguageState(lang);
    localStorage.setItem("rongyok-language", lang);
  }, []);

  const t = useCallback(
    (key: string, params?: Record<string, string | number>): string => {
      const translation = translations[key];
      if (!translation) {
        console.warn(`Missing translation for key: ${key}`);
        return key;
      }

      let text = translation[language] || translation.en || key;

      if (params) {
        Object.entries(params).forEach(([paramKey, value]) => {
          text = text.replace(`{${paramKey}}`, String(value));
        });
      }

      return text;
    },
    [language]
  );

  return (
    <I18nContext.Provider value={{ language, setLanguage, t }}>
      {children}
    </I18nContext.Provider>
  );
}

export function useI18n() {
  const context = useContext(I18nContext);
  if (!context) {
    throw new Error("useI18n must be used within an I18nProvider");
  }
  return context;
}

export { translations };
export type { Language };
