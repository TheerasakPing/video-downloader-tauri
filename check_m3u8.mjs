import axios from 'axios';

const url = 'https://surrit.com/75bed0ea-3994-4d09-9fae-44204e3e04c0/360p/video.m3u8';
const referer = 'https://njavtv.com/th/dass-876-uncensored-leak';

async function check() {
  try {
    const resp = await axios.get(url, {
      headers: {
        'Referer': referer,
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36'
      }
    });
    console.log(resp.data);
  } catch (e) {
    console.error(e.message);
  }
}

check();
