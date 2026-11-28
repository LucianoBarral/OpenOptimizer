# Changelog

All notable changes to Open Optimizer. Format based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), versions follow [SemVer](https://semver.org/).

> English version of [CHANGELOG.md](CHANGELOG.md) (Portuguese). Both must have the same versions and items: a test checks it, and a version missing from either file **is not published**.

## [0.3.0] - 2026-11-28

### Added
- **FPS measured in Gaming Mode sessions**: average FPS, 1% low, 0.1% low and frametime of every session, recorded automatically. The first 15 s (loading) are left out and sessions with less than 1 minute measured don't enter comparisons.
- **Measured performance** section on the Gaming Mode screen: last session, trend per game and **before → after** the latest optimization change (median of up to 3 sessions on each side). With no "before" session, the app says so instead of inventing a number.

### Security
- Measurement uses Intel's PresentMon 2.6.0 (MIT, signed by Intel), which only listens to Windows events (ETW): nothing is injected into or opened on the game process. The version is pinned and its SHA-256 is checked at build time.

## [0.2.2] - 2026-11-27

### Added
- **Release notes in the app**, in Portuguese and English: after an update, a window shows what changed; the full list is in Settings → About.
- What's new in the update notice is now formatted instead of raw text.

### Changed
- Settings moved to the bottom of the sidebar, apart from the features.
- More space between the update notice and the window controls.

### Fixed
- The installer now tells Windows the app icon changed, so the taskbar doesn't keep the old icon after an update.

## [0.2.1] - 2026-11-27

### Fixed
- The default Windows scrollbar (white and thick) replaced with a thin dark one that matches the app.
- Content scrolled under the invisible top strip and the minimize/maximize/close buttons; the strip is now fixed and content fades out as it reaches it.
- Taskbar showed the old icon (the v0.1.0 asterisk): the window only had the small icon set and Windows fetched the large one from a stale cache. The app now sets the large icon from its own executable.

## [0.2.0] - 2026-11-27

### Added
- **Automatic Gaming Mode**: background process (`oo-gamingd.exe`, ~0.6 MB of RAM) that detects when a game starts and applies a 0.5 ms timer, the Ultimate Performance plan, standby list cleanup (when free memory drops below 1 GB) and high priority; everything is undone when the game closes.
- Detection without touching the game: process snapshot by name; no handle is ever opened on the game process.
- Game discovery: Windows' own list (Game Bar), Epic, Steam and manual add.
- **Gaming Mode** screen with live state, counters and game list; real card on the Overview.
- Settings → **Animations**: Always (default) or Follow Windows.

### Fixed
- Animations almost absent on PCs with Windows "Animation effects" turned off.
- Old icon on the taskbar and in Task Manager after updating (Windows icon cache).

### Security
- Gaming Mode scheduled task created from XML (no battery limit and no 72 h limit), and system executables always called by absolute path.

## [0.1.2] - 2026-11-27

### Added
- Visual identity: new logo on the icon, installer, sidebar and tray.
- Animated launch (the logo's needle speeds up) and "Optimizing" / "Restoring" screens with the logo.
- System tray icon (Open / Optimize now / Quit) and a prompt when closing: tray or quit.
- Inter font and window controls inside the app (no title bar).

## [0.1.1] - 2026-11-27

### Fixed
- Memory: with the window minimized, WebView2 switches to low-memory mode (~111 MB → ~23 MB).

## [0.1.0] - 2026-11-27

### Added
- First release: reversal engine with a write-ahead journal, 5 optimizations (MMCSS, Game DVR, power throttling, Ultimate Performance, mouse acceleration), real metrics, history with undo, PT-BR/EN and signed automatic updates.

[0.3.0]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.3.0
[0.2.2]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.2.2
[0.2.1]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.2.1
[0.2.0]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.2.0
[0.1.2]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.1.2
[0.1.1]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.1.1
[0.1.0]: https://github.com/LucianoBarral/OpenOptimizer/releases/tag/v0.1.0
