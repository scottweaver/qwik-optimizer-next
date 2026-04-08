---
phase: 13-final-acceptance
verified: 2026-04-07T06:00:00Z
status: gaps_found
score: 0/3 success criteria verified
re_verification:
  previous_status: gaps_found
  previous_score: 2/3
  gaps_closed: []
  gaps_remaining:
    - "Parity report shows 201/201 full match -- actual: 11/201 (down from 107/201 at previous verification)"
  regressions:
    - "Full match count regressed from 107/201 to 11/201 after plans 13-09 through 13-11 (net -96 fixtures)"
    - "Segment count match regressed from 196/201 to 125/201 (net -71 fixtures)"
    - "Diagnostics match regressed from 201/201 to 197/201 (net -4 fixtures)"
    - "Current 11/201 full match is below the v0.1.0 baseline of 79/201"
gaps:
  - truth: "Parity report shows 201/201 full match (root module + segment count + diagnostics all green)"
    status: failed
    reason: "Parity report shows 11/201 full match (5%). This is a regression from 107/201 at the previous verification. Plans 13-09 through 13-11 made architectural changes to QRL hoisting (universal module-scope hoisting for all non-Lib strategies in plan 13-10) that broke 96 previously-matching fixtures. The changes are directionally correct but incomplete -- they implement the right output format but introduced regressions before all fixtures could benefit."
    artifacts:
      - path: "crates/qwik-optimizer-oxc/tests/snapshot_tests.rs"
        issue: "parity_report confirms 11/201 full match; 190 fixtures fail with root, segment, or diagnostics mismatch"
    missing:
      - "Signal optimization (_fnSignal/_wrapProp/_hf*) not integrated into JSX transform -- inlined_fn.rs is implemented but jsx_transform.rs has 0 references to it"
      - "JSX event handler extraction (onClick$, onInput$, etc.) not implemented -- affects ~40+ segment count mismatches"
      - "Non-exported const stripping -- SWC removes wrapperQrl(q_X) const when X is unexported, OXC keeps it (affects ~40+ root module mismatches)"
      - "Unused import removal -- SWC removes imports that become unused after transformation, OXC keeps all non-marker imports (affects ~30+ root module mismatches)"
      - "Symbol name parity -- hash-based naming produces different symbol names than SWC across many fixtures"
      - "Segment count regressed 196/201 to 125/201 -- universal QRL hoisting change broke segment extraction for 71 additional fixtures"
  - truth: "No regressions from v0.1.0 baseline (all previously passing tests still pass)"
    status: failed
    reason: "The v0.1.0 baseline had 79/201 SWC parity. Current state is 11/201 -- below the v0.1.0 baseline. 223/223 OXC insta snapshot tests pass, but these compare OXC output to itself (not to SWC). SWC parity regressed by 96 fixtures from the state at previous verification, and is now below even the v0.1.0 baseline."
    artifacts:
      - path: "crates/qwik-optimizer-oxc/src/transform.rs"
        issue: "Universal QRL hoisting change in plan 13-10 broke 71 segment count fixtures (196->125); total full match dropped 96 fixtures"
    missing:
      - "Fix or revert the universal QRL hoisting change so segment count returns to at least 196/201"
      - "Validate architectural changes against parity_report before committing (treat parity regression as a blocker)"
  - truth: "Any stragglers discovered during final verification are identified, fixed, and re-verified"
    status: failed
    reason: "Plans 09-11 introduced more mismatches than they fixed. 190 fixtures now fail (up from 94 at previous verification). Remaining issues are documented in 13-11-SUMMARY.md (identified) but none were fixed to a passing state. The phase left the codebase with lower SWC parity than when it started."
    artifacts:
      - path: "crates/qwik-optimizer-oxc/tests/snapshot_tests.rs"
        issue: "190 fixtures remain in mismatched state after plans 09-11; parity_report shows Mismatches (190)"
    missing:
      - "Fix regressions introduced by plans 09-11 (segment count: 196->125, full match: 107->11)"
      - "JSX event handler segment extraction (onClick$, onInput$, etc.) -- 40+ fixtures"
      - "Non-exported const stripping -- 40+ fixtures"
      - "Unused import removal -- 30+ fixtures"
      - "Signal optimization integration: wire inlined_fn.rs into jsx_transform.rs"
---

# Phase 13: Final Acceptance Verification Report

**Phase Goal:** Full SWC parity achieved -- 201/201 fixtures match on root module, segment count, and diagnostics
**Verified:** 2026-04-07T06:00:00Z
**Status:** gaps_found
**Re-verification:** Yes -- after gap closure plans 13-09, 13-10, 13-11

## Goal Achievement

