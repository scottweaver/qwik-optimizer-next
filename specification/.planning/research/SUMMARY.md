# Project Research Summary

**Project:** Qwik OXC Optimizer
**Domain:** Rust-based AST transformation compiler for Qwik resumability
**Researched:** 2026-04-01
**Confidence:** HIGH

## Executive Summary

The Qwik OXC Optimizer is a Rust library that transforms JavaScript/TypeScript source files containing Qwik's `$()` boundary markers into multi-module output suitable for resumable lazy-loading. It is a complete port of the existing SWC-based optimizer to OXC (the Oxidation Compiler), driven by the industry shift toward OXC as the successor to SWC in Rust-based JS tooling. The core challenge is not invention but accurate behavioral specification: the SWC optimizer embeds decades of implicit decisions across 18 modules and 18,500 lines of code that must be made explicit before they can be re-implemented idiomatically in OXC.

The recommended approach is a three-stage pipeline (Parse → Transform → Emit) built atop OXC 0.123 with a single `traverse_mut` pass as the transform core. Pre-traverse mutations (const replacement, export stripping) handle cases that require a stable AST before the main pass. Segment modules are assembled from string-extracted body code rather than AST construction, which avoids OXC's arena lifetime constraints. The optimizer exposes one public function (`transform_modules`) via two binding surfaces: NAPI v3 for Node.js and WASM (via wasm-bindgen fallback if NAPI v3 WASM support is insufficient for browser-only contexts). Jack Shelton's reference implementation (`qwik-oxc-optimizer`, v5.0, 162 spec files) provides verified ground truth for both the correct OXC patterns and the behavioral edge cases.

The primary risk is the complexity of capture analysis, which contains at least 8 distinct symbol categories each with different handling rules. Getting capture semantics wrong produces `ReferenceError` crashes at runtime in lazy-loaded segments — failures that are silent during static testing. A secondary risk is OXC's release cadence: the library ships 2-3 breaking releases per week pre-1.0. Both risks are well-understood and manageable with the mitigation strategies documented in the pitfalls research: write capture taxonomy first with exhaustive edge cases, and pin OXC versions with batch upgrade passes validated by the 162-spec test suite.

---

## Key Findings

### Recommended Stack

The core optimizer crate (`qwik-core`) depends exclusively on OXC and small utility crates. The OXC umbrella crate at version 0.123 provides parser, AST, semantic analysis, and codegen under a single synchronized version. `oxc_traverse` (same version, separate crate) provides the `Traverse` trait for the main pass. Serde + serde_json handle all public type serialization for NAPI/WASM wire formats. The test suite uses `insta` snapshot testing against the 162 spec files — there is no viable manual alternative for multi-line JS output comparison.

Binding crates (`qwik-napi`, `qwik-wasm`) are thin wrappers around `qwik-core` and must never be imported by core. This pattern matches both the existing SWC Qwik optimizer and OXC itself. NAPI v3 is the primary target because it promises unified Node.js + WASM from one codebase; confidence is MEDIUM pending validation that its `wasm32-wasip1-threads` target works for browser-only Qwik use cases.

**Core technologies:**
- `oxc` (0.123, features: codegen/semantic/serialize/ast_visit): Parser, AST, semantic analysis, codegen — the single dependency that gates the entire JS toolchain
- `oxc_traverse` (0.123): `Traverse` trait for single-pass `traverse_mut` with `enter_*`/`exit_*` hooks and `TraverseCtx` for scope/symbol queries
- `serde` + `serde_json`: Public type serialization for all NAPI/WASM wire formats; required by all downstream consumers
- `insta` (json feature): Snapshot testing for 162 spec file input/output pairs; the `cargo insta review` workflow is purpose-built for this use case
- `napi` v3 + `napi-derive` v3: Node.js native addon bindings (current: 3.8.4); MEDIUM confidence on WASM target
- `anyhow`: Error handling with context chains; correct for application-level code where callers receive string errors via NAPI/WASM
- `rayon` (optional, behind `parallel` feature): File-level parallelism; must be feature-gated because WASM targets cannot use threads

**Do not use:** `transformer` OXC feature flag (Babel compat, 40+ transitive deps), `lazy_static` (use `std::sync::LazyLock`), `derivative`, `simple-error`, `tokio` in core, or any SWC crates.

**Rust toolchain:** Edition 2024, workspace resolver 3, MSRV 1.88 (required by napi-rs v3).

### Expected Features

The optimizer is an all-or-nothing system with no useful partial implementation. The 14 CONV transformation types are all table stakes — missing any one produces broken output. Entry strategies and emit modes are fully enumerated and equally required.

