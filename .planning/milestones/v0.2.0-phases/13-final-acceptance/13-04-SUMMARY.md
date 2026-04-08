---
phase: 13-final-acceptance
plan: 04
subsystem: transform
tags: [qrl-naming, hash, collision-counter, swc-parity]

requires:
  - phase: 13-02
    provides: QRL const declarations and PURE annotations
  - phase: 13-03
    provides: Import/export parity fixes
provides:
  - QRL naming collision counter ordering matching SWC Fold semantics
  - Pre-registration of segment names at enter time for correct nested segment ordering
affects: [13-05, 13-06, 13-07, 13-08]

tech-stack:
  added: []
  patterns:
    - "Pre-register segment names at Traverse enter time to match SWC Fold ordering"

key-files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/transform.rs
    - crates/qwik-optimizer-oxc/tests/snapshots/ (17 snapshot files)

key-decisions:
  - "Pre-register names in enter_call_expression via SegmentScope.pre_registered_name to match SWC's Fold ordering where outer nodes call register_context_name before children"

patterns-established:
  - "SegmentScope pre-registration: outer segments must reserve collision counter slots before inner nested segments are visited"

requirements-completed: [ACC-01]

duration: 13min
completed: 2026-04-06
---

# Phase 13 Plan 04: Hash/QRL Naming Fix Summary

**Pre-register segment names at enter time to fix collision counter ordering, eliminating spurious _1 suffixes and matching SWC Fold semantics -- 89->95 full match (+6)**

## Performance

- **Duration:** 13 min
- **Started:** 2026-04-06T20:23:25Z
- **Completed:** 2026-04-06T20:37:00Z
- **Tasks:** 1
- **Files modified:** 18

## Accomplishments
- Fixed QRL naming collision counter ordering: outer segments now register before inner nested segments, matching SWC's Fold traversal order
- Eliminated spurious `_1` disambiguation suffixes (e.g., `Header_component_1_HASH` -> `Header_component_HASH`) across 17 fixtures
- Full match improved from 89/201 (44%) to 95/201 (47%), root match from 90 to 96
- All 514 tests pass with no regressions

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix hash computation and QRL naming** - `82c908b` (feat)

## Files Created/Modified
- `crates/qwik-optimizer-oxc/src/transform.rs` - Added `pre_registered_name` field to `SegmentScope` struct; pre-register names at enter_call_expression time; use pre-registered names in exit_expression
- `crates/qwik-optimizer-oxc/tests/snapshots/` (17 files) - Updated snapshots reflecting corrected QRL names and hash values

## Decisions Made
- Pre-register names in `enter_call_expression` rather than trying to reorder exit processing. This is the minimal change that correctly matches SWC's Fold ordering where outer nodes process (and register names) before their children are visited. The OXC Traverse pattern visits inner exit handlers before outer ones, causing the opposite registration order without pre-registration.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Fixed compilation error from parallel agent's extract_extra_args_code method**
- **Found during:** Task 1
- **Issue:** Another parallel agent added `extract_extra_args_code` method with lifetime `'a` that needed `'_` and a `.span()` call that needed `GetSpan` trait
- **Fix:** Changed lifetime to `'_` and used `oxc::span::GetSpan::span(e)` for fully qualified call
- **Files modified:** `crates/qwik-optimizer-oxc/src/transform.rs`
- **Verification:** Compilation succeeds
- **Committed in:** 82c908b

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Necessary to unblock compilation. No scope creep.

## Issues Encountered
- Parallel agent contention: another agent repeatedly modified `transform.rs` during editing, requiring Python-based atomic patching to avoid Edit tool read-after-write conflicts
- Root cause analysis required understanding SWC's Fold vs OXC's Traverse ordering semantics -- the collision counter itself was correct, but the registration ORDER differed due to different AST traversal patterns

## Next Phase Readiness
- Hash/naming fixes complete, remaining 106 root mismatches are structural (missing exports, wrong JSX transforms, etc.)
- Ready for next plan in phase 13

---
*Phase: 13-final-acceptance*
*Completed: 2026-04-06*
