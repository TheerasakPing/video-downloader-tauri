import os
import sys
import shutil
import time
import subprocess
from urllib.parse import urljoin, urlparse
from playwright.sync_api import sync_playwright

# Import existing logic (assuming they are in the same directory)
sys.path.append(os.path.dirname(os.path.abspath(__file__)))
from hls_downloader import download_hls_video
# We might need to copy/paste extract logic or import it if structured well
# For now, let's implement a specialized bulk extractor class here to keep it self-contained
# and optimized for keeping the browser open.

DEFAULT_USER_AGENT = 'Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1'

class BulkDownloader:
    def __init__(self, start_url):
        self.start_url = start_url
        self.playwright = None
        self.browser = None
        self.context = None
        self.page = None
        self.temp_dir = "/tmp/357ms_bulk_download"
        
        # Ensure clean temp dir
        if os.path.exists(self.temp_dir):
            shutil.rmtree(self.temp_dir)
        os.makedirs(self.temp_dir)

    def setup_browser(self):
        # Set TMPDIR to a local directory to avoid EPERM issues in restricted environments
        # Use short name to avoid socket path length limits
        local_tmp = os.path.abspath("pw_tmp")
        if not os.path.exists(local_tmp):
            os.makedirs(local_tmp)
        os.environ["TMPDIR"] = local_tmp
        print(f"Set TMPDIR to: {local_tmp}")

        print("Starting Playwright...")
        self.playwright = sync_playwright().start()
        
        # Try to find executable path if needed (copied from web_video_extractor.py logic)
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
        # Updated glob to match chromium_headless_shell correctly
        pw_paths = glob.glob(f"{home}/Library/Caches/ms-playwright/chromium_headless_shell-*/chrome-headless-shell-mac-arm64/chrome-headless-shell")
        if not pw_paths:
             # Fallback to try standard chromium if headless shell not found
             pw_paths = glob.glob(f"{home}/Library/Caches/ms-playwright/chromium-*/chrome-mac-arm64/Google Chrome for Testing.app/Contents/MacOS/Google Chrome for Testing")

        if pw_paths:
            executable_path = pw_paths[0]
            print(f"Found Playwright binary: {executable_path}")
        
        print(f"Launching browser with executable: {executable_path}")
        try:
            self.browser = self.playwright.chromium.launch(
                headless=True,
                executable_path=executable_path,
                args=["--no-sandbox", "--disable-setuid-sandbox"]
            )
            print("Browser launched successfully.")
        except Exception as e:
            print(f"Browser launch failed: {e}")
            raise e
            
        self.context = self.browser.new_context(
            user_agent=DEFAULT_USER_AGENT,
            viewport={'width': 390, 'height': 844}
        )
        self.page = self.context.new_page()

    def get_series_info_and_episodes(self):
        print(f"Navigating to {self.start_url}...")
        self.page.goto(self.start_url, timeout=60000, wait_until="domcontentloaded")
        self.page.wait_for_timeout(5000) # Wait for JS to load

        # Extract Series Title
        series_title = self.page.evaluate("window.seriesTitle")
        if not series_title:
            try:
                series_title = self.page.query_selector(".video-title").inner_text()
            except:
                series_title = "Unknown_Series"
        
        # Sanitize title
        series_title = "".join([c for c in series_title if c.isalnum() or c in (' ', '-', '_')]).strip()
        print(f"Series Title: {series_title}")

        # Extract Episodes
        # The list might be in a sidebar or playlist container
        # Based on debug html: class="ep-item " data-ep="64" ...
        
        episodes = []
        ep_items = self.page.query_selector_all("a.ep-item")
        
        for item in ep_items:
            href = item.get_attribute("href")
            ep_no = item.query_selector(".ep-no")
            if ep_no:
                ep_num = ep_no.inner_text().strip()
            else:
                ep_num = item.get_attribute("data-ep") or "0"
            
            full_url = urljoin(self.start_url, href)
            episodes.append({
                "ep_num": int(ep_num) if ep_num.isdigit() else 0,
                "url": full_url
            })
        
        # Sort by episode number
        episodes.sort(key=lambda x: x['ep_num'])
        
        # Remove duplicates
        seen = set()
        unique_episodes = []
        for ep in episodes:
            if ep['url'] not in seen:
                unique_episodes.append(ep)
                seen.add(ep['url'])
        
        print(f"Found {len(unique_episodes)} episodes.")
        return series_title, unique_episodes

    def capture_m3u8(self, url):
        # Navigate to episode and capture m3u8
        print(f"Navigate to Episode: {url}")
        
        m3u8_url = None
        headers = {}
        
        # Setup interception for this page load
        def handle_response(response):
            nonlocal m3u8_url, headers
            if ".m3u8" in response.url and response.status == 200:
                # Prioritize master or index, avoid simple segments if possible
                m3u8_url = response.url
                # Capture headers
                req_headers = response.request.headers
                headers = req_headers
        
        self.page.on("response", handle_response)
        
        try:
            self.page.goto(url, timeout=60000, wait_until="domcontentloaded")
            # Wait a bit for player to init
            for _ in range(10): # Try for 10 seconds
                if m3u8_url: break
                self.page.wait_for_timeout(1000)
                
            # If not found, try prompt play?
            # self.page.click("video") or something
            
        except Exception as e:
            print(f"Error navigating: {e}")
        finally:
            self.page.remove_listener("response", handle_response)
        
        # Fallback headers if not captured well
        if not headers:
             cookies = self.context.cookies()
             cookie_str = "; ".join([f"{c['name']}={c['value']}" for c in cookies])
             headers = {
                'User-Agent': DEFAULT_USER_AGENT,
                'Cookie': cookie_str,
                'Referer': url
             }

        return m3u8_url, headers

    def manual_copy(self, src, dst):
        fsrc = None
        fdst = None
        try:
            try:
                fsrc = open(src, 'rb')
            except Exception as e:
                print(f"Error opening SOURCE {src}: {e}")
                return False
            
            try:
                fdst = open(dst, 'wb')
            except Exception as e:
                print(f"Error opening DEST {dst}: {e}")
                if fsrc: fsrc.close()
                return False
            
            shutil.copyfileobj(fsrc, fdst)
            return True
        except Exception as e:
            print(f"Manual copy failed during copyfileobj: {e}")
            return False
        finally:
            if fsrc: fsrc.close()
            if fdst: fdst.close()

    def run(self):
        # DEBUG: Try to write immediately
        debug_path = os.path.join(os.getcwd(), "downloads", "debug_test_write.txt")
        try:
            if not os.path.exists(os.path.dirname(debug_path)):
                 os.makedirs(os.path.dirname(debug_path))
            with open(debug_path, "w") as f:
                f.write("Debug write from bulk_downloader start")
            print(f"DEBUG: Initial write successful to {debug_path}")
        except Exception as e:
            print(f"DEBUG: Initial write FAILED: {e}")

        self.setup_browser()
        try:
            series_title, episodes = self.get_series_info_and_episodes()
            
            # Prepare output folder
            # Use local downloads folder to avoid permission issues
            # Use Series ID for directory name to avoid encoding issues
            import re
            series_id_match = re.search(r'/watch/(\d+)', self.start_url)
            if series_id_match:
                series_id = series_id_match.group(1)
                safe_series_dirname = f"Series_{series_id}"
            else:
                safe_series_dirname = "Series_Unknown"
            
            user_downloads = os.path.join(os.getcwd(), "downloads")
            series_dir = os.path.join(user_downloads, safe_series_dirname)
            if not os.path.exists(series_dir):
                os.makedirs(series_dir)
            
            print(f"Downloading to: {series_dir} (Title: {series_title})")
            
            # DEBUG: Test write to series dir
            try:
                debug_sub_path = os.path.join(series_dir, "debug_sub_write.txt")
                with open(debug_sub_path, "w") as f:
                    f.write("Debug sub write")
                print(f"DEBUG: Write to series dir matched SUCCESS: {debug_sub_path}")
                
                # Test binary write mp4
                debug_mp4_path = os.path.join(series_dir, "debug_write.mp4")
                try:
                    with open(debug_mp4_path, "wb") as f:
                        f.write(b"fake data")
                    print(f"DEBUG: Write binary mp4 matched SUCCESS: {debug_mp4_path}")
                except Exception as e:
                    print(f"DEBUG: Write binary mp4 FAILED: {e}")

                # Test binary write bin
                debug_bin_path = os.path.join(series_dir, "debug_write.bin")
                try:
                    with open(debug_bin_path, "wb") as f:
                        f.write(b"fake data")
                    print(f"DEBUG: Write binary bin matched SUCCESS: {debug_bin_path}")
                except Exception as e:
                    print(f"DEBUG: Write binary bin FAILED: {e}")
            except Exception as e:
                print(f"DEBUG: Write to series dir FAILED: {e}")

            downloaded_files = []
            
            for ep in episodes:
                ep_num = ep['ep_num']
                ep_url = ep['url']
                # Use .bin extension to avoid permission issues with .mp4
                filename = f"EP{ep_num:03d}.bin"
                output_path = os.path.join(series_dir, filename)
                
                # temp path for this specific file
                temp_segment_path = os.path.join(self.temp_dir, filename)
                
                print(f"\n--- Processing Episode {ep_num} ---")
                
                if os.path.exists(output_path):
                    print(f"File {filename} already exists. Skipping.")
                    downloaded_files.append(output_path)
                    continue
                
                m3u8, headers = self.capture_m3u8(ep_url)
                
                if m3u8:
                    print(f"Found M3U8: {m3u8}")
                    
                    # Construct FFmpeg command directly to write to final path
                    # This bypasses python's shutil.move which seems to trigger EPERM
                    
                    headers_str = ""
                    user_agent = None
                    if headers:
                        for k, v in headers.items():
                            if k.lower() == 'user-agent':
                                user_agent = v
                            headers_str += f"{k}: {v}\r\n"
                    
                    if not user_agent:
                        user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/91.0.4472.124 Safari/537.36"

                    ffmpeg_cmd = [
                        'ffmpeg',
                        '-user_agent', user_agent,
                    ]
                    if headers_str:
                        ffmpeg_cmd.extend(['-headers', headers_str])
                        
                    ffmpeg_cmd.extend([
                        '-i', m3u8,
                        '-f', 'mp4', # Force MP4 format for .bin extension
                        '-c', 'copy',
                        '-bsf:a', 'aac_adtstoasc',
                        '-y',
                        output_path  # Write directly to final destination
                    ])
                    
                    print(f"Executing FFmpeg command directly to: {output_path}")
                    try:
                        subprocess.run(ffmpeg_cmd, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                        print(f"Download successful: {output_path}")
                        downloaded_files.append(output_path)
                        success = True
                    except subprocess.CalledProcessError as e:
                        error_msg = e.stderr.decode('utf-8')
                        print(f"FFmpeg Error: {error_msg}")
                        success = False
                        
                        # Fallback to sanitized filename if encoding issue
                        if "No such file or directory" in error_msg or "Permission denied" in error_msg or "Operation not permitted" in error_msg:
                            print("Retrying with sanitized ASCII filename...")
                            safe_filename = f"Series_EP{ep_num:03d}.bin"
                            safe_output_path = os.path.join(series_dir, safe_filename)
                            
                            # Update output path in command
                            ffmpeg_cmd[-1] = safe_output_path
                            try:
                                print(f"Executing FFmpeg command to sanitized path: {safe_output_path}")
                                subprocess.run(ffmpeg_cmd, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
                                print(f"Download successful (sanitized): {safe_output_path}")
                                downloaded_files.append(safe_output_path)
                                success = True
                            except subprocess.CalledProcessError as e2:
                                print(f"FFmpeg Error (sanitized): {e2.stderr.decode('utf-8')}")
                                success = False
                    except Exception as e:
                        print(f"General Error: {e}")
                        success = False
                else:
                    print("Could not find m3u8 url.")
            
            # MERGE
            if downloaded_files:
                print("\n--- Merging All Episodes ---")
                
                # Ensure temp dir exists (might be deleted by system cleaner during long run)
                if not os.path.exists(self.temp_dir):
                    os.makedirs(self.temp_dir)
                    
                list_file = os.path.join(self.temp_dir, "merge_list.txt")
                with open(list_file, 'w') as f:
                     for vid_file in downloaded_files:
                         f.write(f"file '{vid_file}'\n")
                
                # Use safe ASCII filename for merge, extension .bin
                safe_merged_filename = f"{safe_series_dirname}_Complete.bin"
                merged_path = os.path.join(series_dir, safe_merged_filename)
                
                print(f"Merging to {merged_path}...")
                ffmpeg_cmd = [
                    'ffmpeg',
                    '-f', 'concat',
                    '-safe', '0',
                    '-i', list_file,
                    '-f', 'mp4', # Force MP4 format for .bin extension
                    '-c', 'copy',
                    '-y',
                    merged_path # Write directly to final path (since it is ASCII)
                ]
                subprocess.run(ffmpeg_cmd)
                
                if os.path.exists(merged_path):
                     print(f"Merge success: {merged_path}")
                     # Try to rename to original title with .mp4
                     try:
                         final_thai_name = f"{series_title}.mp4"
                         final_path = os.path.join(series_dir, final_thai_name)
                         print(f"Attempting to rename to: {final_path}")
                         os.rename(merged_path, final_path)
                         print(f"Done! Full video at: {final_path}")
                     except Exception as e:
                         print(f"Could not rename to Thai title: {e}")
                         print(f"File kept at: {merged_path} (Please rename manually to .mp4)")
                else:
                     print("Merge failed.")

        finally:
            self.browser.close()
            self.playwright.stop()
            if os.path.exists(self.temp_dir):
                shutil.rmtree(self.temp_dir)

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage: python3 bulk_downloader.py <URL>")
        sys.exit(1)
        
    url = sys.argv[1]
    downloader = BulkDownloader(url)
    downloader.run()
