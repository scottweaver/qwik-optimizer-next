# Feature Landscape

**Domain:** Qwik Optimizer (AST transformation compiler for resumability)
**Researched:** 2026-04-01
**Source confidence:** HIGH -- derived directly from SWC source code and Jack's 162 spec corpus

## Table Stakes

Features the optimizer must have or Qwik applications break. These are the 14 CONV transformation types, 7 entry strategies, 5 emit modes, and 2 binding surfaces.

### The 14 CONV Transformation Types

Every one of these is table stakes. An optimizer missing any CONV produces broken output.

| # | CONV ID | Feature | Complexity | Notes |
|---|---------|---------|------------|-------|
| 1 | CONV-01 | **Dollar Detection** -- Identify `$`-suffixed function calls (`component$`, `useTask$`, `event$`, custom `xxx$`) as marker functions requiring QRL extraction | Medium | Foundation of everything. Must detect both imported markers and locally-defined `$`-suffixed functions. Pattern: `ident.ends_with('$')` on imports from `@qwik.dev/core` and local exports. |
| 2 | CONV-02 | **QRL Wrapping** -- Replace `$`-suffixed calls with their `Qrl` counterparts (`component$` -> `componentQrl`, `useTask$` -> `useTaskQrl`) passing a `qrl()` or `inlinedQrl()` reference | High | The core transformation. Rewrites `component$(fn)` to `componentQrl(inlinedQrl(fn, "hash", [...captures]))`. Must handle dev mode variants (`qrlDEV`, `inlinedQrlDEV`) with source location metadata. |
| 3 | CONV-03 | **Capture Analysis** -- Determine which variables from enclosing scopes are referenced inside `$()` boundaries and must be serialized as captured bindings | High | Most complex analysis. Must walk the AST to find free variables, distinguish between imports (not captured), locals defined inside the `$` body (not captured), and outer-scope locals (captured). Stack-based for nested `$()` calls. 16 known deviation edge cases in Jack's impl around JSX event handler scoping. |
| 4 | CONV-04 | **Props Destructuring** -- Transform destructured component props into `_rawProps.propName` access patterns for signal reactivity tracking | Medium | Runs as a pre-pass before main transform. Converts `({count, name}) => ...` to `(_rawProps) => { /* count -> _rawProps.count */ }`. Handles rest props via `_restProps()`. Must run for all emit modes including Lib. |
| 5 | CONV-05 | **Segment Extraction** -- Extract `$()` callback bodies into separate output modules (segments) that can be lazy-loaded | High | Produces the `TransformModule` entries for each segment. Generates canonical filenames with hash suffixes. Builds segment code as JS string including imports needed by the segment body. Handles nested segments (parent-child relationships). Variable migration moves root-level vars exclusively used by one segment into that segment. |
| 6 | CONV-06 | **JSX Transform** -- Convert JSX elements to `_jsxSorted()` / `_jsxSplit()` calls with sorted static/dynamic prop separation, key generation, and special attribute handling | High | Large subsystem (~29 functions in SWC). Separates props into const (static) and var (dynamic) categories. Handles `class`/`className` normalization, `bind:value`/`bind:checked` two-way binding sugar, `q:slot`, `ref`, `children` extraction, key counter auto-generation, and `_fnSignal` optimization for inline expressions. |
| 7 | CONV-07 | **Signal Optimization (_fnSignal)** -- Convert inline JSX expressions referencing signals into `_fnSignal()` calls for fine-grained reactivity | Medium | When a JSX prop value is an expression over captured variables (e.g., `value={count.value + 1}`), generates a `_fnSignal(fn, [captures])` call that Qwik can subscribe to reactively. Uses `convert_inlined_fn` to create parameterized functions replacing captured idents with positional params (`p0`, `p1`). |
| 8 | CONV-08 | **PURE Annotations** -- Add `/* @__PURE__ */` comments to tree-shakeable calls (only `componentQrl`) | Low | Simple but precise. Only `componentQrl` gets PURE annotation because components are tree-shakeable. All other QRL wrappers (`useTaskQrl`, etc.) have side effects and must NOT be annotated. Incorrect PURE annotations break tree-shaking in bundlers. |
| 9 | CONV-09 | **Dead Branch Elimination** -- Remove unreachable code after const replacement (e.g., `if (isServer) { ... }` on client) | Medium | Depends on CONV-10 running first. After `isServer`/`isBrowser`/`isDev` are replaced with `true`/`false`, SWC's simplifier DCE removes dead branches. OXC needs equivalent dead code elimination. Client-side tree-shaking also removes side effects via `Treeshaker`. |
| 10 | CONV-10 | **Const Replacement** -- Replace `isServer`, `isBrowser`, `isDev` imports from `@qwik.dev/core/build` with boolean literals based on build config | Low | Straightforward identifier-to-literal replacement. Must handle imports from both `@qwik.dev/core/build` and `@qwik.dev/core`. Skipped in Test emit mode. Must run before CONV-09 (dead branch elimination). |
| 11 | CONV-11 | **Code Stripping** -- Remove exports and context-specific code based on `strip_exports`, `strip_ctx_name`, and `strip_event_handlers` config | Medium | Three stripping mechanisms: (1) `strip_exports` removes named exports by symbol name, replacing with throwing stubs. (2) `strip_ctx_name` removes segments whose `ctx_name` matches (e.g., strip server-only handlers). (3) `strip_event_handlers` removes all event handler segments. Used for server/client code splitting. |
| 12 | CONV-12 | **Import Rewriting** -- Rewrite module imports: strip consumed `$`-suffixed imports, add `Qrl`-suffixed equivalents, add runtime helper imports (`_jsxSorted`, `qrl`, etc.) | Medium | Multi-step: (1) Rename legacy `@builder.io/qwik` to `@qwik.dev/core` via `RenameTransform`. (2) Strip imports that were consumed by `$()` detection. (3) Add synthetic imports for QRL wrapper functions and JSX helpers. (4) Segment modules get their own import sets resolved from the global collector. |
| 13 | CONV-13 | **sync$ Serialization** -- Transform `sync$()` calls into `_qrlSync()` with the function body serialized as a string | Medium | Special case: `sync$` does not extract to a separate segment. Instead, the function body is serialized to a string and passed to `_qrlSync()` for runtime eval. Must preserve function semantics in stringified form. |
| 14 | CONV-14 | **Noop QRL Handling** -- Replace `$`-suffixed calls with `_noopQrl()` / `_noopQrlDEV()` when the callback body is empty or unused | Low | Edge case optimization. When a `$()` call has no meaningful body, emit a noop QRL instead of extracting an empty segment. |

