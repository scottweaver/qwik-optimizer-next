---
phase: 01-core-pipeline-specification
plan: 04
subsystem: specification-segment-extraction-import-rewriting
tags: [spec, segment-extraction, import-rewriting, self-import, captures, new_module]
dependency_graph:
  requires: [01-01, 01-02, 01-03]
  provides: [segment-extraction-spec, import-rewriting-spec, self-import-pattern-spec]
  affects: [01-05]
tech_stack:
  added: []
  patterns: [segment-module-construction, 4-mechanism-import-rewriting, self-import-_auto_-prefix]
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - Segment Extraction and Import Rewriting sections placed after Capture Analysis in Stage 4 Core Transform
  - Import Rewriting documented as 4 distinct mechanisms (legacy rename, consumed stripping, synthetic addition, per-segment resolution)
  - Self-import pattern documented with _auto_ prefix as cross-cutting concern between Segment Extraction and Import Rewriting
metrics:
  duration: 5m
  completed: "2026-04-01T18:54:00Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 1
---

# Phase 01 Plan 04: Segment Extraction and Import Rewriting Summary

Segment extraction section documents create_segment registration and new_module 8-step construction pipeline (captures injection, QRL hoisting, import resolution, topological sort, deduplication, named export) with nested segment parent-child relationships via segment_stack. Import rewriting section documents 4 mechanisms (legacy @builder.io rename, consumed import stripping via DCE, synthetic ensure_core_import addition, per-segment resolve_import_for_id with collision handling and _auto_ self-import pattern) -- with 6 total input/output examples from Jack's snapshots.

## What Was Done

### Task 1: Segment Extraction (CONV-05 / SPEC-05)
- Added `### Segment Extraction (CONV-05)` section after Capture Analysis in Stage 4 Core Transform
- Documented `create_segment` function: canonical_filename computation, entry classification, import_path construction, QRL creation, Segment struct push
- Full Segment struct field table (name, canonical_filename, expr, scoped_idents, local_idents, display_name, ctx_kind, ctx_name, parent_segment, etc.)
- Nested segments subsection: segment_stack tracking, parent assignment, component nesting, SegmentAnalysis.parent exposure
- `new_module` 8-step construction process: captures injection, QRL hoisting, self-referential var fix, extra top items, import resolution, topological sort, deduplication, named export
- Segment construction loop (parse.rs:446-583): NewModuleCtx creation, DCE, hygiene, codegen
- 3 examples: Basic Segment (example_6), Segment with Captures (example_multi_capture), Nested Segments (example_capture_imports)
- 2 SegmentAnalysis JSON metadata blocks per D-15
- Commit: 2633099

### Task 2: Import Rewriting (CONV-12 / SPEC-12)
- Added `### Import Rewriting (CONV-12)` section after Segment Extraction
- Mechanism 1: Legacy Rename -- RenameTransform rewrites @builder.io/qwik to @qwik.dev/core (and qwik-city/qwik-react variants), runs before GlobalCollect
- Mechanism 2: Consumed Import Stripping -- relies on DCE to remove unreferenced $-suffixed imports after QwikTransform
- Mechanism 3: Synthetic Import Addition -- ensure_core_import registers qrl/componentQrl/etc. in GlobalCollect with synthetic flag; ensure_export creates _auto_ exports for self-import pattern
- Mechanism 4: Per-Segment Import Resolution -- resolve_import_for_id with 4-step priority (exact match, unique-by-symbol fallback, export self-import, no match) plus collision handling with _N suffix renaming
- Common synthetic imports table (qrl, componentQrl, inlinedQrl, _jsxSorted, etc.)
- Dev-mode explicit imports (_qrlDev, _inlinedQrlDev, _noopQrlDev)
- 3 examples: Root Module Import Transformation (example_6), Segment Import Resolution (example_capture_imports), Self-Import Pattern (example_segment_variable_migration)
- Updated Stage 4 placeholder note to reflect completed sections
- Commit: 2fd1aaf

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 2633099 | Segment Extraction and Import Rewriting sections (both written together at same insertion point) |
| 2 | 2fd1aaf | Updated Stage 4 placeholder note |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Missing] Combined Task 1 and Task 2 into single edit**
- **Found during:** Task 1
- **Issue:** Both sections needed to be inserted at the exact same location (between Capture Analysis edge cases and Infrastructure: Hash Generation). Writing them separately would have required re-reading the file and finding a new insertion point.
- **Fix:** Wrote both sections in a single edit, committed together. Task 2 commit updates the Stage 4 placeholder note as a separate change.
- **Files modified:** specification/qwik-optimizer-spec.md

## Known Stubs

None -- all content is complete specification text with source references, examples, and cross-links.

## Self-Check: PASSED

- specification/qwik-optimizer-spec.md: FOUND
- 01-04-SUMMARY.md: FOUND
- Commit 2633099: FOUND
- Commit 2fd1aaf: FOUND
