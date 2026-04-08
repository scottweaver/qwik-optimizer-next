# Phase 11: Root Module Code Generation - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-06
**Phase:** 11-root-module-code-generation
**Areas discussed:** Mismatch Categorization Strategy, Import Ordering Approach, Codegen Formatting Parity, Variable Declaration Handling
**Mode:** Auto (all recommended defaults selected)

---

## Mismatch Categorization Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| Systematic diff analysis first | Categorize all mismatches by type, then fix by category | ✓ |
| Fix by fixture complexity | Start with simplest fixtures, work up to complex | |
| Fix by category blindly | Pick a category (imports), fix all, move to next | |

**User's choice:** [auto] Systematic diff analysis first (recommended default)
**Notes:** Gives planner clear categories for targeted plans. 159 root-only + 14 combined = 173 total mismatches.

---

## Import Ordering Approach

| Option | Description | Selected |
|--------|-------------|----------|
| Holistic rewrite | Rewrite import insertion logic to match SWC ordering | ✓ |
| Patch specific bugs | Fix individual ordering issues as encountered | |
| Normalize in tests | Accept different ordering, normalize in comparison | |

**User's choice:** [auto] Holistic rewrite (recommended default)
**Notes:** Current exit_program inserts at position 0 in reverse. SWC has specific ordering semantics. Patching would be fragile.

---

## Codegen Formatting Parity

| Option | Description | Selected |
|--------|-------------|----------|
| Fix codegen to match | Fix output to match SWC exactly; normalize only cosmetic differences | ✓ |
| Normalize more in tests | Extend normalize() to accept more formatting variations | |
| Custom codegen | Replace OXC Codegen with custom emitter for full control | |

**User's choice:** [auto] Fix codegen to match (recommended default)
**Notes:** OXC Codegen already produces double-quoted strings matching SWC. Focus on statement ordering, expression structure, whitespace between sections.

---

## Variable Declaration Handling

| Option | Description | Selected |
|--------|-------------|----------|
| Fix using existing dependency_analysis | Leverage existing infrastructure for variable migration | ✓ |
| Rewrite from SWC reference | Port SWC's approach directly | |
| Manual per-fixture fixes | Fix declarations fixture by fixture | |

**User's choice:** [auto] Fix using existing dependency_analysis (recommended default)
**Notes:** dependency_analysis.rs already has infrastructure for analyzing root module variable usage.

---

## Claude's Discretion

- Implementation order across categories
- Diff analysis tool/script approach
- Number of plans
- Combined root+segment fixture handling

## Deferred Ideas

None — all discussion stayed within phase scope.
