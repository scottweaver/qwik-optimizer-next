---
phase: 05-core-oxc-implementation
plan: 03
subsystem: parse-collect-layer
tags: [oxc, parser, semantic, global-collect, entry-strategy, rename-imports]
dependency_graph:
  requires: [05-01, 05-02]
  provides: [parse-module, global-collect, entry-policy, rename-imports]
  affects: [05-04, 05-05, 05-06]
tech_stack:
  added: []
  patterns: [VisitMut-pre-pass, EntryPolicy-trait-dispatch, arena-string-allocation]
key_files:
  created:
    - crates/qwik-optimizer-oxc/src/parse.rs
    - crates/qwik-optimizer-oxc/src/collector.rs
    - crates/qwik-optimizer-oxc/src/entry_strategy.rs
    - crates/qwik-optimizer-oxc/src/rename_imports.rs
  modified:
    - crates/qwik-optimizer-oxc/src/lib.rs
    - crates/qwik-optimizer-oxc/src/types.rs
decisions:
  - "Used oxc::span::Str instead of Atom for StringLiteral.value mutation (OXC 0.123 API change)"
  - "SegmentData added to types.rs as internal type with minimal fields needed for entry policy"
metrics:
  duration: 7m
  completed: 2026-04-02
---

# Phase 05 Plan 03: Parse + Collect Layer Summary

OXC Parser/SemanticBuilder wrapper, GlobalCollect import/export/root indexing, EntryPolicy trait with 5 strategy implementations, and @builder.io legacy import renaming pre-pass using VisitMut.

## What Was Built

### parse.rs -- Parser + SemanticBuilder Wrapper
- `parse_module()`: Parses JS/TS/JSX/TSX source into OXC Program AST with Scoping
- `source_type_from_filename()`: Maps file extensions to OXC SourceType (Qwik enables JSX in .ts)
- `parse_path()`: Decomposes relative file paths into stem/name/rel_dir/abs_dir
- `output_extension()`: Maps (input_ext, transpile_ts, transpile_jsx) to output extension
- `ParseResult` struct: Bundles Program, SourceType, and Scoping
- Error handling: Returns diagnostics for recoverable errors, Err for panicked parse

### collector.rs -- GlobalCollect (Stage 2)
- `GlobalCollect` struct with imports (IndexMap), exports (IndexMap), root (IndexMap), rev_imports (HashMap)
- Handles all import forms: named, default, namespace
- Handles all export forms: named specifiers, inline declarations, default, re-exports
- Root-level declarations: var, function, class (correctly excludes imports/exports)
- Recursive binding pattern support for destructured exports
- `global_collect_from_str()` convenience for tests
- Query methods: `is_import()`, `get_import()`, `get_imported_local()`, `is_global()`, `add_synthetic_import()`

### entry_strategy.rs -- EntryPolicy Trait + 5 Implementations
- `EntryPolicy` trait with `get_entry_for_sym(context, segment) -> Option<String>`
- `InlineStrategy`: All segments share "entry_segments" chunk (Inline + Hoist)
- `SingleStrategy`: All segments share "entry_segments" chunk
- `PerSegmentStrategy`: Each segment gets own chunk (Segment + Hook)
- `PerComponentStrategy`: Groups by root component name
- `SmartStrategy`: Heuristic -- pure event handlers get own chunk, rest per-component
- `parse_entry_strategy()` factory and `is_inline()` helper

### rename_imports.rs -- Legacy Import Rename Pre-pass
- `RenameTransform` VisitMut visitor that rewrites @builder.io import sources
- Rename table: qwik-city -> router, qwik-react -> react, qwik -> core
- Prefix order is load-bearing (longer prefixes checked first)
- Arena string allocation via `Allocator::alloc_str()` + `Str::from()`
- Only import declarations affected; export-from intentionally unchanged

## Test Results

- **123 unit tests passing** (up from 48 before this plan)
- **10 integration tests passing** (unchanged)
- **0 failures**

New tests added: 75 (26 parse, 19 collector, 21 entry_strategy, 9 rename_imports)

## Commits

| Task | Name | Commit | Files |
|------|------|--------|-------|
| 1 | Parse + GlobalCollect | 9475c37 | parse.rs, collector.rs, lib.rs |
| 2 | EntryStrategy + RenameImports | 9839adb | entry_strategy.rs, rename_imports.rs, lib.rs, types.rs |

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] OXC 0.123 API: Str replaces Atom for StringLiteral.value**
- **Found during:** Task 2
- **Issue:** Plan referenced `oxc::span::Atom` for arena string allocation, but OXC 0.123 uses `oxc::span::Str` (from oxc_str crate)
- **Fix:** Used `Str::from(interned)` instead of `Atom::from(interned)` for import source mutation
- **Files modified:** crates/qwik-optimizer-oxc/src/rename_imports.rs
- **Commit:** 9839adb

**2. [Rule 2 - Missing] SegmentData type needed for entry_strategy**
- **Found during:** Task 2
- **Issue:** EntryPolicy trait requires SegmentData for policy decisions, but it was not in types.rs
- **Fix:** Added SegmentData struct to types.rs with fields needed for entry policy (ctx_kind, ctx_name, scoped_idents, origin, etc.)
- **Files modified:** crates/qwik-optimizer-oxc/src/types.rs
- **Commit:** 9839adb

## Known Stubs

None -- all modules are fully implemented with working logic and comprehensive tests.

## Self-Check: PASSED

All 4 created files verified present. Both task commits (9475c37, 9839adb) verified in git log.
