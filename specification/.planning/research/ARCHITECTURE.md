# Architecture Patterns: OXC-Based JavaScript Transformation Systems

**Domain:** Qwik optimizer -- AST transformation pipeline porting from SWC to OXC
**Researched:** 2026-04-01
**Confidence:** HIGH (verified against Jack's working OXC implementation, SWC source, OXC docs.rs v0.99.0)

## Recommended Architecture: Three-Stage Pipeline (Parse, Transform, Emit)

The Qwik OXC optimizer should be structured as a three-stage pipeline operating on a single arena-allocated AST per input file:

1. **Parse** -- `oxc_parser` produces `Program<'a>`, then `SemanticBuilder` produces `Scoping` (scope tree + symbol table + reference table). Pre-traverse mutations (const replacement, export stripping) happen here.
2. **Transform** -- A single `traverse_mut` pass using `impl Traverse for QwikTransform` that both analyzes and mutates the AST. This is the pragmatic approach proven by Jack's implementation, not the idealized two-phase model.
3. **Emit** -- `oxc::codegen::Codegen` serializes the mutated main module. Segment modules are built from extracted data (not from AST clones) and codegen'd separately.

### Why Single-Pass Transform Instead of Two-Phase Analyze-Then-Emit

Jack's earlier architecture research recommended a strict two-phase model (analyze-only traverse, then separate emit). His actual working implementation uses a **single `traverse_mut` pass** that interleaves analysis and mutation -- the same conceptual approach as SWC's `Fold`, adapted to OXC's mutable-reference model.

This pragmatic choice works because:

- **OXC's `enter_*`/`exit_*` hooks provide ordering control.** Enter fires pre-children, exit fires post-children. You can analyze in `enter_*` and mutate in `exit_*` within the same pass.
- **The optimizer's mutations are mostly local.** Rewriting a `$()` call to `qrl()` does not invalidate the scope information needed for sibling nodes. The `Scoping` data from `SemanticBuilder` remains valid for reads during the pass because identifier resolution was completed before traversal began.
- **Pre-traverse mutations handle the cases that would break single-pass.** Const replacement (`isServer`/`isDev`/`isBrowser`) and export stripping run before `traverse_mut`, so the traversal sees a stable AST for these concerns.
- **Segment body extraction uses span-based source slicing**, not AST cloning. The transform records spans and the original source code, then `code_move` builds segment modules from string data + metadata. This sidesteps the "can't read ahead while mutating" problem entirely.

The specification should document both the ideal two-phase model (as an architectural principle) and the pragmatic single-pass reality (as the implementation pattern), because future refactoring might benefit from phase separation as the codebase matures.

## System Overview

```
                            INPUT
    +----------------------------------------------------------+
    |  Vec<TransformModuleInput> { path, code }                |
    |  + TransformModulesOptions (config)                      |
    +---------------------------+------------------------------+
                                |
                    (per input file, parallelizable)
                                |
                                v
    +----------------------------------------------------------+
    |  Stage 1: PARSE                                          |
    |                                                          |
    |  1a. Allocator::default() -- per-file arena              |
    |  1b. Parser::new().parse() -> Program<'a>                |
    |  1c. SemanticBuilder::new().build() -> Scoping           |
    |  1d. collector::collect() -> CollectResult               |
    |      (imports, exports, root declarations)               |
    |  1e. const_replace::replace_build_constants()            |
    |      (isServer/isDev/isBrowser -> true/false)            |
    |  1f. filter_exports::filter_exports()                    |
    |      (strip_exports config applied)                      |
    +---------------------------+------------------------------+
                                |
                                v
    +----------------------------------------------------------+
    |  Stage 2: TRANSFORM (single traverse_mut pass)           |
    |                                                          |
    |  QwikTransform: impl Traverse<'a, ()>                    |
    |                                                          |
    |  enter_call_expression:                                  |
    |    - Detect $-boundary calls (component$, $, etc.)       |
    |    - Push capture tracking state                         |
    |    - Record segment metadata (hash, name, captures)      |
    |    - Handle sync$() detection                            |
    |    - Handle props destructuring detection                |
    |                                                          |
    |  exit_expression:                                        |
    |    - Replace $() calls with qrl()/inlinedQrl() calls     |
    |    - Serialize segment body code (span-based extraction)  |
    |    - Pop capture tracking state                          |
    |    - Apply props destructuring rewrites                  |
    |                                                          |
    |  enter/exit_jsx_element:                                 |
    |    - Transform JSX to _jsxSorted/_jsxSplit calls         |
    |    - Extract event handler $-lambdas                     |
    |    - Signal wrapping (_wrapProp, _fnSignal)              |
    |    - Key generation                                      |
    |                                                          |
    |  exit_program:                                           |
    |    - Rewrite imports (add qrl helpers, lazy imports)     |
    |    - Insert hoisted function declarations                |
    |    - Add #__PURE__ annotations                           |
    |                                                          |
    |  State accumulated: Vec<SegmentData>, ImportTracker,     |
    |  Vec<(u32, String)> body codes, Vec<Diagnostic>          |
    +---------------------------+------------------------------+
                                |
                                v
    +----------------------------------------------------------+
    |  Stage 3: EMIT                                           |
    |                                                          |
    |  3a. Codegen main module:                                |
    |      oxc::codegen::Codegen::new()                        |
    |        .with_source_text(source)                         |
    |        .build(&program) -> (code, source_map)            |
    |      + Post-hoc hoisted statement injection              |
    |                                                          |
    |  3b. Build segment modules (per extracted segment):      |
    |      code_move::build_segment_code_with_hoisted()        |
    |        -> raw segment code string                        |
    |      code_move::emit_segment_with_map()                  |
    |        -> parse + codegen for source map                 |
    |                                                          |
    |  Output: TransformOutput                                 |
    |    - Vec<TransformModule> (main + segments)              |
    |    - Vec<Diagnostic> (warnings/errors)                   |
    |    - is_type_script, is_jsx flags                        |
    +----------------------------------------------------------+
```

## Component Boundaries

| Component | Responsibility | Communicates With | SWC Equivalent |
|-----------|---------------|-------------------|----------------|
| **`lib.rs`** | Public API entry point `transform_modules()`; per-file orchestration loop; option normalization | All internal modules | `parse.rs` (top half) |
| **`parse.rs`** | `Parser::new().parse()` + `SemanticBuilder` wrapper; produces `Program<'a>` + `Scoping` | `lib.rs` | `parse.rs` (parse fn) |
| **`collector.rs`** | `collect()` -- single-pass read of imports, exports, root declarations; builds `CollectResult` | `lib.rs`, `transform.rs` | `collector.rs` |
| **`const_replace.rs`** | `replace_build_constants()` -- replaces `isServer`/`isDev`/`isBrowser` with boolean literals | `lib.rs` (pre-traverse) | `const_replace.rs` |
| **`filter_exports.rs`** | `filter_exports()` -- removes exports matching `strip_exports` config | `lib.rs` (pre-traverse) | `filter_exports.rs` |
| **`transform.rs`** | `QwikTransform: impl Traverse` -- the core single-pass transformation; $-call detection, QRL wrapping, capture analysis, segment recording | `collector.rs` (reads CollectResult), `entry_strategy.rs`, `hash.rs`, `jsx_transform.rs`, `props_destructuring.rs` | `transform.rs` (QwikTransform: Fold) |
| **`jsx_transform.rs`** | JSX-specific transform logic: `_jsxSorted`/`_jsxSplit` rewriting, signal wrapping, event handler extraction | `transform.rs` (called during traverse) | JSX portions of `transform.rs` |
| **`props_destructuring.rs`** | Reconstructs destructured component props for signal forwarding | `transform.rs` | `props_destructuring.rs` |
| **`entry_strategy.rs`** | `EntryStrategy` enum, `should_inline()`, segment grouping logic | `transform.rs`, `lib.rs` | `entry_strategy.rs` |
| **`hash.rs`** | Content-based hash generation for segment canonical filenames | `transform.rs` | `hash.rs` |
| **`import_rewrite.rs`** | Import statement mutation logic (add qrl helpers, rewrite sources) | `transform.rs` | Part of `transform.rs` fold_module |
| **`code_move.rs`** | Segment module construction from body code strings + metadata | `lib.rs` (post-traverse) | `code_move.rs` |
| **`emit.rs`** | `emit_module()` -- wraps `oxc::codegen::Codegen` with source map support | `lib.rs` | `emit_source_code()` in `parse.rs` |
| **`types.rs`** | All shared types: options, output structs, `SegmentData`, `CollectResult`, `Diagnostic` | All modules | Spread across `parse.rs`, `transform.rs`, `utils.rs` |
| **`words.rs`** | String constants (`@qwik.dev/core`, QRL function names, etc.) | All modules | `words.rs` |
| **`errors.rs`** / **`diagnostics`** | Diagnostic creation helpers | All modules | `utils.rs` |
| **`is_const.rs`** | Determines if an expression is a constant (for signal optimization) | `jsx_transform.rs` | Part of `transform.rs` |

## Data Flow: How Information Moves Through the Pipeline

### Flow 1: Module-Level Metadata (Imports/Exports/Declarations)

```
Source code
  -> Parser -> Program AST
  -> collector::collect() reads AST, produces CollectResult {
       imports: HashMap<Id, Import>,
       exports: HashMap<Id, Export>,
       root: HashMap<Id, RootDecl>,
       qwik_import_source: Option<String>,
     }
  -> CollectResult passed to QwikTransform constructor
  -> QwikTransform reads it during traverse for:
     - Determining if a callee is a known $-function
     - Building segment import lists
     - Deciding which imports to add/remove
```

### Flow 2: Segment Extraction

```
traverse_mut enter_call_expression:
  detect $-call -> push to dollar_call_stack, push capture_stack entry

  (children visited -- identifiers in body recorded to capture_stack)

traverse_mut exit_expression:
  pop capture_stack -> compute captured vs local variables
  serialize body code via span slice of original source
  record SegmentData { name, hash, display_name, captures, ctx_name, ... }
  replace $-call AST node with qrl()/inlinedQrl() call
  store body code in segment_body_codes: Vec<(u32, String)>

After traverse_mut returns:
  lib.rs iterates segments
  code_move builds segment module string from body code + imports + metadata
  emit produces (code, source_map) per segment
  -> TransformModule { path, code, map, segment: Some(SegmentAnalysis) }
```

### Flow 3: JSX Transformation

```
traverse_mut enter/exit JSX nodes:
  jsx_transform::transform_jsx_element_inner()
    - Rewrites <div onClick$={...}> to _jsxSorted("div", {...})
    - Extracts event handler lambdas as segments (same as Flow 2)
    - Applies signal wrapping (_wrapProp, _fnSignal) for reactive props
    - Generates stable keys
  Records import needs in ImportTracker (needs_jsx_sorted, etc.)

traverse_mut exit_program:
  import_rewrite adds needed JSX imports to module
```

### Flow 4: Import Rewriting

```
ImportTracker accumulates flags during traverse:
  needs_qrl, needs_inlined_qrl, needs_captures,
  needs_jsx_sorted, needs_jsx_split, needs_wrap_prop, ...
  qrl_imports: Vec<String> (e.g., "componentQrl")
  lazy_imports: Vec<(hash, path)>

exit_program or finalize_segments:
  import_rewrite::rewrite_imports() reads ImportTracker
  Adds import { qrl, _jsxSorted, ... } from "@qwik.dev/core"
  Adds lazy import consts: const lazyAbc = ()=> import("./segment")
  Removes unused original imports
```

## Key Difference: SWC vs OXC Architectural Patterns

### Ownership Model (Fundamental Divergence)

**SWC `Fold`:** Takes ownership of each node, returns a (possibly different) node. The transform "consumes" the old AST and produces a new one. This makes analysis-during-mutation natural because you own the children when processing a parent.

```rust
// SWC: ownership transfer
fn fold_call_expr(&mut self, node: CallExpr) -> CallExpr {
    let node = node.fold_children_with(self); // children transformed first
    self.maybe_rewrite(node) // return new or modified node
}
```

**OXC `Traverse`:** Provides mutable references to nodes in-place. The transform mutates the existing AST. `enter_*` fires before children are visited, `exit_*` fires after. Semantic analysis (scoping) is a separate pre-pass.

```rust
// OXC: mutable reference
fn exit_expression(&mut self, expr: &mut Expression<'a>, ctx: &mut TraverseCtx<'a, ()>) {
    if self.is_pending_dollar_call(expr) {
        *expr = self.build_qrl_call(expr, ctx); // assign new value
    }
}
```

### Scope Resolution

| Aspect | SWC | OXC |
|--------|-----|-----|
| When computed | Inline during parse (hygiene marks) | Separate `SemanticBuilder` pass before traverse |
| Variable identity | `Id = (Atom, SyntaxContext)` | `SymbolId` from `Scoping` |
| Scope lookup | `SyntaxContext` equality | `scoping.symbol_id_for()`, `scoping.scope_id_for()` |
| Available during traverse | Always (part of every identifier) | Via `TraverseCtx::scoping()` |

### Multi-Module Output

| Aspect | SWC | OXC (Jack's impl) |
|--------|-----|--------------------|
| Segment AST construction | `new_module()` builds `Module` AST from scratch using heap allocation | String-based: serialize body via span slice, `build_segment_code_with_hoisted()` assembles code string, re-parse + codegen for source map |
| Memory model | Heap-allocated nodes, independent of original AST | Arena-allocated; segment code strings escape arena as `String` |
| Source maps | SWC's `SourceMap` tracks spans through fold | Re-parse segment code string with OXC parser, then codegen with `source_map_path` |

### Statement Manipulation

| Operation | SWC | OXC |
|-----------|-----|-----|
| Insert statement | Return `Vec<Stmt>` from fold (parent expands) | Collect insertions, apply in `exit_program` batch |
| Delete statement | Return empty vec or `Stmt::Empty` | Mark with empty, filter in exit handler |
| Replace expression | Return different `Expr` variant | `*expr = new_expr` on mutable reference |

### Post-Traverse Fixups

**SWC** runs `hygiene()` and `fixer()` passes after the main fold to clean up identifier conflicts and fix AST well-formedness. **OXC** generally does not need these because `SemanticBuilder` resolves scoping upfront and `AstBuilder` produces well-formed nodes. However, Jack's implementation does use a **post-traverse string manipulation** step: hoisted function declarations are injected into the codegen output by string splicing (finding the last `import` line and inserting after it), not by AST mutation.

## Patterns to Follow

### Pattern 1: Pre-Traverse Mutation for Stable Analysis

**What:** Run const replacement and export stripping BEFORE `traverse_mut`, so the main traversal sees a stable, simplified AST.

**When:** Any transformation that changes what the main traversal needs to see (removing dead branches, stripping exports that should not be processed).

**Why:** If `isServer` is replaced with `true` before traversal, the traversal does not need to handle conditional logic. If stripped exports are removed before traversal, `$-call` detection does not record segments that will be thrown away.

**Evidence:** Both SWC and Jack's OXC implementation do this. In SWC: `ConstReplacerVisitor` and `StripExportsVisitor` run before `QwikTransform`. In OXC: `const_replace::replace_build_constants()` and `filter_exports::filter_exports()` run before `traverse_mut`.

### Pattern 2: Span-Based Body Extraction

**What:** Extract segment function bodies by slicing the original source code at recorded spans, rather than serializing AST subtrees.

**When:** Building segment modules for the "segment" (non-inline) entry strategy.

**Why:** Avoids the complexity of cloning arena-allocated AST subtrees. The original source code is available as a string; spans mark exactly where each `$-callback` body starts and ends. The extracted string is then assembled into a complete segment module with imports and export wrapper, re-parsed by OXC for source map generation.

**Evidence:** Jack's implementation stores `source_code: String` on `QwikTransform` and uses span offsets to extract body code into `segment_body_codes: Vec<(u32, String)>`.

### Pattern 3: ImportTracker Accumulator

**What:** Use a flag-based accumulator (`ImportTracker`) during traversal to record which imports the output module will need, then batch-apply all import additions in a single pass at `exit_program`.

**When:** Always. Import rewriting is the final step of the traverse.

**Why:** Import needs are discovered incrementally during traversal (this `$-call` needs `qrl`, this JSX needs `_jsxSorted`, etc.). Accumulating flags and applying them all at once avoids multiple passes over the import declarations.

### Pattern 4: Per-File Arena Allocation

**What:** Create one `Allocator` per input file. All AST nodes for that file live in the arena. After codegen serializes to `String`, the arena is dropped.

**When:** Always. This is OXC's fundamental memory model.

**Why:** Arena allocation is fast (bump pointer), deallocation is instant (drop the arena). Memory does not leak because the entire arena is freed after the file is processed. Parallelism at the file level means each thread has its own arena with no contention.

## Anti-Patterns to Avoid

### Anti-Pattern 1: Cloning Arena-Allocated ASTs for Segments

**What:** `program.clone()` then pruning to build segment modules.

**Why bad:** Every node is arena-allocated. Cloning means allocating a new arena's worth of nodes, then deleting most of them. Segments are structurally different from the original (they have a single export, different imports). Building from scratch is both simpler and cheaper.

**Instead:** Use span-based body extraction (Pattern 2) or `AstBuilder` construction for segments.

### Anti-Pattern 2: Manual Scope Tracking Duplicating SemanticBuilder

**What:** Maintaining a hand-rolled `decl_stack: Vec<Vec<Binding>>` for variable resolution.

**Why bad:** `oxc_semantic` already builds a complete scope tree and symbol table. Manual tracking misses edge cases (catch clauses, for-in/for-of bindings, class field initializers) and duplicates work.

**Instead:** Use `Scoping` from `SemanticBuilder`. Access via `TraverseCtx::scoping()` during traversal.

**Caveat:** Jack's current implementation uses a simple `capture_stack: Vec<(Vec<String>, HashSet<String>)>` for capture tracking rather than full semantic scope queries. This is a pragmatic shortcut that works for the common case but may miss edge cases. The specification should document the correct semantic-based approach while noting the simplified implementation.

### Anti-Pattern 3: Two Separate Traverse Passes When One Suffices

**What:** Running `traverse_mut` twice -- once for analysis, once for mutation.

**Why bad:** Each `traverse_mut` call walks the entire AST. For a 10K-line file with complex JSX, this doubles traversal time. The OXC `Traverse` trait's `enter_*`/`exit_*` hooks provide sufficient ordering control to interleave analysis and mutation in a single pass for the Qwik optimizer's needs.

**Instead:** Single `traverse_mut` with analysis in `enter_*` and mutation in `exit_*`. Use pre-traverse mutations (Pattern 1) for concerns that must complete before traversal begins.

**When two passes ARE justified:** If a future transformation requires global knowledge that can only be computed after visiting the entire AST (e.g., whole-module dead code analysis), a separate analysis pass becomes necessary. The current optimizer does not have this requirement.

## Suggested Build Order (Dependencies Between Components)

Components should be built in dependency order. The specification should describe them in this order so the implementation can proceed incrementally.

```
Layer 1: Foundation (no internal dependencies)
    types        -- All shared data structures, options, output types
    words        -- String constants (@qwik.dev/core, function names)
    hash         -- Content-based hashing for segment names
    diagnostics  -- Diagnostic creation helpers
    is_const     -- Expression constness analysis

Layer 2: Parse + Collect (depends on Layer 1)
    parse        -- Parser + SemanticBuilder wrapper
    collector    -- Import/export/declaration collection (reads AST)
    entry_strategy -- EntryStrategy enum and grouping logic

Layer 3: Pre-Traverse Mutations (depends on Layers 1-2)
    const_replace    -- isServer/isDev/isBrowser replacement
    filter_exports   -- strip_exports removal

Layer 4: Core Transform (depends on all above)
    props_destructuring -- Props reconstruction for signal forwarding
    jsx_transform       -- JSX rewriting (_jsxSorted, _jsxSplit, signals)
    import_rewrite      -- Import addition/removal logic
    transform           -- QwikTransform: impl Traverse (the heart)

Layer 5: Emit (depends on Layers 1, 4)
    code_move    -- Segment module construction from extracted data
    emit         -- oxc::codegen::Codegen wrapper

Layer 6: Integration (depends on all)
    lib          -- Public API, per-file orchestration loop
```

**Key dependency chains:**
- `transform.rs` depends on `collector.rs`, `entry_strategy.rs`, `hash.rs`, `jsx_transform.rs`, `props_destructuring.rs`, `import_rewrite.rs`
- `lib.rs` depends on `parse.rs`, `collector.rs`, `const_replace.rs`, `filter_exports.rs`, `transform.rs`, `code_move.rs`, `emit.rs`
- `code_move.rs` depends on `types.rs` and `words.rs` only (works with strings, not AST)
- `jsx_transform.rs` depends on `is_const.rs`, `words.rs`

**Build Layer 1 first, then 2, then 3 and 4 can partially overlap.** Within Layer 4, `transform.rs` should be built last since it integrates all other Layer 4 modules. `emit.rs` and `code_move.rs` are relatively independent and can be built in parallel with Layer 4.

## The `traverse_mut` API Contract

The current OXC `traverse_mut` signature (v0.99.0):

```rust
pub fn traverse_mut<'a, State, Tr: Traverse<'a, State>>(
    traverser: &mut Tr,
    allocator: &'a Allocator,
    program: &mut Program<'a>,
    scoping: Scoping,
    state: State,
) -> Scoping
```

Key points:
- **`Scoping`** is consumed (moved in) and returned. It contains `ScopeTree`, `SymbolTable`, and `ReferenceTable`.
- **`State`** is a generic parameter for user-defined state passed through. Jack's implementation uses `()`.
- **`Traverse<'a, State>`** trait has `enter_*` and `exit_*` methods for every AST node type. Only override the ones you need.
- The returned `Scoping` reflects any scope changes made during traversal (e.g., if nodes were added/removed).

## Scalability Considerations

| Concern | At 100 modules | At 10K modules | At 100K modules |
|---------|----------------|----------------|-----------------|
| Per-file transform | ~1-2ms (parse + semantic + traverse + codegen) | Same per file | Same per file |
| Memory per file | One arena (~1-5MB), dropped after codegen | Same | Same |
| Parallelism | `rayon::par_iter` over input files | Essential | Essential + consider batching |
| Segment count/file | 1-10 typical | Same | Same |
| Peak memory | Proportional to largest single file | Thread count * largest file | Batch to limit threads |

The optimizer is embarrassingly parallel at the file level. Each file's pipeline is independent: allocate arena, parse, collect, pre-mutate, traverse, emit, drop arena. Memory scales with thread count, not total project size.

## Sources

- [oxc_traverse v0.99.0 docs](https://docs.rs/oxc_traverse/0.99.0/oxc_traverse/) -- Traverse trait, traverse_mut signature (HIGH confidence)
- [oxc_semantic Scoping](https://docs.rs/oxc_semantic/latest/oxc_semantic/struct.Scoping.html) -- Scoping struct, scope/symbol/reference tables (HIGH confidence)
- [OXC transformer architecture blog](https://oxc.rs/blog/2024-09-29-transformer-alpha) -- Performance benchmarks, design rationale (HIGH confidence)
- Jack's OXC implementation: `qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/` -- Working code, ground truth for actual patterns used (HIGH confidence)
- Jack's architecture research: `qwik-oxc-optimizer/.planning/research/ARCHITECTURE.md` -- Two-phase idealized model, component boundaries (HIGH confidence for design intent, MEDIUM for implementation reality)
- SWC optimizer source: `qwik/packages/optimizer/core/src/parse.rs` -- 20-step pipeline, fold-based architecture (HIGH confidence)
- [OXC statement manipulation discussion](https://github.com/oxc-project/oxc/issues/6993) -- Statement insertion/deletion patterns (MEDIUM confidence)

---
*Architecture research for: OXC-based Qwik optimizer transformation system*
*Researched: 2026-04-01*
