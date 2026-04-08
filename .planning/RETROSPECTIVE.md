# Project Retrospective

*A living document updated after each milestone. Lessons feed forward into future planning.*

## Milestone: v0.1.0 — Qwik Optimizer Spec & OXC Implementation

**Shipped:** 2026-04-03
**Phases:** 9 | **Plans:** 35 | **Timeline:** 4 days (2026-03-31 → 2026-04-03)

### What Was Built
- Comprehensive behavioral specification covering all 14 CONV transformations in a single document
- Feature-complete OXC implementation with 444 tests (211 snapshots + 233 unit tests)
- NAPI (Node.js) and WASM (browser) bindings with same JSON interface as SWC version
- 57/201 SWC root module parity with gap analysis for remaining mismatches

### What Worked
- **Spec-first approach**: Writing the behavioral spec before implementing forced deep understanding of each transformation. Implementation phases moved faster because the spec resolved ambiguities upfront.
- **Gap-closure phases**: The milestone audit after Phase 6 identified concrete gaps. Creating focused gap-closure phases (7-9) was more effective than reopening earlier phases.
- **Snapshot testing with insta**: 211 fixture snapshots caught regressions immediately during implementation. `cargo insta review` workflow was efficient for validating bulk changes.
- **Wave-based plan execution**: Parallelizing independent plans within phases kept throughput high.

### What Was Inefficient
- **Phase 1/2 plan tracking**: Plans 01-02 and 01-04 were never formally executed — their content was written during other phases but the plan infrastructure wasn't updated. Phase 2 had no plans until retroactive creation. This caused confusion during the milestone audit.
- **SWC parity measurement came late**: Measuring parity at Phase 8 (0.5%) after most implementation was done meant structural decisions (import ordering, comment placement) were already baked in. Earlier measurement would have aligned these patterns sooner.
- **Audit predated gap closure**: The milestone audit was created before gap-closure phases, leaving a stale `gaps_found` status that needed manual reasoning to override.

### Patterns Established
- **Two-phase OXC pattern**: Analyze (Visit/VisitMut) then emit (Traverse with enter/exit hooks). Semantic analysis before mutation prevents scope invalidation.
- **String assembly + re-parse**: For segment codegen, assemble import/body strings then re-parse for normalized output rather than attempting AST cloning.
- **Span-based extraction**: Record source spans during analysis, slice original source at those spans for segment construction.
- **Descriptive symbol naming**: 6 Traverse hooks build human-readable names (e.g., `Header_component_J4uyIhaBNR4`) for SWC-compatible hashes.

### Key Lessons
1. **Measure parity early and continuously** — define the comparison methodology in Phase 5, not Phase 8
2. **Formally execute all plans or remove them** — phantom unexecuted plans create noise in audits
3. **Gap-closure is a first-class workflow** — plan for a cleanup phase after initial implementation rather than expecting perfection
4. **OXC API evolves fast** — version 0.123 changed Atom→Str, required adapting patterns from Jack's 0.113 code

### Cost Observations
- Model mix: Primarily Opus for implementation, Sonnet for research/planning
- Sessions: ~15 across 4 days
- Notable: Bulk snapshot updates (Phase 8, 189 files) were the highest single-plan cost

---

## Milestone: v0.2.0 — Full SWC Parity

**Shipped:** 2026-04-08
**Phases:** 4 | **Plans:** 18 | **Timeline:** 5 days (2026-04-03 → 2026-04-07)

### What Was Built
- Segment extraction parity: 125/201 → 195/201 (97%) via JSX dollar-attr extraction and edge case fixes
- Diagnostics parity: 197/201 → 201/201 (100%) via C02/C03/C05 fixes
- Root module code generation rewrite: import assembly, dead code elimination, PURE annotations, quote preservation
- 11 gap-closure plans in Phase 13 bringing full parity from 28/201 (14%) to 200/201 (99.5%)
- Universal QRL hoisting with unified _noopQrl/.s()/.w() code path

### What Worked
- **Parity diff report as diagnostic tool**: The categorized diff report (Phase 13-01) provided a clear roadmap for all remaining fixes, enabling systematic triage of 121 root mismatches by category.
- **Iterative gap closure**: Phase 13's 11 plans each targeted specific mismatch categories. Each plan moved the parity number visibly (28→79→89→95→99→107→200), maintaining momentum.
- **Post-processing patterns**: String-level transformations (PURE annotations, arrow spacing, quote restoration) were more reliable than fighting OXC codegen limitations.
- **Pre-registration for ordering**: Matching SWC's Fold semantics for collision counters required pre-registering segment names at enter time — a non-obvious but critical insight.

### What Was Inefficient
- **Late discovery of structural patterns**: Some patterns (const sorting, JSX key prefixes, arrow spacing) could have been caught earlier with a more systematic diff analysis at the start of Phase 13.
- **Multiple passes over same fixtures**: Each gap-closure plan re-ran all 201 tests. A more targeted approach could have reduced iteration cycles.
- **1 remaining failure**: example_1's spurious `$` import was not caught during the milestone — a more thorough review of the final diff would have caught it.

### Patterns Established
- **Parity diff report**: Run a categorized diff analysis at the start of each parity phase to map the full problem space before planning fixes.
- **Post-processing string transforms**: When OXC codegen doesn't support a formatting option, apply it as a text-level post-processing step.
- **Pre-registration for SWC ordering semantics**: SWC's Fold trait processes nodes in definition order; OXC Traverse processes in AST order. Bridge the gap by pre-registering names at enter time.

### Key Lessons
1. **Categorize before fixing** — the diff report was the highest-leverage artifact in the milestone, turning 121 unknown mismatches into 8 named categories
2. **OXC codegen has formatting gaps** — arrow spacing, PURE comments, and quote styles all required post-processing workarounds
3. **SWC Fold vs OXC Traverse ordering matters** — collision counter ordering, const sorting, and import assembly all depended on matching SWC's specific traversal order
4. **String-level transforms are pragmatic** — fighting the AST for formatting wins is less reliable than post-processing the codegen output

### Cost Observations
- Model mix: Primarily Opus for all phases
- Sessions: ~10 across 5 days
- Notable: Phase 13 (11 plans) consumed the bulk of effort; earlier phases (10-12) were efficient

---

## Cross-Milestone Trends

### Process Evolution

| Milestone | Timeline | Phases | Key Change |
|-----------|----------|--------|------------|
| v0.1.0 | 4 days | 9 | Initial milestone — established spec-first + gap-closure pattern |
| v0.2.0 | 5 days | 4 | Parity-focused — diff report + iterative gap closure pattern |

### Cumulative Quality

| Milestone | Tests | SWC Parity | Requirements |
|-----------|-------|------------|--------------|
| v0.1.0 | 444 | 57/201 (28%) | 35/35 (100%) |
| v0.2.0 | 200+ | 200/201 (99.5%) | 13/13 (100%) |

### Top Lessons (Verified Across Milestones)

1. **Categorize problems before fixing them** — v0.1.0's gap analysis and v0.2.0's diff report were both the highest-leverage artifacts in their milestones
2. **Gap-closure is a first-class workflow** — both milestones needed dedicated cleanup phases after initial implementation
3. **Post-processing beats fighting the framework** — when the AST tool doesn't support a pattern, string-level transforms are more reliable than workarounds
