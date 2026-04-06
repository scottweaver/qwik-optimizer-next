---
phase: 11-root-module-code-generation
plan: 01
subsystem: transform-engine
tags: [imports, dead-code-elimination, marker-source-tracking, synthetic-imports]
dependency_graph:
  requires: [phase-10-segment-extraction]
  provides: [marker-fn-source-tracking, synthetic-import-emission, expanded-dead-import-elimination]
  affects: [root-module-output, import-ordering]
tech_stack:
  added: []
  patterns: [marker-source-tracking, find-wrapper-source-lookup, synthetic-import-emission]
key_files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/transform.rs
    - crates/qwik-optimizer-oxc/tests/snapshots/ (43 snapshot files updated)
decisions:
  - "D-10: QRL wrapper imports use find_wrapper_source() helper to resolve original import source per marker function"
  - "D-11: Marker function stripping expanded to all import sources, not just core module"
  - "D-12: Dead import elimination expanded to all imports, not just core module"
  - "D-13: Synthetic imports from collect.synthetic emitted at top of module via insert-at-0 pattern"
metrics:
  duration: 8m
  completed: 2026-04-06
  tasks: 1
  files_modified: 44
---

# Phase 11 Plan 01: Import Assembly Rewrite Summary

Marker source tracking and SWC-compatible import assembly with expanded dead import elimination, improving root module parity from 28/201 to 36/201.

## Changes Made

### Task 1: Track marker function source modules and rewrite exit_program import assembly

**marker_fn_sources tracking** -- Added a `marker_fn_sources: HashMap<String, String>` field to `QwikTransform` that maps each marker function ctx_name (e.g., `"globalAction$"`) to its original import source module (e.g., `"@qwik.dev/router"`). Populated during `QwikTransform::new()` by scanning `collect.imports` alongside marker function detection.

**find_wrapper_source() helper** -- Added a method that converts a QRL wrapper name back to a marker name (e.g., `"globalActionQrl"` -> `"globalAction$"`) and looks up the original source module in `marker_fn_sources`. Falls back to `self.core_module` if not found.

**Wrapper imports use tracked source** -- The exit_program wrapper import loop now calls `self.find_wrapper_source(&wrapper)` instead of always using `self.core_module`. This fixes 10 fixtures where non-core marker functions (globalAction$, serverAuth$, etc.) had their QRL wrappers imported from the wrong source.

**Synthetic import emission** -- Added a new block in exit_program that emits imports from `collect.synthetic` (e.g., `_restProps` from props destructuring). These are inserted at position 0 after wrapper imports, so they end up at the top of the module.

**Marker stripping expanded to all sources** -- The marker function import stripping now removes $-suffixed imports from ALL source modules, not just core module. This matches SWC behavior where marker functions are stripped regardless of source.

**Dead import elimination expanded** -- The dead import elimination pass now covers ALL imports (not just core module). Side-effect-only imports (bare `import "module"`) are preserved. Synthetic imports from `collect.synthetic` are added to the protection set.

## Parity Results

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| Full match | 28/201 (14%) | 36/201 (18%) | +8 |
| Root module match | 28/201 | 36/201 | +8 |
| Segment count match | 195/201 | 195/201 | No change |
| Diagnostics match | 193/201 | 193/201 | No change |

## Deviations from Plan

None -- plan executed exactly as written.

## Decisions Made

- **D-10:** QRL wrapper imports use `find_wrapper_source()` helper that strips "Qrl" suffix and looks up marker source, falling back to core_module
- **D-11:** Marker function stripping expanded to all sources -- SWC strips markers regardless of source module
- **D-12:** Dead import elimination expanded to all imports -- removes unused specifiers from third-party imports too
- **D-13:** Synthetic imports emitted via same insert-at-0 pattern as other imports

## Known Stubs

None -- all changes are complete and functional.

## Self-Check: PASSED

- FOUND: crates/qwik-optimizer-oxc/src/transform.rs (contains marker_fn_sources, find_wrapper_source)
- FOUND: 06bafa9 (task 1 commit)
