import { chromium } from 'playwright';
import fs from 'fs';

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36',
  locale: 'th-TH',
});
const page = await context.newPage();

const testUrl = 'https://njavtv.com/th/dass-812-uncensored-leak';

console.log('[Test] Navigating to:', testUrl);
await page.goto(testUrl, { waitUntil: 'networkidle', timeout: 60000 });
await page.waitForTimeout(5000);

// Extract cookies
const cookies = await context.cookies();
console.log('\n[Cookies] Found:', cookies.length);
cookies.forEach(c => console.log(`  ${c.name}=${c.value.substring(0, 30)}...`));

// Save cookies for later use
fs.writeFileSync('/tmp/njavtv_cookies.json', JSON.stringify(cookies, null, 2));

// Step 1: Extract page info
console.log('\n[Step 1] Extracting page structure...');
const pageInfo = await page.evaluate(() => {
  const html = document.documentElement.outerHTML;
  const results = {
    title: document.title,
    ogTitle: document.querySelector('meta[property="og:title"]')?.content,
    ogImage: document.querySelector('meta[property="og:image"]')?.content,
    hasHlsJs: !!window.Hls,
    hasWindowHls: !!(window.hls && window.hls.url),
    videoElements: document.querySelectorAll('video').length,
    iframes: document.querySelectorAll('iframe').length,
  };

  // Look for script tags with config
  const scripts = Array.from(document.querySelectorAll('script'))
    .filter(s => s.textContent)
    .map(s => s.textContent);
  
  // Search for m3u8 in scripts
  const m3u8Matches = [];
  scripts.forEach((script, idx) => {
    const matches = script.match(/https?:\/\/[^"'\s]+\.m3u8[^"'\s]*/g);
    if (matches) {
      m3u8Matches.push({ scriptIdx: idx, urls: matches });
    }
  });
  results.m3u8InScripts = m3u8Matches;

  // Search for player config
  const configPatterns = [];
  scripts.forEach((script, idx) => {
    if (script.includes('new Hls') || script.includes('new Plyr') || script.includes('sources:')) {
      configPatterns.push({ scriptIdx: idx, snippet: script.substring(0, 200) });
    }
  });
  results.configPatterns = configPatterns;

  return results;
});

console.log('Page Info:', JSON.stringify(pageInfo, null, 2));

// Step 2: Check for window.hls
console.log('\n[Step 2] Checking window.hls...');
const windowHls = await page.evaluate(() => {
  if (window.hls) {
    return {
      url: window.hls.url,
      config: window.hls.config ? 'exists' : 'none',
      currentLevel: window.hls.currentLevel,
      levels: window.hls.levels?.length,
    };
  }
  return { exists: false };
});
console.log('window.hls:', JSON.stringify(windowHls, null, 2));

// Step 3: Check video elements
console.log('\n[Step 3] Checking video elements...');
const videoElements = await page.evaluate(() => {
  const videos = Array.from(document.querySelectorAll('video'));
  return videos.map((v, idx) => ({
    index: idx,
    src: v.src?.substring(0, 100),
    currentSrc: v.currentSrc?.substring(0, 100),
    readyState: v.readyState,
    networkState: v.networkState,
    duration: v.duration,
    error: v.error?.message,
    // Check if HLS is attached
    hasHls: !!v._hls || !!v.hls,
  }));
});
console.log('Video Elements:', JSON.stringify(videoElements, null, 2));

// Step 4: Check network requests
console.log('\n[Step 4] Checking network requests...');
await page.waitForTimeout(2000);
const networkRequests = await page.evaluate(() => {
  const entries = performance.getEntriesByType('resource');
  return entries
    .filter(e => 
      e.name.includes('.m3u8') || 
      e.name.includes('playlist') || 
      e.name.includes('njavtv') ||
      e.name.includes('api')
    )
    .slice(-20) // Last 20 requests
    .map(e => ({
      name: e.name.substring(0, 150),
      type: e.initiatorType,
      duration: Math.round(e.duration),
    }));
});
console.log('Network Requests:', JSON.stringify(networkRequests, null, 2));

// Step 5: Try to extract video ID and call API
console.log('\n[Step 5] Extracting video ID from URL...');
const videoIdMatch = testUrl.match(/\/([a-z0-9-]+)$/);
const videoId = videoIdMatch ? videoIdMatch[1] : null;
console.log('Video ID:', videoId);

if (videoId) {
  console.log('\n[Step 6] Calling /api/playlists/{id}...');
  try {
    const playlistResp = await page.evaluate(async (vid) => {
      const resp = await fetch(`/api/playlists/${vid}`, {
        method: 'GET',
        credentials: 'include',
        headers: {
          'Accept': 'application/json, text/plain, */*',
          'X-Requested-With': 'XMLHttpRequest',
        }
      });
      const contentType = resp.headers.get('content-type');
      const text = await resp.text();
      return {
        status: resp.status,
        contentType,
        text: text.substring(0, 2000),
      };
    }, videoId);
    console.log('Playlist API Response:', JSON.stringify(playlistResp, null, 2));
  } catch (e) {
    console.log('Playlist API Error:', e.message);
  }
}

// Save full HTML for analysis
const fullHtml = await page.content();
fs.writeFileSync('/tmp/njavtv_full_page.html', fullHtml);
console.log('\n[Done] Full HTML saved to /tmp/njavtv_full_page.html');

await browser.close();
