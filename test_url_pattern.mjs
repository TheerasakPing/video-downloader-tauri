// Test NjavTV URL patterns
const url = "https://njavtv.com/th/dass-812-uncensored-leak";

console.log("Testing URL pattern matching...");
console.log("URL:", url);

// Check if it matches njavtv pattern
const isNjavtv = url.includes("njavtv.com");
console.log("Is NjavTV URL:", isNjavtv);

// The Chrome detector should handle this URL
// Let's trace what happens:
// 1. NjavtvParser::get_series_info() is called
// 2. It tries to fetch the page - may get Cloudflare
// 3. Since it's a single video page, it returns direct_page_url = Some(url)
// 4. Chrome detector is called with the URL
// 5. Chrome detector detects NjavTV and polls window.hls.url

console.log("\nExpected flow:");
console.log("1. Parser returns NjavtvSeriesInfo with:");
console.log("   - total_episodes: 1");
console.log("   - direct_page_url: Some('https://njavtv.com/th/dass-812-uncensored-leak')");
console.log("2. Chrome detector is called with the page URL");
console.log("3. Chrome detector detects njavtv.com and polls window.hls.url");
console.log("4. Should return: https://surrit.com/{id}/playlist.m3u8");

// Let's also check the exact URL structure
const urlParts = url.split('/');
console.log("\nURL parts:", urlParts);
console.log("Path:", urlParts.slice(3).join('/'));
