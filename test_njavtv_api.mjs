import { chromium } from 'playwright';

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36',
  locale: 'th-TH',
});
const page = await context.newPage();

console.log('[API Test] Navigating to get cookies...');
await page.goto('https://njavtv.com/dm13/th/cus-1267', { waitUntil: 'networkidle', timeout: 60000 });
await page.waitForTimeout(5000);

const cookies = await context.cookies();
console.log('[API Test] Cookies:', cookies.map(c => c.name).join(', '));

console.log('\n[API Test] Calling /api/playlists/cus-1267...');
const playlistResult = await page.evaluate(async () => {
  try {
    const resp = await fetch('/api/playlists/cus-1267', {
      method: 'GET',
      credentials: 'include',
      headers: {
        'Accept': 'application/json, text/plain, */*',
        'X-Requested-With': 'XMLHttpRequest',
      }
    });
    const data = await resp.json();
    return JSON.stringify(data, null, 2);
  } catch(e) {
    return 'Error: ' + e.message;
  }
});
console.log('[API Test] Playlist response (first 2000 chars):');
console.log(playlistResult.substring(0, 2000));

console.log('\n[API Test] Calling /api/items/gikrii6s/view...');
const viewResult = await page.evaluate(async () => {
  try {
    const resp = await fetch('/api/items/gikrii6s/view', {
      method: 'POST',
      credentials: 'include',
      headers: {
        'Accept': 'application/json',
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({})
    });
    const data = await resp.json();
    return JSON.stringify(data, null, 2);
  } catch(e) {
    return 'Error: ' + e.message;
  }
});
console.log('[API Test] View response:');
console.log(viewResult.substring(0, 1000));

console.log('\n[API Test] Calling /api/playlists/cus-1267 AFTER view...');
const playlistResult2 = await page.evaluate(async () => {
  try {
    const resp = await fetch('/api/playlists/cus-1267', {
      method: 'GET',
      credentials: 'include',
      headers: { 'Accept': 'application/json, text/plain, */*' }
    });
    const data = await resp.json();
    return JSON.stringify(data, null, 2);
  } catch(e) {
    return 'Error: ' + e.message;
  }
});
console.log('[API Test] Playlist response after view (first 2000 chars):');
console.log(playlistResult2.substring(0, 2000));

const fs = await import('fs');
fs.writeFileSync('/tmp/njavtv_playlist_api.json', playlistResult);
fs.writeFileSync('/tmp/njavtv_view_api.json', viewResult);
fs.writeFileSync('/tmp/njavtv_playlist_api2.json', playlistResult2);

await browser.close();
