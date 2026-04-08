---
phase: 13-final-acceptance
plan: 08
subsystem: codegen
tags: [oxc, codegen, parity, arrow-spacing, jsx-keys, import-sorting]

requires:
  - phase: 13-final-acceptance (plans 01-07)
    provides: 99/201 parity baseline with segment count and diagnostics at 196/201 and 201/201

provides:
  - Arrow function spacing normalization (OXC => SWC format)
  - Alphabetical sorting of hoisted const q_* declarations
  - File hash prefix for JSX dev keys (u6_N format)
  - Locally-defined wrapper import elimination
  - Synthesized import sorting

affects: [final-acceptance, parity]

tech-stack:
  added: []
  patterns:
    - "Post-processing pipeline in emit.rs for OXC->SWC codegen normalization"
    - "SipHash file hash prefix for JSX key generation"

key-files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/emit.rs
    - crates/qwik-optimizer-oxc/src/transform.rs
    - crates/qwik-optimizer-oxc/src/jsx_transform.rs
    - crates/qwik-optimizer-oxc/src/hash.rs
    - crates/qwik-optimizer-oxc/src/code_move.rs
    - crates/qwik-optimizer-oxc/src/inlined_fn.rs

key-decisions:
  - "Arrow spacing normalization as post-processing (OXC codegen has no option to control this)"
  - "Alphabetical sorting of hoisted consts matches SWC BTreeMap iteration order"
  - "JSX key prefix uses SipHash of scope+rel_path matching SWC's DefaultHasher approach"

patterns-established:
  - "emit.rs post-processing pipeline: normalize_pure -> inject_pure -> preserve_quotes -> sort_imports -> sort_hoisted_consts -> insert_separators"

requirements-completed: []

duration: 31min
completed: 2026-04-06
---

# Phase 13 Plan 08: Final Straggler Sweep Summary

**Arrow spacing normalization, hoisted const sorting, and JSX key prefix fixes bring parity from 99/201 to 107/201 (+8 fixtures)**

## Performance

- **Duration:** 31 min
- **Started:** 2026-04-06T21:29:21Z
- **Completed:** 2026-04-06T22:00:21Z
- **Tasks:** 1
- **Files modified:** 154

## Accomplishments

- Arrow function spacing normalization: `() =>` to `()=>` matching SWC output across all codegen paths
- Hoisted `const q_*` declarations sorted alphabetically to match SWC BTreeMap ordering
- JSX dev keys now use file hash prefix (`"u6_0"`) matching SWC format
- Locally-defined wrapper functions (e.g., `useMemoQrl`) no longer generate spurious imports
- Import ordering: synthesized imports sorted alphabetically before user imports

## Task Commits

Each task was committed atomically:

1. **Task 1a: Arrow spacing + const sorting + wrapper imports** - `5d0781d` (feat)
2. **Task 1b: JSX key file hash prefix** - `49d17d1` (feat)
3. **Task 1c: Import sorting + sort_hoisted_consts fix** - `1053aae` (feat)

## Files Created/Modified

- `crates/qwik-optimizer-oxc/src/emit.rs` - Added normalize_arrow_spacing, sort_hoisted_consts, sort_imports post-processing
- `crates/qwik-optimizer-oxc/src/transform.rs` - Added jsx_key_prefix field, locally-defined wrapper import skip, HoistedConst span fields
- `crates/qwik-optimizer-oxc/src/jsx_transform.rs` - Updated gen_jsx_key with prefix parameter, threaded through classify/transform functions
- `crates/qwik-optimizer-oxc/src/hash.rs` - Added compute_file_hash_prefix for JSX key generation
- `crates/qwik-optimizer-oxc/src/code_move.rs` - Arrow spacing normalization in segment codegen
- `crates/qwik-optimizer-oxc/src/inlined_fn.rs` - Arrow spacing normalization in inlined fn codegen
- `crates/qwik-optimizer-oxc/tests/snapshots/*.snap` - 150+ snapshot updates for arrow formatting and key format

## Decisions Made

1. **Arrow spacing as post-processing**: OXC codegen has no option to omit spaces around `=>`. Post-processing with string replacement is the only approach. Handles string-safety via per-line quote tracking.

2. **Alphabetical const sorting**: Verified that ALL 201 SWC expected snapshots have `const q_*` declarations in alphabetical order, matching BTreeMap iteration. Simple sort in emit.rs resolves all ordering mismatches.

3. **JSX key prefix**: SWC uses `base64(SipHash(scope+rel_path))[0..2]` as key prefix. Implemented matching computation using existing SipHash infrastructure.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Non-sourcemap emit path missing sort_hoisted_consts**
- **Found during:** Task 1c
- **Issue:** sort_hoisted_consts was only called in the source-maps branch of emit_module, not the non-source-maps branch
- **Fix:** Added sort_hoisted_consts call to both code paths
- **Files modified:** crates/qwik-optimizer-oxc/src/emit.rs

---

**Total deviations:** 1 auto-fixed (1 blocking)
**Impact on plan:** Minor omission caught during implementation.

## Remaining Gap Analysis

**Current parity: 107/201 (53%)**

The remaining 94 mismatches fall into categories requiring deeper architectural work:

| Category | Count | Description |
|----------|-------|-------------|
| Signal optimization | 21 | _fnSignal/_wrapProp not applied in component bodies |
| Import content | 31 | Wrong imports due to signal opt gap and inline body differences |
| Other structural | 38 | Object formatting, dead code, bare vs const expressions |
| Order-only | 5 | Comment and import ordering edge cases |

**Root cause:** The signal optimization system (_fnSignal, _wrapProp, q:p prop injection) is partially implemented but not fully applied to JSX attribute values in component bodies. This is the single largest remaining gap and fixing it would cascade-fix many import and structural mismatches.

**Recommendation:** A dedicated phase focused on signal optimization implementation would address 40-50 of the remaining mismatches.

## Issues Encountered

- Signal optimization (_fnSignal/_wrapProp) requires substantial JSX attribute analysis that is beyond the scope of a straggler sweep
- SWC's import ordering depends on BTreeMap iteration and the order of synthetic import accumulation during Fold traversal, which is complex to replicate exactly
- Some fixtures have bare `qrl()` expression statements instead of `const q_` assignments; SWC only creates const bindings when the QRL is referenced by a wrapper call

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

- Parity at 107/201 (53%) -- significant improvement from 99/201 baseline
- Signal optimization is the clear next priority for further parity improvement
- All 514 tests passing with no regressions
- Diagnostics remain at 201/201 (100%) and segment count at 196/201 (97%)

---
*Phase: 13-final-acceptance*
*Completed: 2026-04-06*
