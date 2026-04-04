---
phase: 10-segment-extraction
plan: 02
subsystem: optimizer
tags: [oxc, qwik, segment-extraction, noop-qrl, strip-ctx-name, non-function-args, c05-diagnostic]

requires:
  - phase: 10-01
    provides: "JSX dollar-attr segment extraction (180/201 parity)"
provides:
  - "Non-function first arg segment extraction for useStyles$, qwikify$, serverAuth$, $() etc."
  - "Correct strip_ctx_name prefix matching (starts_with instead of exact equality)"
  - "Noop segments always produce separate module files regardless of entry strategy"
  - "C05 diagnostic for locally-defined marker functions missing Qrl counterpart"
affects: [11-root-module, 12-diagnostics, 13-acceptance]

tech-stack:
  added: []
  patterns:
    - "Non-function arg segment extraction using GetSpan for universal span extraction"
    - "Noop segments always is_inline: false for separate module generation"
    - "strip_ctx_name prefix matching via starts_with (matching SWC behavior)"

key-files:
  created: []
  modified:
    - "crates/qwik-optimizer-oxc/src/transform.rs"
    - "crates/qwik-optimizer-oxc/tests/spec_examples.rs"

key-decisions:
  - "strip_ctx_name uses starts_with prefix matching to match SWC behavior -- 'server' matches 'serverStuff$', 'serverAuth$', etc."
  - "Noop segments always get is_inline: false regardless of entry strategy -- SWC produces separate module files for all stripped handlers"
  - "Non-function first args in marker function calls produce segments using universal GetSpan for expression code extraction"
  - "C05 diagnostic added for locally-defined marker functions without Qrl counterpart export"

patterns-established:
  - "Universal arg span extraction: use GetSpan trait instead of matching specific Argument variants"
  - "collect_descendent_idents handles identifiers, template literals, objects, call expressions in addition to functions"

requirements-completed: [SEG-01, SEG-02, SEG-03, SEG-04, SEG-05]

duration: 32min
completed: 2026-04-04
---

# Phase 10 Plan 02: Remaining Segment Count Mismatches Summary

**Segment count parity improved from 180/201 to 195/201 by fixing strip_ctx_name prefix matching, noop segment module generation, non-function arg extraction, and C05 diagnostic for missing Qrl counterparts**

## Performance

- **Duration:** 32 min
- **Started:** 2026-04-04T02:36:26Z
- **Completed:** 2026-04-04T03:08:31Z
- **Tasks:** 1
- **Files modified:** 1 source file + 1 test file + 20 snapshot files

## Accomplishments
- Segment count parity improved from 180/201 (90%) to 195/201 (97%) -- 15 fixtures fixed
- Fixed strip_ctx_name to use starts_with prefix matching, matching SWC's behavior
- Fixed noop segments to always produce separate module files (is_inline: false)
- Added non-function first arg handling for marker functions (useStyles$, qwikify$, serverAuth$, etc.)
- Added C05 diagnostic for locally-defined marker functions missing Qrl counterpart export
- All 511 tests pass (264 unit + 223 snapshot + 24 spec)

## Task Commits

1. **Task 1: Diagnose and fix remaining segment count mismatches** - `c8cd7d2` (feat)

## Files Created/Modified
- `crates/qwik-optimizer-oxc/src/transform.rs` - strip_ctx_name fix, noop is_inline fix, non-function arg handling, C05 diagnostic, expanded collect_descendent_idents, universal expr_code extraction
- `crates/qwik-optimizer-oxc/tests/spec_examples.rs` - Updated spec_example_23 to expect C03 for non-function QRL scope in lib mode
- 20 snapshot files updated reflecting new segment extraction behavior

