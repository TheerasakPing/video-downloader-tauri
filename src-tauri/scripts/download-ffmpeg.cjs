const fs = require("fs");
const path = require("path");
const { execSync } = require("child_process");

const platform = process.argv[2];
if (!platform) {
  console.error("Please provide a platform target (e.g., x86_64-apple-darwin)");
  process.exit(1);
}

const resourcesDir = path.join(__dirname, "../resources/bin");
if (!fs.existsSync(resourcesDir)) {
  fs.mkdirSync(resourcesDir, { recursive: true });
}

// Map Rust target triple to ffmpeg-static-electron platform suffix
// We'll use binaries from a reliable source like eugeneware/ffmpeg-static or similar
// For simplicity and reliability in this specific context, we'll use a direct download
// approach similar to what many Tauri apps do, or use a known working release.
//
// However, since we need specific sidecars, let's look at how we can get them.
// A common pattern is to download from a release that enables them.
// For now, let's use the 'ffmpeg-static' approach logic but manually handling the URLs
// or using a specific repo that hosts these builds for Tauri.
//
// Actually, a very common way is to use `ffmpeg-static` package logic but just dl the file.
// Let's use `ffbinaries` or similar if appropriate, OR just direct URLs to a known repo like
// `eugeneware/ffmpeg-static` releases.

// Platform mapping
// macos-aarch64 -> darwin-arm64
// macos-x86_64 -> darwin-x64
// linux-x86_64 -> linux-x64
// windows-x86_64 -> win32-x64

let ffmpegUrl = "";
let ffprobeUrl = "";
let ffmpegName = `ffmpeg-${platform}`;
let ffprobeName = `ffprobe-${platform}`;

// We will use binaries from `mwader/static-ffmpeg` or `eugeneware` or similar.
// `mwader` provides docker images, not direct binaries easily.
// `ffbinaries` API is good but sometimes rate limited.
// Let's use a reliable static build source. `evermeet.cx` for mac, `gyan.dev` for windows, `johnvansickle` for linux.
// OR use a gathered repo like `rust-ffmpeg/ffmpeg-next`? No.
// Let's use `btb/ffmpeg-static` or similar.
// Better yet, let's use the URLs usually found in these setup scripts.

// Simplified for the requested platforms:
// macOS Arm64
// macOS Intel
// Windows x64
// Linux x64

// Using https://github.com/eugeneware/ffmpeg-static/releases (which wraps widely used binaries)
// But we need to rename them carefully.

const getUrls = (target) => {
  // These are example URLs - in a real prod env we might want to mirror these
  // or use a specific version.

  // MacOS (Intel & Arm)
  // https://evermeet.cx/ffmpeg/ffmpeg-109056-g0e7c5c0-zip (snapshot) - unstable url
  // Let's use standardized releases from a repo if possible.

  // Using `ffmpeg-static-electron` releases structure or similar is safer/easier?
  // Let's stick to the `supeffective` or similar trusted sources if we can't find a direct single repo.

  // Actually, `tag-init/tauri-ffmpeg-setup` logic is good.
  // Let's try to grab from `https://github.com/eugeneware/ffmpeg-static/releases`
  // but they are packed in libs.

  // Let's use: https://github.com/ffbinaries/ffbinaries-prebuilt/releases/
  // It has v4.4.1 usually.

  const version = "v4.4.1";
  const baseUrl = `https://github.com/ffbinaries/ffbinaries-prebuilt/releases/download/${version}`;

  let os = "";
  let arch = "";
  let ext = "";

  if (target.includes("apple-darwin")) {
    os = "darwin";
    // check arch
    if (target.includes("aarch64"))
      arch = "arm64"; // ffbinaries might not have arm64 for 4.4.1 easily?
    else arch = "64";

    // ffbinaries 4.4.1 might only have 64bit for mac?
    // Let's check a more modern source for M1 support if needed.
    // 4.4.1 is old.

    // Let's switch to: https://github.com/eugeneware/ffmpeg-static/releases/tag/b6.0
    // They upload binaries for all platforms.
    // Actually, let's use a very standard source:
    // https://github.com/porn-vault/ffmpeg-static-prebuilt/releases
    // OR just handle it per OS.
  }

  // Let's go with a known working set of URLs for Tauri apps often used in tutorials.
  // Mac: https://evermeet.cx/ffmpeg/getrelease/zip
  // Win: https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.zip
  // Linux: https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz

  // BUT, extracting from zip/tar is annoying in a simple JS script without deps.
  // We want direct binaries if possible.

  // Let's use `create-desktop-shortcuts` approach? No.

  // Let's use the `ffbinaries` logic but implement a simple downloader.
  // For M1 macs, we might need a specific one.

  // Alternative: Use an npm package in the package.json to do this?
  // `npm install --save-dev ffmpeg-static` downloads it to node_modules.
  // We can just copy it from there!
  // `ffmpeg-static` usually only downloads for the CURRENT platform.

  // OK, best simple solution:
  // Use `supeffective/ffmpeg-binaries` or similar?
  // Let's try to define explicit URLs for this specific app's needs.

  if (target.includes("windows")) {
    return {
      ffmpeg:
        "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-win32-x64.exe",
      ffprobe:
        "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffprobe-win32-x64.exe",
      isWin: true,
    };
  } else if (target.includes("linux")) {
    return {
      ffmpeg:
        "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-linux-x64",
      ffprobe:
        "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffprobe-linux-x64",
      isWin: false,
    };
  } else if (target.includes("apple-darwin")) {
    if (target.includes("aarch64")) {
      return {
        ffmpeg:
          "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-darwin-arm64",
        ffprobe:
          "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffprobe-darwin-arm64",
        isWin: false,
      };
    } else {
      return {
        ffmpeg:
          "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffmpeg-darwin-x64",
        ffprobe:
          "https://github.com/eugeneware/ffmpeg-static/releases/latest/download/ffprobe-darwin-x64",
        isWin: false,
      };
    }
  }
  return null;
};

const config = getUrls(platform);
if (!config) {
  console.error(`Unknown target: ${platform}`);
  process.exit(1);
}

const downloadFile = (url, dest, isWin) => {
  console.log(`Downloading ${url} to ${dest}...`);
  // Use curl because it's available on all GitHub actions runners (Mac, Win, Linux)
  // On Windows Powershell, curl is an alias but we can use generic 'curl' command.
  try {
    execSync(`curl -L -o "${dest}" "${url}"`);
    if (!isWin) {
      execSync(`chmod +x "${dest}"`);
    }
    console.log(`Success: ${dest}`);
  } catch (e) {
    console.error(`Failed to download ${url}: ${e.message}`);
    process.exit(1);
  }
};

const ext = config.isWin ? ".exe" : "";
downloadFile(
  config.ffmpeg,
  path.join(resourcesDir, `${ffmpegName}${ext}`),
  config.isWin,
);
downloadFile(
  config.ffprobe,
  path.join(resourcesDir, `${ffprobeName}${ext}`),
  config.isWin,
);
