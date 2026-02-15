
import sys
import os

# Add current directory to path so we can import web_video_extractor
sys.path.append(os.getcwd())

try:
    from web_video_extractor import extract_and_download
except ImportError:
    # Try importing assuming we are running from project root
    sys.path.append(os.path.join(os.getcwd(), 'scripts'))
    from web_video_extractor import extract_and_download

url = "https://www.357ms.com/watch/5905"
print(f"Testing extraction for {url}")
success, m3u8_url = extract_and_download(url, only_return_url=True)

if success:
    print(f"SUCCESS: Found URL: {m3u8_url}")
else:
    print("FAILED: Could not find m3u8 URL")
