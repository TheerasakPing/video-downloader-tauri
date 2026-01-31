#!/usr/bin/env python3
"""
Script ดาวน์โหลดวิดีโอจากเว็บ xn--82c7abb4jua0l.com
วิธีใช้: python download_video.py <URL ของหน้าวิดีโอ>
ตัวอย่าง: python download_video.py "https://xn--82c7abb4jua0l.com/หลักสูตรของครูเริ่มต้นแล้ว/"

ต้องการ: pip install playwright selenium requests
ถ้าใช้ Playwright: playwright install chromium
"""

import sys
import re
import json
import subprocess
import os
import requests
from bs4 import BeautifulSoup
from urllib.parse import urljoin, urlparse

# เลือก browser automation method
USE_PLAYWRIGHT = True  # เปลี่ยนเป็น False ถ้าอยากใช้ Selenium

def extract_video_url_lightweight(page_url):
    """
    ดึง Video URL ด้วย requests + BeautifulSoup (เร็วกว่า ไม่ต้องเปิด browser)
    """
    print(f"⚡ กำลังดึง video URL แบบ Lightweight: {page_url}")
    
    headers = {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
    }
    
    try:
        # 1. ดึงหน้าเว็บหลัก เพื่อหา iframe
        response = requests.get(page_url, headers=headers, timeout=15)
        response.raise_for_status()
        soup = BeautifulSoup(response.text, 'html.parser')
        
        iframe_src = None
        
        # หา iframe
        # Pattern 1: <iframe src="...">
        iframes = soup.find_all('iframe')
        for iframe in iframes:
            src = iframe.get('src') or iframe.get('data-lazy-src')
            if src and ('baiwarp' in src or 'play.baiwarp.com' in src):
                iframe_src = src
                break
        
        if not iframe_src:
             print("⚠️  ไม่พบ iframe เป้าหมายในหน้าแรก (Lightweight mode)")
             return None
             
        print(f"🔗 พบ iframe: {iframe_src}")
        
        # 2. เข้าไปดึงข้อมูลใน iframe
        iframe_resp = requests.get(iframe_src, headers=headers, timeout=15)
        iframe_resp.raise_for_status()
        iframe_html = iframe_resp.text
        
        # 3. แกะ window.playerConfig
        # Pattern: window.playerConfig = {...};
        config_match = re.search(r'window\.playerConfig\s*=\s*(\{.*?\});', iframe_html)
        if config_match:
            try:
                config_json_str = config_match.group(1)
                config_data = json.loads(config_json_str)
                
                if 'medias' in config_data and 'asset' in config_data:
                    asset_domain = config_data.get('asset')
                    media_id = config_data['medias'].get('original')
                    
                    if asset_domain and media_id:
                        final_url = f"https://{asset_domain}/{media_id}/video.m3u8"
                        print(f"✅ เจอ video URL (Config): {final_url}")
                        return final_url
            except Exception as e:
                print(f"⚠️  แกะ JSON config ไม่สำเร็จ: {e}")
        
        # 4. Fallback: หา .m3u8 ใน source code โดยตรง
        m3u8_matches = re.findall(r'["\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']', iframe_html)
        if m3u8_matches:
            final_url = m3u8_matches[0].replace('\\/', '/')
            print(f"✅ เจอ video URL (Regex): {final_url}")
            return final_url
            
        print("⚠️  ไม่พบข้อมูลวิดีโอใน iframe (Lightweight mode)")
        return None

    except Exception as e:
        print(f"❌ เกิดข้อผิดพลาดแบบ Lightweight: {e}")
        return None


if USE_PLAYWRIGHT:
    try:
        from playwright.sync_api import sync_playwright
    except ImportError:
        print("⚠️  ไม่มี Playwright ติดตั้งอยู่ ลอง: pip install playwright && playwright install chromium")
        print("🔄 หรือตั้งค่า USE_PLAYWRIGHT = False เพื่อใช้ Selenium")
        sys.exit(1)
else:
    try:
        from selenium import webdriver
        from selenium.webdriver.chrome.service import Service
        from selenium.webdriver.chrome.options import Options
    except ImportError:
        print("⚠️  ไม่มี Selenium ติดตั้งอยู่ ลอง: pip install selenium")
        sys.exit(1)


