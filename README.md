# par_tui – Arch Linux Update Manager

A TUI wrapper for safe Arch Linux system updates with selective package exclusion.

## Features

- **Async TUI Loading** – Immediate startup with background package scanning
- **Real-time Progress** – Animated spinner and status updates during scan
- **Selective Updates** – Exclude packages temporarily or permanently
- **Dual Update Modes**
  - Full system update (paru)
  - Official repositories only (pacman)
- **Permanent Exclusions** – Save package ignore list to config
- **Scan Failure Handling** – Graceful degradation on partial scan failures
- **Smart Scrolling** – Navigate through large package lists with centered cursor

## Requirements

- `pacman-contrib` (for `checkupdates`)
- `paru` (optional, for AUR support)

## Install / Build

```bash
# Clone
git clone https://github.com/ry2x/par_tui.git
cd par_tui

# Build & run (debug)
cargo run

# Build release
cargo build --release

# Install manually
sudo install -Dm755 target/release/par_tui /usr/bin/par_tui

# For Arch Linux, this project has PKGBUILD
makepkg -si
```

## Usage

Simply run:

```bash
par_tui
```

The app will:
1. Launch TUI immediately
2. Scan for updates in background
3. Display available updates with real-time progress
4. Allow you to select packages to exclude
5. Execute the update command with your selections

### Key Bindings

| Key | Action |
|-----|--------|
| `j` / `k` / `↑` / `↓` | Navigate package list |
| `Space` | Toggle temporary ignore for this session |
| `p` | Toggle permanent ignore (saved to config) |
| `Enter` | Update entire system (paru) |
| `o` | Update official repositories only (pacman) |
| `?` | Show help modal with GitHub link |
| `q` | Quit |

## Configuration

Config file: `~/.config/par_tui/config.toml`

```toml
[exclude]
# Always ignored packages
permanent = ["my-custom-kernel-bin", "experimental-driver-git"]

[behavior]
# Warn if AUR updates are attempted while official repos are stale (>3 days)
warn_stale_system = true

# Arguments always passed to pacman / paru
extra_args = []
```

### Permanent Exclusions

Press `p` on any package in the TUI to toggle permanent exclusion. Changes are saved immediately to `config.toml`.

## Notes

- The TUI uses **viewport scrolling** – only visible items are rendered for performance
- **Retry mechanism**: `checkupdates` will retry up to 3 times on failure
- **Thread safety**: Background scan thread is properly cleaned up on quit
- Configuration directory is created automatically on first run

## Architecture

Built with strict layer separation for maintainability:

- `models/` – Pure data structures
- `io/` – System interactions (commands, files, terminal)
- `parser/` – String → Model transformation
- `core/` – Business logic (filtering, planning)
- `ui/` – Presentation layer (TUI rendering)

See [`.github/instructions/directory.instructions.md`](.github/instructions/directory.instructions.md) for details.

## Contributions

**Contributions are welcome! Feel free to open issues or PRs on GitHub.**

## Tech

- ratatui + crossterm (TUI framework)
- serde + toml (configuration)
- std::thread + mpsc (async scanning)

## License

Licensed under either of:

- MIT License
- Apache License, Version 2.0

at your option.

