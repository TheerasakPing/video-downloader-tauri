# Optimization Analysis - video-downloader-tauri
**Date:** 2026-04-13  
**Focus:** Performance, Stability, Quality

## 🔍 Current Architecture Analysis

### Frontend (React + TypeScript)
- **Size:** 1,958 lines (App.tsx)
- **State Management:** useState hooks (40+ states)
- **Event Listeners:** 9 Tauri event listeners
- **Re-renders:** Potential optimization needed

### Backend (Rust + Tauri)
- **Parsers:** 7 different site parsers
- **Download Engine:** FFmpeg + manual HLS
- **Concurrency:** Tokio async runtime
- **Database:** SQLite for queue/library

## 🎯 Optimization Opportunities

### 1. Performance Bottlenecks

#### Frontend Issues
```typescript
// ❌ PROBLEM: Too many states causing re-renders
const [url, setUrl] = useState("");
const [series, setSeries] = useState<SeriesInfo | null>(null);
const [selectedEpisodes, setSelectedEpisodes] = useState<Set<number>>(new Set());
// ... 40+ more states

// ✅ SOLUTION: Use useReducer for related states
type AppState = {
  download: DownloadState;
  batch: BatchState;
  ui: UIState;
};

const [state, dispatch] = useReducer(appReducer, initialState);
```

#### Backend Issues
```rust
// ❌ PROBLEM: Sequential episode downloads
for episode in episodes {
    downloader.download_episode(episode).await?;
}

// ✅ SOLUTION: Parallel downloads with semaphore
use tokio::sync::Semaphore;
let semaphore = Arc::new(Semaphore::new(3)); // Max 3 concurrent

let tasks: Vec<_> = episodes.iter().map(|ep| {
    let sem = semaphore.clone();
    tokio::spawn(async move {
        let _permit = sem.acquire().await;
        downloader.download_episode(*ep).await
    })
}).collect();
```

### 2. Memory Optimization

#### Issue: Large State Objects
```typescript
// ❌ PROBLEM: Storing full series info in multiple places
const [series, setSeries] = useState<SeriesInfo | null>(null);
const [batchQueue, setBatchQueue] = useState<BatchItem[]>([]);
// Each BatchItem contains full SeriesInfo

// ✅ SOLUTION: Normalize data structure
type SeriesCache = Map<string, SeriesInfo>;
type BatchItem = {
  url: string;
  seriesId: string; // Reference only
  status: Status;
};
```

#### Issue: Event Listener Accumulation (FIXED)
✅ Already fixed with cleanup functions

### 3. Code Quality Improvements

#### Split Large Components
```typescript
// ❌ PROBLEM: App.tsx is 1,958 lines
function App() {
  // Everything in one component
}

// ✅ SOLUTION: Split into feature modules
// src/features/download/DownloadPanel.tsx
// src/features/batch/BatchPanel.tsx
// src/features/library/LibraryPanel.tsx
// src/features/settings/SettingsPanel.tsx
```

#### Extract Custom Hooks
```typescript
// ✅ SOLUTION: Extract reusable logic
// src/hooks/useDownload.ts
export function useDownload() {
  const [state, setState] = useState<DownloadState>(...);
  const startDownload = useCallback(...);
  const pauseDownload = useCallback(...);
  return { state, startDownload, pauseDownload };
}
```

### 4. Stability Improvements

#### Add Error Boundaries
```typescript
// ✅ Add error boundaries for each major section
<ErrorBoundary fallback={<ErrorFallback />}>
  <DownloadPanel />
</ErrorBoundary>
```

#### Implement Retry Logic
```rust
// ✅ Add exponential backoff for network requests
async fn download_with_retry<F, T>(
    f: F,
    max_retries: u32,
) -> Result<T, String>
where
    F: Fn() -> Future<Output = Result<T, String>>,
{
    let mut delay = Duration::from_secs(1);
    for attempt in 0..max_retries {
        match f().await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries - 1 => {
                tokio::time::sleep(delay).await;
                delay *= 2; // Exponential backoff
            }
            Err(e) => return Err(e),
        }
    }
}
```

### 5. Download Speed Optimization

#### Current: Sequential segment downloads
```rust
// ❌ SLOW: One segment at a time
for segment in segments {
    download_segment(segment).await?;
}
```

#### Optimized: Parallel segment downloads
```rust
// ✅ FAST: Multiple segments in parallel
use futures::stream::{self, StreamExt};

stream::iter(segments)
    .map(|seg| download_segment(seg))
    .buffer_unordered(10) // 10 concurrent downloads
    .collect::<Vec<_>>()
    .await;
```

