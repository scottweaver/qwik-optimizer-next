---
phase: 12-diagnostics-parity
plan: 01
subsystem: transform
tags: [oxc, diagnostics, C02, C03, C05, parity]

# Dependency graph
requires:
  - phase: 11-root-module-code-generation
    provides: transform.rs with C02 diagnostic emission
provides:
  - "201/201 diagnostic parity with SWC reference implementation"
  - "C02 export-symbol gate preventing false positives"
  - "C05 MissingQrlImplementation diagnostic"
  - "C03 ordering fix (classify_captures before C03 check)"
affects: [13-final-verification]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "classify_captures before C03 check (module-level declarations must be reclassified first)"
    - "marker_fn_sources lookup to distinguish imported from locally-defined marker functions"

key-files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/transform.rs

key-decisions:
  - "Move classify_captures before C03 check to prevent false positives from module-level declarations"
  - "Suppress C03 when first arg is a simple identifier (matches SWC const-inlining behavior)"
  - "Use marker_fn_sources to distinguish imported from local $-functions for C05 check"

patterns-established:
  - "C02 gate pattern: has_export_symbol + root.contains_key + self_imports check"
  - "C05 placement: before segment creation in exit_call_expression, with early return"

requirements-completed: [DIAG-01, DIAG-02]

# Metrics
duration: 12min
completed: 2026-04-06
---

# Phase 12 Plan 01: Diagnostics Parity Summary

**C02 export-symbol gate + C03 ordering fix + C05 MissingQrlImplementation -- diagnostics 192/201 to 201/201**

## Performance

- **Duration:** 12 min
- **Started:** 2026-04-06T17:04:06Z
- **Completed:** 2026-04-06T17:16:14Z
- **Tasks:** 2
- **Files modified:** 10

## Accomplishments
- Achieved 201/201 diagnostic parity with SWC reference implementation
- Fixed 8 C02 false positives by adding export-symbol, root, and self-imports gate
- Fixed 6 C03 false positives by reordering classify_captures before C03 check and suppressing C03 for simple identifier first args
- Implemented C05 MissingQrlImplementation diagnostic for locally-defined $-functions missing Qrl exports
- Segment count improved from 194/201 to 195/201 (C05 correctly prevents segment extraction)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add export-symbol gate to C02 and fix C03 false positives** - `a195470` (fix)
2. **Task 2: Implement C05 MissingQrlImplementation diagnostic** - `dc99dcb` (feat)

## Files Created/Modified
- `crates/qwik-optimizer-oxc/src/transform.rs` - C02 gate, C03 ordering fix, C05 implementation
- `crates/qwik-optimizer-oxc/tests/snapshot_tests.rs` - Removed debug code
- `crates/qwik-optimizer-oxc/tests/snapshots/...` - 8 updated snapshot files reflecting corrected diagnostics

## Decisions Made
- **Move classify_captures before C03:** Module-level declarations (self-imports) must be reclassified before C03 checks, otherwise C03 falsely fires on identifiers that are module-level consts/vars (not actual captures)
- **Suppress C03 for identifier first args:** SWC inlines const initializers before C03 check, which either converts to function expressions or eliminates captures. OXC equivalent: when first arg is a simple identifier, skip C03 (the identifier is the value, not a captured variable)
- **Use marker_fn_sources for C05:** ctx_name for imported functions is the specifier (not local name), so collect.imports.contains_key doesn't work for aliased imports. marker_fn_sources maps specifiers to sources only for imported marker functions

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Fixed C03 false positives from module-level declarations**
- **Found during:** Task 1 (C02 gate implementation)
- **Issue:** 6 of the 8 "diagnostics" mismatches were actually C03 false positives, not C02. Module-level declarations (e.g., STYLES, rawFn, FeatureSchema) were not reclassified as self-imports before the C03 capture check
- **Fix:** Moved classify_captures call before C03 check in exit_call_expression
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** a195470

**2. [Rule 1 - Bug] Fixed C03 false positives for identifier-ref first arguments**
- **Found during:** Task 1 (after C03 ordering fix, 2 fixtures still mismatched)
- **Issue:** example_invalid_segment_expr1 still had C03 false positives for `style` and `render` variables passed directly as identifiers to $ calls (e.g., `$(render)`, `useStyles$(style)`). SWC avoids this by inlining const initializers before C03 check
- **Fix:** Added check to suppress C03 when first argument is a simple Identifier
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** a195470

**3. [Rule 1 - Bug] Fixed C05 check for aliased imports**
- **Found during:** Task 2 (C05 implementation)
- **Issue:** Initial C05 check used collect.imports.contains_key(&ctx_name) which failed for aliased imports (e.g., `$ as onRender` where ctx_name="$" but imports key is "onRender"), causing false C05 for example_renamed_exports
- **Fix:** Changed to marker_fn_sources.contains_key(&ctx_name) which correctly distinguishes imported from locally-defined marker functions
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** dc99dcb

---

**Total deviations:** 3 auto-fixed (3 bugs)
**Impact on plan:** All auto-fixes necessary for correctness. No scope creep. The plan correctly identified the C02 and C05 issues but the C03 false positives and aliased import edge case were discovered during implementation.

## Issues Encountered
None beyond the deviations documented above.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Diagnostics parity is complete (201/201)
- Segment count parity improved to 195/201
- Root module parity remains at 80/201 (separate concern)
- Ready for Phase 13 final verification

---
*Phase: 12-diagnostics-parity*
*Completed: 2026-04-06*