def extract_video_url_with_playwright(page_url):
    """
    ใช้ Playwright ดึง video URL จากหน้า embed
    """
    print(f"🔍 กำลังดึง video URL ด้วย Playwright: {page_url}")

    video_url = None

    with sync_playwright() as p:
        # เปิด browser (headless = True เพื่อไม่ให้แสดงหน้าต่าง)
        browser = p.chromium.launch(headless=True)

        # Intercept network request เพื่อดึง video URL
        def handle_route(route):
            url = route.request.url
            # ดักจับ request ไปยัง media.vdohls.com
            if "media.vdohls.com" in url and (".m3u8" in url or ".mp4" in url or "/video/" in url):
                nonlocal video_url
                if video_url is None:  # เอาแค่ URL แรก
                    video_url = url
                    print(f"✅ เจอ video URL: {url}")
            route.continue_()

        context = browser.new_context()
        page = context.new_page()

        # Route interception
        page.route("**/*", handle_route)

        # เปิดหน้าเว็บ
        page.goto(page_url, wait_until="networkidle", timeout=30000)

        # รอสักครู่ให้ player load
        page.wait_for_timeout(5000)

        # ถ้าไม่เจอ video URL จาก network request ลอง inspect DOM
        if video_url is None:
            try:
                # ลองดูจาก player config
                content = page.content()
                config_match = re.search(r'"asset"\s*:\s*"([^"]+)"', content)
                media_match = re.search(r'"medias"\s*:\s*\{[^}]*"original"\s*:\s*"([^"]+)"', content)

                if config_match and media_match:
                    asset = config_match.group(1)
                    media_id = media_match.group(1)

                    # ลองสร้าง URL ตาม pattern ที่เป็นไปได้
                    possible_urls = [
                        f"https://{asset}/hls/{media_id}/master.m3u8",
                        f"https://{asset}/v/{media_id}.m3u8",
                        f"https://{asset}/video/{media_id}.mp4",
                        f"https://{asset}/media/{media_id}/playlist.m3u8",
                    ]

                    for test_url in possible_urls:
                        print(f"🔎 ทดลอง URL: {test_url}")
                        try:
                            response = page.request.get(test_url)
                            if response.status == 200:
                                video_url = test_url
                                print(f"✅ URL ใช้งานได้: {test_url}")
                                break
                        except:
                            continue
            except Exception as e:
                print(f"⚠️  เกิดข้อผิดพลาดขณะ inspect DOM: {e}")

        browser.close()

    return video_url


def extract_video_url_with_selenium(page_url):
    """
    ใช้ Selenium ดึง video URL จากหน้า embed
    """
    print(f"🔍 กำลังดึง video URL ด้วย Selenium: {page_url}")

    chrome_options = Options()
    chrome_options.add_argument("--headless")
    chrome_options.add_argument("--no-sandbox")
    chrome_options.add_argument("--disable-dev-shm-usage")

    driver = webdriver.Chrome(options=chrome_options)

    try:
        # Enable network logging
        driver.get("chrome://net-internals/#sockets")
        # วิธีนี้ซับซ้อน ใช้ approach อื่นแทน

        driver.get(page_url)

        # รอให้ page load
        import time
        time.sleep(10)

        # Inspect DOM
        content = driver.page_source
        config_match = re.search(r'"asset"\s*:\s*"([^"]+)"', content)
        media_match = re.search(r'"medias"\s*:\s*\{[^}]*"original"\s*:\s*"([^"]+)"', content)

        if config_match and media_match:
            asset = config_match.group(1)
            media_id = media_match.group(1)

            print(f"📦 Asset: {asset}")
            print(f"📦 Media ID: {media_id}")

            # Return config เพื่อให้ caller ประมวลผลต่อ
            driver.quit()
            return {"asset": asset, "media_id": media_id}

        driver.quit()
        return None

    except Exception as e:
        print(f"❌ เกิดข้อผิดพลาด: {e}")
        driver.quit()
        return None


