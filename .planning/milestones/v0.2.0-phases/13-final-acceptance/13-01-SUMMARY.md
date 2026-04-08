---
phase: 13-final-acceptance
plan: 01
subsystem: testing
tags: [oxc, codegen, quote-style, diagnostics, parity]

# Dependency graph
requires:
  - phase: 11-root-module-code-generation
    provides: "Root module emit pipeline (emit.rs)"
provides:
  - "parity_diff_report diagnostic test for categorized mismatch analysis"
  - "Quote style preservation for user-written import specifiers"
affects: [13-02, 13-03, 13-04, 13-05, 13-06, 13-07, 13-08]

# Tech tracking
tech-stack:
  added: []
  patterns: ["Post-processing quote restoration via source correlation"]

key-files:
  created: []
  modified:
    - "crates/qwik-optimizer-oxc/src/emit.rs"
    - "crates/qwik-optimizer-oxc/tests/snapshot_tests.rs"

key-decisions:
  - "Binding-level matching for quote restoration: compare import names (not just module specifier) to distinguish user vs synthesized imports"
  - "Line-aligned diff categorization with dehash helper for hash-insensitive comparison"

patterns-established:
  - "preserve_original_quotes: post-processing step correlating output imports with source imports by binding names"

requirements-completed: [ACC-01]

# Metrics
duration: 6min
completed: 2026-04-06
---

# Phase 13 Plan 01: Diagnostic Triage and Quote Style Fix Summary

**Categorized diff diagnostic for 121 root mismatches plus quote style preservation restoring single quotes on user-written imports**

## Performance

- **Duration:** 6 min
- **Started:** 2026-04-06T19:59:57Z
- **Completed:** 2026-04-06T20:06:00Z
- **Tasks:** 2
- **Files modified:** 36 (1 emit.rs + 1 snapshot_tests.rs + 34 updated insta snapshots)

## Accomplishments
- Created `parity_diff_report` diagnostic test that categorizes all 121 root-mismatched fixtures into STRUCTURAL (110), MISSING_QRL_CONST (10), MISSING_PURE (1) categories
- Implemented `preserve_original_quotes` post-processing in emit.rs that restores single quotes on user-written imports while keeping double quotes on synthesized imports
- Updated 34 insta snapshots to reflect corrected quote output
- No parity regressions: 79/201 full match maintained, all 514 tests passing

## Task Commits

Each task was committed atomically:

1. **Task 1: Create parity_diff_report diagnostic test** - `188aee5` (feat)
2. **Task 2: Fix quote style preservation in root module emit** - `5af4eb0` (feat)

## Files Created/Modified
- `crates/qwik-optimizer-oxc/src/emit.rs` - Added preserve_original_quotes, try_restore_single_quotes, extract_import_names functions
- `crates/qwik-optimizer-oxc/tests/snapshot_tests.rs` - Added parity_diff_report test, categorize_diff, dehash helpers
- `crates/qwik-optimizer-oxc/tests/snapshots/*.snap` - 34 snapshot files updated with corrected quote style

## Decisions Made
- Binding-level matching: To distinguish user-written imports from synthesized ones (both may reference same module like `@qwik.dev/core`), we compare the imported binding names. If any binding in the output import line also appears in a source import line with single quotes, it's user-written.
- Simple line-aligned diff for diagnostic (not LCS): Sufficient for diagnosis since most mismatches are structural and easy to spot visually.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Updated 34 insta snapshots after quote fix**
- **Found during:** Task 2 (quote preservation)
- **Issue:** Changing quote output in emit.rs caused 34 existing insta snapshot tests to fail
- **Fix:** Ran `cargo insta test --accept` to update snapshots to new correct output
- **Files modified:** 34 snapshot files under crates/qwik-optimizer-oxc/tests/snapshots/
- **Verification:** All 514 tests pass, parity count unchanged at 79/201
- **Committed in:** 5af4eb0 (Task 2 commit)

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Expected consequence of quote fix. No scope creep.

## Issues Encountered
None

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Diagnostic report reveals 110 STRUCTURAL, 10 MISSING_QRL_CONST, 1 MISSING_PURE remaining mismatches
- Quote style is now correct for all user-written imports
- Ready for plans 02-08 to address remaining root module mismatches

---
*Phase: 13-final-acceptance*
*Completed: 2026-04-06*
