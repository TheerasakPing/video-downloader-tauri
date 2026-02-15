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
    
    video_info = {
        "m3u8_url": None,
        "headers": {},
        "key_url": None,
        "key_response_status": None
    }

    try:
        with sync_playwright() as p:
            executable_path = get_playwright_executable()
            print(f"Launching browser with: {executable_path}")
            
            # Setup user data dir
            base_cache_dir = "/tmp/rongyok_cache"
            user_data_dir = os.path.join(base_cache_dir, "playwright_357ms_video_context")
            os.makedirs(user_data_dir, exist_ok=True)

            context = p.chromium.launch_persistent_context(
                user_data_dir,
                headless=True,
                executable_path=executable_path,
                args=["--no-sandbox", "--disable-setuid-sandbox"],
                user_agent=DEFAULT_USER_AGENT,
                viewport={'width': 390, 'height': 844}
            )
            
            page = context.new_page()

            # Network logging
            def handle_request(route, request):
                # Continue all requests
                route.continue_()

            def handle_response(response):
                url = response.url
                status = response.status
                
                # Ignore common resources
                if any(ext in url for ext in [".png", ".jpg", ".jpeg", ".gif", ".css", ".woff", ".svg"]):
                    return

                print(f"Response: {status} {url}")
                
                if ".m3u8" in url:
                    print(f"Found m3u8: {url}")
                    video_info["m3u8_url"] = url
                    req = response.request
                    headers = req.headers
                    video_info["headers"] = {
                        "User-Agent": headers.get("user-agent", DEFAULT_USER_AGENT),
                        "Referer": headers.get("referer", url),
                        "Cookie": headers.get("cookie", "")
                    }
                
                if "api/v1/hls/config" in url:
                    print(f"Found CONFIG API: {url}")
                    try:
                        print(f"Config Body: {response.text()}")
                    except:
                        pass

                # Check for potential key response (small binary or text)
                # Key is usually 16 bytes.
                try:
                    # Only check body for small files or specific types
                    if "text" in response.headers.get("content-type", "") or "application/octet-stream" in response.headers.get("content-type", "") or "application/json" in response.headers.get("content-type", ""):
                        body_len = 0
                        try:
                            body = response.body()
                            body_len = len(body)
                        except:
                            pass
                        
                        if body_len == 16:
                            print(f"POTENTIAL KEY FOUND (16 bytes): {url}")
                            print(f"Key Hex: {body.hex()}")
                            video_info["key_url"] = url
                            video_info["key_hex"] = body.hex()
                            
                except Exception as e:
                    pass

                if ".key" in url or "fake.key" in url:
                    print(f"Explicit Key URL: {url} Status: {status}")


            # Enable request interception if needed, but for now just events
            page.on("response", handle_response)
            
            try:
                page.goto(url, timeout=60000, wait_until="domcontentloaded")
                # Wait for video to load/play. 
                # 357ms might need a click or just wait.
                # Usually m3u8 is requested automatically.
                page.wait_for_timeout(10000) 
                
                # Check if we got m3u8
                if not video_info["m3u8_url"]:
                    print("m3u8 not captured yet, trying to click play...")
                    # Try clicking video overlay if exists
                    try:
                        page.click(".vjs-big-play-button", timeout=2000)
                        page.wait_for_timeout(5000)
                    except:
                        pass
                
                # If we have m3u8 but no key yet, wait a bit more
                if video_info["m3u8_url"] and not video_info.get("key_hex"):
                    print("Waiting for key...")
                    page.wait_for_timeout(5000)

            finally:
                context.close()
                
    except Exception as e:
        print(f"Error extracting video: {e}")
        return {"error": str(e)}

    return video_info

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