### Observable Truths (from ROADMAP.md Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | Parity report shows 201/201 full match (root module + segment count + diagnostics all green) | FAILED | `cargo test parity_report --nocapture` reports: Full match 11/201 (5%), Root module 13/201, Segment count 125/201, Diagnostics 197/201. REGRESSED from 107/201 at previous verification. |
| 2 | No regressions from v0.1.0 baseline (all previously passing tests still pass) | FAILED | v0.1.0 baseline was 79/201 SWC parity. Current state is 11/201 -- below the v0.1.0 baseline. 223/223 OXC insta snapshot tests pass, but these compare OXC to itself (not to SWC expected output). SWC parity regressed 96 fixtures. |
| 3 | Any stragglers discovered during final verification are identified, fixed, and re-verified | FAILED | 190 fixtures now fail (up from 94 at previous verification). Plans 09-11 introduced more mismatches than they fixed. Remaining issues are documented but not resolved. |

**Score:** 0/3 success criteria verified

### Re-verification Summary

| Metric | Previous Verification | Current State | Change |
|--------|----------------------|---------------|--------|
| Full match | 107/201 (53%) | 11/201 (5%) | -96 fixtures |
| Root module match | 107/201 | 13/201 | -94 fixtures |
| Segment count match | 196/201 | 125/201 | -71 fixtures |
| Diagnostics match | 201/201 | 197/201 | -4 fixtures |
| OXC snapshot tests | 224/224 pass | 223/223 pass | (count changed due to test refactor) |

The gap addressed by plans 13-09 through 13-11 was not closed. The plans made architectural changes that are directionally correct but incomplete, introducing regressions in previously-passing fixtures before the new output format was complete enough to compensate.

### Artifacts from Plans 09-11 (Verified as Implemented)

| Artifact | Plan | Status | Details |
|----------|------|--------|---------|
| `transform.rs::auto_exports: Vec<String>` field | 13-09 | VERIFIED | Line 435 in transform.rs |
| `transform.rs::ensure_export` method | 13-09 | VERIFIED | `fn ensure_export` at line 648; called at line 1379 during traversal |
| `emit.rs::collapse_single_prop_objects` | 13-09 | VERIFIED | Function at line 100; called in both emit paths (lines 52, 61, 71) |
| `transform.rs::extra_top_items` universal hoisting | 13-10 | VERIFIED | Lines 1607-1645: all non-Lib strategies push to `extra_top_items` |
| `transform.rs::ref_assignments` + `.s()` body | 13-10 | VERIFIED | Lines 1622-1626: `.s()` call generated via `codegen_first_arg` |
| `transform.rs::codegen_first_arg` | 13-10 | VERIFIED | Function at line 1946 for serializing transformed first argument |
| `lib.rs::post_process_root_module` | 13-11 | VERIFIED | PURE annotation fixup, arrow spacing, const ordering, import ordering |
| `snapshot_tests.rs::parity_report` | All | VERIFIED | Function at line 1057; test passes; reports 11/201 |
| `inlined_fn.rs::convert_inlined_fn` (_fnSignal) | 13-09/10 | ORPHANED | Implemented in inlined_fn.rs; NOT imported by jsx_transform.rs (0 usages) |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| `inlined_fn.rs::convert_inlined_fn` | `jsx_transform.rs` JSX prop processing | Called during JSX attribute value processing | NOT_WIRED | `jsx_transform.rs` has 0 references to `inlined_fn` module -- signal optimization is orphaned |
| `transform.rs::ensure_export` | `lib.rs` variable migration | Called in classify_captures processing | WIRED | Called at line 1379 for root-level bindings in self_imports |
| `transform.rs::extra_top_items` | `exit_program` root module emission | All non-Lib strategies emit extra_top_items | WIRED | Lines 1858-1876 in exit_program |
| `props_destructuring.rs` | `transform.rs` Stage 4b | `transform_props_destructuring` called | WIRED | Called at line 2155 in main transform pipeline |

### Data-Flow Trace (Level 4)

Not applicable -- phase artifacts are transform pipeline functions and test utilities, not components rendering dynamic data.

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| parity_report produces accurate counts | `cargo test parity_report --nocapture` | Full match: 11/201 (5%), Root: 13/201, Segments: 125/201, Diag: 197/201 | PASS (test passes; result confirms severe gap) |
| Full OXC snapshot test suite has no failures | `cargo test -p qwik-optimizer-oxc --test snapshot_tests` | ok. 223 passed; 0 failed | PASS (snapshots compare OXC to itself; does not reflect SWC parity) |
| Signal optimization wired into JSX transform | `grep inlined_fn jsx_transform.rs` | No results (0 usages) | FAIL (_fnSignal implementation exists but is not called from jsx_transform.rs) |

### Requirements Coverage

