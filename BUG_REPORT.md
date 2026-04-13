# Bug Report: video-downloader-tauri
**Date:** 2026-04-13  
**Status:** 10 bugs found

## Summary
- 🔴 CRITICAL: 4 bugs (app crash)
- 🟡 HIGH: 3 bugs (stability)
- 🟠 MEDIUM: 3 bugs (memory leaks)

## CRITICAL BUGS

### Bug #1: Event Listener Memory Leak
**File:** src/App.tsx:548-750  
**Issue:** setupEventListeners() doesn't return cleanup, listeners accumulate  
**Impact:** Memory leak, performance degradation  
**Fix:** Return unsubscribe functions from setupEventListeners()

### Bug #2: Crypto Key Unwrap Panic
**File:** src-tauri/src/downloader.rs:386-387  
**Issue:** key_bytes.try_into().unwrap() panics if not 16 bytes  
**Impact:** App crash on invalid HLS key  
**Fix:** Handle error with match/Result

### Bug #3: FFmpeg Stderr Unwrap Panic
**File:** src-tauri/src/downloader.rs:728  
**Issue:** child.stderr.take().unwrap() panics if stderr missing  
**Impact:** App crash on FFmpeg error  
**Fix:** Handle Option with match

### Bug #4: Header Parse Unwrap Panic
**File:** src-tauri/src/downloader.rs:874  
**Issue:** effective_referer.parse().unwrap() panics on invalid URL  
**Impact:** App crash on bad referer  
**Fix:** Handle parse error gracefully

## HIGH PRIORITY BUGS

### Bug #5: Batch Processing Race Condition
**File:** src/App.tsx:608-750  
**Issue:** isBatchItemRunningRef stale closure, items process multiple times  
**Impact:** Batch queue unreliable  
**Fix:** Use state instead of ref

### Bug #6: HTML Selector Unwrap Panics
**File:** src-tauri/src/hsck_parser.rs:103, 131, 183  
**Issue:** Selector::parse().unwrap() panics on invalid selector  
**Impact:** Parser crashes on HTML changes  
**Fix:** Use lazy_static or handle errors

### Bug #7: Regex Compile Unwrap Panics
**File:** src-tauri/src/hsck_parser.rs:110, 135  
**Issue:** Regex::new().unwrap() panics on invalid pattern  
**Impact:** Parser crashes  
**Fix:** Use lazy_static for compile-time validation

## MEDIUM PRIORITY BUGS

### Bug #8: Mutex Poison Not Handled
**File:** src-tauri/src/lib.rs:1843, 1849  
**Issue:** .read().unwrap() panics if mutex poisoned  
**Impact:** App crash on thread panic  
**Fix:** Use SafeMutexLock trait

### Bug #9: Non-null Assertion Without Check
**File:** src/App.tsx:625  
**Issue:** item.info! used without null check  
**Impact:** Runtime error if info missing  
**Fix:** Add proper null checks

### Bug #10: Missing useEffect Cleanup
**File:** src/App.tsx:804-817  
**Issue:** setupEventListeners() cleanup not returned  
**Impact:** Memory leak on unmount  
**Fix:** Return cleanup function

## Recommendations
1. Fix CRITICAL bugs immediately (prevent crashes)
2. Fix HIGH bugs this week (improve stability)
3. Fix MEDIUM bugs this month (prevent leaks)
4. Add clippy lints to catch unwrap() calls
5. Add error handling tests
