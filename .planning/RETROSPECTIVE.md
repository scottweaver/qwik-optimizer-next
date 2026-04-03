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

## Cross-Milestone Trends

### Process Evolution

| Milestone | Timeline | Phases | Key Change |
|-----------|----------|--------|------------|
| v0.1.0 | 4 days | 9 | Initial milestone — established spec-first + gap-closure pattern |

### Cumulative Quality

| Milestone | Tests | SWC Parity | Requirements |
|-----------|-------|------------|--------------|
| v0.1.0 | 444 | 57/201 (28%) | 35/35 (100%) |

### Top Lessons (Verified Across Milestones)

1. Measure output parity early — structural alignment decisions compound
2. Gap-closure phases are cheaper than reopening completed phases
