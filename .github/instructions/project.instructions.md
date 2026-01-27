# par_tui — Project Instructions

This document defines **what the project does**, **how it must behave**, and **non‑negotiable design constraints**. It is intended for contributors and AI coding agents.

---

## 1. Purpose

`par_tui` is a Rust-based wrapper tool for **Arch Linux system updates**. It enables safe updates while **intentionally excluding specific packages** (e.g. AUR `-git` or experimental packages), without compromising overall system integrity.

The tool must:

* Preserve system consistency
* Avoid permanent system configuration changes
* Hand control back to the real terminal for package manager interaction

---

## 2. Update Modes (Core Behavior)

At startup, the user selects exactly one mode:

| Mode              | Command                         | Scope               | Intent                              |
| ----------------- | ------------------------------- | ------------------- | ----------------------------------- |
| **Entire System** | `paru -Syu --ignore ...`        | Official + AUR      | Full system update including AUR    |
| **Official Only** | `sudo pacman -Syu --ignore ...` | Official repos only | Fast base-system update without AUR |

The generated command **must match the selected mode exactly**.

---

## 3. Execution Workflow (Strict Order)

The application must follow this sequence:

1. **Environment Detection**

   * Verify availability of:

     * `checkupdates` (from `pacman-contrib`)
     * `paru`

2. **Non‑Destructive Scan**

   * Official packages: `checkupdates`
   * AUR packages: `paru -Qua`
   * No system database must be modified at this stage

3. **Filtering**

   * Apply permanent exclusions from `config.toml`

4. **TUI Interaction**

   * Display upgradable packages
   * Allow the user to select packages to *temporarily exclude* for this run

5. **Command Execution**

   * Exit TUI
   * Restore terminal from raw mode
   * Execute the update command via `std::process::Command`
   * Inherit `stdin`, `stdout`, and `stderr`
   * All `sudo` and `paru` prompts must be handled directly by the user

---

## 4. Critical Design Constraints

### 4.1 Partial Upgrade Safety

* **Risk**: Excessive use of `--ignore` may cause dependency breakage
* **Requirement**: The design must allow (or plan for) warnings when ignored packages are required by other packages being updated

### 4.2 Configuration Locality

* **Do NOT modify** `/etc/pacman.conf`
* `IgnorePkg` must never be written
* Ignoring packages must apply **only when using this tool**

### 4.3 AUR `-git` / VCS Packages

* VCS packages may not bump versions
* The configuration must support a `force_rebuild` mechanism
* When enabled, `--devel` must be dynamically added to `paru`

---

## 5. Configuration File (`config.toml`)

```toml
[exclude]
# Always ignored packages
permanent = ["my-custom-kernel-bin", "experimental-driver-git"]

[behavior]
# Warn if AUR updates are attempted while official repos are stale (>3 days)
warn_stale_system = true

# Arguments always passed to pacman / paru
extra_args = ["--noconfirm"]
```

Configuration file is in ~/.config/partui ($XDG_CONFIG_HOME).

---

## 6. Technology Stack (Fixed)

* **Language**: Rust
* **TUI**: `ratatui` + `crossterm`
* **Config Parsing**: `serde` + `toml`
* **Package Tools**:

  * `checkupdates` (external binary)
  * `paru` (external binary)

---

## 7. Non‑Goals

The project explicitly does **not**:

* Replace `pacman` or `paru`
* Persist ignore rules globally
* Automate or suppress security‑critical prompts

---

## 8. Guiding Principle

> *Scan safely, decide interactively, execute transparently.*

All implementation decisions must align with this principle.

