# Roadmap: Qwik Optimizer Specification & OXC Implementation

## Overview

This roadmap delivers a comprehensive behavioral specification of the Qwik v2 optimizer followed by a feature-complete OXC implementation. The spec is built incrementally across four phases — starting with the core QRL pipeline and capture taxonomy (the hardest part), then layering JSX and props transforms, build mode machinery, and finally the public API contract with binding specs. Two implementation phases follow: first the core transform engine validated against Jack's 162 spec files, then entry strategies, emit modes, and NAPI/WASM bindings. Every spec phase adds sections to a single comprehensive markdown document; every implementation phase builds on the spec as its authoritative reference.

## Phases

**Phase Numbering:**
- Integer phases (1, 2, 3): Planned milestone work
- Decimal phases (2.1, 2.2): Urgent insertions (marked with INSERTED)

Decimal phases appear between their surrounding integers in numeric order.

- [x] **Phase 1: Core Pipeline Specification** - Specify dollar detection, QRL wrapping, capture analysis, segment extraction, import rewriting, and supporting infrastructure
- [x] **Phase 2: JSX, Props & Signal Specification** - Specify the JSX transform subsystem, props destructuring, and signal optimization
- [ ] **Phase 3: Build Modes & Remaining Transforms Specification** - Specify PURE annotations, const replacement, DCE, code stripping, sync$, noop QRL, entry strategies, emit modes, and pipeline ordering
- [ ] **Phase 4: Public API, Bindings & Cross-Cutting Specification** - Specify the public API contract, NAPI/WASM bindings, OXC migration notes, and representative examples
- [ ] **Phase 5: Core OXC Implementation** - Implement the core transform engine passing all 162 behavioral tests with idiomatic OXC patterns
- [ ] **Phase 6: Strategies, Modes & Binding Implementation** - Implement all entry strategies, emit modes, NAPI and WASM bindings for drop-in replacement

## Phase Details

### Phase 1: Core Pipeline Specification
**Goal**: The spec document contains complete behavioral descriptions of the core QRL extraction pipeline — the transformations that every other feature depends on — plus the capture analysis taxonomy that is the single highest-risk area of the entire project
**Depends on**: Nothing (first phase)
**Requirements**: SPEC-01, SPEC-02, SPEC-03, SPEC-05, SPEC-12, SPEC-21, SPEC-22, SPEC-23, SPEC-24, SPEC-25, SPEC-30
**Success Criteria** (what must be TRUE):
  1. The spec document describes dollar detection rules such that a reader can determine whether any given function call triggers QRL extraction without consulting SWC source
  2. The spec document contains the complete 8-category capture analysis taxonomy with edge case examples for each category, including the self-import reclassification behavior for module-level declarations
  3. The spec document describes segment extraction behavior — filename generation, hash computation, nested segment relationships, and variable migration — with input/output examples
  4. The spec document describes import rewriting rules (consumed import stripping, synthetic import addition, per-segment resolution) with before/after examples
  5. The spec document describes source map generation contracts for both root and segment modules
**Plans:** 1/5 plans executed

Plans:
- [x] 01-01-PLAN.md — Pipeline Overview, GlobalCollect, Hash Generation, Path Resolution (infrastructure)
- [ ] 01-02-PLAN.md — Dollar Detection (CONV-01), QRL Wrapping (CONV-02)
- [x] 01-03-PLAN.md — Capture Analysis (CONV-03) with 8-category taxonomy and 16 edge cases
- [ ] 01-04-PLAN.md — Segment Extraction (CONV-05), Import Rewriting (CONV-12)
- [x] 01-05-PLAN.md — Variable Migration, Source Map Generation

### Phase 2: JSX, Props & Signal Specification
**Goal**: The spec document contains complete behavioral descriptions of the JSX transform subsystem (the largest single component), props destructuring, and signal optimization — building on the core pipeline specified in Phase 1
**Depends on**: Phase 1
**Requirements**: SPEC-04, SPEC-06, SPEC-07
**Success Criteria** (what must be TRUE):
  1. The spec document describes JSX transformation rules (`_jsxSorted`/`_jsxSplit` conversion, static/dynamic prop separation, class normalization, bind sugar, slot/ref/children/key handling) with input/output examples
  2. The spec document describes signal optimization rules (`_fnSignal` generation for inline JSX expressions, positional parameter creation) with examples showing when optimization applies vs when it does not
  3. The spec document describes props destructuring transformation (`_rawProps` access patterns, `_restProps()` handling) and explicitly states the pre-pass ordering requirement relative to capture analysis
**Plans:** 3 plans

Plans:
- [ ] 02-01-PLAN.md — Props Destructuring (CONV-04): _rawProps, _restProps, pre-pass ordering
- [x] 02-02-PLAN.md — JSX Transform (CONV-06): branch point, element types, prop classification, special attributes, children, keys, flags, spreads
- [ ] 02-03-PLAN.md — Signal Optimization (CONV-07): _fnSignal, _wrapProp, decision table, hoisting