### Entry Strategies (7 variants)

Control how extracted segments are grouped into output files. All 7 must be supported.

| Strategy | Behavior | Use Case | Complexity |
|----------|----------|----------|------------|
| **Inline** | All segments stay in the root module (`entry_segments` grouping) | Development, SSR | Low |
| **Hoist** | Same grouping as Inline, but segments are hoisted as top-level declarations with `.s()` registration calls | Production bundling (Qwik v2 default) | Medium -- hoisting logic, `.s()` call emission, QRL ID tracking |
| **Single** | All segments grouped into one output file | Simple apps, testing | Low |
| **Hook** (alias: Segment) | Each segment gets its own output file | Maximum granularity, per-route loading | Low |
| **Segment** | Identical to Hook | Renamed alias | Low |
| **Component** | Segments grouped by their root component context | Per-component bundles | Medium -- context tracking via `stack_ctxt` |
| **Smart** | Event handlers without captures get own files; everything else grouped by component | Production optimization | Medium -- conditional logic based on `SegmentKind` and `scoped_idents` |

**Key architectural note:** `Inline` and `Hoist` share the same `EntryPolicy` (InlineStrategy) but differ in post-processing. Hoist adds top-level const declarations and `.s()` registration side effects. This distinction is critical and easy to miss.

### Emit Modes (5 variants)

| Mode | Behavior | Notes |
|------|----------|-------|
| **Prod** | Full optimization: const replacement, DCE, tree-shaking, minification | Default production build |
| **Dev** | QRL wrapping with `qrlDEV`/`inlinedQrlDEV` including source locations, `isDev=true` | Development builds, HMR source info |
| **Lib** | Only props destructuring and mechanical QRL wrapping, no const replacement or DCE | Publishing Qwik libraries (pre-compiled `.qwik.mjs`) |
| **Test** | Like Prod but skips const replacement (keeps `isServer`/`isBrowser` as identifiers) | Unit testing where environment isn't known |
| **Hmr** | Like Dev but with `_useHmr()` hook injection | Hot module replacement |