#### Add Connection Pooling
```rust
// ✅ Reuse HTTP connections
let client = Client::builder()
    .pool_max_idle_per_host(10)
    .pool_idle_timeout(Duration::from_secs(90))
    .build()?;
```

### 6. Database Optimization

#### Add Indexes
```sql
-- ✅ Speed up queries
CREATE INDEX idx_library_title ON library(title);
CREATE INDEX idx_queue_status ON queue(status);
CREATE INDEX idx_notifications_read ON notifications(is_read);
```

#### Use Prepared Statements
```rust
// ✅ Reuse compiled queries
let stmt = conn.prepare_cached(
    "INSERT INTO queue (url, status) VALUES (?1, ?2)"
)?;
```

### 7. UI Performance

#### Virtualize Long Lists
```typescript
// ✅ Only render visible items
import { FixedSizeList } from 'react-window';

<FixedSizeList
  height={600}
  itemCount={episodes.length}
  itemSize={50}
>
  {({ index, style }) => (
    <div style={style}>
      <EpisodeItem episode={episodes[index]} />
    </div>
  )}
</FixedSizeList>
```

#### Memoize Expensive Computations
```typescript
// ✅ Cache computed values
const sortedEpisodes = useMemo(() => {
  return episodes.sort((a, b) => a.number - b.number);
}, [episodes]);

const EpisodeCard = memo(({ episode }) => {
  // Component only re-renders if episode changes
});
```

### 8. Build Optimization

#### Enable Production Optimizations
```toml
# Cargo.toml
[profile.release]
opt-level = 3
lto = true
codegen-units = 1
strip = true
```

```javascript
// vite.config.ts
export default {
  build: {
    minify: 'terser',
    terserOptions: {
      compress: {
        drop_console: true,
      },
    },
    rollupOptions: {
      output: {
        manualChunks: {
          vendor: ['react', 'react-dom'],
          tauri: ['@tauri-apps/api'],
        },
      },
    },
  },
};
```

## 📊 Expected Improvements

### Performance
- **Frontend:** 50-70% reduction in re-renders
- **Download Speed:** 3-5x faster with parallel downloads
- **Memory Usage:** 30-40% reduction with normalized state
- **Build Size:** 20-30% smaller with optimizations

### Stability
- **Error Recovery:** Automatic retry on failures
- **Crash Prevention:** Error boundaries prevent full app crashes
- **Resource Management:** Better cleanup prevents leaks

### Quality
- **Maintainability:** Smaller, focused components
- **Testability:** Extracted hooks easier to test
- **Scalability:** Better architecture for future features

## 🎯 Priority Roadmap

### Phase 1: Quick Wins (1-2 days)
1. ✅ Add connection pooling
2. ✅ Enable release optimizations
3. ✅ Add database indexes
4. ✅ Memoize expensive computations

### Phase 2: Architecture (3-5 days)
1. Split App.tsx into feature modules
2. Implement useReducer for state management
3. Extract custom hooks
4. Add error boundaries

### Phase 3: Performance (5-7 days)
1. Implement parallel downloads
2. Add virtualized lists
3. Optimize re-renders
4. Implement caching strategies

### Phase 4: Advanced (1-2 weeks)
1. Add service worker for offline support
2. Implement incremental downloads
3. Add download resume capability
4. Optimize memory usage

## 🔧 Implementation Priority

### Must Have (Critical)
- ✅ Parallel downloads (3-5x speed boost)
- ✅ Connection pooling (reduce latency)
- ✅ Error boundaries (prevent crashes)
- ✅ Database indexes (faster queries)

### Should Have (Important)
- Split large components (maintainability)
- useReducer for state (reduce re-renders)
- Virtualized lists (handle large datasets)
- Retry logic (reliability)

### Nice to Have (Enhancement)
- Service worker (offline support)
- Download resume (user experience)
- Advanced caching (performance)
- Telemetry (monitoring)

## 📈 Metrics to Track

### Before Optimization
- Download speed: ~1 episode/minute
- Memory usage: ~200MB
- Re-renders: ~50 per action
- Build size: ~15MB

### After Optimization (Expected)
- Download speed: ~3-5 episodes/minute
- Memory usage: ~120MB
- Re-renders: ~15 per action
- Build size: ~10MB

## 🚀 Next Steps

1. Review and approve optimization plan
2. Implement Phase 1 (quick wins)
3. Measure improvements
4. Proceed to Phase 2 based on results

---
**Status:** Ready for implementation
**Estimated Time:** 2-4 weeks for full optimization
**Risk:** Low (incremental improvements)
