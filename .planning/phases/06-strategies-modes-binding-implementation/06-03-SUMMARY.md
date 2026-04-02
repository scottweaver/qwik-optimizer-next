---
phase: 06-strategies-modes-binding-implementation
plan: 03
subsystem: bindings
tags: [napi, wasm, binding, transform-api]
dependency_graph:
  requires: [qwik-optimizer-oxc]
  provides: [qwik-optimizer-napi, qwik-optimizer-wasm]
  affects: [Cargo.toml]
tech_stack:
  added: [napi-rs-v3, napi-derive, napi-build, wasm-bindgen, serde-wasm-bindgen, js-sys]
  patterns: [json-serde-bridge, async-napi-binding, synchronous-wasm-binding]
key_files:
  created:
    - crates/qwik-optimizer-napi/Cargo.toml
    - crates/qwik-optimizer-napi/build.rs
    - crates/qwik-optimizer-napi/src/lib.rs
    - crates/qwik-optimizer-wasm/Cargo.toml
    - crates/qwik-optimizer-wasm/src/lib.rs
  modified:
    - Cargo.toml
    - Cargo.lock
decisions:
  - "NAPI v3 unified WASM (D-39) validated and found non-viable -- EMNAPI_LINK_DIR requirement for wasm32-wasip1-threads"
  - "wasm-bindgen fallback adopted for WASM target as planned"
  - "NAPI async feature required for async fn transform_modules (napi v3 needs explicit async feature)"
  - "napi-build stays at v2 (v3 not published to crates.io; napi-build v2 is compatible with napi v3)"
metrics:
  duration: 3m
  completed: "2026-04-02T17:20:25Z"
  tasks: 2
  files: 7
---

# Phase 06 Plan 03: NAPI and WASM Binding Crates Summary

Two thin binding crates wrapping transform_modules() -- NAPI-RS v3 for Node.js (async, JSON in/out) and wasm-bindgen for browser/edge (sync, JsValue in/out).

## What Was Done

### Task 1: NAPI Binding Crate (1e3f755)

Created `crates/qwik-optimizer-napi/` with NAPI-RS v3:
- `Cargo.toml`: napi v3 with serde-json + async features, napi-derive v3, napi-build v2
- `build.rs`: Standard napi_build::setup() call
- `src/lib.rs`: Single async fn transform_modules() that deserializes JSON config to TransformModulesOptions, calls core transform_modules(), serializes result back to JSON
- Added to workspace members in root Cargo.toml

### Task 2: WASM Binding Crate (79e8075)

Validated D-39 (NAPI v3 unified WASM): Build for wasm32-wasip1-threads fails with `EMNAPI_LINK_DIR must be set: NotPresent` -- confirms the unified approach is not yet viable without additional toolchain setup.

Created `crates/qwik-optimizer-wasm/` with wasm-bindgen fallback:
- `Cargo.toml`: wasm-bindgen, serde-wasm-bindgen, js-sys dependencies
- `src/lib.rs`: Synchronous transform_modules() matching the SWC WASM pattern exactly -- deserialize JsValue via serde-wasm-bindgen, call core, serialize result with maps-as-objects
- Added to workspace members in root Cargo.toml

## Verification Results

- `cargo check --workspace` -- all 3 crates compile (native target)
- `cargo check -p qwik-optimizer-wasm --target wasm32-unknown-unknown` -- compiles for WASM
- `cargo test -p qwik-optimizer-oxc` -- 444 tests pass (233 unit + 211 snapshot), no regression
- Core crate rayon is behind optional `parallel` feature -- not enabled for WASM builds

## Decisions Made

1. **napi-build v2 not v3**: The plan specified napi-build v3 but only v2 is published. v2 is fully compatible with napi v3 runtime.
2. **Async feature required**: NAPI v3's `#[napi] pub async fn` requires the `async` feature flag on the napi crate (pulls in tokio internally). Without it, `execute_tokio_future_with_finalize_callback` is missing.
3. **D-39 WASM validation complete**: NAPI v3 cannot target WASM without EMNAPI toolchain. wasm-bindgen is the correct fallback.

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 3 - Blocking] napi-build v3 does not exist on crates.io**
- **Found during:** Task 1
- **Issue:** Plan specified `napi-build = "3"` but latest published version is 2.3.1
- **Fix:** Changed to `napi-build = "2"` which is compatible with napi v3
- **Files modified:** crates/qwik-optimizer-napi/Cargo.toml
- **Commit:** 1e3f755

**2. [Rule 3 - Blocking] NAPI v3 async fn requires explicit async feature**
- **Found during:** Task 1
- **Issue:** `#[napi] pub async fn` generates code using `execute_tokio_future_with_finalize_callback` which requires `napi` crate's `async` feature
- **Fix:** Added `"async"` to napi features list
- **Files modified:** crates/qwik-optimizer-napi/Cargo.toml
- **Commit:** 1e3f755

## Known Stubs

None -- both bindings are fully wired to the core crate's transform_modules() function.

## Self-Check: PASSED

- All 5 created files exist on disk
- Commits 1e3f755 and 79e8075 found in git log
- Workspace compiles, WASM target compiles, 444 tests pass
