---
phase: 03-build-modes-remaining-transforms-specification
plan: 04
subsystem: specification
tags: [pipeline-dag, ordering-constraints, dependency-graph, SPEC-17, mermaid, transformation-pipeline]
dependency_graph:
  requires: [03-01-PLAN, 03-02-PLAN, 03-03-PLAN]
  provides: [Transformation Pipeline spec section with Mermaid DAG and ordering constraints table]
  affects: []
tech_stack:
  added: []
  patterns: [mermaid-dag-diagram, constraints-table-with-rationale, conditional-execution-summary]
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - "Pipeline DAG uses subgraph groupings matching Stages 1-6 from Pipeline Overview for consistency per D-28"
  - "Steps 14a/14b split to show mutually exclusive treeshaker clean vs side effect preservation paths"
  - "19 ordering constraints documented with parse.rs line references for traceability per D-06"
metrics:
  duration: 3m
  completed: "2026-04-01T23:09:00Z"
  tasks: 2
  files: 1
---

# Phase 3 Plan 4: Transformation Pipeline DAG Summary

Full 20-step pipeline dependency DAG in Mermaid format with 19-row ordering constraints table documenting every dependency rationale and SWC source reference.

## What Was Done

### Task 1: Read parse.rs pipeline and existing Pipeline Overview for DAG construction
- Read parse.rs lines 220-450 to verify the 20-step transformation ordering
- Read existing Pipeline Overview Mermaid diagram (spec lines 15-70) for style matching
- Read FEATURES.md dependency DAG for constraint catalog
- Mapped all dependency edges with rationale from source code analysis

### Task 2: Write Transformation Pipeline spec section
- Appended ~149 lines to specification/qwik-optimizer-spec.md after the Emit Modes section
- **Pipeline Dependency DAG**: Mermaid flowchart with all 20+ steps as nodes, dependency arrows with labels, subgraph groupings for Stages 1-6, conditional step annotations
- **Ordering Constraints Table**: 19 rows documenting every ordering dependency with Before Step, After Step, Rationale, and parse.rs source reference
- **Conditional Execution Summary**: Table of 12 conditional steps with their guard conditions, skip conditions, and relevant config fields
- Documented mutually exclusive paths (side effect preservation vs treeshaker cleanup)
- Documented unconditional steps for implementer clarity

**Commit:** 1a96d7c

## Deviations from Plan

### Auto-adjusted Issues

**1. [Rule 2 - Enhancement] Extended step numbering for branching paths**
- **Found during:** Task 2
- **Issue:** The Pipeline Overview uses linear numbering 1-20, but the actual pipeline has mutually exclusive branches (step 13 vs step 14a) and conditional second/third DCE passes
- **Fix:** Used 14a/14b numbering and added a note explaining the relationship between linear Pipeline Overview numbering and actual branching DAG structure
- **Files modified:** specification/qwik-optimizer-spec.md
- **Commit:** 1a96d7c

## Verification Results

- "## Transformation Pipeline" heading: 1 occurrence (correct)
- Mermaid code blocks: 3 total (Pipeline Overview + new DAG = expected)
- Ordering constraints rows: 19 (exceeds minimum of 10)
- Pipeline steps in DAG: 21 references (20 steps with 14 split into 14a/14b)
- Const Replacement -> DCE dependency: explicitly documented with rationale

## Known Stubs

None -- all content is complete specification text with no placeholders.

## Self-Check: PASSED

- FOUND: specification/qwik-optimizer-spec.md
- FOUND: 03-04-SUMMARY.md
- FOUND: commit 1a96d7c

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Read parse.rs pipeline and existing Pipeline Overview | (read-only, no commit) | - |
| 2 | Write Transformation Pipeline spec section | 1a96d7c | specification/qwik-optimizer-spec.md |
