# Phase 4: Public API, Bindings & Cross-Cutting Specification - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Write the final specification sections that complete the document: public API type definitions (TransformModulesOptions, TransformOutput, SegmentAnalysis, Diagnostics), NAPI and WASM binding contracts, per-transformation OXC migration notes, and 20+ representative examples covering all 14 CONVs. These are appended to `specification/qwik-optimizer-spec.md` (currently 5,254 lines from Phases 1-3).

</domain>

<decisions>
## Implementation Decisions

### OXC Migration Notes
- **D-29:** OXC migration notes in a dedicated appendix section ("OXC Migration Guide") at the end of the spec. Grouped by transformation with SWC→OXC pattern mapping. Does NOT modify existing Phases 1-3 content. Uses Scott's earlier OXC conversion (per D-07) for concrete pattern examples.

### Representative Examples
- **D-30:** A "Representative Examples" appendix section with 20+ curated examples from Jack's 162 snapshots, covering all 14 CONVs. Complements the inline examples already in each CONV section from Phases 1-3.

### API Type Documentation
- **D-31:** TransformModulesOptions, TransformOutput, and related types documented as actual Rust struct type definitions with doc comments. Precise and directly useful for implementation.

### Binding Contracts
- **D-32:** NAPI and WASM bindings documented minimally: function signature, JSON serialization format, async behavior (NAPI), platform-specific gotchas. These are thin wrappers — don't over-document.

### Carrying Forward
- D-01: Pipeline-ordered structure
- D-05: SWC is source of truth
- D-06: SWC source references
- D-07: Scott's OXC conversion for migration examples
- D-13/16: Example format with snapshot names

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### SWC Source (Source of Truth)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/lib.rs` — TransformModulesOptions, TransformOutput, SegmentAnalysis, all public types
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/utils.rs` — Diagnostic type
- `/Users/scottweaver/Projects/qwik/packages/optimizer/napi/src/lib.rs` — NAPI binding (transform_modules async wrapper)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/wasm/src/lib.rs` — WASM binding (wasm-bindgen)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/src/types.ts` — TypeScript type definitions (JS-side contract)

### Jack's OXC Implementation (Reference)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/types.rs` — OXC type definitions
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/swc-snapshots/` — 162 SWC reference snapshots
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/oxc-snapshots/` — 162 OXC output snapshots

### Scott's OXC Conversion (Migration Pattern Reference)
- `/Users/scottweaver/Projects/qwik-optimizer/optimizer/src/transform.rs` — Traverse patterns
- `/Users/scottweaver/Projects/qwik-optimizer/optimizer/src/component/` — Component identity structures
- `/Users/scottweaver/Projects/qwik-optimizer/napi/src/lib.rs` — NAPI binding patterns

### Existing Spec (Phases 1-3 Output)
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/qwik-optimizer-spec.md` — Current spec (5,254 lines)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phases 1-3 already have ~15-20 inline examples across all CONVs — the appendix adds curated comprehensive examples
- Research ARCHITECTURE.md and PITFALLS.md contain OXC migration insights that feed into the appendix

### Integration Points
- API type section goes near the top of the spec (after Pipeline Overview, before transformations)
- OXC Migration Guide appendix goes at the end
- Representative Examples appendix goes at the end
- Binding contracts go after transformation sections, before appendices

</code_context>

<specifics>
## Specific Ideas

- The Rust type definitions should match SWC lib.rs exactly for behavioral fidelity
- OXC migration notes should cover: Fold→Traverse, SyntaxContext→Scoping, ownership→arena, GlobalCollect→SemanticBuilder, code_move string-based construction
- Example selection should prioritize diversity: basic transforms, nested captures, JSX with signals, build mode variants, entry strategy differences

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 04-public-api-bindings-cross-cutting-specification*
*Context gathered: 2026-04-01*
