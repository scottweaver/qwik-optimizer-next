---
phase: 04-public-api-bindings-cross-cutting-specification
plan: 02
subsystem: binding-contracts-migration-guide
tags: [spec, napi, wasm, oxc, migration, bindings]
dependency_graph:
  requires: [04-01]
  provides: [SPEC-26, SPEC-27, SPEC-28]
  affects: [04-03, phase-05, phase-06]
tech_stack:
  added: []
  patterns: [ffi-contract-documentation, swc-to-oxc-migration-patterns, per-conv-traceability]
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - "D-32 applied: Binding contracts kept minimal -- function signature, serialization, async behavior, error handling, platform notes only"
  - "D-29 applied: OXC migration guide is a dedicated Appendix A, not inline modifications to Phases 1-3 content"
  - "D-07 applied: Scott's transform.rs referenced for concrete OXC Traverse patterns in migration examples"
  - "NAPI v3 migration notes included as forward-looking subsection (no tokio, wasm32-wasip1-threads, proc macro)"
metrics:
  duration: 4m
  completed: 2026-04-02T00:41:00Z
  tasks: 2
  files: 1
---

# Phase 04 Plan 02: Binding Contracts and OXC Migration Guide Summary

NAPI/WASM binding contracts with function signatures, serialization formats, async/sync behavior, and error handling per D-32; Appendix A with 6 SWC-to-OXC migration patterns and per-CONV summary table per D-29.

## What Was Done

### Task 1: Binding Contracts Section (e3f9995)
Added `## Binding Contracts` section after Public API Types with 3 subsections:
- **NAPI Binding**: `#[js_function(1)]` signature, serde-based JS object deserialization, `execute_tokio_future` + `spawn_blocking` async Promise return, `napi::Error::from_reason` string errors, Windows mimalloc allocator note
- **WASM Binding**: `#[wasm_bindgen]` signature, `serde_wasm_bindgen::from_value` input / `Serializer::new().serialize_maps_as_objects(true)` output, synchronous blocking behavior with Web Worker recommendation, `js_sys::Error` wrapping
- **NAPI v3 Migration Notes**: 3 key improvements for OXC implementation -- native async (no tokio), wasm32-wasip1-threads potential, `#[napi]` proc macro simplification

Section is 113 lines total, well under the 200-line target per D-32 minimal binding contracts.

### Task 2: Appendix A OXC Migration Guide (09d528b)
Added `## Appendix A: OXC Migration Guide` with 6 migration patterns and a summary table:
- **Pattern 1: Fold/VisitMut to Traverse Trait** -- ownership fold vs mutable reference enter/exit hooks, TraverseCtx for scope/AST/ancestors
- **Pattern 2: SyntaxContext to Scoping/SymbolId** -- (Atom, SyntaxContext) identity vs SemanticBuilder side tables with SymbolId/ReferenceId
- **Pattern 3: Ownership Transfer to Arena Allocation** -- Box<Expr> heap vs Box<'a, Expression<'a>> arena, AstBuilder for new nodes
- **Pattern 4: Code Move from AST Cloning to String Construction** -- cross-arena impossibility resolved by span-based source slicing
- **Pattern 5: GlobalCollect to SemanticBuilder + Collector** -- manual visitor replaced by semantic pre-pass plus domain-specific collector
- **Pattern 6: Deferred Statement Insertion via exit_program** -- pending_imports/pending_hoisted Vecs flushed at exit_program hook
- **Per-CONV Migration Summary**: Table mapping all 14 CONVs to their most relevant migration patterns

## Deviations from Plan

None -- plan executed exactly as written.

## Verification

- `## Binding Contracts` heading found at line 6141
- `## Appendix A: OXC Migration Guide` heading found at line 6254
- `### Migration Pattern` count: 6 (all 6 patterns documented)
- Section ordering confirmed: Public API Types (5256) -> Binding Contracts (6141) -> Appendix A (6254)
- SPEC-26 satisfied: NAPI binding contract with function signature, JSON serialization, async Promise behavior
- SPEC-27 satisfied: WASM binding contract with function signature, synchronous behavior, Web Worker recommendation
- SPEC-28 satisfied: 6 migration patterns with SWC/OXC code examples and per-CONV summary covering all 14 CONVs
- No existing Phases 1-3 content modified (appendix-only addition per D-29)

## Known Stubs

None -- all binding contracts and migration patterns are fully documented with code examples.
