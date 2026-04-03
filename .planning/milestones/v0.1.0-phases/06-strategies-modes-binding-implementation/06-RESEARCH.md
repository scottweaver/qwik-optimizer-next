# Phase 6: Strategies, Modes & Binding Implementation - Research

**Researched:** 2026-04-01
**Domain:** Entry strategy wiring, emit mode branching, NAPI-RS v3 bindings, WASM bindings
**Confidence:** HIGH

## Summary

Phase 6 wires behavioral variations (7 entry strategies, 5 emit modes) into the existing OXC core crate, then exposes the result through NAPI and WASM bindings for drop-in SWC replacement. The core transform from Phase 5 already implements all 14 CONVs and has 475 tests passing, but several strategy/mode-specific behaviors are not yet implemented: the Hoist strategy `.s()` post-processing, HMR `_useHmr()` injection, and the Lib mode segment-extraction skip. The EntryPolicy trait and all 5 implementations exist but are not yet wired into the segment grouping pipeline.

The NAPI binding is a thin wrapper: the SWC version is 42 lines calling `transform_modules()` with JSON serde. NAPI-RS v3 replaces the v2 tokio-based async pattern with native async support. The WASM binding is 21 lines using `serde-wasm-bindgen`. Both are structurally simple -- the complexity is in ensuring the core crate's `transform_modules()` public API is fully correct before wrapping it.

**Primary recommendation:** Implement missing mode/strategy behaviors in the core crate first (Hoist `.s()`, HMR injection, mode-specific skips), validate with spec fixtures, then add NAPI and WASM binding crates as thin wrappers.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-37:** Separate `crates/qwik-optimizer-napi/` crate depending on qwik-optimizer-oxc. Clean separation of binding from transform logic.
- **D-38:** NAPI-RS v3. Newer version, handles async natively without explicit tokio setup.
- **D-39:** NAPI v3 unified approach -- try `wasm32-wasip1-threads` target first for browser WASM. If browser compatibility fails, fall back to wasm-bindgen with separate crate. Validate early.
- **D-40:** Wire EntryPolicy selection into the segment grouping logic in code_move.rs via the existing `transform_modules()` public API. The Hoist strategy needs a `.s()` post-processing pass that converts `inlinedQrl()` to `_noopQrl()` + top-level `.s()` registration.
- **D-41:** Thread EmitMode through the transform pipeline to control per-CONV behavioral variations. Key mode-specific behaviors: Dev/Hmr use `qrlDEV`/`inlinedQrlDEV` with source locations, Test skips const replacement, Lib skips segment extraction and DCE, Hmr injects `_useHmr()`.

### Claude's Discretion
None specified -- all decisions are locked.

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| IMPL-03 | OXC implementation supports all 7 entry strategies | EntryPolicy trait + 5 implementations exist in entry_strategy.rs. Missing: wiring into segment grouping in code_move.rs, Hoist `.s()` post-processing in transform.rs |
| IMPL-04 | OXC implementation supports all 5 emit modes | EmitMode enum exists; Dev/Prod QRL variant switching already works; missing: HMR `_useHmr()` injection, Lib mode segment-extraction skip behavior, Test mode const-replacement skip (already implemented in const_replace.rs) |
| IMPL-06 | NAPI binding exposes `transform_modules` to Node.js with same JSON interface as SWC version | No NAPI crate exists yet. Reference: SWC's 42-line NAPI binding. D-37/D-38 lock the approach. |
| IMPL-07 | WASM binding exposes `transform_modules` to browsers/edge with same interface as SWC version | No WASM crate exists yet. Reference: SWC's 21-line WASM binding. D-39 locks the approach (try unified NAPI v3, fall back to wasm-bindgen). |
</phase_requirements>

## Standard Stack

### Core (already in workspace)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `oxc` | `=0.123.0` | Parser, AST, codegen, semantic | Already pinned (D-36) |
| `oxc_traverse` | `=0.123.0` | Traverse trait for transform | Already in use |
| `serde` | `1` | JSON serialization for public types | Already in use |
| `serde_json` | `1` | JSON encoding/decoding | Already in use |