### Phase 3: Build Modes & Remaining Transforms Specification
**Goal**: The spec document contains complete behavioral descriptions for all remaining CONV transformations and the strategy/mode system that controls optimizer behavior across different build contexts
**Depends on**: Phase 1
**Requirements**: SPEC-08, SPEC-09, SPEC-10, SPEC-11, SPEC-13, SPEC-14, SPEC-15, SPEC-16, SPEC-17
**Success Criteria** (what must be TRUE):
  1. The spec document describes all 7 entry strategies with grouping rules, behavioral differences, and the Inline/Hoist shared EntryPolicy distinction
  2. The spec document describes all 5 emit modes with per-transformation behavioral differences (especially dev mode QRL variants and test mode const replacement exceptions)
  3. The spec document describes the transformation pipeline ordering DAG — which CONVs run before/after which, and why ordering matters (e.g., const replacement before DCE, props destructuring before capture analysis)
  4. The spec document describes PURE annotations with the explicit whitelist (componentQrl only) and anti-list of side-effectful wrappers, const replacement, dead branch elimination, code stripping, sync$ serialization, and noop QRL handling
**Plans:** 4 plans

Plans:
- [ ] 03-01-PLAN.md — Build Environment Transforms: Const Replacement (CONV-10), Dead Branch Elimination (CONV-09), Code Stripping (CONV-11)
- [x] 03-02-PLAN.md — QRL Special Cases: PURE Annotations (CONV-08), sync$ Serialization (CONV-13), Noop QRL Handling (CONV-14)
- [x] 03-03-PLAN.md — Entry Strategies (all 7) and Emit Modes (all 5 with Mode x CONV table)
- [ ] 03-04-PLAN.md — Transformation Pipeline: Mermaid DAG and ordering constraints table

### Phase 4: Public API, Bindings & Cross-Cutting Specification
**Goal**: The spec document is complete — all public-facing contracts are documented, OXC migration guidance is embedded per-transformation, and representative examples from Jack's 162 spec files are included as verification anchors
**Depends on**: Phase 1, Phase 2, Phase 3
**Requirements**: SPEC-18, SPEC-19, SPEC-20, SPEC-26, SPEC-27, SPEC-28, SPEC-29
**Success Criteria** (what must be TRUE):
  1. The spec document contains the complete TransformModulesOptions type definition with all config fields, types, defaults, and valid values
  2. The spec document contains the complete TransformOutput, TransformModule, SegmentAnalysis, and SegmentKind type definitions with field semantics
  3. The spec document contains NAPI and WASM binding contracts (function signatures, serialization, async behavior) sufficient to implement bindings without referencing SWC source
  4. The spec document contains OXC migration notes per transformation section — explicitly calling out where SWC and OXC patterns diverge (Fold vs Traverse, SyntaxContext vs Scoping, ownership vs arena)
  5. The spec document contains at least 20 representative input/output examples covering all 14 CONVs, extracted from Jack's 162 spec files
**Plans**: TBD

### Phase 5: Core OXC Implementation
**Goal**: A working qwik-core Rust crate implements all 14 CONV transformations using idiomatic OXC patterns, passing all 162 behavioral tests from Jack's spec corpus
**Depends on**: Phase 4
**Requirements**: IMPL-01, IMPL-02, IMPL-05, IMPL-08, IMPL-09
**Success Criteria** (what must be TRUE):
  1. Running `cargo test` executes all 162 behavioral test cases from Jack's spec corpus and all pass (semantic equivalence, not byte-for-byte matching)
  2. The implementation uses OXC's Traverse trait with enter/exit hooks, arena allocators, SemanticBuilder, and Codegen — no SWC patterns (Fold, SyntaxContext, std::mem::replace ownership transfer)
  3. The implementation uses OXC Scoping for capture analysis where it improves correctness over manual scope tracking
  4. All 14 CONV transformation types produce functionally equivalent output to the SWC version for the full test corpus
**Plans**: TBD

### Phase 6: Strategies, Modes & Binding Implementation
**Goal**: The optimizer is a drop-in replacement for the SWC version — all entry strategies and emit modes work, and Node.js/browser consumers can call it through NAPI and WASM bindings with the same JSON interface
**Depends on**: Phase 5
**Requirements**: IMPL-03, IMPL-04, IMPL-06, IMPL-07
**Success Criteria** (what must be TRUE):
  1. All 7 entry strategies (Inline, Hoist, Single, Hook, Segment, Component, Smart) produce correct segment grouping and output module organization
  2. All 5 emit modes (Prod, Dev, Lib, Test, Hmr) produce correct behavioral variations (dev QRL variants, test const exceptions, etc.)
  3. The NAPI binding exposes `transform_modules` to Node.js with the same JSON interface as the SWC version and produces equivalent output
  4. The WASM binding exposes `transform_modules` to browsers/edge with the same interface as the SWC version
**Plans**: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 1 -> 2 -> 3 -> 4 -> 5 -> 6

| Phase | Plans Complete | Status | Completed |
|-------|----------------|--------|-----------|
| 1. Core Pipeline Specification | 1/5 | In Progress|  |
| 2. JSX, Props & Signal Specification | 1/3 | In Progress | - |
| 3. Build Modes & Remaining Transforms Specification | 2/4 | In Progress | - |
| 4. Public API, Bindings & Cross-Cutting Specification | 0/? | Not started | - |
| 5. Core OXC Implementation | 0/? | Not started | - |
| 6. Strategies, Modes & Binding Implementation | 0/? | Not started | - |
