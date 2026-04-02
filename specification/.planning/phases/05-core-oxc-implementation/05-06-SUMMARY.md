---
phase: 05-core-oxc-implementation
plan: 06
subsystem: jsx-props-signals
tags: [jsx-transform, props-destructuring, signal-optimization, conv-04, conv-06, conv-07]
dependency_graph:
  requires: [05-05]
  provides: [jsx-transform, props-destructuring, signal-optimization]
  affects: [transform-pipeline, import-tracking]
tech_stack:
  added: []
  patterns: [visitor-mut-prepass, exit-expression-jsx-rewrite, eligibility-checks]
key_files:
  created:
    - crates/qwik-optimizer-oxc/src/props_destructuring.rs
    - crates/qwik-optimizer-oxc/src/jsx_transform.rs
    - crates/qwik-optimizer-oxc/src/inlined_fn.rs
  modified:
    - crates/qwik-optimizer-oxc/src/transform.rs
    - crates/qwik-optimizer-oxc/src/lib.rs
decisions:
  - Props destructuring implemented as VisitMut pre-pass, running before Traverse
  - JSX transform integrated into exit_expression via take-and-replace pattern
  - Signal optimization uses string-based rendering for _fnSignal construction
metrics:
  duration: ~14m
  completed: 2026-04-02
  tasks: 2
  files: 5
---

# Phase 05 Plan 06: JSX Transform, Props Destructuring, and Signal Optimization Summary

Implemented CONV-04 (props destructuring), CONV-06 (JSX transform), and CONV-07 (signal optimization) -- the remaining Layer 4 transformations that cover the bulk of real-world Qwik component transformation behavior.

## One-liner

Props destructuring rewrites destructured params to _rawProps access, JSX elements become _jsxSorted/_jsxSplit calls with static/dynamic prop separation, and _fnSignal wraps eligible signal expressions.

## Tasks Completed

### Task 1: Props Destructuring (CONV-04) -- dc41418
**files:** props_destructuring.rs, transform.rs, lib.rs

Implemented as a `VisitMut` pre-pass that runs BEFORE capture analysis. Transforms destructured arrow function parameters to `_rawProps` member access patterns:

- Basic destructuring: `({foo, bar}) => ...` becomes `(_rawProps) => { const foo = _rawProps.foo; ... }`
- String-keyed props: `{"my-prop": x}` uses computed access `_rawProps["my-prop"]`
- Default values: Uses nullish coalescing (`??`) not logical OR
- Rest props: `({foo, ...rest})` generates `_restProps(_rawProps, ["foo"])` with import tracking
- Expression body conversion: Arrow expression bodies converted to block with return
- Skip conditions: non-object param, 2+ params, no return, precompiled lib bodies, non-const defaults
- 12 unit tests covering all behavioral rules

### Task 2: JSX Transform (CONV-06) and Signal Optimization (CONV-07) -- fa418da
**files:** jsx_transform.rs, inlined_fn.rs, transform.rs, lib.rs

**jsx_transform.rs (CONV-06):** Converts JSX elements/fragments to function calls:
- `_jsxSorted(tag, varProps, constProps, children, flags, key)` for static elements
- `_jsxSplit(...)` for elements with spread props
- Tag classification: lowercase=intrinsic (string), uppercase=component (identifier)
- Prop classification using `is_const_expression` for static/dynamic split
- className -> class normalization
- key extraction from props, deterministic counter-based key generation
- Fragment support via `_Fragment` identifier
- Children handling: text nodes, expression containers, nested JSX
- Namespaced attributes (e.g., `bind:value`)

**inlined_fn.rs (CONV-07):** Signal optimization with 6-check eligibility:
1. Not an arrow function (already a QRL boundary)
2. No call expressions (side effects)
3. Must have object member usage (signal access pattern)
4. No abort constructs (=>, function, class, @)
5. Rendered length <= 150 chars
6. Must have captures

- `ObjectUsageChecker` visitor for member/call detection
- `ReplaceIdentifiers` for `p0`/`p1` parameter substitution via string replacement
- Server mode adds third string argument for debugging
- 7 unit tests for eligibility checks + 6 integration tests for JSX pipeline

## Integration

- Props destructuring runs as Stage 4b (after filter_exports, before Traverse)
- JSX transform hooks into `exit_expression` via take-and-replace pattern
- Import needs tracked: `needs_jsx_sorted_import`, `needs_jsx_split_import`, `needs_fragment_import`, `needs_fn_signal_import`, `needs_wrap_prop_import`
- JSX key counter maintained on QwikTransform struct

## Verification

- `cargo build -p qwik-optimizer-oxc`: Compiles cleanly
- `cargo test -p qwik-optimizer-oxc`: 207 lib tests pass, 10 snapshot tests pass, 0 failures
- Props destructuring runs before capture analysis in pipeline
- JSX elements correctly transformed to _jsxSorted/_jsxSplit calls
- Spread props correctly route through _jsxSplit
- className normalization verified

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

- JSX `bind:value`/`bind:checked` sugar expansion is recognized but not fully expanded to dual props (the attribute name passes through as `bind:value` in const props). Full bind sugar requires integration with the event handler extraction path.
- `_wrapProp` fast-path for simple `store.property` signal access in JSX props is implemented in `inlined_fn.rs` logic but not yet wired into the JSX prop classification pipeline. The `_fnSignal` path is fully functional.
- The `ref` prop is treated as a regular prop rather than being extracted to a separate argument position.

## Self-Check: PASSED

- All 5 files exist (3 created, 2 modified)
- Both commits found: dc41418, fa418da
- Line counts meet minimums: props_destructuring(571), jsx_transform(748), inlined_fn(366)
- Must-have patterns present: _jsxSorted(11), _fnSignal(8), _rawProps(32)
- cargo test: 207 lib tests pass, 0 failures