**Must have (table stakes — all 14 CONVs):**
- CONV-01/02/03: Dollar detection, QRL wrapping, and capture analysis — the core pipeline; every other feature depends on these
- CONV-04: Props destructuring — must run as a pre-pass before capture analysis to avoid wrong captures
- CONV-05: Segment extraction — produces the multi-module output that enables lazy loading
- CONV-06/07: JSX transform and signal optimization (`_jsxSorted`, `_jsxSplit`, `_fnSignal`) — large independent subsystem (~29 functions)
- CONV-08: PURE annotations — `componentQrl` only; all other `*Qrl` wrappers have side effects and must NOT be annotated
- CONV-09/10: Dead branch elimination and const replacement (`isServer`/`isBrowser`/`isDev`) — const replacement must run first as a pre-pass
- CONV-11/12/13/14: Code stripping, import rewriting, sync$ serialization, noop QRL handling
- All 7 entry strategies (Inline, Hoist, Single, Hook/Segment, Component, Smart)
- All 5 emit modes (Prod, Dev, Lib, Test, Hmr)
- Both binding surfaces: NAPI (primary) and WASM (browser/edge)
- Full public API contract: `TransformModulesOptions` → `TransformOutput` with `TransformModule[]` and `SegmentAnalysis`
- Source map generation for both root modules and extracted segment modules

**Should have (OXC-native improvements over SWC):**
- Semantic capture analysis via OXC `Scoping` — reduces the 16 known capture analysis deviations in Jack's impl
- Idiomatic two-phase analyze-then-mutate (analysis in `enter_*`, mutation in `exit_*`) — eliminates SWC's `pending_expr_replacement` hack
- Better error recovery — OXC handles more malformed input that SWC rejects

