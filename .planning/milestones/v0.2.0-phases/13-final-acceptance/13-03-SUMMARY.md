---
phase: 13-final-acceptance
plan: 03
subsystem: codegen
tags: [oxc, codegen, indentation, parity]

requires:
  - phase: 13-01
    provides: "Baseline parity metrics and quote restoration"
provides:
  - "4-space indentation in all codegen output matching SWC"
  - "Full match improvement from 28 to 79 fixtures"
affects: [13-final-acceptance]

tech-stack:
  added: []
  patterns: ["OXC CodegenOptions IndentChar::Space with indent_width=4 for SWC parity"]

key-files:
  created: []
  modified:
    - "crates/qwik-optimizer-oxc/src/emit.rs"
    - "crates/qwik-optimizer-oxc/src/code_move.rs"
    - "crates/qwik-optimizer-oxc/src/transform.rs"
    - "crates/qwik-optimizer-oxc/src/inlined_fn.rs"

key-decisions:
  - "Used OXC CodegenOptions indent_char/indent_width (Approach 1) rather than post-processing tab replacement"

patterns-established:
  - "All Codegen::new() calls must use IndentChar::Space with indent_width=4 for SWC parity"

requirements-completed: [ACC-01]

duration: 4min
completed: 2026-04-06
---

# Phase 13 Plan 03: Indentation Fix Summary

**Configure OXC Codegen for 4-space indentation, improving full parity from 28 to 79 fixtures (39%)**

## Performance

- **Duration:** 4 min
- **Started:** 2026-04-06T20:05:45Z
- **Completed:** 2026-04-06T20:10:03Z
- **Tasks:** 1
- **Files modified:** 140 (4 source + 136 snapshots)

## Accomplishments

- Configured OXC Codegen to use `IndentChar::Space` with `indent_width: 4` across all codegen call sites (emit.rs, code_move.rs, transform.rs, inlined_fn.rs)
- Full parity improved from 28/201 (14%) to 79/201 (39%) -- the largest single improvement
- Root module match improved from 57/201 to 80/201
- All 514 tests pass (266 unit + 224 snapshot + 24 spec)

## Task Commits

1. **Task 1: Configure OXC Codegen for 4-space indentation** - `b988f75` (feat)

## Files Created/Modified

- `crates/qwik-optimizer-oxc/src/emit.rs` - Added IndentChar::Space + indent_width=4 to both source-map and non-source-map codegen paths
- `crates/qwik-optimizer-oxc/src/code_move.rs` - Same indent config for segment emission
- `crates/qwik-optimizer-oxc/src/transform.rs` - Same indent config for transform_code output
- `crates/qwik-optimizer-oxc/src/inlined_fn.rs` - Same indent config for inlined function expression codegen
- 136 insta snapshot files updated to reflect 4-space indentation

## Decisions Made

- Used OXC's native `CodegenOptions` indent configuration (Approach 1) rather than post-processing tab-to-space replacement. This is cleaner and avoids an extra string pass.

## Deviations from Plan

None - plan executed exactly as written.

## Issues Encountered

None.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Indentation parity resolved for all output
- Remaining root module mismatches (121 fixtures) are due to other code generation differences (not indentation)

## Known Stubs

None.

## Self-Check: PASSED

---
*Phase: 13-final-acceptance*
*Completed: 2026-04-06*
