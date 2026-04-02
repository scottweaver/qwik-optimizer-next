# Phase 4: Public API, Bindings & Cross-Cutting Specification - Research

**Researched:** 2026-04-01
**Domain:** Public API type definitions, NAPI/WASM binding contracts, OXC migration guide, representative examples appendix
**Confidence:** HIGH

## Summary

Phase 4 completes the Qwik Optimizer specification by adding four final sections to the existing 5,254-line document: (1) public API types as Rust struct definitions, (2) minimal NAPI and WASM binding contracts, (3) an OXC Migration Guide appendix, and (4) a Representative Examples appendix with 20+ curated snapshots covering all 14 CONVs.

All public types have been extracted from the SWC source (`lib.rs`, `parse.rs`, `transform.rs`, `entry_strategy.rs`, `utils.rs`) and cross-referenced against Jack's OXC types (`types.rs`) and the TypeScript definitions (`types.ts`). The types are stable across all three implementations with only minor differences (SWC uses `Atom`, Jack uses `String`; SWC has `SegmentKind` with 2 variants, Jack's `CtxKind` has 3 including `JSXProp`). The NAPI binding is a 42-line thin wrapper using tokio spawn_blocking; the WASM binding is 21 lines of serde-wasm-bindgen serialization. Both are straightforward to document minimally per D-32.

**Primary recommendation:** Structure the phase as four independent spec sections written sequentially: API types first (foundational reference for other sections), then binding contracts (reference the types), then OXC migration appendix, then representative examples appendix (the most labor-intensive, requiring careful snapshot selection).

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-29:** OXC migration notes in a dedicated appendix section ("OXC Migration Guide") at the end of the spec. Grouped by transformation with SWC->OXC pattern mapping. Does NOT modify existing Phases 1-3 content. Uses Scott's earlier OXC conversion (per D-07) for concrete pattern examples.
- **D-30:** A "Representative Examples" appendix section with 20+ curated examples from Jack's 162 snapshots, covering all 14 CONVs. Complements the inline examples already in each CONV section from Phases 1-3.
- **D-31:** TransformModulesOptions, TransformOutput, and related types documented as actual Rust struct type definitions with doc comments. Precise and directly useful for implementation.
- **D-32:** NAPI and WASM bindings documented minimally: function signature, JSON serialization format, async behavior (NAPI), platform-specific gotchas. These are thin wrappers -- don't over-document.

### Carrying Forward
- D-01: Pipeline-ordered structure
- D-05: SWC is source of truth
- D-06: SWC source references
- D-07: Scott's OXC conversion for migration examples
- D-13/16: Example format with snapshot names

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SPEC-18 | TransformModulesOptions (all config fields with types, defaults, valid values) | Complete type catalog extracted from SWC lib.rs (16 fields), cross-referenced with Jack's types.rs and types.ts. All defaults documented. |
| SPEC-19 | TransformOutput, TransformModule, SegmentAnalysis, SegmentKind (all output fields) | Complete type catalog from SWC parse.rs and transform.rs. SegmentAnalysis has 15 fields, TransformModule has 6 fields (5 serialized), TransformOutput has 4 fields. |
| SPEC-20 | Diagnostic type and error/warning categories | Diagnostic struct from utils.rs (7 fields), DiagnosticCategory enum (3 variants), DiagnosticScope enum (1 variant). Three error codes identified from errors.rs (C02, C03, C05). |
| SPEC-26 | NAPI binding contract | SWC NAPI binding fully read (42 lines). Uses napi v2 `js_function(1)` pattern, tokio `spawn_blocking`, `env.from_js_value`/`env.to_js_value` for serde. |
| SPEC-27 | WASM binding contract | SWC WASM binding fully read (21 lines). Uses wasm-bindgen, serde-wasm-bindgen `from_value`/`Serializer`, synchronous (no async). |
| SPEC-28 | OXC migration notes per transformation | 6 key SWC->OXC pattern divergences cataloged from PITFALLS.md, ARCHITECTURE.md, and Scott's transform.rs. Covers Fold->Traverse, SyntaxContext->Scoping, ownership->arena, string-based code_move, deferred imports, parse-roundtrip. |
| SPEC-29 | Representative input/output examples (minimum 20, all 14 CONVs) | 201 snapshots available (not 162 -- the count grew). Curated selection of 24 snapshots documented below covering all 14 CONVs with diversity across strategies, modes, and edge cases. |
</phase_requirements>

## Complete Public API Type Catalog

### Source Files Read

