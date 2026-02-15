import requests
import re
import json
import sys
import os
import shutil
from bs4 import BeautifulSoup
from urllib.parse import urljoin, urlparse
from playwright.sync_api import sync_playwright

# Import download function from hls_downloader
try:
    from .hls_downloader import download_hls_video
except ImportError:
    try:
        from hls_downloader import download_hls_video
    except ImportError:
        # Fallback if file not in same dir (though usually it is in this project)
        print("ไม่พบไฟล์ hls_downloader.py กรุณาตรวจสอบว่าไฟล์ดังกล่าวอยู่ในโฟลเดอร์เดียวกัน")
        sys.exit(1)

def extract_with_playwright(url):
    """
    Extensions to extract video URL using Playwright
    """
    print(f"Attempting extraction with Playwright for: {url}")
    
    # Setup local directories for temp files and user data to avoid permission issues
    base_cache_dir = "/tmp/rongyok_cache"
    local_tmp_dir = os.path.join(base_cache_dir, "playwright_tmp")
    user_data_dir = os.path.join(base_cache_dir, "playwright_user_data")
    
    os.makedirs(local_tmp_dir, exist_ok=True)
    os.makedirs(user_data_dir, exist_ok=True)
    
    # Force Playwright to use our local tmp dir
    os.environ["TMPDIR"] = local_tmp_dir
    
    m3u8_url = None
    headers = None
    
    try:
        with sync_playwright() as p:
            # Try to find existing executable to bypass version check issues
            executable_path = None
            potential_paths = [
                os.path.expanduser("~/Library/Caches/ms-playwright/chromium_headless_shell-1208/chrome-headless-shell-mac-arm64/chrome-headless-shell"),
                os.path.expanduser("~/Library/Caches/ms-playwright/chromium-1208/chrome-mac-arm64/Chromium.app/Contents/MacOS/Chromium"),
                os.path.expanduser("~/Library/Caches/ms-playwright/chromium-1148/chrome-mac/Chromium.app/Contents/MacOS/Chromium"),
            ]
            for p_path in potential_paths:
                if os.path.exists(p_path):
                    executable_path = p_path
                    print(f"Using custom executable path: {executable_path}")
                    break
            
            # Persistent context with mobile user agent to encourage HLS
            context = p.chromium.launch_persistent_context(
                user_data_dir,
                headless=True,
                executable_path=executable_path,
                user_agent='Mozilla/5.0 (iPhone; CPU iPhone OS 17_0 like Mac OS X) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/17.0 Mobile/15E148 Safari/604.1',
                viewport={'width': 390, 'height': 844},
                args=[
                    "--no-sandbox", 
                    "--disable-setuid-sandbox", 
                    "--disable-dev-shm-usage"
                ]
            )
            
            page = context.pages[0]
            
            # Intercept network requests to find m3u8
            # We use a list to store found URLs since inner function can't easily assign to outer local var in python 2/3 compat way without nonlocal
            found_urls = []
            def handle_response(response):
                if ".m3u8" in response.url and response.status == 200:
                    print(f"Captured m3u8 URL from network: {response.url}")
                    found_urls.append(response.url)
            
            page.on("response", handle_response)
            
            try:
                print(f"Navigating to {url}...")
                page.goto(url, timeout=60000, wait_until="domcontentloaded")
                
                # Wait for potential cloudflare or initial scripts
                page.wait_for_timeout(8000)
                
                # Check if we found it via network
                if found_urls:
                    m3u8_url = found_urls[0]
                    print(f"Using network-captured URL: {m3u8_url}")
                
                # Check for iframes content (fallback)
                if not m3u8_url:
                    iframes = page.query_selector_all("iframe")
                    print(f"Found {len(iframes)} iframes")
                    for frame in iframes:
                        src = frame.get_attribute("src") or frame.get_attribute("data-lazy-src")
                        print(f"Iframe src: {src}")
                        if src and "baiwarp" in src:
                            print(f"Found Baiwarp iframe: {src}")
                            
                            # Navigate to the iframe URL explicitly to let it load and run its JS
                            if src.startswith("//"):
                                src = "https:" + src
                                
                            print(f"Navigating to iframe src: {src}")
                            page.goto(src, timeout=60000, wait_until="networkidle")
                            
                            # Now inspect the content of the iframe page (which is now the main page)
                            content = page.content()
                            
                            # Look for playerConfig
                            config_match = re.search(r'window\.playerConfig\s*=\s*(\{.*?\});', content)
                            if config_match:
                                try:
                                    config_data = json.loads(config_match.group(1))
                                    if 'medias' in config_data and 'asset' in config_data:
                                        asset_domain = config_data.get('asset')
                                        media_id = config_data['medias'].get('original')
                                        if asset_domain and media_id:
                                            m3u8_url = f"https://{asset_domain}/{media_id}/video.m3u8"
                                            print(f"Found URL from playerConfig: {m3u8_url}")
                                            break
                                except Exception as e:
                                    print(f"Error parsing playerConfig: {e}")
                            
                            # Fallback: Regex scan
                            if not m3u8_url:
                                matches = re.findall(r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']', content)
                                if matches:
                                    m3u8_url = matches[0].replace('\\/', '/')
                                    print(f"Found URL via regex in iframe: {m3u8_url}")
                                    break
                        
                        if m3u8_url: break
                
                # If still not found, scan the main page content again (maybe it wasn't in an iframe or we are already there)
                if not m3u8_url:
                    content = page.content()
                    matches = re.findall(r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']', content)
                    if matches:
                         m3u8_url = matches[0].replace('\\/', '/')
                         print(f"Found URL via regex in main page: {m3u8_url}")

                # Extract cookies and user agent for requests
                if m3u8_url:
                    cookies = context.cookies()
                    cookie_str = "; ".join([f"{c['name']}={c['value']}" for c in cookies])
                    ua = page.evaluate("navigator.userAgent")
                    parsed_url = urlparse(page.url)
                    headers = {
                        'User-Agent': ua,
                        'Cookie': cookie_str,
                        'Referer': page.url,
                        'Origin': f"{parsed_url.scheme}://{parsed_url.netloc}"
                    }
                    print("Extracted headers and cookies from Playwright session")

            except Exception as e:
                print(f"Playwright navigation error: {e}")
            finally:
                context.close()
                
    except Exception as e:
        print(f"Playwright initialization error: {e}")
        
    return m3u8_url, headers

