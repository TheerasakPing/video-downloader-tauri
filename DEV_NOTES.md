# Development Notes

## Current State

- **Version**: 1.7.3
- **Last Updated**: 2026-01-23
- **Status**: Released

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
│   ├── MiniMode.tsx       # Floating mini progress window
│   ├── VideoPreview.tsx   # Video preview modal
│   ├── ShortcutsHelp.tsx  # Keyboard shortcuts dialog
│   ├── PresetSelector.tsx # Download preset selector
│   └── ThemeSelector.tsx  # Color theme selector
├── hooks/
│   ├── useLogger.ts       # Logging hook
│   ├── useSettings.ts     # Settings management
│   ├── useHistory.ts      # Download history
│   ├── useSpeedGraph.ts   # Speed data tracking
│   ├── useUpdater.ts      # OTA update functionality
│   ├── useKeyboardShortcuts.ts  # Keyboard shortcuts
│   ├── useDownloadPresets.ts    # Download presets
│   ├── useI18n.tsx              # Multi-language (Thai/English)
│   └── useCustomTheme.ts        # Custom color themes
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

### v1.7.3 - Hotfix Release

- **New App Icon**: Modern violet/fuchsia gradient logo (SVG/PNG/ICNS/ICO)
- **Keyboard Shortcuts**:
  - `Ctrl+V` - Paste URL
  - `Ctrl+D` - Start download
  - `Space` - Pause/Resume
  - `Escape` - Cancel
  - `Ctrl+M` - Toggle Mini Mode
  - `Ctrl+Tab` - Navigate tabs
- **Download Presets**: High Quality, Turbo Mode, Data Saver, Night Mode
- **Theme System**: 5 Color Themes (Violet, Ocean Blue, Emerald, Amber, Rose)
- **Mini Mode**: Floating window for monitoring downloads
- **Multi-language**: Complete Thai/English support

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