| Source | Path | What Was Extracted |
|--------|------|--------------------|
| SWC lib.rs | `qwik/packages/optimizer/core/src/lib.rs` | `TransformModulesOptions`, `TransformModuleInput`, `transform_modules()` |
| SWC parse.rs | `qwik/packages/optimizer/core/src/parse.rs` | `TransformOutput`, `TransformModule`, `SegmentAnalysis`, `MinifyMode`, `EmitMode`, `ErrorBuffer`, `QwikManifest`, `QwikBundle` |
| SWC transform.rs | `qwik/packages/optimizer/core/src/transform.rs` | `SegmentKind`, `Segment` (internal), `SegmentData` (internal), `Captures` (internal) |
| SWC entry_strategy.rs | `qwik/packages/optimizer/core/src/entry_strategy.rs` | `EntryStrategy`, `EntryPolicy` trait |
| SWC utils.rs | `qwik/packages/optimizer/core/src/utils.rs` | `Diagnostic`, `DiagnosticCategory`, `DiagnosticScope`, `SourceLocation` |
| SWC errors.rs | `qwik/packages/optimizer/core/src/errors.rs` | `Error` enum (3 variants: FunctionReference=C02, CanNotCapture=C03, MissingQrlImplementation=C05) |
| SWC types.ts | `qwik/packages/optimizer/src/types.ts` | TypeScript contract (JS-facing API) |
| Jack's types.rs | `qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/types.rs` | OXC equivalents with doc comments |

### Type Inventory: Input Types (Config)

**TransformModulesOptions** (16 fields)
| Field | Rust Type (SWC) | JSON (camelCase) | Default | Notes |
|-------|-----------------|------------------|---------|-------|
| `src_dir` | `String` | `srcDir` | (required) | Base directory for resolving relative paths |
| `root_dir` | `Option<String>` | `rootDir` | `None` | Monorepo root override |
| `input` | `Vec<TransformModuleInput>` | `input` | (required) | Files to transform |
| `source_maps` | `bool` | `sourceMaps` | (required, no default in SWC) | Jack defaults to `true` |
| `minify` | `MinifyMode` | `minify` | (required in SWC) | Jack defaults to `Simplify` |
| `transpile_ts` | `bool` | `transpileTs` | (required in SWC) | Strip TS type annotations |
| `transpile_jsx` | `bool` | `transpileJsx` | (required in SWC) | Transpile JSX to function calls |
| `preserve_filenames` | `bool` | `preserveFilenames` | (required in SWC) | Preserve original filenames in output paths |
| `entry_strategy` | `EntryStrategy` | `entryStrategy` | (required in SWC) | How to split extracted segments |
| `explicit_extensions` | `bool` | `explicitExtensions` | (required in SWC) | Use explicit file extensions in imports |
| `mode` | `EmitMode` | `mode` | (required in SWC) | Build target output mode |
| `scope` | `Option<String>` | `scope` | `None` | Optional scope prefix for segment names |
| `core_module` | `Option<String>` | `coreModule` | `None` (defaults to `@qwik.dev/core` internally) | Override core import path |
| `strip_exports` | `Option<Vec<Atom>>` | `stripExports` | `None` | Export names to strip |
| `strip_ctx_name` | `Option<Vec<Atom>>` | `stripCtxName` | `None` | Context names to strip |
| `strip_event_handlers` | `bool` | `stripEventHandlers` | (required in SWC) | Strip event handler registrations |
| `reg_ctx_name` | `Option<Vec<Atom>>` | `regCtxName` | `None` | Context names to register |
| `is_server` | `Option<bool>` | `isServer` | `None` (defaults to `true` when `None`) | SSR targeting flag |

**Key difference SWC vs Jack:** SWC does not implement `Default` for `TransformModulesOptions` (all fields are required in the struct). Jack adds defaults via `#[serde(default)]` annotations for ergonomic JSON deserialization. The spec should document the SWC-faithful types but note sensible defaults for OXC implementation.

**TransformModuleInput** (3 fields)
| Field | Rust Type | JSON | Notes |
|-------|-----------|------|-------|
| `path` | `String` | `path` | File path relative to src_dir |
| `dev_path` | `Option<String>` | `devPath` | HMR/dev mode path override |
| `code` | `String` | `code` | Source code content |

### Type Inventory: Output Types

**TransformOutput** (4 fields)
| Field | Rust Type | JSON | Notes |
|-------|-----------|------|-------|
| `modules` | `Vec<TransformModule>` | `modules` | All output modules (root + segments) |
| `diagnostics` | `Vec<Diagnostic>` | `diagnostics` | Errors and warnings |
| `is_type_script` | `bool` | `isTypeScript` | Whether input was TypeScript |
| `is_jsx` | `bool` | `isJsx` | Whether input contained JSX |

