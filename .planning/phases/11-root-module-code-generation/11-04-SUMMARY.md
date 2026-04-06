---
phase: 11-root-module-code-generation
plan: 04
subsystem: transform
tags: [oxc, root-module, dce, display-name, qrlDEV, const-stripping]

requires:
  - phase: 11-02
    provides: dead import/variable elimination
  - phase: 11-03
    provides: display_name/hash computation

provides:
  - Root module body structure fixes (extension, stack_ctxt, const stripping)
  - Dev mode qrlDEV metadata injection
  - Unreferenced const stripping with fixpoint loop

affects: [phase-12, phase-13]

tech-stack:
  added: []
  patterns:
    - "Text post-processing for dev metadata injection (avoids OXC codegen span violations)"
    - "Fixpoint text-level const stripping for SWC DCE parity"

key-files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/lib.rs
    - crates/qwik-optimizer-oxc/src/transform.rs
    - crates/qwik-optimizer-oxc/tests/snapshots/ (78 snapshot files)

key-decisions:
  - "D-50: Dev metadata injected as text post-processing to avoid OXC codegen sourcemap span violations"
  - "D-51: Const stripping uses text-level fixpoint loop rather than AST mutation (OXC arena allocation prevents in-place Statement type changes)"

patterns-established:
  - "Text post-emit transformation: modify codegen output string for patterns that cannot be achieved at AST level"

metrics:
  duration: 36m
  completed: "2026-04-06"
  tasks: 1
  files: 80
---

# Phase 11 Plan 04: Remaining Body Structure Fixes Summary

Root module body structure mismatches fixed via 4 targeted changes: file extension fix, stack_ctxt double-pop fix, dev metadata injection, and unreferenced const stripping. Root module parity improved from 62/201 to 80/201.

## What Changed

### 1. File Extension in QRL Import Paths (fix)
- **Problem**: When `explicit_extensions=true`, OXC used the input file extension (`.tsx`) in QRL dynamic import paths instead of the output extension (`.js`)
- **Fix**: Changed `file_extension` computation in `lib.rs` to use `output_extension()` instead of raw input extension
- **Impact**: Fixed ~5 fixtures (example_class_name, example_default_export, etc.)

### 2. Stack Context Double-Pop Bug (fix)
- **Problem**: In `exit_expression`, the marker function name (e.g., "component") was popped from `stack_ctxt` twice -- once at the explicit pop after `register_context_name` and again at a stale unconditional pop. This corrupted the display_name computation, producing names like `Header_WjUaUQN7Oxg` instead of `Header_component_J4uyIhaBNR4`.
- **Fix**: Removed the stale unconditional `self.stack_ctxt.pop()` at the end of exit_expression
- **Impact**: Fixed ~16 fixtures with hash/name mismatches

### 3. Dev Mode qrlDEV Metadata Injection (enhancement)
- **Problem**: OXC emitted `qrlDEV(()=>import("./path"), "sym")` without the metadata object that SWC includes: `{ file, lo, hi, displayName }`
- **Fix**: Store dev metadata in `QwikTransform.dev_metadata` HashMap during traversal, inject into emitted code as text post-processing (avoiding OXC codegen span violations)
- **Impact**: Dev mode fixtures now structurally correct (span values still differ slightly from SWC)

### 4. Unreferenced Const Stripping (enhancement)
- **Problem**: OXC kept `const X = wrapperQrl(q_...)` even when `X` was not exported or referenced elsewhere. SWC strips these to bare expression statements.
- **Fix**: Text-level fixpoint pass that detects unreferenced const declarations and strips them to bare expressions or removes dead assignment lines entirely.
- **Impact**: Fixed ~10 fixtures where unreferenced wrappers were kept

## Parity Results

| Metric | Before | After | Delta |
|--------|--------|-------|-------|
| Root module match | 62/201 | 80/201 | +18 |
| Full match | 62/201 | 79/201 | +17 |
| Segment count | 194/201 | 194/201 | 0 |
| Diagnostics | 192/201 | 192/201 | 0 |

## Remaining Mismatches (121 root module)

### By Category
| Category | Count | Notes |
|----------|-------|-------|
| Hash/display_name differences | ~40 | Deduplication counter (_1 suffix), remaining stack_ctxt edge cases |
| Inline entry strategy | ~26 | Missing inlinedQrl support for Inline/Hoist strategies |
| Import ordering | ~10 | Different import insertion order vs SWC |
| Dev mode span values | ~5 | lo/hi byte offsets differ between OXC and SWC |
| Signal function (_fnSignal) | ~4 | Missing signal optimization in root module |
| Dead import edge cases | ~5 | Over/under-stripping of imports |
| Other structural | ~31 | Mixed issues (comments, quote style, JSX import source, etc.) |

### Root Causes for Phase 13
1. **Inline entry strategy**: Not implemented -- requires `inlinedQrl()` code generation
2. **Display name deduplication**: Counter-based names (`_1` suffix) differ from SWC
3. **Import ordering**: SWC orders by first-use; OXC orders by insertion position
4. **Signal optimization**: `_fnSignal`/`_hf0` patterns not generated in root module

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed stack_ctxt double-pop in exit_expression**
- **Found during:** Task 1 diagnostic phase
- **Issue:** Marker function name popped twice from stack_ctxt, corrupting all display_name computations for named marker calls (component$, useTask$, etc.)
- **Fix:** Removed stale unconditional stack_ctxt.pop() call
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Commit:** 4a2ede3

**2. [Rule 1 - Bug] Fixed file extension in QRL import paths**
- **Found during:** Task 1 diagnostic phase
- **Issue:** Used raw input extension (.tsx) instead of transpiled output extension (.js) for explicit_extensions mode
- **Fix:** Changed to use output_extension() for file_extension computation
- **Files modified:** crates/qwik-optimizer-oxc/src/lib.rs
- **Commit:** 4a2ede3

## Known Stubs

None -- all changes are functional implementations.

## Notes

The 160+ target from the plan was optimistic given the scope of remaining issues. The hash/display_name differences (largest category) require deep stack_ctxt tracking changes and the inline entry strategy requires fundamental new code generation capability. The 62->80 improvement addresses the most impactful and readily fixable issues. The remaining 121 mismatches are documented with root causes for Phase 13 follow-up.
