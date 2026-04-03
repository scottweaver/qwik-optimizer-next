# Phase 8: Implementation Gap Closure - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-03
**Phase:** 08-implementation-gap-closure
**Areas discussed:** Signal optimization wiring, PURE annotation injection, SWC parity strategy, Spec example test activation
**Mode:** Auto (--auto flag, all defaults selected)

---

## Signal Optimization Wiring (CONV-07)

| Option | Description | Selected |
|--------|-------------|----------|
| During JSX prop classification | Call convert_inlined_fn from jsx_transform.rs prop analysis path, matching SWC's flow | ✓ |
| Post-processing pass | Separate pass after JSX transform to find signal-eligible expressions | |

**User's choice:** [auto] During JSX prop classification (recommended default)
**Notes:** Matches SWC architecture where signal optimization is part of prop analysis

---

## PURE Annotation Injection (CONV-08)

| Option | Description | Selected |
|--------|-------------|----------|
| String-based injection | Insert /*#__PURE__*/ during code assembly, matching Phase 5 string assembly pattern | ✓ |
| AST comment attachment | Use OXC's leading comment API to attach PURE comment to AST nodes | |

**User's choice:** [auto] String-based injection (recommended default)
**Notes:** Consistent with established string assembly then re-parse pattern from Phase 5

---

## SWC Parity Improvement Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Fix by most common pattern divergence | Categorize 200 mismatches, fix highest-impact patterns first | ✓ |
| Fix by CONV type | Work through CONV-01 to CONV-14 systematically | |
| Fix by fixture complexity | Start with simplest fixtures, work up to complex ones | |

**User's choice:** [auto] Fix by most common pattern divergence (recommended default)
**Notes:** Maximizes parity percentage per fix

---

## Spec Example Test Activation

| Option | Description | Selected |
|--------|-------------|----------|
| Un-ignore all at once | Remove all #[ignore] annotations, document failures | ✓ |
| Incremental by CONV | Un-ignore one CONV group at a time | |

**User's choice:** [auto] Un-ignore all at once (recommended default)
**Notes:** Gives clear picture of remaining gaps

---

## Claude's Discretion

- Order of parity fixes within the triage strategy
- Whether to batch parity fixes by pattern or by fixture
- Internal refactoring needed to support CONV-07/08 wiring
- How to categorize and report spec_examples.rs test failures

## Deferred Ideas

None
