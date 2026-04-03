# Phase 7: Spec Gap Closure — Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-03
**Phase:** 07-spec-gap-closure
**Areas discussed:** Spec section approach, Verification scope, Example selection
**Mode:** --auto (all decisions auto-selected)

---

## Spec Section Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Extract from SWC source | Write CONV-01/02 by analyzing SWC transform.rs per Phase 1 D-05/D-06 conventions | auto |
| Write from OXC implementation | Derive spec from the existing OXC code | |
| Hybrid | Use both SWC and OXC as sources | |

**User's choice:** [auto] Extract from SWC source (recommended default — SWC is source of truth per D-05)
**Notes:** Phase 1 context already established SWC as behavioral truth. CONV-01/02 sections must describe SWC behavior.

---

## Verification Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Verify and patch | Check existing CONV-09/10/11 sections, fix gaps, update checkboxes | auto |
| Full rewrite | Rewrite CONV-09/10/11 sections from scratch | |
| Checkbox only | Just update REQUIREMENTS.md checkboxes without verification | |

**User's choice:** [auto] Verify and patch (recommended default — sections likely exist, just need verification)
**Notes:** Phase 3 completed all plans with summaries, suggesting sections were written. No VERIFICATION.md exists for Phase 3 to confirm.

---

## Example Selection

| Option | Description | Selected |
|--------|-------------|----------|
| Multi-CONV fixtures | Prioritize fixtures exercising multiple CONVs for coverage efficiency | auto |
| One per CONV | Select exactly one representative fixture per CONV | |
| Curated showcase | Hand-pick the most illustrative fixtures regardless of coverage | |

**User's choice:** [auto] Multi-CONV fixtures (recommended default — maximizes coverage with minimal examples)
**Notes:** Need 20+ examples covering all 14 CONVs. Multi-CONV fixtures reduce total count needed.

---

## Claude's Discretion

- Specific fixture selection for representative examples
- Whether CONV-09/10/11 need content patches or just checkbox updates
- Cross-reference style within new sections

## Deferred Ideas

None