### New Dependencies (NAPI crate)
| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `napi` | `3` (features: `["serde-json"]`) | Node.js native addon bindings | D-38 locks NAPI-RS v3. `serde-json` feature enables direct serde type conversion. |
| `napi-derive` | `3` | `#[napi]` proc macro for exports | Companion to napi crate |
| `napi-build` | `3` | Build script for cdylib output | Required build dependency |

### New Dependencies (WASM crate -- fallback only)
| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `wasm-bindgen` | `0.2` | Browser WASM bindings | Only if NAPI v3 `wasm32-wasip1-threads` fails for browser use |
| `serde-wasm-bindgen` | `0.6` | Serde-to-JsValue bridge | Pairs with wasm-bindgen |
| `js-sys` | `0.3` | JS Error type construction | Pairs with wasm-bindgen |

### Alternatives Considered
| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| NAPI-RS v3 | NAPI-RS v2 + tokio | v2 requires explicit tokio rt setup; v3 handles async natively. D-38 locks v3. |
| NAPI v3 unified WASM | wasm-bindgen separate crate | Unified avoids maintaining two binding surfaces. D-39 says try unified first. |
| tokio in NAPI crate | No tokio | v3 does not require tokio for async. Only add if CPU-bound spawn_blocking is needed. |

## Architecture Patterns

### Workspace Structure After Phase 6
```
crates/
  qwik-optimizer-oxc/       # Core transform (exists)
    src/
      lib.rs                 # transform_modules() public API
      transform.rs           # QwikTransform -- needs Hoist .s() + HMR injection
      entry_strategy.rs      # EntryPolicy trait + 5 impls (exists, needs wiring)
      code_move.rs           # Segment construction (needs EntryPolicy integration)
      ...
  qwik-optimizer-napi/       # NEW: Node.js binding (D-37)
    Cargo.toml
    build.rs                 # napi_build::setup()
    src/
      lib.rs                 # #[napi] async fn transform_modules(...)
  qwik-optimizer-wasm/       # NEW: WASM binding (D-39, fallback only)
    Cargo.toml
    src/
      lib.rs                 # #[wasm_bindgen] pub fn transform_modules(...)
```

### Pattern 1: Hoist Strategy `.s()` Post-Processing
**What:** After `inlinedQrl()` calls are created (same as Inline), `hoist_qrl_to_module_scope()` converts each to a `_noopQrl()` const declaration + `.s()` registration call.
**When to use:** When `EntryStrategy::Hoist` is active and mode is NOT `Lib`.
**Three-part pattern:**
1. `const q_symbolName = /*#__PURE__*/ _noopQrl("symbolName");` -- top-level const
2. `q_symbolName.s(fnBody);` -- module-scope registration (or `(q_X.s(value), q_X)` comma expr for non-global idents)
3. `.w([captures])` -- appended at usage site if captures exist

**SWC reference:** `transform.rs:1459-1608` (`hoist_qrl_to_module_scope`)

**Key implementation detail from SWC:**
```rust
// 1. Remove fn_body from inlinedQrl args
// 2. Convert remaining args to _noopQrl call with PURE annotation
// 3. Store const decl in extra_top_items (keyed by id)
// 4. Emit .s(fn_body) as ref_assignment (module scope) OR
//    inline comma expr (q_X.s(value), q_X) for non-global idents
// 5. Append .w([captures]) if capture_array exists
```

### Pattern 2: HMR `_useHmr()` Injection
**What:** In `EmitMode::Hmr`, inject `_useHmr(devPath)` as the first statement inside `component$` function bodies (not other `$`-calls).
**When to use:** Mode is `Hmr` AND the marker call is `component$` (detected via `is_qcomponent`).
**Implementation:** Before creating the synthetic segment for a component body, prepend the `_useHmr()` call statement with the dev_path argument.

