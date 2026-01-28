# par_tui - Project Status Memo

**Date**: 2026-01-28 (JST)
**Status**: Core Implementation Complete

---

## 📊 Project Overview

Arch Linux system update manager with TUI for safe package exclusion management.

### Key Features
- ✅ Safe update scanning (checkupdates + paru)
- ✅ Interactive TUI with package selection
- ✅ Permanent/temporary exclusion support
- ✅ Two update modes (Entire System / Official Only)
- ✅ Configuration file support ($XDG_CONFIG_HOME/partui)
- ✅ Command execution with inherited stdio

---

## 📁 Architecture

### Directory Structure (Layered Architecture)
```
src/
├── main.rs           # Application entrypoint
├── lib.rs            # Library interface for tests
├── models/           # Pure data structures
│   ├── config.rs     # Config, ExcludeConfig, BehaviorConfig
│   └── package.rs    # Package, PackageRepository
├── io/               # External I/O operations
│   ├── command.rs    # checkupdates, paru execution
│   ├── file.rs       # Config file read/write
│   └── terminal.rs   # TUI runner (ratatui + crossterm)
├── parser/           # String → Model transformation
│   ├── pacman.rs     # Parse checkupdates output
│   ├── paru.rs       # Parse paru -Qua output
│   └── toml.rs       # Config serialization
├── core/             # Business logic
│   ├── filter.rs     # Package exclusion logic
│   └── planner.rs    # Command building & execution
└── ui/               # Presentation layer
    ├── app.rs        # AppState & event handling
    └── view.rs       # Ratatui rendering

tests/
├── filter_tests.rs       # 5 tests
├── integration_tests.rs  # 6 tests
├── parser_tests.rs       # 4 tests
├── planner_tests.rs      # 6 tests
└── ui_tests.rs           # 7 tests
```

### Dependency Rules
- models → (nothing)
- parser → models
- core → models
- io → models, ui
- ui → models
- main.rs → all layers

---

## 🧪 Test Coverage

**Total**: 28 tests (all passing)

| Suite | Tests | Coverage |
|-------|-------|----------|
| parser_tests | 4 | checkupdates/paru output parsing |
| filter_tests | 5 | permanent/temporary exclusions |
| planner_tests | 6 | command building, mode filtering |
| ui_tests | 7 | AppState, cursor, toggles, stats |
| integration_tests | 6 | end-to-end command generation |

---

## 📦 Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
toml = "0.8"
ratatui = "0.29"
crossterm = "0.28"
```

---

## �� Configuration

### Location
- `$XDG_CONFIG_HOME/partui/config.toml`
- Default: `~/.config/partui/config.toml`

### Format
```toml
[exclude]
permanent = ["my-kernel-git", "experimental-driver"]

[behavior]
warn_stale_system = true
extra_args = ["--noconfirm"]
```

---

## 🎨 TUI Design

### Layout (3-pane)
```
┌─────────────────────────────────────────────────────────────┐
│ par_tui - [Updates Found: 14]              [Help: ?]        │
├─────────────────────────────────────────────────────────────┤
│ > [ ] [Official] linux        6.1.10 -> 6.1.12              │
│   [x] [AUR]      my-app-git   r12    -> r13    (PERM)       │
│   ...                                                        │
├─────────────────────────────────────────────────────────────┤
│ Mode: Entire System (paru)                                  │
│ Stats: Official (10) | AUR (4) | To Ignore: 1               │
├─────────────────────────────────────────────────────────────┤
│ [Enter] Entire  [S-Enter] Official  [Space] Toggle  [q]     │
└─────────────────────────────────────────────────────────────┘
```

### Key Bindings
- `j/k` or `↓/↑`: Navigate
- `Space`: Toggle package ignore
- `Enter`: Execute entire system update (paru)
- `Shift+Enter`: Execute official only (pacman)
- `?`: Toggle help modal
- `q`: Quit

---

## 🔍 Code Quality

### Clippy Status
- **Warnings**: 12 (all dead_code/unused - intentional for future features)
- **Pedantic lints**: Enabled
- **Critical lints**: unwrap_used (deny), float_cmp (deny)

### Documentation
- ✅ All public functions documented
- ✅ `# Errors` sections for Result-returning functions
- ✅ `#[must_use]` on pure functions

---

## 🚀 Execution Flow

1. **Load config** from `$XDG_CONFIG_HOME/partui/config.toml`
2. **Check commands** (checkupdates, paru availability)
3. **Scan packages**
   - Official: `checkupdates`
   - AUR: `paru -Qua`
4. **Launch TUI** with AppState
5. **User interaction**
   - Navigate packages
   - Toggle temporary excludes
   - Select update mode
6. **Exit TUI** and restore terminal
7. **Execute command** with inherited stdio
   - Entire: `paru -Syu --ignore <list>`
   - Official: `sudo pacman -Syu --ignore <list>`

---

## 📝 Recent Changes

### Last Session (2026-01-28)
1. ✅ Implemented comprehensive test suite (28 tests)
2. ✅ Added documentation to all public functions
3. ✅ Added `#[must_use]` attributes
4. ✅ Reverted error confirmation prompt (user request)
5. ✅ Reduced Clippy warnings from 27 to 12

---

## 🎯 Current State

**Status**: Production-ready core implementation

### Working Features
- ✅ Package scanning (with error handling)
- ✅ Configuration loading
- ✅ TUI with full interaction
- ✅ Command generation and execution
- ✅ Permanent/temporary exclusions
- ✅ Two update modes
- ✅ 28 passing tests

### Known Limitations
- ⚠️ `checkupdates` errors can occur (network/sync issues)
  - Currently shown as warning, TUI proceeds if AUR packages available
- ℹ️ Some dead_code warnings for future features

---

## 📊 Statistics

- **Source files**: 19 Rust files
- **Test files**: 5 test suites
- **Total lines**: ~680 lines (src + tests)
- **Commits**: 12+ commits
- **Test coverage**: Core logic fully tested

---

## 🔜 Potential Future Work

### Not Required for Core Functionality
- [ ] README.md creation
- [ ] AUR PKGBUILD
- [ ] CI/CD setup
- [ ] Dependency conflict warnings
- [ ] VCS package rebuild support (--devel flag)
- [ ] Stale system warnings (>3 days)

### Architecture Notes
- Clean layer separation maintained
- All architectural constraints followed
- No forbidden cross-layer dependencies
- Test coverage excellent for critical paths

---

## 💡 Development Notes

### Commands
```bash
# Build
cargo build

# Test
cargo test

# Clippy
cargo clippy --all-targets

# Run
cargo run
```

### Project Principles
1. **Directory structure = Architecture**
2. **Pure layers** (models, parser, core)
3. **Side effects isolated** (io only)
4. **Truth > Comfort** (critical review over validation)
5. **No partial upgrades without awareness**

---

**End of Status Memo**
