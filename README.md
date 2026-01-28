# par_tui

Arch Linux system update wrapper with TUI for selective package exclusion.

## Features

- **Async TUI Loading**: Immediate startup with background package scanning
- **Real-time Progress**: Animated spinner and status updates
- **Selective Updates**: Exclude packages temporarily or permanently
- **Dual Update Modes**: 
  - Full system update (paru)
  - Official repositories only (pacman)
- **Permanent Exclusions**: Save package ignore list to config
- **Scan Failure Handling**: Graceful degradation on partial scan failures

## Installation

### From AUR (Recommended)

```bash
paru -S par_tui
```

### From Source

```bash
git clone https://github.com/ry2x/par_tui.git
cd par_tui
cargo build --release
sudo install -Dm755 target/release/par_tui /usr/bin/par_tui
```

## Usage

Simply run:

```bash
par_tui
```

### Key Bindings

- `j` / `k` / `↑` / `↓` - Navigate package list
- `Space` - Toggle temporary ignore
- `p` - Toggle permanent ignore (saved to config)
- `Enter` - Update entire system (paru)
- `o` - Update official repositories only (pacman)
- `?` - Show help
- `q` - Quit

## Configuration

Configuration file: `~/.config/par_tui/config.toml`

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

## Requirements

- `pacman-contrib` (for `checkupdates`)
- `paru` (optional, for AUR support)

## License

Licensed under either of:

- MIT License
- Apache License, Version 2.0

at your option.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.
