# Requirements: Qwik Optimizer Specification & OXC Implementation

**Defined:** 2026-04-01
**Core Value:** The specification must be comprehensive and precise enough that an OXC implementation can be built from it without referencing the SWC source code.

## v1 Requirements

Requirements for initial release. Each maps to roadmap phases.

### Specification — Core Transformations

- [x] **SPEC-01**: Spec documents CONV-01 (Dollar Detection) — identification of `$`-suffixed function calls as marker functions requiring QRL extraction, including imported markers from `@qwik.dev/core` and locally-defined `$`-suffixed functions
- [x] **SPEC-02**: Spec documents CONV-02 (QRL Wrapping) — replacement of `$`-suffixed calls with `Qrl` counterparts, `qrl()`/`inlinedQrl()` reference generation, dev mode variants (`qrlDEV`, `inlinedQrlDEV`)
- [x] **SPEC-03**: Spec documents CONV-03 (Capture Analysis) — free variable detection across `$()` boundaries with full taxonomy of 8 capture categories (imports, inner locals, outer locals, loop variables, shadowed bindings, destructured params, hoisted functions, TS type-only imports)
- [x] **SPEC-04**: Spec documents CONV-04 (Props Destructuring) — transformation of destructured component props to `_rawProps` access patterns, `_restProps()` handling, pre-pass ordering requirement
- [x] **SPEC-05**: Spec documents CONV-05 (Segment Extraction) — extraction of `$()` callback bodies into separate output modules, canonical filename generation, hash suffixes, nested segment parent-child relationships, variable migration
- [x] **SPEC-06**: Spec documents CONV-06 (JSX Transform) — `_jsxSorted()`/`_jsxSplit()` conversion, static/dynamic prop separation, `class`/`className` normalization, `bind:value`/`bind:checked` sugar, `q:slot`, `ref`, children extraction, key counter generation
- [x] **SPEC-07**: Spec documents CONV-07 (Signal Optimization) — `_fnSignal()` generation for inline JSX expressions, parameterized function creation with positional params (`p0`, `p1`)
- [x] **SPEC-08**: Spec documents CONV-08 (PURE Annotations) — `/*#__PURE__*/` on `componentQrl` only, with explicit anti-list of side-effectful wrappers that must NOT be annotated
- [x] **SPEC-09**: Spec documents CONV-09 (Dead Branch Elimination) — unreachable code removal after const replacement, client-side tree-shaking via Treeshaker
- [x] **SPEC-10**: Spec documents CONV-10 (Const Replacement) — `isServer`/`isBrowser`/`isDev` replacement with boolean literals, import source handling, Test mode exception
- [x] **SPEC-11**: Spec documents CONV-11 (Code Stripping) — `strip_exports`, `strip_ctx_name`, `strip_event_handlers` mechanisms with throwing stub generation
- [x] **SPEC-12**: Spec documents CONV-12 (Import Rewriting) — legacy rename, consumed import stripping, synthetic import addition, per-segment import resolution
- [x] **SPEC-13**: Spec documents CONV-13 (sync$ Serialization) — `_qrlSync()` with stringified function body
- [x] **SPEC-14**: Spec documents CONV-14 (Noop QRL Handling) — `_noopQrl()`/`_noopQrlDEV()` for empty/unused callbacks

### Specification — Strategies & Modes

- [x] **SPEC-15**: Spec documents all 7 entry strategies (Inline, Hoist, Single, Hook, Segment, Component, Smart) with grouping rules and behavioral differences — noting Inline/Hoist shared EntryPolicy with Hoist's `.s()` registration post-processing
- [x] **SPEC-16**: Spec documents all 5 emit modes (Prod, Dev, Lib, Test, Hmr) with behavioral differences per transformation
- [x] **SPEC-17**: Spec documents the transformation pipeline ordering — which CONVs run before/after which, the dependency DAG, and why ordering matters

### Specification — Public API

