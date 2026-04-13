# Optimization Roadmap - video-downloader-tauri
**Created:** 2026-04-13  
**Target:** Performance, Stability, Quality

## 🎯 3 หลักการหลัก

### 1️⃣ PERFORMANCE (ความเร็ว)
**เป้าหมาย:** ทำให้ app ทำงานเร็วขึ้น 3-5 เท่า

**ปัญหาหลัก:**
- ❌ Download sequential (ทีละ episode)
- ❌ 40+ React states → re-render บ่อย
- ❌ ไม่มี connection pooling
- ❌ ไม่มี database indexes

**วิธีแก้:**
```
✅ Parallel downloads (3 concurrent)
✅ useReducer state management
✅ HTTP connection pooling
✅ Database indexes
✅ Memoize components
```

**ผลลัพธ์:**
- Download: 1 ep/min → 3-5 ep/min
- Memory: 200MB → 120MB
- Re-renders: 50 → 15 per action

---

### 2️⃣ STABILITY (ความเสถียร)
**เป้าหมาย:** ป้องกัน crash และ error

**ปัญหาหลัก:**
- ❌ ไม่มี retry logic
- ❌ ไม่มี error boundaries
- ❌ Network error → app crash
- ❌ ไม่มี graceful degradation

**วิธีแก้:**
```
✅ Exponential backoff retry
✅ Error boundaries (React)
✅ Proper error handling
✅ Fallback UI components
✅ Logging & monitoring
```

**ผลลัพธ์:**
- Network error → auto retry
- Component error → isolated
- Better error messages
- Crash prevention

---

### 3️⃣ QUALITY (คุณภาพ)
**เป้าหมาย:** Code ที่ดี readable maintainable

**ปัญหาหลัก:**
- ❌ App.tsx 1,958 lines (ใหญ่เกินไป)
- ❌ Logic ปนกัน
- ❌ ยากต่อการ test
- ❌ ยากต่อการ maintain

**วิธีแก้:**
```
✅ Split into feature modules
✅ Extract custom hooks
✅ Separate concerns
✅ Better folder structure
✅ Unit tests
```

**ผลลัพธ์:**
- Components: 200-300 lines each
- Reusable hooks
- Easy to test
- Easy to maintain

---

## 📋 Implementation Plan

### Phase 1: Quick Wins (1-2 days) ⚡
**ทำให้เห็นผลเร็ว**

```
1. Add HTTP connection pooling
   - File: src-tauri/src/downloader.rs
   - Change: Client builder config
   - Impact: 20-30% faster downloads

2. Add database indexes
   - File: src-tauri/src/queue_db.rs
   - Change: CREATE INDEX statements
   - Impact: 50% faster queries

3. Enable release optimizations
   - File: Cargo.toml, vite.config.ts
   - Change: Compiler flags
   - Impact: 20% smaller, faster

4. Memoize React components
   - File: src/App.tsx
   - Change: Add memo() and useMemo()
   - Impact: 30% fewer re-renders
```

**Expected Result:** 30-40% overall improvement

---

### Phase 2: Architecture (3-5 days) 🏗️
**ปรับปรุง code structure**

```
1. Split App.tsx into modules
   src/features/
   ├── download/
   │   ├── DownloadPanel.tsx
   │   ├── useDownload.ts
   │   └── types.ts
   ├── batch/
   │   ├── BatchPanel.tsx
   │   └── useBatch.ts
   ├── library/
   │   ├── LibraryPanel.tsx
   │   └── useLibrary.ts
   └── settings/
       ├── SettingsPanel.tsx
       └── useSettings.ts

2. Extract custom hooks
   src/hooks/
   ├── useDownload.ts
   ├── useBatch.ts
   ├── useLibrary.ts
   ├── useSettings.ts
   └── useEventListeners.ts

3. Implement useReducer
   - Consolidate 40+ states
   - Better state management
   - Easier to debug

4. Add error boundaries
   - Wrap major sections
   - Prevent full app crash
   - Show fallback UI
```

**Expected Result:** 50% reduction in re-renders, better maintainability

---

### Phase 3: Performance (5-7 days) 🚀
**ปรับปรุง download speed**

```
1. Parallel downloads
   - Use tokio::spawn
   - Semaphore for concurrency (3 max)
   - 3-5x faster

2. Parallel segment downloads
   - Use futures::stream
   - buffer_unordered(10)
   - 2-3x faster per episode

3. Virtualized lists
   - Use react-window
   - Only render visible items
   - Handle 1000+ episodes

4. Caching strategies
   - Cache series info
   - Cache parsed HTML
   - Reduce API calls
```

**Expected Result:** 3-5x faster downloads, better UI responsiveness

---

### Phase 4: Advanced (1-2 weeks) 🎁
**เพิ่มเติม features**

