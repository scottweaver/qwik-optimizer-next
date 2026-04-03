# Phase 8: Implementation Gap Closure - Research

**Researched:** 2026-04-03
**Domain:** QRL hoisting, PURE annotations, signal optimization wiring, SWC parity
**Confidence:** HIGH

## Summary

The current OXC implementation has a fundamental structural difference from SWC in the Segment strategy path: the OXC code places `qrl(() => import(...), "symbol")` inline at the original call site, but SWC hoists ALL QRL creation calls to module scope as `const q_symbolName = /*#__PURE__*/ qrl(...)` declarations, replacing the call site with just `q_symbolName` (or `q_symbolName.w([captures])` when captures exist). This hoisting pattern is the single largest source of root module mismatches (122 of 200 total mismatches are root-only, and 159/201 fixtures use Segment strategy).

Additionally, the `/*#__PURE__*/` annotation scope is broader than the CONTEXT.md decisions indicate. The spec and SWC snapshots confirm that `/*#__PURE__*/` applies to ALL `qrl()`, `inlinedQrl()`, and `_noopQrl()` calls (not just `componentQrl`), plus `componentQrl()` wrapper calls and `_jsxSorted()`/`_jsxSplit()` calls inside segment modules. The existing code already handles `_noopQrl` PURE in Hoist strategy (line 1328), but misses `qrl()` calls entirely.

**Primary recommendation:** Implement `hoist_qrl_if_needed` for Segment strategy first -- this single change will affect 159 fixtures and is the highest-impact parity improvement. Then wire PURE annotations on all QRL creation calls and `componentQrl()` wrappers.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-42: `convert_inlined_fn` (in `inlined_fn.rs`) must be called from the JSX prop classification path in `jsx_transform.rs`. During JSX prop visit, when a prop value is an expression that qualifies for signal wrapping, call `convert_inlined_fn` to produce `_fnSignal()` output. This matches SWC's flow where signal optimization happens during prop analysis.
- D-43: When `convert_inlined_fn` produces a signal, set `needs_fn_signal_import = true` on the transform state (already exists at `transform.rs:426`). The `_fnSignal` import is already wired in the synthetic import emission path (`transform.rs:1631-1632`).
- D-44: Use string-based injection during code assembly to insert `/*#__PURE__*/` before `componentQrl()` calls. This matches the established Phase 5 pattern of string assembly then re-parse for segment construction (D-05/P07). The `_needs_pure` flag at `transform.rs:1312` should drive the injection.
- D-45: PURE annotation applies ONLY to `componentQrl` calls, not to other `*Qrl` wrappers. The existing whitelist check (`qrl_wrapper_name == "componentQrl"`) is correct. Anti-list: `useTaskQrl`, `useVisibleTaskQrl`, `useResourceQrl`, etc.
- D-46: Triage root module mismatches by most common pattern divergence first, not by CONV type. Current state: 200/201 root module mismatches. Likely high-impact categories: import ordering/format, JSX output differences, QRL call format differences, whitespace/formatting. Fix the patterns that affect the most fixtures first to maximize parity percentage per fix.
- D-47: Target: at least 50/201 root module matches (25% parity). This is the minimum bar -- higher is better.
- D-48: Un-ignore all 24 `spec_examples.rs` tests at once, then document specific failures. This gives a clear picture of remaining gaps. Each failure should have a categorized reason (missing CONV, wrong output format, etc.) rather than leaving them permanently ignored.
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

### Deferred Ideas (OUT OF SCOPE)
None
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| IMPL-02 | OXC implementation supports all 14 CONV transformation types | CONV-07 (signal optimization) and CONV-08 (PURE annotations) are the two unwired CONVs. `convert_inlined_fn` exists but is never called; PURE flag computed but never injected. Wiring these completes CONV coverage. |
| IMPL-05 | OXC implementation produces functionally equivalent output to SWC version | Current parity: 1/201 root module match. Primary blocker is missing QRL hoisting in Segment strategy. Secondary blockers: missing PURE annotations, missing signal optimization. Target: 50/201 minimum. |
</phase_requirements>

