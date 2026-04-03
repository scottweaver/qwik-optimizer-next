---
phase: 08-implementation-gap-closure
plan: 02
subsystem: jsx-transform, signal-optimization
tags: [CONV-07, signal-optimization, jsx-transform, pipeline-wiring]
dependency_graph:
  requires: [08-01]
  provides: [CONV-07-pipeline-integration]
  affects: [jsx_transform.rs, transform.rs]
tech_stack:
  added: []
  patterns: [SignalOptContext for threading transform state to JSX module]
key_files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/jsx_transform.rs
    - crates/qwik-optimizer-oxc/src/transform.rs
decisions:
  - "D-44: Signal optimization context passed via SignalOptContext struct rather than individual parameters, keeping JSX transform module decoupled from full QwikTransform state"
  - "D-45: IdentCollector::collect used for per-expression capture analysis during prop classification, matching SWC's per-prop signal eligibility checking"
metrics:
  duration: 7m
  completed: 2026-04-03
  tasks_completed: 1
  tasks_total: 1
  files_modified: 2
---

# Phase 08 Plan 02: Signal Optimization Pipeline Wiring Summary

Wire convert_inlined_fn (CONV-07) into JSX prop classification path with SignalOptContext for state threading and _fnSignal import flag propagation.

## What Was Done

### Task 1: Wire convert_inlined_fn into JSX prop classification

Integrated the existing `convert_inlined_fn` function from `inlined_fn.rs` into the JSX prop classification pipeline in `jsx_transform.rs`, completing CONV-07 support.

**Changes to `jsx_transform.rs`:**
- Added `SignalOptContext` struct to carry transform state (decl_stack_flat, is_server, allocator) into the JSX module without coupling to `QwikTransform`
- Extended `transform_jsx_element` signature with optional `signal_ctx` parameter
- Extended `classify_props` to accept signal context and return `signal_used` flag
- Added `try_signal_optimize` function that:
  - Skips call expressions (not eligible per SWC behavior)
  - Collects identifiers from prop value expression via `IdentCollector::collect`
  - Computes scoped_idents (captures) against the flattened declaration stack
  - Builds `(name, is_const)` pairs for `convert_inlined_fn`
  - Calls `convert_inlined_fn` and parses the resulting code string into AST
- Added `parse_signal_expr` helper to convert generated `_fnSignal(...)` code into AST expressions
- Set `needs_fn_signal` in `JsxImportNeeds` when signal optimization fires

**Changes to `transform.rs`:**
- In `exit_expression`, flattens `decl_stack` and constructs `SignalOptContext`
- Passes context to `transform_jsx_element` for JSX elements
- Propagates `needs_fn_signal` flag to `self.needs_fn_signal_import` for import emission

## Commits

| Task | Commit | Message |
|------|--------|---------|
| 1 | 2bd5074 | feat(08-02): wire convert_inlined_fn into JSX prop classification path |

## Deviations from Plan

None - plan executed exactly as written.

## Verification

- `cargo test -p qwik-optimizer-oxc` -- all tests pass (0 failures)
- `cargo test -p qwik-optimizer-oxc --test snapshot_tests swc_parity` -- 223 tests pass, parity maintained
- `grep convert_inlined_fn jsx_transform.rs` confirms wiring at import (line 23) and call site (line 419)
- `grep needs_fn_signal_import transform.rs` confirms flag propagation (line 1093)

## Known Stubs

None. The signal optimization pipeline is fully wired. The `convert_inlined_fn` function contains complete eligibility logic (6 checks) and `_fnSignal` code generation. Whether specific fixtures exercise this path depends on their prop expressions meeting all 6 eligibility checks.
