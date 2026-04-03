# Phase 9: Metadata & Verification Cleanup - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-03
**Phase:** 09-metadata-verification-cleanup
**Areas discussed:** Parallel feature fate, Verification depth, Checkbox audit scope, IMPL-03/04 status

---

## Parallel Feature Fate

| Option | Description | Selected |
|--------|-------------|----------|
| Remove the dead flag (Recommended) | Delete the parallel feature and rayon dep from Cargo.toml. It's dead code — PERF-02 is explicitly deferred to v2. | ✓ |
| Wire rayon into transform_modules | Actually implement parallel module processing. Adds real value but is non-trivial and scoped for v2. | |
| Keep but mark as stub | Leave the feature flag, add compile_error! or todo! to make it clear it's not functional. | |

**User's choice:** Remove the dead flag
**Notes:** Clean and honest approach. PERF-02 is v2 scope.

## Verification Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Criteria checklist (Recommended) | Read success criteria from ROADMAP.md, confirm each met by checking spec/code/tests, write concise pass/fail VERIFICATION.md. | ✓ |
| Full re-validation | Re-run tests, re-measure parity, re-read spec sections, produce detailed evidence. | |
| Minimal stub | Create VERIFICATION.md files that say "Verified — see phase execution summaries." | |

**User's choice:** Criteria checklist
**Notes:** Efficient and meaningful — sufficient for retroactive verification.

## Checkbox Audit Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Full reconciliation (Recommended) | Update REQUIREMENTS.md checkboxes, fix traceability table, update ROADMAP.md progress table, fix stale phase statuses. | ✓ |
| Requirements only | Just update REQUIREMENTS.md checkboxes and traceability table. Leave ROADMAP.md as-is. | |
| You decide | Claude picks appropriate scope based on what's actually stale. | |

**User's choice:** Full reconciliation
**Notes:** Update both REQUIREMENTS.md and ROADMAP.md comprehensively.

## IMPL-03/04 Status

| Option | Description | Selected |
|--------|-------------|----------|
| Investigate actual state (Recommended) | Check code for what entry strategies and emit modes work. Update requirements based on evidence. | ✓ |
| Mark as incomplete | Leave IMPL-03/04 as pending. Assume Phase 6 plans 01/02 are unfinished. | |
| Close as sufficient | Core strategies/modes work. Mark complete with caveats. | |

**User's choice:** Investigate actual state
**Notes:** Evidence-based — determine what actually works before updating checkboxes.
