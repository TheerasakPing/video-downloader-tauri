

## WORKING MEMORY
[2026-04-13T03:24:57.755Z] ## บัคที่พบในโปรเจกต์ video-downloader-tauri

### 🔴 Critical Bugs (ร้ายแรง)

1. **Memory Leak: Event Listeners ไม่ cleanup**
   - File: `src/App.tsx` line 548-750
   - Issue: `setupEventListeners()` ใช้ `listen()` แต่ไม่เก็บ unsubscribe function
   - Impact: Event listeners จะสะสมทุกครั้งที่ component mount
   - Fix: ต้อง return cleanup function จาก useEffect

2. **Panic on Invalid Crypto Data**
   - File: `src-tauri/src/downloader.rs` line 386-387
   - Issue: `try_into().unwrap()` จะ panic ถ้า key/iv ไม่ใช่ 16 bytes
   - Impact: App crash เมื่อ HLS key format ผิด
   - Fix: ต้อง handle error แทน unwrap

3. **Panic on FFmpeg stderr**
   - File: `src-tauri/src/downloader.rs` line 728
   - Issue: `child.stderr.take().unwrap()` จะ panic ถ้า stderr ไม่มี
   - Impact: App crash เมื่อ FFmpeg process ผิดพลาด
   - Fix: ต้อง handle Option properly

4. **Panic on Header Parse**
   - File: `src-tauri/src/downloader.rs` line 874
   - Issue: `effective_referer.parse().unwrap()` จะ panic ถ้า referer invalid
   - Impact: App crash เมื่อ referer URL ผิดรูปแบบ
   - Fix: ต้อง handle parse error

### 🟡 High Priority Bugs

5. **Race Condition: Batch Processing**
   - File: `src/App.tsx` line 608-750
   - Issue: `isBatchItemRunningRef.current` ใช้ ref แต่ state อาจ out of sync
   - Impact: Batch items อาจ process ซ้ำหรือ skip
   - Fix: ต้อง use proper state management

6. **Missing Selector Error Handling**
   - File: `src-tauri/src/hsck_parser.rs` line 103, 131, 183
   - Issue: `Selector::parse().unwrap()` จะ panic ถ้า selector invalid
   - Impact: App crash เมื่อ HTML structure เปลี่ยน
   - Fix: ต้อง handle parse error gracefully

7. **Regex Unwrap Panics**
   - File: `src-tauri/src/hsck_parser.rs` line 110, 135
   - Issue: `Regex::new().unwrap()` จะ panic ถ้า regex pattern invalid
   - Impact: App crash ถ้า regex compile fail
   - Fix: ต้อง handle error

### 🟠 Medium Priority Bugs

8. **Mutex Poison Recovery**
   - File: `src-tauri/src/lib.rs` line 1843, 1849
   - Issue: `.read().unwrap()` และ `.write().unwrap()` จะ panic ถ้า mutex poisoned
   - Impact: App crash ถ้า thread panic ขณะ lock
   - Fix: ใช้ SafeMutexLock trait ที่มีอยู่แล้ว

9. **No Cleanup for Event Listeners**
   - File: `src/App.tsx` setupEventListeners
   - Issue: ไม่มี return cleanup function
   - Impact: Memory leak เมื่อ component unmount
   - Fix: ต้อง return unsubscribe functions

10. **Potential Null Reference**
    - File: `src/App.tsx` line 625
    - Issue: `item.info!` ใช้ non-null assertion แต่ info อาจ undefined
    - Impact: Runtime error ถ้า info missing
    - Fix: ต้อง check null properly

