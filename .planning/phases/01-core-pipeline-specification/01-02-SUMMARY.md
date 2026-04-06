---
phase: 01-core-pipeline-specification
plan: 02
subsystem: specification-core-transform
tags: [spec, dollar-detection, qrl-wrapping, conv-01, conv-02, marker-functions]
dependency_graph:
  requires: [spec-document-structure, pipeline-overview, hash-algorithm-spec, path-resolution-spec]
  provides: [dollar-detection-spec, qrl-wrapping-spec, marker-functions-rules, callee-conversion-rules, qrl-creation-paths]
  affects: [01-03, 01-04]
tech_stack:
  added: []
  patterns: [behavioral-rules-with-source-refs, input-output-examples, swc-code-snippets]
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - Dollar detection documented as 6 behavioral rules covering imported markers, local markers, callee conversion, special cases, detection site, and non-markers
  - QRL wrapping documented as 6 behavioral rules covering three creation paths, dev mode variants, captures emission, PURE annotation, symbol name, and import path
  - Examples drawn from Jack's SWC snapshots (example_6, example_capture_imports, example_multi_capture, example_noop_dev_mode) with constructed non-marker edge case
metrics:
  duration: 3m
  completed: "2026-04-01T18:43:00Z"
---

# Phase 01 Plan 02: Dollar Detection and QRL Wrapping Summary

Dollar Detection (CONV-01) and QRL Wrapping (CONV-02) sections added to the spec document under Stage 4: Core Transform, documenting the marker_functions HashMap construction from @qwik.dev/core imports and local exports, convert_qrl_word callee conversion, three QRL creation paths (create_qrl for segment strategy, create_inline_qrl for inline/hoist, create_noop_qrl for stripped callbacks), dev mode variants (qrlDEV/inlinedQrlDEV/_noopQrlDEV with source location metadata), captures emission with .w() call mechanism, and PURE annotation rules.

## Changes Made

### Task 1: Dollar Detection (CONV-01)
- **Commit:** 955ac58
- Added "### Dollar Detection (CONV-01)" section under Stage 4: Core Transform
- 6 behavioral rules: imported markers, local markers, callee conversion (convert_qrl_word), special cases (sync$, bare $, component$), detection site (fold_call_expr), non-markers
- 3 examples: Basic Dollar Extraction (example_6), Multiple Markers from Same Import (example_capture_imports), Non-Marker Dollar Function (constructed edge case)
- SWC source references: transform.rs:189-202, transform.rs:179-187, words.rs

### Task 2: QRL Wrapping (CONV-02)
- **Commit:** 67ffefd
- Added "### QRL Wrapping (CONV-02)" section immediately after Dollar Detection
- 6 behavioral rules: three QRL creation paths, dev mode variants, captures emission (.w() mechanism), PURE annotation (componentQrl only for wrapper calls), symbol name format, import path construction
- 3 examples: Basic QRL Wrapping (example_6), QRL with Captures (example_multi_capture), Dev Mode and Noop QRL (example_noop_dev_mode)
- SWC source references: transform.rs:1888-2062, transform.rs:3000-3027, transform.rs:2013-2029, transform.rs:1372-1457

## Cross-References

- Dollar Detection references GlobalCollect (marker_functions built from global_collect.imports and export_local_ids)
- QRL Wrapping references Hash Generation (symbol names) and Path Resolution (import paths)
- QRL Wrapping links forward to Capture Analysis (scoped_idents determining captures array)
- Both sections include must_haves key_links: Dollar Detection -> QRL Wrapping (markers to replacement), QRL Wrapping -> Hash Generation (symbol names use hashes)

## Deviations from Plan

None -- plan executed exactly as written.

## Self-Check: PASSED
