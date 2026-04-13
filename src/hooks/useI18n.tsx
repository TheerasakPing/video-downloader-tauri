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
  "app.ffmpeg": { en: "FFmpeg", th: "FFmpeg" },

  // Tabs
  "tab.download": { en: "Download", th: "ดาวน์โหลด" },
  "tab.library": { en: "Library", th: "ห้องสมุด" },
  "tab.browse": { en: "Browse", th: "เรียกดู" },
  "tab.files": { en: "Files", th: "ไฟล์" },
  "tab.history": { en: "History", th: "ประวัติ" },
  "tab.settings": { en: "Settings", th: "ตั้งค่า" },
  "tab.logs": { en: "Logs", th: "บันทึก" },

  // Download Tab
  "download.url": { en: "Series URL", th: "URL ซีรีส์" },
  "download.urlPlaceholder": { en: "https://rongyok.com/watch/?series_id=XXX", th: "https://rongyok.com/watch/?series_id=XXX" },
  "download.outputDir": { en: "Output Directory", th: "โฟลเดอร์บันทึก" },
  "download.outputDirPlaceholder": { en: "~/Downloads/rongyok", th: "~/Downloads/rongyok" },
  "download.fetch": { en: "Fetch", th: "ดึงข้อมูล" },
  "download.paste": { en: "Paste", th: "วาง" },
  "download.browse": { en: "Browse", th: "เรียกดู" },
  "download.open": { en: "Open", th: "เปิด" },
  "download.start": { en: "Download", th: "ดาวน์โหลด" },
  "download.pause": { en: "Pause", th: "หยุดชั่วคราว" },
  "download.resume": { en: "Resume", th: "ดำเนินต่อ" },
  "download.cancel": { en: "Cancel", th: "ยกเลิก" },
  "download.autoMerge": { en: "Auto merge", th: "รวมอัตโนมัติ" },
  "download.dragDrop": { en: "Drag & drop URL here", th: "ลาก URL มาวางที่นี่" },
  "download.dropHere": { en: "Drop to add URL", th: "ปล่อยเพื่อเพิ่ม URL" },
  "download.urlAdded": { en: "URL added from drop", th: "เพิ่ม URL จากการลากวางแล้ว" },
  "download.smartQueue": { en: "SMART QUEUE", th: "คิวอัจฉริยะ" },
  "download.clipboardMonitor": { en: "Clipboard Monitor (Auto Capture)", th: "ตรวจสอบคลิปบอร์ด (จับภาพอัตโนมัติ)" },
  "download.startQueue": { en: "Start Processing Queue (Sequential)", th: "เริ่มประมวลผลคิว (ตามลำดับ)" },
  "download.pauseQueue": { en: "Pause Queue", th: "หยุดคิวชั่วคราว" },
  "download.showQueue": { en: "Show Queue List", th: "แสดงรายการคิว" },
  "download.hideQueue": { en: "Hide Queue List", th: "ซ่อนรายการคิว" },
  "download.clearQueue": { en: "Clear Queue", th: "ล้างคิว" },
  "download.clearQueueConfirm": { en: "Clear entire queue?", th: "ล้างคิวทั้งหมด?" },
  "download.running": { en: "RUNNING", th: "กำลังทำงาน" },
  "download.queueEmpty": { en: "Queue is empty", th: "คิวว่างเปล่า" },
  "download.detectedUrls": { en: "Detected {count} URLs - Switching to Batch Mode", th: "ตรวจพบ {count} URL - กำลังสลับไปโหมดชุด" },
  "download.readyToDownload": { en: "Ready to download {count} series", th: "พร้อมดาวน์โหลด {count} ซีรีส์" },
  "download.pleaseSelectEpisodes": { en: "Please select at least one episode", th: "กรุณาเลือกอย่างน้อยหนึ่งตอน" },
  "download.selectEpisodes": { en: "Download ({count})", th: "ดาวน์โหลด ({count})" },
  "download.currentSpeed": { en: "{speed} MB/s", th: "{speed} MB/s" },
  "download.episodeProgress": { en: "EP {episode}", th: "ตอน {episode}" },
  "download.overallProgress": { en: "Overall", th: "ทั้งหมด" },
  "download.merging": { en: "Merging...", th: "กำลังรวม..." },
  "download.merged": { en: "Merged:", th: "รวมแล้ว:" },
  "download.noEpisodeDownloading": { en: "No episode currently downloading", th: "ไม่มีตอนที่กำลังดาวน์โหลด" },

  // Episodes
  "episodes.title": { en: "Episodes", th: "ตอน" },
  "episodes.selectAll": { en: "Select All", th: "เลือกทั้งหมด" },
  "episodes.deselectAll": { en: "Deselect All", th: "ยกเลิกเลือก" },
  "episodes.selected": { en: "selected", th: "ที่เลือก" },
  "episodes.of": { en: "of", th: "จาก" },

  // Quality
  "quality.title": { en: "Quality", th: "คุณภาพ" },
  "quality.auto": { en: "Auto", th: "อัตโนมัติ" },
  "quality.best": { en: "Best", th: "ดีที่สุด" },

  // Library Tab
  "library.search": { en: "Search library...", th: "ค้นหาในห้องสมุด..." },
  "library.allSources": { en: "All sources", th: "ทุกแหล่ง" },
  "library.sortBy": { en: "Sort by", th: "เรียงตาม" },
  "library.dateAdded": { en: "Date Added", th: "วันที่เพิ่ม" },
  "library.title": { en: "Title", th: "ชื่อเรื่อง" },
  "library.progress": { en: "Progress", th: "ความคืบหน้า" },
  "library.source": { en: "Source", th: "แหล่ง" },
  "library.lastDownloaded": { en: "Last Downloaded", th: "ดาวน์โหลดครั้งล่าสุด" },
  "library.ascending": { en: "Ascending", th: "น้อยไปมาก" },
  "library.descending": { en: "Descending", th: "มากไปน้อย" },
  "library.favorites": { en: "★ Favorites", th: "★ รายการโปรด" },
  "library.allStatus": { en: "All status", th: "ทุกสถานะ" },
  "library.complete": { en: "Complete", th: "สมบูรณ์" },
  "library.inProgress": { en: "In Progress", th: "กำลังดำเนินการ" },
  "library.notStarted": { en: "Not Started", th: "ยังไม่เริ่ม" },
  "library.stats": { en: "Stats", th: "สถิติ" },
  "library.all": { en: "All", th: "ทั้งหมด" },
  "library.newTag": { en: "+ New Tag", th: "+ แท็กใหม่" },
  "library.tagName": { en: "Tag name:", th: "ชื่อแท็ก:" },
  "library.empty": { en: "No series in library yet", th: "ยังไม่มีซีรีส์ในห้องสมุด" },
  "library.emptyHint": { en: "Fetch a series to auto-add it here", th: "ดึงข้อมูลซีรีส์เพื่อเพิ่มที่นี่โดยอัตโนมัติ" },
  "library.imported": { en: "Imported {count} series", th: "นำเข้า {count} ซีรีส์แล้ว" },
  "library.exported": { en: "Library exported to {path}", th: "ส่งออกห้องสมุดไปยัง {path}" },
  "library.importFailed": { en: "Import failed: {error}", th: "การนำเข้าล้มเหลว: {error}" },
  "library.exportFailed": { en: "Export failed: {error}", th: "การส่งออกล้มเหลว: {error}" },
  "library.new": { en: "New", th: "ใหม่" },
  "library.toggleFavorite": { en: "Toggle favorite", th: "สลับรายการโปรด" },
  "library.watched": { en: "watched", th: "ดูแล้ว" },
  "library.filtersAndSorting": { en: "Filters and sorting", th: "ตัวกรองและการเรียง" },
  "library.removeFromFavorites": { en: "Remove from favorites", th: "ลบออกจากรายการโปรด" },
  "library.addToFavorites": { en: "Add to favorites", th: "เพิ่มในรายการโปรด" },

  // Library Detail
  "library.detail.episodes": { en: "Episodes", th: "ตอน" },
  "library.detail.downloaded": { en: "Downloaded", th: "ดาวน์โหลดแล้ว" },
  "library.detail.watch": { en: "Watch", th: "ดู" },
  "library.detail.download": { en: "Download", th: "ดาวน์โหลด" },
  "library.detail.source": { en: "Source:", th: "แหล่ง:" },
  "library.detail.added": { en: "Added:", th: "เพิ่มเมื่อ:" },
  "library.detail.tags": { en: "Tags", th: "แท็ก" },
  "library.detail.addTag": { en: "+ Add Tag", th: "+ เพิ่มแท็ก" },
  "library.detail.refetch": { en: "Refetch", th: "ดึงข้อมูลใหม่" },
  "library.detail.remove": { en: "Remove", th: "ลบออก" },
  "library.detail.confirmRemove": { en: "Remove '{title}' from library?", th: "ลบ '{title}' ออกจากห้องสมุด?" },
  "library.detail.goBack": { en: "Go back", th: "กลับ" },
  "library.detail.lastDownload": { en: "Last Downloaded", th: "ดาวน์โหลดครั้งล่าสุด" },
  "library.detail.filter": { en: "Filter", th: "กรอง" },
  "library.detail.filterAll": { en: "All", th: "ทั้งหมด" },
  "library.detail.filterWatched": { en: "Watched", th: "ดูแล้ว" },
  "library.detail.filterUnwatched": { en: "Unwatched", th: "ยังไม่ดู" },
  "library.detail.epPrefix": { en: "Ep", th: "ตอน" },
  "library.detail.markWatched": { en: "Mark as watched", th: "ทำเครื่องหมายว่าดูแล้ว" },
  "library.detail.markUnwatched": { en: "Mark as unwatched", th: "ทำเครื่องหมายว่ายังไม่ดู" },

  // Browse Tab
  "browse.searchPlaceholder": { en: "Search across all sites...", th: "ค้นหาทั้งหมด..." },
  "browse.search": { en: "Search or browse categories to discover content", th: "ค้นหาหรือเรียกดูหมวดหมู่เพื่อค้นหาเนื้อหา" },
  "browse.loadMore": { en: "Load more", th: "โหลดเพิ่ม" },
  "browse.goBack": { en: "Go back", th: "กลับ" },
  "browse.backToBrowse": { en: "Back to browse", th: "กลับไปเรียกดู" },
  "browse.source": { en: "Source", th: "แหล่ง" },
  "browse.episodes": { en: "Episodes", th: "ตอน" },
  "browse.loadSeries": { en: "Load Series", th: "โหลดซีรีส์" },
  "browse.loadingInfo": { en: "Loading series info...", th: "กำลังโหลดข้อมูลซีรีส์..." },
  "browse.couldNotLoad": { en: "Could not load series", th: "ไม่สามารถโหลดซีรีส์ได้" },
  "browse.retry": { en: "Retry", th: "ลองใหม่" },

  // Settings Tab
  "settings.download": { en: "Download Settings", th: "ตั้งค่าดาวน์โหลด" },
  "settings.concurrent": { en: "Concurrent Downloads", th: "ดาวน์โหลดพร้อมกัน" },
  "settings.concurrentDesc": { en: "Number of episodes to download at once", th: "จำนวนตอนที่ดาวน์โหลดพร้อมกัน" },
  "settings.speedLimit": { en: "Speed Limit", th: "จำกัดความเร็ว" },
  "settings.speedLimitDesc": { en: "0 = Unlimited", th: "0 = ไม่จำกัด" },
  "settings.speedLimitUnit": { en: "KB/s", th: "KB/s" },
  "settings.fileNaming": { en: "File Naming", th: "การตั้งชื่อไฟล์" },
  "settings.fileNamingDesc": { en: "Format for episode files", th: "รูปแบบชื่อไฟล์" },
  "settings.autoMerge": { en: "Auto Merge", th: "รวมอัตโนมัติ" },
  "settings.autoMergeDesc": { en: "Merge videos after download", th: "รวมวิดีโอหลังดาวน์โหลด" },
  "settings.groupBySource": { en: "Group by Website", th: "จัดกลุ่มตามเว็บ" },
  "settings.groupBySourceDesc": { en: "Save to subfolder by source", th: "บันทึกไว้ในโฟลเดอร์ย่อยตามแหล่ง" },
  "settings.deleteAfterMerge": { en: "Delete After Merge", th: "ลบหลังรวม" },
  "settings.deleteAfterMergeDesc": { en: "Remove episode files after merging", th: "ลบไฟล์ตอนหลังรวม" },
  "settings.titanServer": { en: "Titan Server (51cg)", th: "เซิร์ฟเวอร์ Titan (51cg)" },
  "settings.titanServerDesc": { en: "Domains (comma-separated, max 5)", th: "โดเมน (คั่นด้วยจุลภาค, สูงสุด 5)" },
  "settings.baanjeenServer": { en: "BaanJeen Server", th: "เซิร์ฟเวอร์ BaanJeen" },
  "settings.baanjeenServerDesc": { en: "Domain for BaanJeen videos", th: "โดเมนสำหรับวิดีโอ BaanJeen" },
  "settings.rongyokServer": { en: "Rongyok Server", th: "เซิร์ฟเวอร์ Rongyok" },
  "settings.rongyokServerDesc": { en: "Domain for Rongyok videos", th: "โดเมนสำหรับวิดีโอ Rongyok" },
  "settings.hsckServer": { en: "HSCK Server", th: "เซิร์ฟเวอร์ HSCK" },
  "settings.hsckServerDesc": { en: "Domain for HSCK videos", th: "โดเมนสำหรับวิดีโอ HSCK" },
  "settings.njavtvServer": { en: "NjavTV Server", th: "เซิร์ฟเวอร์ NjavTV" },
  "settings.njavtvServerDesc": { en: "Domain for NjavTV videos (Cloudflare)", th: "โดเมนสำหรับวิดีโอ NjavTV (Cloudflare)" },
  "settings.avkuyServer": { en: "Avkuy Server", th: "เซิร์ฟเวอร์ Avkuy" },
  "settings.avkuyServerDesc": { en: "Domain for Avkuy videos (Cloudflare)", th: "โดเมนสำหรับวิดีโอ Avkuy (Cloudflare)" },
  "settings.serverDomains": { en: "Server Domains", th: "โดเมนเซิร์ฟเวอร์" },
  "settings.notifications": { en: "Notifications", th: "การแจ้งเตือน" },
  "settings.systemNotifications": { en: "System Notifications", th: "แจ้งเตือนระบบ" },
  "settings.systemNotificationsDesc": { en: "Show notification when download completes", th: "แสดงการแจ้งเตือนเมื่อดาวน์โหลดเสร็จ" },
  "settings.sound": { en: "Sound Alert", th: "เสียงแจ้งเตือน" },
  "settings.soundDesc": { en: "Play sound when done", th: "เล่นเสียงเมื่อเสร็จ" },
  "settings.appearance": { en: "Appearance", th: "หน้าตา" },
  "settings.theme.light": { en: "Light", th: "สว่าง" },
  "settings.theme.dark": { en: "Dark", th: "มืด" },
  "settings.theme.system": { en: "System", th: "ระบบ" },
  "settings.theme.highContrast": { en: "High Contrast", th: "คอนทราสต์สูง" },
  "settings.language": { en: "Language", th: "ภาษา" },
  "settings.languageDesc": { en: "Interface language", th: "ภาษาอินเทอร์เฟซ" },
  "settings.colorTheme": { en: "Color Theme", th: "ธีมสี" },
  "settings.downloadSchedule": { en: "Download Schedule", th: "ตารางดาวน์โหลด" },
  "settings.enableSchedule": { en: "Enable Scheduling", th: "เปิดใช้ตารางเวลา" },
  "settings.enableScheduleDesc": { en: "Limit downloads to specific time window", th: "จำกัดดาวน์โหลดในช่วงเวลาที่กำหนด" },
  "settings.activeStart": { en: "Active Start", th: "เริ่มใช้งาน" },
  "settings.activeEnd": { en: "Active End", th: "สิ้นสุดการใช้งาน" },
  "settings.speedDuringActive": { en: "Speed During Active (KB/s)", th: "ความเร็วในช่วงใช้งาน (KB/s)" },
  "settings.speedOutsideActive": { en: "Speed Outside Active (KB/s)", th: "ความเร็วนอกช่วงใช้งาน (KB/s)" },
  "settings.autoPause": { en: "Auto Pause", th: "หยุดอัตโนมัติ" },
  "settings.autoPauseDesc": { en: "Pause outside window", th: "หยุดนอกช่วงเวลา" },
  "settings.autoResume": { en: "Auto Resume", th: "ดำเนินการอัตโนมัติ" },
  "settings.autoResumeDesc": { en: "Resume in window", th: "ดำเนินการในช่วงเวลา" },
  "settings.proxy": { en: "Proxy Configuration", th: "การตั้งค่าพร็อกซี" },
  "settings.proxyType": { en: "Proxy Type", th: "ประเภทพร็อกซี" },
  "settings.proxyTypeDesc": { en: "Network proxy for downloads", th: "พร็อกซีเครือข่ายสำหรับดาวน์โหลด" },
  "settings.proxyDirect": { en: "No Proxy (Direct)", th: "ไม่ใช้พร็อกซี (ตรง)" },
  "settings.proxyHttp": { en: "HTTP Proxy", th: "พร็อกซี HTTP" },
  "settings.proxySocks5": { en: "SOCKS5 Proxy", th: "พร็อกซี SOCKS5" },
  "settings.proxyHost": { en: "Host", th: "โฮสต์" },
  "settings.proxyPort": { en: "Port", th: "พอร์ต" },
  "settings.retry": { en: "Retry & Fallback", th: "ลองใหม่และสำรอง" },
  "settings.maxRetries": { en: "Max Retries", th: "ลองใหม่สูงสุด" },
  "settings.maxRetriesDesc": { en: "Attempts before marking as failed (0-10)", th: "จำนวนครั้งก่อนจะถือว่าล้มเหลว (0-10)" },
  "settings.retryDelay": { en: "Retry Delay", th: "หน่วงเวลาลองใหม่" },
  "settings.retryDelayDesc": { en: "Delay between retry attempts (ms)", th: "หน่วงเวลาระหว่างการลองใหม่ (ms)" },
  "settings.autoRetry": { en: "Auto-retry on failure", th: "ลองใหม่อัตโนมัติเมื่อล้มเหลว" },
  "settings.autoRetryDesc": { en: "Automatically retry failed downloads", th: "ลองดาวน์โหลดที่ล้มเหลวใหม่อัตโนมัติ" },
  "settings.skipFailed": { en: "Skip failed segments", th: "ข้ามส่วนที่ล้มเหลว" },
  "settings.skipFailedDesc": { en: "Skip unrecoverable HLS segments", th: "ข้ามส่วน HLS ที่กู้คืนไม่ได้" },
  "settings.updates": { en: "Updates", th: "อัปเดต" },
  "settings.checkUpdates": { en: "Check for Updates", th: "ตรวจสอบอัปเดต" },
  "settings.currentVersion": { en: "Current version: {version}", th: "เวอร์ชันปัจจุบัน: {version}" },
  "settings.checking": { en: "Checking...", th: "กำลังตรวจสอบ..." },
  "settings.checkNow": { en: "Check Now", th: "ตรวจสอบ" },
  "settings.webhooks": { en: "Webhook Notifications", th: "การแจ้งเตือนเว็บฮุก" },
  "settings.webhooksDesc": { en: "Configure webhook notifications for downloads and updates", th: "ตั้งค่าการแจ้งเตือนเว็บฮุกสำหรับดาวน์โหลดและอัปเดต" },
  "settings.configureWebhooks": { en: "Configure Webhooks", th: "ตั้งค่าเว็บฮุก" },
  "settings.openFolder": { en: "Open Output Folder", th: "เปิดโฟลเดอร์" },
  "settings.resetSettings": { en: "Reset Settings", th: "รีเซ็ตตั้งค่า" },
  "settings.downloadSettings": { en: "Download Settings", th: "ตั้งค่าดาวน์โหลด" },
  "settings.concurrentDownloads": { en: "Concurrent Downloads", th: "ดาวน์โหลดพร้อมกัน" },
  "settings.kbs": { en: "KB/s", th: "KB/s" },
  "settings.soundAlert": { en: "Sound Alert", th: "เสียงแจ้งเตือน" },
  "settings.light": { en: "Light", th: "สว่าง" },
  "settings.dark": { en: "Dark", th: "มืด" },
  "settings.system": { en: "System", th: "ระบบ" },
  "settings.enableScheduling": { en: "Enable Scheduling", th: "เปิดใช้งานตาราง" },
  "settings.proxyConfiguration": { en: "Proxy Configuration", th: "ตั้งค่าพร็อกซี" },
  "settings.noProxy": { en: "No Proxy (Direct)", th: "ไม่ใช้พร็อกซี (ตรง)" },
  "settings.httpProxy": { en: "HTTP Proxy", th: "พร็อกซี HTTP" },
  "settings.socks5Proxy": { en: "SOCKS5 Proxy", th: "พร็อกซี SOCKS5" },
  "settings.retryFallback": { en: "Retry & Fallback", th: "ลองใหม่และสำรอง" },
  "settings.checkForUpdates": { en: "Check for Updates", th: "ตรวจสอบการอัปเดต" },
  "settings.webhookNotifications": { en: "Webhook Notifications", th: "แจ้งเตือนผ่านเว็บฮุก" },
  "settings.openOutputFolder": { en: "Open Output Folder", th: "เปิดโฟลเดอร์ผลลัพธ์" },
  "settings.fileNamingEp001": { en: "ep_001.mp4", th: "ep_001.mp4" },
  "settings.fileNamingEpisode1": { en: "episode_1.mp4", th: "episode_1.mp4" },
  "settings.fileNamingTitleEp1": { en: "Title_EP1.mp4", th: "Title_EP1.mp4" },

  // Scheduler
  "scheduler.title": { en: "Download Scheduler", th: "ตัวจัดตารางดาวน์โหลด" },
  "scheduler.addSchedule": { en: "Add Schedule", th: "เพิ่มตาราง" },
  "scheduler.cancel": { en: "Cancel", th: "ยกเลิก" },
  "scheduler.name": { en: "Schedule Name", th: "ชื่อตาราง" },
  "scheduler.namePlaceholder": { en: "e.g., Daily Series Download", th: "เช่น ดาวน์โหลดซีรีส์ประจำวัน" },
  "scheduler.url": { en: "Video URL", th: "URL วิดีโอ" },
  "scheduler.urlPlaceholder": { en: "https://example.com/series", th: "https://example.com/series" },
  "scheduler.outputDir": { en: "Output Directory", th: "โฟลเดอร์บันทึก" },
  "scheduler.browseFolder": { en: "Browse folder", th: "เรียกดูโฟลเดอร์" },
  "scheduler.schedule": { en: "Schedule", th: "ตารางเวลา" },
  "scheduler.customSchedule": { en: 'Or use custom: "daily HH:MM", "weekly N HH:MM" (0=Sun), or "hourly"', th: 'หรือใช้แบบกำหนดเอง: "daily HH:MM", "weekly N HH:MM" (0=อา.), หรือ "hourly"' },
  "scheduler.create": { en: "Create Schedule", th: "สร้างตาราง" },
  "scheduler.paused": { en: "Paused", th: "หยุดชั่วคราว" },
  "scheduler.enable": { en: "Enable schedule", th: "เปิดใช้ตาราง" },
  "scheduler.disable": { en: "Disable schedule", th: "ปิดใช้ตาราง" },
  "scheduler.delete": { en: "Delete schedule", th: "ลบตาราง" },
  "scheduler.deleteConfirm": { en: "Delete schedule \"{name}\"?", th: "ลบตาราง \"{name}\"?" },
  "scheduler.lastRun": { en: "Last run:", th: "ทำงานครั้งล่าสุด:" },
  "scheduler.nextRun": { en: "Next run:", th: "ทำงานครั้งต่อไป:" },
  "scheduler.never": { en: "Never", th: "ไม่เคย" },
  "scheduler.noSchedules": { en: "No scheduled downloads", th: "ไม่มีการดาวน์โหลดตามตาราง" },
  "scheduler.noSchedulesHint": { en: "Create a schedule to automatically download content", th: "สร้างตารางเพื่อดาวน์โหลดเนื้อหาอัตโนมัติ" },
  "scheduler.daily": { en: "Daily at {time}", th: "ทุกวันเวลา {time}" },
  "scheduler.weekly": { en: "Weekly on {day} at {time}", th: "ทุก{day} เวลา {time}" },
  "scheduler.hourly": { en: "Hourly", th: "ทุกชั่วโมง" },
  "scheduler.sunday": { en: "Sunday", th: "อาทิตย์" },
  "scheduler.monday": { en: "Monday", th: "จันทร์" },
  "scheduler.tuesday": { en: "Tuesday", th: "อังคาร" },
  "scheduler.wednesday": { en: "Wednesday", th: "พุธ" },
  "scheduler.thursday": { en: "Thursday", th: "พฤหัสบดี" },
  "scheduler.friday": { en: "Friday", th: "ศุกร์" },
  "scheduler.saturday": { en: "Saturday", th: "เสาร์" },

  // Backup & Data
  "backup.title": { en: "Data & Backup", th: "ข้อมูลและสำรอง" },
  "backup.create": { en: "Create Backup", th: "สร้างข้อมูลสำรอง" },
  "backup.creating": { en: "Creating...", th: "กำลังสร้าง..." },
  "backup.restore": { en: "Restore Backup", th: "คืนค่าข้อมูลสำรอง" },
  "backup.restoring": { en: "Restoring...", th: "กำลังคืนค่า..." },
  "backup.created": { en: "Backup saved to {path}", th: "บันทึกข้อมูลสำรองที่ {path}" },
  "backup.createFailed": { en: "Backup failed: {error}", th: "การสร้างข้อมูลสำรองล้มเหลว: {error}" },
  "backup.restoreFailed": { en: "Restore failed: {error}", th: "การคืนค่าล้มเหลว: {error}" },
  "backup.restored": { en: "Restored {count} rows from backup", th: "คืนค่า {count} รายการจากข้อมูลสำรอง" },
  "backup.restoreConfirm": { en: "This will overwrite existing data. Are you sure?", th: "นี่จะเขียนทับข้อมูลที่มีอยู่ คุณแน่ใจหรือไม่?" },
  "backup.findDuplicates": { en: "Find Duplicates", th: "ค้นหาซ้ำ" },
  "backup.findDuplicatesDesc": { en: "Detect series with similar titles from different sources", th: "ตรวจจับซีรีส์ที่มีชื่อคล้ายกันจากแหล่งต่างกัน" },
  "backup.scan": { en: "Scan", th: "สแกน" },
  "backup.scanning": { en: "Scanning...", th: "กำลังสแกน..." },
  "backup.noDuplicates": { en: "No duplicates found", th: "ไม่พบรายการซ้ำ" },
  "backup.foundDuplicates": { en: "Found {count} duplicate group(s)", th: "พบ {count} กลุ่มที่ซ้ำกัน" },
  "backup.keep": { en: "Keep", th: "เก็บ" },
  "backup.duplicates": { en: "Duplicates:", th: "รายการซ้ำ:" },
  "backup.removeEntry": { en: "Remove this entry from library?", th: "ลบรายการนี้ออกจากห้องสมุด?" },
  "backup.entryRemoved": { en: "Entry removed", th: "ลบรายการออกแล้ว" },
  "backup.removeFailed": { en: "Failed to remove: {error}", th: "ลบไม่สำเร็จ: {error}" },
  "backup.clickToScan": { en: "Click \"Scan\" to find duplicate entries", th: "คลิก \"สแกน\" เพื่อค้นหารายการซ้ำ" },

  // Webhooks
  "webhook.title": { en: "Webhook Notifications", th: "การแจ้งเตือนเว็บฮุก" },
  "webhook.subtitle": { en: "Get notified about downloads and updates", th: "รับการแจ้งเตือนเกี่ยวกับดาวน์โหลดและอัปเดต" },
  "webhook.enable": { en: "Enable Webhooks", th: "เปิดใช้เว็บฮุก" },
  "webhook.enableDesc": { en: "Send notifications to configured endpoints", th: "ส่งการแจ้งเตือนไปยังปลายทางที่ตั้งค่า" },
  "webhook.type": { en: "Webhook Type", th: "ประเภทเว็บฮุก" },
  "webhook.url": { en: "Webhook URL", th: "URL เว็บฮุก" },
  "webhook.urlPlaceholder": { en: "https://your-webhook-url.com/endpoint", th: "https://your-webhook-url.com/endpoint" },
  "webhook.discordUrl": { en: "https://discord.com/api/webhooks/...", th: "https://discord.com/api/webhooks/..." },
  "webhook.lineNoUrl": { en: "Not required for LINE Notify", th: "ไม่จำเป็นสำหรับ LINE Notify" },
  "webhook.secret": { en: "Authorization Secret (Optional)", th: "ข้อมูลลับการยืนยัน (ไม่บังคับ)" },
  "webhook.lineToken": { en: "LINE Notify Token", th: "โทเค็น LINE Notify" },
  "webhook.lineTokenPlaceholder": { en: "Your LINE Notify token", th: "โทเค็น LINE Notify ของคุณ" },
  "webhook.bearerPlaceholder": { en: "Bearer token (optional)", th: "โทเค็น Bearer (ไม่บังคับ)" },
  "webhook.events": { en: "Events to Notify", th: "เหตุการณ์ที่จะแจ้งเตือน" },
  "webhook.eventDownloadComplete": { en: "Download Complete", th: "ดาวน์โหลดเสร็จสิ้น" },
  "webhook.eventDownloadFailed": { en: "Download Failed", th: "ดาวน์โหลดล้มเหลว" },
  "webhook.eventNewEpisode": { en: "New Episode Detected", th: "พบตอนใหม่" },
  "webhook.save": { en: "Save Configuration", th: "บันทึกการตั้งค่า" },
  "webhook.test": { en: "Test", th: "ทดสอบ" },
  "webhook.checkEpisodes": { en: "Check Episodes", th: "ตรวจสอบตอน" },
  "webhook.saved": { en: "Webhook configuration saved successfully!", th: "บันทึกการตั้งค่าเว็บฮุกสำเร็จ!" },
  "webhook.noNewEpisodes": { en: "No new episodes found. All series are up to date!", th: "ไม่พบตอนใหม่ ซีรีส์ทั้งหมดเป็นปัจจุบันแล้ว!" },
  "webhook.foundNewEpisodes": { en: "Found {count} series with new episodes:\n{series}", th: "พบ {count} ซีรีส์ที่มีตอนใหม่:\n{series}" },
  "webhook.setupInstructions": { en: "Setup Instructions", th: "คำแนะนำการติดตั้ง" },
  "webhook.discordSetup1": { en: "• Create a Discord webhook in your server settings", th: "• สร้างเว็บฮุก Discord ในการตั้งค่าเซิร์ฟเวอร์" },
  "webhook.discordSetup2": { en: "• Paste the webhook URL above", th: "• วาง URL เว็บฮุกด้านบน" },
  "webhook.discordSetup3": { en: "• Select which events to trigger notifications", th: "• เลือกเหตุการณ์ที่จะให้แจ้งเตือน" },
  "webhook.lineSetup1": { en: "• Generate a LINE Notify token at notify.bot.line.me", th: "• สร้างโทเค็น LINE Notify ที่ notify.bot.line.me" },
  "webhook.lineSetup2": { en: "• Paste your token in the secret field", th: "• วางโทเค็นของคุณในช่องข้อมูลลับ" },
  "webhook.lineSetup3": { en: "• URL field can be left empty for LINE Notify", th: "• ช่อง URL สามารถปล่อยว่างได้สำหรับ LINE Notify" },
  "webhook.customSetup1": { en: "• Enter your custom webhook endpoint URL", th: "• ป้อน URL ปลายทางเว็บฮุกที่กำหนดเอง" },
  "webhook.customSetup2": { en: "• Optional: Add Bearer token for authentication", th: "• ไม่บังคับ: เพิ่มโทเค็น Bearer สำหรับการยืนยัน" },
  "webhook.customSetup3": { en: "• Payload format: { event, title, message, timestamp }", th: "• รูปแบบ payload: { event, title, message, timestamp }" },

  // Import/Export
  "importExport.export": { en: "Export", th: "ส่งออก" },
  "importExport.import": { en: "Import", th: "นำเข้า" },
  "importExport.exportTitle": { en: "Export Library", th: "ส่งออกห้องสมุด" },
  "importExport.importTitle": { en: "Import Library", th: "นำเข้าห้องสมุด" },
  "importExport.exportHint": { en: "Export library to JSON", th: "ส่งออกห้องสมุดเป็น JSON" },
  "importExport.importHint": { en: "Import library from JSON", th: "นำเข้าห้องสมุดจาก JSON" },

  // Mini mode
  "mini.title": { en: "Mini Mode", th: "โหมดเล็ก" },
  "mini.expand": { en: "Expand", th: "ขยาย" },

  // Files Tab
  "files.title": { en: "Downloaded Files", th: "ไฟล์ที่ดาวน์โหลด" },
  "files.empty": { en: "No files in output directory", th: "ไม่มีไฟล์ในโฟลเดอร์ปลายทาง" },
  "files.refresh": { en: "Refresh", th: "รีเฟรช" },
  "files.openFolder": { en: "Open Folder", th: "เปิดโฟลเดอร์" },
  "files.delete": { en: "Delete", th: "ลบ" },
  "files.play": { en: "Play", th: "เล่น" },
  "files.select": { en: "Select", th: "เลือก" },
  "files.selectAll": { en: "Select All", th: "เลือกทั้งหมด" },
  "files.deselect": { en: "Deselect", th: "ยกเลิกเลือก" },
  "files.deleteSelected": { en: "Delete ({count})", th: "ลบ ({count})" },
  "files.episodes": { en: "Episodes", th: "ตอน" },
  "files.merged": { en: "Merged", th: "รวมแล้ว" },
  "files.totalSize": { en: "Total Size", th: "ขนาดรวม" },
  "files.count": { en: "{count} files • {size}", th: "{count} ไฟล์ • {size}" },

  // History Tab
  "history.title": { en: "Download History", th: "ประวัติดาวน์โหลด" },
  "history.empty": { en: "No download history yet", th: "ยังไม่มีประวัติการดาวน์โหลด" },
  "history.clear": { en: "Clear All", th: "ล้างทั้งหมด" },
  "history.totalDownloads": { en: "Total Downloads", th: "ดาวน์โหลดทั้งหมด" },
  "history.episodes": { en: "Episodes", th: "ตอน" },
  "history.totalSize": { en: "Total Size", th: "ขนาดรวม" },
  "history.successRate": { en: "Success Rate", th: "อัตราสำเร็จ" },
  "history.downloadHistory": { en: "Download History", th: "ประวัติการดาวน์โหลด" },
  "history.episodesCount": { en: "{completed}/{total} episodes", th: "{completed}/{total} ตอน" },
  "history.seriesId": { en: "ID:", th: "ID:" },

  // Logs Tab
  "logs.title": { en: "Application Logs", th: "บันทึกแอป" },
  "logs.clear": { en: "Clear Logs", th: "ล้างบันทึก" },
  "logs.debug": { en: "Debug Log", th: "บันทึกดีบัก" },
  "logs.copy": { en: "Copy logs", th: "คัดลอกบันทึก" },
  "logs.noLogs": { en: "No logs yet", th: "ยังไม่มีบันทึก" },

  // Queue
  "queue.title": { en: "Download Queue", th: "คิวดาวน์โหลด" },
  "queue.pending": { en: "Pending", th: "รอดำเนินการ" },
  "queue.active": { en: "Active", th: "กำลังดำเนินการ" },
  "queue.done": { en: "Done", th: "เสร็จสิ้น" },
  "queue.empty": { en: "Download queue is empty", th: "คิวดาวน์โหลดว่างเปล่า" },
  "queue.downloadingProgress": { en: "Downloading {progress}%", th: "กำลังดาวน์โหลด {progress}%" },
  "queue.moveUp": { en: "Move up in queue", th: "เลื่อนขึ้นในคิว" },
  "queue.moveDown": { en: "Move down in queue", th: "เลื่อนลงในคิว" },
  "queue.removeFromQueue": { en: "Remove from queue", th: "ลบออกจากคิว" },
  "queue.pauseDownload": { en: "Pause download", th: "หยุดดาวน์โหลดชั่วคราว" },

  // Notifications
  "notifications.title": { en: "Notifications", th: "การแจ้งเตือน" },
  "notifications.markAllRead": { en: "Mark all read", th: "ทำเครื่องหมายทั้งหมดว่าอ่านแล้ว" },
  "notifications.clearAll": { en: "Clear all", th: "ล้างทั้งหมด" },
  "notifications.clearConfirm": { en: "Clear all notifications older than 30 days?", th: "ล้างการแจ้งเตือนที่เก่ากว่า 30 วัน?" },
  "notifications.noNotifications": { en: "No notifications yet", th: "ยังไม่มีการแจ้งเตือน" },
  "notifications.unread": { en: "{count} unread notification{plural}", th: "{count} การแจ้งเตือนที่ยังไม่ได้อ่าน" },
  "notifications.justNow": { en: "Just now", th: "เมื่อสักครู่" },
  "notifications.minutesAgo": { en: "{m}m ago", th: "{m} นาทีที่แล้ว" },
  "notifications.hoursAgo": { en: "{h}h ago", th: "{h} ชั่วโมงที่แล้ว" },
  "notifications.daysAgo": { en: "{d}d ago", th: "{d} วันที่แล้ว" },

  // Shortcuts
  "shortcuts.title": { en: "Keyboard Shortcuts", th: "ปุ่มลัด" },
  "shortcuts.close": { en: "Close", th: "ปิด" },

  // Status
  "status.ready": { en: "Ready", th: "พร้อม" },
  "status.fetching": { en: "Fetching...", th: "กำลังดึงข้อมูล..." },
  "status.downloading": { en: "Downloading...", th: "กำลังดาวน์โหลด..." },
  "status.paused": { en: "Paused", th: "หยุดชั่วคราว" },
  "status.completed": { en: "Completed", th: "เสร็จสิ้น" },
  "status.failed": { en: "Failed", th: "ล้มเหลว" },
  "status.merging": { en: "Merging...", th: "กำลังรวม..." },
  "status.error": { en: "Error", th: "ข้อผิดพลาด" },

  // Toast Messages
  "toast.clipboardEmpty": { en: "Clipboard is empty", th: "คลิปบอร์ดว่างเปล่า" },
  "toast.failedToReadClipboard": { en: "Failed to read clipboard", th: "อ่านคลิปบอร์ดไม่สำเร็จ" },
  "toast.enterUrl": { en: "Please enter a URL", th: "กรุณาป้อน URL" },
  "toast.failedToFetch": { en: "Failed to fetch: {error}", th: "ดึงข้อมูลไม่สำเร็จ: {error}" },
  "toast.loaded": { en: "Loaded: {title} ({episodes} episodes)", th: "โหลดแล้ว: {title} ({episodes} ตอน)" },
  "toast.cachedUrls": { en: "Cached {count} video URLs", th: "แคช URL วิดีโอ {count} รายการ" },
  "toast.autoCaptured": { en: "Auto-captured {count} links", th: "จับภาพ {count} ลิงก์โดยอัตโนมัติ" },
  "toast.addedToQueue": { en: "Added {count} links to queue", th: "เพิ่ม {count} ลิงก์ไปยังคิว" },
  "toast.downloadComplete": { en: "All {count} episodes of {title} downloaded!", th: "ดาวน์โหลด {count} ตอนของ {title} สำเร็จ!" },
  "toast.downloadPartial": { en: "Downloaded {completed}/{total} episodes ({failed} failed)", th: "ดาวน์โหลด {completed}/{total} ตอน ({failed} ล้มเหลว)" },
  "toast.downloadFailed": { en: "Download failed: {error}", th: "ดาวน์โหลดล้มเหลว: {error}" },
  "toast.cancelled": { en: "Cancelled download", th: "ยกเลิกการดาวน์โหลด" },
  "toast.failedToCancel": { en: "Failed to cancel: {error}", th: "ยกเลิกไม่สำเร็จ: {error}" },
  "toast.ffprobeAvailable": { en: "FFmpeg is available", th: "มี FFmpeg อยู่" },
  "toast.ffprobeNotFound": { en: "FFmpeg not found - video merging will be disabled", th: "ไม่พบ FFmpeg - การรวมวิดีโอจะถูกปิดใช้งาน" },
  "toast.couldNotCheckFfmpeg": { en: "Could not check FFmpeg status", th: "ไม่สามารถตรวจสอบสถานะ FFmpeg" },
  "toast.outputFolderSet": { en: "Output folder set to: {path}", th: "ตั้งค่าโฟลเดอร์ปลายทางเป็น: {path}" },
  "toast.couldNotSelectFolder": { en: "Could not select folder", th: "ไม่สามารถเลือกโฟลเดอร์" },
  "toast.couldNotOpenFolder": { en: "Could not open folder", th: "ไม่สามารถเปิดโฟลเดอร์" },
  "toast.deletedFiles": { en: "Deleted {count} file{plural}", th: "ลบ {count} ไฟล์" },
  "toast.failedToDelete": { en: "Failed to delete files", th: "ลบไฟล์ไม่สำเร็จ" },
  "toast.couldNotPlayFile": { en: "Could not open file", th: "ไม่สามารถเปิดไฟล์" },
  "toast.queueReady": { en: "Queue: Ready - {title}", th: "คิว: พร้อม - {title}" },
  "toast.queueFetchFailed": { en: "Queue Fetch failed for {url}: {error}", th: "คิว: ดึงข้อมูล {url} ไม่สำเร็จ: {error}" },
  "toast.mergeComplete": { en: "Merged to: {path}", th: "รวมไปที่: {path}" },
  "toast.mergeFailed": { en: "Merge failed: {error}", th: "การรวมล้มเหลว: {error}" },

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
  "common.of": { en: "of", th: "จาก" },
  "common.yes": { en: "Yes", th: "ใช่" },
  "common.no": { en: "No", th: "ไม่" },

  // Mini Mode (extended)
  "mini.downloading": { en: "Downloading...", th: "กำลังดาวน์โหลด..." },
  "mini.close": { en: "Close", th: "ปิด" },
  "mini.episode": { en: "Episode", th: "ตอน" },
  "mini.overall": { en: "Overall", th: "ทั้งหมด" },
  "mini.resume": { en: "Resume", th: "ดำเนินต่อ" },
  "mini.pause": { en: "Pause", th: "หยุดชั่วคราว" },
  "mini.expandTitle": { en: "Expand", th: "ขยาย" },
  "mini.closeTitle": { en: "Close", th: "ปิด" },

  // Episodes (extended)
  "episodes.selectAllAria": { en: "Select all episodes", th: "เลือกตอนทั้งหมด" },
  "episodes.all": { en: "All", th: "ทั้งหมด" },
  "episodes.deselectAllAria": { en: "Deselect all episodes", th: "ยกเลิกเลือกตอนทั้งหมด" },
  "episodes.none": { en: "None", th: "ไม่เลือก" },
  "episodes.episodeTitle": { en: "Episode {number}", th: "ตอน {number}" },

  // Queue (extended)
  "queue.failed": { en: "Failed", th: "ล้มเหลว" },
  "queue.paused": { en: "Paused", th: "หยุดชั่วคราว" },
  "queue.episode": { en: "Episode", th: "ตอน" },

  // Preset
  "preset.title": { en: "Quick Presets", th: "พรีเซ็ตด่วน" },
  "preset.selectAria": { en: "Select {name} preset", th: "เลือกพรีเซ็ต {name}" },

  // Settings (extended aria labels)
  "settings.lightThemeAria": { en: "Light theme", th: "ธีมสว่าง" },
  "settings.darkThemeAria": { en: "Dark theme", th: "ธีมมืด" },
  "settings.systemThemeAria": { en: "System theme", th: "ธีมระบบ" },
  "settings.englishLangAria": { en: "English language", th: "ภาษาอังกฤษ" },
  "settings.thaiLangAria": { en: "Thai language", th: "ภาษาไทย" },
  "settings.proxyHostPlaceholder": { en: "127.0.0.1", th: "127.0.0.1" },
  "settings.proxyPortPlaceholder": { en: "8080", th: "8080" },
  "settings.titanServerPlaceholder": { en: "51cg1.com, 51cm.com", th: "51cg1.com, 51cm.com" },
  "settings.baanjeenPlaceholder": { en: "xn--82c7abb4jua0l.com", th: "xn--82c7abb4jua0l.com" },
  "settings.rongyokPlaceholder": { en: "rongyok.com", th: "rongyok.com" },
  "settings.hsckPlaceholder": { en: "hsck123.com", th: "hsck123.com" },
  "settings.njavtvPlaceholder": { en: "njavtv.com", th: "njavtv.com" },

  // History (extended)
  "history.deleteRecord": { en: "Delete history record", th: "ลบรายการประวัติ" },
  "history.id": { en: "ID:", th: "ID:" },
  "history.episodesSuffix": { en: " episodes", th: " ตอน" },

  // Logs (extended)
  "logs.clearLogs": { en: "Clear logs", th: "ล้างบันทึก" },
  "logs.copyLogs": { en: "Copy logs", th: "คัดลอกบันทึก" },

  // Shortcuts
  "shortcuts.help": { en: "Help", th: "ช่วยเหลือ" },
  "shortcuts.shortcuts": { en: "Shortcuts", th: "ปุ่มลัด" },
  "shortcuts.gettingStarted": { en: "Getting Started", th: "เริ่มต้นใช้งาน" },
  "shortcuts.closeShortcuts": { en: "Close keyboard shortcuts", th: "ปิดปุ่มลัด" },
  "shortcuts.pasteUrl": { en: "Paste URL from clipboard", th: "วาง URL จากคลิปบอร์ด" },
  "shortcuts.pasteUrlDesc": { en: "Press Ctrl+V to paste a video link from clipboard, or type URL in search box", th: "กด Ctrl+V เพื่อวางลิงก์วิดีโอจากคลิปบอร์ด หรือพิมพ์ URL ในช่องค้นหา" },
  "shortcuts.pasteUrlShortcut": { en: "Ctrl + V", th: "Ctrl + V" },
  "shortcuts.selectEpisodes": { en: "Select episodes", th: "เลือกตอน" },
  "shortcuts.selectEpisodesDesc": { en: "Choose the episodes you want to download, or select all episodes", th: "เลือกตอนที่ต้องการดาวน์โหลด หรือเลือกตอนทั้งหมด" },
  "shortcuts.selectEpisodesShortcut": { en: "Click / Ctrl+A", th: "คลิก / Ctrl+A" },
  "shortcuts.startDownload": { en: "Start download", th: "เริ่มดาวน์โหลด" },
  "shortcuts.startDownloadDesc": { en: "Click Download button or use Ctrl+D shortcut to start downloading", th: "คลิกปุ่มดาวน์โหลดหรือใช้ปุ่มลัด Ctrl+D เพื่อเริ่มดาวน์โหลด" },
  "shortcuts.startDownloadShortcut": { en: "Ctrl + D", th: "Ctrl + D" },
  "shortcuts.manageQueue": { en: "Manage queue", th: "จัดการคิว" },
  "shortcuts.manageQueueDesc": { en: "Use Smart Queue to download multiple series sequentially", th: "ใช้คิวอัจฉริยะเพื่อดาวน์โหลดหลายซีรีส์ตามลำดับ" },
  "shortcuts.manageQueueShortcut": { en: "Queue Panel", th: "แผงคิว" },
  "shortcuts.addToLibrary": { en: "Add to library", th: "เพิ่มลงห้องสมุด" },
  "shortcuts.addToLibraryDesc": { en: "Series are auto-added to library. Manage tags and favorites", th: "ซีรีส์จะถูกเพิ่มลงห้องสมุดอัตโนมัติ จัดการแท็กและรายการโปรด" },
  "shortcuts.addToLibraryShortcut": { en: "Library Tab", th: "แท็บห้องสมุด" },
  "shortcuts.tip": { en: "Tip: Enable Clipboard Monitor to copy multiple links and auto-download", th: "เคล็ดลับ: เปิดใช้ตรวจสอบคลิปบอร์ดเพื่อคัดลอกหลายลิงก์และดาวน์โหลดอัตโนมัติ" },
  "shortcuts.all": { en: "All", th: "ทั้งหมด" },

  // Update Dialog
  "update.title": { en: "Update Available!", th: "มีอัปเดตใหม่!" },
  "update.close": { en: "Close update dialog", th: "ปิดกล่องอัปเดต" },
  "update.whatsNew": { en: "What's New:", th: "มีอะไรใหม่:" },
  "update.downloading": { en: "Downloading update...", th: "กำลังดาวน์โหลดอัปเดต..." },
  "update.failed": { en: "Update Failed", th: "อัปเดตล้มเหลว" },
  "update.openGithub": { en: "Open GitHub Releases", th: "เปิด GitHub Releases" },
  "update.later": { en: "Later", th: "ทีหลัง" },
  "update.updating": { en: "Updating...", th: "กำลังอัปเดต..." },
  "update.retryUpdate": { en: "Retry Update", th: "ลองอัปเดตอีกครั้ง" },
  "update.updateNow": { en: "Update Now", th: "อัปเดตเลย" },

  // Theme (extended)
  "theme.selectAria": { en: "Select {name} theme", th: "เลือกธีม {name}" },

  // App (extended)
  "app.dropUrl": { en: "Drop URL here", th: "วาง URL ที่นี่" },
  "app.openMiniMode": { en: "Open mini mode", th: "เปิดโหมดเล็ก" },
  "app.openShortcuts": { en: "Open keyboard shortcuts help", th: "เปิดความช่วยเหลือปุ่มลัด" },
  "app.smartQueueLabel": { en: "SMART QUEUE", th: "คิวอัจฉริยะ" },
  "app.runningLabel": { en: "RUNNING", th: "กำลังทำงาน" },
  "app.on": { en: "ON", th: "เปิด" },
  "app.auto": { en: "AUTO", th: "อัตโนมัติ" },
  "app.removeFromBatch": { en: "Remove from batch queue", th: "ลบออกจากคิวชุด" },
  "app.selectOutputFolder": { en: "Select Output Folder", th: "เลือกโฟลเดอร์ปลายทาง" },
  "app.mergeCompleteTitle": { en: "Merge Complete", th: "รวมเสร็จสิ้น" },
  "app.mergeCompleteBody": { en: "Videos merged successfully!", th: "รวมวิดีโอสำเร็จ!" },
  "app.pausedDownload": { en: "Paused download", th: "หยุดดาวน์โหลดชั่วคราวแล้ว" },
  "app.pauseCompleted": { en: "Pause completed (download may have finished)", th: "หยุดชั่วคราวเสร็จแล้ว (ดาวน์โหลดอาจเสร็จแล้ว)" },
  "app.resumedDownload": { en: "Resumed download", th: "ดำเนินการดาวน์โหลดต่อแล้ว" },
  "app.resumeCompleted": { en: "Resume completed (download may have finished)", th: "ดำเนินการต่อเสร็จแล้ว (ดาวน์โหลดอาจเสร็จแล้ว)" },
  "app.applicationStarted": { en: "Application started", th: "แอปเริ่มต้นแล้ว" },
  "app.episodeDownloaded": { en: "Episode {episode} downloaded", th: "ตอน {episode} ดาวน์โหลดเสร็จแล้ว" },
  "app.episodeFailed": { en: "Episode {episode} failed: {error}", th: "ตอน {episode} ล้มเหลว: {error}" },
  "app.mergingVideos": { en: "Merging videos...", th: "กำลังรวมวิดีโอ..." },
  "app.foundFiles": { en: "Found {count} files", th: "พบ {count} ไฟล์" },
  "app.couldNotListFiles": { en: "Could not list files", th: "ไม่สามารถแสดงรายการไฟล์" },
  "app.playingFile": { en: "Playing: {path}", th: "กำลังเล่น: {path}" },
  "app.pastedUrl": { en: "Pasted URL: {url}", th: "วาง URL แล้ว: {url}" },
  "app.fetchingUrl": { en: "Fetching: {url}", th: "กำลังดึงข้อมูล: {url}" },
  "app.autoLoaded": { en: "Auto-loaded: {title} ({episodes} episodes)", th: "โหลดอัตโนมัติ: {title} ({episodes} ตอน)" },
  "app.fetchingInfo": { en: "Fetching info...", th: "กำลังดึงข้อมูล..." },
  "app.downloadingStatus": { en: "Downloading", th: "กำลังดาวน์โหลด" },
  "app.logsCount": { en: "Logs ({count})", th: "บันทึก ({count})" },
  "app.startQueue": { en: "Start Queue", th: "เริ่มคิว" },
  "app.showQueueList": { en: "Show Queue List", th: "แสดงรายการคิว" },
  "app.hideQueueList": { en: "Hide Queue List", th: "ซ่อนรายการคิว" },
  "app.overall": { en: "Overall", th: "โดยรวม" },
  "app.mergedLabel": { en: "Merged:", th: "รวมแล้ว:" },
  "app.autoMerge": { en: "Auto merge", th: "รวมอัตโนมัติ" },
  "app.pause": { en: "Pause", th: "หยุดชั่วคราว" },
  "app.resume": { en: "Resume", th: "ดำเนินการต่อ" },
  "app.cancel": { en: "Cancel", th: "ยกเลิก" },
  "app.clear": { en: "Clear", th: "ล้าง" },
  "app.paste": { en: "Paste", th: "วาง" },
  "app.ready": { en: "Ready", th: "พร้อม" },
  "app.completed": { en: "Completed", th: "เสร็จสิ้น" },
  "app.error": { en: "Error", th: "ข้อผิดพลาด" },

  // Tabs
  "tabs.download": { en: "Download", th: "ดาวน์โหลด" },
  "tabs.library": { en: "Library", th: "ไลบรารี" },
  "tabs.browse": { en: "Browse", th: "เรียกดู" },
  "tabs.files": { en: "Files", th: "ไฟล์" },
  "tabs.history": { en: "History", th: "ประวัติ" },
  "tabs.settings": { en: "Settings", th: "ตั้งค่า" },

  // Error Boundary
  "errorBoundary.title": { en: "Something went wrong", th: "เกิดข้อผิดพลาด" },
  "errorBoundary.message": { en: "An unexpected error occurred", th: "เกิดข้อผิดพลาดที่ไม่คาดคิด" },
  "errorBoundary.tryAgain": { en: "Try Again", th: "ลองอีกครั้ง" },

  // Video Preview
  "preview.episodeTitle": { en: "{title} - Episode {number}", th: "{title} - ตอน {number}" },
  "preview.episodePreview": { en: "Episode {number} Preview", th: "ตัวอย่างตอน {number}" },
  "preview.close": { en: "Close preview", th: "ปิดตัวอย่าง" },
  "preview.loading": { en: "Loading preview...", th: "กำลังโหลดตัวอย่าง..." },
  "preview.notAvailable": { en: "Preview not available", th: "ไม่มีตัวอย่าง" },
  "preview.cannotPreview": { en: "This video cannot be previewed. Try downloading it instead.", th: "ไม่สามารถดูตัวอย่างวิดีโอนี้ได้ ลองดาวน์โหลดแทน" },
  "preview.unsupported": { en: "Your browser does not support the video tag.", th: "เบราว์เซอร์ของคุณไม่รองรับแท็กวิดีโอ" },
  "preview.selectEpisode": { en: "Select an episode to preview", th: "เลือกตอนเพื่อดูตัวอย่าง" },

  // Video Player
  "player.close": { en: "Close video player", th: "ปิดเครื่องเล่นวิดีโอ" },
  "player.loading": { en: "Loading video...", th: "กำลังโหลดวิดีโอ..." },
  "player.failed": { en: "Failed to load video", th: "โหลดวิดีโอไม่สำเร็จ" },
  "player.fileNotSupported": { en: "The file may not exist or is not supported", th: "ไฟล์อาจไม่มีอยู่หรือไม่รองรับ" },
  "player.unsupported": { en: "Your browser does not support the video tag.", th: "เบราว์เซอร์ของคุณไม่รองรับแท็กวิดีโอ" },
  "player.unmute": { en: "Unmute", th: "เปิดเสียง" },
  "player.mute": { en: "Mute", th: "ปิดเสียง" },
  "player.exitFullscreen": { en: "Exit Fullscreen", th: "ออกจากเต็มจอ" },
  "player.fullscreen": { en: "Fullscreen", th: "เต็มจอ" },
  "player.toggleFullscreen": { en: "Toggle fullscreen", th: "สลับเต็มจอ" },

  // Metadata
  "metadata.rating": { en: "Rating", th: "คะแนน" },
  "metadata.year": { en: "Year", th: "ปี" },
  "metadata.genre": { en: "Genre", th: "ประเภท" },
  "metadata.duration": { en: "Duration", th: "ระยะเวลา" },
  "metadata.noDescription": { en: "No description available", th: "ไม่มีคำอธิบาย" },
  "metadata.episodes": { en: "Episodes", th: "ตอน" },
  "metadata.min": { en: "min", th: "นาที" },

  // File Browser (extra)
  "files.open": { en: "Open", th: "เปิด" },
  "files.eps": { en: "Eps", th: "ตอน" },
  "files.size": { en: "Size", th: "ขนาด" },
  "files.playFile": { en: "Play file", th: "เล่นไฟล์" },

  // Library Stats
  "stats.noStats": { en: "No statistics available", th: "ไม่มีสถิติ" },
  "stats.totalSeries": { en: "Total Series", th: "ซีรีส์ทั้งหมด" },
  "stats.totalEpisodes": { en: "Total Episodes", th: "ตอนทั้งหมด" },
  "stats.completed": { en: "{count} completed", th: "{count} เสร็จสิ้น" },
  "stats.storageUsed": { en: "Storage Used", th: "พื้นที่ใช้ไป" },
  "stats.favorites": { en: "Favorites", th: "รายการโปรด" },
  "stats.tags": { en: "{count} tags", th: "{count} แท็ก" },
  "stats.bySource": { en: "By Source", th: "ตามแหล่ง" },
  "stats.sourceInfo": { en: "{series} series · {episodes} eps", th: "{series} ซีรีส์ · {episodes} ตอน" },
  "stats.noData": { en: "No data", th: "ไม่มีข้อมูล" },
  "stats.watchStatus": { en: "Watch Status", th: "สถานะการรับชม" },
  "stats.completeTitle": { en: "Complete: {count}", th: "สมบูรณ์: {count}" },
  "stats.inProgressTitle": { en: "In Progress: {count}", th: "กำลังดำเนินการ: {count}" },
  "stats.notStartedTitle": { en: "Not Started: {count}", th: "ยังไม่เริ่ม: {count}" },
  "stats.monthlyDownloads": { en: "Monthly Downloads (Last 6 Months)", th: "ดาวน์โหลดรายเดือน (6 เดือนล่าสุด)" },
  "stats.noMonthlyData": { en: "No data for the last 6 months", th: "ไม่มีข้อมูล 6 เดือนล่าสุด" },

  // Scheduler (extra)
  "scheduler.nameRequired": { en: "Name is required", th: "ต้องระบุชื่อ" },
  "scheduler.urlRequired": { en: "URL is required", th: "ต้องระบุ URL" },
  "scheduler.outputDirRequired": { en: "Output directory is required", th: "ต้องระบุโฟลเดอร์บันทึก" },
  "scheduler.browseOutputFolder": { en: "Browse for output folder", th: "เลือกโฟลเดอร์บันทึก" },
  "scheduler.selectPreset": { en: "Schedule preset", th: "เลือกตาราง" },
  "scheduler.scheduleNameAria": { en: "Schedule name", th: "ชื่อตาราง" },
  "scheduler.videoUrlAria": { en: "Video URL", th: "URL วิดีโอ" },
  "scheduler.outputDirAria": { en: "Output directory", th: "โฟลเดอร์บันทึก" },
  "scheduler.outputDirPlaceholder": { en: "/path/to/downloads", th: "/path/to/downloads" },

  // Notifications (extra)
  "notifications.markAllReadAria": { en: "Mark all notifications as read", th: "ทำเครื่องหมายแจ้งเตือนทั้งหมดว่าอ่านแล้ว" },
  "notifications.unreadLabel": { en: "Unread", th: "ยังไม่อ่าน" },
  "notifications.clearAllAria": { en: "Clear all notifications", th: "ล้างการแจ้งเตือนทั้งหมด" },
  "notifications.notificationsAria": { en: "Notifications", th: "การแจ้งเตือน" },

  // Backup (extra)
  "backup.saveTitle": { en: "Save Backup", th: "บันทึกข้อมูลสำรอง" },
  "backup.selectTitle": { en: "Select Backup File", th: "เลือกไฟล์ข้อมูลสำรอง" },
  "backup.failedFindDuplicates": { en: "Failed to find duplicates: {error}", th: "ค้นหาซ้ำไม่สำเร็จ: {error}" },
  "backup.episodesCount": { en: "{completed}/{total} episodes", th: "{completed}/{total} ตอน" },

  // Webhook types
  "webhook.discord": { en: "Discord", th: "Discord" },
  "webhook.lineNotify": { en: "LINE Notify", th: "LINE Notify" },
  "webhook.customWebhook": { en: "Custom Webhook", th: "เว็บฮุกที่กำหนดเอง" },
  "webhook.lineNoUrlHint": { en: "LINE Notify uses the token below instead of URL", th: "LINE Notify ใช้โทเค็นด้านล่างแทน URL" },
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