## Architecture Patterns

### Critical Finding: QRL Hoisting in Segment Strategy

The SWC optimizer hoists ALL QRL creation calls to module scope, regardless of entry strategy. This is the single largest structural divergence from the current OXC output.

**SWC pattern (Segment strategy, non-Lib mode):**
```
// Hoisted QRL declarations at module scope
const q_symbolName_hash = /*#__PURE__*/ qrl(() => import("./path"), "symbolName_hash");

// Original call site replaced with just the identifier
export const myFunc = q_symbolName_hash;
// OR with captures:
export const myFunc = q_symbolName_hash.w([capture1, capture2]);
// OR with wrapper:
export const MyComp = /*#__PURE__*/ componentQrl(q_symbolName_hash);
```

**Current OXC pattern (WRONG for Segment strategy):**
```
// QRL call inline at original call site
export const myFunc = qrl(() => import("./path"), "symbolName_hash");
```

**Impact:** This affects 159/201 fixtures (all Segment strategy). Fixing this single pattern should convert most of the 122 root-only mismatches to matches.

**Implementation approach:** Extend the existing `extra_top_items` / hoisting infrastructure (currently used only for Hoist strategy) to also work for Segment strategy. The Segment path needs:
1. Create `const q_{symbol_name} = /*#__PURE__*/ qrl(() => import("./path"), "symbol_name")` as a `HoistedConst`
2. Replace the call site with `q_{symbol_name}` identifier (or `q_{symbol_name}.w([caps])` with captures)
3. Wrap with `componentQrl(q_{symbol_name})` etc. at the original position

### PURE Annotation Scope (Broader Than D-44/D-45)

**CRITICAL RESEARCH FINDING:** The CONTEXT.md decisions D-44 and D-45 state PURE applies "ONLY to componentQrl calls." However, the specification (line 706-710) and 152/201 SWC snapshots demonstrate that `/*#__PURE__*/` applies to:

1. **ALL `qrl()` calls** -- every hoisted `const q_X = /*#__PURE__*/ qrl(...)` declaration
2. **ALL `_noopQrl()` calls** -- already implemented for Hoist strategy (transform.rs:1328)
3. **ALL `inlinedQrl()` calls** -- in Inline strategy root modules
4. **`componentQrl()` wrapper calls** -- as D-44/D-45 state
5. **`_jsxSorted()` / `_jsxSplit()` calls** -- in segment module return statements

Items that do NOT get PURE: `useTaskQrl()`, `useVisibleTaskQrl()`, `useStylesQrl()`, `useResourceQrl()`, `globalActionQrl()`, and other side-effecting wrappers.

The planner should note this discrepancy with D-44/D-45 and implement the broader PURE annotation scope per the spec and SWC evidence, since IMPL-05 (SWC parity) is the driving requirement. The D-44/D-45 decisions were made during discuss without full visibility into the SWC snapshot evidence.

**Confidence: HIGH** -- verified against specification Rule 4 (line 706-710) and 152/201 SWC snapshot files.

### Signal Optimization Wiring (CONV-07)

The `convert_inlined_fn` function in `inlined_fn.rs` is complete and unit-tested. It needs to be called from `jsx_transform.rs::classify_props()` during prop classification.

**Integration point:** In `classify_props()` (jsx_transform.rs:231), after determining a prop value expression is dynamic (non-const), attempt signal wrapping via `convert_inlined_fn`. If it returns `Some(fn_signal_code)`, use the signal code as the prop value instead.

