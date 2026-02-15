import sys
import os
import json
import argparse
import re
from urllib.parse import urljoin
from playwright.sync_api import sync_playwright

# Redirect standard output to stderr to keep stdout clean for JSON output
original_stdout = sys.stdout
sys.stdout = sys.stderr

# Force local temp dir to avoid permission issues
base_cache_dir = "/tmp/rongyok_cache"
local_tmp_dir = os.path.join(base_cache_dir, "playwright_tmp")
os.makedirs(local_tmp_dir, exist_ok=True)
os.environ["TMPDIR"] = local_tmp_dir

# Ensure we can import from the same directory
current_dir = os.path.dirname(os.path.abspath(__file__))
sys.path.append(current_dir)

try:
    from web_video_extractor import extract_and_download
except ImportError:
    # Fallback or error if web_video_extractor is not found
    sys.stderr.write("Error: web_video_extractor module not found.\n")
    # We might implement fallback logic here if needed, but for now we rely on it.

DEFAULT_USER_AGENT = 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1'

def get_playwright_executable():
    executable_path = None
    possible_paths = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Microsoft Edge.app/Contents/MacOS/Microsoft Edge",
        "/usr/bin/google-chrome",
        "/usr/bin/chromium-browser",
    ]
    # Check for Playwright's own cache
    import glob
    home = os.path.expanduser("~")
    pw_paths = glob.glob(f"{home}/Library/Caches/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-mac-arm64/chrome-headless-shell")
    if not pw_paths:
         # Fallback to try standard chromium if headless shell not found
         pw_paths = glob.glob(f"{home}/Library/Caches/ms-playwright/chromium-*/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing")

    if pw_paths:
        executable_path = pw_paths[0]
    
    return executable_path

def fetch_series_info(url):
    print(f"Fetching series info for: {url}")
    
    series_info = {
        "title": "Unknown Series",
        "cover_url": None,
        "episodes": []
    }

    try:
        with sync_playwright() as p:
            executable_path = get_playwright_executable()
            print(f"Launching browser with: {executable_path}")
            
            # Setup user data dir to avoid permission issues
            # Use /tmp as home dir might be restricted in some environments
            base_cache_dir = "/tmp/rongyok_cache"
            user_data_dir = os.path.join(base_cache_dir, "playwright_357ms_context")
            os.makedirs(user_data_dir, exist_ok=True)

            context = p.chromium.launch_persistent_context(
                user_data_dir,
                headless=True,
                executable_path=executable_path,
                args=["--no-sandbox", "--disable-setuid-sandbox"],
                user_agent=DEFAULT_USER_AGENT,
                viewport={'width': 390, 'height': 844}
            )
            browser = context # Alias for finally block close
            page = context.new_page()
            
            try:
                page.goto(url, timeout=60000, wait_until="domcontentloaded")
                page.wait_for_timeout(5000) # Wait for JS

                # Extract Series Title
                series_title = None
                try:
                    series_title = page.query_selector("h1").inner_text()
                except:
                    pass

                if not series_title:
                     # Fallback
                     series_title = page.evaluate("window.seriesTitle")
                
                if series_title:
                     series_info["title"] = series_title.strip()

                # Extract Episodes
                ep_items = page.query_selector_all("a.ep-card")
                episodes = []
                
                for item in ep_items:
                    href = item.get_attribute("href")
                    if not href:
                        continue
                        
                    if not href.startswith("http"):
                         base_url = "https://www.357ms.com"
                         href = base_url + href

                    ep_no_el = item.query_selector(".ep-number")
                    if ep_no_el:
                        ep_text = ep_no_el.inner_text().strip()
                        ep_text = ep_text.replace("EP.", "").strip()
                    else:
                        ep_text = item.get_attribute("data-ep") or "0"
                    
                    full_url = urljoin(url, href)
                    
                    # Try to parse integer episode number
                    try:
                        ep_num = int(ep_text)
                    except ValueError:
                        ep_num = 0
                        
                    episodes.append({
                        "number": ep_num,
                        "title": ep_text, # Use the number as title for now, or look for more info
                        "url": full_url
                    })
                
                # Sort and Dedup
                episodes.sort(key=lambda x: x['number'])
                seen = set()
                unique_episodes = []
                for ep in episodes:
                    if ep['url'] not in seen:
                        unique_episodes.append(ep)
                        seen.add(ep['url'])
                
                series_info["episodes"] = unique_episodes
                print(f"Found {len(unique_episodes)} episodes")

            finally:
                browser.close()

    except Exception as e:
        print(f"Error fetching series info: {e}")
        # Return what we have, or empty if critical failure
        return {"error": str(e)}

    return series_info

def extract_video_info(url):
    print(f"Extracting video info for: {url}")
    # Use existing extract_and_download logic via import
    try:
        # extract_and_download prints to stdout (which is now stderr), returns (success, m3u8_url)
        # Note: headers are not returned by extract_and_download in the current version of web_video_extractor
        # I might need to modify web_video_extractor to return headers or re-implement here.
        # Let's check web_video_extractor.py content again.
        # It calls `extract_with_playwright` which returns headers.
        # But `extract_and_download` returns `success, m3u8_url`. It swallows headers?
        # Let's re-read web_video_extractor.py quickly.
        # Line 314: return success, m3u8_url.
        # Yes, headers are swallowed.
        # But download_hls_video uses them.
        # If I need headers for Rust, I should modify web_video_extractor or use a custom version here.
        # Given I have Playwright here, I can just use `extract_with_playwright` from web_video_extractor directly?
        # No, `extract_with_playwright` is internal to that script (but python defs are public).
        # Let's try importing it.
        from web_video_extractor import extract_with_playwright, extract_and_download
        
        # Try extract_with_playwright first as it gives headers
        try:
             res = extract_with_playwright(url)
             if isinstance(res, tuple):
                 m3u8, headers = res
                 if m3u8:
                     return {
                         "m3u8_url": m3u8,
                         "headers": headers or {}
                     }
        except Exception as e:
            print(f"Direct Playwright extraction failed: {e}")
        
        # Fallback to generic extract_and_download capability (regex etc)
        success, m3u8 = extract_and_download(url, only_return_url=True)
        if success and m3u8:
            return {
                "m3u8_url": m3u8,
                "headers": {} # Headers might be missing if using requests regex fallback
            }
        
    except Exception as e:
        print(f"Error extracting video: {e}")
        return {"error": str(e)}
        
    return {"error": "Video not found"}

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("mode", choices=["series", "video"], help="Mode of operation")
    parser.add_argument("url", help="Target URL")
    args = parser.parse_args()

    result = {}
    if args.mode == "series":
        result = fetch_series_info(args.url)
    elif args.mode == "video":
        result = extract_video_info(args.url)
    
    # Write JSON to the original stdout
    print(json.dumps(result), file=original_stdout)
