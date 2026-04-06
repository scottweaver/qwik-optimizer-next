---
phase: 05-core-oxc-implementation
plan: 01
subsystem: foundation
tags: [types, hash, words, errors, is_const, oxc]
dependency_graph:
  requires: []
  provides: [types, hash, words, errors, is_const]
  affects: [05-02, 05-03, 05-04, 05-05, 05-06, 05-07]
tech_stack:
  added: [oxc@0.123.0, oxc_traverse@0.123.0, siphasher@1, base64@0.22, indexmap@2, serde, serde_json, anyhow, pathdiff, path-slash]
  patterns: [SipHash-1-3 deterministic hashing, serde rename_all camelCase, base64 URL_SAFE_NO_PAD]
key_files:
  created:
    - crates/qwik-optimizer-oxc/Cargo.toml
    - crates/qwik-optimizer-oxc/src/lib.rs
    - crates/qwik-optimizer-oxc/src/types.rs
    - crates/qwik-optimizer-oxc/src/words.rs
    - crates/qwik-optimizer-oxc/src/hash.rs
    - crates/qwik-optimizer-oxc/src/errors.rs
    - crates/qwik-optimizer-oxc/src/is_const.rs
  modified:
    - Cargo.toml
decisions:
  - "OXC pinned at 0.123.0 with codegen/semantic/serialize/ast_visit features (no transformer)"
  - "SipHash 1-3 with seed (0,0) for deterministic segment hashing matching SWC behavior"
  - "Edition 2024 for the crate, resolver 3 for workspace"
  - "is_const_expression uses basic constness analysis; context-aware version deferred to collector module"
metrics:
  duration: 5m
  completed: 2026-04-02
  tasks: 2
  files: 8
  tests: 53
---

# Phase 5 Plan 1: Crate Scaffold & Foundation Modules Summary

OXC 0.123.0 crate scaffold with all public types, dollar-to-QRL name mapping, deterministic SipHash segment hashing, diagnostic helpers, and expression constness analysis -- 53 unit tests passing with golden hash vectors validated against SWC snapshots.

## What Was Built

### Task 1: Crate Scaffold
- Added `[workspace]` section to root `Cargo.toml` with `resolver = "3"` and member `crates/qwik-optimizer-oxc`
- Created crate manifest with OXC 0.123.0 pinned and all dependencies per D-36 research
- OXC features: `codegen`, `semantic`, `serialize`, `ast_visit` (transformer intentionally excluded per D-08)
- Module declarations in `lib.rs` with `pub use types::*` re-export

### Task 2: Foundation Modules
- **types.rs** (340 lines): All public types -- `TransformModulesOptions`, `TransformOutput`, `TransformModule`, `SegmentAnalysis`, `CtxKind`, `EntryStrategy`, `EmitMode`, `MinifyMode`, `Diagnostic`, `DiagnosticCategory`, `SourceLocation`. All serde-annotated with `rename_all = "camelCase"` and spec-matching rename attributes. `TransformOutput::default()` and `TransformOutput::append()` implemented.
- **words.rs** (120 lines): `dollar_to_qrl_name()` maps `foo$` -> `fooQrl`, `$` -> `Qrl`. `classify_ctx_kind()` classifies dollar call sites as Function or EventHandler based on `on[A-Z]` pattern matching with namespace prefix stripping.
- **hash.rs** (280 lines): `compute_segment_hash()` using SipHasher13 with seed (0,0), base64 URL_SAFE_NO_PAD encoding, `-`/`_` -> `0` replacement producing 11-char alphanumeric hashes. `escape_sym()` for identifier normalization. `register_context_name()` full 6-step naming pipeline with collision counters. `parse_symbol_name()` for re-processing existing inlinedQrl symbols. Golden hash vectors validated: `LUXeXe0DQrg`, `HTDRsvUbLiE`, `f0BGwWm4eeY`, `fsHooibmyyE`.
- **errors.rs** (50 lines): `create_source_error()`, `create_error()`, `create_warning()` diagnostic factory functions.
- **is_const.rs** (130 lines): `is_const_expression()` recursive constness analysis for JSX prop classification. Handles literals, template literals, typeof, ternary, binary, object/array expressions, parenthesized expressions.

## Verification Results

- `cargo check -p qwik-optimizer-oxc`: compiles (warnings only for unused functions that will be used by later modules)
- `cargo test -p qwik-optimizer-oxc`: 53 tests passed, 0 failed
- `cargo clippy -p qwik-optimizer-oxc`: no errors (warnings for unused functions and too_many_arguments matching reference API)
- No SWC crate dependencies in Cargo.lock

## Deviations from Plan

### Auto-added (Rule 2)

**1. [Rule 2 - Missing] Added register_context_name, escape_sym, parse_symbol_name to hash.rs**
- **Found during:** Task 2
- **Issue:** The plan mentioned only `generate_hash` but the full naming pipeline (register_context_name with collision counters, escape_sym for identifier normalization, parse_symbol_name for re-processing) is essential for every subsequent module that creates segments
- **Fix:** Implemented the full naming pipeline matching Jack's reference implementation, with comprehensive tests including golden hash vectors
- **Files modified:** crates/qwik-optimizer-oxc/src/hash.rs

**2. [Rule 2 - Missing] Added create_error and create_warning to errors.rs**
- **Found during:** Task 2
- **Issue:** Plan only mentioned create_diagnostic but the crate needs separate helpers for different severity levels
- **Fix:** Added create_error() and create_warning() alongside create_source_error()
- **Files modified:** crates/qwik-optimizer-oxc/src/errors.rs

**3. [Rule 2 - Scope] Deferred is_const_expr_with_context to collector module**
- **Found during:** Task 2
- **Issue:** The context-aware version of is_const requires GlobalCollect which depends on the collector module (Plan 05-03)
- **Fix:** Implemented basic is_const_expression only; context-aware version will be added when collector module exists
- **Files modified:** crates/qwik-optimizer-oxc/src/is_const.rs

## Known Stubs

None -- all functions are fully implemented with no placeholder data.

## Self-Check: PASSED

All 8 files verified present. Commits f2c560b (Task 1) and c010885 (Task 2) confirmed in git log.
