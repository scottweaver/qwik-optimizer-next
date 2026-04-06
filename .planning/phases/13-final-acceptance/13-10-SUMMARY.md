---
phase: 13-final-acceptance
plan: 10
subsystem: transform-pipeline
tags: [qrl-hoisting, signal-optimization, s-body-codegen, noopqrl, module-scope]
dependency_graph:
  requires: [13-09]
  provides: [universal-qrl-hoisting, s-body-codegen, module-scope-const-pattern]
  affects: [transform.rs, snapshots]
tech_stack:
  added: []
  patterns: [clone-in-codegen, universal-hoisting, argument-to-expression-codegen]
key_files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/transform.rs
    - crates/qwik-optimizer-oxc/tests/snapshots/ (185 snapshot files)
decisions:
  - "All non-Lib strategies universally hoist QRLs to module scope via extra_top_items (matching SWC hoist_qrl_to_module_scope)"
  - "Bare $() calls produce q_X directly with no wrapper; component$() -> componentQrl(q_X)"
  - "Nested QRLs stay inline in the AST; code_move handles their extraction in segment modules"
  - ".s() body generated from transformed AST via OXC CloneIn + Codegen (not source text slicing)"
  - "Signal optimization (_fnSignal/_wrapProp/_hf*) deferred to 13-11 due to architectural constraint: JSX transform is standalone and lacks access to QwikTransform.decl_stack"
metrics:
  duration: "17m"
  completed: "2026-04-06"
  tasks_completed: 1
  tasks_total: 2
---

# Phase 13 Plan 10: Signal Optimization Body Emission Summary

Universal QRL module-scope hoisting and transformed .s() body codegen for Hoist/Inline entry strategies.

## What Changed

### Task 1: Universal QRL Hoisting and .s() Body Codegen (401544b)

**QRL Hoisting (all strategies):**
- Unified all non-Lib entry strategies (Segment, Inline, Hoist) to hoist QRLs to module scope
- Segment strategy: `const q_X = /*#__PURE__*/ qrl(() => import("./path"), "sym")`
- Inline/Hoist strategy: `const q_X = /*#__PURE__*/ _noopQrl("sym")` + `q_X.s(body)`
- Lib mode unchanged: keeps `inlinedQrl(fn, "sym")` inline at call site
- Bare `$()` calls produce the QRL ident directly (no `Qrl()` wrapper)
- `component$()`, `useTask$()` etc. produce `componentQrl(q_X)`, `useTaskQrl(q_X)` wrappers

**Nested QRL handling:**
- Only top-level QRLs (segment_stack empty) are hoisted to root module
- Nested QRLs stay inline in the AST; segment modules process them via code_move
- Nested QRL segments get is_inline=false for Segment strategy (so they get their own module files) or is_inline=true for Inline/Hoist (staying in parent)

**.s() body from transformed AST:**
- Added `codegen_first_arg()` helper that serializes the TRANSFORMED first argument to a string
- Uses OXC's `CloneIn` to copy the argument into a fresh allocator, wraps in a mini Program, and codegens it
- This means .s() bodies now contain JSX already transformed to `_jsxSorted()` calls
- Previously used source text slicing which captured ORIGINAL (untransformed) function bodies

**exit_program hoisting for all strategies:**
- Changed the exit_program insertion logic from Hoist-only to all non-Lib strategies
- extra_top_items (const declarations) and ref_assignments (.s() calls) now inserted for Segment, Inline, and Hoist modes

**185 snapshot files updated** reflecting the new hoisting pattern.

### Task 2: Signal Optimization (_fnSignal/_wrapProp/_hf*) -- Partially Deferred

**Status:** Infrastructure complete, signal optimization deferred.

The .s() body codegen (Task 1) was the critical prerequisite. It now correctly generates transformed function bodies. However, implementing the signal optimization chain requires:

1. `create_synthetic_qqsegment` -- analyzes JSX prop values for signal optimization eligibility
2. `convert_inlined_fn` -- replaces scoped idents with positional params (p0, p1, ...) and builds _fnSignal calls
3. `hoist_fn_signal_call` -- deduplicates and hoists _hf* arrow functions to module scope
4. `convert_to_getter` -- orchestrates the above for each JSX prop value

**Architectural constraint:** The JSX transform in `jsx_transform.rs` is a standalone function without access to `QwikTransform.decl_stack`, `global_collect`, or `hoisted_fn_signals`. Signal optimization requires knowing which identifiers are scoped variables (from decl_stack) vs globals vs imports -- information only available on QwikTransform.

**Recommended approach for 13-11:**
- Thread a signal optimization context struct through the JSX transform
- Or move signal optimization to a pre/post pass within exit_expression before the .s() body is serialized
- The _wrapProp fast path (simple `obj.prop` member expressions) is the simplest to implement first

## Fixture Analysis

**Signal fixtures that improved partially (show correct .s() body structure):**
- example_props_wrapping: .s() body has transformed JSX (_jsxSorted), _rawProps rewriting
- example_props_optimization: .s() body has transformed JSX, _rawProps 
- example_derived_signals_div: .s() body has transformed JSX

**What these fixtures still need for full match:**
- _fnSignal calls for computed expressions
- _wrapProp calls for simple member expressions on scoped idents
- _hf* hoisted function deduplication
- Symbol name parity (pre-existing hash naming difference)
- PURE comment format parity (/*#__PURE__*/ vs /* @__PURE__ */)
- Import cleanup (consumed $-suffixed specifiers)

**Parity metrics:**
- Full match: 1/201 (unchanged -- blocked by symbol name differences)
- Root module match: 1/201 (unchanged -- blocked by symbol names + PURE format)
- Segment count match: 125/201 (maintained)
- Diagnostics match: 197/201 (maintained)
- No regressions

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] QRL hoisting missing for Segment strategy**
- **Found during:** Task 1
- **Issue:** SWC universally hoists QRLs to module scope for ALL strategies via hoist_qrl_to_module_scope, but OXC only hoisted for Hoist strategy. Segment strategy QRLs were emitted inline at the call site.
- **Fix:** Unified all non-Lib strategies to use module-scope hoisting with extra_top_items
- **Files modified:** transform.rs

**2. [Rule 1 - Bug] Bare $() calls wrapped in Qrl() call**
- **Found during:** Task 1
- **Issue:** dollar_to_qrl_name("$") returns "Qrl", causing bare $() results to be wrapped as Qrl(q_X) instead of just q_X
- **Fix:** Added special case: bare `$` ctx_name produces q_X directly, no wrapper call
- **Files modified:** transform.rs

**3. [Rule 1 - Bug] Nested QRLs hoisted to root module**
- **Found during:** Task 1
- **Issue:** All QRLs pushed to extra_top_items (root module scope), but nested QRLs should stay in their parent segment module
- **Fix:** Only hoist when segment_stack is empty (top-level); nested QRLs stay inline
- **Files modified:** transform.rs

## Deferred Items

1. **Signal optimization (_fnSignal/_wrapProp/_hf*)**: Requires JSX transform to have access to QwikTransform state (decl_stack, hoisted_fn_signals). Deferred to 13-11.
2. **PURE comment format**: OXC codegen emits `/* @__PURE__ */` but SWC expects `/*#__PURE__*/`. Needs post-processing or codegen option.
3. **Symbol name parity**: Hash-based naming produces different symbols than SWC. Pre-existing issue.

## Self-Check: PASSED
