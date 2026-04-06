import { chromium } from 'playwright';

// This test mimics what the Rust Chrome detector does
const browser = await chromium.launch({ 
  headless: true,
  args: ['--no-sandbox', '--disable-dev-shm-usage', '--disable-gpu']
});
const context = await browser.newContext({
  userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
});
const page = await context.newPage();

const testUrl = 'https://njavtv.com/th/dass-812-uncensored-leak';

console.log('[Test] Navigating to:', testUrl);
await page.goto(testUrl, { waitUntil: 'networkidle', timeout: 60000 });
await page.waitForTimeout(3000);

// Check window.hls (this is what the Rust code does)
console.log('\n[Check 1] window.hls.url...');
const hlsUrl = await page.evaluate(() => {
  if (window.hls && window.hls.url) {
    return window.hls.url;
  }
  return null;
});

if (hlsUrl) {
  console.log('✓ Found m3u8 URL:', hlsUrl);
  
  // Test if we can access this URL with cookies
  console.log('\n[Check 2] Testing m3u8 URL accessibility...');
  const cookies = await context.cookies();
  console.log('Cookies:', cookies.map(c => c.name).join(', '));
  
  // Try to fetch the m3u8
  try {
    const m3u8Content = await page.evaluate(async (url) => {
      try {
        const resp = await fetch(url, {
          credentials: 'include',
          headers: {
            'Accept': 'application/vnd.apple.mpegurl',
          }
        });
        if (resp.ok) {
          const text = await resp.text();
          return { success: true, length: text.length, first500: text.substring(0, 500) };
        }
        return { success: false, status: resp.status };
      } catch (e) {
        return { success: false, error: e.message };
      }
    }, hlsUrl);
    
    console.log('M3U8 fetch result:', JSON.stringify(m3u8Content, null, 2));
  } catch (e) {
    console.log('M3U8 fetch error:', e.message);
  }
} else {
  console.log('✗ window.hls.url not found');
  
  // Try alternative: check video elements
  console.log('\n[Check 3] Video elements...');
  const videos = await page.evaluate(() => {
    const vids = Array.from(document.querySelectorAll('video'));
    return vids.map(v => ({
      src: v.src?.substring(0, 100),
      currentSrc: v.currentSrc?.substring(0, 100),
    }));
  });
  console.log('Videos:', JSON.stringify(videos, null, 2));
}

await browser.close();