### Binding Surfaces (2 required)

| Binding | Implementation | Notes |
|---------|---------------|-------|
| **NAPI** | Rust crate exposing `transform_modules` to Node.js via napi-rs | Primary binding for Vite/Rollup plugins. Uses `tokio::task::spawn_blocking` for async. Single function: takes `TransformModulesOptions` JSON, returns `TransformOutput` JSON. |
| **WASM** | Rust crate exposing `transform_modules` to browsers/edge via wasm-bindgen | Uses `serde_wasm_bindgen` for JS<->Rust marshaling. Same single-function interface. Used for playground, edge SSR. |

### Public API Contract

The wire format between JS plugins and the Rust optimizer. Must match exactly for drop-in replacement.

| Type | Purpose | Key Fields |
|------|---------|------------|
| **TransformModulesOptions** | Input config | `src_dir`, `root_dir`, `input[]` (path + code), `source_maps`, `minify`, `transpile_ts`, `transpile_jsx`, `preserve_filenames`, `entry_strategy`, `explicit_extensions`, `mode`, `scope`, `core_module`, `strip_exports`, `strip_ctx_name`, `strip_event_handlers`, `reg_ctx_name`, `is_server` |
| **TransformOutput** | Output result | `modules[]`, `diagnostics[]`, `is_type_script`, `is_jsx` |
| **TransformModule** | Per-module output | `path`, `code`, `map` (source map), `segment` (SegmentAnalysis if extracted), `is_entry`, `order` |
| **SegmentAnalysis** | Segment metadata | `origin`, `name`, `entry`, `display_name`, `hash`, `canonical_filename`, `path`, `extension`, `parent`, `ctx_kind`, `ctx_name`, `captures`, `loc`, `param_names`, `capture_names` |
| **SegmentKind** | Segment classification | `Function`, `EventHandler`, `JSXProp` |

### Source Map Generation

| Feature | Complexity | Notes |
|---------|------------|-------|
| Root module source maps | Medium | Standard codegen-based source map for the transformed root module |
| Segment source maps | Medium | Each extracted segment module needs its own source map. String-based segment construction complicates mapping. |

### Core Supporting Infrastructure

| Feature | What It Does | Complexity |
|---------|-------------|------------|
| **GlobalCollect** (collector) | First-pass analysis: collects all imports, exports, and root-level declarations for the entire module | Medium |
| **Variable Migration** (dependency_analysis) | Moves root-level variables exclusively used by one segment into that segment, removing from root module | High -- requires dependency graph analysis, export cleanup |
| **Side Effect Preservation** (add_side_effect) | For Inline/Hoist strategies, re-adds bare import statements for modules that had side effects | Low |
| **Treeshaker** (clean_side_effects) | Client-side removal of `new`/`call` expressions that appeared after simplification | Medium |
| **Path Resolution** | Canonical filename generation, relative path computation, extension mapping based on transpile flags | Medium |
| **Hash Generation** | Deterministic segment hashing for stable QRL identifiers across builds | Low |

## Differentiators

Features where the OXC implementation can improve over the SWC version.

| Feature | Value Proposition | Complexity | Notes |
|---------|-------------------|------------|-------|
| **Two-phase analyze-then-emit** | OXC's architecture enables cleaner separation of read-only analysis and mutation, preventing the scope/symbol invalidation bugs that plagued the SWC Fold-based approach | Medium | Jack's key architectural insight. SWC's ownership-based `Fold` model forces interleaved analysis and mutation. OXC's `Traverse` with `SemanticBuilder` allows complete analysis before any mutation. |
| **Arena allocation** | OXC's arena allocator eliminates per-node heap allocation, reducing GC pressure and improving cache locality | Low | Comes free with idiomatic OXC usage. No special effort needed, just don't fight the allocator. |
| **Semantic analysis via OXC Scoping** | OXC provides built-in scope analysis and symbol resolution, potentially simplifying capture analysis (CONV-03) compared to SWC's manual `GlobalCollect` + `IdentCollector` | High | Could reduce the 16 capture analysis deviations in Jack's impl. OXC's semantic layer knows variable scopes natively. |
| **Faster parsing** | OXC parser is benchmarked significantly faster than SWC parser | Low | Free improvement from the parser choice. |
| **Better error recovery** | OXC parser handles more malformed input gracefully vs SWC's stricter parsing (3 of Jack's 5 known deviations are SWC parser limitations) | Low | Resolves edge cases where SWC rejects valid-ish source that OXC handles. |
| **Built-in PURE annotation support** | OXC codegen has `expression_call_with_pure` for PURE comments, cleaner than SWC's manual comment injection | Low | Already validated in Jack's v5.0. |
| **Idiomatic `VisitMut` instead of `Fold`** | OXC's `VisitMut` with `TraverseCtx` is more ergonomic than SWC's ownership-transfer `Fold` pattern, reducing boilerplate and bug surface | Medium | Eliminates the `pending_expr_replacement` hack needed in SWC's `QwikTransform` to return different node types from fold methods. |

