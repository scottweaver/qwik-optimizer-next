<!-- GSD:project-start source:PROJECT.md -->
## Project

**Qwik Optimizer Specification & OXC Implementation**

A comprehensive behavioral specification of the Qwik v2 optimizer (build/v2 branch), derived from the existing SWC-based Rust implementation and Jack Shelton's 162 OXC spec files. The spec serves as the authoritative reference for building a feature-complete, idiomatic OXC-based Qwik optimizer. This project lives in the `specification/` subdirectory of the `qwik-optimizer-next` Rust workspace.

**Core Value:** The specification must be comprehensive and precise enough that an OXC implementation can be built from it without referencing the SWC source code — capturing every transformation behavior, edge case, and output contract.

### Constraints

- **Behavioral fidelity**: The OXC implementation must produce functionally equivalent output to the SWC version for all 162 test cases (cosmetic differences in formatting/naming are acceptable per Jack's precedent)
- **OXC idioms**: Implementation must use OXC's `Traverse` trait, arena allocators, `SemanticBuilder`, and `Codegen` — not SWC patterns translated to OXC APIs
- **Single spec document**: The specification is one comprehensive markdown file, not split across multiple docs
- **Foundation**: Jack's 162 spec files (`.planning/spec/*.md` in `qwik-oxc-optimizer`) are the behavioral test corpus
<!-- GSD:project-end -->

<!-- GSD:stack-start source:research/STACK.md -->
## Technology Stack

## Recommended Stack
### Core Framework: OXC (Oxidation Compiler)
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `oxc` | `0.123` | Umbrella crate: parser, AST, codegen, semantic analysis | Single dependency gates the entire JS toolchain. OXC is the de facto successor to SWC for new Rust-based JS tooling (used by Rolldown/Vite, Biome ecosystem). Umbrella crate keeps sub-crate versions synchronized. | HIGH |
| `oxc_traverse` | `0.123` | AST traversal with `Traverse` trait | Separate crate required -- not re-exported by umbrella. Provides enter/exit visitor pattern with ancestor access and `TraverseCtx` for scope/symbol queries during mutation. This is the core pattern for the optimizer's analyze-then-emit architecture. | HIGH |
| Flag | Why Excluded |
|------|-------------|
| `transformer` | Pulls in Babel-compat transpilation layers (JSX transforms, TS stripping, decorator handling). The Qwik optimizer does its own JSX transforms and import rewriting -- Babel compat adds ~40 transitive deps for zero benefit. |
| `minifier` | Not needed. Optimizer produces human-readable segments; minification is the bundler's job. |
| `mangler` | Same as minifier -- out of scope. |
| `full` | Enables everything. Massive compile time hit for features we don't use. |
| `cfg` | Control flow graph analysis. Not needed for dollar-sign detection or capture analysis. |
| `isolated_declarations` | .d.ts emit. Irrelevant to optimizer. |
### Serialization & I/O
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `serde` | `1` (features: `["derive"]`) | Serialize/deserialize all public types (`TransformOutput`, `SegmentAnalysis`, config) | Industry standard. Required by downstream NAPI/WASM bindings. No alternative worth considering. | HIGH |
| `serde_json` | `1` | JSON encoding for transform options and output | Pairs with serde. Transform options arrive as JSON from JS callers. | HIGH |
### Hashing & Encoding
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `base64` | `0.22` | Segment hash computation, source map encoding | Standard crate, stable API. Used by both SWC and OXC versions. | HIGH |
### Error Handling
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `anyhow` | `1` | Error handling with context chains | Good for application-level code (optimizer is an application, not a library consumed by arbitrary Rust callers). Context chains make debugging transform failures tractable. | HIGH |
### Path Manipulation
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `pathdiff` | `0.2` | Relative path computation between segments | Computes `./foo` style relative imports between root module and extracted segments. Tiny, focused, stable. | HIGH |
| `path-slash` | `0.2` | Cross-platform path normalization (backslash -> forward slash) | Windows compatibility for import paths. Tiny, stable. | HIGH |
### Parallelism
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `rayon` | `1` (optional, behind `parallel` feature) | Parallel module transformation | Must be optional -- WASM targets cannot use threads (unless WASI-threads, which is immature). Feature-gating rayon behind `parallel` is the correct pattern, matching Jack's implementation. | HIGH |
### Testing
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `insta` | `1` (features: `["json"]`) | Snapshot testing against 162 spec files | Perfect fit: spec files define input/output pairs, snapshot testing validates output matches expected. JSON feature enables structured comparison of `TransformOutput`. Resolved version in Jack's lock: 1.47.2. | HIGH |
### Node.js Bindings (NAPI crate -- separate workspace member)
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `napi` | `3` (features: `["serde-json"]`) | Node.js native addon bindings | NAPI-RS v3 is current (3.8.4 as of 2026-03-28). Major upgrade from SWC version's v2. V3 adds WASM compilation from same codebase via `wasm32-wasip1-threads`. Requires Rust 1.88+. | MEDIUM |
| `napi-derive` | `3` | Procedural macros for NAPI exports | Companion to napi crate. Current: 3.5.3. | MEDIUM |
| `napi-build` | `3` | Build script for NAPI compilation | Required build dependency for cdylib output. | MEDIUM |
### WASM Bindings (fallback -- separate workspace member, only if NAPI v3 WASM insufficient)
| Technology | Version | Purpose | Why | Confidence |
|------------|---------|---------|-----|------------|
| `wasm-bindgen` | `0.2` | Browser WASM bindings (fallback) | Proven pattern from SWC version. Only use if NAPI v3's `wasm32-wasip1-threads` target doesn't work for browser-only Qwik use cases. | MEDIUM |
| `serde-wasm-bindgen` | `0.6` | Serde integration for wasm-bindgen | Bridges serde types to JS values across WASM boundary. | MEDIUM |
## Version Pinning Strategy
## Workspace Structure
## Alternatives Considered
| Category | Recommended | Alternative | Why Not |
|----------|-------------|-------------|---------|
| Parser/AST | OXC | SWC | SWC is the incumbent but OXC is the explicit migration target. OXC has better arena allocation, faster parsing, and the Traverse trait is more ergonomic than SWC's Fold/VisitMut for the two-phase pattern needed. |
| Parser/AST | OXC | tree-sitter | tree-sitter is for editors, not compilers. No codegen, no semantic analysis, CST not AST. |
| Error handling | anyhow | thiserror | thiserror is better for libraries with typed errors. The optimizer is an application -- callers get string error messages via NAPI/WASM, not Rust error types. anyhow's context chains are more useful. |
| Error handling | anyhow | miette | miette adds beautiful diagnostics but the optimizer reports errors as JSON to JS callers, not terminal output. Unnecessary weight. |
| Serialization | serde + serde_json | Manual serialization | No. serde is the Rust ecosystem standard. Every downstream consumer expects it. |
| Testing | insta | manual assert_eq | 162 test cases with multi-line JS output. Manual assertions would be unmaintainable. Insta's `cargo insta review` workflow is purpose-built for this. |
| NAPI | napi-rs v3 | neon | neon is effectively abandoned. napi-rs is the ecosystem standard (used by SWC, OXC, Rspack, Rolldown). |
| Parallelism | rayon | tokio | Transforms are CPU-bound, not I/O-bound. Rayon's work-stealing is the right model. Tokio adds async complexity for no benefit. |
| Hashing | std + base64 | sha2/blake3 | The optimizer's segment hashing is a simple content hash for cache-busting, not cryptographic. If stronger hashing is needed later, add blake3 -- but the SWC version uses base64-encoded content hashes and that's sufficient. |
| Path handling | pathdiff + path-slash | manual string manipulation | Battle-tested edge case handling (Windows, trailing slashes, relative roots). Not worth reimplementing. |
## What NOT to Use
| Technology | Why Not |
|------------|---------|
| `swc_ecmascript` / `swc_common` | The entire point is migrating away from SWC. Do not bring in SWC crates even for "just one utility." |
| `oxc` `"transformer"` feature | Pulls in Babel transform infrastructure. The Qwik optimizer has its own JSX transform and import rewriting -- Babel compat layers would conflict and bloat. |
| `lazy_static` | The SWC version uses it. Use `std::sync::LazyLock` instead (stable since Rust 1.80). Zero-dep replacement. |
| `derivative` | The SWC version uses it for custom Debug/Clone. Use standard `#[derive()]` or manual impls. derivative is a proc macro that slows compilation. |
| `simple-error` | The SWC version uses it. anyhow subsumes its functionality. |
| `relative-path` | The SWC version uses it alongside pathdiff. pathdiff + path-slash cover all needs. |
| `tokio` | The SWC NAPI binding uses tokio for async runtime. NAPI v3 does not require tokio -- it handles async natively. If needed for NAPI runtime, limit to the NAPI crate only, never in core. |
## Installation
# In a fresh workspace:
# Core dependencies
# Dev dependencies
# NAPI binding dependencies (separate crate)
## Rust Edition & Toolchain
| Setting | Value | Why |
|---------|-------|-----|
| Edition | `2024` | Jack's code uses 2024 edition. Provides latest language features (let chains, async closures). Requires Rust 1.85+. |
| Resolver | `3` | Workspace dependency resolver v3 (Cargo edition 2024 default). Better feature unification. |
| MSRV | `1.88` | Required by napi-rs v3. Also ensures LazyLock, let-else, and other modern features are available. |
## Sources
- [OXC crates.io page](https://crates.io/crates/oxc) -- latest version 0.123.0 (2026-03-30)
- [OXC umbrella crate docs](https://docs.rs/oxc/latest/oxc/) -- module listing and feature flags
- [OXC feature flags](https://lib.rs/crates/oxc/features) -- full feature flag documentation
- [oxc_traverse docs](https://docs.rs/oxc_traverse/latest/oxc_traverse/) -- Traverse trait, TraverseCtx, Ancestor types
- [OXC GitHub releases](https://github.com/oxc-project/oxc/releases) -- release cadence (2-3x/week)
- [OXC DeepWiki: NAPI architecture](https://deepwiki.com/oxc-project/oxc/8-end-user-tools) -- binding structure
- [NAPI-RS v3 announcement](https://napi.rs/blog/announce-v3) -- WASM support, migration from v2
- [napi crate](https://crates.io/crates/napi) -- latest 3.8.4 (2026-03-28)
- [napi-derive crate](https://docs.rs/crate/napi/latest) -- latest 3.5.3 (2026-03-28)
- [insta snapshot testing](https://insta.rs/) -- latest 1.46.0 (lock file shows 1.47.2)
- [OXC Semantic Analysis docs](https://oxc.rs/docs/learn/parser_in_rust/semantic_analysis) -- SemanticBuilder usage
- Jack Shelton's `qwik-oxc-optimizer` Cargo.toml -- working reference implementation at OXC 0.113
- Qwik SWC optimizer `qwik-core` Cargo.toml -- incumbent dependency baseline
<!-- GSD:stack-end -->

<!-- GSD:conventions-start source:CONVENTIONS.md -->
## Conventions

Conventions not yet established. Will populate as patterns emerge during development.
<!-- GSD:conventions-end -->

<!-- GSD:architecture-start source:ARCHITECTURE.md -->
## Architecture

Architecture not yet mapped. Follow existing patterns found in the codebase.
<!-- GSD:architecture-end -->

<!-- GSD:workflow-start source:GSD defaults -->
## GSD Workflow Enforcement

Before using Edit, Write, or other file-changing tools, start work through a GSD command so planning artifacts and execution context stay in sync.

Use these entry points:
- `/gsd:quick` for small fixes, doc updates, and ad-hoc tasks
- `/gsd:debug` for investigation and bug fixing
- `/gsd:execute-phase` for planned phase work

Do not make direct repo edits outside a GSD workflow unless the user explicitly asks to bypass it.
<!-- GSD:workflow-end -->



<!-- GSD:profile-start -->
## Developer Profile

> Profile not yet configured. Run `/gsd:profile-user` to generate your developer profile.
> This section is managed by `generate-claude-profile` -- do not edit manually.
<!-- GSD:profile-end -->
