#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
TitanScript for 51cg1.com
ดาวน์โหลดวิดีโอจาก 51cg1.com รองรับทั้งแบบวิดีโอเดียวและหลายวิดีโอ
"""

import sys
import re
import json
import requests
import subprocess
from bs4 import BeautifulSoup
from urllib.parse import urljoin

def extract_m3u8_from_html(html_content, base_url):
    """
    พยายามค้นหา .m3u8 จาก HTML content ด้วยวิธีการต่างๆ
    """
    urls = set()
    
    # Method 1: Regex ค้นหา .m3u8 โดยตรง
    # Pattern: http... .m3u8
    matches = re.findall(r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']', html_content)
    for match in matches:
        url = match.replace('\\/', '/')
        urls.add(url)
        
    # Method 2: ค้นหา Source tag
    soup = BeautifulSoup(html_content, 'html.parser')
    for source in soup.find_all('source'):
        src = source.get('src')
        if src and '.m3u8' in src:
            urls.add(src)
            
    # Method 3: ค้นหา iframe และตามเข้าไปดู
    for iframe in soup.find_all('iframe'):
        src = iframe.get('src') or iframe.get('data-lazy-src')
        if src:
            # ตรวจสอบว่าเป็น player ที่รู้จักหรือไม่ เช่น baiwarp
            if 'baiwarp' in src or 'play.baiwarp.com' in src:
                print(f"🔎 พบ Iframe ผู้เล่น Baiwarp: {src}")
                try:
                    iframe_resp = requests.get(src, timeout=10)
                    iframe_html = iframe_resp.text
                    
                    # Pattern window.playerConfig = {...}
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
                    
                    # Regex fallback ใน iframe
                    iframe_matches = re.findall(r'[\"\'](https?://[^\"\']*?\.m3u8[^\"\']*?)[\"\']', iframe_html)
                    for m in iframe_matches:
                        urls.add(m.replace('\\/', '/'))
                        
                except Exception as e:
                    print(f"⚠️ ไม่สามารถแกะข้อมูล iframe {src}: {e}")

    # Method 4: DPlayer data-config
    # <div class="dplayer" data-config='{...}'>
    dplayers = soup.find_all('div', class_='dplayer')
    for dp in dplayers:
        config_str = dp.get('data-config')
        if config_str:
            try:
                # config_str might be HTML escaped or just JSON
                data = json.loads(config_str)
                if 'video' in data and 'url' in data['video']:
                    video_url = data['video']['url']
                    video_url = video_url.replace('\\/', '/')
                    print(f"✅ พบ DPlayer video: {video_url}")
                    urls.add(video_url)
            except Exception as e:
                print(f"⚠️ Error parsing DPlayer config: {e}")

    # Method 5: Regex fallback for escaped urls (like https:\/\/...)
    # Matches strings that start with http, contain .m3u8, inside quotes, possibly with backslashes
    fallback_matches = re.findall(r'[\"\'](https?(:?|\\/|/)[^\"\']*?\.m3u8[^\"\']*?)[\"\']', html_content)
    for match in fallback_matches:
        # match is tuple if groups used, or string. Regex has groups.
        if isinstance(match, tuple):
             match = match[0]
        url = match.replace('\\/', '/')
        urls.add(url)

    return list(urls)

def download_video(url, output_name=None):
    """
    ดาวน์โหลดวิดีโอด้วย yt-dlp
    """
    print(f"⬇️ กำลังดาวน์โหลด: {url}")
    cmd = [
        'yt-dlp',
        '--no-playlist',
        '--merge-output-format', 'mp4',
        url
    ]
    
    if output_name:
        cmd.extend(['-o', output_name])
        
    try:
        subprocess.run(cmd, check=True)
        print("✅ ดาวน์โหลดเสร็จสิ้น")
    except subprocess.CalledProcessError as e:
        print(f"❌ ดาวน์โหลดล้มเหลว: {e}")

def main():
    if len(sys.argv) < 2:
        print("Usage: python download_51cg1.py <URL>")
        sys.exit(1)
        
    url = sys.argv[1]
    print(f"🚀 เริ่มทำงานกับ URL: {url}")
    
    headers = {
        'User-Agent': 'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    }
    
    try:
        response = requests.get(url, headers=headers, timeout=15)
        response.raise_for_status()
        
        # ดึง Title
        soup = BeautifulSoup(response.text, 'html.parser')
        title = soup.title.string.strip() if soup.title else "video"
        print(f"📄 หัวข้อหน้าเว็บ: {title}")
        
        video_urls = extract_m3u8_from_html(response.text, url)
        
        if not video_urls:
            print("❌ ไม่พบวิดีโอในหน้านี้")
            return
            
        print(f"🎥 พบวิดีโอทั้งหมด {len(video_urls)} รายการ")
        
        for i, video_url in enumerate(video_urls):
            print(f"[{i+1}/{len(video_urls)}] {video_url}")
            # sanitize filename
            safe_title = re.sub(r'[\\/*?:"<>|]', "", title)
            filename = f"{safe_title}_{i+1}.mp4" if len(video_urls) > 1 else f"{safe_title}.mp4"
            download_video(video_url, filename)
            
    except Exception as e:
        print(f"❌ เกิดข้อผิดพลาด: {e}")

if __name__ == "__main__":
    main()
