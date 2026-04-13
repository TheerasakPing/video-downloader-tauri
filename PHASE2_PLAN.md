# Phase 2: Architecture Refactoring - Execution Plan
**Date:** 2026-04-13  
**Duration:** 3-5 days  
**Status:** Starting

## 🎯 Objectives

1. Split App.tsx (1,958 lines → 300 line modules)
2. Implement useReducer (40+ states → 1 reducer)
3. Extract custom hooks
4. Add error boundaries

**Expected Result:** 50% reduction in re-renders, better maintainability

## 📋 Step-by-Step Plan

### Step 1: Create Folder Structure (30 min)
```
src/
├── features/
│   ├── download/
│   │   ├── DownloadPanel.tsx
│   │   ├── useDownload.ts
│   │   └── types.ts
│   ├── batch/
│   │   ├── BatchPanel.tsx
│   │   └── useBatch.ts
│   ├── library/
│   │   ├── LibraryPanel.tsx
│   │   └── useLibrary.ts
│   └── settings/
│       ├── SettingsPanel.tsx
│       └── useSettings.ts
├── hooks/
│   ├── useAppState.ts (useReducer)
│   └── useEventListeners.ts
└── components/ (existing)
```

### Step 2: Extract State Management (1-2 hours)
- Create useAppState.ts with useReducer
- Define AppState type
- Define AppAction types
- Implement reducer function

### Step 3: Extract Custom Hooks (2-3 hours)
- useDownload.ts
- useBatch.ts
- useLibrary.ts
- useEventListeners.ts

### Step 4: Split Components (3-4 hours)
- Extract DownloadPanel
- Extract BatchPanel
- Extract LibraryPanel
- Extract SettingsPanel

### Step 5: Add Error Boundaries (1 hour)
- Create ErrorBoundary component
- Wrap major sections
- Add fallback UI

### Step 6: Update App.tsx (1 hour)
- Use new hooks
- Import feature panels
- Remove old code
- Test everything

### Step 7: Verification (1 hour)
- TypeScript compiles
- No runtime errors
- All features work
- Git commit & push

## 🚀 Let's Start!

**Current:** App.tsx is 1,958 lines  
**Target:** App.tsx ~200 lines, features in modules  
**Benefit:** Easier to maintain, test, and extend