**Dependencies needed at call site:**
- `scoped_idents`: The captured identifiers for the current expression (from the parent QRL's capture analysis)
- `is_const`: Whether the expression uses only const bindings
- `is_server`: From transform config
- `allocator`: Available from `classify_props` parameters

**Challenge:** `classify_props` currently doesn't have access to the transform state's `scoped_idents` or `is_server`. The function signature needs to be extended, or signal wrapping must happen at a higher level (in `transform.rs::exit_expression` where JSX transform is called, which has access to the transform state).

### Root Module Mismatch Categories

**Confidence: HIGH** -- derived from running the parity report and examining SWC snapshots.

| Category | Estimated Impact | Fix Complexity |
|----------|-----------------|----------------|
| QRL hoisting to module scope | ~120+ fixtures | MEDIUM -- extend existing hoisting infrastructure |
| PURE annotation on qrl()/componentQrl() | ~150 fixtures (overlaps with hoisting) | LOW -- string injection |
| Import ordering (single import per specifier) | ~100+ fixtures | LOW -- change import emission |
| Missing `//` comment separators between sections | ~150+ fixtures | LOW -- add separator comments |
| Signal optimization (_fnSignal) emission | ~10 fixtures | MEDIUM -- wire convert_inlined_fn |

**Import format difference:** SWC emits one import per specifier:
```javascript
import { qrl } from "@qwik.dev/core";
import { componentQrl } from "@qwik.dev/core";
```

Current OXC emits combined imports (one per specifier already, but ordering may differ). The `//` comment lines between import block, hoisted declarations, and body are also a SWC convention that affects comparison.

### Spec Example Test Structure

The 24 spec_examples.rs tests all contain `todo!("Wire to transform_modules() when implemented")`. To un-ignore them, each test needs:
1. Remove `#[ignore]` attribute
2. Replace `todo!()` with actual `transform_modules()` call using the `SpecExampleConfig`
3. Compare output against expected patterns documented in the test comments

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| QRL hoisting | New hoisting mechanism | Extend existing `extra_top_items` + `HoistedConst` infrastructure | Already proven for Hoist strategy; same pattern applies to Segment |
| PURE comment injection | AST-level comment manipulation | String-based injection via format!() | OXC's comment handling is complex; string injection is the established pattern (D-05/P07) |
| Expression serialization for _fnSignal | Custom serializer | `serialize_expression_inner` in inlined_fn.rs | Already exists and handles OXC Codegen wrapping |

## Common Pitfalls

### Pitfall 1: PURE Annotation Scope Mismatch
**What goes wrong:** Implementing PURE only on `componentQrl()` as D-44/D-45 specify, resulting in 150+ fixtures still mismatching
**Why it happens:** D-44/D-45 were decided without examining SWC snapshot evidence showing PURE on ALL qrl creation calls
**How to avoid:** Implement PURE per the spec (Rule 4, line 706-710): on all `qrl()`, `inlinedQrl()`, `_noopQrl()` calls, plus `componentQrl()` wrappers
**Warning signs:** Parity not improving despite fixing hoisting; PURE-related diff in snapshot comparisons

### Pitfall 2: Hoisting Order vs. Declaration Order
**What goes wrong:** Hoisted `const q_X = ...` declarations emitted in wrong order relative to their usage
**Why it happens:** Segments are detected in AST traversal order (depth-first), but the SWC output orders hoisted declarations in a specific pattern
**How to avoid:** Observe SWC ordering in snapshots -- hoisted QRL consts appear in traversal order (as encountered during the transform pass), which should naturally match if using the existing `extra_top_items` push order
**Warning signs:** Tests pass but ordering differs from SWC snapshots

### Pitfall 3: Signal Optimization Context Missing
**What goes wrong:** `convert_inlined_fn` called without proper `scoped_idents` from the enclosing QRL's capture analysis
**Why it happens:** JSX prop classification happens inside `jsx_transform.rs` which is called from `exit_expression` in `transform.rs`, but the capture context is on the QwikTransform state
**How to avoid:** Pass the necessary context (scoped_idents, is_server) through the JSX transform call chain, or perform signal analysis at the `exit_expression` level where the transform state is available
**Warning signs:** `_fnSignal` calls with wrong captures or missing captures

### Pitfall 4: Segment Strategy vs. Hoist Strategy Conflation
**What goes wrong:** Applying Hoist-specific patterns (_noopQrl + .s()) to Segment strategy, or vice versa
**Why it happens:** Both patterns involve hoisting to module scope, but the mechanisms differ
**How to avoid:** Keep the paths clearly separated:
- **Segment strategy:** `const q_X = /*#__PURE__*/ qrl(() => import(...), "sym")` -- uses `qrl()` with dynamic import
- **Hoist/Inline strategy:** `const q_X = /*#__PURE__*/ _noopQrl("sym"); q_X.s(fnBody);` -- uses `_noopQrl()` + `.s()` registration
**Warning signs:** Segment fixtures producing `_noopQrl` output or Hoist fixtures producing `qrl()` output

### Pitfall 5: Import Deduplication
**What goes wrong:** Multiple `import { qrl } from "@qwik.dev/core"` lines when only one is needed
**Why it happens:** SWC emits one import per specifier (e.g., separate `import { qrl }` and `import { componentQrl }` statements), but each specifier appears only once
**How to avoid:** The existing `imports_to_add` mechanism already deduplicates. Ensure the hoisted QRL pattern doesn't trigger additional import additions beyond what the import flags already provide
**Warning signs:** Duplicate import lines in output

## Code Examples

### QRL Hoisting for Segment Strategy (Proposed)

```rust
// In the Segment strategy branch (transform.rs ~line 1492):
// Instead of modifying call.arguments inline, create a HoistedConst

let import_path = format!("./{}", names.canonical_filename);
let qrl_callee = if is_dev { "qrlDEV" } else { "qrl" };
let ident_name = format!("q_{}", names.symbol_name);

// Build hoisted declaration
let qrl_rhs = format!(
    r#"/*#__PURE__*/ {}(()=>import("{}"), "{}")"#,
    qrl_callee, import_path, names.symbol_name
);

self.extra_top_items.push(HoistedConst {
    name: ident_name.clone(),
    rhs_code: qrl_rhs,
    symbol_name: names.symbol_name.clone(),
});

// Replace call site with q_symbolName (or q_symbolName.w([caps]))
let replacement = if has_captures {
    let caps_str = scoped_idents.join(", ");
    format!("{}.w([{}])", ident_name, caps_str)
} else {
    ident_name.clone()
};

// Wrap in qrl_wrapper if needed (e.g., componentQrl(q_sym))
let wrapper_code = if qrl_wrapper_name == pending.ctx_name.replace('$', "Qrl") {
    // The callee was already renamed to e.g. componentQrl
    format!("{}({})", qrl_wrapper_name, replacement)
} else {
    replacement
};
```

### PURE on componentQrl Wrapper

```rust
// When building the wrapper call for componentQrl:
let wrapper_expr = if qrl_wrapper_name == "componentQrl" {
    format!("/*#__PURE__*/ {}({})", qrl_wrapper_name, replacement)
} else {
    format!("{}({})", qrl_wrapper_name, replacement)
};
```

### Signal Optimization Call Site

```rust
// In jsx_transform.rs classify_props or transform.rs exit_expression:
// After determining a prop value is dynamic, attempt signal wrapping

use crate::inlined_fn::convert_inlined_fn;

let (fn_signal_opt, _arrow_code, _is_const) = convert_inlined_fn(
    &value_expr,
    &scoped_idents,  // from enclosing QRL capture analysis
    is_const,
    is_server,
    allocator,
);

if let Some(fn_signal_code) = fn_signal_opt {
    // Use fn_signal_code as the prop value instead of the original expression
    // Also set needs_fn_signal_import = true on the transform state
}
```

## Current Parity Baseline (Verified 2026-04-03)

| Metric | Current | Target |
|--------|---------|--------|
| Full match | 1/201 (0.5%) | 50/201 (25%) |
| Root module match | 1/201 | 50/201 |
| Segment count match | 125/201 | -- |
| Diagnostics match | 197/201 | -- |
| Spec examples passing | 0/24 | Document all failure reasons |

**Fixture distribution:**
- Segment strategy: 159 fixtures
- Inline strategy: 22 fixtures
- Hoist strategy: 13 fixtures
- Single strategy: 4 fixtures
- Smart strategy: 3 fixtures
- Test mode: 187 fixtures
- Dev mode: 7 fixtures
- Prod mode: 5 fixtures
- HMR mode: 1 fixture
- Lib mode: 1 fixture

**Mismatch breakdown (200 mismatches):**
- Root-only: 122 (correct segments + diagnostics, wrong root module)
- Includes segment count issues: 76
- Diagnostics-only: 4
- (Categories overlap -- some have both root and segment issues)

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (Rust built-in) + insta 1.47.2 |
| Config file | `Cargo.toml` (workspace) |
| Quick run command | `cargo test -p qwik-optimizer-oxc --test snapshot_tests swc_parity -- --nocapture` |
| Full suite command | `cargo test -p qwik-optimizer-oxc` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| IMPL-02 | All 14 CONVs supported (CONV-07/08 wiring) | integration | `cargo test -p qwik-optimizer-oxc --test snapshot_tests -- --nocapture` | Yes |
| IMPL-05 | SWC parity >= 50/201 | integration | `cargo test -p qwik-optimizer-oxc --test snapshot_tests swc_parity -- --nocapture` | Yes |
| IMPL-05 | Spec examples un-ignored | integration | `cargo test -p qwik-optimizer-oxc --test spec_examples -- --nocapture` | Yes (currently all ignored) |

### Sampling Rate
- **Per task commit:** `cargo test -p qwik-optimizer-oxc --test snapshot_tests swc_parity -- --nocapture`
- **Per wave merge:** `cargo test -p qwik-optimizer-oxc`
- **Phase gate:** Parity report shows >= 50/201 root module matches

### Wave 0 Gaps
None -- existing test infrastructure covers all phase requirements. The snapshot tests and parity report are already in place.

## Project Constraints (from CLAUDE.md)

- OXC 0.123 pinned; no SWC crates allowed
- Use `Traverse` trait, arena allocators, `SemanticBuilder`, `Codegen` -- OXC idioms
- `std::sync::LazyLock` instead of `lazy_static`
- NEVER modify `.snap` files -- if tests fail, the code is broken
- Snapshot files under `tests/swc_expected/` are read-only golden references
- Pull all 3 reference repos before work (feedback_pull_refs.md)
- Behavioral fidelity: functionally equivalent output (cosmetic differences acceptable)

## Sources

### Primary (HIGH confidence)
- `specification/qwik-optimizer-spec.md` lines 625-710 -- QRL Wrapping rules, PURE annotation scope
- `specification/qwik-optimizer-spec.md` lines 4682-4722 -- InlineStrategy (Inline/Hoist) behavior
- 201 SWC snapshot files in `tests/swc_expected/` -- behavioral truth for expected output
- `fixtures.json` -- fixture configuration (entry_strategy, mode distribution)

### Secondary (HIGH confidence)
- `crates/qwik-optimizer-oxc/src/transform.rs` -- current OXC implementation, lines 1300-1584 (QRL wrapping paths)
- `crates/qwik-optimizer-oxc/src/inlined_fn.rs` -- complete convert_inlined_fn implementation
- `crates/qwik-optimizer-oxc/src/jsx_transform.rs` -- JSX prop classification (integration point for CONV-07)
- `crates/qwik-optimizer-oxc/tests/snapshot_tests.rs` -- parity report infrastructure

### Tertiary (MEDIUM confidence)
- SWC source at `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` -- reference for hoisting behavior (not directly verified in this research, but spec describes it)

## Metadata

**Confidence breakdown:**
- QRL hoisting pattern: HIGH -- verified against spec + 152 SWC snapshots
- PURE annotation scope: HIGH -- verified against spec Rule 4 + snapshot evidence (contradicts D-44/D-45)
- Signal optimization wiring: HIGH -- code exists, integration point identified
- Parity improvement estimate: MEDIUM -- 50/201 target achievable if hoisting + PURE are implemented correctly

**Research date:** 2026-04-03
**Valid until:** 2026-04-17 (stable domain, no external dependencies changing)
