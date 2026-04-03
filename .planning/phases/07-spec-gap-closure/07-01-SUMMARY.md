---
phase: 07-spec-gap-closure
plan: "01"
subsystem: specification
tags: [spec, conv, gap-closure, documentation]
dependency_graph:
  requires: []
  provides: [SPEC-01, SPEC-02, SPEC-09, SPEC-10, SPEC-11]
  affects: [specification/qwik-optimizer-spec.md]
tech_stack:
  added: []
  patterns: []
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - "Phase Coverage paragraph updated: Dollar Detection (CONV-01) and QRL Wrapping (CONV-02) correctly attributed to Phase 7 gap closure, not Phase 1"
  - "Cross-reference at spec line 1493 confirmed valid: resolves to CONV-02 section at line 619"
  - "CONV-10 example_build_server input block confirmed truncated and repaired with closing tags, code fence, and Key Observations"
metrics:
  duration: 2m
  completed_date: "2026-04-03"
  tasks_completed: 2
  files_modified: 1
---

# Phase 07 Plan 01: CONV Section Verification and CONV-10 Fix Summary

Repaired CONV-10 example_build_server truncated input block and verified completeness of all 5 CONV sections (01, 02, 09, 10, 11) against their requirement criteria.

## What Was Fixed

### CONV-10 Truncated Example (Task 1)

The `example_build_server` input code block in the spec was truncated — the JSX closing `</Cmp>` was followed immediately by the `## Stage 6` header with no closing `);`, `});`, code fence, or Key Observations section.

**Fix applied at spec lines 3885-3899:**

- Added `);` (JSX expression closing)
- Added `});` (component$ callback closing)
- Added closing code fence ` ``` `
- Added **Key Observations** section documenting:
  - `isServer`/`isServer2` both become `true` on server builds
  - `isBrowser`/`isb` becomes `false`, dead `if (isb)` block removed
  - `threejs` and `leaflet` imports removed (dead branch refs), `mongodb` preserved
  - `isDev` passes through as captured import
- Added **See also** cross-reference to `example_dead_code` and Appendix B Example 12

**Commit:** `7a4df32`

### Phase Coverage Paragraph Update (Task 2, Step A)

The Phase Coverage paragraph (spec line 89) incorrectly claimed that "Dollar Detection" and "QRL Wrapping" were specified in Phase 1. These were actually added in later phases. Updated to:

- Removed "Dollar Detection" and "QRL Wrapping" from Phase 1 list
- Added to Later phases: "Stage 4 Core Transform gap closure (Phase 7 -- Dollar Detection CONV-01, QRL Wrapping CONV-02)"

**Commit:** `c0dcd0d`

## What Was Verified

### CONV-01 (Dollar Detection) — spec lines 428-618

All D-03 required topics confirmed present:
- Rule 1: Imported marker detection from `@qwik.dev/core`
- Rule 2: Local marker detection (exported `$`-suffixed functions)
- Rule 3: `convert_qrl_word` callee conversion table
- Rule 4: Special cases (sync$, component$, bare $)
- Rule 5: Detection site (fold_call_expr)
- Rule 6: Non-marker exclusion rule
- 3 input/output examples (example_6, example_capture_imports, non_marker_edge_case)
- SWC source references (transform.rs:189-202, 179-187, words.rs)

**Status: Complete. Satisfies SPEC-01 / D-03.**

### CONV-02 (QRL Wrapping) — spec lines 619-807

All D-04 required topics confirmed present:
- Rule 1: Three QRL creation paths (`create_qrl`, `create_inline_qrl`, `create_noop_qrl`)
- Rule 2: Dev mode variants (`qrlDEV`, `inlinedQrlDEV`, `_noopQrlDEV`)
- Rule 3: Captures emission (`scoped_idents` / `emit_captures`)
- Rule 4: PURE annotation rule (`create_internal_call` with `pure: true`)
- Rules 5-6: Symbol name and import path construction
- 2 input/output examples

**Status: Complete. Satisfies SPEC-02 / D-04.**

### CONV-09 (Dead Branch Elimination) — spec lines 3979-4067

Confirmed by research: 3 DCE mechanisms (SWC Simplifier, Treeshaker, Post-migration DCE) with conditions table, source references.

**Status: Complete. Satisfies SPEC-09.**

### CONV-10 (Const Replacement) — spec lines 3825-3977

Confirmed by research: 8 behavioral rules covering all isServer/isBrowser/isDev replacement behaviors. Example `example_build_server` now complete after Task 1 repair.

**Status: Complete after fix. Satisfies SPEC-10.**

### CONV-11 (Code Stripping) — spec lines 4069-4154+

Three mechanisms verified substantive:
- Mechanism 1: `strip_exports` (throwing stub replacement)
- Mechanism 2: `strip_ctx_name` (segment suppression by prefix)
- Mechanism 3: `strip_event_handlers` (3 rules: SegmentKind check, noop treatment, SSR use case)

**Status: Complete. Satisfies SPEC-11.**

### Cross-Reference Verification (Task 2, Step B)

The reference at spec line 1493 ("See QRL Wrapping section for `create_qrl` details") was examined. It resolves correctly to the CONV-02 "QRL Wrapping" section at line 619. No change needed.

## Deviations from Plan

None — plan executed exactly as written. All content was present as confirmed by research; only the CONV-10 truncation fix and Phase Coverage update were needed.

## Self-Check: PASSED

- [x] `specification/qwik-optimizer-spec.md` exists and modified
- [x] Commit `7a4df32` exists (Task 1: CONV-10 fix)
- [x] Commit `c0dcd0d` exists (Task 2: Phase Coverage + verification)
- [x] `grep -n "Key Observations" spec.md | grep "^3890:"` confirmed
- [x] `grep -n "^## Stage 6" spec.md` shows line 3900 (after closing fence at 3888)
- [x] Line count 8105 > 8091 baseline
