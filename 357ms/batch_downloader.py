import requests
from bs4 import BeautifulSoup
import time
import sys
import os
from urllib.parse import urljoin

# Import extraction logic from existing script
try:
    from web_video_extractor import extract_and_download
except ImportError:
    # If running from scripts directory
    if os.path.exists("web_video_extractor.py"):
        import web_video_extractor
        extract_and_download = web_video_extractor.extract_and_download
    else:
        print("ไม่พบไฟล์ web_video_extractor.py")
        sys.exit(1)

def get_story_links(category_url):
    """
    ดึงลิงก์หน้าเนื้อหา (Story Pages) จากหน้าหมวดหมู่
    """
    print(f"กำลังดึงข้อมูลจากหมวดหมู่: {category_url}")
    headers = {
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
    }
    
    try:
        response = requests.get(category_url, headers=headers, timeout=15)
        response.raise_for_status()
        soup = BeautifulSoup(response.text, 'html.parser')
        
        links = []
        # Pattern identification based on inspecting HTML structure
        # Looking for <a class="x-get-info warp" href="...">
        articles = soup.find_all('article')
        
        for article in articles:
            link_tag = article.find('a', class_='x-get-info')
            if link_tag and link_tag.get('href'):
                href = link_tag.get('href')
                raw_title = link_tag.get('title')
                if raw_title:
                    title = str(raw_title).replace('ดูหนัง : ', '')
                else:
                    title = 'No Title'
                links.append({'url': href, 'title': title})
                
        print(f"พบ {len(links)} เรื่องในหน้านี้")
        return links
        
    except Exception as e:
        print(f"เกิดข้อผิดพลาดในการดึงหน้าหมวดหมู่: {e}")
        return []

def batch_process(category_url):
    """
    Process all videos in a category
    """
    stories = get_story_links(category_url)
    
    if not stories:
        print("ไม่พบเนื้อหาที่จะดาวน์โหลด")
        return

    print(f"เริ่มการดาวน์โหลด {len(stories)} รายการ...")
    print("=" * 50)

    success_count = 0
    fail_count = 0

    for i, story in enumerate(stories, 1):
        print(f"\n[{i}/{len(stories)}] กำลังดำเนินการ: {story['title']}")
        print(f"URL: {story['url']}")
        
        try:
            # Reuse existing logic to extract and download
            # We don't need 'only_return_url=True' because we want to download it directly
            # But the extract_and_download function prints a lot, so we might want to suppress or handle it
            result_bool, result_url = extract_and_download(story['url'], only_return_url=False)
            
            if result_bool:
                print(f"✓ สำเร็จ: {story['title']}")
                success_count += 1
            else:
                print(f"✗ ล้มเหลว: {story['title']}")
                fail_count += 1
                
        except Exception as e:
            print(f"✗ Error: {e}")
            fail_count += 1
            
        # Delay to be polite to the server
        print("พัก 5 วินาที...")
        time.sleep(5)

    print("=" * 50)
    print(f"สรุปผลการทำงาน:")
    print(f"สำเร็จ: {success_count}")
    print(f"ล้มเหลว: {fail_count}")
    print("=" * 50)

if __name__ == "__main__":
    # Test URL from the task
    TEST_CATEGORY_URL = "https://xn--82c7abb4jua0l.com/category/18/"
    
    if len(sys.argv) > 1:
        target_url = sys.argv[1]
    else:
        target_url = TEST_CATEGORY_URL
        
    batch_process(target_url)