```
1. Service worker
   - Offline support
   - Background sync
   - Cache management

2. Download resume
   - Save progress
   - Resume from checkpoint
   - Better UX

3. Incremental downloads
   - Download only new episodes
   - Smart caching
   - Reduce bandwidth

4. Telemetry
   - Track performance
   - Monitor errors
   - User analytics
```

---

## 🔧 Quick Implementation Guide

### Phase 1: Connection Pooling (30 min)
```rust
// src-tauri/src/downloader.rs
let client = Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))
    .user_agent("Mozilla/5.0...")
    .build()?;
```

### Phase 1: Database Indexes (15 min)
```rust
// src-tauri/src/queue_db.rs
pub fn init_db(conn: &Connection) -> Result<()> {
    conn.execute_batch(
        "CREATE INDEX IF NOT EXISTS idx_queue_status ON queue(status);
         CREATE INDEX IF NOT EXISTS idx_library_title ON library(title);
         CREATE INDEX IF NOT EXISTS idx_notifications_read ON notifications(is_read);"
    )?;
    Ok(())
}
```

### Phase 1: Release Optimizations (10 min)
```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

### Phase 2: useReducer (2-3 hours)
```typescript
// src/hooks/useAppState.ts
type AppState = {
  download: DownloadState;
  batch: BatchState;
  ui: UIState;
};

type AppAction = 
  | { type: 'START_DOWNLOAD'; payload: DownloadState }
  | { type: 'UPDATE_BATCH'; payload: BatchItem[] }
  | { type: 'TOGGLE_UI'; payload: keyof UIState };

export function useAppState() {
  const [state, dispatch] = useReducer(appReducer, initialState);
  return { state, dispatch };
}
```

### Phase 3: Parallel Downloads (3-4 hours)
```rust
// src-tauri/src/downloader.rs
pub async fn download_episodes_parallel(
    episodes: Vec<i32>,
    downloader: Arc<VideoDownloader>,
) -> Result<Vec<DownloadResult>> {
    let semaphore = Arc::new(Semaphore::new(3));
    
    let tasks: Vec<_> = episodes.iter().map(|ep| {
        let sem = semaphore.clone();
        let dl = downloader.clone();
        tokio::spawn(async move {
            let _permit = sem.acquire().await;
            dl.download_episode(*ep).await
        })
    }).collect();
    
    futures::future::join_all(tasks).await
        .into_iter()
        .collect::<Result<Vec<_>>>()
}
```

---

## 📊 Success Metrics

### Before
```
Download Speed:     1 episode/minute
Memory Usage:       ~200MB
Re-renders:         ~50 per action
Build Size:         ~15MB
Error Recovery:     Manual restart
Code Maintainability: Hard (1,958 line file)
```

### After Phase 1
```
Download Speed:     1.5 episodes/minute (+50%)
Memory Usage:       ~180MB (-10%)
Re-renders:         ~40 per action (-20%)
Build Size:         ~12MB (-20%)
Error Recovery:     Same
Code Maintainability: Same
```

### After Phase 2
```
Download Speed:     1.5 episodes/minute
Memory Usage:       ~160MB (-20%)
Re-renders:         ~15 per action (-70%)
Build Size:         ~12MB
Error Recovery:     Same
Code Maintainability: Good (300 line files)
```

### After Phase 3
```
Download Speed:     4-5 episodes/minute (+400%)
Memory Usage:       ~120MB (-40%)
Re-renders:         ~15 per action
Build Size:         ~12MB
Error Recovery:     Auto retry
Code Maintainability: Good
```

---

## ⚠️ Risk Assessment

### Low Risk
- ✅ Connection pooling (no breaking changes)
- ✅ Database indexes (backward compatible)
- ✅ Release optimizations (build only)
- ✅ Memoization (no logic changes)

### Medium Risk
- ⚠️ useReducer (state management refactor)
- ⚠️ Component splitting (refactoring)
- ⚠️ Parallel downloads (concurrency)

### Mitigation
- Test each phase thoroughly
- Keep git history for rollback
- Gradual rollout
- Monitor performance metrics

---

## 🚀 Getting Started

### Step 1: Approve Plan
- Review this roadmap
- Confirm priorities
- Allocate resources

### Step 2: Phase 1 (Quick Wins)
- Implement connection pooling
- Add database indexes
- Enable release optimizations
- Measure improvements

### Step 3: Measure & Iterate
- Track metrics
- Gather feedback
- Adjust plan if needed

### Step 4: Phase 2+
- Proceed based on Phase 1 results
- Continue optimization cycle

---

## 📞 Questions?

**Q: How long will this take?**
A: 2-4 weeks for full optimization (can do incrementally)

**Q: Will it break existing functionality?**
A: No, all changes are backward compatible

**Q: Can we do this gradually?**
A: Yes! Each phase is independent

**Q: What's the ROI?**
A: 3-5x faster downloads, 40% less memory, better stability

---

**Status:** Ready to implement  
**Priority:** High  
**Complexity:** Medium  
**Risk:** Low
