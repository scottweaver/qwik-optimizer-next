# Phase 9: Metadata & Verification Cleanup - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Update stale requirement checkboxes in REQUIREMENTS.md, write missing VERIFICATION.md reports for phases 3-7, fix stale ROADMAP.md progress statuses, investigate actual IMPL-03/04 completion status, and remove the dead `parallel` feature flag. This is cleanup/documentation work — no new features, no spec changes, no transform code changes.

</domain>

<decisions>
## Implementation Decisions

### Parallel Feature Disposition
- **D-52:** Remove the `parallel` feature flag and `rayon` optional dependency from `crates/qwik-optimizer-oxc/Cargo.toml`. The feature compiles but has zero usage in any source file. PERF-02 (parallel processing) is explicitly deferred to v2 in REQUIREMENTS.md — re-add cleanly when that work begins.

### Verification Report Approach
- **D-53:** Write VERIFICATION.md for phases 3, 4, 5, 6, and 7 using a criteria checklist approach: read each phase's success criteria from ROADMAP.md, confirm each criterion is met by checking the spec/code/tests, and write a concise pass/fail report. No full re-validation or re-running of test suites needed — these are retroactive confirmations.

### Requirement & Roadmap Reconciliation
- **D-54:** Full reconciliation across both REQUIREMENTS.md and ROADMAP.md:
  - Update all 6 pending requirement checkboxes (SPEC-06, SPEC-18, SPEC-19, SPEC-20, IMPL-03, IMPL-04) based on actual completion evidence
  - Fix the traceability table statuses to match checkbox state
  - Update ROADMAP.md progress table: fix plan counts, add completion dates for finished phases, correct stale "In Progress" statuses (phases 1, 5, 6, 8 all have completed work)

### IMPL-03/04 Investigation
- **D-55:** Investigate the actual state of entry strategies (IMPL-03) and emit modes (IMPL-04) by reading the implementation code. Determine which of the 7 strategies and 5 modes actually work vs are stubs. Update requirements based on evidence — if gaps remain, document them honestly rather than marking complete or leaving as blanket "pending."

### Carrying Forward
- D-34: Spec-driven approach, consult references when stuck
- D-36: OXC 0.123.0 pinned

### Claude's Discretion
- Order of operations (verifications first vs checkboxes first vs parallel feature first)
- VERIFICATION.md format details and evidence depth per criterion
- How to structure the IMPL-03/04 investigation (grep-based vs reading module files)
- Whether to update STATE.md progress percentage after cleanup

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Requirements & Roadmap
- `.planning/REQUIREMENTS.md` — All requirement definitions, checkboxes, and traceability table (the primary target of checkbox updates)
- `.planning/ROADMAP.md` — Phase definitions, success criteria (source of truth for verification reports), and progress table (target of status fixes)

### Existing Verifications (Pattern Reference)
- `.planning/phases/01-core-pipeline-specification/01-VERIFICATION.md` — Example of verification report format
- `.planning/phases/02-jsx-props-signal-specification/02-VERIFICATION.md` — Example of verification report format
- `.planning/phases/08-implementation-gap-closure/08-VERIFICATION.md` — Most recent verification, use as template

### Implementation Code (IMPL-03/04 Investigation)
- `crates/qwik-optimizer-oxc/src/code_move.rs` — Entry strategy implementation (EntryPolicy)
- `crates/qwik-optimizer-oxc/src/transform.rs` — Main transform pipeline, emit mode threading
- `crates/qwik-optimizer-oxc/src/lib.rs` — Public API entry point

### Cargo Configuration
- `crates/qwik-optimizer-oxc/Cargo.toml` — parallel feature flag and rayon dep to remove

### Spec Document
- `specification/qwik-optimizer-spec.md` — Authoritative reference for verifying spec requirement completion

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Existing VERIFICATION.md files (phases 1, 2, 8) provide consistent format to follow
- Phase execution summaries in each phase directory document what was actually built

### Established Patterns
- VERIFICATION.md format: phase header, success criteria list, pass/fail per criterion with evidence
- REQUIREMENTS.md traceability table maps requirements to phases and statuses

### Integration Points
- REQUIREMENTS.md checkbox state must match traceability table status column
- ROADMAP.md progress table must reflect actual phase directory contents
- STATE.md progress percentage derives from completed phases/plans

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches. The work is mechanical cleanup with an investigation component for IMPL-03/04.

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope.

</deferred>

---

*Phase: 09-metadata-verification-cleanup*
*Context gathered: 2026-04-03*
