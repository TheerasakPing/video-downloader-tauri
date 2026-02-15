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

def download_hls_video(m3u8_url, output_filename=None, max_retries=3, headers=None):
    """
    ดาวน์โหลดวิดีโอ HLS (m3u8) และรวมเป็นไฟล์ MP4
    """
    print(f"กำลังเริ่มดาวน์โหลดจาก: {m3u8_url}")
    
    # ถ้าไม่ได้ระบุชื่อไฟล์ ให้ลองดึงอัตโนมัติ
    if not output_filename:
        video_title = get_video_title(m3u8_url)
        output_filename = f"{video_title}.mp4"
        print(f"ตั้งชื่อไฟล์อัตโนมัติเป็น: {output_filename}")
    
    try:
        # Prepare FFmpeg command
        # Headers string construction
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

        # Use temp directory for ffmpeg output to avoid permission issues on some volumes
        # explicitly use /tmp because TMPDIR env var might be set to project dir which ffmpeg can't write to
        import platform
        if platform.system() == "Windows":
             import tempfile
             base_temp = tempfile.gettempdir()
        else:
             base_temp = "/tmp"
             
        temp_output_path = os.path.join(base_temp, os.path.basename(output_filename))
        print(f"Temporary output path: {temp_output_path}")

        ffmpeg_cmd.extend([
            '-i', m3u8_url,
            '-c', 'copy',
            '-bsf:a', 'aac_adtstoasc',
            '-y',
            temp_output_path
        ])
        
        print(f"Executing FFmpeg command: {' '.join(ffmpeg_cmd)}")
        
        subprocess.run(ffmpeg_cmd, check=True, stdout=subprocess.PIPE, stderr=subprocess.PIPE)
        
        # Move file back to original destination
        print(f"Moving file from {temp_output_path} to {output_filename}")
        try:
            if os.path.exists(output_filename):
                os.remove(output_filename)
            shutil.move(temp_output_path, output_filename)
            print(f"ดาวน์โหลดเสร็จสมบูรณ์! บันทึกไฟล์ที่: {output_filename}")
        except Exception as move_err:
            print(f"ดาวน์โหลดเสร็จสิ้น แต่ไม่สามารถย้ายไฟล์ไปที่ {output_filename} ได้ (Permission Error)")
            print(f"ไฟล์ของคุณอยู่ที่: {temp_output_path}")
            print(f"Error details: {move_err}")
            # Do NOT delete temp file here!
            return True
            
        return True
        
    except subprocess.CalledProcessError as e:
        error_msg = e.stderr.decode('utf-8')
        print(f"FFmpeg Error: {error_msg}")
        # Clean up temp file if needed
        # if os.path.exists(temp_output_path): os.remove(temp_output_path)
        return False
        
    except Exception as e:
        print(f"เกิดข้อผิดพลาด: {e}")
        return False

if __name__ == "__main__":
    target_url = "https://media.vdohls.com/R48Ss-m5w_Tea/video.m3u8"
    # ไม่ระบุชื่อไฟล์ เพื่อให้ระบบ auto detect
    download_hls_video(target_url)
