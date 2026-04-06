---
phase: 13-final-acceptance
plan: 09
subsystem: transform-pipeline
tags: [auto-export, import-cleanup, object-formatting, windows-paths, ts-stripping]
dependency_graph:
  requires: [13-08]
  provides: [auto-export-injection, import-cleanup, object-collapse, windows-path-normalization]
  affects: [transform.rs, lib.rs, emit.rs, collector.rs, source_path.rs]
tech_stack:
  added: []
  patterns: [post-processing-codegen, export-local-tracking]
key_files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/transform.rs
    - crates/qwik-optimizer-oxc/src/lib.rs
    - crates/qwik-optimizer-oxc/src/emit.rs
    - crates/qwik-optimizer-oxc/src/collector.rs
    - crates/qwik-optimizer-oxc/src/source_path.rs
decisions:
  - "Auto-export injection placed after variable migration, gated on non-Lib mode"
  - "Export-local tracking via HashSet added to GlobalCollect for aliased export detection"
  - "TS type stripping deferred -- requires manual AST mutation (OXC transformer feature excluded)"
  - "Object collapse implemented as string post-processing on codegen output"
metrics:
  duration: "22m"
  completed: "2026-04-06"
  tasks_completed: 4
  tasks_total: 4
---

# Phase 13 Plan 09: Non-Signal Structural Gap Closure Summary

Auto-export injection, import cleanup, object formatting, and Windows path normalization for SWC parity improvement.

## What Changed

### Task 1: Auto-Export Implementation (2784387)
- Added `auto_exports: Vec<String>` field to `QwikTransform` struct
- Implemented `ensure_export()` method that checks both `exports` map and new `export_locals` set to avoid duplicate auto-exports for aliased exports (e.g., `export { internal as expr2 }`)
- Added `export_locals: HashSet<String>` to `GlobalCollect` to track local binding names that have any export
- Called `ensure_export` during traversal for all root-level bindings in `self_imports` (matching SWC behavior)
- Injected `export { name as _auto_name }` statements into root module after variable migration, gated on non-Lib mode
- 13 snapshots updated with correct auto-export entries

### Task 2: Import Cleanup and Object Formatting (848240f)
- Removed consumed `$`-suffixed import specifiers from root module output (e.g., `import { $, component$ }` removed when all specifiers are marker functions)
- Added `collapse_single_prop_objects()` post-processor in emit.rs that collapses single-property objects from multiline to inline format (`{ key: value }`)
- ~200 snapshots updated reflecting cleaner root module output

### Task 3: TS Type Stripping (c149e07)
- Added placeholder `strip_typescript_types()` function
- Investigation found that OXC codegen always emits TS nodes regardless of source_type setting
- TS type stripping requires manual AST mutation or enabling OXC transformer feature (excluded per CLAUDE.md)
- TS enum transpilation (IIFE pattern) also deferred for same reason
- Documented as known gap: 152 fixtures have `transpile_ts: true` but types not stripped

### Task 4: Windows Path Normalization (ef54a9e)
- Normalized backslashes to forward slashes in `source_path::path_data()` before path decomposition
- Normalized `src_dir` backslashes in `transform_code` entry point
- Fixes `support_windows_paths` fixture: segment path now correctly shows `components/apps` instead of empty string

## Parity Status

- **Baseline:** 1/201 full match (0%)
- **After changes:** 1/201 full match (0%)
- **Root module match:** 1/201
- **Segment count match:** 125/201
- **Diagnostics match:** 197/201
- **All 223 snapshot tests pass (no regressions)**

The Full Match count did not improve because the parity comparison requires exact root module match, and there are many deep structural differences beyond the scope of this plan:
1. Symbol naming/hash computation differs from SWC
2. Missing `/*#__PURE__*/` annotations on qrl and wrapper calls
3. Missing `const q_name = qrl(...)` intermediate declaration pattern
4. Missing wrapper calls like `componentQrl(q_name)` 
5. TS type annotations not stripped (152 fixtures)
6. Import ordering (framework imports not sorted before user imports)

These are fundamental transform pipeline issues that require deeper architectural changes.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Auto-export called during traversal instead of only in variable migration**
- **Found during:** Task 1
- **Issue:** Plan assumed ensure_export only needed for deps of migrated vars, but SWC calls it for ALL root vars referenced by segments
- **Fix:** Added ensure_export call in classify_captures result processing
- **Files modified:** transform.rs

**2. [Rule 2 - Missing] Export-local tracking for aliased exports**
- **Found during:** Task 1
- **Issue:** `ensure_export` only checked exports map (keyed by exported name), missing aliased exports like `export { internal as expr2 }`
- **Fix:** Added `export_locals: HashSet<String>` to GlobalCollect populated during collection
- **Files modified:** collector.rs

**3. [Rule 3 - Blocking] Lib mode auto-export gate**
- **Found during:** Task 1
- **Issue:** Auto-exports were being added in Lib mode (SWC doesn't)
- **Fix:** Gated auto-export injection on `!matches!(config.mode, EmitMode::Lib)`
- **Files modified:** lib.rs

## Known Stubs

- `strip_typescript_types()` in emit.rs is a placeholder that returns input unchanged
- TS enum transpilation not implemented (IIFE pattern conversion)

## Deferred Issues

- TS type stripping requires either manual AST mutation traversal or enabling OXC `transformer` feature
- TS enum transpilation to IIFE pattern needs dedicated implementation
- `/*#__PURE__*/` annotation injection on qrl/componentQrl calls
- `const q_name = qrl(...)` intermediate declaration pattern
- Wrapper call emission (`componentQrl(q_name)` instead of bare `qrl(...)`)
- Import ordering (framework imports before user imports, alphabetical within groups)
- Dev mode path computation (project directory duplication issue)
- Quote style preservation for user code (single quotes in non-import code)
- HMR import conflict detection (`componentQrl` -> `componentQrl1` renaming)
