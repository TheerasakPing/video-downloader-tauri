import { chromium } from 'playwright';

const browser = await chromium.launch({ headless: true });
const context = await browser.newContext({
  userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/130.0.0.0 Safari/537.36',
  locale: 'th-TH',
});
const page = await context.newPage();

console.log('[Test] Navigating to njavtv.com...');
await page.goto('https://njavtv.com/dm13/th/cus-1267', { waitUntil: 'networkidle', timeout: 60000 });

await page.waitForTimeout(8000);

const title = await page.title();
console.log('[Test] Page title:', title);

const html = await page.content();
console.log('[Test] HTML length:', html.length);

if (html.includes('_cf_chl') || html.includes('Just a moment')) {
  console.log('[Test] STILL behind Cloudflare after 8s');
  await page.waitForTimeout(15000);
  const html2 = await page.content();
  if (html2.includes('_cf_chl')) {
    console.log('[Test] STILL behind Cloudflare after 23s total');
  } else {
    console.log('[Test] Cloudflare passed!');
  }
} else {
  console.log('[Test] No Cloudflare challenge detected');
}

const videoSrcs = await page.evaluate(() => {
  const urls = [];
  document.querySelectorAll('video').forEach(v => {
    if (v.src) urls.push('video.src: ' + v.src);
    if (v.currentSrc) urls.push('video.currentSrc: ' + v.currentSrc);
  });
  document.querySelectorAll('source').forEach(s => {
    if (s.src) urls.push('source.src: ' + s.src);
  });
  return urls;
});
console.log('[Test] Video elements:', videoSrcs);

const m3u8Match = html.match(/https?:[^"']+\.m3u8[^"']*/);
console.log('[Test] m3u8 in HTML:', m3u8Match ? m3u8Match[0] : 'NOT FOUND');

const apiMatch = html.match(/\/api\/[a-zA-Z0-9\/]+/g);
console.log('[Test] API endpoints found:', apiMatch ? [...new Set(apiMatch)].slice(0, 10) : 'NONE');

const iframes = await page.evaluate(() => {
  return Array.from(document.querySelectorAll('iframe')).map(f => f.src);
});
console.log('[Test] Iframes:', iframes);

const scripts = await page.evaluate(() => {
  const results = [];
  document.querySelectorAll('script').forEach(s => {
    if (s.src && (s.src.includes('jwplayer') || s.src.includes('video') || s.src.includes('player'))) {
      results.push(s.src);
    }
  });
  return results;
});
console.log('[Test] Video player scripts:', scripts);

const fs = await import('fs');
fs.writeFileSync('/tmp/njavtv_full.html', html);
console.log('[Test] Saved full HTML to /tmp/njavtv_full.html');

await browser.close();