def extract_iframe_url(page_url):
    """
    ดึง URL ของ iframe จากหน้าเว็บหลัก
    """
    print(f"🔍 กำลังดึง iframe URL จาก: {page_url}")

    try:
        result = subprocess.run(
            ['curl', '-s', page_url],
            capture_output=True,
            text=True,
            check=True
        )
        html = result.stdout
    except subprocess.CalledProcessError as e:
        print(f"❌ ไม่สามารถดึงหน้าเว็บ: {e}")
        return None

    iframe_pattern = r'<iframe[^>]*src=["\']([^"\']*(?:play\.baiwarp\.com|baiwarp\.com)[^"\']*)["\']'
    match = re.search(iframe_pattern, html)

    if not match:
        iframe_pattern = r'<iframe[^>]*data-lazy-src=["\']([^"\']*)["\']'
        match = re.search(iframe_pattern, html)

    if match:
        iframe_url = match.group(1)
        iframe_url = iframe_url.replace('&amp;', '&')
        print(f"✅ เจอ iframe: {iframe_url}")
        return iframe_url

    print("❌ ไม่พบ iframe บนหน้าเว็บ")
    return None


def is_51cg1_url(url):
    """
    ตรวจสอบว่า URL เป็นของเว็บ 51cg1 หรือไม่
    """
    return "51cg1.com" in url or "51cg" in url


def extract_video_url_51cg1(page_url):
    """
    TitanScript Logic สำหรับดึง URL วิดีโอจาก 51cg1.com
    Ported from download_51cg1.py
    """
    print(f"🕵️‍♂️ กำลังวิเคราะห์หน้าเว็บ 51cg1: {page_url}")
    
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    }
    
    try:
        response = requests.get(page_url, headers=headers, timeout=15)
        response.raise_for_status()
        html_content = response.text
    except Exception as e:
        print(f"❌ ไม่สามารถโหลดหน้าเว็บได้: {e}")
        return []

    urls = set()
    soup = BeautifulSoup(html_content, 'html.parser')
    
    # Method 1: Regex ค้นหา .m3u8 โดยตรง
    matches = re.findall(r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']', html_content)
    for match in matches:
        url = match.replace('\\/', '/')
        urls.add(url)
        
    # Method 2: ค้นหา Source tag
    for source in soup.find_all('source'):
        src = source.get('src')
        if src and '.m3u8' in src:
            urls.add(src)
            
    # Method 3: ค้นหา iframe และตามเข้าไปดู
    for iframe in soup.find_all('iframe'):
        src = iframe.get('src') or iframe.get('data-lazy-src')
        if src:
            if 'baiwarp' in src or 'play.baiwarp.com' in src:
                print(f"🔎 พบ Iframe ผู้เล่น Baiwarp: {src}")
                try:
                    iframe_resp = requests.get(src, timeout=10)
                    iframe_html = iframe_resp.text
                    
                    config_match = re.search(r'window\.playerConfig\s*=\s*(\{.*?\});', iframe_html)
                    if config_match:
                        data = json.loads(config_match.group(1))
                        if 'medias' in data and 'asset' in data:
                            asset = data['asset']
                            media_id = data['medias'].get('original')
                            if asset and media_id:
                                final_url = f"https://{asset}/{media_id}/video.m3u8"
                                print(f"✅ แกะ URL จาก Player Config สำเร็จ: {final_url}")
                                urls.add(final_url)
                    
                    iframe_matches = re.findall(r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']', iframe_html)
                    for m in iframe_matches:
                        urls.add(m.replace('\\/', '/'))
                        
                except Exception as e:
                    print(f"⚠️ ไม่สามารถแกะข้อมูล iframe {src}: {e}")

    # Method 4: DPlayer data-config
    dplayers = soup.find_all('div', class_='dplayer')
    for dp in dplayers:
        config_str = dp.get('data-config')
        if config_str:
            try:
                data = json.loads(config_str)
                if 'video' in data and 'url' in data['video']:
                    video_url = data['video']['url']
                    video_url = video_url.replace('\\/', '/')
                    print(f"✅ พบ DPlayer video: {video_url}")
                    urls.add(video_url)
            except Exception as e:
                print(f"⚠️ Error parsing DPlayer config: {e}")

    # Method 5: Regex fallback for escaped urls
    fallback_matches = re.findall(r'[\"\'](https?(:?|\\/|/)[^\"\']*?\.m3u8[^\"\']*?)[\"\']', html_content)
    for match in fallback_matches:
        if isinstance(match, tuple):
             match = match[0]
        url = match.replace('\\/', '/')
        urls.add(url)

    return list(urls)



