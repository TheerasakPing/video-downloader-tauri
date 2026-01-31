import requests
import re
import json
import sys
import os
from bs4 import BeautifulSoup
from urllib.parse import urljoin

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

    try:
        response = requests.get(page_url, headers=headers, timeout=15)
        response.raise_for_status()
        html_content = response.text
        
        # 1. ค้นหาด้วย BeautifulSoup (หาใน tag video/source)
        soup = BeautifulSoup(html_content, 'html.parser')
        m3u8_url = None
        
        # Check iframe src first (common for embedded players)
        if not m3u8_url:
            iframes = soup.find_all('iframe')
            for iframe in iframes:
                src = iframe.get('src') or iframe.get('data-lazy-src')
                if src:
                    # Convert to string to avoid Pyrefly error if it's a list or BS4 object
                    src = str(src)
                    print(f"เจอ iframe: {src}")
                    # If iframe is from play.baiwarp.com, it might contain the ID
                    if 'play.baiwarp.com/embed/' in src:
                        # Extract ID
                        video_id = src.split('embed/')[-1]
                        # Construct potential m3u8 or player URL
                        print(f"เจอ Baiwarp ID: {video_id}")
                        # Note: This might require another request to the iframe URL or constructing a known m3u8 pattern
                        # But typically we need to fetch the iframe content first.
                        # Let's try to fetch the iframe content if it's a relative link or full http link
                        
                        # In the observed HTML, it's https://play.baiwarp.com/embed/fxCsM-7_5OIjk
                        
                        try:
                            print(f"กำลังเข้าไปดึงข้อมูลจาก iframe: {src}")
                            iframe_resp = requests.get(src, headers=headers, timeout=10)
                            iframe_html = iframe_resp.text
                            
                            # Search inside iframe content
                            iframe_patterns = [
                                r'file:\s*[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']',
                                r'source:\s*[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']',
                                r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']'
                            ]
                            
                            for pattern in iframe_patterns:
                                matches = re.findall(pattern, iframe_html)
                                if matches:
                                    m3u8_url = matches[0].replace('\\/', '/')
                                    print(f"เจอ .m3u8 ใน iframe: {m3u8_url}")
                                    break
                            
                            # If no m3u8 found but we have a packed JS or something, might need more decoding
                            # For Baiwarp specifically, it often uses packed code or hidden m3u8
                            if not m3u8_url and 'baiwarp' in src:
                                # Try to find file: "..." pattern which is common in their player
                                file_pattern = r'file:\s*["\']([^"\']+\.m3u8[^"\']*)["\']'
                                file_match = re.search(file_pattern, iframe_html)
                                if file_match:
                                    m3u8_url = file_match.group(1).replace('\\/', '/')
                                    print(f"เจอ .m3u8 ใน iframe (pattern2): {m3u8_url}")
                                
                                # Try generic "https....m3u8"
                                if not m3u8_url:
                                    all_m3u8 = re.findall(r'https?://[^\s"\']+\.m3u8[^\s"\']*', iframe_html)
                                    if all_m3u8:
                                        m3u8_url = all_m3u8[0].replace('\\/', '/')
                                        print(f"เจอ .m3u8 ใน iframe (generic regex): {m3u8_url}")
                                
                                # Try unpacking or looking for simple concatenation if m3u8 is not obvious
                                if not m3u8_url:
                                    # Example: + "vdohls.com/" + ...
                                    if 'vdohls.com' in iframe_html or 'm3u8' in iframe_html:
                                        print("อาจมีการซ่อนลิงก์ด้วย JavaScript (Packer/Obfuscator)")
                                        # Strategy 3: Check for specific Baiwarp pattern where ID is passed to a player function
                                        # or constructed via variables.
                                        # In many cases, the ID in the iframe src IS the key to the m3u8.
                                        # Let's try to CONSTRUCT the m3u8 url from the ID if we have it
                                        
                                        if 'video_id' in locals() and video_id:
                                            # Common patterns for this host (derived from knowledge or inspection)
                                            # https://media.vdohls.com/{video_id}/video.m3u8
                                            potential_url = f"https://media.vdohls.com/{video_id}/video.m3u8"
                                            print(f"ลองสร้าง URL จาก ID: {potential_url}")
                                            
                                            # Verify if it exists
                                            try:
                                                # Use verify=False to bypass SSL issues if any
                                                print(f"กำลังตรวจสอบ URL (index): {potential_url_index}")
                                                head_check = requests.head(potential_url_index, headers=headers, timeout=5, verify=False)
                                                if head_check.status_code == 200:
                                                    m3u8_url = potential_url_index
                                                    print(f"ยืนยัน URL (index) ถูกต้อง: {m3u8_url}")
                                            except:
                                                pass
                                        
                                        # Another pattern: sometimes the video_id needs to be used in a different subdomain or path
                                        if not m3u8_url and 'video_id' in locals() and video_id:
                                            # Try without 'video.m3u8' suffix, sometimes it redirects
                                            # Or maybe it's just hls/video.m3u8
                                            pass

                                        # Fallback to inspecting the iframe src if it looks like a direct video link already
                                        if not m3u8_url and 'm3u8' in src:
                                            m3u8_url = src
                                            print(f"ใช้ URL จาก src โดยตรง: {m3u8_url}")

                                        if not m3u8_url:
                                            # Simple extraction of anything that looks like a path
                                            # Looking for path segments often found in m3u8 URLs
                                            path_match = re.search(r'["\']([a-zA-Z0-9_\-]+/video\.m3u8)["\']', iframe_html)
                                            if path_match:
                                                # Construct URL if we found the path but not the domain
                                                m3u8_url = "https://media.vdohls.com/" + path_match.group(1)
                                                print(f"สร้าง URL จาก Path ที่พบ: {m3u8_url}")

                        except Exception as e:
                            print(f"ไม่สามารถดึงข้อมูลจาก iframe ได้: {e}")

                    if m3u8_url: break

        # Check <source> tags
        sources = soup.find_all('source')
        for source in sources:
            src = source.get('src')
            if src and '.m3u8' in src:
                m3u8_url = src
                print(f"เจอ .m3u8 ใน tag <source>: {m3u8_url}")
                break
        
        # Check <video> tags if not found
        if not m3u8_url:
            videos = soup.find_all('video')
            for video in videos:
                src = video.get('src')
                if src and '.m3u8' in src:
                    m3u8_url = src
                    print(f"เจอ .m3u8 ใน tag <video>: {m3u8_url}")
                    break

        # 2. ค้นหาด้วย Regex (ค้นหาใน JavaScript variables หรือ hidden text)
        if not m3u8_url:
            print("ไม่พบใน tag HTML ปกติ กำลังค้นหาใน Source code ด้วย Regex...")
            # Regex pattern to find http(s) links ending with .m3u8
            # Patterns: "url": "...", 'url': '...', src: "...", etc.
            # Looking for any string starting with http and ending with .m3u8 inside quotes
            patterns = [
                r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']',
                r'(https?://[^\"\s]*?\.m3u8[^\"\s]*)' # Less strict, might catch plain text
            ]
            
            for pattern in patterns:
                matches = re.findall(pattern, html_content)
                if matches:
                    # Filter out obviously wrong matches if needed
                    valid_matches = [m for m in matches if '.m3u8' in m]
                    if valid_matches:
                        m3u8_url = valid_matches[0]
                        # Handle escaped slashes like http:\/\/
                        m3u8_url = m3u8_url.replace('\\/', '/')
                        print(f"เจอ .m3u8 ด้วย Regex: {m3u8_url}")
                        break
        
        if m3u8_url:
            # Convert m3u8_url to string if it's a NavigableString or other BS4 object
            m3u8_url = str(m3u8_url)
            
            # Handle relative URLs if necessary
            if not m3u8_url.startswith('http'):
                m3u8_url = urljoin(page_url, m3u8_url)
                print(f"แปลง Relative URL เป็น: {m3u8_url}")
            
            print(f"พบลิงก์วิดีโอ: {m3u8_url}")
            print("-" * 30)
            
            if only_return_url:
                print(f"คืนค่า URL แทนการดาวน์โหลด: {m3u8_url}")
                return True, m3u8_url

            # เรียกใช้ฟังก์ชันดาวน์โหลด
            success = download_hls_video(m3u8_url)
            
            if success:
                print("กระบวนการเสร็จสมบูรณ์")
                return True, m3u8_url
            else:
                print("ดาวน์โหลดไม่สำเร็จ")
                return False, m3u8_url
        else:
            print("ไม่พบลิงก์ .m3u8 ในหน้าเว็บนี้")
            print("ข้อแนะนำ: เว็บอาจโหลดวิดีโอด้วย JavaScript ภายหลัง หรือมีการเข้ารหัสลิงก์ซับซ้อน")
            return False, None
            
    except requests.RequestException as e:
        print(f"เกิดข้อผิดพลาดในการเข้าถึงหน้าเว็บ: {e}")
        return False, None
    except Exception as e:
        print(f"เกิดข้อผิดพลาดที่ไม่คาดคิด: {e}")
        return False, None

if __name__ == "__main__":
    # Test with a known URL containing m3u8 (or the one provided if it was a page, but it's a direct m3u8 link)
    # The user provided a direct m3u8 link in the prompt: https://media.vdohls.com/R48Ss-m5w_Tea/video.m3u8
    # But this script is designed to extract from a *page*.
    # Let's test with a direct m3u8 link handling as well.
    
    test_input = "https://media.vdohls.com/R48Ss-m5w_Tea/video.m3u8" 
    
    print(f"ทดสอบกับ Input: {test_input}")
    
    if test_input.endswith('.m3u8'):
        print("Input เป็นลิงก์ .m3u8 โดยตรง ส่งไปดาวน์โหลดทันที...")
        download_hls_video(test_input)
    else:
        extract_and_download(test_input)
