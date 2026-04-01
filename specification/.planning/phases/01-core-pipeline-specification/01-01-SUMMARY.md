---
phase: 01-core-pipeline-specification
plan: 01
subsystem: specification-infrastructure
tags: [spec, pipeline, globalcollect, hash-generation, path-resolution]
dependency_graph:
  requires: []
  provides: [spec-document-structure, pipeline-overview, globalcollect-spec, hash-algorithm-spec, path-resolution-spec]
  affects: [01-02, 01-03, 01-04]
tech_stack:
  added: []
  patterns: [mermaid-diagrams, input-output-examples, swc-source-references]
key_files:
  created:
    - specification/qwik-optimizer-spec.md
  modified: []
decisions:
  - Spec document follows pipeline execution order (D-01) with 6 stage groupings
  - Mermaid flowchart shows full 20-step pipeline at document top (D-02)
  - SWC source references included for every section (D-06)
  - Examples follow basic + edge-case pattern (D-14)
metrics:
  duration: 4m
  completed: "2026-04-01T18:36:00Z"
---

# Phase 01 Plan 01: Spec Foundation and Infrastructure Summary

Spec document created with Pipeline Overview (20-step Mermaid diagram), GlobalCollect behavioral specification (5 rules, 6 methods, 2 examples), Hash Generation (two-level DefaultHasher with URL_SAFE_NO_PAD base64 encoding, display name construction with escape_sym and deduplication), and Path Resolution (parse_path, canonical filename, import path construction with explicit_extensions).

## What Was Built

### Pipeline Overview
- Mermaid flowchart diagram showing the 20-step transformation pipeline grouped into 6 stages
- Prose descriptions of each stage (2-3 sentences each)
- Phase coverage notes indicating which stages this phase vs later phases document

### GlobalCollect (SPEC-21)
- Data structure table: imports (IndexMap<Id, Import>), exports (IndexMap<Atom, ExportInfo>), root (IndexMap<Id, Span>), canonical_ids (HashMap<Atom, Id>)
- 5 behavioral rules covering single-pass collection, import/export/root handling, and canonical ID registration
- 6 key methods documented: is_global, import, add_export, remove_root_and_exports_for_id, get_imported_local, export_local_ids
- Example 1 (basic_collect): Module with imports, exports, root declarations showing full GlobalCollect output
- Example 2 (synthetic_import): Synthetic import addition during transform showing before/after state

### Hash Generation (SPEC-23)
- Two-level hashing: file hash (scope + rel_path) and symbol hash (scope + rel_path + display_name)
- Exact base64 algorithm: u64.to_le_bytes() -> URL_SAFE_NO_PAD -> replace '-'/'_' with '0'
- Display name construction: stack_ctxt join, escape_sym, digit prefix, segment_names deduplication
- Symbol name format: Dev mode ({display_name}_{hash64}) vs Prod mode (s_{hash64})
- Example 1 (example_6): Basic $() call showing full hash computation pipeline
- Example 2 (dedup_dollar): Two $() calls in same function showing deduplication suffix

### Path Resolution (SPEC-24)
- parse_path() behavior: file_stem, extension, file_name, rel_dir extraction
- Output extension mapping table (6 combinations of transpile flags)
- get_canonical_filename(): display_name + last token of symbol_name
- Import path: "./" + canonical_filename (+ extension if explicit_extensions)
- Example 1 (basic_path): Full path resolution for src/components/Counter.tsx
- Example 2 (explicit_ext): explicit_extensions true vs false comparison

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1 | 176a16f | Pipeline Overview + GlobalCollect sections |
| 2 | 3561dfd | Hash Generation + Path Resolution sections |

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None -- all sections are fully specified with behavioral rules and examples.

## Self-Check: PASSED

- specification/qwik-optimizer-spec.md: FOUND
- 01-01-SUMMARY.md: FOUND
- Commit 176a16f: FOUND
- Commit 3561dfd: FOUND