**Defer (not in scope per PROJECT.md):**
- Pre-compiled QRL handling (`.qwik.mjs` library code passthrough — 2 of Jack's 5 known deviations)
- Performance benchmarking against SWC (correctness first)
- Custom minification (use OXC built-in DCE)
- Any bundler plugin integration (Vite/Rollup wiring is a separate layer)
- Incremental/cached transformation (the optimizer is a stateless pure function)

### Architecture Approach

The optimizer is structured as a three-stage pipeline operating independently per input file, parallelizable across files via rayon. Stage 1 (Parse) runs `oxc_parser` to produce `Program<'a>` on a per-file arena, then `SemanticBuilder` to produce `Scoping`, then `collector::collect()` for import/export metadata, then pre-traverse mutations (const replacement, export stripping). Stage 2 (Transform) runs a single `traverse_mut` pass with `QwikTransform: impl Traverse` that analyzes in `enter_*` hooks and mutates in `exit_*` hooks, extracting segment body code via span slicing of the original source string. Stage 3 (Emit) codegens the mutated main module and builds segment modules from extracted string data with re-parse for source map generation.

**Major components (in build order):**

Layer 1 — Foundation:
1. `types.rs` — All shared data structures: options, output types, `SegmentData`, `CollectResult`, `Diagnostic`
2. `words.rs` — String constants (`@qwik.dev/core`, QRL function names, etc.)
3. `hash.rs` — Deterministic content-based segment name hashing
4. `diagnostics.rs` — Diagnostic creation helpers
5. `is_const.rs` — Expression constness analysis for signal optimization

Layer 2 — Parse + Collect:
6. `parse.rs` — `Parser` + `SemanticBuilder` wrapper
7. `collector.rs` — Import/export/root-declaration analysis
8. `entry_strategy.rs` — `EntryStrategy` enum and segment grouping logic

Layer 3 — Pre-Traverse Mutations:
9. `const_replace.rs` — `isServer`/`isDev`/`isBrowser` → boolean literals
10. `filter_exports.rs` — `strip_exports` config removal

Layer 4 — Core Transform:
11. `props_destructuring.rs` — Props reconstruction for signal forwarding
12. `jsx_transform.rs` — JSX rewriting (`_jsxSorted`, `_jsxSplit`, signal wrapping)
13. `import_rewrite.rs` — Import addition/removal with `ImportTracker` accumulator
14. `transform.rs` — `QwikTransform: impl Traverse` — the integration heart

Layer 5 — Emit:
15. `code_move.rs` — Segment module construction from extracted string data
16. `emit.rs` — `oxc::codegen::Codegen` wrapper with source map support

Layer 6 — Integration:
17. `lib.rs` — Public API `transform_modules()`, per-file orchestration loop

### Critical Pitfalls

1. **Capture taxonomy incomplete** — Jack's Phase 24 identified 8 distinct capture categories (module-level decls → self-imports, user imports → re-emitted imports, loop vars, destructured props, TS type-only imports, shadowed vars, hoisted functions, `AssignmentPattern` defaults). Missing any category produces runtime `ReferenceError` crashes. Mitigation: write the capture classification table as the very first spec section; count categories — if fewer than 8, the spec is incomplete.

2. **Module-level declaration to self-import reclassification** — The single most impactful behavioral distinction: when a nested segment references a module-level declaration, SWC generates a self-import (`import { X } from "./module_stem"`) NOT a capture entry. This is invisible in SWC source code and caused 46 of 52 missing-import-used deviations in Jack's Phase 24. Mitigation: spec must have a dedicated "self-import" section, explicitly distinguish top-level vs nested segment behavior, and treat reclassification as a post-processing step separate from core capture analysis.

3. **Specifying OXC patterns using SWC idioms** — Using SWC concepts (fold, SyntaxContext, ownership transfer, `std::mem::replace`) in the spec guides implementers toward patterns that fight OXC's borrow checker. Key OXC-native requirements: string-based segment code construction (not AST construction), parse-roundtrip for JSX lambda serialization (casting between `JSXExpression` and `Expression` is unsound despite shared variant types), deferred import insertion in `exit_program`, span-based tracking. Mitigation: every transformation section must have a behavioral contract + separate OXC migration notes; grep spec for SWC terms before review.

4. **Parse error bailout in capture analysis** — Bailing on any OXC parser error during lambda body analysis drops captures for valid code (e.g., `await` in non-async context produces semantic errors but a valid AST). Mitigation: spec must state "proceed on semantic errors, bail only on empty program body"; include `() => await someFunction()` as a mandatory test vector.

5. **PURE annotation scope too broad** — Annotating all `*Qrl` wrappers as `/*#__PURE__*/` causes bundlers to silently delete `useTask$`, `useVisibleTask$`, and `server$` hooks in production. Only `componentQrl` is tree-shakeable. Mitigation: spec must have a whitelist table (not a blanket rule), and the test suite must include a production build that verifies hook execution.

---

## Implications for Roadmap

Based on combined research, the optimizer requires a 4-phase implementation. The FEATURES.md phase structure maps directly to implementation layers and is validated by the architectural dependency graph.

### Phase 1: Specification Foundation — Capture Taxonomy and Core Pipeline

**Rationale:** The optimizer cannot be partially implemented — broken output from any missing CONV is indistinguishable from a catastrophic failure. The specification must be written before implementation begins, and capture semantics must be specified before any other transformation because every other CONV depends on it. Phase 1 delivers a complete behavioral spec for the core pipeline with sufficient test vectors to drive TDD.

**Delivers:** Complete behavioral specification for CONVs 01-03+05+12 (dollar detection, QRL wrapping, capture analysis, segment extraction, import rewriting), the GlobalCollect analysis infrastructure, the public API contract, and the 8-category capture taxonomy with self-import reclassification documented as a first-class concept. All spec inputs validated through OXC parser.

**Addresses:** CONV-01, CONV-02, CONV-03, CONV-05, CONV-12, all entry strategy mechanics, public type definitions, source map requirements.

**Avoids:** Pitfalls 1, 2, 3, 4, 14, 15 — capture taxonomy completeness, self-import reclassification, OXC-native patterns, parse error tolerance, behavioral language, multi-boundary test vectors.

**Research flag:** STANDARD — ground truth from Jack's implementation and SWC source is well-documented. No additional research needed.

### Phase 2: Implementation Core — Parse, Collect, Transform, Emit

**Rationale:** Once the behavioral spec is complete and test vectors are in place, the core implementation can proceed in dependency order (Layers 1-6 of the architecture). This phase implements the minimum viable optimizer that can extract segments and generate loadable QRLs. The 162-spec test suite drives correctness validation.

**Delivers:** Working `qwik-core` crate with the full 17-module architecture, passing the subset of spec files that exercise CONVs 01-03+05+12. Output validates against spec test vectors via `insta` snapshot tests.

**Uses:** OXC 0.123 (umbrella: codegen/semantic/serialize/ast_visit), `oxc_traverse` 0.123, `serde`/`serde_json`, `anyhow`, `base64`, `pathdiff`, `path-slash`, `insta`.

**Implements:** All 6 architectural layers (types → parse → pre-traverse → core transform → emit → lib orchestration), per-file arena allocation pattern, span-based segment body extraction, `ImportTracker` accumulator pattern.

**Avoids:** Pitfalls 6 (exhaustive `BindingPattern` matching — no wildcard `_`), 9 (string-based segment construction, not AST construction).

**Research flag:** STANDARD — Jack's reference implementation is the primary guide; no additional research needed. Flag: validate NAPI v3 WASM browser target early in this phase to avoid late-stage binding surprises.

### Phase 3: JSX + Props + Build Modes

**Rationale:** JSX is a large independent subsystem (~29 functions) that can be developed in parallel once the core QRL pipeline is stable. Build mode differentiation (emit modes, const replacement, DCE) and props destructuring are layered additions that do not change core segment extraction semantics.

**Delivers:** Complete optimizer passing all 162 spec files. JSX transform (`_jsxSorted`, `_jsxSplit`, signal optimization via `_fnSignal`, key generation), props destructuring pre-pass, const replacement + dead branch elimination, code stripping, `sync$` serialization, noop QRL handling. All 5 emit modes and all 7 entry strategies functional. Variable migration and treeshaker passes complete.

**Addresses:** CONV-04, CONV-06, CONV-07, CONV-08, CONV-09, CONV-10, CONV-11, CONV-13, CONV-14. All entry strategies (Inline/Hoist/Single/Hook/Component/Smart). All emit modes (Prod/Dev/Lib/Test/Hmr).

**Avoids:** Pitfall 8 (parse-roundtrip for JSX lambda serialization — no `JSXExpression`↔`Expression` casting), Pitfall 5 (display name collision via `wrapper_callee_name` context), Pitfall 10 (PURE annotation whitelist: `componentQrl` only), Pitfall 7 (transformation ordering DAG: props-destructuring → capture-analysis, const-replace → DCE).

**Research flag:** NEEDS RESEARCH for the JSX subsystem specifically — the JSX transform is the largest single component (~29 functions, 16 known edge cases around JSX event handler scoping). Consider a targeted research-phase pass on JSX event handler capture scoping rules before implementing `jsx_transform.rs`.

### Phase 4: Binding Surfaces and Integration

**Rationale:** NAPI and WASM bindings are pure wiring around the stable `transform_modules` function. Building them last ensures the API contract is frozen before exposing it to consumers. This phase also validates end-to-end behavior via Vite/Rollup plugin integration tests.

**Delivers:** `qwik-napi` crate with NAPI v3 Node.js bindings and `qwik-wasm` crate (wasm-bindgen or NAPI v3 WASM per earlier validation). Workspace builds for both targets. Integration test confirming drop-in replacement for existing SWC optimizer in a Qwik application.

**Uses:** `napi` v3 (features: serde-json), `napi-derive` v3, `napi-build` v3. Fallback: `wasm-bindgen` 0.2 + `serde-wasm-bindgen` 0.6 if NAPI v3 WASM is insufficient for browser contexts.

**Implements:** Thin binding wrappers only. Core crate must not gain any dependency on binding crates.

**Research flag:** NEEDS RESEARCH for NAPI v3 WASM browser target validation. The `wasm32-wasip1-threads` target is relatively new and the Qwik playground/edge SSR use case (browser-only, no WASI runtime) may require the proven `wasm-bindgen` fallback. This should be validated early in Phase 2 and confirmed in Phase 4.

### Phase Ordering Rationale

- Phase 1 before Phase 2: A spec with test vectors is required for TDD; the capture taxonomy in particular cannot be reverse-engineered from implementation (it took Jack an 11-plan bug-fix campaign to discover all edge cases in post-hoc analysis).
- Phase 2 before Phase 3: The QRL pipeline must be proven correct before adding JSX complexity; JSX event handlers are themselves `$()` boundaries and require the capture system to work correctly.
- Phase 3 before Phase 4: Bindings expose the final API; the API contract must be stable (all CONVs passing) before it is published via NAPI/WASM.
- Within Phase 2, build in Layer order (1 → 2 → 3 → 4 → 5 → 6): each layer has compile-time dependencies on previous layers; the dependency graph is strict.
- JSX subsystem within Phase 3 can be developed independently in parallel with build mode work, as both depend on the Phase 2 core but not on each other.

### Research Flags

**Needs additional research during planning:**
- **Phase 3 (JSX transform):** 16 known edge cases around JSX event handler capture scoping; the `jsx_transform.rs` module has the most complexity per LOC in the entire system. A `/gsd:research-phase` pass on JSX event handler capture rules and `_fnSignal` generation is recommended before writing that spec section.
- **Phase 4 (NAPI v3 WASM):** Browser-only WASM use case (playground, edge SSR) may not be supported by `wasm32-wasip1-threads`. Needs validation against an actual Qwik app before committing to the NAPI v3 WASM path.

**Standard patterns (skip research-phase):**
- **Phase 1 (spec foundation):** Ground truth is complete — Jack's 162 spec files plus Phase 24 FINAL-AUDIT cover all edge cases. No additional research needed.
- **Phase 2 (core implementation):** OXC patterns are well-documented in Jack's implementation + OXC docs.rs. `traverse_mut` API is stable.
- **Phase 4 (NAPI binding):** NAPI v3 is the ecosystem standard with clear migration docs. Binding pattern is identical to OXC, Rspack, and Rolldown.

---

## Confidence Assessment

| Area | Confidence | Notes |
|------|------------|-------|
| Stack | HIGH | All core dependencies verified against crates.io + Jack's lock file. NAPI v3 MEDIUM due to WASM browser target uncertainty. |
| Features | HIGH | Derived directly from SWC source code (18 modules, 18.5K LOC) and Jack's validated 162-spec corpus. Feature set is exhaustive. |
| Architecture | HIGH | Verified against Jack's working OXC implementation (v5.0) and OXC docs.rs v0.99.0. Pragmatic single-pass pattern confirmed over idealized two-phase. |
| Pitfalls | HIGH | Derived from Jack's Phase 24 bug-fix campaign (293 → 4 runtime-breaking deviations). Each pitfall has a specific commit that fixed it. |

**Overall confidence:** HIGH

### Gaps to Address

- **NAPI v3 WASM for browser:** The `wasm32-wasip1-threads` target may require a WASI runtime, making it unsuitable for browser-only Qwik use cases (playground, edge SSR). Validate against a minimal Qwik browser test early in Phase 2, before Phase 4 binding work begins. Fallback is the proven `wasm-bindgen` + `serde-wasm-bindgen` pattern from the SWC version.

- **JSX event handler capture scoping edge cases:** Jack's implementation notes 16 known deviations specifically around JSX event handler scoping. The Phase 24 campaign focused on non-JSX capture issues. It is possible JSX-specific edge cases remain undocumented. Recommend a targeted research pass when writing the JSX transform spec section.

- **Display name depth algorithm:** SWC includes full JSX element hierarchy (e.g., `Foo_component_div_button_q_e_click`) while OXC produces shorter paths (e.g., `Foo_component_button_q_e_click`). This affects 63 naming-convention pairs and cascades into hash differences. The spec must make an explicit decision: match SWC exactly (full hierarchy) or accept the shorter OXC-native form. This is a decision, not a gap in knowledge — but it must be made in Phase 1 before any segment naming tests are written.

- **OXC version upgrade path:** OXC releases 2-3x per week with breaking changes. The current pin at 0.123 will be stale by the time implementation begins. The team should establish an OXC upgrade cadence (monthly batches, validated by full 162-spec run) and designate an owner for tracking OXC breaking changes.

---

## Sources

### Primary (HIGH confidence)

- Jack Shelton's `qwik-oxc-optimizer` v5.0 — working OXC implementation, 162 spec files, Phase 24 FINAL-AUDIT (293 → 4 runtime-breaking deviations), 28 validated key decisions
- SWC optimizer source: `qwik/packages/optimizer/core/src/` — 18 modules, ~18.5K LOC, ground truth for behavioral spec
- [OXC crates.io](https://crates.io/crates/oxc) — latest version 0.123.0 (2026-03-30)
- [oxc_traverse docs.rs v0.99.0](https://docs.rs/oxc_traverse/0.99.0/oxc_traverse/) — `traverse_mut` signature, `Traverse` trait, `TraverseCtx`
- [oxc_semantic docs.rs](https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.Scoping.html) — `Scoping` struct, scope/symbol/reference tables
- [napi crate](https://crates.io/crates/napi) — latest 3.8.4 (2026-03-28)

### Secondary (MEDIUM confidence)

- [NAPI-RS v3 announcement](https://napi.rs/blog/announce-v3) — WASM compilation via `wasm32-wasip1-threads`, migration from v2
- [OXC transformer alpha blog](https://oxc.rs/blog/2024-09-29-transformer-alpha) — performance benchmarks, design rationale
- [OXC DeepWiki NAPI architecture](https://deepwiki.com/oxc-project/oxc/8-end-user-tools) — binding structure pattern

### Tertiary (LOW confidence)

- [OXC statement manipulation issue #6993](https://github.com/oxc-project/oxc/issues/6993) — statement insertion/deletion patterns; needs validation against current API

---
*Research completed: 2026-04-01*
*Ready for roadmap: yes*