### Pattern 3: NAPI v3 Async Binding
**What:** Thin async wrapper around `transform_modules()`.
**SWC v2 pattern (42 lines):**
```rust
// SWC used NAPI v2 with tokio:
#[js_function(1)]
fn transform_modules(ctx: CallContext) -> Result<JsObject> {
    let config: TransformModulesOptions = ctx.env.from_js_value(opts)?;
    ctx.env.execute_tokio_future(
        async move { task::spawn_blocking(move || qwik_core::transform_modules(config)).await },
        |env, result| env.to_js_value(&result),
    )
}
```
**NAPI v3 pattern:**
```rust
use napi::bindgen_prelude::*;
use napi_derive::napi;

#[napi]
pub async fn transform_modules(config: serde_json::Value) -> Result<serde_json::Value> {
    let opts: TransformModulesOptions = serde_json::from_value(config)
        .map_err(|e| napi::Error::from_reason(e.to_string()))?;
    let result = qwik_optimizer_oxc::transform_modules(opts);
    serde_json::to_value(&result)
        .map_err(|e| napi::Error::from_reason(e.to_string()))
}
```

### Pattern 4: WASM Binding (wasm-bindgen fallback)
**What:** Synchronous wrapper with serde-wasm-bindgen conversion.
**Exact SWC pattern (21 lines):**
```rust
#[wasm_bindgen]
pub fn transform_modules(config_val: JsValue) -> Result<JsValue, JsValue> {
    let config: TransformModulesOptions = from_value(config_val)?;
    let result = qwik_core::transform_modules(config)
        .map_err(|e| Error::from(JsValue::from_str(&e.to_string())))?;
    let serializer = Serializer::new().serialize_maps_as_objects(true);
    result.serialize(&serializer).map_err(JsValue::from)
}
```

### Anti-Patterns to Avoid
- **Tokio in the NAPI crate:** NAPI-RS v3 handles async natively. Do NOT add tokio as a dependency unless absolutely necessary for `spawn_blocking` (the SWC v2 pattern).
- **EmitMode checks scattered randomly:** All mode-specific branching should be clearly documented inline. The existing pattern (`let is_dev = matches!(self.mode, EmitMode::Dev | EmitMode::Hmr)`) is good -- continue using it.
- **Implementing Hoist as a separate pass:** The SWC approach integrates `hoist_qrl_to_module_scope` directly into the QRL wrapping flow (called right after `create_inline_qrl`). Keep it integrated, not as a post-pass.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| NAPI JS-Rust bridge | Manual FFI with napi-sys | `napi-rs` v3 with `#[napi]` macro | Handles type conversion, async, error propagation |
| WASM JS-Rust bridge | Manual wasm_bindgen glue | `serde-wasm-bindgen` | Handles nested object serialization correctly |
| NAPI build script | Custom build.rs logic | `napi-build` crate | Generates required NAPI registration code |
| JSON serde in NAPI | Manual JsValue conversion | `napi` `serde-json` feature | Direct serde_json::Value to/from JS |

## Common Pitfalls

### Pitfall 1: Hoist `.s()` Ordering
**What goes wrong:** `.s()` registration calls appear before the `const q_X = _noopQrl(...)` declaration, causing runtime ReferenceError.
**Why it happens:** The SWC code has `ref_assignments` as a separate list that gets drained at module scope. If drained at the wrong point, ordering breaks.
**How to avoid:** Emit `const` declarations first (via `extra_top_items`), then `.s()` calls (via `ref_assignments`), both before any export statements. The drain in `exit_program` must respect this order.
**Warning signs:** Test fixtures with Hoist strategy produce JavaScript that fails at runtime.

### Pitfall 2: Non-Global Ident `.s()` Inline Emission
**What goes wrong:** A `.s()` call for a non-globally-accessible identifier gets placed at module scope where the identifier is not in scope.
**Why it happens:** Some identifiers (e.g., function parameters, local variables) cannot be referenced at module scope. The SWC code detects this and emits `(q_X.s(value), q_X)` as a comma expression at the usage site instead.
**How to avoid:** Check `global_collect.is_global(&id)` before deciding module-scope vs inline emission. See SWC `transform.rs:1561-1591`.
**Warning signs:** Variables referenced in `.s()` calls that don't exist at module scope.

### Pitfall 3: NAPI v3 WASM SharedArrayBuffer Requirement
**What goes wrong:** WASM build works locally but fails in browsers that don't enable SharedArrayBuffer.
**Why it happens:** `wasm32-wasip1-threads` requires SharedArrayBuffer, which needs `Cross-Origin-Opener-Policy: same-origin` and `Cross-Origin-Embedder-Policy: require-corp` HTTP headers.
**How to avoid:** Validate early (D-39). If browser targets can't guarantee these headers, fall back to the wasm-bindgen approach (separate crate, no SharedArrayBuffer needed).
**Warning signs:** WASM loads but threads fail silently, or browser console shows SharedArrayBuffer errors.

