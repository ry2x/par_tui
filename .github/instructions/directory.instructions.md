# Directory Architecture Instructions

This document defines the **directory strategy as architecture** for `par_tui`.

The directory structure is not cosmetic. It encodes **responsibilities, dependency direction, and forbidden interactions**. Any implementation must respect these rules.

---

## 1. Architectural Principle

> **Directory structure = Architecture**

Each top‑level directory represents a *layer*. Crossing layers incorrectly is a **design violation**, even if the code compiles.

The goals are:

* Prevent partial‑upgrade risks caused by tangled logic
* Make unsafe behavior structurally impossible
* Keep AI / agent‑generated code inside strict boundaries

---

## 2. Layer Overview

```
src/
├── main.rs            # Application entrypoint (Shell)
├── models/            # Pure data definitions
├── io/                # External I/O (OS, filesystem, terminal)
├── parser/            # String → Model transformation
├── core/              # Decision making / business logic
└── ui/                # Presentation (TUI)
```

Each layer has **one primary responsibility** and **explicit dependency rules**.

---

## 3. Layer Responsibilities

> This section defines **file‑level responsibilities**. Each file is treated as a stable architectural unit.

No file may take on responsibilities outside what is declared here, even if it appears convenient.

### 3.1 `models/` — Data Layer (Pure)

**Files**:

* `models/package.rs`
* `models/config.rs`

**Purpose**:

* Define domain data structures only

**Allowed**:

* Structs / enums
* `serde` derives
* Type aliases

**Forbidden**:

* I/O of any kind
* Parsing logic
* Business rules
* Methods with side effects

> Files in `models/` must compile and make sense even if *every other directory is removed*.

---

### 3.2 `io/` — Input / Output Layer

**Files**:

* `io/command.rs`
* `io/file.rs`
* `io/terminal.rs`

**Purpose**:

* Perform real side effects

**File‑level rules**:

* `command.rs`: execute external commands and return raw output
* `file.rs`: read/write configuration files
* `terminal.rs`: raw terminal and TUI mode control

**Allowed**:

* OS access
* Returning `String` / `Vec<String>` / raw data

**Forbidden**:

* Parsing output
* Interpreting meaning
* Applying business decisions

> `io` files must never return domain decisions — only facts.

---

### 3.3 `parser/` — Translation Layer

**Files**:

* `parser/pacman.rs`
* `parser/paru.rs`
* `parser/toml.rs`

**Purpose**:

* Convert raw strings into domain models

**File‑level rules**:

* Each parser handles **one external format**
* Input: `&str`
* Output: `models::*`

**Allowed**:

* Pure transformation logic
* Validation of format correctness

**Forbidden**:

* Executing commands
* Reading files
* UI or terminal handling

> Parsers must be fully unit‑testable with static strings.

---

### 3.4 `core/` — Intelligence Layer

**Files**:

* `core/filter.rs`
* `core/planner.rs`

**Purpose**:

* Decide *what should happen*

**File‑level rules**:

* `filter.rs`: determine which packages are excluded and why
* `planner.rs`: construct final execution plans and command arguments

**Allowed**:

* Use `models`
* Pure decision logic

**Forbidden**:

* Calling `io`
* Executing system commands
* Accessing terminal or filesystem

> `core` files must be deterministic and side‑effect free.

---

### 3.5 `ui/` — Presentation Layer

**Files**:

* `ui/app.rs`
* `ui/view.rs`

**Purpose**:

* Display information and collect user input

**File‑level rules**:

* `app.rs`: state machine and user interaction flow
* `view.rs`: rendering only (no state mutation)

**Allowed**:

* Rendering logic
* Consuming prepared data

**Forbidden**:

* Business rules
* Command execution
* Configuration access

> UI files reflect decisions — they do not create them.

---

### 3.6 `main.rs` — Shell / Orchestrator

**Files**:

* `main.rs`

**Purpose**:

* Wire all layers together

**File‑level rules**:

* Call layers in the defined order
* Translate errors into user‑visible failures

**Forbidden**:

* Business logic
* Parsing
* Decision making

> If `main.rs` feels clever, it is wrong.

---

## 4. Dependency Rules (Hard Constraints)

### 4.1 Allowed Dependency Direction

```
      models
     ↗  ↑  ↖
parser  core  ui
  ↑     ↑     ↑
  └─ main.rs ─┘
     ↓
     io
```

### 4.2 Forbidden Dependencies

The following are **explicitly forbidden**:

* `core → io`
* `core → ui`
* `parser → io`
* `models → any other layer`
* `ui → io`
* 'ui → core: Forbidden (Optional recommendation)'

Violations indicate architectural breakage.

---

## 5. Testing Implications

* `models`, `parser`, and `core` **must be unit‑testable** without OS access
* `io` is tested indirectly or via integration tests only
* Sample command outputs should be stored as test fixtures

---

## 6. Architectural Enforcement Rule

If a feature requires breaking these rules:

1. The architecture is incomplete
2. The rules must be updated *before* implementation
3. Silent rule‑breaking is not allowed

---

## 7. Guiding Rule

> **If you are unsure where code belongs, it probably does not belong in `core`.**

Architecture exists to protect the system — not to be convenient.


# YOU SHOULD KNOW

If a file needs to break its rules to be convenient, the architecture is wrong — not the file.
