# Phase 3: Build Modes & Remaining Transforms Specification - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Write the behavioral specification sections for PURE annotations (CONV-08), dead branch elimination (CONV-09), const replacement (CONV-10), code stripping (CONV-11), sync$ serialization (CONV-13), noop QRL handling (CONV-14), all 7 entry strategies, all 5 emit modes, and the transformation pipeline ordering DAG. These sections are appended to the existing `specification/qwik-optimizer-spec.md` document (currently 3,817 lines from Phases 1-2).

</domain>

<decisions>
## Implementation Decisions

### Entry Strategies
- **D-25:** Each of the 5 distinct EntryPolicy implementations gets its own subsection with grouping rules and an example. Notes that Inline/Hoist share the same EntryPolicy (InlineStrategy) but Hoist adds unique `.s()` registration post-processing. Hook/Segment share PerSegmentStrategy.

### Emit Modes
- **D-26:** Emit modes documented as a cross-reference table (Mode × CONV → behavioral difference), followed by brief per-mode descriptions. Compact and scannable for an implementer checking "what changes in Dev mode?"

### Smaller CONVs (PURE, const replace, DCE, stripping, sync$, noop)
- **D-27:** Organization at Claude's discretion — either individual sections or grouped under an umbrella heading, based on how naturally they cluster. Each gets rules + 1-2 examples (shorter than the big CONVs).

### Pipeline Ordering DAG
- **D-28:** Mermaid DAG diagram showing transformation dependencies (consistent with Phase 1's pipeline diagram), plus a constraints table listing each ordering dependency with rationale.

### Carrying Forward from Prior Phases
- D-01: Pipeline-ordered document structure
- D-04: Rules + examples per CONV section
- D-05: SWC is source of truth
- D-06: SWC source file references for traceability
- D-13: Examples show input + all output modules
- D-16: Descriptive names with Jack's snapshot name in parentheses
- D-24: Inline examples + "See also" snapshot lists per subsection

### Claude's Discretion
- Smaller CONVs organization (individual vs grouped) — D-27

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### SWC Source (Source of Truth)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/parse.rs` — Pipeline orchestration with emit mode branching, const replacement call site, DCE passes (~1,798 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/const_replace.rs` — isServer/isBrowser/isDev replacement (~96 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/clean_side_effects.rs` — Treeshaker for client-side DCE (~90 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/filter_exports.rs` — Export stripping via strip_exports/strip_ctx_name (~76 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/entry_strategy.rs` — All 5 EntryPolicy implementations (~124 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` — PURE annotation logic, sync$ handling, noop QRL, emit mode conditionals (~5,157 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/lib.rs` — EmitMode enum, EntryStrategy enum, MinifyMode

### Jack's OXC Implementation (Reference)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/const_replace.rs` — OXC const replacement (~294 LOC)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/filter_exports.rs` — OXC export stripping (~136 LOC)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/swc-snapshots/` — Snapshots for build modes, stripping, etc.

### Existing Spec (Phases 1-2 Output)
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/qwik-optimizer-spec.md` — Current spec (3,817 lines)

### Research
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/.planning/research/FEATURES.md` — Entry strategies table, emit modes table, CONV dependency DAG

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 1 Pipeline Overview already shows the 20-step transformation sequence — the DAG section formalizes the dependency relationships
- Research FEATURES.md contains the complete feature dependency DAG that maps directly to the ordering section
- Jack's snapshots include build-mode-specific tests (example_build_server, example_strip_client_code, example_strip_server_code, etc.)

### Established Patterns
- Spec sections follow prior phase conventions
- The spec document grows by appending new sections

### Integration Points
- Entry strategies affect Segment Extraction (Phase 1) — cross-reference needed
- Emit modes affect QRL Wrapping (Phase 1), JSX Transform (Phase 2), and Signal Optimization (Phase 2) — the Mode × CONV table captures this
- Const replacement feeds into DCE — ordering dependency
- Pipeline DAG references all CONVs from Phases 1-3

</code_context>

<specifics>
## Specific Ideas

- The Hoist strategy's `.s()` registration is the most complex entry strategy behavior and deserves extra attention
- The Mode × CONV cross-reference table should be comprehensive — it's the single place an implementer looks to understand mode differences
- Pipeline DAG should show the full 20-step sequence from parse.rs, not just CONV dependencies
- Const replacement → DCE is the most important ordering dependency to document clearly

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 03-build-modes-remaining-transforms-specification*
*Context gathered: 2026-04-01*
