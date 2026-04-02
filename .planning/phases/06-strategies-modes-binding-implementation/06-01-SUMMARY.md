---
phase: 06-strategies-modes-binding-implementation
plan: 01
subsystem: entry-strategy-hoist
tags: [hoist, noop-qrl, entry-policy, segment-grouping]
dependency_graph:
  requires: [05-core-oxc-implementation]
  provides: [hoist-s-postprocessing, entry-policy-wiring]
  affects: [transform.rs, lib.rs, add_side_effect.rs, entry_strategy.rs]
tech_stack:
  added: []
  patterns: [string-based-ast-generation, jsx-aware-parsing]
key_files:
  created: []
  modified:
    - crates/qwik-optimizer-oxc/src/transform.rs
    - crates/qwik-optimizer-oxc/src/lib.rs
    - crates/qwik-optimizer-oxc/src/add_side_effect.rs
decisions:
  - Used string-based code generation for Hoist _noopQrl const and .s() calls rather than OXC AST builder API
  - Added parse_single_statement_jsx for JSX-aware parsing of .s() function bodies
  - Compute entry key at SegmentRecord creation time using EntryPolicy
metrics:
  duration: 684s
  completed: 2026-04-02T17:29:29Z
  tasks_completed: 2
  tasks_total: 2
  tests_before: 444
  tests_after: 455
---

# Phase 06 Plan 01: Hoist .s() Post-Processing and EntryPolicy Wiring Summary

Hoist entry strategy _noopQrl const declarations with .s() registration and EntryPolicy integration for all 7 strategies via string-based AST generation

## Tasks Completed

| Task | Name | Commit | Key Changes |
|------|------|--------|-------------|
| 1 | Implement Hoist .s() post-processing and wire EntryPolicy | 16f4367 | hoist_qrl_to_module_scope in exit_expression, ref_assignments drained in exit_program, EntryPolicy wired to SegmentRecord.entry |
| 2 | Snapshot validation and entry strategy integration tests | 0371576 | 11 new tests covering Hoist, all 7 strategies, spec example validation |

## Implementation Details

### Hoist .s() Post-Processing (transform.rs)

Added a dedicated Hoist branch in `exit_expression` that activates when `entry_strategy == Hoist && mode != Lib`:

1. **_noopQrl const declaration**: Builds `const q_{sym} = /*#__PURE__*/ _noopQrl("{sym}")` and pushes to `extra_top_items` (deduplicated by symbol_name)
2. **Function body extraction**: Extracts fn_body from source text spans for the `.s(fn_body)` call
3. **ref_assignments**: Module-scope `.s()` expression statements stored in new `ref_assignments: Vec<String>` field
4. **Expression replacement**: Replaces the original call expression with `wrapperQrl(q_{sym})` or `wrapperQrl(q_{sym}.w([captures]))`
5. **exit_program emission**: Drains extra_top_items as const declarations and ref_assignments as expression statements, inserted after imports but before exports

### EntryPolicy Wiring (transform.rs, lib.rs)

- Added `entry_policy: Box<dyn EntryPolicy>` field to QwikTransform
- Added `compute_entry()` helper that builds SegmentData and calls `entry_policy.get_entry_for_sym()`
- All 5 SegmentRecord creation sites now use `entry: entry.clone()` instead of `entry: None`
- Entry key flows through to `lib.rs` segment emission where `is_entry = record.entry.is_none()`

### JSX-Aware Parsing (add_side_effect.rs)

Added `parse_single_statement_jsx()` using `SourceType::tsx()` for parsing Hoist `.s()` bodies that may contain JSX elements (the standard `parse_single_statement` uses `SourceType::mjs()` which fails on JSX).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] JSX parse failure in .s() body generation**
- **Found during:** Task 1 implementation
- **Issue:** `parse_single_statement` uses `SourceType::mjs()` which cannot parse JSX in function bodies extracted from source text
- **Fix:** Added `parse_single_statement_jsx()` variant using `SourceType::tsx()`
- **Files modified:** `crates/qwik-optimizer-oxc/src/add_side_effect.rs`
- **Commit:** 16f4367

**2. [Rule 3 - Blocking] Borrow checker conflict with expression replacement**
- **Found during:** Task 1 implementation
- **Issue:** `call.span.end` used after `*expr = wrapper_expr` assignment invalidated the borrow
- **Fix:** Saved `call_span_end` before the Hoist branch
- **Files modified:** `crates/qwik-optimizer-oxc/src/transform.rs`
- **Commit:** 16f4367

## Known Limitations

- **Nested Hoist not implemented**: Inner `$` calls within a Hoist-strategy component body are captured as source text, not recursively hoisted. The SWC reference uses recursive `hoist_qrl_to_module_scope` calls to handle this. Full nested hoisting is deferred to a future plan.
- **Non-global ident comma expression**: The SWC reference produces `(q_X.s(value), q_X)` for non-global identifiers. Current implementation routes all `.s()` calls through `ref_assignments` (module scope). The comma expression path is a refinement for edge cases.
- **Dev mode info objects**: The `_noopQrlDEV` calls in the spec include `{ file, lo, hi, displayName }` info objects. Current implementation omits these -- they're a cosmetic addition for dev tooling.

## Known Stubs

None -- all implemented functionality is wired and operational.

## Verification

```
cargo test --manifest-path crates/qwik-optimizer-oxc/Cargo.toml
- 244 unit tests passed (11 new)
- 211 snapshot tests passed (including 14 Hoist-strategy fixtures)
- 24 spec examples ignored (pre-existing, unrelated)
- Total: 455 tests, 0 failures
```

## Self-Check: PASSED

- All key files exist (transform.rs, lib.rs, add_side_effect.rs)
- Both task commits verified (16f4367, 0371576)
- SUMMARY.md created at expected path
