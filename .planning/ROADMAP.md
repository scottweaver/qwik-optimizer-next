# Roadmap: Qwik Optimizer Specification & OXC Implementation

## Milestones

- [x] **v0.1.0 -- Qwik Optimizer Spec & OXC Implementation** -- Phases 1-9 (shipped 2026-04-03)
- [ ] **v0.2.0 -- Full SWC Parity** -- Phases 10-13 (in progress)

## Phases

<details>
<summary>v0.1.0 (Phases 1-9) -- SHIPPED 2026-04-03</summary>

- [x] Phase 1: Core Pipeline Specification (5/5 plans) -- completed 2026-04-02
- [x] Phase 2: JSX, Props & Signal Specification (3/3 plans) -- completed 2026-04-02
- [x] Phase 3: Build Modes & Remaining Transforms Specification (4/4 plans) -- completed 2026-04-02
- [x] Phase 4: Public API, Bindings & Cross-Cutting Specification (3/3 plans) -- completed 2026-04-02
- [x] Phase 5: Core OXC Implementation (7/7 plans) -- completed 2026-04-02
- [x] Phase 6: Strategies, Modes & Binding Implementation (3/3 plans) -- completed 2026-04-02
- [x] Phase 7: Spec Gap Closure (2/2 plans) -- completed 2026-04-03
- [x] Phase 8: Implementation Gap Closure (5/5 plans) -- completed 2026-04-03
- [x] Phase 9: Metadata & Verification Cleanup (3/3 plans) -- completed 2026-04-03

Full details: [v0.1.0-ROADMAP.md](milestones/v0.1.0-ROADMAP.md)

</details>

### v0.2.0 Full SWC Parity (In Progress)

**Milestone Goal:** Achieve 201/201 full SWC parity -- every OXC-transformed fixture produces output matching the SWC reference for root module, segment count, and diagnostics.

**Baseline:** 28/201 (14%) full match | 57/201 root match | 125/201 segment count match | 197/201 diagnostics match

- [x] **Phase 10: Segment Extraction** - Fix dollar-sign expression detection in loops, JSX handlers, ternaries, spread props, and inline strategies (completed 2026-04-04)
- [x] **Phase 11: Root Module Code Generation** - Fix import ordering, declarations, exports, QRL references, and comment structure (completed 2026-04-06)
- [x] **Phase 12: Diagnostics Parity** - Fix error reporting to match SWC for all 201 fixtures (completed 2026-04-06)
- [ ] **Phase 13: Final Acceptance** - Verify 201/201 full match and fix any remaining stragglers

## Phase Details

### Phase 10: Segment Extraction
**Goal**: The optimizer correctly extracts separate segments for every dollar-sign expression regardless of AST position -- loops, JSX handlers, ternaries, spread props, and inline strategies
**Depends on**: Nothing (first phase of v0.2.0)
**Requirements**: SEG-01, SEG-02, SEG-03, SEG-04, SEG-05
**Success Criteria** (what must be TRUE):
  1. Dollar-sign expressions inside for/for-of/while loop bodies each produce their own segment (not collapsed into parent)
  2. Multiple event handler attributes on a single JSX element each produce separate segments
  3. Nested combinations of loops and ternary expressions with dollar-sign calls produce the correct total segment count matching SWC
  4. JSX elements with spread props plus additional handler props produce the correct number of segments
  5. Fixtures using inline and inlined QRL strategies produce segment counts matching SWC output
**Plans**: 2 plans

Plans:
- [x] 10-01-PLAN.md — JSX attribute segment extraction: port jsx_event_to_html_attribute, restructure classify_props, implement segment extraction for $-suffixed JSX attributes
- [x] 10-02-PLAN.md — Straggler fixes: diagnose and fix remaining segment count mismatches (loop captures, bind expansion, inline strategy edge cases)

