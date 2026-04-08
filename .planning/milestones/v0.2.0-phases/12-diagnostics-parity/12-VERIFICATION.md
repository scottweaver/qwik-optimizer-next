---
phase: 12-diagnostics-parity
verified: 2026-04-06T18:30:00Z
status: passed
score: 3/3 must-haves verified
re_verification: false
---

# Phase 12: Diagnostics Parity Verification Report

**Phase Goal:** Error diagnostics match SWC for all 201 fixtures -- same errors reported for invalid references and missing custom inlined functions
**Verified:** 2026-04-06
**Status:** passed
**Re-verification:** No -- initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | OXC does not emit C02 FunctionReference for identifiers that are export symbols | VERIFIED | `classify_captures` in transform.rs:907-911 gates C02 with `!collect.has_export_symbol(name) && !collect.root.contains_key(name) && !self_imports.contains(...)`. 8 fixture snapshots updated in commit a195470. |
| 2 | OXC emits C05 MissingQrlImplementation when a locally-defined $-suffixed function is called but the corresponding Qrl export is missing | VERIFIED | C05 implemented at transform.rs:2057-2088. Uses `marker_fn_sources.contains_key` to distinguish local vs imported, calls `dollar_to_qrl_name`, emits diagnostic and returns early. Updated in commit dc99dcb. |
| 3 | All 201 fixtures produce the same diagnostic presence/absence as SWC (201/201 diagnostics match) | VERIFIED | `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture` reports `Diagnostics match: 201/201` |

**Score:** 3/3 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/qwik-optimizer-oxc/src/transform.rs` | C02 export-symbol gate and C05 diagnostic implementation | VERIFIED | Contains `has_export_symbol` at line 909, `root.contains_key` at line 910, `self_imports.contains` at line 911 for C02 gate. Contains `C05` code at line 2072, message pattern at line 2075, early return at line 2086. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `crates/qwik-optimizer-oxc/src/transform.rs` | `crates/qwik-optimizer-oxc/src/collector.rs` | `has_export_symbol()` call in `classify_captures` | WIRED | `collect.has_export_symbol(name)` called at transform.rs:909; `has_export_symbol` defined at collector.rs:139. |

### Data-Flow Trace (Level 4)

Not applicable -- this phase modifies a transform pass, not a UI component or data-rendering artifact. The data flow is verified by the behavioral spot-check (parity test).

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| 201/201 diagnostic parity | `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture` | `Diagnostics match: 201/201` | PASS |
| `example_missing_custom_inlined_functions` fixture passes | `cargo test -p qwik-optimizer-oxc --test snapshot_tests -- "example_missing_custom_inlined_functions"` | `1 passed; 0 failed` | PASS |
| `example_capturing_fn_class` still passes (C02 fires for legitimate case) | `cargo test -p qwik-optimizer-oxc --test snapshot_tests -- "example_capturing_fn_class"` | `1 passed; 0 failed` | PASS |
| Full test suite shows no regressions | `cargo test -p qwik-optimizer-oxc` | `264 passed; 0 failed` (unit) + `223 passed; 0 failed` (snapshot) + `24 passed; 0 failed` (spec) = 511 total, 0 failures | PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|-------------|-------------|--------|----------|
| DIAG-01 | 12-01-PLAN.md | Error diagnostics match SWC for invalid references and missing custom inlined functions | SATISFIED | C02 gate suppresses 8 false positives; C05 fires for `example_missing_custom_inlined_functions`. Diagnostics match 201/201. |
| DIAG-02 | 12-01-PLAN.md | Diagnostic presence/absence matches SWC for all 201 fixtures | SATISFIED | `parity_report` test confirms `Diagnostics match: 201/201`. |

**Orphaned requirements check:** REQUIREMENTS.md maps only DIAG-01 and DIAG-02 to Phase 12. No orphaned requirements.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/qwik-optimizer-oxc/src/transform.rs` | 655-659 | `ensure_export` is a no-op placeholder | Info | Pre-existing from before Phase 12; called in lib.rs:489 but does nothing. Not related to diagnostics parity. No data flows to user-visible output through this path in Phase 12 scope. |

The `ensure_export` placeholder is pre-existing and does not affect the phase 12 goal. The diagnostics subsystem does not depend on it.

### Human Verification Required

None. The 201/201 parity metric is machine-verifiable and confirmed by the parity report test. All diagnostic behaviors are covered by the automated snapshot test suite.

### Deviations from Plan (Documented in SUMMARY)

Three auto-fixed bugs were discovered and resolved during implementation:

1. **C03 false positives from module-level declarations** -- Fixed by moving `classify_captures` before the C03 check (transform.rs:2114-2124). This was not in the original plan tasks but was necessary to achieve 201/201.
2. **C03 false positives for identifier-ref first arguments** -- Fixed by suppressing C03 when the first argument is a simple `Identifier` (transform.rs:2140-2144). Matches SWC's const-inlining behavior.
3. **C05 aliased import false positives** -- Fixed by using `marker_fn_sources.contains_key` instead of `collect.imports.contains_key`, handling aliased imports correctly (e.g., `$ as onRender`).

All deviations were bug fixes that kept the implementation within scope. No scope creep.

### Gaps Summary

No gaps. All three observable truths verified. Both requirements satisfied. All 511 tests pass. The phase goal is fully achieved.

---

_Verified: 2026-04-06_
_Verifier: Claude (gsd-verifier)_
