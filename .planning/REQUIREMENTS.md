# Requirements: Qwik Optimizer OXC -- Full SWC Parity

**Defined:** 2026-04-03
**Core Value:** The OXC implementation must produce functionally equivalent output to the SWC version for all 201 test fixtures.

## v0.2.0 Requirements

Requirements for full SWC parity. Each maps to roadmap phases.

### Root Module Code Generation

- [x] **ROOT-01**: Root module import statements match SWC ordering and format for all fixtures
- [x] **ROOT-02**: Root module variable declarations and expressions match SWC output
- [x] **ROOT-03**: Root module export structure matches SWC output (default exports, re-exports, named exports)
- [x] **ROOT-04**: Root module QRL references and hoisted declarations match SWC format
- [x] **ROOT-05**: Root module comment separators and whitespace structure match SWC output

### Segment Extraction

- [x] **SEG-01**: Dollar-sign expressions inside for/for-of/while loops produce separate segments per iteration handler
- [x] **SEG-02**: Multiple event handlers on JSX elements each produce separate segments
- [x] **SEG-03**: Nested loop and ternary dollar-sign expressions produce correct segment count
- [x] **SEG-04**: Spread props with additional handler props produce correct segments
- [x] **SEG-05**: Inline and inlined QRL strategies produce correct segment counts

### Diagnostics

- [x] **DIAG-01**: Error diagnostics match SWC for invalid references and missing custom inlined functions
- [x] **DIAG-02**: Diagnostic presence/absence matches SWC for all 201 fixtures

### Acceptance

- [x] **ACC-01**: Parity report shows 201/201 full match (root module + segment count + diagnostics)

## Future Requirements

### Performance & Parallelism

- **PERF-01**: Performance benchmarking -- OXC optimizer vs SWC optimizer on representative Qwik applications
- **PERF-02**: Parallel module processing via rayon (feature-gated behind `parallel`)

## Out of Scope

| Feature | Reason |
|---------|--------|
| Source map parity | Source maps are not compared in the parity report; focus on code output |
| Segment code content parity | Parity report checks segment count, not segment code content |
| SegmentAnalysis JSON parity | Metadata format differences are acceptable per project constraints |
| New transformation types | v0.2.0 is about fixing existing transformations, not adding new ones |

## Traceability

| Requirement | Phase | Status |
|-------------|-------|--------|
| ROOT-01 | Phase 11 | Complete |
| ROOT-02 | Phase 11 | Complete |
| ROOT-03 | Phase 11 | Complete |
| ROOT-04 | Phase 11 | Complete |
| ROOT-05 | Phase 11 | Complete |
| SEG-01 | Phase 10 | Complete |
| SEG-02 | Phase 10 | Complete |
| SEG-03 | Phase 10 | Complete |
| SEG-04 | Phase 10 | Complete |
| SEG-05 | Phase 10 | Complete |
| DIAG-01 | Phase 12 | Complete |
| DIAG-02 | Phase 12 | Complete |
| ACC-01 | Phase 13 | Complete |

**Coverage:**
- v0.2.0 requirements: 13 total
- Mapped to phases: 13
- Unmapped: 0

---
*Requirements defined: 2026-04-03*
*Last updated: 2026-04-03 after roadmap creation*
