# Phase 1: Core Pipeline Specification - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Write the behavioral specification sections for the core QRL extraction pipeline: dollar detection (CONV-01), QRL wrapping (CONV-02), capture analysis (CONV-03), segment extraction (CONV-05), import rewriting (CONV-12), and supporting infrastructure (GlobalCollect, variable migration, hash generation, path resolution, source maps). This produces sections of a single comprehensive markdown spec document — not code.

</domain>

<decisions>
## Implementation Decisions

### Spec Document Structure
- **D-01:** Document organized in pipeline execution order (parse → collect → pre-transforms → core transform → emit), not CONV numbering. Reader follows the data flow as the optimizer processes it.
- **D-02:** Mermaid flowchart diagram at the top of the spec showing the full transformation pipeline with data flow between stages.
- **D-03:** Cross-referencing approach is Claude's discretion — use whatever combination of inline references and dependency tables best serves readability.
- **D-04:** Each CONV section structured as: behavioral rules (when it triggers, what it does, edge cases) followed by 2-3 input/output examples. Enough detail to implement from without SWC source.

### Source Material Strategy
- **D-05:** SWC v2 optimizer is the source of truth for behavioral correctness. When SWC and Jack's OXC implementation disagree, SWC wins. Jack's accepted deviations are noted but not adopted as target behavior.
- **D-06:** Each spec section includes SWC source file references for traceability (e.g., "Source: transform.rs:298-350"). These serve as audit trail, not as implementation guidance.
- **D-07:** Scott's earlier OXC conversion (`/Users/scottweaver/Projects/qwik-optimizer`) is used to provide OXC pattern examples in migration notes — e.g., how Traverse replaces Fold, how arena allocation works in practice.
- **D-08:** AST JSON dumps from Jack's snapshots are included at Claude's discretion — only where they add clarity (e.g., showing what structurally changed in capture analysis), not as default for every example.

### Capture Analysis
- **D-09:** Capture taxonomy presented as a Mermaid decision tree flowchart showing the classification logic, followed by a table with each of the 8 categories, its rule, and examples.
- **D-10:** All 16 of Jack's known capture edge cases documented as explicit named spec test cases (e.g., "CAPTURE-EDGE-01: Loop variable in for-of"). Implementation must handle all 16.
- **D-11:** Self-import reclassification (module-level declarations referenced by segments) documented at Claude's discretion — either as a dedicated subsection or as an expanded table row, based on complexity.
- **D-12:** Variable migration is a separate spec section that cross-references capture analysis for dependency graph data. Not inlined as a capture subsection.

### Example Format
- **D-13:** Examples show input source code, then each output module (root + all segments) as code blocks. This is the natural unit matching how the optimizer works.
- **D-14:** 2-3 examples per CONV section: one basic case, one with captures/nesting, one edge case. Covers behavioral range without bloat.
- **D-15:** Segment metadata (SegmentAnalysis JSON) included at Claude's discretion — wherever it adds value to understanding the behavioral contract.
- **D-16:** Examples named descriptively with Jack's snapshot name in parentheses for traceability: "Basic Dollar Extraction (example_6)".

### Claude's Discretion
- Cross-referencing style (inline vs dependency table vs both) — D-03
- AST JSON dump inclusion per example — D-08
- Self-import reclassification presentation format — D-11
- Segment metadata inclusion per example — D-15

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### SWC Source (Source of Truth)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` — Main QwikTransform visitor, dollar detection, capture analysis, QRL wrapping (~5,157 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/code_move.rs` — Segment module creation, import resolution (~1,521 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/collector.rs` — GlobalCollect: import/export/root-declaration analysis (~528 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/dependency_analysis.rs` — Variable migration dependency mapping (~587 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/parse.rs` — Pipeline orchestration, 20-step sequence (~1,798 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/rename_imports.rs` — Legacy import path renaming
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/lib.rs` — Public API types (TransformModulesOptions, TransformOutput)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/test.rs` — 50+ snapshot tests showing expected behavior (~7,287 LOC)

### Jack's OXC Implementation (Reference)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/transform.rs` — OXC Traverse implementation (~2,235 LOC)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/collector.rs` — OXC global collection (~1,808 LOC)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/code_move.rs` — OXC segment construction (~539 LOC)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/swc-snapshots/` — SWC reference snapshots (162 .snap files)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/oxc-snapshots/` — OXC output snapshots (162 .snap files)

### Scott's OXC Conversion (OXC Pattern Reference)
- `/Users/scottweaver/Projects/qwik-optimizer/optimizer/src/transform.rs` — OXC Traverse patterns, component identity (~1,186 LOC)
- `/Users/scottweaver/Projects/qwik-optimizer/optimizer/src/component/` — QRL, Id, Segment structures
- `/Users/scottweaver/Projects/qwik-optimizer/optimizer/src/import_clean_up.rs` — Import rewriting patterns

### Research
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/.planning/research/FEATURES.md` — All 14 CONVs with dependencies
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/.planning/research/ARCHITECTURE.md` — Pipeline architecture, SWC vs OXC patterns
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/.planning/research/PITFALLS.md` — 15 domain pitfalls including capture analysis bugs

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Jack's 162 snapshot pairs (swc-snapshots/ and oxc-snapshots/) provide verified input/output pairs for every spec example
- Research FEATURES.md contains the complete feature dependency DAG that maps to the spec's pipeline ordering diagram
- Research PITFALLS.md §1 (Capture Analysis) contains the 8-category taxonomy with specific deviation counts from Jack's Phase 24

### Established Patterns
- The spec document will be a single markdown file in the `specification/` directory
- Mermaid diagrams for pipeline flow and capture analysis decision tree
- Input/output example format: source code blocks with descriptive + snapshot names

### Integration Points
- The spec document sections written in Phase 1 will be extended by Phases 2-4
- Phase 2 (JSX/Props) adds sections after core transforms
- Phase 3 (Build Modes) adds strategy/mode sections
- Phase 4 (API/Bindings) adds public contract and cross-cutting sections

</code_context>

<specifics>
## Specific Ideas

- Capture analysis taxonomy must include all 16 edge cases as named spec test cases (CAPTURE-EDGE-01 through CAPTURE-EDGE-16)
- Self-import reclassification fixed 46 of Jack's 293 runtime deviations — it deserves prominent treatment
- Pipeline diagram should show the 20-step sequence from parse.rs with grouping into logical stages
- Each CONV section needs SWC source file references for traceability back to the reference implementation

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 01-core-pipeline-specification*
*Context gathered: 2026-04-01*
