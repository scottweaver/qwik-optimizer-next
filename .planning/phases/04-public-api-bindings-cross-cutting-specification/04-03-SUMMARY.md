---
phase: 04-public-api-bindings-cross-cutting-specification
plan: 03
subsystem: specification
tags: [appendix, examples, representative-examples, conv-coverage, snapshots]
dependency_graph:
  requires: [04-02]
  provides: [SPEC-29]
  affects: [specification/qwik-optimizer-spec.md]
tech_stack:
  patterns: [curated-snapshot-examples, conv-coverage-table, input-output-verification-pairs]
key_files:
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - "D-30 fulfilled: 24 curated examples in dedicated Appendix B, complementing inline CONV examples"
metrics:
  duration: "~8m"
  completed: "2026-04-02"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 1
---

# Phase 04 Plan 03: Representative Examples Appendix Summary

24 curated input/output examples in Appendix B covering all 14 CONVs, extracted from Jack's 162+ SWC reference snapshots with input source, transform config, expected output, and key observations per example.

## Tasks Completed

### Task 1: Extract and write examples 1-12 (core CONVs)
- **Commit:** 973d11d
- Appended "## Appendix B: Representative Examples" after Appendix A
- Wrote examples 1-12 covering CONVs 01-12: dollar detection, QRL wrapping, capture analysis, props destructuring, segment extraction, JSX transforms, signal optimization, PURE annotations, dead branch elimination, const replacement
- Each example includes input source, expected output (root + segments), config, and key observations with CONV cross-references

### Task 2: Write examples 13-24 and CONV coverage table
- **Commit:** 1593d6d
- Wrote examples 13-24: code stripping (strip_exports + strip_ctx_name), import rewriting, sync$ serialization, noop QRL handling, inline entry strategy, dev mode, prod mode, bind: sugar, loop captures, lib mode, preserve_filenames
- Added CONV Coverage Summary table verifying all 14 CONVs have at least one example
- Closing paragraph references the full 162+ snapshot corpus

## Verification Results

1. `grep -c "^### Example"` in Appendix B returns 24
2. "## Appendix B: Representative Examples" section exists
3. "CONV Coverage Summary" table exists
4. All 14 CONVs (CONV-01 through CONV-14) appear in coverage table
5. Section ordering: Public API Types -> Binding Contracts -> Appendix A -> Appendix B

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] Cherry-picked Phase 04-01 and 04-02 content into worktree**
- **Found during:** Pre-Task 1 setup
- **Issue:** This worktree branched from an earlier commit that did not include Phase 04 plans 01-02 output (Appendix A, Public API Types, Binding Contracts sections). Plan depends_on 04-02.
- **Fix:** Copied the spec file from commit 09d528b (04-02 completion) to provide the correct base with Appendix A already present
- **Files modified:** specification/qwik-optimizer-spec.md

## Key Decisions

- D-30 fulfilled: Examples are in a dedicated appendix, not inlined into CONV sections
- Examples are concise: long snapshots are truncated with key portions highlighted
- Each example cross-references the relevant CONV section for deeper details
- Coverage table at the end provides an at-a-glance verification of completeness

## Self-Check: PASSED
