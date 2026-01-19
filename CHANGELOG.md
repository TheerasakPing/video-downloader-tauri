# Changelog

All notable changes to Rongyok Video Downloader will be documented in this file.

## [1.5.3] - 2026-01-19

### Fixed
- **Updater Signing Pipeline**: All platforms now have working signatures
  - macOS (ARM64 & x86_64): `.app.tar.gz` with valid signatures
  - Linux: `.AppImage` with valid signatures
  - Windows: `.exe` with valid signatures
- Fixed `latest.json` generation for auto-updater
- Fixed workflow YAML syntax issues with heredoc

### Changed
- Replaced heredoc with `jq` for reliable JSON generation in CI
- Updated signing key format to Tauri v2 compatible format

## [1.5.0] - 2026-01-19

### Fixed
- **Updater Configuration**: Added `createUpdaterArtifacts: true` to enable signing
- Fixed signing key format (base64-encoded full pubkey file)
- Fixed password configuration for signing keys

## [1.4.0] - 2026-01-18

### Added
- **Auto-paste from Clipboard**: Automatically paste URL when app gains focus
- **Auto-fetch**: Automatically fetch series info after pasting URL
- **thongyok.com Support**: Added support for thongyok.com URLs
- **Improved History Panel**: Now stores up to 500 download records

### Changed
- Enhanced clipboard integration with Tauri clipboard plugin

## [1.3.0] - 2026-01-18

### Added
- **Drag & Drop URL**: Drag URL text directly onto the app window to add it
- **Keyboard Shortcuts**:
  - `Ctrl+V` - Paste URL from clipboard
  - `Ctrl+D` - Start download
  - `Space` - Pause/Resume download
  - `Escape` - Cancel download
  - `Ctrl+M` - Toggle Mini Mode
  - `Ctrl+Tab/Shift+Tab` - Navigate between tabs
- **Keyboard Shortcuts Help**: Press `?` or click icon to see all shortcuts
- **Download Presets**: Quick preset buttons for common settings
  - High Quality (max concurrent, no speed limit)
  - Save Bandwidth (limited concurrent, 500 KB/s limit)
  - Fast Download (max concurrent, no limit)
  - Night Mode (limited speed, quiet)
- **Mini Mode**: Floating mini window showing only download progress
- **Multi-language Support**: Switch between English and Thai
  - Language selector in Settings
  - Full translation for all UI elements
- **Custom Color Themes**: 5 built-in color themes
  - Violet (default)
  - Ocean Blue
  - Emerald
  - Rose
  - Amber
- **Video Preview Component**: Ready for video preview before download

### Changed
- Settings panel now includes Language and Color Theme sections
- App entry wrapped with I18nProvider for translation support

## [1.2.0] - 2026-01-18

### Added
- **Glowing Icon Effects**: Beautiful glowing icons throughout the app
  - CSS classes for glowing icons with multiple color variants (violet, fuchsia, emerald, blue, amber, red, cyan, slate)
  - Animated pulse glow effect for active/important icons
  - Enhanced Logo with SVG glow filters and sparkle animations
  - Glowing effects on header tabs, action buttons, and settings sections
  - Button glow effects (`btn-glow-violet`, `btn-glow-emerald`, `btn-glow-red`)
  - Tab glow effect for active tabs

### Changed
- Updated Logo component with enhanced visual effects
- Improved visual feedback on interactive elements

## [1.1.0] - 2026-01-18

### Added
- **OTA Update Feature**: Check for updates directly from GitHub releases
  - Auto-check on app startup (3 seconds after launch)
  - Manual check via "Check Now" button in Settings
  - Update dialog with version info, release notes, and progress bar
  - Download and install updates with app restart
- **Clear URL Button**: X button to clear the URL input field
- **Enhanced Icons**: More icons throughout the UI
- **Custom Logo**: New gradient logo with play button and download arrow design

### Changed
- Updated header with gradient title styling
- Improved tab icons

## [1.0.0] - 2026-01-18

### Added
- Initial release of Rongyok Video Downloader
- Video downloading from rongyok.com
- Episode selection with select all/deselect all
- Concurrent downloads (configurable 1-5 simultaneous)
- Speed limiting option
- Auto-merge videos with FFmpeg
- Download progress with speed graph
- File browser for downloaded files
- Download history tracking
- Settings panel with customization options
  - Theme selection (Light/Dark/System)
  - Notification settings
  - Sound alerts
  - File naming options
- Cross-platform support (Windows, macOS Intel/ARM, Linux)
