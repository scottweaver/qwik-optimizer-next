---
phase: 13-final-acceptance
plan: 02
subsystem: codegen
tags: [oxc, qrl, pure-annotation, const-stripping, tree-shaking, root-module]

requires:
  - phase: 13-01
    provides: quote style preservation and parity diff diagnostic test
provides:
  - "const q_* hoisted QRL declarations preserved through const stripping"
  - "PURE annotations injected on all wrapper calls (componentQrl, _jsxSorted, _jsxSplit, _noopQrl)"
  - "10 additional full parity matches (79->89/201)"
affects: [13-03, 13-04, 13-05, 13-06, 13-07, 13-08]

tech-stack:
  added: []
  patterns: ["post-processing PURE annotation injection via string matching in emit.rs"]

key-files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/lib.rs
    - crates/qwik-optimizer-oxc/src/emit.rs

key-decisions:
  - "Exclude q_* names from const stripping candidates rather than changing reference scanning logic"
  - "Use post-processing string injection for PURE annotations rather than AST-level comment manipulation"
  - "PURE annotation injection applied globally (root + segments) matching SWC behavior"

patterns-established:
  - "const stripping exclusion: names starting with q_ are hoisted QRL references, never strip candidates"
  - "PURE injection pipeline: normalize_pure_annotations -> inject_pure_annotations -> preserve_original_quotes"

requirements-completed: [ACC-01]

duration: 15min
completed: 2026-04-06
---

# Phase 13 Plan 02: QRL Const Declarations and PURE Annotations Summary

**Fixed const stripping bug preserving q_* hoisted QRL declarations, added PURE annotations on wrapper calls (componentQrl, _jsxSorted, _jsxSplit, _noopQrl) -- 79->89 full parity matches**

## Performance

- **Duration:** 15 min
- **Started:** 2026-04-06T20:05:40Z
- **Completed:** 2026-04-06T20:20:41Z
- **Tasks:** 2
- **Files modified:** 49 (1 source + 47 snapshots for Task 2, 1 source for Task 1)

## Accomplishments
- Fixed critical const stripping bug in `strip_unreferenced_wrapper_consts_text` that was removing `const q_*` hoisted QRL declarations, creating dangling references in the root module (e.g., `component(q_foo)` with no matching const)
- Added post-processing PURE annotation injection for wrapper calls (`componentQrl`, `_jsxSorted`, `_jsxSplit`, `_noopQrl`, `_noopQrlDEV`) matching SWC tree-shaking hint behavior
- Parity improvement: 79->89 full match (13%), 80->90 root module match -- 10 MISSING_QRL_CONST fixtures fully resolved

## Task Commits

Each task was committed atomically:

1. **Task 1: Fix const stripping bug and emit const q_NAME declarations** - `ca2b390` (fix)
2. **Task 2: Add PURE annotations to wrapper calls in root module body** - `0d3f1f8` (feat)

## Files Created/Modified
- `crates/qwik-optimizer-oxc/src/lib.rs` - Added q_* exclusion guard in const stripping candidate filter
- `crates/qwik-optimizer-oxc/src/emit.rs` - Added inject_pure_annotations post-processing function
- `crates/qwik-optimizer-oxc/tests/snapshots/*.snap` - 47 snapshot files updated with correct PURE annotations

## Decisions Made
- Excluded q_* names from const stripping candidates: the root cause was that `const q_*` lines had `__PURE__` in their RHS, making them candidates, and their references lived on OTHER candidate lines (excluded from reference scanning), causing them to appear unreferenced
- Used post-processing string injection for PURE: OXC codegen does preserve AST comments, but the wrapper calls are created by callee renaming (component$ -> componentQrl) at the AST level without adding comments; post-processing is simpler and avoids OXC comment API complexities
- Investigated but rejected excluding all PURE-annotated consts from stripping: SWC DOES strip unreferenced PURE consts (e.g., `const App = componentQrl(...)` -> bare `componentQrl(...)`), so the PURE exclusion caused regressions in 8 fixtures

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Investigated PURE-annotated const exclusion regression**
- **Found during:** Task 2 (PURE annotation investigation)
- **Issue:** Initial approach of excluding PURE-annotated lines from const stripping candidates caused 8 regressions (89->81 parity) because SWC DOES strip unreferenced PURE wrapper consts
- **Fix:** Reverted PURE exclusion approach, used post-processing string injection instead
- **Files modified:** crates/qwik-optimizer-oxc/src/lib.rs (reverted), crates/qwik-optimizer-oxc/src/emit.rs (new function)
- **Verification:** Parity maintained at 89/201 with no regressions

---

**Total deviations:** 1 auto-fixed (1 bug)
**Impact on plan:** Approach 2 (post-processing) used for PURE annotations instead of approach 1 (AST-level). No scope creep.

## Issues Encountered
- Full match target of 100/201 not reached (achieved 89/201). The remaining 111 mismatches are STRUCTURAL issues (ordering differences, import stripping, expression formatting) beyond the scope of this plan's const stripping and PURE annotation fixes. The MISSING_QRL_CONST category (10 fixtures) was fully resolved. MISSING_PURE (1 fixture) has the annotation but is still classified as MISSING_PURE due to line alignment differences caused by other structural issues in the same fixture.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- Const stripping and PURE annotation issues resolved
- 111 remaining root module mismatches are STRUCTURAL (ordering, import stripping, formatting)
- Ready for subsequent plans targeting structural fixes

## Self-Check: PASSED

- FOUND: crates/qwik-optimizer-oxc/src/lib.rs
- FOUND: crates/qwik-optimizer-oxc/src/emit.rs
- FOUND: ca2b390 (Task 1 commit)
- FOUND: 0d3f1f8 (Task 2 commit)

---
*Phase: 13-final-acceptance*
*Completed: 2026-04-06*
