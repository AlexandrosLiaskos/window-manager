<p align="center">
    <img src="assets/icon.png" alt="Window Manager" width="128" height="128"/>
</p>

<h1 align="center">Window Manager</h1>

<p align="center">
    <em>A lightweight, single-focus window manager for Windows 11.<br/>
    Only one window visible at a time — distraction-free productivity.</em>
</p>

<p align="center">
    <a href="https://github.com/AlexandrosLiaskos/window-manager/releases/latest/download/WindowManager-0.3.0-Setup.exe">
        <img src="https://img.shields.io/badge/Download_Installer-100000?style=for-the-badge&logo=windows&logoColor=white&labelColor=002b36&color=85c8c8"
            alt="Download Installer"/></a>
</p>

<p align="center">
    <a href="https://github.com/AlexandrosLiaskos/window-manager/graphs/contributors">
        <img src="https://img.shields.io/github/contributors/AlexandrosLiaskos/window-manager?color=%2385c8c8&style=for-the-badge"/></a>
    <a href="https://github.com/AlexandrosLiaskos/window-manager/blob/main/LICENSE">
        <img src="https://img.shields.io/github/license/AlexandrosLiaskos/window-manager?color=%2385c8c8&style=for-the-badge"/></a>
    <a href="https://github.com/AlexandrosLiaskos/window-manager/releases">
        <img src="https://img.shields.io/github/v/release/AlexandrosLiaskos/window-manager?color=%2385c8c8&style=for-the-badge"
            alt="latest release version"/></a>
</p>

<p align="center">
    <a href="https://github.com/AlexandrosLiaskos/window-manager/stargazers">
        <img src="https://img.shields.io/github/stars/AlexandrosLiaskos/window-manager?color=%2385c8c8&style=for-the-badge"/></a>
    <a href="https://github.com/AlexandrosLiaskos/window-manager/network/members">
        <img src="https://img.shields.io/github/forks/AlexandrosLiaskos/window-manager?color=%2385c8c8&style=for-the-badge"/></a>
    <a href="https://github.com/AlexandrosLiaskos/window-manager/issues">
        <img src="https://img.shields.io/github/issues/AlexandrosLiaskos/window-manager?color=%2385c8c8&style=for-the-badge"/></a>
</p>

---

## Features

- **Single Focus Mode** - Only one window visible at a time
- **Centered Windows** - Active window centered at configurable size (default 95%)
- **Automatic Management** - Windows are managed as you switch to them
- **System Tray** - Runs quietly in the background
- **Lightweight** - ~2MB, no runtime dependencies
- **DPI Aware** - Crisp text on high-resolution displays

<img width="2879" height="1799" alt="image" src="https://github.com/user-attachments/assets/e6c9f581-ceb0-4b74-be25-a233060ef5e7" />

## Installation

### Download Installer

1. Click the **Download Installer** button above
2. Run `WindowManager-X.X.X-Setup.exe`
3. Follow the installation wizard
4. Launch from Start Menu

### Build from Source

```powershell
git clone https://github.com/AlexandrosLiaskos/window-manager.git
cd window-manager
cargo build --release
.\target\release\window-manager.exe
```

## Usage

### Getting Started

1. Launch Window Manager from Start Menu
2. Select "Monocle" mode and press Enter
3. Switch between windows using Alt+Tab or clicking

### How It Works

- When you switch to a window, it becomes the focused window
- All other managed windows are minimized
- The focused window is centered at 95% of screen size

### Hotkeys

| Hotkey | Action |
|--------|--------|
| `Alt+Shift+M` | Toggle window manager on/off |

### System Tray

Right-click the tray icon for options:
- **Enable/Disable** - Toggle window management
- **Exit** - Close the application

## Configuration

Edit `config.toml` in the installation directory:

```toml
[general]
# Window size as percentage of screen (10-100%)
window_size = 95

# Start enabled
enabled = true

[hotkeys]
# Toggle window manager ON/OFF
toggle_enabled = "Alt+Shift+M"

[exclusions]
# Window classes to ignore
classes = ["Shell_TrayWnd", "Progman"]

# Window titles to ignore (substring match)
titles = ["Program Manager"]
```

## Building the Installer

Requirements:
- [Rust](https://rustup.rs/) toolchain
- [Inno Setup 6](https://jrsoftware.org/isdl.php)

```powershell
cd installer
.\build.ps1
```

Output: `target\installer\WindowManager-X.X.X-Setup.exe`

## Architecture

```
src/
├── main.rs        # Entry point, message loop
├── config.rs      # TOML configuration
├── manager.rs     # Window management logic
├── window.rs      # Window operations
├── positioning.rs # Screen calculations
├── hooks.rs       # Windows event hooks
├── hotkeys.rs     # Global hotkey registration
├── splash.rs      # Mode selection UI
├── tray.rs        # System tray icon
└── mode.rs        # Window mode definitions
```

## Roadmap

- [x] Monocle mode (single focus)
- [ ] Tiling mode (grid layout)
- [ ] Floating mode (traditional)
- [ ] Per-monitor support
- [ ] Window animations

## License

MIT License - see [LICENSE](LICENSE) for details.

## Contributing

Contributions welcome! Please open an issue or pull request.
