import { chromium } from 'playwright';

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36',
  locale: 'th-TH',
});
const page = await context.newPage();

const videoUrl = 'https://njavtv.com/dm13/th/cus-1267';
const videoId = 'cus-1267';

console.log('[Flow Test] Navigating to page...');
await page.goto(videoUrl, { waitUntil: 'networkidle', timeout: 60000 });
await page.waitForTimeout(3000);

// Step 1: Check if we need to call /api/items/{id}/view first
console.log('\n[Flow Test] Step 1: Calling /api/items/{id}/view...');
const viewResult = await page.evaluate(async (vid) => {
  try {
    // First, try to find the item ID from the page
    const html = document.documentElement.outerHTML;
    const itemIdMatch = html.match(/\/api\/items\/([a-zA-Z0-9]+)\/view/);
    const itemId = itemIdMatch ? itemIdMatch[1] : null;

    if (!itemId) {
      return { error: 'Could not find item ID in page', html: html.substring(0, 1000) };
    }

    const resp = await fetch(`/api/items/${itemId}/view`, {
      method: 'POST',
      credentials: 'include',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
        'X-Requested-With': 'XMLHttpRequest',
      },
      body: JSON.stringify({})
    });

    const data = await resp.json();
    return { success: true, itemId, status: resp.status, data };
  } catch(e) {
    return { error: e.message };
  }
}, videoId);

console.log('[Flow Test] View result:', JSON.stringify(viewResult, null, 2).substring(0, 1000));

// Step 2: Now try /api/playlists/{id}
console.log('\n[Flow Test] Step 2: Calling /api/playlists/{id}...');
const playlistResult = await page.evaluate(async (vid) => {
  try {
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

    if (contentType && contentType.includes('application/json')) {
      return { success: true, status: resp.status, data: JSON.parse(text) };
    } else {
      return { error: 'Not JSON', status: resp.status, contentType, text: text.substring(0, 500) };
    }
  } catch(e) {
    return { error: e.message };
  }
}, videoId);

console.log('[Flow Test] Playlist result:', JSON.stringify(playlistResult, null, 2).substring(0, 2000));

// Step 3: Try to find the actual video player config
console.log('\n[Flow Test] Step 3: Searching for video player config...');
const playerConfig = await page.evaluate(() => {
  const html = document.documentElement.outerHTML;
  const results = {};

  // Look for HLS config
  const hlsMatch = html.match(/new\s+Hls\s*\(\s*\{[^}]+\}\s*\)/g);
  if (hlsMatch) results.hlsInstances = hlsMatch;

  // Look for player config
  const playerMatch = html.match(/player\s*=\s*new\s+Plyr\s*\([^)]+\)/g);
  if (playerMatch) results.playerInstances = playerMatch;

  // Look for video sources
  const sourcesMatch = html.match(/sources\s*:\s*\[[^\]]+\]/g);
  if (sourcesMatch) results.sources = sourcesMatch;

  // Look for m3u8 URLs in scripts
  const scripts = Array.from(document.querySelectorAll('script'))
    .map(s => s.textContent || s.innerHTML)
    .join('\n');
  const m3u8InScripts = scripts.match(/https?:\/\/[^"'\s]+\.m3u8[^"'\s]*/g);
  if (m3u8InScripts) results.m3u8InScripts = [...new Set(m3u8InScripts)];

  // Look for config objects
  const configMatch = html.match(/window\.__INITIAL_STATE__\s*=\s*({[^;]+});/);
  if (configMatch) results.initialState = configMatch[1].substring(0, 500);

  return results;
});

console.log('[Flow Test] Player config:', JSON.stringify(playerConfig, null, 2));

// Step 4: Check network requests
console.log('\n[Flow Test] Step 4: Checking network requests...');
await page.waitForTimeout(2000);

const resources = await page.evaluate(() => {
  const entries = performance.getEntriesByType('resource');
  return entries
    .filter(e =>
      e.name.includes('.m3u8') ||
      e.name.includes('playlist') ||
      e.name.includes('video') ||
      e.name.includes('master')
    )
    .map(e => e.name);
});

console.log('[Flow Test] Video-related network requests:', resources);

// Step 5: Try to extract from actual video element
console.log('\n[Flow Test] Step 5: Checking video elements...');
const videoInfo = await page.evaluate(() => {
  const videos = Array.from(document.querySelectorAll('video'));
  return videos.map(v => ({
    src: v.src,
    currentSrc: v.currentSrc,
    readyState: v.readyState,
    networkState: v.networkState,
    error: v.error ? v.error.message : null
  }));
});

console.log('[Flow Test] Video elements:', JSON.stringify(videoInfo, null, 2));

await browser.close();
