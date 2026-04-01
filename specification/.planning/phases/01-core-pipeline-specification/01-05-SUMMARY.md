---
phase: 01-core-pipeline-specification
plan: 05
subsystem: specification
tags: [variable-migration, source-maps, post-transform, infrastructure]
dependency_graph:
  requires:
    - 01-01 (GlobalCollect section for root-level binding tracking)
    - 01-03 (Capture Analysis for scoped_idents/local_idents)
  provides:
    - Variable Migration behavioral spec (SPEC-22)
    - Source Map Generation behavioral spec (SPEC-25)
  affects:
    - specification/qwik-optimizer-spec.md
tech_stack:
  added: []
  patterns:
    - 5-step migration pipeline with fixed-point safety filter
    - Dual-fidelity source map contract (root=high, segment=mixed)
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - Variable Migration placed as top-level ## section (per D-12) between Stage 4 content and Infrastructure sections
  - Source Map Generation placed as ## Infrastructure section (consistent with Hash Generation and Path Resolution patterns)
  - Used ## Infrastructure: Source Map Generation header (matching existing convention) rather than ### as plan suggested
metrics:
  duration: 4m
  completed: "2026-04-01T18:52:00Z"
---

# Phase 1 Plan 5: Variable Migration and Source Map Generation Summary

5-step variable migration pipeline with 6 migration conditions (including transitive dependency and safety filter) plus source map generation contracts documenting root module (high fidelity) vs segment module (mixed fidelity) output.

## What Was Done

### Task 1: Variable Migration Section (SPEC-22)
- **Commit:** f4677b4
- Added `## Variable Migration` section with complete 5-step pipeline specification
- Documented `analyze_root_dependencies`, `build_root_var_usage_map`, `build_main_module_usage_set`, `find_migratable_vars`, and post-migration cleanup
- Specified all 6 migration conditions: single-segment usage, not user-exported, not imported, not main-module referenced, transitive dependencies migratable, safety filter against breaking root declarations
- Documented shared declarator unit constraint (destructuring patterns must migrate together)
- Added cross-references to Capture Analysis (scoped_idents/local_idents), Segment Extraction (migrated_root_vars), GlobalCollect, and Import Rewriting
- Added Example 1: Basic Variable Migration -- helper function migrated into segment
- Added Example 2: Non-Migratable Shared Dependency -- CONFIG used by 2 segments blocks migration, derived blocked by transitive dependency on CONFIG
- Updated Phase Coverage text to reflect completed Plan 05 sections

### Task 2: Source Map Generation Section (SPEC-25)
- **Commit:** d1290d3
- Added `## Infrastructure: Source Map Generation` section
- Documented emit_source_code 5-step generation mechanism (JsWriter, Emitter, V3 format, sourceRoot, return tuple)
- Specified root module source maps: high fidelity, spans preserved through in-place AST mutation
- Specified segment module source maps: mixed fidelity -- function body maps accurately (original spans), generated imports/captures/exports have no mapping
- Documented configuration options (source_maps bool, root_dir/sourceRoot)
- Documented TransformModule output contract with map: Option<String> field
- Added Example 1: Root Module Source Map -- JSON structure with sources, sourcesContent, mappings, sourceRoot
- Added Example 2: Segment Module Source Map -- mapping coverage breakdown (function body accurate, generated code unmapped)

## Deviations from Plan

### Minor Adjustments

**1. Source Map section header convention**
- **Plan said:** Use `### Source Map Generation` under Infrastructure
- **Actual:** Used `## Infrastructure: Source Map Generation` to match existing convention (`## Infrastructure: Hash Generation`, `## Infrastructure: Path Resolution`)
- **Rationale:** Consistency with document structure. All infrastructure sections are top-level `##` sections.

No other deviations. Plan executed as written.

## Decisions Made

1. Variable Migration placed as a separate `##` section between the Stage 4 content (Capture Analysis) and Infrastructure sections, per D-12
2. Source Map Generation uses `## Infrastructure:` prefix matching Hash Generation and Path Resolution sections
3. Phase Coverage text updated to reflect all Phase 1 sections are now specified

## Known Stubs

None. All sections are fully specified with behavioral rules and examples.

## Verification Results

- Variable Migration is a separate top-level `##` section at line 920 (per D-12)
- Source Map Generation is at line 1476 under Infrastructure
- Variable Migration cross-references Capture Analysis and Segment Extraction
- All function names documented: analyze_root_dependencies, build_root_var_usage_map, build_main_module_usage_set, find_migratable_vars
- All 6 migration conditions specified including safety filter
- 4 code block examples total (2 per section)
- Source references to dependency_analysis.rs and parse.rs included
