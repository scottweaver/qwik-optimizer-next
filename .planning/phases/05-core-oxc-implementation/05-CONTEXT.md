# Phase 5: Core OXC Implementation - Context

**Gathered:** 2026-04-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Build a working `qwik-optimizer-oxc` Rust crate that implements all 14 CONV transformations using idiomatic OXC patterns. The crate must pass all 162 behavioral tests from Jack's SWC snapshot corpus plus spec-derived tests from Appendix B. This is the first implementation phase — entry strategies, emit modes, and bindings are Phase 6.

</domain>

<decisions>
## Implementation Decisions

### Crate Structure & Workspace
- **D-33:** The new crate lives at `crates/qwik-optimizer-oxc/` in the existing `qwik-optimizer-next` workspace (alongside `specification/`). Add to the root `Cargo.toml` workspace members.

### Reference Code Reuse
- **D-34:** Spec-driven fresh build. Write from the spec document (`specification/qwik-optimizer-spec.md`). Consult Jack's implementation and Scott's conversion when stuck on OXC API usage, but don't copy-paste. The goal is idiomatic design driven by the spec, not inherited patterns.

### Test Strategy
- **D-35:** Both test sources: (1) Copy Jack's 162 SWC snapshots (`swc-snapshots/*.snap`) as the full regression suite using insta for snapshot testing. (2) Derive focused behavioral tests from the 24+ spec examples in Appendix B.

### OXC Version
- **D-36:** Target the latest stable OXC release at build time. Pin with exact version in Cargo.toml. Upgrade deliberately with the 162-test suite as the gating mechanism.

### Carrying Forward
- D-05: SWC is source of truth for behavioral correctness
- D-08 (from IMPL requirements): Idiomatic OXC patterns (Traverse, arena allocators, SemanticBuilder, Codegen)
- D-09 (from IMPL requirements): OXC Scoping for capture analysis

### Claude's Discretion
- Internal module organization within the crate (how to split transform.rs, collector.rs, etc.)
- Whether to use a single `traverse_mut` pass or two-phase analyze-then-emit (both are valid OXC approaches per research)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary Reference (Spec Document)
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/qwik-optimizer-spec.md` — The complete 8,091-line behavioral specification. This is the authoritative reference for all implementation work.

### SWC Source (Behavioral Truth)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/` — All SWC source files for verifying spec accuracy when questions arise

### Jack's OXC Implementation (OXC API Reference)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/` — Reference for OXC API usage patterns
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/swc-snapshots/` — 162 SWC reference snapshots (TEST CORPUS — copy these)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/oxc-snapshots/` — 162 OXC output snapshots (reference only)

### Scott's OXC Conversion (Pattern Reference)
- `/Users/scottweaver/Projects/qwik-optimizer/optimizer/src/` — Earlier OXC conversion with idiomatic patterns
- `/Users/scottweaver/Projects/qwik-optimizer/optimizer/Cargo.toml` — OXC 0.94.0 dependency configuration

### Research
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/.planning/research/STACK.md` — OXC crate recommendations
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/.planning/research/ARCHITECTURE.md` — Two-phase vs single-pass architecture analysis
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/.planning/research/PITFALLS.md` — 15 domain pitfalls including capture analysis bugs

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- The `qwik-optimizer-next` workspace already has a `Cargo.toml` at the root with a `src/main.rs` hello world
- Jack's 162 `.snap` files are the ready-made test corpus
- The spec document's Appendix A (OXC Migration Guide) maps each SWC pattern to its OXC equivalent
- The spec's Appendix B provides 24+ curated examples with expected outputs

### Established Patterns
- OXC `Traverse` trait with `enter_*/exit_*` methods for AST traversal
- Arena allocation via `oxc_allocator::Allocator`
- `SemanticBuilder` for scope/symbol analysis
- `Codegen` for JavaScript output generation
- String-based segment construction (collect during traversal, build post-traversal)

### Integration Points
- The crate exposes `transform_modules(config: TransformModulesOptions) -> Result<TransformOutput, Error>` as its public API
- Phase 6 adds entry strategies, emit modes, and NAPI/WASM bindings on top of this core

</code_context>

<specifics>
## Specific Ideas

- Start with the foundation: parsing, GlobalCollect, hash generation, path resolution
- Then core transform: dollar detection, capture analysis, QRL wrapping, segment extraction
- Then JSX, props destructuring, signal optimization
- Build mode transforms (const replace, DCE, stripping) and import rewriting last
- The 162-test suite should be running from the earliest possible moment to catch regressions
- Capture analysis is the highest-risk area — 82% of Jack's runtime bugs originated there

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 05-core-oxc-implementation*
*Context gathered: 2026-04-02*