### Pitfall 4: Lib Mode Must Not Extract Segments
**What goes wrong:** Library builds produce segment files that shouldn't exist.
**Why it happens:** `EmitMode::Lib` forces `is_inline_mode() == true` (already implemented), but if segment output filtering doesn't also check mode, segments leak through.
**How to avoid:** The existing `is_inline_mode()` check already returns true for Lib mode. Verify that segment modules are not emitted separately when mode is Lib.
**Warning signs:** Lib mode output contains separate `.tsx_*.js` files instead of a single `.qwik.mjs`.

### Pitfall 5: HMR Injection Only in component$ Bodies
**What goes wrong:** `_useHmr()` gets injected into `useTask$`, `$()`, or other non-component `$`-calls.
**Why it happens:** The injection check must verify BOTH `mode == Hmr` AND `is_qcomponent` (the marker is `component$`, not `componentQrl`).
**How to avoid:** Gate on the `is_qcomponent` flag from the SWC transform, which is true only when the call is directly `component$(...)`.
**Warning signs:** `_useHmr` appears in segment bodies that are not component renders.

### Pitfall 6: transform_modules Return Type Must Be Owned
**What goes wrong:** NAPI/WASM bindings fail because `transform_modules()` returns references tied to arena allocators.
**Why it happens:** OXC uses arena allocation. If the public API leaks arena references, bindings can't serialize them.
**How to avoid:** The existing `transform_modules()` returns `TransformOutput` with owned `String` fields -- this is correct. Verify no arena references leak through the public API.
**Warning signs:** Lifetime errors when compiling the NAPI/WASM crate against the core crate.

## Code Examples

### Hoist `.s()` Post-Processing (SWC Reference Pattern)
```rust
// Source: SWC transform.rs:1459-1608
// Pseudocode for OXC implementation:

fn hoist_qrl_to_module_scope(&mut self, call_expr: CallExpr) -> Expression {
    // 1. Skip in Lib mode -- return unchanged
    if matches!(self.mode, EmitMode::Lib) {
        return Expression::CallExpression(call_expr);
    }

    // 2. Extract capture array from args (if present)
    let capture_array = extract_capture_array(&mut call_expr);

    // 3. For inlinedQrl: extract fn_body (first arg), keep remaining args
    let fn_body = call_expr.arguments.remove(0);

    // 4. Get symbol name from second arg string literal
    let symbol_name = extract_symbol_name(&call_expr);

    // 5. Build _noopQrl(...remaining_args) with PURE annotation
    let noop_call = build_noop_qrl_call(&call_expr, is_dev);

    // 6. Create const q_symbolName = /*#__PURE__*/ _noopQrl("symbolName")
    let ident_name = format!("q_{}", symbol_name);
    self.extra_top_items.insert(ident_name, const_decl(noop_call));

    // 7. Emit .s(fn_body) -- module scope or inline depending on global check
    let is_non_global = !self.global_collect.is_global(&ident_name);
    if !is_non_global {
        self.ref_assignments.push(create_dot_s_call(&ident_name, fn_body));
    }

    // 8. Build result: ident or (q_X.s(value), q_X) comma expr
    let result = if is_non_global {
        comma_expr(dot_s_call, ident)
    } else {
        ident
    };

    // 9. Append .w([captures]) if captures exist
    if let Some(captures) = capture_array {
        append_dot_w(result, captures)
    } else {
        result
    }
}
```

### HMR Injection (SWC Reference Pattern)
```rust
// Source: SWC transform.rs:4117-4144
// In the component$ handling code, before creating the segment:

if is_qcomponent && self.mode == EmitMode::Hmr {
    let hmr_import = self.ensure_core_import("_useHmr");
    let dev_path = self.dev_path_or_abs_path();
    let hmr_call = expr_stmt(call_expr(hmr_import, [string_lit(dev_path)]));
    component_body = prepend_stmt(component_body, hmr_call);
}
```

