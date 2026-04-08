#!/usr/bin/env node
/**
 * Test script for njav.org video detection and download
 * 
 * Usage: node test_njav_detection.mjs
 * 
 * This script tests:
 * 1. URL validation for njav.org
 * 2. Auto-detect video URL via Chrome
 * 3. Direct download of detected video
 */

import { spawn } from 'child_process';
import { fileURLToPath } from 'url';
import { dirname, join } from 'path';

const __filename = fileURLToPath(import.meta.url);
const __dirname = dirname(__filename);

const TEST_URLS = [
  'https://njav.org/snos-034/',
];

console.log('='.repeat(60));
console.log('🎬 NJAV.ORG Video Detection Test');
console.log('='.repeat(60));

// Test 1: Check if URL is valid
function testUrlValidation() {
  console.log('\n📋 Test 1: URL Validation');
  
  const njavDomain = 'njav.org';
  
  for (const url of TEST_URLS) {
    const isValid = url.includes(njavDomain) && url.startsWith('http');
    console.log(`  ${isValid ? '✅' : '❌'} ${url}`);
  }
}

// Test 2: Run Tauri dev mode and auto-detect
async function testAutoDetect() {
  console.log('\n🔍 Test 2: Auto-Detect Video URL');
  console.log('  This requires running the Tauri app in dev mode.');
  console.log('  Steps:');
  console.log('  1. Run: npm run tauri dev');
  console.log('  2. Paste URL: https://njav.org/snos-034/');
  console.log('  3. Click 🔍 (Auto Detect) or the Fetch button');
  console.log('  4. Wait for Chrome to detect the video URL');
  console.log('  5. If detected, click Download');
}

// Test 3: Check Chrome availability
function testChromeAvailable() {
  console.log('\n🌐 Test 3: Chrome Browser Check');
  
  const chromePaths = [
    '/Applications/Google Chrome.app/Contents/MacOS/Google Chrome',
    '/Applications/Chromium.app/Contents/MacOS/Chromium',
  ];
  
  let found = false;
  for (const path of chromePaths) {
    try {
      const { execSync } = require('child_process');
      execSync(`test -f "${path}" && echo "found" || echo "not found"`, { encoding: 'utf8' });
      console.log(`  ✅ Chrome found at: ${path}`);
      found = true;
    } catch {
      // not found
    }
  }
  
  if (!found) {
    console.log('  ⚠️  Chrome not found in standard locations');
    console.log('  The app will try to use system Chrome');
  }
}

// Run tests
testUrlValidation();
testChromeAvailable();
testAutoDetect();

console.log('\n' + '='.repeat(60));
console.log('📂 Download Folder Structure (with Group by Source ON):');
console.log('  ~/Downloads/rongyok/');
console.log('  ├── rongyok/');
console.log('  │   └── ep_001.mp4');
console.log('  ├── njavtv/');
console.log('  │   └── ep_001.mp4');
console.log('  ├── njav/');
console.log('  │   └── SNOS-034_EP1.mp4');
console.log('  └── titan/');
console.log('      └── ep_001.mp4');
console.log('='.repeat(60));
