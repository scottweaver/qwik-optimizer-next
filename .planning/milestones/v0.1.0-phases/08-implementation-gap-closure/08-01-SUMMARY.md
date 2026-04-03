---
phase: 08-implementation-gap-closure
plan: 01
subsystem: transform
tags: [qrl-hoisting, pure-annotations, segment-strategy, oxc, codegen]

# Dependency graph
requires:
  - phase: 05-core-oxc-implementation
    provides: QwikTransform with Segment/Hoist/Inline strategy paths, extra_top_items hoisting, HoistedConst struct
provides:
  - QRL hoisting for Segment strategy (const q_X = /*#__PURE__*/ qrl(...) at module scope)
  - PURE annotation normalization (OXC -> SWC format) in emit.rs and code_move.rs
  - Broadened exit_program hoisting emission to all strategies
affects: [08-02, 08-03, parity-improvement]

# Tech tracking
tech-stack:
  added: []
  patterns:
    - "PURE annotation normalization: post-process OXC codegen output to replace /* @__PURE__ */ with /*#__PURE__*/"
    - "Segment strategy QRL hoisting: reuse Hoist strategy extra_top_items infrastructure"

key-files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/transform.rs
    - crates/qwik-optimizer-oxc/src/emit.rs
    - crates/qwik-optimizer-oxc/src/code_move.rs

key-decisions:
  - "D-49: Segment strategy reuses existing extra_top_items/HoistedConst infrastructure from Hoist strategy"
  - "D-50: PURE annotation normalization done as string post-processing in codegen, not AST-level comment manipulation"
  - "D-51: Parity target of 30/201 not achievable due to pre-existing symbol naming and import stripping gaps"

patterns-established:
  - "PURE normalization: code.replace('/* @__PURE__ */', '/*#__PURE__*/') applied in emit_module and emit_segment"
  - "Strategy-agnostic hoisting: exit_program checks !extra_top_items.is_empty() instead of EntryStrategy::Hoist"

requirements-completed: [IMPL-02, IMPL-05]

# Metrics
duration: 12min
completed: 2026-04-03
---

# Phase 8 Plan 01: QRL Hoisting and PURE Annotation Summary

**Segment strategy QRL calls hoisted to module scope with PURE annotations, matching SWC structural pattern across 159 fixtures**

## Performance

- **Duration:** 12 min
- **Started:** 2026-04-03T19:58:36Z
- **Completed:** 2026-04-03T20:10:36Z
- **Tasks:** 2 (1 implementation, 1 verification-only)
- **Files modified:** 3 source files + 167 snapshot files

## Accomplishments
- Segment strategy QRL calls now hoisted to module scope as `const q_X = /*#__PURE__*/ qrl(...)` matching SWC output structure
- Call sites correctly replaced: bare `$()` -> identifier, `component$()` -> `/*#__PURE__*/ componentQrl(q_X)`, with `.w([caps])` for captures
- exit_program hoisting emission generalized to work for all strategies (not just Hoist)
- PURE annotation normalization applied in both root module and segment codegen paths
- All 478 tests pass (255 unit + 223 integration, 0 failures)

## Task Commits

Each task was committed atomically:

1. **Task 1: Implement Segment strategy QRL hoisting with PURE annotations** - `2aeaa91` (feat)
2. **Task 2: Fix import format** - No code changes needed; import format already matches one-import-per-specifier pattern

**Plan metadata:** (pending final commit)

## Files Created/Modified
- `crates/qwik-optimizer-oxc/src/transform.rs` - Segment strategy branch rewritten to hoist QRLs; exit_program guard broadened
- `crates/qwik-optimizer-oxc/src/emit.rs` - PURE annotation normalization in emit_module
- `crates/qwik-optimizer-oxc/src/code_move.rs` - PURE annotation normalization in emit_segment
- `crates/qwik-optimizer-oxc/tests/snapshots/*.snap` - 167 snapshot files updated to reflect new hoisting output

## Decisions Made
- D-49: Reused Hoist strategy's `extra_top_items` / `HoistedConst` infrastructure for Segment strategy hoisting, avoiding code duplication
- D-50: Applied PURE annotation normalization as string post-processing (`/* @__PURE__ */` -> `/*#__PURE__*/`) because OXC codegen always normalizes PURE comments to `@` format
- D-51: The plan's parity target of 30/201 root module matches is not achievable in this plan alone. Pre-existing gaps (symbol naming differences, consumed import stripping, original import retention) prevent root module matches regardless of hoisting correctness. The structural change (hoisting) is correct and verified against SWC reference patterns.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] PURE annotation format normalization**
- **Found during:** Task 1
- **Issue:** OXC codegen emits `/* @__PURE__ */` but SWC uses `/*#__PURE__*/`. Parsing PURE comments through OXC always normalizes them.
- **Fix:** Added string-based post-processing in emit.rs::emit_module and code_move.rs::emit_segment to replace OXC format with SWC format
- **Files modified:** emit.rs, code_move.rs
- **Verification:** Snapshot output shows correct `/*#__PURE__*/` format
- **Committed in:** 2aeaa91

**2. [Rule 1 - Bug] is_bare_dollar check used wrong comparison**
- **Found during:** Task 1
- **Issue:** Initial `is_bare_dollar` check compared `qrl_wrapper_name == "qrl"` but `dollar_to_qrl_name("$")` returns `"Qrl"` (capital Q). This caused bare `$()` calls to be wrapped in `Qrl(q_sym)` instead of replaced with just `q_sym`.
- **Fix:** Changed check to `pending.ctx_name == "$"` which correctly identifies bare dollar calls
- **Files modified:** transform.rs
- **Committed in:** 2aeaa91

---

**Total deviations:** 2 auto-fixed (1 blocking, 1 bug)
**Impact on plan:** Both fixes necessary for correctness. No scope creep.

## Issues Encountered

**Parity target not achievable:** The plan's acceptance criterion of "root module match count >= 30/201" cannot be met because root module parity is blocked by pre-existing issues unrelated to QRL hoisting:
1. **Symbol naming**: Our optimizer generates different display names and hashes than SWC (e.g., `_u6BtPnK76q4` vs `Header_component_J4uyIhaBNR4`)
2. **Consumed import stripping**: SWC strips consumed imports (`$`, `onRender`, etc.) from the import list; our code retains all original imports
3. **Import renaming**: SWC renames `component$` to `componentQrl` in the import declaration; we add a separate synthetic import

These are deeper architectural issues that require separate plans to address.

## Known Stubs

None -- all functionality is wired and operational.

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness
- QRL hoisting is structurally correct, ready for signal optimization wiring (Plan 02) and spec examples activation (Plan 03)
- Import stripping and symbol naming fixes needed in a future plan for parity improvement

---
*Phase: 08-implementation-gap-closure*
*Completed: 2026-04-03*