### NAPI v3 Cargo.toml
```toml
# Source: D-37, D-38, napi.rs/docs
[package]
name = "qwik-optimizer-napi"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
napi = { version = "3", features = ["serde-json"] }
napi-derive = "3"
qwik-optimizer-oxc = { path = "../qwik-optimizer-oxc" }

[build-dependencies]
napi-build = "3"
```

### NAPI v3 build.rs
```rust
extern crate napi_build;

fn main() {
    napi_build::setup();
}
```

### WASM Fallback Cargo.toml
```toml
# Source: SWC wasm/Cargo.toml pattern + D-39
[package]
name = "qwik-optimizer-wasm"
version = "0.1.0"
edition = "2024"

[lib]
crate-type = ["cdylib"]

[dependencies]
js-sys = "0.3"
qwik-optimizer-oxc = { path = "../qwik-optimizer-oxc" }
serde = "1"
serde-wasm-bindgen = "0.6"
wasm-bindgen = "0.2"
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| NAPI-RS v2 + tokio `spawn_blocking` | NAPI-RS v3 native async | July 2025 (v3 release) | No tokio dependency needed in NAPI crate |
| Separate WASM crate always | NAPI v3 unified WASM via `wasm32-wasip1-threads` | July 2025 | Single codebase for both Node.js and WASM targets |
| `wasm-bindgen` for all browser WASM | `wasm32-wasip1-threads` with SharedArrayBuffer | 2025 | Requires COOP/COEP headers; not all environments support it |

**Deprecated/outdated:**
- NAPI-RS v2 `execute_tokio_future` pattern: v3's `#[napi] async fn` replaces it
- `#[module_exports]` attribute: v3 uses `#[napi]` directly on functions

## Open Questions

1. **NAPI v3 WASM browser viability**
   - What we know: NAPI v3 supports `wasm32-wasip1-threads` for WASM, but requires SharedArrayBuffer (COOP/COEP headers)
   - What's unclear: Whether Qwik's target environments (Vite dev server, Cloudflare Workers, etc.) can always set these headers
   - Recommendation: D-39 says validate early. Create a minimal NAPI v3 WASM build, test in target environments. Fall back to wasm-bindgen if headers are a problem.

2. **NAPI v3 CPU-bound async pattern**
   - What we know: v3 supports `#[napi] async fn` but `transform_modules()` is CPU-bound (not I/O)
   - What's unclear: Whether v3 automatically dispatches CPU work to a thread pool or blocks the event loop
   - Recommendation: If `#[napi] async fn` runs on the main thread, wrap in `tokio::task::spawn_blocking` (add `tokio` with `rt` feature). Test with a large input to verify non-blocking behavior.

3. **`wasm32-wasip1-threads` target availability**
   - What we know: Only `aarch64-apple-darwin` is currently installed. `wasm32-wasip1-threads` would need `rustup target add wasm32-wasip1-threads`.
   - What's unclear: Whether the target is stable in Rust 1.94.1
   - Recommendation: Check `rustup target add wasm32-wasip1-threads` availability as first validation step.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All crates | Yes | 1.94.1 (>= 1.88 MSRV) | -- |
| Node.js | NAPI binding testing | Yes | 22.1.0 | -- |
| npm | NAPI package.json | Yes | 10.7.0 | -- |
| wasm32-wasip1-threads target | NAPI v3 WASM | No (not installed) | -- | `rustup target add`, or fall back to wasm-bindgen |
| wasm-pack | WASM fallback build | No (not installed) | -- | Manual `cargo build --target wasm32-unknown-unknown` |
| cargo-napi | NAPI build CLI | Unknown | -- | `cargo install @napi-rs/cli` or use `napi build` from npx |

**Missing dependencies with no fallback:**
- None blocking -- core crate work requires only existing toolchain

