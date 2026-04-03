# Phase 7: Spec Gap Closure — Missing CONV Sections - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Close specification gaps identified by the v0.1.0 milestone audit: write the missing Dollar Detection (CONV-01) and QRL Wrapping (CONV-02) spec sections, verify that existing CONV-09/10/11 sections are complete and checkboxes can be updated, add 20+ representative input/output examples covering all 14 CONVs, and update all stale requirement checkboxes. This is spec document work only — no Rust code changes.

</domain>

<decisions>
## Implementation Decisions

### Spec Writing Approach
- **D-01:** Follow all conventions established in Phase 1 context (D-01 through D-16): pipeline execution order, Mermaid diagrams, 2-3 examples per CONV, SWC source references for traceability.
- **D-02:** CONV-01 and CONV-02 sections extracted from SWC `transform.rs` source (per Phase 1 D-05/D-06). SWC is source of truth; Jack's accepted deviations noted but not adopted.
- **D-03:** CONV-01 section covers: imported marker detection (`$`-suffixed from `@qwik.dev/core`), local marker detection, `marker_functions` HashMap, `convert_qrl_word` callee conversion, special cases (`sync$`, `component$`, bare `$`), non-marker exclusion rule.
- **D-04:** CONV-02 section covers: three QRL creation paths (`create_qrl` for Segment/Hook/Single/Component/Smart, `create_inline_qrl` for Inline/Hoist, `create_noop_qrl` for stripped callbacks), dev mode variants (`qrlDEV`/`inlinedQrlDEV`), captures emission, PURE annotation rule.

### Verification Scope
- **D-05:** For CONV-09 (DCE), CONV-10 (Const Replacement), CONV-11 (Code Stripping): verify existing spec content is present and complete, patch any gaps found, then update REQUIREMENTS.md checkboxes. Do not rewrite from scratch.
- **D-06:** For SPEC-29 (representative examples): add 20+ examples covering all 14 CONVs, sourced from the 201 SWC fixtures. Prioritize fixtures that exercise multiple CONVs to minimize example count while maximizing coverage.

### Claude's Discretion
- Selection of specific SWC fixtures for the 20+ representative examples
- Whether CONV-09/10/11 sections need patches or just checkbox updates
- Cross-reference style within new sections (following Phase 1 D-03)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Spec Document (Target)
- `specification/qwik-optimizer-spec.md` — The single comprehensive spec document. New CONV-01/02 sections added here. Existing CONV-09/10/11 sections verified here.

### SWC Source (Source of Truth for CONV-01/02)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` — Dollar detection logic (lines ~179-202), QRL wrapping (`create_qrl` at ~1888-2062)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/code_move.rs` — Segment creation paths referenced by CONV-02

### Audit (Gap Definitions)
- `.planning/v0.1.0-MILESTONE-AUDIT.md` — Defines exactly which requirements are unsatisfied and why
- `.planning/phases/01-core-pipeline-specification/01-VERIFICATION.md` — Phase 1 verification confirming SPEC-01/02 gaps

### Prior Phase Context (Conventions)
- `.planning/phases/01-core-pipeline-specification/01-CONTEXT.md` — Decisions D-01 through D-16 establishing all spec writing conventions

### SWC Fixtures (Example Source)
- `crates/qwik-optimizer-oxc/tests/swc_expected/*.snap` — 201 SWC golden snapshots as source for representative examples

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- The spec document already has ~2000 lines of CONV sections (03-08, 12-14) that establish the pattern for new sections
- Phase 1 VERIFICATION.md precisely identifies what's missing from CONV-01/02

### Established Patterns
- Each CONV section: behavioral rules → edge cases → 2-3 input/output examples → SWC source references
- Examples use descriptive names with Jack's snapshot name in parentheses for traceability

### Integration Points
- New CONV-01 section inserts after pipeline overview, before CONV-03 (Capture Analysis)
- New CONV-02 section inserts after CONV-01, before CONV-03
- Representative examples appendix added at end of spec document

</code_context>

<specifics>
## Specific Ideas

- Phase 1 VERIFICATION.md gives exact line numbers and missing content descriptions for CONV-01/02 — use as a checklist
- The 01-02-PLAN.md (unexecuted) may contain useful task breakdowns for the spec sections

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 07-spec-gap-closure*
*Context gathered: 2026-04-03*
