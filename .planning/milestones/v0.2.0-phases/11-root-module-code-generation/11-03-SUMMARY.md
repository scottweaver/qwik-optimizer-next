---
phase: 11-root-module-code-generation
plan: 03
subsystem: transform
tags: [oxc, display-name, hash, stack-ctxt, segment-naming]

# Dependency graph
requires:
  - phase: 11-01
    provides: "Baseline root module generation pipeline"
provides:
  - "Correct display_name/hash computation matching SWC for all segments"
  - "Complete stack_ctxt population with var names, function names, JSX element/attr names, export default file stem, and call expression callee names"
affects: [11-04, 12-diagnostics]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "enter/exit Traverse hooks for stack_ctxt context building"
    - "Conditional marker name push (skip bare $ and sync$)"

key-files:
  created: []
  modified:
    - "crates/qwik-optimizer-oxc/src/transform.rs"
    - "crates/qwik-optimizer-oxc/src/lib.rs"

key-decisions:
  - "D-10: Gate marker name push for bare $ and sync$ to avoid empty string in display_name"
  - "D-11: Move stack_ctxt pop after register_context_name to include marker name in hash"
  - "D-12: Non-marker ident call expressions push callee name to match SWC fold_call_expr behavior"
  - "D-13: Fix SmartStrategy test to reflect correct component grouping with populated stack_ctxt"

patterns-established:
  - "stack_ctxt enter/exit symmetry: every Traverse enter_X push has a matching exit_X pop"
  - "SegmentScope.pushed_ctx_name tracks conditional pushes for balanced pop"
  - "call_name_pushed Vec<bool> tracks non-marker call expression callee pushes"

requirements-completed: [ROOT-04]

# Metrics
duration: 15min
completed: 2026-04-06
---

# Phase 11 Plan 03: Display Name / Hash Fix Summary

**Complete stack_ctxt population for SWC-matching display_name and segment hash computation across all 201 fixtures**

## Performance

- **Duration:** 15 min
- **Started:** 2026-04-06T09:00:00Z
- **Completed:** 2026-04-06T09:15:00Z
- **Tasks:** 1
- **Files modified:** 187 (2 source + 185 snapshots)

## Accomplishments
- Implemented 7 missing Traverse enter/exit hooks for stack_ctxt context building (variable declarators, function/class declarations, JSX elements, JSX attributes, export default declarations, non-marker call expressions)
- Fixed marker function name push to skip bare `$` and `sync$` (matching SWC behavior)
- Moved stack_ctxt pop to after register_context_name so marker name is included in hash computation
- Added file_stem and rel_dir fields to QwikTransform for export default naming
- All 478 tests pass (255 lib + 223 snapshot)

## Task Commits

Each task was committed atomically:

1. **Task 1: Add marker function name to stack_ctxt for segment display_name computation** - `f5e8dd5` (feat)

## Files Created/Modified
- `crates/qwik-optimizer-oxc/src/transform.rs` - Added 7 Traverse hooks for stack_ctxt, SegmentScope.pushed_ctx_name field, call_name_pushed tracking, file_stem/rel_dir fields
- `crates/qwik-optimizer-oxc/src/lib.rs` - Updated QwikTransform::new call with file_stem/rel_dir, fixed SmartStrategy test
- `crates/qwik-optimizer-oxc/tests/snapshots/` - 185 snapshot files updated with correct display names and hashes

## Decisions Made
- Gate the marker function name push for bare `$` and `sync$` calls to avoid empty-string context entries (SWC doesn't push for these)
- Move the stack_ctxt pop from before to after register_context_name so the full context including the marker name is available during hash computation
- Push callee ident for ALL non-special call expressions (matching SWC fold_call_expr else branch at line 4097)
- Updated SmartStrategy test to reflect correct behavior: component$ segments with parent variable context are grouped, not independent

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added 6 additional stack_ctxt enter/exit hooks beyond marker function push**
- **Found during:** Task 1
- **Issue:** Plan focused on marker function name push, but the root cause was that ALL stack_ctxt pushes were missing (variable declarators, function declarations, class declarations, JSX elements, JSX attributes, export default, non-marker calls). Without these, display_name was empty/generic.
- **Fix:** Implemented all 7 missing Traverse hooks to match SWC's stack_ctxt behavior
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** f5e8dd5

**2. [Rule 1 - Bug] Fixed pop ordering in exit_expression**
- **Found during:** Task 1
- **Issue:** stack_ctxt.pop() for marker name happened before register_context_name, excluding the marker name from hash computation
- **Fix:** Moved pop to after register_context_name call
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** f5e8dd5

**3. [Rule 1 - Bug] Fixed SmartStrategy test expecting incorrect behavior**
- **Found during:** Task 1
- **Issue:** Test expected is_entry=true for component$ segment, but with correct stack_ctxt the segment is correctly grouped with its parent context (is_entry=false)
- **Fix:** Updated test assertion and renamed to reflect correct behavior
- **Files modified:** crates/qwik-optimizer-oxc/src/lib.rs
- **Committed in:** f5e8dd5

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 missing critical)
**Impact on plan:** All fixes necessary for correct display_name computation. The plan underestimated the scope (only described marker function push), but the broader fix was required for correctness.

## Issues Encountered
- Root module parity stayed at 1/201 because root module differences are about import restructuring, const q_* declarations, PURE annotations, and comment separators -- not display names. Display names and hashes now match SWC.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Display names and segment hashes now match SWC across all fixtures
- Root module structure (imports, declarations, exports) still needs Phase 11 plans 01, 02, 04
- Segment count match is 125/201 (separate from display name issues)

---
*Phase: 11-root-module-code-generation*
*Completed: 2026-04-06*
