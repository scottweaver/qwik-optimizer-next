# Phase 6: Strategies, Modes & Binding Implementation - Context

**Gathered:** 2026-04-02
**Status:** Ready for planning

<domain>
## Phase Boundary

Wire all 7 entry strategies and 5 emit modes into the working core crate, then expose it through NAPI and WASM bindings. The core transform (Phase 5) already implements all 14 CONVs — this phase adds behavioral variations and native binding surfaces for drop-in SWC replacement.

</domain>

<decisions>
## Implementation Decisions

### NAPI Binding
- **D-37:** Separate `crates/qwik-optimizer-napi/` crate depending on qwik-optimizer-oxc. Clean separation of binding from transform logic.
- **D-38:** NAPI-RS v3. Newer version, handles async natively without explicit tokio setup.

### WASM Binding
- **D-39:** NAPI v3 unified approach — try `wasm32-wasip1-threads` target first for browser WASM. If browser compatibility fails, fall back to wasm-bindgen with separate crate. Validate early.

### Entry Strategy Integration
- **D-40:** Wire EntryPolicy selection into the segment grouping logic in code_move.rs via the existing `transform_modules()` public API. The Hoist strategy needs a `.s()` post-processing pass that converts `inlinedQrl()` to `_noopQrl()` + top-level `.s()` registration.

### Emit Mode Wiring
- **D-41:** Thread EmitMode through the transform pipeline to control per-CONV behavioral variations. Key mode-specific behaviors: Dev/Hmr use `qrlDEV`/`inlinedQrlDEV` with source locations, Test skips const replacement, Lib skips segment extraction and DCE, Hmr injects `_useHmr()`.

### Carrying Forward
- D-33: Crate at `crates/qwik-optimizer-oxc/`
- D-36: OXC 0.123.0 pinned

</decisions>

<canonical_refs>
## Canonical References

### Primary Reference (Spec Document)
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/qwik-optimizer-spec.md` — Entry Strategies section, Emit Modes section (with Mode × CONV table), Binding Contracts section

### Existing Implementation (Phase 5 Output)
- `/Users/scottweaver/Projects/qwik-optimizer-next/crates/qwik-optimizer-oxc/src/` — Working core crate with all 14 CONVs
- `/Users/scottweaver/Projects/qwik-optimizer-next/crates/qwik-optimizer-oxc/src/entry_strategy.rs` — 5 EntryPolicy implementations already built
- `/Users/scottweaver/Projects/qwik-optimizer-next/crates/qwik-optimizer-oxc/src/types.rs` — EmitMode, EntryStrategy enums

### Jack's Bindings (Reference)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-napi/` — Jack's NAPI v2 binding (reference for v3 migration)

### SWC Bindings (Source of Truth)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/napi/src/lib.rs` — SWC NAPI binding (42 lines)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/wasm/src/lib.rs` — SWC WASM binding (21 lines)

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- entry_strategy.rs already has all 5 EntryPolicy implementations from Phase 5
- types.rs already has EmitMode and EntryStrategy enums with serde
- transform.rs already has the QwikTransform that accepts mode/strategy config
- 211 snapshot tests validate behavioral correctness

### Integration Points
- `transform_modules()` in lib.rs is the public API — strategies and modes flow through here
- code_move.rs needs EntryPolicy integration for segment grouping
- transform.rs needs EmitMode branching for dev QRL variants, test const skip, etc.
- NAPI crate wraps `transform_modules()` with async JSON serialization
- WASM crate wraps `transform_modules()` with serde-wasm-bindgen

</code_context>

<specifics>
## Specific Ideas

- Hoist strategy `.s()` post-processing is the most complex entry strategy behavior
- EmitMode::Lib should skip segment extraction entirely (pre-compiled library code)
- EmitMode::Dev adds source location objects to QRL calls (file, line, column)
- EmitMode::Hmr injects `_useHmr()` after component detection
- NAPI v3 async: `transform_modules` should run on a blocking thread pool
- Validate WASM browser target early before committing to the unified approach

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 06-strategies-modes-binding-implementation*
*Context gathered: 2026-04-02*
