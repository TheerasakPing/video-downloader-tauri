# Development Notes

## Current State
- **Version**: 1.2.0
- **Last Updated**: 2026-01-18
- **Status**: Development (v1.3.0 in progress)

## Tech Stack
- **Frontend**: React 19 + TypeScript + Vite
- **Backend**: Rust (Tauri v2)
- **Styling**: Tailwind CSS
- **Icons**: Lucide React

## Project Structure
```
src/
├── App.tsx                 # Main application component
├── main.tsx               # App entry with I18nProvider
├── index.css              # Global styles including glowing effects
├── components/
│   ├── Button.tsx         # Reusable button component
│   ├── Input.tsx          # Input field component
│   ├── Logo.tsx           # Animated logo with glow effects
│   ├── ProgressBar.tsx    # Download progress bar
│   ├── EpisodeSelector.tsx # Episode grid selector
│   ├── SeriesCard.tsx     # Series info display
│   ├── LogPanel.tsx       # Log viewer
│   ├── SettingsPanel.tsx  # Settings configuration
│   ├── HistoryPanel.tsx   # Download history
│   ├── SpeedGraph.tsx     # Download speed visualization
│   ├── FileBrowser.tsx    # File management
│   ├── DownloadQueue.tsx  # Queue management
│   ├── UpdateDialog.tsx   # OTA update dialog
│   ├── MiniMode.tsx       # NEW: Floating mini progress window
│   ├── VideoPreview.tsx   # NEW: Video preview modal
│   ├── ShortcutsHelp.tsx  # NEW: Keyboard shortcuts dialog
│   ├── PresetSelector.tsx # NEW: Download preset selector
│   └── ThemeSelector.tsx  # NEW: Color theme selector
├── hooks/
│   ├── useLogger.ts       # Logging hook
│   ├── useSettings.ts     # Settings management
│   ├── useHistory.ts      # Download history
│   ├── useSpeedGraph.ts   # Speed data tracking
│   ├── useUpdater.ts      # OTA update functionality
│   ├── useKeyboardShortcuts.ts  # NEW: Keyboard shortcuts
│   ├── useDownloadPresets.ts    # NEW: Download presets
│   ├── useSpeedSchedule.ts      # NEW: Speed limiting schedule
│   ├── useI18n.tsx              # NEW: Multi-language (Thai/English)
│   └── useCustomTheme.ts        # NEW: Custom color themes
└── types/
    └── index.ts           # TypeScript type definitions

src-tauri/
├── src/
│   ├── lib.rs             # Tauri app setup with plugins
│   ├── commands.rs        # Tauri commands (IPC)
│   └── downloader.rs      # Video download logic
├── tauri.conf.json        # Tauri configuration
├── Cargo.toml             # Rust dependencies
└── capabilities/
    └── default.json       # App permissions
```

## Key Features Implemented

### v1.3.0 (In Progress) - New Features
- **Drag & Drop URL**: Drag URL text directly onto app to start download
- **Keyboard Shortcuts**:
  - `Ctrl+V` - Paste URL
  - `Ctrl+D` - Start download
  - `Space` - Pause/Resume
  - `Escape` - Cancel
  - `Ctrl+M` - Toggle Mini Mode
  - `Ctrl+Tab/Shift+Tab` - Navigate tabs
- **Download Presets**: Quick preset buttons (High Quality, Save Bandwidth, Fast Download, Night Mode)
- **Mini Mode**: Floating mini window showing only download progress
- **Multi-language**: Thai/English interface (useI18n hook)
- **Custom Themes**: 5 color themes (Violet, Ocean Blue, Emerald, Rose, Amber)
- **Video Preview**: Component ready for video preview before download
- **Speed Schedule**: Hook ready for time-based speed limiting

### v1.2.0 - Glowing Icons
- Added CSS classes in `index.css`:
  - `.icon-glow` - Base glowing icon container
  - `.icon-glow-{color}` - Color variants (violet, fuchsia, emerald, blue, amber, red, cyan, slate)
  - `.icon-glow-animated` - Pulsing animation
  - `.btn-glow-{color}` - Button glow effects
  - `.tab-glow-active` - Active tab glow
- Enhanced Logo.tsx with SVG filters and animations

### v1.1.0 - OTA Updates
- `useUpdater.ts` hook for update checking
- `UpdateDialog.tsx` for update UI
- Tauri plugins: `tauri-plugin-updater`, `tauri-plugin-process`
- GitHub releases endpoint for update checking

### v1.0.0 - Core Functionality
- Video fetching and parsing from rongyok.com
- Concurrent download management
- FFmpeg integration for video merging
- Settings persistence with localStorage
- Download history tracking

## GitHub Actions
- **Workflow**: `.github/workflows/release.yml`
- **Trigger**: Push tag `v*`
- **Builds**: Windows (.exe, .msi), macOS Intel/ARM (.dmg), Linux (.deb, .rpm, .AppImage)
- **Note**: `createUpdaterArtifacts: false` (no signing key configured)

## How to Continue Development

### Running Locally
```bash
cd /Volumes/       DriveE /code_project/_rongyok_video_downloader_rust
npm run tauri dev
```

### Building
```bash
npm run build          # Frontend only
npm run tauri build    # Full app build
```

### Creating a Release
1. Update version in:
   - `src-tauri/tauri.conf.json`
   - `src-tauri/Cargo.toml`
   - `src/components/SettingsPanel.tsx` (display version)
2. Commit changes: `git commit -m "chore: bump version to X.Y.Z"`
3. Create and push tag: `git tag vX.Y.Z && git push origin vX.Y.Z`
4. GitHub Actions will build and publish the release

## Known Issues
- GitHub Dependabot warning (1 moderate vulnerability)
- `set-output` deprecation warnings in GitHub Actions

## Future Improvements
- [ ] Add signing keys for auto-update functionality
- [ ] Implement actual pause/resume downloads (Rust backend)
- [ ] Video quality selection
- [ ] System tray integration (requires tauri-plugin-system-tray)
- [ ] Speed schedule integration with download logic
