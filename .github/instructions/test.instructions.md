---
applyTo: 'tests/*.rs'
---

# Test Architecture Instructions

This document defines **how tests are structured and why** in `par_tui`.

Tests are not written only to verify correctness — they exist to **enforce architectural boundaries**. Any test that violates these rules is considered a design error.

---

## 1. Guiding Principle

> **Tests must mirror the production architecture.**

If a test needs to cross layers, the production design is either broken or unclear. Tests must never compensate for architectural violations.

---

## 2. Test Directory Structure

```
tests/
├── fixtures/           # Raw command outputs (facts)
│   ├── pacman_qu.txt
│   └── paru_qua.txt
├── parser_tests.rs     # Tests for parser/* using fixtures
├── core_tests.rs       # Tests for core/* decision logic
└── common/
    └── mod.rs          # Test-only data builders
```

Each directory and file corresponds to a **specific layer responsibility**.

---

## 3. `fixtures/` — Raw Output Storage

**Purpose**:

* Store *unmodified*, real-world command output samples

**Rules**:

* Files must contain raw text exactly as produced by commands
* No preprocessing or normalization
* Fixtures represent **facts**, not interpretations

**Usage**:

* Loaded via `include_str!`
* Used only by parser tests

**Forbidden**:

* Generated or mocked output
* Programmatic manipulation

> Fixtures model the external world. They must remain dumb and realistic.

---

## 4. `parser_tests.rs` — Parser Verification

**Purpose**:

* Verify that raw command output is correctly translated into domain models

**Allowed**:

* `parser::*`
* `models::*`
* `fixtures/*`

**Rules**:

* Input must be raw strings from fixtures
* Output must be validated domain structures

**Forbidden**:

* Calling `io::*`
* Executing system commands
* Using `core::*`

> Parser tests prove that string → model translation is deterministic and correct.

---

## 5. `core_tests.rs` — Decision Logic Verification

**Purpose**:

* Verify business rules and decision making

**Allowed**:

* `core::*`
* `models::*`
* `tests/common::*`

**Rules**:

* Inputs must be constructed models
* Tests must not depend on real command output

**Forbidden**:

* Using fixtures directly
* Calling `parser::*`
* Any form of I/O

> Core tests validate *thinking*, not *observation*.

---

## 6. `tests/common/` — Test Data Builders

**Purpose**:

* Improve test readability
* Avoid duplication of model construction

**Rules**:

* Must depend on `models` only
* Must not contain assertions
* Must not embed business logic

**Examples**:

* Package builders
* Config builders

> `common` exists to make tests clearer — not smarter.

---

## 7. What Is Not Tested

The following are **explicitly excluded** from unit tests:

* `io/*`
* Terminal raw-mode handling
* External command execution

**Reason**:

* These involve real side effects
* They are validated indirectly through parser and core tests

Integration tests may be added separately if needed.

---

## 8. Architectural Enforcement Rule

If a test requires breaking these rules:

1. The production architecture is unclear or incomplete
2. The architecture documentation must be updated first
3. Tests must never silently bypass layer boundaries

---

## 9. Final Rule

> **If a test feels easier by crossing layers, it is the test that is wrong.**

Tests exist to protect the architecture — not convenience.