## Anti-Features

Things to deliberately NOT build in the OXC optimizer.

| Anti-Feature | Why Avoid | What to Do Instead |
|--------------|-----------|-------------------|
| **Byte-for-byte SWC output matching** | Cosmetic differences in whitespace, identifier naming, import ordering are acceptable. Chasing exact match wastes effort and fights OXC's codegen style. | Verify semantic equivalence: same segments extracted, same captures, same QRL hashes, same runtime behavior. |
| **Pre-compiled QRL extraction** | Extracting already-compiled `inlinedQrl()` calls (from `.qwik.mjs` library code) is a different concern from source compilation. 2 of Jack's 5 deviations are this. | Pass pre-compiled library code through unchanged. Only transform source files with `$()` markers. |
| **SWC Fold patterns in OXC** | Translating SWC's `Fold` trait usage into OXC creates unidiomatic code and prevents using OXC's strengths (semantic analysis, arena allocation). | Use OXC's native two-phase: `Traverse` for analysis, `TraverseMut` for mutation. |
| **Bundler plugin integration** | The optimizer is a standalone AST transform. Bundler-specific logic (Vite/Rollup plugin wiring, file watching, dev server) is a separate layer. | Expose clean `transform_modules` API. Let the TS/JS plugin layer handle bundler integration. |
| **TypeScript type checking** | The optimizer strips types (via `transpile_ts`) but does not type-check. | Use OXC's `typescript::strip` equivalent. Type checking is the IDE/tsc's job. |
| **Custom minifier** | SWC uses its built-in `simplify::simplifier` for DCE. Building a custom equivalent is unnecessary. | Use OXC's minifier/DCE capabilities or a post-processing step. The optimizer's job is transformation, not minification. |
| **`transform_fs` batch API** | The SWC NAPI only exposes `transform_modules`. The commented-out `transform_code` single-file variant was never shipped. Don't add APIs that don't exist. | Match the existing single API: `transform_modules` taking a vec of inputs. |
| **Interactive/incremental transformation** | The optimizer is a pure function: input config -> output modules. No caching, no incremental builds, no state between invocations. | Keep the transform stateless. Let the build tool handle caching. |

## Feature Dependencies

```
CONV-10 (Const Replacement) --> CONV-09 (Dead Branch Elimination)
    [const replacement creates dead branches that DCE removes]

CONV-04 (Props Destructuring) --> CONV-03 (Capture Analysis)
    [destructuring changes variable references that capture analysis reads]

CONV-01 (Dollar Detection) --> CONV-02 (QRL Wrapping)
    [must detect markers before wrapping them]

CONV-01 (Dollar Detection) --> CONV-05 (Segment Extraction)
    [must detect markers before extracting segments]

CONV-03 (Capture Analysis) --> CONV-02 (QRL Wrapping)
    [captures list is passed to qrl()/inlinedQrl() calls]

CONV-03 (Capture Analysis) --> CONV-07 (Signal Optimization)
    [_fnSignal needs captured variable list]

CONV-05 (Segment Extraction) --> CONV-12 (Import Rewriting)
    [segment modules need their own import resolution]

CONV-06 (JSX Transform) --> CONV-07 (Signal Optimization)
    [_fnSignal is generated during JSX prop processing]

GlobalCollect --> All CONVs
    [import/export metadata required by every transformation]

Variable Migration --> CONV-05 (Segment Extraction)
    [runs after segments are extracted to optimize root module]

Treeshaker --> CONV-09 (Dead Branch Elimination)
    [client-side cleanup runs after DCE]
```

### Execution Order (from SWC parse.rs)

The SWC optimizer executes transformations in this specific order:

```
1. Parse source code
2. Strip exports (strip_exports config)           -- CONV-11 partial
3. TypeScript strip (if transpile_ts)
4. JSX transpile (if transpile_jsx, uses React automatic runtime)
5. Rename legacy imports (RenameTransform)         -- CONV-12 partial
6. Resolver (SWC mark-based scope resolution)
7. GlobalCollect (import/export/root analysis)
8. Props Destructuring                             -- CONV-04
9. Const Replacement (if not Lib/Test mode)        -- CONV-10
10. QwikTransform Fold (main pass):                -- CONV-01,02,03,05,06,07,08,12,13,14
    - Dollar detection
    - QRL wrapping
    - Capture analysis
    - Segment extraction
    - JSX transforms
    - Signal optimization
    - PURE annotations
    - sync$ serialization
    - Noop QRL handling
    - Import rewriting
11. Treeshaker mark (if minify && client)
12. Simplifier DCE                                 -- CONV-09
13. Side effect preservation (if Inline/Hoist)
14. Treeshaker clean (if minify && client && not Inline/Hoist)
15. Variable migration
16. Export cleanup for migrated vars
17. Second DCE pass (if vars migrated)
18. Hygiene + Fixer
19. Codegen (root module + segment modules)
20. Source map generation
```

## MVP Recommendation

The optimizer is an all-or-nothing system -- there is no useful partial implementation. However, implementation can be phased by building the pipeline incrementally:

### Phase 1: Foundation (must ship together)
1. **GlobalCollect** -- import/export analysis
2. **CONV-01** (Dollar Detection) -- marker function identification
3. **CONV-03** (Capture Analysis) -- free variable detection
4. **CONV-02** (QRL Wrapping) -- the core QRL transformation
5. **CONV-05** (Segment Extraction) -- multi-module output generation
6. **CONV-12** (Import Rewriting) -- correct import sets per module
7. Source map generation
8. Public API contract (`TransformModulesOptions` / `TransformOutput`)

*Rationale:* These 6 CONVs plus infrastructure form the minimum viable optimizer that can extract segments and generate loadable QRLs. Without any one of them, output is broken.

### Phase 2: JSX + Optimization
1. **CONV-06** (JSX Transform) -- large but independent subsystem
2. **CONV-07** (Signal Optimization) -- depends on JSX
3. **CONV-04** (Props Destructuring) -- pre-pass for components
4. **CONV-08** (PURE Annotations) -- small, depends on QRL wrapping

*Rationale:* JSX transforms are a large independent subsystem. Can be developed in parallel once the QRL pipeline works.

### Phase 3: Build Modes
1. **CONV-10** (Const Replacement) -- environment booleans
2. **CONV-09** (Dead Branch Elimination) -- depends on const replacement
3. **CONV-11** (Code Stripping) -- server/client splitting
4. **CONV-13** (sync$ Serialization) -- special case
5. **CONV-14** (Noop QRL) -- edge case
6. All 5 emit modes (Dev/Prod/Lib/Test/Hmr)
7. All 7 entry strategies
8. Variable migration
9. Treeshaker

*Rationale:* Build mode differentiation and optimization passes can be layered on after core correctness is established.

### Phase 4: Binding Surfaces
1. NAPI crate
2. WASM crate

*Rationale:* Bindings are pure wiring around the core `transform_modules` function. Build last when the core is stable.

### Defer
- **Performance benchmarking against SWC** -- correctness first, measure later
- **Pre-compiled QRL handling** -- out of scope per PROJECT.md
- **Custom minification** -- use OXC's built-in capabilities

## Sources

- SWC optimizer source: `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/` (18 modules, ~18.5k LOC) -- HIGH confidence, primary source
- Jack's OXC optimizer: `/Users/scottweaver/Projects/qwik-oxc-optimizer/` (v5.0, 162 specs, 11.4k LOC) -- HIGH confidence, validated against SWC
- Jack's PROJECT.md with 14 CONV types catalogued -- HIGH confidence
- Entry strategy source: `entry_strategy.rs` with 5 EntryPolicy implementations -- HIGH confidence
- Emit modes from `parse.rs` EmitMode enum -- HIGH confidence
- NAPI binding: `napi/src/lib.rs` single-function interface -- HIGH confidence
- WASM binding: `wasm/src/lib.rs` single-function interface -- HIGH confidence
