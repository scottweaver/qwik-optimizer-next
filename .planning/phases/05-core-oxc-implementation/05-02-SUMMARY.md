---
phase: 05-core-oxc-implementation
plan: 02
subsystem: test-infrastructure
tags: [snapshot-tests, fixtures, test-harness, spec-examples, appendix-b]
dependency_graph:
  requires: []
  provides: [snapshot-corpus, snapshot-parser, fixture-configs, spec-behavioral-tests]
  affects: [05-03, 05-04, 05-05, 05-06, 05-07]
tech_stack:
  added: [serde, serde_json]
  patterns: [macro-generated-tests, custom-snap-parser, ignored-test-progressive-activation]
key_files:
  created:
    - crates/qwik-optimizer-oxc/tests/snapshot_tests.rs
    - crates/qwik-optimizer-oxc/tests/spec_examples.rs
    - crates/qwik-optimizer-oxc/fixtures.json
    - crates/qwik-optimizer-oxc/tests/snapshots/ (201 .snap files)
    - crates/qwik-optimizer-oxc/Cargo.toml
    - crates/qwik-optimizer-oxc/src/lib.rs
  modified:
    - Cargo.toml (workspace members)
decisions:
  - "Structural snapshot comparison (parse .snap into typed data) over string comparison -- enables cosmetic tolerance"
  - "snapshot_test! macro generates 201 ignored tests for progressive activation"
  - "Multi-input fixtures (relative_paths) handled by checking fixture input count rather than requiring ==INPUT== section"
metrics:
  duration: 8m31s
  completed: 2026-04-02
  tasks_completed: 3
  tasks_total: 3
  files_created: 206
  files_modified: 1
---

# Phase 5 Plan 2: Test Harness + 201 SWC Snapshot Corpus Summary

Custom .snap parser with 201 snapshot corpus, 201 ignored transform tests, and 24 Appendix B behavioral tests covering all 14 CONVs.

## What Was Done

### Task 1: Copy Snapshot Corpus and fixtures.json
- Copied all 201 `.snap` files from `swc-snapshots/` to `crates/qwik-optimizer-oxc/tests/snapshots/`
- Copied `fixtures.json` with 201 fixture configurations
- Created minimal crate scaffold (Cargo.toml with serde/serde_json, lib.rs placeholder) as deviation to unblock compilation

### Task 2: Build Custom Snapshot Test Harness
- Implemented complete `.snap` format parser in `snapshot_tests.rs`:
  - YAML frontmatter skipping
  - `==INPUT==` section extraction
  - Module section parsing (`===== filename (ENTRY POINT)==` and `===== filename ==`)
  - Source map extraction from `Some("...")` lines
  - SegmentAnalysis JSON extraction from `/* { ... } */` blocks
  - `== DIAGNOSTICS ==` section extraction
- Defined serde-compatible `FixtureConfig`, `FixtureInput` types matching fixtures.json schema
- Defined `SnapshotData`, `SegmentSnapshot`, `RootSnapshot` structures
- 10 parser smoke tests (all pass):
  - Fixtures loading and count verification
  - All 201 snapshot files exist and are loadable
  - Simple snapshot parsing (example_1: 3 segments + root)
  - Multi-segment snapshot parsing
  - Dead code snapshot verification
  - Skip-transform edge case handling
  - Fixture config deserialization
  - SegmentAnalysis JSON parseability
  - All 201 snapshots parseable
  - Diagnostics with errors (example_capturing_fn_class)
- 201 ignored transform tests via `snapshot_test!` macro

### Task 3: Appendix B Spec-Derived Behavioral Tests
- Created `spec_examples.rs` with 24 test functions from Appendix B
- Tests organized in mod blocks by CONV: conv_01 through conv_14
- Each test includes input code, config, and expected behavior comments
- CONV_COVERAGE constant documents full coverage matrix
- All 14 CONVs covered across the 24 examples

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Created minimal crate scaffold for compilation**
- **Found during:** Task 1
- **Issue:** Plan 05-01 (crate scaffold) runs in parallel wave 1 and may not be complete yet. Tests cannot compile without a Cargo.toml and lib.rs.
- **Fix:** Created minimal `crates/qwik-optimizer-oxc/Cargo.toml` with serde/serde_json deps and a placeholder `src/lib.rs`. Plan 05-01 will expand these with full dependencies and foundation modules.
- **Files created:** `crates/qwik-optimizer-oxc/Cargo.toml`, `crates/qwik-optimizer-oxc/src/lib.rs`
- **Commit:** 8fbdadf

**2. [Rule 1 - Bug] Fixed multi-input fixture snapshot parsing**
- **Found during:** Task 2
- **Issue:** The `relative_paths` fixture has 2 inputs and its snapshot lacks an `==INPUT==` section. The parser's "all snapshots parseable" test failed.
- **Fix:** Updated test to check fixture input count before flagging empty input as an error.
- **Files modified:** `crates/qwik-optimizer-oxc/tests/snapshot_tests.rs`
- **Commit:** dc1aeb8

## Decisions Made

1. **Structural comparison over string comparison**: Parse .snap files into typed `SnapshotData` structures for semantic comparison. This enables cosmetic tolerance (whitespace, formatting) when comparing transform output against snapshots.

2. **Macro-generated ignored tests**: Use `snapshot_test!` macro to generate all 201 tests as `#[ignore]`. Progressive un-ignoring as transforms are implemented avoids maintaining a separate test-enable list.

3. **Multi-input fixture handling**: Rather than requiring an `==INPUT==` section in every snapshot, accept that multi-input fixtures store their input only in `fixtures.json`.

## Verification Results

- 201 `.snap` files present in `tests/snapshots/`
- `fixtures.json` parseable with 201 entries
- `cargo test -p qwik-optimizer-oxc --test snapshot_tests`: 10 passed, 201 ignored
- `cargo test -p qwik-optimizer-oxc --test spec_examples`: 0 passed, 24 ignored (all compile)

## Known Stubs

None -- all parser and test infrastructure is fully implemented. The 225 ignored tests (201 snapshot + 24 spec) are intentionally ignored pending transform implementation, not stubs.

## Self-Check: PASSED

All files verified present. All 3 task commits verified. 201 .snap files confirmed.