## Decisions Made
- Used starts_with prefix matching for strip_ctx_name to match SWC: the config value "server" should strip "serverStuff$", "serverAuth$", etc.
- Noop segments always produce separate module files (is_inline: false) even in Inline/Hoist strategies, matching SWC behavior
- Non-function first args are handled by expanding collect_descendent_idents to handle Identifier, TemplateLiteral, ObjectExpression, and CallExpression argument types
- C05 diagnostic fires early in enter_call_expression to prevent segment creation for locally-defined markers without Qrl counterparts

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] strip_ctx_name used exact equality instead of starts_with**
- **Found during:** Task 1 (diagnosing example_noop_dev_mode)
- **Issue:** should_emit_segment checked `s == ctx_name` but SWC uses `ctx_name.starts_with(s)`
- **Fix:** Changed to `ctx_name.starts_with(s.as_str())`
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** c8cd7d2

**2. [Rule 1 - Bug] Noop segments had is_inline: true preventing module generation**
- **Found during:** Task 1 (diagnosing example_noop_dev_mode onClick$ missing)
- **Issue:** Both JSX dollar-attr and exit_expression noop paths set is_inline: true, causing lib.rs to skip module generation for stripped handlers
- **Fix:** Changed to is_inline: false for all noop segments
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** c8cd7d2

**3. [Rule 2 - Missing Critical] Non-function first arg segment extraction**
- **Found during:** Task 1 (diagnosing example_capture_imports, example_with_style)
- **Issue:** enter_call_expression gated on first_arg_is_function, preventing segment creation for useStyles$('string'), $(identifier), serverAuth$({object}), etc.
- **Fix:** Removed function-only gate, expanded collect_descendent_idents, universal GetSpan for expr_code extraction
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** c8cd7d2

**4. [Rule 2 - Missing Critical] C05 diagnostic for missing Qrl counterpart**
- **Found during:** Task 1 (diagnosing example_missing_custom_inlined_functions overproduction)
- **Issue:** Locally-defined marker functions (e.g., exported useMemo$) created segments even when the Qrl counterpart (useMemoQrl) didn't exist as an export
- **Fix:** Added C05 diagnostic check in enter_call_expression for non-imported markers
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** c8cd7d2

---

**Total deviations:** 4 auto-fixed (2 bugs, 2 missing critical)
**Impact on plan:** All fixes were the core work items of the plan. No scope creep.

## Issues Encountered
- Reordering classify_captures before C03 check caused regression in fun_with_scopes and other tests -- reverted to original order. The C03 diagnostic correctly fires for non-function args referencing module-level identifiers (matching SWC behavior).
- spec_example_23 test assertion was too strict -- `useStyle$(STYLES)` in lib mode correctly produces C03 in both SWC and OXC. Updated test to allow C03.

## Remaining Segment Count Mismatches (6/201)

| Fixture | Expected | Actual | Root Cause |
|---------|----------|--------|------------|
| example_3 | 2 | 0 | OXC parse error -- input has syntax SWC tolerates but OXC rejects |
| example_immutable_analysis | 5 | 0 | OXC parse error -- same parser tolerance difference |
| example_jsx_import_source | 1 | 3 | Overproduction -- @jsxImportSource react pragma not detected, dollar-attrs extracted from React JSX |
| example_qwik_react | 2 | 0 | Pre-existing inlinedQrl() calls need handle_inlined_qsegment processing |
| relative_paths | 3 | 1 | Pre-existing inlinedQrl() calls need handle_inlined_qsegment processing |
| should_preserve_non_ident_explicit_captures | 1 | 0 | Pre-existing inlinedQrl() calls need handle_inlined_qsegment processing |

**Categories:**
- 2 parse errors (unfixable without OXC parser changes)
- 1 JSX import source detection (requires @jsxImportSource pragma handling)
- 3 pre-existing inlinedQrl processing (requires handle_inlined_qsegment implementation)

## Known Stubs
None -- all code paths produce real segment extraction and module generation.

## Next Phase Readiness
- Segment extraction is functionally complete for all standard patterns
- 195/201 segment count parity achieved (97%)
- Phase 10 complete -- ready for Phase 11 (Root Module code generation)
- 3 inlinedQrl fixtures may be addressed in Phase 13 (final acceptance)

---
*Phase: 10-segment-extraction*
*Completed: 2026-04-04*