def extract_and_download(page_url, only_return_url=False):
    """
    ดึงลิงก์ .m3u8 จากหน้าเว็บและดาวน์โหลด
    :param page_url: URL ของหน้าเว็บ
    :param only_return_url: ถ้า True จะคืนค่า URL กลับไปแทนการดาวน์โหลดทันที
    :return: (success_bool, m3u8_url_or_None)
    """
    print(f"กำลังวิเคราะห์หน้าเว็บ: {page_url}")
    
    headers = {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
        'Referer': page_url,
        'Accept': 'text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,image/apng,*/*;q=0.8',
        'Accept-Language': 'en-US,en;q=0.9,th;q=0.8'
    }

    m3u8_url = None

    # Method 1: Requests (Fast, but might fail due to Cloudflare)
    try:
        response = requests.get(page_url, headers=headers, timeout=15)
        response.raise_for_status()
        html_content = response.text
        
        soup = BeautifulSoup(html_content, 'html.parser')
        
        # Check iframe src first (common for embedded players)
        if not m3u8_url:
            iframes = soup.find_all('iframe')
            for iframe in iframes:
                src = iframe.get('src') or iframe.get('data-lazy-src')
                if src:
                    src = str(src)
                    print(f"เจอ iframe: {src}")
                    if 'play.baiwarp.com/embed/' in src:
                        # try to get relative link
                        
                        try:
                            # Handle relative protocol
                            if src.startswith('//'):
                                src = 'https:' + src
                                
                            print(f"กำลังเข้าไปดึงข้อมูลจาก iframe: {src}")
                            iframe_resp = requests.get(src, headers=headers, timeout=10)
                            iframe_html = iframe_resp.text
                            
                            # Strategy 4: Extract from window.playerConfig
                            config_match = re.search(r'window\.playerConfig\s*=\s*(\{.*?\});', iframe_html)
                            if config_match:
                                try:
                                    config_json_str = config_match.group(1)
                                    config_data = json.loads(config_json_str)
                                    if 'medias' in config_data and 'asset' in config_data:
                                        asset_domain = config_data.get('asset')
                                        media_id = config_data['medias'].get('original')
                                        if asset_domain and media_id:
                                            m3u8_url = f"https://{asset_domain}/{media_id}/video.m3u8"
                                            print(f"เจอข้อมูลใน playerConfig: {m3u8_url}")
                                except Exception as e:
                                    print(f"แกะ playerConfig ไม่สำเร็จ: {e}")

                            # Fallbacks regex
                            if not m3u8_url:
                                iframe_patterns = [
                                    r'file:\s*[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']',
                                    r'source:\s*[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']',
                                    r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']'
                                ]
                                for pattern in iframe_patterns:
                                    matches = re.findall(pattern, iframe_html)
                                    if matches:
                                        m3u8_url = matches[0].replace('\\/', '/')
                                        break
                                        
                        except Exception as e:
                            print(f"ไม่สามารถดึงข้อมูลจาก iframe ได้ (อาจติด Cloudflare): {e}")

                    if m3u8_url: break

        # Check <source> tags
        if not m3u8_url:
            sources = soup.find_all('source')
            for source in sources:
                src = source.get('src')
                if src and '.m3u8' in src:
                    m3u8_url = src
                    break
        
        # Check <video> tags
        if not m3u8_url:
            videos = soup.find_all('video')
            for video in videos:
                src = video.get('src')
                if src and '.m3u8' in src:
                    m3u8_url = src
                    break

        # Regex on main page
        if not m3u8_url:
             patterns = [
                r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']',
                r'(https?://[^\"\s]*?\.m3u8[^\"\s]*)'
            ]
             for pattern in patterns:
                matches = re.findall(pattern, html_content)
                if matches:
                    valid = [m for m in matches if '.m3u8' in m]
                    if valid:
                        m3u8_url = valid[0].replace('\\/', '/')
                        break

    except Exception as e:
        print(f"Requests-based extraction failed or encountered error: {e}")

    # Method 2: Playwright (Fallback)
    if not m3u8_url:
        print("ไม่พบ URL ด้วยวิธีปกติ กำลังลองใช้ Playwright...")
        try:
             m3u8_url, pw_headers = extract_with_playwright(page_url)
             if pw_headers:
                 headers.update(pw_headers)
        except ValueError:
             # Handle case where it might strip return single value if something goes wrong or legacy
             res = extract_with_playwright(page_url)
             if isinstance(res, tuple):
                 m3u8_url, pw_headers = res
                 if pw_headers: headers.update(pw_headers)
             else:
                 m3u8_url = res

    if m3u8_url:
        m3u8_url = str(m3u8_url)
        if not m3u8_url.startswith('http'):
            m3u8_url = urljoin(page_url, m3u8_url)
        
        print(f"พบลิงก์วิดีโอสำเร็จ: {m3u8_url}")
        print("-" * 30)
        
        if only_return_url:
            print(f"คืนค่า URL แทนการดาวน์โหลด: {m3u8_url}")
            return True, m3u8_url

        success = download_hls_video(m3u8_url, headers=headers)
        return success, m3u8_url
    else:
        print("ไม่พบลิงก์ .m3u8 แม้จะใช้ Playwright แล้ว")
        return False, None

if __name__ == "__main__":
    if len(sys.argv) > 1:
        url = sys.argv[1]
    else:
        # Default test URL if none provided
        url = "https://media.vdohls.com/R48Ss-m5w_Tea/video.m3u8" 
    
    print(f"ทดสอบกับ Input: {url}")
    
    if url.endswith('.m3u8'):
        download_hls_video(url)
    else:
        extract_and_download(url)
