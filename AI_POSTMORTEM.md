# AI Postmortem: Missteps and Corrective Actions

This document tracks missteps made during AI-assisted development and the corrective actions taken.

---

## 001: Duplicated input source code into fixtures.json

**Date:** 2026-04-02

**What happened:** When building the snapshot test harness, AI duplicated the input source code from the `.snap` files' `==INPUT==` sections into a `code` field in `fixtures.json`. This created two sources of truth for the same data — the `.snap` files (the original SWC source of truth) and `fixtures.json`.

**Why it was wrong:** The `.snap` files are the canonical record ported from the SWC implementation. Duplicating their input code into `fixtures.json` violates the single-source-of-truth principle and risks the two copies drifting apart. The test harness should read input directly from the `.snap` files.

**Corrective action:**
- Removed the `code` field from all `inputs` entries in `fixtures.json`
- Modified `snapshot_tests.rs` to source input code exclusively from the `==INPUT==` sections of `.snap` files
- Fixed pre-existing bugs where tests referenced `snap.input` (non-existent field) instead of `snap.inputs`

---

## 002: Tests referenced non-existent `snap.input` field

**Date:** 2026-04-02

**What happened:** Several smoke tests in `parser_smoke_tests` referenced `snap.input` (singular), but the `SnapshotData` struct only has an `inputs` field (plural, `Vec<SnapshotInput>`). These tests would not have compiled.

**Why it was wrong:** This indicates the tests were written without verifying they compiled, or were written against an earlier version of the struct that was later changed without updating the tests.

**Corrective action:**
- Replaced all `snap.input` references with calls to a new `first_input_code(&snap)` helper that safely accesses `snap.inputs[0].code`

---

## 003: Path utility functions placed in parse.rs instead of leveraging Rust's type system

**Date:** 2026-04-02

**What happened:** `source_type_from_filename()` and `output_extension()` were implemented as free functions in `parse.rs`, taking raw `&str` arguments. These functions operate on source file paths, not on parsing logic, making `parse.rs` a grab-bag of loosely related utilities.

**Why it was wrong:** Raw `&str` parameters provide no type-level distinction between "a source file path" and "any arbitrary string." This misses an opportunity to use Rust's type system to make the domain model explicit. It also violates single-responsibility — `parse.rs` should focus on parsing, not filename-to-extension mapping.

**Corrective action:**
- Created `source_path.rs` with a `SourcePath<'a>(&'a str)` newtype wrapper
- Moved `source_type_from_filename()` -> `SourcePath::source_type()`
- Moved `output_extension()` -> `SourcePath::output_extension()`
- Updated all call sites in `parse.rs`, `lib.rs`, and `transform.rs`
- Moved corresponding tests from `parse.rs` to `source_path.rs`

---

## 004: Path decomposition function (`parse_path`) placed in parse.rs

**Date:** 2026-04-02

**What happened:** `parse_path()` and its return type `PathData` were defined in `parse.rs` alongside AST parsing logic. This function decomposes a source file path into stem, filename, relative directory, and absolute directory — purely path manipulation with no relation to AST parsing.

**Why it was wrong:** Same root cause as #003 — `parse.rs` was accumulating unrelated path utilities. With `SourcePath` already established as the domain type for source file paths, `parse_path` was an obvious method on that type. Leaving it in `parse.rs` meant callers had to know to look in two different modules for path-related operations.

**Corrective action:**
- Moved `PathData` struct and `parse_path()` from `parse.rs` to `source_path.rs`
- Renamed `parse_path()` to `SourcePath::path_data()` — called as `SourcePath("src/routes/index.tsx").path_data(src_dir)`
- Updated all call sites in `lib.rs` and `transform.rs`
- Moved corresponding tests from `parse.rs` to `source_path.rs`
- `parse.rs` now contains only AST parsing logic (`parse_module`, `ParseResult`)

---

## 005: Boolean flag in enum variant instead of distinct variants

**Date:** 2026-04-02

**What happened:** `IdentType::Var(bool)` used a boolean to distinguish `const` (`true`) from `let`/`var` (`false`) bindings. Call sites read `IdentType::Var(true)` and `IdentType::Var(false)` — a classic "boolean trap" where the meaning of `true`/`false` is invisible without checking the enum definition.

**Why it was wrong:** Boolean parameters and enum payloads that encode domain distinctions are error-prone and unreadable. Rust's enum system exists precisely to make these distinctions explicit. A match arm like `IdentType::Var(c) => { if !c { ... } }` obscures intent compared to separate `Const`/`Let` arms.

**Corrective action:**
- Split `IdentType::Var(bool)` into `IdentType::Const` and `IdentType::Let`
- Updated the `compute_scoped_idents` match to use separate arms for `Const` and `Let`
- Updated `collect_binding_to_decl` to map `is_const` bool -> `IdentType::Const`/`IdentType::Let` at the boundary
- Updated all test call sites

---

## 006: `compute_scoped_idents` returned an unused boolean in a tuple

**Date:** 2026-04-02

**What happened:** `compute_scoped_idents` returned `(Vec<String>, bool)` where the `bool` indicated whether all captured identifiers were `const`. The only real call site discarded this value (`_is_const`), and the test assertions on it were testing dead logic.

**Why it was wrong:** Returning unused data in a tuple is noise — it complicates the signature, forces callers to destructure with `_` placeholders, and suggests the value matters when it doesn't. It also kept the `Const`/`Let` match arms artificially separate when they could be collapsed.

**Corrective action:**
- Changed return type from `(Vec<String>, bool)` to `Vec<String>`
- Removed the `is_const` tracking variable and collapsed `Const`/`Let` into a single match arm
- Simplified the call site and test destructuring

---

## 007: Hand-rolled snapshot tests that didn't verify output

**Date:** 2026-04-03

**What happened:** The snapshot test harness manually parsed the SWC `.snap` files and ran `transform_modules`, but the `snapshot_test!` macro only checked that the transform didn't panic and produced non-empty output. It never compared the actual output against the expected output from the `.snap` files. Functions like `compare_snapshot` and `normalize_code` existed but were never called. The project's `Cargo.toml` already listed `insta` as a dev-dependency, but it was unused.

**Why it was wrong:** Tests that don't verify output are false confidence. The entire purpose of the 201 SWC `.snap` files is to define expected transform behavior — ignoring them means regressions go undetected. Additionally, hand-rolling snapshot comparison logic duplicates what `insta` provides out of the box with better tooling (`cargo insta review`, diff visualization, CI integration).

**Corrective action:**
- Moved SWC `.snap` files to `tests/swc_expected/` to separate the reference corpus from insta-managed snapshots
- Rewrote `snapshot_test!` macro to use `insta::assert_snapshot!` — each test now captures the actual transform output as an insta snapshot for regression detection
- Added `format_transform_output()` to serialize transform results in a format matching the SWC `.snap` structure for easy visual comparison
- Added `swc_parity::parity_report` test that compares actual output against SWC expected and prints a summary (root match, segment count, diagnostics) without failing
- Removed dead code: `compare_snapshot`, `normalize_code`, `run_snapshot_test`, `snapshot_test_debug!`
- Sorted diagnostics in output formatting to prevent non-deterministic ordering from causing false failures