def download_video(video_url, output_dir=".", quality="best"):
    """
    ดาวน์โหลดวิดีโอด้วย yt-dlp
    """
    print(f"🎬 กำลังดาวน์โหลด: {video_url}")

    cmd = [
        'yt-dlp',
        '-f', quality,
        '-o', f'{output_dir}/%(title)s.%(ext)s',
        '--no-playlist',
        '--merge-output-format', 'mp4',
        video_url
    ]

    try:
        subprocess.run(cmd, check=True)
        print("✅ ดาวน์โหลดเสร็จสิ้น!")
        return True
    except subprocess.CalledProcessError as e:
        print(f"❌ ดาวน์โหลดไม่สำเร็จ: {e}")
        return False


def download_from_page(page_url, output_dir=".", quality="best"):
    """
    ดาวน์โหลดวิดีโอจากหน้าเว็บ
    """
    # 0. ตรวจสอบว่าเป็นเว็บ 51cg1 หรือไม่
    if is_51cg1_url(page_url):
        print("🔍 ตรวจพบ URL ของ 51cg1.com กำลังใช้ TitanScript logic...")
        video_urls = extract_video_url_51cg1(page_url)
        if video_urls:
            # สมมติว่าดาวน์โหลดวิดีโอแรกที่เจอ หรือวนลูปดาวน์โหลด (ตาม logic เดิมดาวน์โหลด 1 ไฟล์)
            # เพื่อความเข้ากันได้กับโปรแกรมหลัก จะเอา URL แรก
            print(f"✅ พบวิดีโอ 51cg1: {video_urls[0]}")
            return download_video(video_urls[0], output_dir, quality)
        else:
            print("❌ ไม่พบวิดีโอในหน้า 51cg1")
            return False

    # 1. ลองใช้ Lightweight method ก่อน (เร็วสุด)
    video_url = extract_video_url_lightweight(page_url)
    if video_url:
        return download_video(video_url, output_dir, quality)

    print("⚠️  Lightweight extraction ไม่สำเร็จ ลองใช้ Browser Automation...")

    # 1. ดึง iframe URL
    iframe_url = extract_iframe_url(page_url)
    if not iframe_url:
        return False

    # 2. ดึง video URL จาก iframe
    if USE_PLAYWRIGHT:
        video_url = extract_video_url_with_playwright(iframe_url)
    else:
        config = extract_video_url_with_selenium(iframe_url)
        if config:
            # ถ้าใช้ Selenium และได้ config ต้องสร้าง video URL
            video_url = None
            # จะต้องประมวลผลเพิ่ม แต่สำหรับตอนนี้ให้คืนค่า fail
        else:
            video_url = None

    if video_url:
        # 3. ดาวน์โหลดวิดีโอ
        return download_video(video_url, output_dir, quality)
    else:
        # ถ้าไม่สามารถดึง video URL ได้ ลองใช้ yt-dlp กับ iframe URL โดยตรง
        print("⚠️  ไม่สามารถดึง video URL โดยตรง ลองใช้ yt-dlp กับ iframe...")
        return download_video(iframe_url, output_dir, quality)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        print("\n📝 ใส่ URL ของหน้าวิดีโอที่ต้องการดาวน์โหลด")
        return 1

    page_url = sys.argv[1]
    output_dir = sys.argv[2] if len(sys.argv) > 2 else "."
    quality = sys.argv[3] if len(sys.argv) > 3 else "best"

    print("🎥 Script ดาวน์โหลดวิดีโอจาก xn--82c7abb4jua0l.com")
    print("=" * 50)

    success = download_from_page(page_url, output_dir, quality)

    if success:
        print("\n✨ เสร็จสิ้น! ตรวจสอบไฟล์ในโฟลเดอร์ที่ระบุ")
        return 0
    else:
        print("\n❌ ดาวน์โหลดไม่สำเร็จ")
        return 1


if __name__ == "__main__":
    sys.exit(main())
