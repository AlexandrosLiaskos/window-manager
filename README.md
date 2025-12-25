# Window Manager

[![Windows](https://img.shields.io/badge/platform-Windows%2011-0078D6?logo=windows)](https://www.microsoft.com/windows)
[![Rust](https://img.shields.io/badge/rust-1.70%2B-orange?logo=rust)](https://www.rust-lang.org/)
[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

A lightweight, single-focus window manager for Windows 11. Only one window visible at a time - distraction-free productivity.

![Window Manager Demo](https://via.placeholder.com/800x400?text=Window+Manager+Demo)

## Features

- **Single Focus Mode** - Only one window visible at a time
- **Centered Windows** - Active window centered at configurable size (default 95%)
- **Automatic Management** - Windows are managed as you switch to them
- **System Tray** - Runs quietly in the background
- **Lightweight** - ~2MB, no runtime dependencies
- **DPI Aware** - Crisp text on high-resolution displays

## Installation

### Download Installer

Download the latest installer from [Releases](../../releases):

1. Run `WindowManager-X.X.X-Setup.exe`
2. Follow the installation wizard
3. Launch from Start Menu

### Build from Source

```powershell
# Clone the repository
git clone https://github.com/user/window-manager.git
cd window-manager

# Build release
cargo build --release

# Run
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