**TransformModule** (6 fields, 5 serialized)
| Field | Rust Type | JSON | Notes |
|-------|-----------|------|-------|
| `path` | `String` | `path` | Output file path |
| `code` | `String` | `code` | Generated JS source |
| `map` | `Option<String>` | `map` | Source map JSON string |
| `segment` | `Option<SegmentAnalysis>` | `segment` | Metadata, present only for segments |
| `is_entry` | `bool` | `isEntry` | Whether this is an entry point |
| `order` | `u64` | (skip_serializing) | Internal sort order, not in JSON output |

**SegmentAnalysis** (15 fields)
| Field | Rust Type | JSON | Notes |
|-------|-----------|------|-------|
| `origin` | `Atom` | `origin` | Source file |
| `name` | `Atom` | `name` | Full segment name with hash |
| `entry` | `Option<Atom>` | `entry` | Entry point name, null if not named |
| `display_name` | `Atom` | `displayName` | Human-readable name |
| `hash` | `Atom` | `hash` | 11-char hash |
| `canonical_filename` | `Atom` | `canonicalFilename` | Segment module filename |
| `path` | `Atom` | `path` | Output path prefix |
| `extension` | `Atom` | `extension` | File extension |
| `parent` | `Option<Atom>` | `parent` | Parent segment if nested |
| `ctx_kind` | `SegmentKind` | `ctxKind` | `"function"` or `"eventHandler"` |
| `ctx_name` | `Atom` | `ctxName` | The `$`-suffixed callee name |
| `captures` | `bool` | `captures` | Whether captures outer scope vars |
| `loc` | `(u32, u32)` | `loc` | Source location as `[start, end]` |
| `param_names` | `Option<Vec<Atom>>` | `paramNames` | Param names (skip if None) |
| `capture_names` | `Option<Vec<Atom>>` | `captureNames` | Captured var names (skip if None) |

### Type Inventory: Enums

**EntryStrategy** (7 variants) -- SWC uses flat enum, Jack uses `#[serde(tag = "type")]` tagged
| Variant | JSON | Notes |
|---------|------|-------|
| `Inline` | `"inline"` | Segments stay in same file |
| `Hoist` | `"hoist"` | Segments hoisted to top |
| `Single` | `"single"` | All segments in one file |
| `Hook` | `"hook"` | Deprecated alias for Segment |
| `Segment` | `"segment"` | Each segment separate file |
| `Component` | `"component"` | Group by parent component |
| `Smart` | `"smart"` | Automatic best choice |

**SWC serialization note:** SWC uses `#[serde(rename_all = "camelCase")]` on the EntryStrategy enum, which means variants serialize as simple strings: `"inline"`, `"hoist"`, etc. However, the TypeScript types.ts defines EntryStrategy as a tagged union with `{ type: "inline" }`. Jack's implementation uses `#[serde(tag = "type")]` to match the TypeScript contract. The spec must document the wire format as the tagged object form (matching types.ts) since that is what JS callers send.

