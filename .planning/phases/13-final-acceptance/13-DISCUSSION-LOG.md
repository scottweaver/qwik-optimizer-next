# Phase 13: Final Acceptance - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md -- this log preserves the alternatives considered.

**Date:** 2026-04-06
**Phase:** 13-final-acceptance
**Areas discussed:** Acceptance threshold, Straggler triage strategy, Verification approach
**Mode:** auto (all decisions auto-selected)

---

## Acceptance Threshold

| Option | Description | Selected |
|--------|-------------|----------|
| Exact match required | All 201 fixtures must fully match on all 3 dimensions | ✓ |
| Allow cosmetic exceptions | Accept minor formatting differences as passing | |

**User's choice:** Exact match required (auto-selected recommended default)
**Notes:** The milestone goal ACC-01 explicitly states 201/201 full match. No room for exceptions.

---

## Straggler Triage Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Diagnose by category, batch fix | Group mismatches by root cause, fix largest categories first | ✓ |
| Fix per-fixture | Address each mismatched fixture individually | |
| Fix by complexity | Start with easiest fixtures, build momentum | |

**User's choice:** Diagnose by category, batch fix (auto-selected recommended default)
**Notes:** 122 root mismatches likely share common root causes. Grouping maximizes impact per fix.

---

## Verification Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Use existing parity_report | Run parity_report test as acceptance gate | ✓ |
| Build new acceptance suite | Create dedicated acceptance test infrastructure | |

**User's choice:** Use existing parity_report (auto-selected recommended default)
**Notes:** parity_report already checks all three dimensions. No new infrastructure needed.

---

## Claude's Discretion

- Fix ordering and plan granularity
- Diagnostic analysis depth

## Deferred Ideas

None
