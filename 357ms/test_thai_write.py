
import os

target_dir = os.path.join(os.getcwd(), "downloads", "สาวไทยขามภพปวนใจแมทพ")
if not os.path.exists(target_dir):
    os.makedirs(target_dir)

filename = "test_write_สาวไทย.txt"
path = os.path.join(target_dir, filename)

print(f"Testing write to: {path}")
try:
    with open(path, 'w', encoding='utf-8') as f:
        f.write("Test content")
    print("Write SUCCESS")
except Exception as e:
    print(f"Write FAILED: {e}")
