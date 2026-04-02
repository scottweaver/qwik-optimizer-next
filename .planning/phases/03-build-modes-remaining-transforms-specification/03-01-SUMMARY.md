---
phase: 03-build-modes-remaining-transforms-specification
plan: 01
subsystem: specification
tags: [const-replacement, dce, code-stripping, build-environment, CONV-10, CONV-09, CONV-11]
dependency_graph:
  requires: [01-01-PLAN, 01-05-PLAN]
  provides: [Stage 5 Build Environment Transforms spec section]
  affects: [03-02-PLAN, 03-03-PLAN, 03-04-PLAN]
tech_stack:
  added: []
  patterns: [stage-grouped-transforms, conditions-table, multi-mechanism-documentation]
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - "Organized CONV-10, CONV-09, CONV-11 under Stage 5 Build Environment Transforms umbrella per D-27 research recommendation"
  - "Used conditions table format for DCE mechanisms to clearly show when each runs"
  - "Documented treeshaker two-phase span-based approach with SideEffectVisitor mutual exclusion for Inline/Hoist"
metrics:
  duration: "3m"
  completed: "2026-04-01T22:55:16Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 1
  lines_added: 405
---

# Phase 03 Plan 01: Build Environment Transforms (CONV-10, CONV-09, CONV-11) Summary

Stage 5 Build Environment Transforms spec section covering const replacement with 8 rules (dual import sources, aliased imports, Lib/Test skip), three-mechanism DCE pipeline (simplifier, treeshaker mark/clean, post-migration), and three code stripping mechanisms (strip_exports throwing stubs, strip_ctx_name noop QRLs, strip_event_handlers) with conditions tables and inline examples from Jack's snapshots.

## What Was Done

### Task 1: Read SWC source files and snapshots for build environment transforms
- Read all 7 SWC source files: const_replace.rs, clean_side_effects.rs, filter_exports.rs, add_side_effect.rs, parse.rs (pipeline steps 2-16)
- Read all 7 Jack's snapshots: example_build_server, example_dead_code, example_drop_side_effects, example_strip_server_code, example_strip_client_code, example_strip_exports_used, example_strip_exports_unused
- Cross-referenced 03-RESEARCH.md findings against actual source code -- no discrepancies found
- Identified insertion point at line 3817 (after Source Map Generation section closing `---`)
- **Commit:** Combined with Task 2 (no file changes in Task 1)

### Task 2: Write Stage 5 Build Environment Transforms spec section
- Appended 405 lines to specification/qwik-optimizer-spec.md (3817 -> 4222 lines)
- **CONV-10 (Const Replacement):** 8 numbered rules covering isServer/isBrowser/isDev replacement, aliased imports, dual import sources (@qwik.dev/core/build and @qwik.dev/core), Lib mode skip, Test mode skip, default is_server=true, visitor recursion behavior
- **CONV-09 (Dead Branch Elimination):** 3 DCE mechanisms documented with conditions table -- SWC Simplifier (primary, Step 12), Treeshaker two-phase mark/clean (Steps 11/13b/13c), Post-migration DCE (Step 16). Explained mutual exclusion between Treeshaker clean and SideEffectVisitor for Inline/Hoist strategies
- **CONV-11 (Code Stripping):** 3 stripping mechanisms -- strip_exports (Step 2 pre-pass with throwing stub generation), strip_ctx_name (Step 10 segment suppression with noop QRL replacement), strip_event_handlers (Step 10 boolean flag for SSR builds)
- Each subsection includes: SWC Source reference (D-06), Pipeline Position, numbered Rules (D-04), inline example with input + all output modules (D-13, D-24), descriptive names with snapshot name in parentheses (D-16), and "See also" snapshot lists (D-24)
- **Commit:** 86f6116

## Deviations from Plan

None -- plan executed exactly as written.

## Verification Results

- `grep -c "## Stage 5" specification/qwik-optimizer-spec.md` returns 1
- `grep -c "### Const Replacement\|### Dead Branch Elimination\|### Code Stripping" specification/qwik-optimizer-spec.md` returns 3
- Each subsection contains "SWC Source:", "Pipeline Position:", numbered Rules, and at least one Example with input/output code blocks
- Spec line count increased by 405 lines (3817 -> 4222), within the 300-500 target range
- SPEC-10, SPEC-09, SPEC-11 requirements addressed

## Known Stubs

None -- all content is complete specification text with no placeholders or TODOs.

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1-2 | 86f6116 | feat(03-01): add Stage 5 Build Environment Transforms spec section |
