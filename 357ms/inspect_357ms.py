import os
import shutil
from playwright.sync_api import sync_playwright

def inspect():
    # Setup local directories for temp files and user data
    current_dir = os.getcwd()
    local_tmp_dir = os.path.join(current_dir, "playwright_tmp")
    user_data_dir = os.path.join(current_dir, "playwright_user_data")
    
    # Ensure directories exist
    os.makedirs(local_tmp_dir, exist_ok=True)
    os.makedirs(user_data_dir, exist_ok=True)
    
    # Force Playwright to use our local tmp dir
    os.environ["TMPDIR"] = local_tmp_dir
    print(f"Set TMPDIR to: {local_tmp_dir}")
    print(f"User Data Dir: {user_data_dir}")

    with sync_playwright() as p:
        try:
            print("Launching browser...")
            # Persistent context allows saving session/cookies and avoids some permission issues
            context = p.chromium.launch_persistent_context(
                user_data_dir,
                headless=True,
                args=[
                    "--no-sandbox", 
                    "--disable-setuid-sandbox",
                    "--disable-dev-shm-usage"
                ]
            )
            
            page = context.pages[0]
            
            url = "https://www.357ms.com/watch/5905"
            print(f"Navigating to {url}...")
            # Increased timeout for slow cloudflare checks
            page.goto(url, timeout=60000, wait_until="domcontentloaded")

            print("Waiting for 10 seconds for Cloudflare/Scripts...")
            page.wait_for_timeout(10000)

            print("--- HTML DUMP START ---")
            print(f"Page Title: {page.title()}")
            
            # Look for iframes
            iframes = page.query_selector_all("iframe")
            print(f"Found {len(iframes)} iframes")
            
            for i, frame in enumerate(iframes):
                src = frame.get_attribute("src")
                lazy_src = frame.get_attribute("data-lazy-src")
                print(f"Iframe {i}: src={src}, data-lazy-src={lazy_src}")
                
                # Check for video players commonly used
                if src and ("baiwarp" in src or "m3u8" in src):
                    print(f"  [POTENTIAL TARGET] Found interesting iframe: {src}")

            # Look for video tags
            videos = page.query_selector_all("video")
            print(f"Found {len(videos)} video tags")
            for i, video in enumerate(videos):
                src = video.get_attribute("src")
                print(f"Video {i}: src={src}")

            # Try to grep for m3u8 in the content
            content = page.content()
            if ".m3u8" in content:
                print("FOUND .m3u8 string in page content!")
                # Extract it lightly
                import re
                matches = re.findall(r'(https?://[^\s"\'<>]+\.m3u8)', content)
                for m in matches:
                    print(f"  - {m}")
            else:
                print("No .m3u8 found in main page content.")
                
            # If we found a potential target iframe, let's try to navigate to it directly or get its content
            # (Note: cross-origin frames might block content access, but we can print the src)

            context.close()
            print("Done.")

        except Exception as e:
            print(f"An error occurred: {e}")
            import traceback
            traceback.print_exc()

if __name__ == "__main__":
    inspect()