### Phase 11: Root Module Code Generation
**Goal**: The root module output for every fixture matches SWC in import ordering, variable declarations, export structure, QRL references, and comment separators
**Depends on**: Phase 10
**Requirements**: ROOT-01, ROOT-02, ROOT-03, ROOT-04, ROOT-05
**Success Criteria** (what must be TRUE):
  1. Import statements in root module output appear in the same order as SWC output (framework imports, segment imports, source re-exports)
  2. Variable declarations and expressions in the root module match SWC output structure
  3. Default exports, re-exports, and named exports in root module match SWC format
  4. QRL references and hoisted constant declarations match SWC naming and placement
  5. Comment separators between logical sections of root module output match SWC whitespace structure
**Plans**: 4 plans

Plans:
- [x] 11-01-PLAN.md — Import assembly rewrite: marker source tracking, synthetic import emission, SWC-ordered import assembly, expanded dead import elimination
- [x] 11-02-PLAN.md — Comprehensive dead code elimination: SWC-matching fixpoint loop removing unused vars AND imports after variable migration
- [x] 11-03-PLAN.md — Display name / hash fix: add marker function name to stack_ctxt for correct segment hashes
- [x] 11-04-PLAN.md — Body structure cleanup: diagnose and fix remaining mismatches (unreferenced var stripping, expression ordering, separators)

### Phase 12: Diagnostics Parity
**Goal**: Error diagnostics match SWC for all 201 fixtures -- same errors reported for invalid references and missing custom inlined functions
**Depends on**: Phase 10
**Requirements**: DIAG-01, DIAG-02
**Success Criteria** (what must be TRUE):
  1. Invalid reference errors and missing custom inlined function errors match SWC diagnostic output
  2. All 201 fixtures produce the same diagnostic presence/absence as SWC (192/201 baseline, fix remaining 9)
**Plans**: 1 plan

Plans:
- [x] 12-01-PLAN.md — C02 export-symbol gate + C05 MissingQrlImplementation: suppress 8 false positive C02 diagnostics, implement missing C05 diagnostic for 1 false negative

### Phase 13: Final Acceptance
**Goal**: Full SWC parity achieved -- 201/201 fixtures match on root module, segment count, and diagnostics
**Depends on**: Phase 11, Phase 12
**Requirements**: ACC-01
**Success Criteria** (what must be TRUE):
  1. Parity report shows 201/201 full match (root module + segment count + diagnostics all green)
  2. No regressions from v0.1.0 baseline (all previously passing tests still pass)
  3. Any stragglers discovered during final verification are identified, fixed, and re-verified
**Plans**: TBD

Plans:
- [ ] 13-01: TBD

## Progress

**Execution Order:**
Phases execute in numeric order: 10 -> 10.x -> 11 -> 11.x -> 12 -> 12.x -> 13

| Phase | Milestone | Plans Complete | Status | Completed |
|-------|-----------|----------------|--------|-----------|
| 1. Core Pipeline Specification | v0.1.0 | 5/5 | Complete | 2026-04-02 |
| 2. JSX, Props & Signal Specification | v0.1.0 | 3/3 | Complete | 2026-04-02 |
| 3. Build Modes & Remaining Transforms | v0.1.0 | 4/4 | Complete | 2026-04-02 |
| 4. Public API, Bindings & Cross-Cutting | v0.1.0 | 3/3 | Complete | 2026-04-02 |
| 5. Core OXC Implementation | v0.1.0 | 7/7 | Complete | 2026-04-02 |
| 6. Strategies, Modes & Bindings | v0.1.0 | 3/3 | Complete | 2026-04-02 |
| 7. Spec Gap Closure | v0.1.0 | 2/2 | Complete | 2026-04-03 |
| 8. Implementation Gap Closure | v0.1.0 | 5/5 | Complete | 2026-04-03 |
| 9. Metadata & Verification Cleanup | v0.1.0 | 3/3 | Complete | 2026-04-03 |
| 10. Segment Extraction | v0.2.0 | 2/2 | Complete    | 2026-04-04 |
| 11. Root Module Code Generation | v0.2.0 | 4/4 | Complete   | 2026-04-06 |
| 12. Diagnostics Parity | v0.2.0 | 1/1 | Complete    | 2026-04-06 |
| 13. Final Acceptance | v0.2.0 | 0/1 | Not started | - |
