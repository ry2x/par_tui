# TUI Design Specification: `par_tui`

This document defines the **terminal user interface (TUI) architecture, layout, and interaction rules** for `par_tui`.

The TUI is a presentation layer only. It must strictly follow the architectural constraints defined in `directory.instruction.md`.

---

## 1. UI Layer Responsibilities (Architectural Compliance)

The UI layer is split by file responsibility:

* **`ui/app.rs` (State & Controller)**

  * Receives user input
  * Manages UI state (cursor position, temporary ignore list, modal visibility)
  * Emits UI events

* **`ui/view.rs` (Renderer)**

  * Pure rendering only
  * Draws Ratatui widgets based on `AppState`
  * Must not mutate state or perform side effects

**Strictly Forbidden**:

* Calling `io/*` directly from any UI file
* Executing system commands
* Making business decisions

> The UI reflects decisions — it does not create them.

---

## 2. Main Screen Layout

The main screen uses a **three-pane layout** optimized for fast scanning and keyboard-only interaction.

### 2.1 Screen Mockup

```text
+-------------------------------------------------------------+
|  par_tui - [Updates Found: 14]                     [Help: ?] |
+-------------------------------------------------------------+
| > [ ] [Official] linux              6.1.10 -> 6.1.12        |
|   [ ] [Official] mesa               23.0.1 -> 23.0.2        |
|   [x] [AUR]      hobby-app-git      r12.a  -> r13.b (PERM)  |
|   [ ] [AUR]      yay                12.0.1 -> 12.0.2        |
|   ...                                                       |
+-------------------------------------------------------------+
| Mode: Entire System (paru)                                  |
| Stats: Official (10) | AUR (4) | To Ignore: 1               |
+-------------------------------------------------------------+
| [Enter] Entire  [S-Enter] Official Only  [Space] Toggle  [q] |
+-------------------------------------------------------------+
```

---

### 2.2 Component Definitions

**Header**

* Application name
* Total number of updates found
* Help hint (`?`)

**Main List**

* `[ ]`: Included in update
* `[x]`: Ignored for this run
* **Source Badge**:

  * `Official` (blue)
  * `AUR` (yellow)
* `(PERM)`: Permanently ignored via `config.toml`

**Status Line**

* Preview of the currently selected execution mode
* Summary statistics

---

## 3. Help Modal & Hyperlinks

Pressing `?` toggles a floating help overlay.

### 3.1 Modal Requirements

* Centered floating window
* Keymap reference
* Descriptive usage notes

**OSC 8 Hyperlinks**:

* Help text may include clickable links (e.g. GitHub Wiki)
* Links are rendered using OSC 8 escape sequences
* Escape sequence generation is handled in `ui/view.rs`

**Mockup**

┌──────────────── HELP ─────────────────┐
│ [Enter]   : Update Entire (paru)      │
│ [S-Enter] : Update Official (pacman)  │
│ [Space]   : Toggle Package Ignore     │
│ [?]       : Open Online Documentation │
│ [q]       : Quit App                  │
│                                       │
│ Link: https://github.com/.../par_tui  │
└───────────────────────────────────────┘

---

### 3.2 External Browser Launch Flow

External programs must never be launched directly from the UI layer.

**Flow**:

1. User activates a link or presses a bound key
2. `ui/app.rs` emits `UIEvent::OpenLink(url)`
3. `main.rs` receives the event
4. `main.rs` calls `io/command.rs` to execute `xdg-open` (or equivalent)

This preserves architectural boundaries.

---

## 4. Interaction & Key Bindings

| Key               | Action            | Notes                                          |
| ----------------- | ----------------- | ---------------------------------------------- |
| `j` / `k`         | Move cursor       | Navigate list                                  |
| `Space`           | Toggle ignore     | Flip `Package.is_ignored`                      |
| **`Enter`**       | **Entire System** | Build `paru -Syu` execution plan               |
| **`Shift+Enter`** | **Official Only** | Force-exclude AUR and build `pacman -Syu` plan |
| `?`               | Toggle help       | Show / hide modal                              |
| `q`               | Quit              | Signal termination to `main.rs`                |

---

## 5. UI State Machine

The UI must follow this lifecycle:

1. **Scanning**

   * Data is being gathered via `io`
   * Display a progress indicator

2. **Interaction**

   * User navigates and toggles packages

3. **Confirming** (optional)

   * Modal confirmation if needed

4. **Handoff**

   * UI exits
   * Terminal raw mode is restored
   * Control returns to `main.rs`

---

## 6. Architectural Design Note

> The UI does not decide *what* to ignore.
>
> When `Enter` or `Shift+Enter` is pressed, **`core/planner.rs`** computes the final `--ignore` list.
> The UI only displays the result and initiates the handoff.

---

## 7. Final Rule

> **If the UI needs to know *why* a decision was made, the architecture is wrong.**

The TUI exists to visualize state and collect intent — nothing more.