**SegmentKind** (2 variants in SWC, 3 in Jack's OXC)
| Variant | JSON | Notes |
|---------|------|-------|
| `Function` | `"function"` | component$, useTask$, $, etc. |
| `EventHandler` | `"eventHandler"` | onClick$, onInput$, etc. |
| `JSXProp` | `"jsxProp"` | Jack added for JSX prop expressions (not in SWC) |

**EmitMode** (5 variants)
| Variant | JSON | Notes |
|---------|------|-------|
| `Prod` | `"prod"` | Production: short s_{hash} symbol names |
| `Dev` | `"dev"` | Development: debug info, qrlDEV variants |
| `Lib` | `"lib"` | Library: standard output |
| `Test` | `"test"` | Test: no const replacement |
| `Hmr` | `"hmr"` | Hot Module Replacement |

**MinifyMode** (2 variants)
| Variant | JSON |
|---------|------|
| `Simplify` | `"simplify"` |
| `None` | `"none"` |

### Type Inventory: Diagnostics

**Diagnostic** (7 fields)
| Field | Rust Type | JSON | Notes |
|-------|-----------|------|-------|
| `category` | `DiagnosticCategory` | `category` | Severity level |
| `code` | `Option<String>` | `code` | Machine-readable code (e.g., `"C02"`) |
| `file` | `Atom` | `file` | Source file path |
| `message` | `String` | `message` | Human-readable message |
| `highlights` | `Option<Vec<SourceLocation>>` | `highlights` | Source code highlight ranges |
| `suggestions` | `Option<Vec<String>>` | `suggestions` | Fix suggestions |
| `scope` | `DiagnosticScope` | `scope` | Always `"optimizer"` |

**DiagnosticCategory** (3 variants)
| Variant | JSON | Semantics |
|---------|------|-----------|
| `Error` | `"error"` | Fails the build |
| `Warning` | `"warning"` | Logs warning, build continues |
| `SourceError` | `"sourceError"` | Error if in project source, warning if in node_modules |

**DiagnosticScope** (1 variant)
| Variant | JSON |
|---------|------|
| `Optimizer` | `"optimizer"` |

**SourceLocation** (6 fields)
| Field | Rust Type | JSON | Notes |
|-------|-----------|------|-------|
| `lo` | `usize` | `lo` | Start byte offset |
| `hi` | `usize` | `hi` | End byte offset |
| `start_line` | `usize` | `startLine` | 1-indexed |
| `start_col` | `usize` | `startCol` | 1-indexed in SWC (see column computation note) |
| `end_line` | `usize` | `endLine` | 1-indexed |
| `end_col` | `usize` | `endCol` | 0-indexed display column |

**Error Codes** (from errors.rs)
| Code | Name | Trigger | Message Pattern |
|------|------|---------|-----------------|
| `C02` | FunctionReference | Identifier reference inside `$()` that is a non-exported function declaration | `"Reference to identifier '{}' can not be used inside a Qrl($) scope because it's a function"` |
| `C03` | CanNotCapture | `$()` scope is not a function but captures local identifiers | `"Qrl($) scope is not a function, but it's capturing local identifiers: {}"` |
| `C05` | MissingQrlImplementation | `reg_ctx_name` matched a `$`-suffixed specifier but the corresponding `Qrl` export was not found | `"Found '{}' but did not find the corresponding '{}' exported in the same file"` |

**Diagnostic production:** Errors are emitted during the QwikTransform pass via SWC's `HANDLER.with()` mechanism. The SWC ErrorBuffer collects these into `Vec<swc_common::errors::Diagnostic>`, which is then converted to the public `Vec<Diagnostic>` in `handle_error()`. Parse errors (from SWC parser) are also collected via the ErrorBuffer. All diagnostics carry `DiagnosticScope::Optimizer`.

## Binding Contracts

### NAPI Binding (SWC Implementation)

**File:** `qwik/packages/optimizer/napi/src/lib.rs` (42 lines)

**Pattern:**
1. Single exported function: `transform_modules`
2. Accepts one `JsObject` argument
3. Deserializes to `TransformModulesOptions` via `ctx.env.from_js_value(opts)`
4. Runs CPU-bound work on tokio `spawn_blocking` thread pool
5. Returns a Promise (via `ctx.env.execute_tokio_future`)
6. Serializes result to JS via `env.to_js_value(&result)`
7. Errors are converted to napi::Error via `.map_err(|e| napi::Error::from_reason(e.to_string()))`

**Key behaviors:**
- Async: returns a Promise, caller awaits
- Uses tokio runtime for thread pool (NAPI v2 pattern)
- Windows: uses mimalloc global allocator
- JSON serialization: handled by napi's built-in serde support (not manual JSON parsing)
- Error format: string message only (no structured error types on NAPI boundary)

**NAPI v3 differences (for OXC implementation):**
- NAPI v3 does not require tokio -- it has native async support
- NAPI v3 supports `wasm32-wasip1-threads` target (potential WASM unification)
- NAPI v3 uses `#[napi]` proc macro instead of `#[js_function(N)]`

### WASM Binding (SWC Implementation)

**File:** `qwik/packages/optimizer/wasm/src/lib.rs` (21 active lines)

**Pattern:**
1. Single exported function: `transform_modules`
2. Accepts `JsValue` parameter
3. Deserializes via `serde_wasm_bindgen::from_value(config_val)`
4. Calls `qwik_core::transform_modules(config)` synchronously
5. Serializes result via `Serializer::new().serialize_maps_as_objects(true)`
6. Returns `Result<JsValue, JsValue>` (JsValue errors for wasm-bindgen)

**Key behaviors:**
- Synchronous: blocks the calling thread (browser main thread or worker)
- No threading (WASM single-threaded)
- Uses `serialize_maps_as_objects(true)` -- HashMap keys become object property names
- Error format: js_sys::Error wrapping string message
- No global allocator override (WASM uses its own allocator)

**Platform gotcha:** WASM binding is synchronous. For large codebases, this blocks the browser main thread. Users should run in a Web Worker. The NAPI binding's async behavior does not translate to WASM -- there is no worker thread pool available.

## OXC Migration Pattern Catalog

Six key SWC-to-OXC pattern divergences identified for the migration appendix, sourced from research/PITFALLS.md, research/ARCHITECTURE.md, and Scott's transform.rs.

### 1. Fold/VisitMut -> Traverse Trait

**SWC pattern:** `impl Fold for QwikTransform` with `fn fold_call_expr()`, `fn fold_module()`, etc. Fold takes ownership, returns new node. Single-pass analyze+transform.

**OXC pattern:** `impl Traverse<'a> for QwikTransform` with `fn enter_call_expression()`, `fn exit_expression()`, etc. Traverse provides mutable references with `TraverseCtx` for scope queries and AST construction.

**Key differences:**
- OXC enter/exit hooks vs SWC's single fold hook (OXC gives pre-children and post-children control)
- OXC passes `&mut TraverseCtx<'a>` to every hook (scope access, AST builder, ancestor access)
- OXC cannot return a different node type from a hook -- mutations must be in-place via `*node = replacement`
- SWC's `FoldWith` recursion is implicit; OXC's traversal is driven by the framework

**Source:** Scott's `transform.rs` lines 34 (`use oxc_traverse::{traverse_mut, Ancestor, Traverse, TraverseCtx}`)

### 2. SyntaxContext-Based Identity -> Scoping/SymbolId

**SWC pattern:** Identifiers carry `SyntaxContext` (a u32 "mark" assigned by the resolver). Two identifiers are the same binding iff `(name, ctxt)` matches. `Id` type is `(Atom, SyntaxContext)`.

**OXC pattern:** `SemanticBuilder` produces `Scoping` with `SymbolId` for declarations and `ReferenceId` for uses. Identifier resolution via `scoping.symbol_id_from_reference(ref_id)`.

**Key differences:**
- SWC: identity is a property of each identifier node (`id!(ident)` macro)
- OXC: identity is a side table keyed by AST node ID -- requires `SemanticBuilder` pre-pass
- SWC: resolver pass must run first to assign marks
- OXC: SemanticBuilder does parsing + resolution in one step
- OXC: after AST mutation, semantic info may become stale for mutated regions (but remains valid for unmutated nodes)

**Source:** Scott's `transform.rs` lines 28-34 (`use oxc_semantic::{NodeId, ReferenceId, ScopeFlags, Scoping, SemanticBuilder, ...}`)

### 3. Ownership Transfer -> Arena Allocation

**SWC pattern:** AST nodes are heap-allocated (`Box<Expr>`). Nodes are moved (ownership transfer) during fold. `std::mem::replace` for in-place swaps.

**OXC pattern:** AST nodes live in arena (`Box<'a, Expression<'a>>`). Nodes cannot be moved between arenas. `mem::replace` still works within the same arena, but creating new nodes requires `AstBuilder::new(allocator)`.

**Key differences:**
- Cannot clone AST nodes across arenas
- New node construction must go through `AstBuilder` or `TraverseCtx::ast`
- Segment modules cannot be built as AST during input traversal (different arena needed)
- Solution: string-based segment code construction (serialize body to string, parse separately)

**Source:** Scott's `transform.rs` lines 12-13 (`use oxc_allocator::{Allocator, Box as OxcBox, ...}`)

### 4. Code Move: AST Cloning -> String-Based Construction

**SWC pattern:** `code_move` module constructs segment module AST by cloning expression nodes from the main module, wrapping them in new Module nodes with import statements.

**OXC pattern:** Segment body is extracted as a source string (span-based slicing from original source). The segment module is built from string concatenation: imports + body code. No AST cloning across arenas.

**Why:** OXC's arena allocation makes cross-arena AST construction expensive/impossible. String-based construction avoids all lifetime issues and is actually simpler.

**Source:** ARCHITECTURE.md Stage 3 ("Segment modules are built from extracted data (not from AST clones)")

### 5. GlobalCollect -> SemanticBuilder + Collector

**SWC pattern:** `GlobalCollect` is a manual visitor that catalogs all imports, exports, and root declarations in a single pass before transforms begin.

**OXC pattern:** `SemanticBuilder` provides scope/symbol tables. A separate `collector::collect()` pass gathers dollar-call-specific metadata (dollar imports, call sites, display names). The two work together.

**Key difference:** OXC's SemanticBuilder gives you scope resolution "for free" but does not know about `$`-specific semantics. The collector pass fills that gap.

**Source:** ARCHITECTURE.md Stage 1 step 1d

### 6. Deferred Statement Insertion -> exit_program

**SWC pattern:** `fold_module` can directly modify the module's body (add/remove statements) because fold takes ownership and returns the modified module.

**OXC pattern:** `traverse_mut` visits nodes depth-first. You cannot insert sibling statements during `enter_call_expression` because the parent's children vector is not accessible for insertion. Solution: accumulate pending imports/hoisted statements in a `Vec`, then insert them all in `exit_program`.

**Source:** ARCHITECTURE.md Stage 2 (`exit_program: Rewrite imports, insert hoisted function declarations, add #__PURE__ annotations`)

## Curated Snapshot Selection (24 examples, all 14 CONVs)

Selection criteria: each CONV covered at least once, priority on diversity (strategies, modes, edge cases, nested captures, JSX variants). Each snapshot name below maps to a `.snap` file in Jack's `swc-snapshots/` directory.

| # | Snapshot Name | Primary CONV(s) | Why Selected |
|---|---------------|-----------------|--------------|
| 1 | `example_1` | CONV-01, CONV-02, CONV-05 | Basic dollar detection + QRL wrapping + segment extraction |
| 2 | `example_functional_component` | CONV-01, CONV-02, CONV-03 | component$ with useStore, capture analysis basics |
| 3 | `example_capture_imports` | CONV-03, CONV-12 | Import captures vs self-imports |
| 4 | `example_multi_capture` | CONV-03 | Multiple captured variables |
| 5 | `destructure_args_colon_props` | CONV-04 | Props destructuring basic case |
| 6 | `example_segment_variable_migration` | CONV-05 | Variable migration into segments |
| 7 | `example_jsx` | CONV-06 | JSX _jsxSorted/_jsxSplit basics |
| 8 | `example_jsx_listeners` | CONV-06 | Event handler JSX transforms |
| 9 | `example_derived_signals_cmp` | CONV-07 | Signal optimization (_fnSignal, _wrapProp) |
| 10 | `example_functional_component_2` | CONV-08 | PURE annotation on componentQrl |
| 11 | `example_dead_code` | CONV-09 | Dead branch elimination |
| 12 | `example_build_server` | CONV-10 | isServer/isBrowser const replacement |
| 13 | `example_strip_client_code` | CONV-11 | Code stripping (strip_exports) |
| 14 | `example_strip_server_code` | CONV-11 | Server code stripping |
| 15 | `rename_builder_io` | CONV-12 | Legacy import rewriting |
| 16 | `example_of_synchronous_qrl` | CONV-13 | sync$ serialization (_qrlSync) |
| 17 | `example_noop_dev_mode` | CONV-14 | Noop QRL handling (_noopQrlDEV) |
| 18 | `example_inlined_entry_strategy` | CONV-02, CONV-05 | Inline entry strategy (inlinedQrl) |
| 19 | `example_dev_mode` | CONV-02 | Dev mode (qrlDEV variants) |
| 20 | `example_prod_node` | CONV-02 | Prod mode (short s_ names) |
| 21 | `example_input_bind` | CONV-06 | bind:value/bind:checked sugar |
| 22 | `should_transform_nested_loops` | CONV-03 | Loop capture edge case |
| 23 | `example_lib_mode` | CONV-10 | Lib mode (no const replacement) |
| 24 | `example_preserve_filenames` | CONV-05 | preserve_filenames config effect |

**Coverage verification:**
- CONV-01 (Dollar Detection): #1, #2
- CONV-02 (QRL Wrapping): #1, #2, #18, #19, #20
- CONV-03 (Capture Analysis): #2, #3, #4, #22
- CONV-04 (Props Destructuring): #5
- CONV-05 (Segment Extraction): #1, #6, #18, #24
- CONV-06 (JSX Transform): #7, #8, #21
- CONV-07 (Signal Optimization): #9
- CONV-08 (PURE Annotations): #10
- CONV-09 (Dead Branch Elimination): #11
- CONV-10 (Const Replacement): #12, #23
- CONV-11 (Code Stripping): #13, #14
- CONV-12 (Import Rewriting): #3, #15
- CONV-13 (sync$ Serialization): #16
- CONV-14 (Noop QRL Handling): #17

All 14 CONVs covered. Several CONVs have multiple examples showing different modes/strategies.

## Spec Document Structure (Where Phase 4 Content Goes)

The existing spec ends at line 5254 with the Transformation Pipeline section. Phase 4 adds four new sections:

```
[Existing spec: lines 1-5254]

## Public API Types                    <-- NEW (SPEC-18, SPEC-19, SPEC-20)
  ### TransformModulesOptions
  ### TransformModuleInput
  ### TransformOutput
  ### TransformModule
  ### SegmentAnalysis
  ### SegmentKind
  ### EntryStrategy
  ### EmitMode
  ### MinifyMode
  ### Diagnostic
  ### DiagnosticCategory
  ### SourceLocation

## Binding Contracts                   <-- NEW (SPEC-26, SPEC-27)
  ### NAPI Binding
  ### WASM Binding

## Appendix A: OXC Migration Guide    <-- NEW (SPEC-28)
  ### Migration Pattern 1-6
  ### Per-CONV Migration Notes

## Appendix B: Representative Examples <-- NEW (SPEC-29)
  ### Example 1-24 (curated snapshots)
```

Per D-29: OXC migration goes as an appendix at the end, does NOT modify Phases 1-3 content.
Per the code_context in CONTEXT.md: API types go near the top (but since Phases 1-3 are already written and we append, they go after the existing content but before appendices). Binding contracts go after transformation sections, before appendices.

**Recommended ordering within the spec:**
1. `## Public API Types` (immediately after existing content)
2. `## Binding Contracts` (after types)
3. `## Appendix A: OXC Migration Guide` (after binding contracts)
4. `## Appendix B: Representative Examples` (at the very end)

## Common Pitfalls

### Pitfall 1: SWC Atom vs String Type Choice
**What goes wrong:** SWC uses `swc_atoms::Atom` (interned string) for most string fields. The spec must use standard `String` in type definitions since the OXC implementation will not use SWC's Atom.
**How to avoid:** Document types with Rust `String` in the spec struct definitions, with a note that SWC uses `Atom` for performance. The JSON wire format is identical.

### Pitfall 2: EntryStrategy Serialization Mismatch
**What goes wrong:** SWC's `EntryStrategy` enum uses `#[serde(rename_all = "camelCase")]` which serializes variants as bare strings: `"inline"`, `"hoist"`, etc. But the TypeScript `types.ts` defines EntryStrategy as a tagged union: `{ type: "inline" }`. Jack's implementation uses `#[serde(tag = "type")]` to match the TS contract. The spec must be explicit about which format is correct.
**How to avoid:** The spec should document the wire format as `{ "type": "inline" }` (tagged object form) since that is what JS callers send via the TypeScript API. Note the SWC internal serialization difference.

### Pitfall 3: SegmentKind JSXProp Variant
**What goes wrong:** SWC's `SegmentKind` has only 2 variants (`Function`, `EventHandler`). Jack's OXC implementation added `JSXProp` as a third variant. The spec must decide whether to include it.
**How to avoid:** Document SWC's 2 variants as the source-of-truth behavior. Note Jack's `JSXProp` addition as an OXC extension in the migration appendix. Per D-05, SWC is source of truth.

### Pitfall 4: TransformModule.order Field Visibility
**What goes wrong:** SWC's `TransformModule.order` field has `#[serde(skip_serializing)]` -- it exists in Rust but is never present in JSON output. If the spec documents it as a public API field, implementers will serialize it unnecessarily.
**How to avoid:** Document `order` as an internal-only field with the skip_serializing annotation.

### Pitfall 5: SourceLocation Column Indexing
**What goes wrong:** SWC's SourceLocation has a non-obvious column computation: `start_col` is 1-indexed, `end_col` is 0-indexed display column. The comment in utils.rs explains this as a cancellation of SWC's exclusive-column / 0-based-column conventions. Getting this wrong produces off-by-one errors in diagnostic highlighting.
**How to avoid:** Document the exact column semantics with the SWC source comment as reference.

### Pitfall 6: Diagnostic Error Codes Are Sparse
**What goes wrong:** The error codes are `C02`, `C03`, `C05` -- not sequential. Code `C04` does not exist. Implementers who expect sequential codes may create phantom error codes.
**How to avoid:** Document the exact codes and their semantic meaning. Note the gaps explicitly.

## Code Examples

### TransformModulesOptions Rust Struct Definition (for spec)
```rust
// Source: qwik/packages/optimizer/core/src/lib.rs:54-74
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransformModulesOptions {
    pub src_dir: String,
    pub root_dir: Option<String>,
    pub input: Vec<TransformModuleInput>,
    pub source_maps: bool,
    pub minify: MinifyMode,
    pub transpile_ts: bool,
    pub transpile_jsx: bool,
    pub preserve_filenames: bool,
    pub entry_strategy: EntryStrategy,
    pub explicit_extensions: bool,
    pub mode: EmitMode,
    pub scope: Option<String>,
    pub core_module: Option<String>,
    pub strip_exports: Option<Vec<String>>,
    pub strip_ctx_name: Option<Vec<String>>,
    pub strip_event_handlers: bool,
    pub reg_ctx_name: Option<Vec<String>>,
    pub is_server: Option<bool>,
}
```

### NAPI Binding Contract (for spec)
```rust
// Source: qwik/packages/optimizer/napi/src/lib.rs
// Simplified contract (SWC uses napi v2 pattern)
#[napi]
pub async fn transform_modules(config: TransformModulesOptions) -> Result<TransformOutput> {
    // 1. Deserialize config from JS object (automatic via napi serde)
    // 2. Spawn CPU-bound transform on blocking thread pool
    // 3. Return result as JS object (automatic via napi serde)
    // 4. Errors become rejected Promise with string message
}
```

### WASM Binding Contract (for spec)
```rust
// Source: qwik/packages/optimizer/wasm/src/lib.rs
#[wasm_bindgen]
pub fn transform_modules(config_val: JsValue) -> Result<JsValue, JsValue> {
    // 1. Deserialize: serde_wasm_bindgen::from_value(config_val)
    // 2. Transform: qwik_core::transform_modules(config) -- SYNCHRONOUS
    // 3. Serialize: Serializer::new().serialize_maps_as_objects(true)
    // 4. Errors become JsValue (js_sys::Error)
}
```

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual review (spec writing, not code) |
| Config file | N/A |
| Quick run command | `wc -l specification/qwik-optimizer-spec.md` (verify content added) |
| Full suite command | Manual review: verify all 14 CONVs covered in examples, all 7 API types documented |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SPEC-18 | TransformModulesOptions documented with all 16 fields | manual | Grep for `TransformModulesOptions` in spec | N/A |
| SPEC-19 | TransformOutput/Module/SegmentAnalysis documented | manual | Grep for `TransformOutput` in spec | N/A |
| SPEC-20 | Diagnostic type documented with 3 error codes | manual | Grep for `DiagnosticCategory` in spec | N/A |
| SPEC-26 | NAPI binding contract documented | manual | Grep for `NAPI` in spec | N/A |
| SPEC-27 | WASM binding contract documented | manual | Grep for `WASM` in spec | N/A |
| SPEC-28 | OXC migration guide with 6 patterns | manual | Grep for `OXC Migration` in spec | N/A |
| SPEC-29 | 20+ examples covering all 14 CONVs | manual | Count examples in appendix | N/A |

### Sampling Rate
- **Per task commit:** `grep -c "^### " specification/qwik-optimizer-spec.md` (verify sections added)
- **Per wave merge:** Full section count + CONV coverage spot check
- **Phase gate:** All 7 SPEC requirements checked by verifier

### Wave 0 Gaps
None -- this is spec writing, not code. No test framework needed.

## Sources

### Primary (HIGH confidence)
- SWC `lib.rs` -- `TransformModulesOptions`, `TransformModuleInput`, `transform_modules()`
- SWC `parse.rs` -- `TransformOutput`, `TransformModule`, `SegmentAnalysis`, `MinifyMode`, `EmitMode`
- SWC `transform.rs` -- `SegmentKind`, `Segment`, `SegmentData` (internal types)
- SWC `entry_strategy.rs` -- `EntryStrategy` enum, `EntryPolicy` trait
- SWC `utils.rs` -- `Diagnostic`, `DiagnosticCategory`, `DiagnosticScope`, `SourceLocation`
- SWC `errors.rs` -- Error codes C02, C03, C05
- SWC `napi/src/lib.rs` -- NAPI binding contract (42 lines)
- SWC `wasm/src/lib.rs` -- WASM binding contract (21 lines)
- SWC `types.ts` -- TypeScript type definitions (JS-facing API contract)
- Jack's `types.rs` -- OXC type definitions with doc comments
- Scott's `transform.rs` -- OXC traverse patterns, arena allocation usage
- research/PITFALLS.md -- 6 critical pitfalls including SWC->OXC migration traps
- research/ARCHITECTURE.md -- Three-stage pipeline architecture

### Secondary (MEDIUM confidence)
- Jack's `swc-snapshots/` directory -- 201 snapshot files (count verified by `ls | wc -l`)

## Metadata

**Confidence breakdown:**
- API types: HIGH -- complete extraction from SWC source, cross-verified against Jack's types and TypeScript definitions
- Binding contracts: HIGH -- complete source read of both NAPI and WASM binding files
- OXC migration patterns: HIGH -- sourced from existing research docs and direct codebase analysis
- Example selection: HIGH -- all 201 snapshots listed, 24 selected with verified CONV coverage
- Pitfalls: HIGH -- derived from direct source code analysis and prior research

**Research date:** 2026-04-01
**Valid until:** 2026-05-01 (stable -- SWC source is frozen v2 branch, Jack's snapshots are fixed corpus)

---

*Phase: 04-public-api-bindings-cross-cutting-specification*
*Research complete: 2026-04-01*
