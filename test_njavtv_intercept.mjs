import { chromium } from 'playwright';

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36',
  locale: 'th-TH',
});
const page = await context.newPage();

// Track all requests
const m3u8Urls = [];
page.on('request', request => {
  const url = request.url();
  if (url.includes('.m3u8') || url.includes('master') || url.includes('playlist')) {
    console.log('[INTERCEPT] Found URL:', url);
    m3u8Urls.push(url);
  }
});

const videoUrl = 'https://njavtv.com/dm13/th/cus-1267';

console.log('[Test] Navigating to page...');
await page.goto(videoUrl, { waitUntil: 'networkidle', timeout: 60000 });

// Wait for Cloudflare
console.log('[Test] Waiting for page to settle...');
await page.waitForTimeout(8000);

// Try clicking play button
console.log('[Test] Looking for play button...');
try {
  const playButton = await page.locator('button:has-text("Play"), .plyr__control--overlaid, [aria-label="Play"]').first();
  if (await playButton.isVisible({ timeout: 2000 })) {
    console.log('[Test] Clicking play button...');
    await playButton.click();
    await page.waitForTimeout(3000);
  }
} catch (e) {
  console.log('[Test] Play button not found, trying video element...');
}

// Try clicking video element directly
try {
  const video = await page.locator('video').first();
  if (await video.isVisible({ timeout: 2000 })) {
    console.log('[Test] Clicking video element...');
    await video.click();
    await page.waitForTimeout(3000);
  }
} catch (e) {
  console.log('[Test] Video element not found');
}

// Wait for video to start loading
console.log('[Test] Waiting for video to start...');
await page.waitForTimeout(5000);

// Get all network requests from performance API
const perfUrls = await page.evaluate(() => {
  const entries = performance.getEntriesByType('resource');
  return entries
    .filter(e => e.name.includes('.m3u8') || e.name.includes('master'))
    .map(e => e.name);
});

console.log('[Test] M3U8 URLs from performance API:', perfUrls);
console.log('[Test] M3U8 URLs from interception:', m3u8Urls);
console.log('[Test] Total unique m3u8 URLs:', [...new Set([...perfUrls, ...m3u8Urls])].length);

// Check video src attributes
const videoInfo = await page.evaluate(() => {
  const videos = Array.from(document.querySelectorAll('video'));
  return videos.map(v => ({
    src: v.src,
    currentSrc: v.currentSrc,
    readyState: v.readyState,
    paused: v.paused
  }));
});

console.log('[Test] Video info:', JSON.stringify(videoInfo, null, 2));

// Save all found URLs for inspection
const allUrls = [...new Set([...perfUrls, ...m3u8Urls])];
if (allUrls.length > 0) {
  console.log('[Test] Sample m3u8 URL:', allUrls[0]);
}

await browser.close();
