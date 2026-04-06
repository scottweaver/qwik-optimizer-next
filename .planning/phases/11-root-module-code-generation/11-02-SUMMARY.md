---
phase: 11-root-module-code-generation
plan: 02
subsystem: root-module-cleanup
tags: [dead-code-elimination, fixpoint, import-removal, variable-migration]
dependency_graph:
  requires: [11-01]
  provides: [remove_unused_qrl_declarations]
  affects: [root-module-output, snapshot-tests]
tech_stack:
  added: []
  patterns: [fixpoint-loop, transitive-closure-propagation, SWC-parity]
key_files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/lib.rs
    - crates/qwik-optimizer-oxc/src/dependency_analysis.rs
    - crates/qwik-optimizer-oxc/tests/snapshots/ (100 snapshot files updated)
decisions:
  - "D-40: Extracted remove_unused_qrl_declarations as standalone function matching SWC architecture"
  - "D-41: Fixpoint only runs in non-Lib modes to preserve Lib mode inlined QRL behavior"
metrics:
  duration: 11m
  completed: 2026-04-06T14:18:42Z
  tasks_completed: 1
  tasks_total: 1
  files_modified: 102
---

# Phase 11 Plan 02: Dead Import and Variable Elimination Summary

SWC-matching fixpoint dead code elimination that removes unused _qrl_/i_ variable declarations AND unused imports after variable migration, raising root module parity from 36/201 to 76/201.

## What Was Done

### Task 1: Replace Step 9 with SWC-matching fixpoint dead code elimination

**Problem:** The existing Step 9 in `apply_variable_migration` only removed `_qrl_`/`i_`-prefixed variable declarations. It never removed unused imports. Additionally, it was nested inside `apply_variable_migration` which has an early return when no variables are migratable -- so the fixpoint never ran for most fixtures.

**Solution:**
1. Extracted the fixpoint loop into a standalone `remove_unused_qrl_declarations` function, matching SWC's architecture where it runs as a separate post-migration step
2. Implemented the full SWC algorithm with 4 phases per iteration:
   - Phase 1: Collect names defined by `_qrl_`/`i_` vars and ALL imports
   - Phase 2: Collect identifiers referenced by non-removable items (not vars/imports)
   - Phase 3: Transitive closure propagation (used vars mark their import dependencies as used)
   - Phase 4: Remove unused `_qrl_`/`i_` declarations and imports where ALL specifiers are unused
3. Added `visit_export_named_declaration` to `IdentRefCollector` for complete reference coverage
4. Placed the call site after `apply_variable_migration` with the same guards (non-Lib, segments present)

**Commit:** c12236c

## Results

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Root module match | 36/201 | 76/201 | +40 |
| Full match | 36/201 | 76/201 | +40 |
| Segment count match | 195/201 | 195/201 | No change |
| Diagnostics match | 193/201 | 193/201 | No change |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixpoint unreachable for most fixtures**
- **Found during:** Task 1 investigation
- **Issue:** The fixpoint loop was placed inside `apply_variable_migration` after the `if migratable.is_empty() { return; }` guard. For fixtures with no migratable vars (the majority), the fixpoint never executed.
- **Fix:** Extracted to standalone `remove_unused_qrl_declarations` function called separately from `transform_code`, matching SWC's architecture.
- **Files modified:** crates/qwik-optimizer-oxc/src/lib.rs

**2. [Rule 1 - Bug] Lib mode regression from unconditional fixpoint**
- **Found during:** Task 1 verification
- **Issue:** Initial implementation ran the fixpoint for ALL modes including Lib. Lib mode test `test_lib_mode_produces_no_separate_segments` failed because the fixpoint removed imports needed for inlined QRL output.
- **Fix:** Added `!matches!(config.mode, EmitMode::Lib)` guard to match SWC behavior (which only calls `remove_unused_qrl_declarations` after migration, which itself is skipped in Lib mode).
- **Files modified:** crates/qwik-optimizer-oxc/src/lib.rs

## Known Stubs

None -- all code paths are fully wired.

## Self-Check: PASSED