**Missing dependencies with fallback:**
- `wasm32-wasip1-threads` target: `rustup target add wasm32-wasip1-threads` (or fall back to wasm-bindgen per D-39)
- `wasm-pack`: only needed if using wasm-bindgen fallback path

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in test + insta 1.x snapshot testing |
| Config file | `crates/qwik-optimizer-oxc/Cargo.toml` (dev-dependencies section) |
| Quick run command | `cargo test --manifest-path crates/qwik-optimizer-oxc/Cargo.toml` |
| Full suite command | `cargo test --workspace` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| IMPL-03 | All 7 entry strategies produce correct output | integration | `cargo test --manifest-path crates/qwik-optimizer-oxc/Cargo.toml -- entry_strategy` | Partial (unit tests exist for EntryPolicy, missing integration tests for Hoist .s()) |
| IMPL-04 | All 5 emit modes produce correct output | integration | `cargo test --manifest-path crates/qwik-optimizer-oxc/Cargo.toml -- emit_mode` | Partial (Dev/Prod QRL variants tested, missing HMR injection tests) |
| IMPL-06 | NAPI binding round-trips transform_modules | integration | `cargo test --manifest-path crates/qwik-optimizer-napi/Cargo.toml` | No (crate doesn't exist) |
| IMPL-07 | WASM binding round-trips transform_modules | integration | `cargo test --manifest-path crates/qwik-optimizer-wasm/Cargo.toml` | No (crate doesn't exist) |

### Sampling Rate
- **Per task commit:** `cargo test --manifest-path crates/qwik-optimizer-oxc/Cargo.toml`
- **Per wave merge:** `cargo test --workspace`
- **Phase gate:** Full workspace test suite green before `/gsd:verify-work`

### Wave 0 Gaps
- [ ] Integration tests for Hoist strategy `.s()` output pattern -- covers IMPL-03
- [ ] Integration tests for HMR `_useHmr()` injection -- covers IMPL-04
- [ ] Integration tests for Lib mode no-segment-extraction -- covers IMPL-04
- [ ] NAPI crate skeleton with compile test -- covers IMPL-06
- [ ] WASM crate skeleton with compile test -- covers IMPL-07
- [ ] Snapshot tests using spec fixtures for `example_inlined_entry_strategy` (Hoist pattern) and HMR mode examples

## Sources

### Primary (HIGH confidence)
- SWC NAPI binding: `/Users/scottweaver/Projects/qwik/packages/optimizer/napi/src/lib.rs` -- 42-line reference implementation
- SWC WASM binding: `/Users/scottweaver/Projects/qwik/packages/optimizer/wasm/src/lib.rs` -- 21-line reference implementation
- SWC `hoist_qrl_to_module_scope`: `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs:1459-1608` -- Hoist `.s()` pattern
- SWC HMR injection: `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs:4117-4144`
- Existing OXC core crate: `/Users/scottweaver/Projects/qwik-optimizer-next/crates/qwik-optimizer-oxc/src/` -- 475 tests, all 14 CONVs
- Qwik Optimizer Spec: `/Users/scottweaver/Projects/qwik-optimizer-next/specification/qwik-optimizer-spec.md` -- Entry Strategies section (lines 4638-4710), Emit Modes section (lines 4960-5100), CONV-14 Hoist pattern (lines 4407-4634)

### Secondary (MEDIUM confidence)
- [NAPI-RS v3 announcement](https://napi.rs/blog/announce-v3) -- v3 features, WASM support
- [NAPI-RS WebAssembly docs](https://napi.rs/docs/concepts/webassembly) -- wasm32-wasip1-threads requirements
- [NAPI-RS async fn docs](https://napi.rs/docs/concepts/async-fn) -- #[napi] async pattern
- [napi crate on crates.io](https://crates.io/crates/napi) -- version 3.8.4

### Tertiary (LOW confidence)
- NAPI v3 `spawn_blocking` behavior with CPU-bound work -- not explicitly documented; needs runtime validation

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - all dependencies are established, versions verified against existing workspace and crates.io
- Architecture: HIGH - SWC reference implementations fully readable, spec documents patterns exhaustively, OXC crate structure clear
- Pitfalls: HIGH - derived from direct SWC code reading and spec analysis, not web search
- Bindings: MEDIUM - NAPI v3 API patterns confirmed via docs, but WASM viability (SharedArrayBuffer) needs runtime validation per D-39

**Research date:** 2026-04-01
**Valid until:** 2026-05-01 (stable domain, OXC 0.123 pinned)
