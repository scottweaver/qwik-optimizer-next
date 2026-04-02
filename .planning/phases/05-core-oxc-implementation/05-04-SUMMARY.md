---
phase: 05-core-oxc-implementation
plan: 04
subsystem: transform-core
tags: [const-replace, filter-exports, dollar-detection, traverse, CONV-01, CONV-10, CONV-11]
dependency_graph:
  requires: [05-01, 05-03]
  provides: [const_replace, filter_exports, QwikTransform, transform_code]
  affects: [05-05, 05-06, 05-07]
tech_stack:
  added: []
  patterns: [VisitMut-pre-pass, Traverse-main-pass, pipeline-orchestration]
key_files:
  created:
    - crates/qwik-optimizer-oxc/src/const_replace.rs
    - crates/qwik-optimizer-oxc/src/filter_exports.rs
    - crates/qwik-optimizer-oxc/src/transform.rs
  modified:
    - crates/qwik-optimizer-oxc/src/lib.rs
    - crates/qwik-optimizer-oxc/src/types.rs
decisions:
  - "Pre-traverse mutations use VisitMut (not Traverse) per spec Pattern 4 and OXC idioms"
  - "filter_ctx_names implemented as separate function from filter_exports for cleaner separation of concerns"
  - "QwikTransform uses owned config copies rather than borrowed references to avoid lifetime complexity"
  - "transform_code orchestrates full pipeline in a single function matching spec pipeline ordering"
metrics:
  duration: "8m"
  completed: "2026-04-02"
  tasks: 2
  files: 5
---

# Phase 05 Plan 04: Pre-traverse Mutations and QwikTransform Skeleton Summary

Layer 3 (const_replace, filter_exports) and Layer 4 foundation (QwikTransform with CONV-01 dollar detection) implemented with 33 passing tests across both modules.

## Completed Tasks

| Task | Name | Commit | Key Files |
|------|------|--------|-----------|
| 1 | const_replace.rs and filter_exports.rs (pre-traverse mutations) | 00fbf5d | const_replace.rs, filter_exports.rs, types.rs |
| 2 | QwikTransform skeleton with dollar detection (CONV-01) | 9825c17 | transform.rs, lib.rs |

## Implementation Details

### Task 1: Pre-traverse Mutations

**const_replace.rs (CONV-10):**
- VisitMut pass replacing `isServer`, `isBrowser`, `isDev` with boolean literals
- Recognizes imports from both `@qwik.dev/core` and `@qwik.dev/core/build`
- Denylist: Lib and Test modes preserve identifiers as-is
- isDev mapping: true for Dev/Hmr modes, false for Prod
- Only replaces identifiers that are actual imports (not local variables with same name)

**filter_exports.rs (CONV-11):**
- `filter_exports`: Replaces stripped export bodies with `() => { throw "Symbol removed..." }` stubs
- Handles both `export const name = fn` and `export function name() {}` patterns
- Single-declarator only (multi-declarator and destructuring skipped per spec)
- `filter_ctx_names`: Strips call expressions matching strip_ctx_name list
- `strip_event_handlers`: Removes all event handler $ calls (on[A-Z]*$ pattern)

**TransformCodeOptions:** Added to types.rs as the per-module config struct.

### Task 2: QwikTransform + Dollar Detection

**QwikTransform struct:**
- Implements `Traverse<'a, ()>` with `enter_call_expression` for dollar detection
- `marker_functions` map populated from GlobalCollect imports (all $-suffixed Named imports from core module)
- Also detects locally-exported $-suffixed functions as markers
- Special-case resolution for `$` (qsegment_fn) and `sync$` (sync_qrl_fn)
- SegmentScope pushed onto segment_stack for each detected $ call

**Dollar detection rules (CONV-01):**
1. Callee must be an identifier matching a known marker function
2. First argument must be a function/arrow expression
3. Call must not be stripped by strip_ctx_name or strip_event_handlers
4. sync$ detected and flagged separately (CONV-13, not QRL extraction)

**transform_code pipeline orchestration:**
1. Parse (parse_module from parse.rs)
2. Pre-traverse rename imports (rename_imports)
3. GlobalCollect (global_collect from collector.rs)
4. Pre-traverse const_replace, filter_exports, filter_ctx_names
5. Create QwikTransform and run traverse_mut
6. Generate output with Codegen

## Decisions Made

1. Pre-traverse mutations use VisitMut per spec Pattern 4 -- these run before the main Traverse pass
2. filter_ctx_names separated from filter_exports for cleaner API (different inputs)
3. QwikTransform uses owned config copies to avoid complex lifetime annotations
4. Pipeline orchestration in transform_code matches spec stage ordering exactly

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] OXC 0.123 API differences from reference code**
- **Found during:** Task 1
- **Issue:** Reference code used `ast.atom(name)` and `ast.expression_void()` which don't exist in OXC 0.123
- **Fix:** Used `name` directly (Into<Ident> impl on &str) and `ast.expression_unary(SPAN, UnaryOperator::Void, ...)` respectively
- **Files modified:** filter_exports.rs

## Test Results

- const_replace: 10 tests (replacement, denylist, import-only, Test mode exception)
- filter_exports: 11 tests (strip exports, strip ctx names, event handlers, edge cases)
- transform: 12 tests (dollar detection, marker resolution, pipeline integration)
- Total: 33 new tests, 166 total passing in crate (156 lib + 10 snapshot)

## Known Stubs

- `exit_expression` in QwikTransform: Empty -- will be filled in Plan 05 with QRL wrapping and capture analysis
- `exit_program` in QwikTransform: Empty -- will be filled in later plans with import rewriting
- Segment emission not yet implemented -- transform_code returns single output module without extracted segments

## Self-Check: PASSED

All 5 created/modified files exist. Both task commits (00fbf5d, 9825c17) verified in git log.
