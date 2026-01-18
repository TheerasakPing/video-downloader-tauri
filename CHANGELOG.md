# Changelog

All notable changes to Rongyok Video Downloader will be documented in this file.

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