- [x] **SPEC-18**: Spec documents TransformModulesOptions (all config fields with types, defaults, valid values)
- [x] **SPEC-19**: Spec documents TransformOutput, TransformModule, SegmentAnalysis, SegmentKind (all output fields with types and semantics)
- [x] **SPEC-20**: Spec documents the Diagnostic type and error/warning categories

### Specification — Infrastructure

- [x] **SPEC-21**: Spec documents GlobalCollect behavior (import/export/root-declaration analysis)
- [x] **SPEC-22**: Spec documents Variable Migration (dependency analysis, segment-exclusive root var movement, export cleanup)
- [x] **SPEC-23**: Spec documents Hash Generation algorithm (deterministic segment hashing for stable QRL identifiers)
- [x] **SPEC-24**: Spec documents Path Resolution (canonical filename generation, relative path computation, extension mapping)
- [x] **SPEC-25**: Spec documents Source Map Generation contracts for both root module and segment modules

### Specification — Bindings

- [x] **SPEC-26**: Spec documents NAPI binding contract (function signature, JSON serialization, async behavior)
- [x] **SPEC-27**: Spec documents WASM binding contract (function signature, wasm-bindgen interface, browser context)

### Specification — Cross-Cutting

- [x] **SPEC-28**: Spec includes OXC migration notes per transformation — where SWC and OXC patterns diverge (Fold vs Traverse, SyntaxContext vs Scoping, ownership vs arena allocation)
- [x] **SPEC-29**: Spec includes representative input/output examples extracted from Jack's 162 spec files (minimum 20 covering all 14 CONVs)
- [x] **SPEC-30**: Spec includes capture analysis taxonomy with edge case examples for all 8 categories

### Implementation — Core

- [x] **IMPL-01**: OXC implementation passes all 162 behavioral tests from Jack's spec corpus
- [x] **IMPL-02**: OXC implementation supports all 14 CONV transformation types
- [ ] **IMPL-03**: OXC implementation supports all 7 entry strategies
- [ ] **IMPL-04**: OXC implementation supports all 5 emit modes
- [x] **IMPL-05**: OXC implementation produces functionally equivalent output to SWC version (semantic equivalence, not byte-for-byte)

### Implementation — Bindings

- [x] **IMPL-06**: NAPI binding exposes `transform_modules` to Node.js with same JSON interface as SWC version
- [x] **IMPL-07**: WASM binding exposes `transform_modules` to browsers/edge with same interface as SWC version

### Implementation — Architecture

- [x] **IMPL-08**: OXC implementation uses idiomatic OXC patterns (Traverse trait, arena allocators, SemanticBuilder, Codegen) — not SWC patterns translated to OXC APIs
- [x] **IMPL-09**: OXC implementation uses semantic analysis (OXC Scoping) for capture analysis where it improves correctness over manual approaches

### Reference Material

- [ ] **REF-01**: Jack's 162 spec files at `/Users/scottweaver/Projects/qwik-oxc-optimizer/.planning/spec/` serve as the behavioral test corpus
- [ ] **REF-02**: Jack's OXC implementation at `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/` serves as a reference implementation
- [ ] **REF-03**: Scott's earlier OXC conversion at `/Users/scottweaver/Projects/qwik-optimizer/` serves as a reference for idiomatic OXC patterns and component identity approaches
- [ ] **REF-04**: SWC-based Qwik v2 optimizer at `/Users/scottweaver/Projects/qwik/packages/optimizer/` is the source of truth for behavioral correctness

## v2 Requirements

Deferred to future release. Tracked but not in current roadmap.

### Performance

- **PERF-01**: Benchmark OXC optimizer vs SWC optimizer on representative Qwik applications
- **PERF-02**: Parallel module processing via rayon

### Advanced

