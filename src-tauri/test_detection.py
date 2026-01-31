import sys
import re
import time
from playwright.sync_api import sync_playwright

def test_url(url):
    print(f"🔍 Testing URL: {url}")

    with sync_playwright() as p:
        browser = p.chromium.launch(headless=False) # Headless=False to see what happens
        context = browser.new_context(
            user_agent="Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36"
        )
        page = context.new_page()

        # 1. Network Interception
        found_video = None
        def handle_route(route):
            req_url = route.request.url
            if ".m3u8" in req_url or ".mp4" in req_url:
                print(f"✅ Network Interception Found: {req_url}")
                nonlocal found_video
                found_video = req_url
            route.continue_()

        page.route("**/*", handle_route)

        print("🚀 Navigating...")
        try:
            page.goto(url, wait_until="domcontentloaded", timeout=60000)
        except Exception as e:
            print(f"⚠️ Navigation timeout/error: {e}")

        # 2. Source Code Inspection (The "Secret Weapon")
        print("🕵️ Inspecting Source Code for 'asset'/'medias'...")
        try:
            content = page.content()
            config_match = re.search(r'"asset"\s*:\s*"([^"]+)"', content)
            media_match = re.search(r'"medias"\s*:\s*\{[^}]*"original"\s*:\s*"([^"]+)"', content)

            if config_match and media_match:
                asset = config_match.group(1)
                media_id = media_match.group(1)
                constructed_url = f"https://{asset}/hls/{media_id}/master.m3u8"
                print(f"✅ Source Code Inspection Found Config!")
                print(f"   Asset: {asset}")
                print(f"   MediaID: {media_id}")
                print(f"   Constructed URL: {constructed_url}")
                found_video = constructed_url
            else:
                print("❌ 'asset'/'medias' config NOT found in source code.")
        except Exception as e:
            print(f"⚠️ Error inspecting source: {e}")

        # 3. Iframe Extraction (if main page failed)
        if not found_video:
            print("🕵️ Looking for iframes...")
            iframes = page.query_selector_all("iframe")
            for i, iframe in enumerate(iframes):
                src = iframe.get_attribute("src") or iframe.get_attribute("data-lazy-src")
                if src:
                    print(f"   Found iframe {i+1}: {src}")
                    if "baiwarp" in src or "vdohls" in src or "player" in src:
                        print(f"   👉 This looks like a player! You should test this URL directly.")
                        # Recursive test could go here, but let's keep it simple

        # Keep browser open for a bit
        print("⏳ Waiting 10 seconds...")
        page.wait_for_timeout(10000)
        browser.close()

if __name__ == "__main__":
    if len(sys.argv) > 1:
        test_url(sys.argv[1])
    else:
        print("Usage: python3 test_detection.py <URL>")