| Requirement | Plan(s) | Description | Status | Evidence |
|-------------|---------|-------------|--------|----------|
| ACC-01 | 13-01 through 13-11 | Parity report shows 201/201 full match (root module + segment count + diagnostics) | BLOCKED | Actual: 11/201 full match (5%). 190 fixtures fail. REQUIREMENTS.md marks this `[x]` complete but that is inaccurate -- ACC-01 is not satisfied. |

**Note:** REQUIREMENTS.md marks ACC-01 as `[x]` (complete). This is incorrect. The parity_report run against the actual codebase on 2026-04-07 shows 11/201 full match. The checkbox does not reflect reality.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/qwik-optimizer-oxc/src/inlined_fn.rs` | 1-260 | `convert_inlined_fn` (_fnSignal) implemented but NOT wired into `jsx_transform.rs` | Blocker | Signal optimization code exists but is orphaned -- this is the single largest remaining architectural gap |
| `crates/qwik-optimizer-oxc/src/types.rs` | 8 | Unused import `std::collections::HashMap` | Info | Compiler warning, no functional impact |
| `crates/qwik-optimizer-oxc/src/transform.rs` | 1947 | Unused variable `allocator` in `codegen_first_arg` | Info | Compiler warning |
| `crates/qwik-optimizer-oxc/src/hash.rs` | 244 | Dead function `parse_symbol_name` (never used) | Warning | Dead code accumulation |
| `crates/qwik-optimizer-oxc/src/errors.rs` | 23 | Dead function `create_error` (never used) | Warning | Dead code accumulation |

### Human Verification Required

None required. All key behaviors are verifiable programmatically through the parity_report test.

---

## Gaps Summary

### What Plans 13-09 through 13-11 Accomplished

The plans implemented significant infrastructure that is necessary for the final architecture:

- Auto-export `_auto_*` injection for variable migration (13-09)
- Import cleanup: removed consumed `$`-suffixed specifiers (13-09)
- Object literal single-line formatting post-processor (13-09)
- Windows path normalization (13-09)
- Universal QRL module-scope hoisting for all non-Lib strategies (13-10)
- `.s()` body generation from TRANSFORMED AST via `codegen_first_arg` (13-10)
- Stack context naming pipeline for correct segment `display_name` (13-11)
- Marker Qrl import generation for actually-used markers (13-11)
- Post-processing: PURE annotation format, arrow spacing, const ordering, import ordering (13-11)

### Why the Phase Goal Was NOT Achieved

**Primary regression: Universal QRL hoisting (plan 13-10)**

Plan 13-10 changed the emit strategy from Hoist-only to all non-Lib strategies. This changed the output format for 185 fixtures. The new format is architecturally correct (matching SWC's `hoist_qrl_to_module_scope` behavior) but since the implementation is incomplete, the new format matches SWC for fewer fixtures (11) than the old format did (107). Net effect: -96 full-match fixtures.

**Critical missing link: Signal optimization not wired**

`inlined_fn.rs` implements `convert_inlined_fn` which builds `_fnSignal` calls. `jsx_transform.rs` does not import or call this module. The 13-10 SUMMARY explicitly documents this: "Signal optimization deferred to 13-11 due to architectural constraint (JSX transform lacks access to QwikTransform state)." Plan 13-11 did not resolve this constraint.

**Remaining gap breakdown (from 13-11-SUMMARY.md):**

| Category | Fixture Count | Root Cause |
|----------|---------------|------------|
| Non-exported const stripping | ~40 | OXC keeps `const X = wrapperQrl(q_X)` when X is unexported; SWC tree-shakes to bare `wrapperQrl(q_X)` |
| Unused import removal | ~30 | OXC keeps all non-marker imports; SWC removes imports unused after transformation |
| JSX event handler extraction | ~40 | `onClick$`, `onInput$` etc. not extracted as separate segments (feature not implemented) |
| Naming/hash collision suffix | ~10 | `_1` collision suffixes differ between SWC and OXC due to segment registration order |
| Signal optimization | ~21+ | `_fnSignal`/`_wrapProp`/`_hf*` not integrated into JSX transform (orphaned in inlined_fn.rs) |
| Export handling | ~10 | Missing `export const App = componentQrl(...)` for some re-exported components |
| Nested QRL extraction | ~20 | Some nested `$()` calls in segments don't produce separate segments |

**Requirement ACC-01 is not satisfied.** The phase goal of 201/201 full SWC parity was not achieved. Current state: 11/201 (5%). The codebase has regressed from the state at the previous verification (107/201) and is now below the v0.1.0 baseline (79/201).

Structured gaps are in the YAML frontmatter above for `/gsd:plan-phase --gaps`.

---

_Verified: 2026-04-07T06:00:00Z_
_Verifier: Claude (gsd-verifier)_
