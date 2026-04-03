---
phase: 08-implementation-gap-closure
plan: 05
subsystem: transform
tags: [oxc, import-stripping, separator-comments, parity, dead-import-elimination]

# Dependency graph
requires:
  - phase: 08-04
    provides: "Descriptive symbol naming (Header_component_J4uyIhaBNR4)"
provides:
  - "Consumed $-suffixed import stripping from root module"
  - "// separator comments between code sections"
  - "Dead import elimination for unused core module imports"
  - "Root-level-only QRL hoisting (child QRLs stay in parent segments)"
  - "Wrapper function synthetic imports (componentQrl, useTaskQrl, etc.)"
  - "Arrow spacing normalization in dynamic imports"
  - "57/201 root module parity (exceeds D-47 50/201 target)"
affects: [08-implementation-gap-closure, future-parity-work]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "Post-emit string processing for separator comments (emit.rs)"
    - "is_root_level field on HoistedConst for root vs child hoisting"
    - "IdentRefCollector Visit-based dead import elimination"

key-files:
  created: []
  modified:
    - "crates/qwik-optimizer-oxc/src/transform.rs"
    - "crates/qwik-optimizer-oxc/src/emit.rs"
    - "crates/qwik-optimizer-oxc/src/code_move.rs"

key-decisions:
  - "D-48: Strip ALL $-suffixed marker function imports, not just called ones"
  - "D-49: Dead import elimination skips Lib mode (different code generation patterns)"
  - "D-50: Separator comments use post-emit string insertion (not AST comment nodes)"
  - "D-51: Root-level hoisting via is_root_level field on HoistedConst"

patterns-established:
  - "Post-emit normalization pipeline: PURE annotations -> arrow spacing -> separator comments"
  - "Consumed import tracking via marker_functions HashMap keys"

requirements-completed: [IMPL-05]

# Metrics
duration: 21min
completed: 2026-04-03
---

# Phase 08 Plan 05: Import Stripping and Separator Comments Summary

**Strip consumed imports, add separator comments, and achieve 57/201 root module parity with SWC output**

## Performance

- **Duration:** 21 min
- **Started:** 2026-04-03T21:20:38Z
- **Completed:** 2026-04-03T21:41:00Z
- **Tasks:** 2
- **Files modified:** 189 (3 source + 186 snapshots)

## Accomplishments
- Root module parity improved from 1/201 to 57/201 (exceeds D-47 target of 50/201)
- Full match improved from 1/201 to 28/201 (14%)
- Consumed $-suffixed marker function imports stripped from root module output
- Separator comments (//) emitted between imports, hoisted QRL consts, and body
- Dead import elimination removes unused core module imports (like `onRender`)
- Child segment QRLs no longer hoisted to root module (only root-level QRLs)
- JSX import flags only set for root-scope JSX (not segment-scope)
- Wrapper function synthetic imports added (componentQrl, useTaskQrl, etc.)

## Task Commits

Each task was committed atomically:

1. **Task 1: Strip consumed imports from root module** - `cf5ad0b` (feat)
2. **Task 2: Add separator comments and measure final parity** - `3375261` (feat)

## Files Created/Modified
- `crates/qwik-optimizer-oxc/src/transform.rs` - consumed_imports tracking, marker function stripping, dead import elimination, root-level hoisting, JSX flag scoping, wrapper import generation
- `crates/qwik-optimizer-oxc/src/emit.rs` - separator comment insertion, arrow spacing normalization
- `crates/qwik-optimizer-oxc/src/code_move.rs` - is_root_level field propagation in HoistedConst

## Decisions Made
- Strip ALL $-suffixed marker function imports from core module (not just called ones), matching SWC behavior
- Dead import elimination uses Visit-based IdentRefCollector to find unreferenced core imports; skips Lib mode where code generation uses different patterns
- Separator comments implemented as post-emit string processing rather than AST comment manipulation (simpler, more reliable)
- Root-level QRL hoisting tracked via `is_root_level` field on HoistedConst -- only root-level consts emitted in exit_program; child consts handled by code_move

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing Critical] Added wrapper function synthetic imports**
- **Found during:** Task 1
- **Issue:** componentQrl/useTaskQrl wrapper functions used in body but no import added
- **Fix:** Collect wrapper names from segment ctx_names and add as synthetic imports
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** cf5ad0b

**2. [Rule 1 - Bug] Fixed non-deterministic import ordering**
- **Found during:** Task 1
- **Issue:** HashSet for wrapper imports caused non-deterministic ordering across runs
- **Fix:** Changed to BTreeSet for deterministic sorted ordering
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** cf5ad0b

**3. [Rule 2 - Missing Critical] Root-level-only QRL hoisting**
- **Found during:** Task 2
- **Issue:** Child segment QRLs incorrectly hoisted to root module (should be in parent segment)
- **Fix:** Added is_root_level field to HoistedConst, filter in exit_program
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs, code_move.rs
- **Committed in:** 3375261

**4. [Rule 2 - Missing Critical] Scoped JSX import flags**
- **Found during:** Task 2
- **Issue:** JSX imports (jsxSorted, etc.) added to root module even when JSX only in segments
- **Fix:** Only set needs_jsx_* flags when segment_stack is empty (root scope)
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** 3375261

**5. [Rule 2 - Missing Critical] Dead import elimination**
- **Found during:** Task 2
- **Issue:** Unused non-$ core imports (e.g., onRender) not stripped, hurting parity
- **Fix:** Added IdentRefCollector to scan for unreferenced core imports
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** 3375261

**6. [Rule 1 - Bug] Arrow spacing normalization**
- **Found during:** Task 2
- **Issue:** OXC codegen emits `() => import(...)` but SWC uses `()=>import(...)`
- **Fix:** Post-emit replacement in emit.rs normalize_pure_annotations
- **Files modified:** crates/qwik-optimizer-oxc/src/emit.rs
- **Committed in:** 3375261

---

**Total deviations:** 6 auto-fixed (4 missing critical, 2 bugs)
**Impact on plan:** All deviations directly contributed to achieving the 57/201 parity target. Without these fixes, parity would have remained at 22/201.

## Parity Report (Final)

| Metric | Before (08-04) | After (08-05) |
|--------|----------------|---------------|
| Root module match | 1/201 | 57/201 |
| Full match | 1/201 | 28/201 |
| Segment count match | 125/201 | 125/201 |
| Diagnostics match | 197/201 | 197/201 |

## Remaining Root Module Differences (144/201 mismatches)

Common remaining issues:
- Hash differences in symbol names (non-matching hashes between OXC and SWC)
- `const` vs bare expression for non-exported variable declarations
- Single vs double quote differences in import source strings
- Import ordering differences (SWC uses different insertion order)
- Some fixtures have deeper structural differences (inline vs segment strategy)

## Issues Encountered
None.

## User Setup Required
None - no external service configuration required.

## Next Phase Readiness
- IMPL-05 parity target achieved (57/201 > 50/201)
- All 502 tests pass
- No regressions in segment count (125/201) or diagnostics (197/201)
- Further parity improvements possible but require addressing hash computation, quote normalization, and deeper structural alignment

---
*Phase: 08-implementation-gap-closure*
*Completed: 2026-04-03*