- **ADV-01**: Pre-compiled QRL extraction (handling `.qwik.mjs` library code with embedded segments)
- **ADV-02**: Incremental/cached transformation for watch mode
- **ADV-03**: TypeScript plugin integration (platform.ts binding swap for Qwik build pipeline)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Byte-for-byte SWC output matching | Cosmetic differences acceptable — semantic equivalence is the target |
| Bundler plugin integration (Vite/Rollup) | Optimizer is a standalone transform; plugin layer is separate |
| TypeScript type checking | Optimizer strips types but doesn't type-check — that's IDE/tsc |
| Custom minifier/DCE engine | Use OXC's built-in capabilities or post-processing |
| SWC Fold patterns in OXC code | Anti-pattern — use OXC's native Traverse/VisitMut |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| SPEC-01 | Phase 7 (gap closure) | Complete |
| SPEC-02 | Phase 7 (gap closure) | Complete |
| SPEC-03 | Phase 1 | Complete |
| SPEC-04 | Phase 2 | Complete |
| SPEC-05 | Phase 1 | Complete |
| SPEC-06 | Phase 9 (gap closure) | Complete |
| SPEC-07 | Phase 2 | Complete |
| SPEC-08 | Phase 3 | Complete |
| SPEC-09 | Phase 7 (gap closure) | Complete |
| SPEC-10 | Phase 7 (gap closure) | Complete |
| SPEC-11 | Phase 7 (gap closure) | Complete |
| SPEC-12 | Phase 1 | Complete |
| SPEC-13 | Phase 3 | Complete |
| SPEC-14 | Phase 3 | Complete |
| SPEC-15 | Phase 3 | Complete |
| SPEC-16 | Phase 3 | Complete |
| SPEC-17 | Phase 3 | Complete |
| SPEC-18 | Phase 9 (gap closure) | Complete |
| SPEC-19 | Phase 9 (gap closure) | Complete |
| SPEC-20 | Phase 9 (gap closure) | Complete |
| SPEC-21 | Phase 1 | Complete |
| SPEC-22 | Phase 1 | Complete |
| SPEC-23 | Phase 1 | Complete |
| SPEC-24 | Phase 1 | Complete |
| SPEC-25 | Phase 1 | Complete |
| SPEC-26 | Phase 4 | Complete |
| SPEC-27 | Phase 4 | Complete |
| SPEC-28 | Phase 4 | Complete |
| SPEC-29 | Phase 7 (gap closure) | Complete |
| SPEC-30 | Phase 1 | Complete |
| IMPL-01 | Phase 5 | Complete |
| IMPL-02 | Phase 8 (gap closure) | Complete |
| IMPL-03 | Phase 9 (gap closure) | Pending |
| IMPL-04 | Phase 9 (gap closure) | Pending |
| IMPL-05 | Phase 8 (gap closure) | Complete |
| IMPL-06 | Phase 6 | Complete |
| IMPL-07 | Phase 6 | Complete |
| IMPL-08 | Phase 5 | Complete |
| IMPL-09 | Phase 5 | Complete |
| REF-01 | — | Reference |
| REF-02 | — | Reference |
| REF-03 | — | Reference |
| REF-04 | — | Reference |

**Coverage:**
- v1 requirements: 35 phase-mapped + 4 reference = 39 total
- Mapped to phases: 35/35 (100%)
- Phase 1: 8 requirements (SPEC-03, 05, 12, 21, 22, 23, 24, 25, 30)
- Phase 2: 2 requirements (SPEC-04, 07)
- Phase 3: 6 requirements (SPEC-08, 13, 14, 15, 16, 17)
- Phase 4: 3 requirements (SPEC-26, 27, 28)
- Phase 5: 3 requirements (IMPL-01, 08, 09)
- Phase 6: 2 requirements (IMPL-06, 07)
- Phase 7 (gap closure): 6 requirements (SPEC-01, 02, 09, 10, 11, 29)
- Phase 8 (gap closure): 2 requirements (IMPL-02, 05)
- Phase 9 (gap closure): 6 requirements (SPEC-06, 18, 19, 20, IMPL-03, 04)

---
*Requirements defined: 2026-04-01*
*Last updated: 2026-04-03 after Phase 7 gap closure execution*
