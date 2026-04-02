---
phase: 01-core-pipeline-specification
plan: 03
subsystem: specification-capture-analysis
tags: [spec, capture-analysis, self-import, edge-cases, taxonomy]
dependency_graph:
  requires: [01-01]
  provides: [capture-analysis-spec, capture-taxonomy, capture-edge-cases, self-import-reclassification-spec]
  affects: [01-04, 01-05]
tech_stack:
  added: []
  patterns: [8-category-capture-taxonomy, self-import-reclassification, IdentCollector-decl_stack-algorithm]
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - Capture analysis section placed under Stage 4 Core Transform with full algorithm, taxonomy, and 16 edge cases
  - Self-import reclassification documented as a first-class subsection (not buried in capture analysis)
  - All 16 CAPTURE-EDGE cases use consistent format with ID, category, rule, input, expected behavior, rationale
metrics:
  duration: 5m
  completed: "2026-04-01T18:44:45Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 1
---

# Phase 1 Plan 3: Capture Analysis Specification Summary

Complete capture analysis section with 8-category taxonomy, Mermaid decision tree, self-import reclassification subsection, 3 input/output examples from Jack's snapshots, and 16 named edge cases (CAPTURE-EDGE-01 through CAPTURE-EDGE-16) providing a full test matrix for capture analysis implementation.

## What Was Done

### Task 1: Capture Analysis Algorithm and 8-Category Taxonomy
- Added `### Capture Analysis (CONV-03)` section under Stage 4: Core Transform
- Documented the 4-step algorithm: IdentCollector collection, decl_stack partition, compute_scoped_idents intersection, GlobalCollect classification
- Created Mermaid flowchart decision tree (D-09) showing classification logic for any variable reference inside `$()` body
- Built 8-category taxonomy table with columns: Category, Is Capture?, How Resolved in Segment, SWC Mechanism, Example
- Wrote Self-Import Reclassification subsection documenting the _auto_ export pattern that fixed 46 of Jack's 293 runtime deviations
- Added 3 input/output examples from Jack's snapshots: example_multi_capture (captures with destructuring), example_capture_imports (import re-emission), example_capturing_fn_class (self-import and Category 8 errors)
- Included parse error tolerance behavioral note (Pitfall 4/6 from research)
- Commit: bbcec1f

### Task 2: 16 Named Capture Edge Cases
- Added `#### Named Capture Edge Cases` subsection within the Capture Analysis section
- Documented all 16 edge cases (CAPTURE-EDGE-01 through CAPTURE-EDGE-16) with consistent format: ID, name, category, rule tested, input code snippet, expected behavior, rationale
- Edge cases cover: loop variables (for-of, for-in, C-style for), nested scope capture, variable shadowing, callback parameters (destructured + rest), function/class ERROR diagnostics, module-level self-imports (const, function, enum), import re-emission (named, default), props destructuring ordering, TypeScript type-only erasure
- Referenced 5 Jack snapshot names per D-16: example_component_with_event_listeners_inside_loop, example_capturing_fn_class, example_capture_imports, example_multi_capture, example_ts_enums
- Added summary note about 293 runtime deviations and 46 self-import fixes
- Commit: b29037a

## Deviations from Plan

None -- plan executed exactly as written.

## Decisions Made

1. **Capture Analysis placement**: Added after Stage 4 header with a note that Dollar Detection, QRL Wrapping, Segment Extraction, and Import Rewriting are added by Plans 02 and 04 (since they run in parallel)
2. **Self-import as first-class subsection**: Gave self-import reclassification its own `####` subsection rather than embedding it in the algorithm overview, emphasizing its importance (46/293 deviations)
3. **Edge case format**: Used consistent structure (ID, name, category, rule tested, input, expected behavior, why it matters) for all 16 edge cases to serve as a spec test matrix

## Known Stubs

None -- all content is complete specification text.

## Self-Check: PASSED

- specification/qwik-optimizer-spec.md: FOUND
- 01-03-SUMMARY.md: FOUND
- Commit bbcec1f (Task 1): FOUND
- Commit b29037a (Task 2): FOUND
