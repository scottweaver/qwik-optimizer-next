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
