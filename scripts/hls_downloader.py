import os
import requests
import subprocess
import shutil
from urllib.parse import urljoin, urlparse
import re

def get_video_title(m3u8_url):
    """
    พยายามดึงชื่อวิดีโอจาก URL หรือ Metadata
    """
    # 1. พยายามดึงจาก URL path
    try:
        path = urlparse(m3u8_url).path
        # แยกส่วน path และเอาส่วนที่ไม่ใช่ ext
        parts = path.split('/')
        # หา part ที่ดูเหมือนจะเป็นชื่อ (ไม่ใช่ video.m3u8 หรือ playlist.m3u8)
        for part in reversed(parts):
            if part and not part.endswith('.m3u8'):
                return part
    except:
        pass
    
    return "downloaded_video"

def download_hls_video(m3u8_url, output_filename=None, max_retries=3):
    """
    ดาวน์โหลดวิดีโอ HLS (m3u8) และรวมเป็นไฟล์ MP4
    """
    print(f"กำลังเริ่มดาวน์โหลดจาก: {m3u8_url}")
    
    # ถ้าไม่ได้ระบุชื่อไฟล์ ให้ลองดึงอัตโนมัติ
    if not output_filename:
        video_title = get_video_title(m3u8_url)
        output_filename = f"{video_title}.mp4"
        print(f"ตั้งชื่อไฟล์อัตโนมัติเป็น: {output_filename}")
    
    # สร้างโฟลเดอร์ชั่วคราว
    temp_dir = "temp_segments"
    if os.path.exists(temp_dir):
        shutil.rmtree(temp_dir)
    os.makedirs(temp_dir)
    
    try:
        # 1. ดาวน์โหลด Playlist
        response = requests.get(m3u8_url, timeout=10)
        response.raise_for_status()
        playlist_content = response.text
        
        base_url = m3u8_url.rsplit('/', 1)[0] + '/'
        
        segments = []
        for line in playlist_content.splitlines():
            line = line.strip()
            if line and not line.startswith('#'):
                # จัดการ Relative URL
                if not line.startswith('http'):
                    segment_url = urljoin(base_url, line)
                else:
                    segment_url = line
                segments.append(segment_url)
        
        print(f"พบจำนวน Segments: {len(segments)}")
        
        if not segments:
            print("ไม่พบ Segment ใน Playlist หรือรูปแบบไม่ถูกต้อง")
            return False

        # 2. ดาวน์โหลด Segments
        segment_files = []
        for i, segment_url in enumerate(segments):
            segment_filename = os.path.join(temp_dir, f"segment_{i:04d}.ts")
            segment_files.append(segment_filename)
            
            success = False
            for attempt in range(max_retries):
                try:
                    print(f"กำลังดาวน์โหลด Segment {i+1}/{len(segments)}...", end='\r')
                    seg_response = requests.get(segment_url, stream=True, timeout=10)
                    seg_response.raise_for_status()
                    
                    with open(segment_filename, 'wb') as f:
                        for chunk in seg_response.iter_content(chunk_size=8192):
                            f.write(chunk)
                    success = True
                    break
                except requests.RequestException as e:
                    print(f"\nดาวน์โหลด Segment {i} ล้มเหลว (ครั้งที่ {attempt+1}): {e}")
            
            if not success:
                print(f"\nไม่สามารถดาวน์โหลด Segment {i} ได้หลังจากลอง {max_retries} ครั้ง")
                return False
        
        print("\nดาวน์โหลด Segments ครบถ้วน")
        
        # 3. สร้าง File List สำหรับ FFmpeg
        list_file_path = os.path.join(temp_dir, "file_list.txt")
        with open(list_file_path, 'w') as f:
            for seg_file in segment_files:
                # ต้องใช้ absolute path เพื่อความชัวร์ หรือ relative path ที่ถูกต้อง
                abs_path = os.path.abspath(seg_file)
                f.write(f"file '{abs_path}'\n")
        
        # 4. รวมไฟล์ด้วย FFmpeg
        print("กำลังรวมไฟล์เป็น MP4...")
        
        # ตรวจสอบว่า output file มีอยู่แล้วหรือไม่ ถ้ามีให้ลบออกก่อน
        if os.path.exists(output_filename):
            os.remove(output_filename)
            
        ffmpeg_cmd = [
            'ffmpeg',
            '-f', 'concat',
            '-safe', '0',
            '-i', list_file_path,
            '-c', 'copy',
            '-y', # Overwrite output files without asking
            output_filename
        ]
        
        subprocess.run(ffmpeg_cmd, check=True, stdout=subprocess.DEVNULL, stderr=subprocess.STDOUT)
        
        print(f"ดาวน์โหลดเสร็จสมบูรณ์! บันทึกไฟล์ที่: {output_filename}")
        return True
        
    except Exception as e:
        print(f"เกิดข้อผิดพลาด: {e}")
        return False
        
    finally:
        # 5. ลบไฟล์ชั่วคราว
        if os.path.exists(temp_dir):
            shutil.rmtree(temp_dir)
            print("ลบไฟล์ชั่วคราวเรียบร้อย")

if __name__ == "__main__":
    target_url = "https://media.vdohls.com/R48Ss-m5w_Tea/video.m3u8"
    # ไม่ระบุชื่อไฟล์ เพื่อให้ระบบ auto detect
    download_hls_video(target_url)
