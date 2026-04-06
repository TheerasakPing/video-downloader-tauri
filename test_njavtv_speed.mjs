// Quick test to verify NjavTV doesn't hang
import { chromium } from 'playwright';

console.log('[Test] Testing NjavTV detection speed...\n');

const start = Date.now();

const browser = await chromium.launch({ 
  headless: true,
  args: ['--no-sandbox', '--disable-dev-shm-usage']
});
const context = await browser.newContext({
  userAgent: 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36',
});
const page = await context.newPage();

const testUrl = 'https://njavtv.com/th/adn-765-uncensored-leak';
console.log('[1/3] Navigating to:', testUrl);
await page.goto(testUrl, { waitUntil: 'networkidle', timeout: 60000 });
console.log(`✓ Page loaded in ${((Date.now() - start) / 1000).toFixed(1)}s`);

console.log('\n[2/3] Waiting 2 seconds for player init...');
await new Promise(r => setTimeout(r, 2000));

console.log('\n[3/3] Checking window.hls.url...');
for (let attempt = 1; attempt <= 10; attempt++) {
  const hlsUrl = await page.evaluate(() => {
    if (window.hls && window.hls.url) return window.hls.url;
    return null;
  });

  if (hlsUrl) {
    const elapsed = ((Date.now() - start) / 1000).toFixed(1);
    console.log(`✓ Found m3u8 on attempt ${attempt} (${elapsed}s):`);
    console.log(`  ${hlsUrl}`);
    console.log(`\n✅ SUCCESS! Detection completed in ${elapsed} seconds`);
    await browser.close();
    process.exit(0);
  }

  console.log(`  Attempt ${attempt}/10: not ready yet, waiting 2s...`);
  await new Promise(r => setTimeout(r, 2000));
}

const elapsed = ((Date.now() - start) / 1000).toFixed(1);
console.log(`\n❌ FAILED: Could not detect video URL in ${elapsed}s`);
await browser.close();
process.exit(1);
