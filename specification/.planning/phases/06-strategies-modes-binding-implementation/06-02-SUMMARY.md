---
phase: 06-strategies-modes-binding-implementation
plan: 02
subsystem: emit-modes
tags: [hmr, lib-mode, test-mode, dev-mode, prod-mode, emit-modes]
dependency_graph:
  requires: [06-01]
  provides: [hmr-injection, emit-mode-validation]
  affects: [crates/qwik-optimizer-oxc/src/lib.rs, crates/qwik-optimizer-oxc/src/code_move.rs]
tech_stack:
  added: []
  patterns: [inject_use_hmr, synthetic_imports, NewModuleCtx]
key_files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/lib.rs
    - crates/qwik-optimizer-oxc/src/code_move.rs
    - crates/qwik-optimizer-oxc/tests/snapshot_tests.rs
decisions:
  - "HMR _useHmr injection via string-based prepend in code_move::inject_use_hmr rather than AST manipulation"
  - "Synthetic imports mechanism added to NewModuleCtx for segment-level import injection"
  - "dev_path defaults to abs_dir/file_name when not provided (matching SWC behavior)"
metrics:
  duration: 6m
  completed: 2026-04-02
---

# Phase 06 Plan 02: Emit Modes and HMR Injection Summary

HMR _useHmr() injection for component$ segments, validated all 5 emit modes (Lib/Prod/Dev/Hmr/Test) with dedicated integration tests.

## What Was Done

### Task 1: HMR _useHmr() injection and emit mode validation

Implemented the D-41 HMR hook injection pattern:

- Added `inject_use_hmr()` function in `code_move.rs` that prepends `_useHmr("devPath")` as the first statement inside component$ function bodies (both arrow and function expressions)
- Wired `dev_path` through the `transform_code` pipeline in `lib.rs` (was previously unused `_dev_path`)
- Added `synthetic_imports` field to `NewModuleCtx` struct to allow segment-level import injection without relying on GlobalCollect
- HMR injection is gated on `EmitMode::Hmr` AND `ctx_name == "component$"` -- other dollar calls (bare `$`, `useTask$`, etc.) are not affected
- Effective dev_path defaults to `abs_dir/file_name` when `dev_path` is None, matching SWC behavior

Validated existing emit mode behaviors:
- **Lib mode**: Already correctly produces no separate segments (all inline via `is_inline_mode()`)
- **Test mode**: Already correctly preserves `isServer`/`isBrowser`/`isDev` identifiers (const_replace skips)
- **Dev mode**: Already correctly uses `qrlDEV`/`inlinedQrlDEV`
- **Prod mode**: Already correctly uses standard `qrl()`

### Task 2: Emit mode integration tests

Added 11 focused integration tests in `snapshot_tests.rs` and 7 unit tests in `lib.rs`:
- HMR mode: `_useHmr` present in component$ segments, absent in bare $ segments, qrlDEV in root
- Lib mode: 0 segment modules, inlinedQrl in root
- Test mode: build constant identifiers preserved
- Dev mode: qrlDEV usage, segments generated
- Prod mode: standard qrl(), no qrlDEV, segments generated
- All 5 modes produce output without errors

## Deviations from Plan

None -- plan executed exactly as written. The Lib/Test/Dev/Prod modes were already correctly implemented from Phase 5; only HMR _useHmr() injection was missing.

## Decisions Made

1. **String-based HMR injection**: Used `inject_use_hmr()` with the same `prepend_into_block_arrow`/`prepend_into_function_body` pattern as `transform_function_expr` for captures. This keeps HMR injection consistent with the existing string-assembly-then-reparse pattern (D-37).

2. **Synthetic imports mechanism**: Added `synthetic_imports: &[String]` to `NewModuleCtx` rather than modifying `generate_imports` or GlobalCollect. This cleanly separates compile-time import needs (from source analysis) from runtime-injected imports (like `_useHmr`).

3. **dev_path default**: When `dev_path` is None, defaults to `abs_dir/file_name` per SWC reference behavior.

## Test Results

- 251 lib unit tests: PASS
- 222 snapshot + integration tests: PASS
- 24 spec example tests: IGNORED (pre-existing, not part of this plan)
- Total: 473 tests, 0 failures

## Self-Check: PASSED

- All modified files exist on disk
- All 3 task commits verified (e5b1fb3, 7766029, 0f87d65)
- SUMMARY.md created at expected path
