# Phase 8: Implementation Gap Closure - Context

**Gathered:** 2026-04-03
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire the disconnected signal optimization (CONV-07) and PURE annotation (CONV-08) code paths into the main transform pipeline, un-ignore the 24 spec_examples.rs tests, and significantly improve SWC parity from the current 1/201 full match (1/201 root module match) to at least 50/201. This is implementation work on the existing `qwik-optimizer-oxc` crate — no spec changes, no new crates.

</domain>

<decisions>
## Implementation Decisions

### Signal Optimization Wiring (CONV-07)
- **D-42:** `convert_inlined_fn` (in `inlined_fn.rs`) must be called from the JSX prop classification path in `jsx_transform.rs`. During JSX prop visit, when a prop value is an expression that qualifies for signal wrapping, call `convert_inlined_fn` to produce `_fnSignal()` output. This matches SWC's flow where signal optimization happens during prop analysis.
- **D-43:** When `convert_inlined_fn` produces a signal, set `needs_fn_signal_import = true` on the transform state (already exists at `transform.rs:426`). The `_fnSignal` import is already wired in the synthetic import emission path (`transform.rs:1631-1632`).

### PURE Annotation Injection (CONV-08)
- **D-44:** Use string-based injection during code assembly to insert `/*#__PURE__*/` before `componentQrl()` calls. This matches the established Phase 5 pattern of string assembly then re-parse for segment construction (D-05/P07). The `_needs_pure` flag at `transform.rs:1312` should drive the injection.
- **D-45:** PURE annotation applies ONLY to `componentQrl` calls, not to other `*Qrl` wrappers. The existing whitelist check (`qrl_wrapper_name == "componentQrl"`) is correct. Anti-list: `useTaskQrl`, `useVisibleTaskQrl`, `useResourceQrl`, etc.

### SWC Parity Improvement Strategy
- **D-46:** Triage root module mismatches by most common pattern divergence first, not by CONV type. Current state: 200/201 root module mismatches. Likely high-impact categories: import ordering/format, JSX output differences, QRL call format differences, whitespace/formatting. Fix the patterns that affect the most fixtures first to maximize parity percentage per fix.
- **D-47:** Target: at least 50/201 root module matches (25% parity). This is the minimum bar — higher is better.

### Spec Example Test Activation
- **D-48:** Un-ignore all 24 `spec_examples.rs` tests at once, then document specific failures. This gives a clear picture of remaining gaps. Each failure should have a categorized reason (missing CONV, wrong output format, etc.) rather than leaving them permanently ignored.

### Carrying Forward
- D-33: Crate at `crates/qwik-optimizer-oxc/`
- D-34: Spec-driven, consult references when stuck
- D-36: OXC 0.123.0 pinned
- D-40: EntryPolicy in code_move.rs
- D-41: EmitMode threading through pipeline

### Claude's Discretion
- Order of parity fixes within the triage strategy
- Whether to batch parity fixes by pattern or by fixture
- Internal refactoring needed to support CONV-07/08 wiring
- How to categorize and report spec_examples.rs test failures

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Primary Reference (Spec Document)
- `specification/qwik-optimizer-spec.md` — CONV-07 (Signal Optimization) and CONV-08 (PURE Annotations) sections define expected behavior

### Implementation (Phase 5/6 Output)
- `crates/qwik-optimizer-oxc/src/inlined_fn.rs` — `convert_inlined_fn` function (exists but unused in pipeline)
- `crates/qwik-optimizer-oxc/src/jsx_transform.rs` — JSX prop classification (integration point for CONV-07)
- `crates/qwik-optimizer-oxc/src/transform.rs` — Main transform pipeline, PURE annotation flag at line 1312, `_fnSignal` import at lines 1631-1632
- `crates/qwik-optimizer-oxc/src/code_move.rs` — Segment emission (string assembly pattern)

### Test Corpus
- `crates/qwik-optimizer-oxc/tests/snapshot_tests.rs` — 201 SWC snapshot tests + parity report
- `crates/qwik-optimizer-oxc/tests/spec_examples.rs` — 24 ignored spec example tests
- `crates/qwik-optimizer-oxc/tests/swc_expected/*.snap` — 201 SWC golden reference snapshots

### SWC Source (Behavioral Truth)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` — SWC signal optimization and PURE annotation logic
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/code_move.rs` — SWC segment emission patterns

### Jack's OXC Implementation (Pattern Reference)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/` — Jack's implementation of CONV-07/08

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `inlined_fn.rs` has complete `convert_inlined_fn` with 6 eligibility checks, `ObjectUsageChecker`, and `_fnSignal` construction — just needs to be called
- `transform.rs` already has `needs_fn_signal_import` flag and `_fnSignal` synthetic import emission
- `transform.rs` already computes `_needs_pure` flag for componentQrl detection
- String assembly pattern established in `code_move.rs` for segment construction
- 201 snapshot tests all pass — regression safety net is solid

### Established Patterns
- String-based code assembly then re-parse (Phase 5 D-05/P07)
- `enter_*/exit_*` visitor hooks in `transform.rs` for pipeline integration
- JSX prop analysis in `jsx_transform.rs` with `PropClassification` enum
- Synthetic import injection via `imports_to_add` vector in `emit_synthetic_imports`

### Integration Points
- `convert_inlined_fn` call site: JSX prop classification in `jsx_transform.rs`
- PURE annotation: code assembly output for `componentQrl` calls
- Parity fixes: primarily `transform.rs` (QRL generation), `code_move.rs` (segment emission), `jsx_transform.rs` (JSX output format)

### Current Parity Baseline
- Full match: 1/201 (0.5%)
- Root module match: 1/201
- Segment count match: 125/201
- Diagnostics match: 197/201
- 24 spec_examples.rs tests ignored

</code_context>

<specifics>
## Specific Ideas

- Start with parity analysis: categorize the 200 root module mismatches to find the top 5 pattern divergences
- Signal optimization and PURE annotations may each fix a category of mismatches, improving parity
- The parity report test (`swc_parity::parity_report`) can be run with `--nocapture` to see detailed mismatch info
- Many root module mismatches may be cosmetic (import ordering, whitespace) — fixing these first gives quick wins
- Segment count matches at 125/201 — 76 fixtures have wrong segment counts, which are deeper issues (likely in dollar detection or segment extraction)

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 08-implementation-gap-closure*
*Context gathered: 2026-04-03*
