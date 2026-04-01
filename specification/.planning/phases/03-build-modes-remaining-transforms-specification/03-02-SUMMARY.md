---
phase: 03-build-modes-remaining-transforms-specification
plan: 02
subsystem: specification
tags: [pure-annotations, sync-qrl, noop-qrl, qrl-special-cases]
dependency_graph:
  requires: [01-01, 01-02]
  provides: [CONV-08-spec, CONV-13-spec, CONV-14-spec]
  affects: [qwik-optimizer-spec.md]
tech_stack:
  added: []
  patterns: [PURE-annotation-placement, sync-serialization, noop-qrl-forms]
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - "PURE anti-list explicitly names all side-effectful wrappers (useTaskQrl, useStyleQrl, useBrowserVisibleTaskQrl, serverStuffQrl, etc.) per D-27 and Pitfall 2"
  - "sync$ serialization documented with all three function forms (function expression, arrow block, arrow concise) per D-13"
  - "Noop QRL documented with both stripped-segment and Hoist-strategy contexts per D-25"
metrics:
  duration: 3m
  completed: 2026-04-01
---

# Phase 3 Plan 02: QRL Special Cases (CONV-08, CONV-13, CONV-14) Summary

Stage 6 QRL Special Cases section appended to spec covering PURE annotation placement rules with explicit anti-list, sync$ minified serialization with three function forms, and noop QRL mode-dependent forms with captures preservation and Hoist .s() registration pattern.

## What Was Done

### Task 1: Read SWC source files and snapshots
Read transform.rs (create_noop_qrl, add_pure_comment, handle_sync_qrl, hoist_qrl_to_module_scope), inlined_fn.rs (render_expr), and 5 Jack's snapshots (impure_template_fns, example_of_synchronous_qrl, example_noop_dev_mode, example_strip_exports_unused, example_inlined_entry_strategy). Extracted PURE placement sites, sync$ serialization rules, and noop QRL behavioral forms. Cross-referenced all findings against 03-RESEARCH.md.

### Task 2: Write Stage 6 QRL Special Cases spec section
Appended ~450 lines to qwik-optimizer-spec.md (3817 -> 4267 lines) as "## Stage 6: QRL Special Cases" with three subsections:

**CONV-08 (PURE Annotations):** Placement sites table (componentQrl, qrl, inlinedQrl, _noopQrl, qSegment), explicit anti-list of side-effectful wrappers, mechanism description, mode-independence note. Two examples from impure_template_fns and example_strip_exports_unused snapshots.

**CONV-13 (sync$ Serialization):** Detection, argument validation, minified serialization pipeline (6 steps), _qrlSync output format, no-segment-extraction rule, import rewriting. Example showing all three function forms from example_of_synchronous_qrl snapshot.

**CONV-14 (Noop QRL Handling):** Two generation contexts (stripped segments, Hoist strategy), mode-dependent forms table (Prod vs Dev/Hmr), PURE annotation cross-reference, captures preservation with .w() chain, noop segment output format, Hoist .s() registration pattern, Lib mode exception. Two examples from example_noop_dev_mode and example_inlined_entry_strategy snapshots.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1-2 | 025d5d8 | feat(03-02): add Stage 6 QRL Special Cases spec section |

## Deviations from Plan

None - plan executed exactly as written.

## Requirements Coverage

| Requirement | Status | Evidence |
|-------------|--------|----------|
| SPEC-08 | Covered | PURE annotation placement sites, anti-list of side-effectful wrappers, componentQrl-only user-visible PURE |
| SPEC-13 | Covered | _qrlSync output format, minified serialization, no segment extraction, three function form examples |
| SPEC-14 | Covered | Mode-dependent forms (Prod vs Dev/Hmr), captures preservation, noop segment output, Hoist .s() pattern |

## Known Stubs

None - all sections are complete with rules and examples.

## Self-Check: PASSED

- FOUND: specification/qwik-optimizer-spec.md
- FOUND: 03-02-SUMMARY.md
- FOUND: commit 025d5d8
- FOUND: Stage 6 heading, CONV-08, CONV-13, CONV-14 subsections
- FOUND: PURE anti-list (useTaskQrl)
