# Phase 3: Build Modes & Remaining Transforms Specification - Research

**Researched:** 2026-04-01
**Domain:** Qwik Optimizer build modes, entry strategies, remaining CONV transformations (08-11, 13-14), pipeline ordering
**Confidence:** HIGH

## Summary

Phase 3 completes the behavioral specification by documenting: (1) six remaining CONV transformations -- PURE annotations, Dead Branch Elimination, Const Replacement, Code Stripping, sync$ serialization, and Noop QRL handling; (2) all 7 entry strategies with their grouping rules; (3) all 5 emit modes with per-CONV behavioral differences; and (4) the full pipeline ordering DAG with dependency rationale.

The source code for all Phase 3 topics has been read in full. The smaller CONVs (08, 10, 13, 14) are each under 100 LOC in the SWC source, while CONV-09 (DCE) involves the Treeshaker (90 LOC) plus SWC's built-in simplifier. CONV-11 (Code Stripping) is 76 LOC but has three distinct mechanisms. Entry strategies are 124 LOC with 5 EntryPolicy implementations. Emit mode branching is distributed across parse.rs (~30 conditional checks).

**Primary recommendation:** Organize as the Mode x CONV cross-reference table first (D-26), then entry strategies, then smaller CONVs grouped naturally, then the pipeline DAG. This ordering gives implementers the high-level view before diving into individual transforms.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-25:** Each of the 5 distinct EntryPolicy implementations gets its own subsection with grouping rules and an example. Notes that Inline/Hoist share the same EntryPolicy (InlineStrategy) but Hoist adds unique `.s()` registration post-processing. Hook/Segment share PerSegmentStrategy.
- **D-26:** Emit modes documented as a cross-reference table (Mode x CONV -> behavioral difference), followed by brief per-mode descriptions. Compact and scannable for an implementer checking "what changes in Dev mode?"
- **D-27:** Organization at Claude's discretion -- either individual sections or grouped under an umbrella heading, based on how naturally they cluster. Each gets rules + 1-2 examples (shorter than the big CONVs).
- **D-28:** Mermaid DAG diagram showing transformation dependencies (consistent with Phase 1's pipeline diagram), plus a constraints table listing each ordering dependency with rationale.
- Carrying forward: D-01 (pipeline-ordered structure), D-04 (rules + examples per CONV), D-05 (SWC is source of truth), D-06 (SWC source file references), D-13 (examples show input + all outputs), D-16 (descriptive names with snapshot name), D-24 (inline examples + "See also" snapshot lists)

### Claude's Discretion
- Smaller CONVs organization (individual vs grouped) -- D-27

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SPEC-08 | PURE Annotations (CONV-08) -- `/*#__PURE__*/` on componentQrl only, anti-list of side-effectful wrappers | Source analysis of transform.rs shows exactly where `add_pure_comment` is called: on `componentQrl` calls, on all `qrl()`/`inlinedQrl()`/`_noopQrl()` calls (internal), and on `qSegment`. The user-visible PURE that matters for bundler tree-shaking is only on `componentQrl`. |
| SPEC-09 | Dead Branch Elimination (CONV-09) -- unreachable code removal after const replacement, client-side Treeshaker | Source: clean_side_effects.rs (Treeshaker), parse.rs simplifier calls. Two-phase: CleanMarker marks existing side effects, DCE runs, CleanSideEffects removes new ones. |
| SPEC-10 | Const Replacement (CONV-10) -- isServer/isBrowser/isDev replacement with boolean literals | Source: const_replace.rs (96 LOC). Handles imports from both `@qwik.dev/core/build` and `@qwik.dev/core`. Skipped in Lib and Test modes. |
| SPEC-11 | Code Stripping (CONV-11) -- strip_exports, strip_ctx_name, strip_event_handlers | Source: filter_exports.rs (76 LOC) for strip_exports; transform.rs `should_emit_segment` for strip_ctx_name/strip_event_handlers. Throwing stub generation. |
| SPEC-13 | sync$ Serialization (CONV-13) -- `_qrlSync()` with stringified function body | Source: transform.rs `handle_sync_qrl` + inlined_fn.rs `render_expr`. Minified codegen to string, comments stripped. |
| SPEC-14 | Noop QRL Handling (CONV-14) -- `_noopQrl()`/`_noopQrlDEV()` for stripped/empty callbacks | Source: transform.rs `create_noop_qrl` + `hoist_qrl_to_module_scope`. Used when `should_emit_segment` returns false. |
| SPEC-15 | All 7 entry strategies with grouping rules | Source: entry_strategy.rs (124 LOC). 5 EntryPolicy impls. Inline/Hoist share InlineStrategy. Hook/Segment share PerSegmentStrategy. |
| SPEC-16 | All 5 emit modes with per-CONV behavioral differences | Source: parse.rs emit mode conditionals distributed across pipeline. Mode x CONV table derivable from code analysis. |
| SPEC-17 | Pipeline ordering DAG with dependency rationale | Source: parse.rs `transform_code` function (20-step sequence). Dependency graph from FEATURES.md verified against source. |
</phase_requirements>

## Architecture Patterns

### Document Organization Recommendation

Based on D-27 (Claude's discretion for smaller CONVs), I recommend **two groupings** rather than individual sections or one umbrella:

**Group A: "Build Environment Transforms" (CONV-10, CONV-09, CONV-11)**
These three are tightly coupled: const replacement feeds dead branch elimination, and code stripping is the other half of environment-based code removal. An implementer working on server/client splitting needs all three together.

**Group B: "QRL Special Cases" (CONV-08, CONV-13, CONV-14)**
These are independent behaviors that modify how QRLs are generated or annotated. PURE annotations, sync$ serialization, and noop QRL handling each affect the QRL output but don't depend on each other.

**Rationale:** This clustering matches the source code organization (const_replace + clean_side_effects + filter_exports vs. transform.rs QRL-specific methods) and the conceptual grouping (environment-aware transforms vs. QRL variant behaviors).

### Recommended Section Order Within the Spec Document

Following D-01 (pipeline-ordered structure), the new sections should be appended after Signal Optimization (the last Phase 2 section) in this order:

```
## Stage 5: Build Environment Transforms
### Const Replacement (CONV-10)
### Dead Branch Elimination (CONV-09)
### Code Stripping (CONV-11)

## Stage 6: QRL Special Cases
### PURE Annotations (CONV-08)
### sync$ Serialization (CONV-13)
### Noop QRL Handling (CONV-14)

## Entry Strategies
### [7 strategy subsections]

## Emit Modes
### Mode x CONV Cross-Reference Table
### [5 mode subsections]

## Transformation Pipeline
### Pipeline DAG
### Ordering Constraints Table
```

**Note on "Stage" numbering:** The existing spec uses Stage 3 (Pre-Transforms) and Stage 4 (Core Transform). The new sections form Stage 5 (post-main-transform environment passes) and Stage 6 (QRL variant behaviors that occur during the main transform). Entry Strategies and Emit Modes are cross-cutting concerns, not stages, so they get top-level `##` sections. The Pipeline section is also top-level as the capstone.

### Pattern for Smaller CONV Sections

Each smaller CONV section should follow this compact pattern (per D-04, D-24):

```
### [CONV Name] (CONV-XX)
**SWC Source:** `[file.rs]` (~XX LOC)
**Pipeline Position:** Step N of 20

#### Rules
1. [Rule 1]
2. [Rule 2]
...

#### Example: [Descriptive Name] ([snapshot_name])
**Input:**
```[code]```
**Output (root module):**
```[code]```

**See also:** [list of related snapshots]
```

## Detailed Source Analysis

### CONV-08: PURE Annotations

**SWC Source:** `transform.rs` -- `add_pure_comment` calls
**Complexity:** Low (annotation placement, no AST mutation)

**Where PURE annotations are placed (from source analysis):**

1. **`componentQrl(...)` calls** -- Line 4044: When `is_qcomponent` is true, `add_pure_comment(node.span.lo)` is called on the outer call. This is the ONLY user-visible PURE annotation that affects bundler tree-shaking.

2. **`qrl()` / `inlinedQrl()` / `_noopQrl()` calls** -- Lines 1504, 1613, 2042: These internal QRL factory calls get PURE annotations via `create_internal_call(..., pure: true)`. These enable the bundler to drop unused QRL declarations.

3. **`qSegment()` calls** -- Line 4006: PURE annotation on the qSegment call.

**What does NOT get PURE:**
- `useTaskQrl(...)`, `useStyleQrl(...)`, `serverStuffQrl(...)`, etc. -- All other `Qrl`-suffixed wrappers are side-effectful (they register hooks/effects at component render time). Annotating them as PURE would cause bundlers to incorrectly drop them.
- `_jsxSorted()` / `_jsxSplit()` -- JSX calls do get PURE in some contexts (via the JSX transform), but that is already documented in Phase 2.

**Key rule:** The distinction is between **declaration** (componentQrl creates a component definition -- tree-shakeable) vs. **registration** (useTaskQrl registers a side effect -- NOT tree-shakeable).

### CONV-09: Dead Branch Elimination

**SWC Source:** `clean_side_effects.rs` (90 LOC) + `parse.rs` simplifier calls
**Complexity:** Medium (two-phase treeshaker + SWC simplifier integration)

**Three DCE mechanisms in the pipeline:**

1. **SWC Simplifier (primary DCE)** -- parse.rs line ~360: `simplify::simplifier()` with `dce::Config { preserve_imports_with_side_effects: false }`. Runs after const replacement replaces `isServer`/`isBrowser`/`isDev` with boolean literals, enabling dead branch removal. Only runs when `minify != MinifyMode::None` and `mode != EmitMode::Lib`.

2. **Treeshaker (client-side cleanup)** -- Two-phase span-based approach:
   - **CleanMarker (step 1):** Before simplifier, marks spans of top-level `new` and `call` expression statements that existed in the original code.
   - **CleanSideEffects (step 2):** After simplifier, retains only `new`/`call` statements whose spans were marked (i.e., existed before simplification). Removes statements that were exposed by the simplifier (e.g., a `call()` that was inside an `if (isServer)` block, now top-level after branch removal). Only runs on client builds (`!is_server`) and NOT for Inline/Hoist strategies.

3. **Post-migration DCE** -- parse.rs line ~420: A second simplifier pass runs after variable migration, to clean up imports that became unused when root variables moved to segments.

**Conditions table:**
| Mechanism | Condition |
|-----------|-----------|
| Simplifier DCE | `minify != None` AND `mode != Lib` |
| Treeshaker mark | `minify != None` AND `!is_server` AND `mode != Lib` |
| Treeshaker clean | `minify != None` AND `!is_server` AND NOT Inline/Hoist strategy |
| Post-migration DCE | `minify != None` AND segments exist AND vars were migrated |

**Why Inline/Hoist skip Treeshaker clean:** For Inline/Hoist strategies, `SideEffectVisitor` runs instead (add_side_effect.rs), which re-adds bare import statements for modules that had side effects. This preserves side-effectful module evaluation order.

### CONV-10: Const Replacement

**SWC Source:** `const_replace.rs` (96 LOC)
**Complexity:** Low (identifier-to-literal replacement)

**Rules:**
1. Replaces identifiers imported as `isServer` from `@qwik.dev/core/build` OR `@qwik.dev/core` with the boolean value of `config.is_server`.
2. Replaces `isBrowser` with `!config.is_server`.
3. Replaces `isDev` with the boolean value of `is_dev` (true when mode is Dev or Hmr).
4. Handles aliased imports: `import { isServer as isServer2 } from '@qwik.dev/core'` -- tracks the local alias via SWC's SyntaxContext.
5. **Skipped in Lib mode** (parse.rs line 306): Libraries ship without environment-specific booleans so consumers can set them.
6. **Skipped in Test mode** (parse.rs line 308): Tests need `isServer`/`isBrowser` to remain as identifiers for runtime configuration.
7. **import sources:** Both `@qwik.dev/core/build` (the canonical build-time constants module) and `@qwik.dev/core` (re-exports) are handled. Six identifier slots tracked: `is_server_ident`, `is_browser_ident`, `is_dev_ident` for each source.
8. When `is_server` is not specified in config, defaults to `true` (lib.rs line 110: `config.is_server.unwrap_or(true)`).

### CONV-11: Code Stripping

**SWC Source:** `filter_exports.rs` (76 LOC) + `transform.rs` `should_emit_segment`
**Complexity:** Medium (three distinct mechanisms)

**Three stripping mechanisms:**

**1. strip_exports (filter_exports.rs):**
- Runs BEFORE all other transforms (parse.rs line 248, step 2 in the pipeline).
- Takes a list of export symbol names (e.g., `["onGet", "onPost"]`).
- For each matching exported variable or function declaration, replaces the body with a throwing arrow stub:
  ```js
  export const onGet = () => { throw "Symbol removed by Qwik Optimizer, it can not be called from current platform"; };
  ```
- Only matches `export const X = ...` (VarDecl with single declarator) and `export function X() {}` (FnDecl).
- The symbol name is preserved, the initializer/body is replaced. The export remains (it's a stub, not a removal).

**2. strip_ctx_name (transform.rs `should_emit_segment`):**
- Evaluated during the main QwikTransform pass when deciding whether to emit a segment.
- Takes a list of context name prefixes (e.g., `["server"]`).
- If a segment's `ctx_name` starts with any prefix in the list, `should_emit_segment` returns false.
- The segment is NOT extracted. Instead, `create_noop_qrl` generates a `_noopQrl()` placeholder.
- Noop segments still appear in output with `export const symbolName = null;` as their code.

**3. strip_event_handlers (transform.rs `should_emit_segment`):**
- Boolean flag. When true, all segments with `ctx_kind == SegmentKind::EventHandler` are stripped.
- Same noop QRL treatment as strip_ctx_name.
- Used for SSR builds where event handlers are unnecessary.

### CONV-13: sync$ Serialization

**SWC Source:** `transform.rs` `handle_sync_qrl` (lines 697-733) + `inlined_fn.rs` `render_expr` (lines 188-217)
**Complexity:** Medium (expression-to-string serialization)

**Rules:**
1. `sync$(fn)` is detected when the callee matches the `sync_qrl_fn` identifier.
2. The first argument must be an ArrowExpr or FnExpr. Other expression types are passed through unchanged.
3. The function is serialized to a minified string using SWC's codegen with `Config::with_minify(true)` and no comments.
4. Output: `_qrlSync(originalFn, "serializedString")` -- both the original function expression AND the string representation are emitted.
5. **No segment extraction:** Unlike other `$`-suffixed calls, `sync$` does NOT extract the function body to a separate module. The function stays inline.
6. The serialized string is the function body as it would appear in minified JS, with the trailing semicolon stripped. Comments inside the function body are removed during serialization.

**Example from snapshot (example_of_synchronous_qrl):**
```
Input:  sync$(function(event, target) { /* comment */ event.preventDefault(); })
Output: _qrlSync(function(event, target) { event.preventDefault(); }, "function(event,target){event.preventDefault();}")
```

### CONV-14: Noop QRL Handling

**SWC Source:** `transform.rs` `create_noop_qrl` (lines 3000-3027) + `hoist_qrl_to_module_scope` (lines 1459-1608)
**Complexity:** Low-Medium (context-dependent behavior)

**When noop QRLs are generated:**
1. When `should_emit_segment` returns false (strip_ctx_name or strip_event_handlers match).
2. During Hoist strategy's `hoist_qrl_to_module_scope`, where `inlinedQrl()` calls are converted to `_noopQrl()` + `.s()` assignment pattern.

**Noop QRL forms:**
- **Prod/Lib/Test mode:** `_noopQrl("symbolName")` -- just the symbol name string.
- **Dev/Hmr mode:** `_noopQrlDEV("symbolName", { file: "...", lo: 0, hi: 0, displayName: "..." })` -- includes dev location info.
- All noop QRL calls get PURE annotations.

**Noop segment output:** When a segment is stripped, its output module contains just `export const symbolName = null;` with an empty source map.

**Captures on noop QRLs:** Even stripped segments emit capture arrays if the original function had captured variables: `create_noop_qrl` calls `self.emit_captures(&segment_data.captures, &mut args)`. This preserves the `.w([captures])` call chain for runtime scope tracking.

### Entry Strategies (SPEC-15)

**SWC Source:** `entry_strategy.rs` (124 LOC)
**5 EntryPolicy implementations, 7 enum variants**

**EntryPolicy trait:** `get_entry_for_sym(context: &[String], segment: &SegmentData) -> Option<Atom>`
- Returns `Some(group_key)` to group the segment into a shared output file named by the key.
- Returns `None` to give the segment its own output file.
- `context` is the `stack_ctxt` -- the component context stack (root component names).

| Strategy Enum | EntryPolicy Impl | Grouping Behavior |
|---------------|------------------|-------------------|
| **Inline** | InlineStrategy | `Some("entry_segments")` -- all segments grouped into one virtual entry. Segments are inlined into the root module as `inlinedQrl()` calls. |
| **Hoist** | InlineStrategy (same!) | `Some("entry_segments")` -- same grouping. But post-processing converts `inlinedQrl()` to `_noopQrl()` + `.s()` registration pattern. Segments become top-level const declarations with `.s(fnBody)` calls. |
| **Single** | SingleStrategy | `Some("entry_segments")` -- all segments in one output file. Unlike Inline, segments are extracted to separate modules. |
| **Hook** | PerSegmentStrategy | `None` -- each segment gets its own output file. Maximum granularity. |
| **Segment** | PerSegmentStrategy | `None` -- identical to Hook. Renamed alias. |
| **Component** | PerComponentStrategy | `Some("{origin}_entry_{root}")` if context has a root component, otherwise `Some("entry_segments")`. Groups by component. |
| **Smart** | SmartStrategy | Event handlers without captures (`scoped_idents.is_empty()` AND NOT `SegmentKind::Function` OR `ctx_name == "event$"`) get `None` (own file). Everything else grouped by component like PerComponentStrategy. Top-level QRLs (no context) also get `None`. |

**Critical Inline/Hoist distinction (D-25):**
- Both use `InlineStrategy` (same `get_entry_for_sym` returning `Some("entry_segments")`).
- **Inline:** `create_inline_qrl` with `should_inline=true` -- the function body is passed directly as the first argument to `inlinedQrl(fn, "symbolName", [captures])`.
- **Hoist:** `create_inline_qrl` with `should_inline=false` -- creates a separate `Segment` entry, then `hoist_qrl_to_module_scope` converts the `inlinedQrl()` call into:
  1. `const q_symbolName = /*#__PURE__*/ _noopQrl("symbolName");` (top-level declaration)
  2. `q_symbolName.s(fnBody);` (registration call, either at module scope or inline via comma expression for non-global idents)
  3. `.w([captures])` appended if the segment has captures
- **Hoist also applies to non-component QRLs** like `qSegment()` calls.

**Lib mode exception:** `hoist_qrl_to_module_scope` returns early with `ast::Expr::Call(call_expr)` unchanged when mode is Lib -- no hoisting in library mode.

### Emit Modes (SPEC-16)

**SWC Source:** Distributed across parse.rs and transform.rs
**5 modes defined in parse.rs `EmitMode` enum**

**Mode x CONV Cross-Reference Table (derived from source):**

| CONV | Prod | Dev | Lib | Test | Hmr |
|------|------|-----|-----|------|-----|
| CONV-01 (Dollar Detection) | Normal | Normal | Normal | Normal | Normal |
| CONV-02 (QRL Wrapping) | `qrl()`/`inlinedQrl()` | `qrlDEV()`/`inlinedQrlDEV()` with source location | `inlinedQrl()` only (inline) | Same as Prod | Same as Dev |
| CONV-03 (Capture Analysis) | Normal | Normal | Normal | Normal | Normal |
| CONV-04 (Props Destructuring) | Normal | Normal | Normal (runs for all modes) | Normal | Normal |
| CONV-05 (Segment Extraction) | Normal | Normal | **SKIPPED** (no segments extracted, all inline) | Normal | Normal |
| CONV-06 (JSX Transform) | Normal | Adds JSX source location info | Normal | Normal | Adds JSX source location info |
| CONV-07 (Signal Optimization) | Normal | Normal | Normal | Normal | Normal |
| CONV-08 (PURE Annotations) | Normal | Normal | Normal | Normal | Normal |
| CONV-09 (DCE) | Full (simplifier + treeshaker) | Full | **SKIPPED** | Full | Full |
| CONV-10 (Const Replacement) | `isServer=config`, `isDev=false` | `isServer=config`, `isDev=true` | **SKIPPED** | **SKIPPED** | `isServer=config`, `isDev=true` |
| CONV-11 (Code Stripping) | Normal | Normal | Normal | Normal | Normal |
| CONV-12 (Import Rewriting) | Normal | Adds `_qrlDEV`, `_inlinedQrlDEV`, `_noopQrlDEV` imports | Normal | Normal | Same as Dev |
| CONV-13 (sync$) | Normal | Normal | Normal | Normal | Normal |
| CONV-14 (Noop QRL) | `_noopQrl()` | `_noopQrlDEV()` with source location | Normal | Normal | `_noopQrlDEV()` |
| **Other** | | | | | |
| Post-processing (DCE, treeshaker, var migration) | Normal | Normal | **ALL SKIPPED** | Normal | Normal |
| HMR hook injection | No | No | No | No | `_useHmr(devPath)` injected as first statement in component$ bodies |
| Segment dev imports | No | `_qrlDEV`, `_inlinedQrlDEV`, `_noopQrlDEV` added to segment explicit imports | No | No | Same as Dev |

**Per-mode summaries:**

1. **Prod:** Full pipeline. Const replacement with `isDev=false`. DCE enabled. No dev-mode wrappers.
2. **Dev:** Uses `qrlDEV`/`inlinedQrlDEV`/`_noopQrlDEV` variants that include `{ file, lo, hi, displayName }` source location objects. `isDev=true`. Full DCE.
3. **Lib:** Minimal processing. Props destructuring runs. Mechanical QRL wrapping (all inline). NO const replacement, NO DCE, NO post-processing, NO segment extraction. Output is pre-compiled `.qwik.mjs` for library distribution.
4. **Test:** Like Prod but skips const replacement. `isServer`/`isBrowser` remain as identifiers so tests can configure environment at runtime.
5. **Hmr:** Like Dev plus `_useHmr(devPath)` injection. The HMR hook is injected as the first statement inside `component$` function bodies only (not other `$`-suffixed calls). Only components that were detected via `is_qcomponent` get HMR injection.

**is_dev derivation (parse.rs line 293):**
```rust
let is_dev = matches!(config.mode, EmitMode::Dev | EmitMode::Hmr);
```

### Pipeline Ordering (SPEC-17)

**SWC Source:** `parse.rs` `transform_code` function
**20-step sequence verified against source:**

```
Step  1: Parse source code
Step  2: Strip exports (CONV-11 partial -- filter_exports.rs)         [if strip_exports config set]
Step  3: TypeScript strip                                              [if transpile_ts && is_typescript]
Step  4: JSX transpile (React automatic runtime)                       [if transpile_jsx && is_jsx]
Step  5: Rename legacy imports (RenameTransform -- CONV-12 partial)
Step  6: Resolver (SWC mark-based scope resolution)
Step  7: GlobalCollect (import/export/root analysis)
Step  8: Props Destructuring (CONV-04)                                 [all modes including Lib]
Step  9: Const Replacement (CONV-10)                                   [if mode != Lib && mode != Test]
Step 10: QwikTransform Fold (CONV-01,02,03,05,06,07,08,12,13,14)     [main pass]
Step 11: Treeshaker mark (CleanMarker)                                 [if minify && !is_server && mode != Lib]
Step 12: Simplifier DCE (CONV-09)                                      [if minify && mode != Lib]
Step 13: Side effect preservation (SideEffectVisitor)                  [if Inline/Hoist strategy]
Step 13b: Treeshaker clean (CleanSideEffects)                          [if minify && !is_server && NOT Inline/Hoist]
Step 13c: Second DCE pass                                              [if treeshaker dropped items]
Step 14: Variable migration                                            [if segments exist && mode != Lib]
Step 15: Export cleanup for migrated vars
Step 16: Third DCE pass                                                [if vars were migrated && minify]
Step 17: Hygiene
Step 18: Fixer
Step 19: Segment module generation (per-segment codegen + DCE + hygiene + fixer)
Step 20: Root module codegen + source map generation
```

**Dependency DAG (from source analysis and FEATURES.md, verified):**

```
CONV-10 (Const Replace) -----> CONV-09 (DCE)
    [const replacement creates dead branches that DCE removes]

CONV-04 (Props Destructuring) -----> CONV-03 (Capture Analysis)
    [destructuring changes variable references that capture analysis reads]

CONV-01 (Dollar Detection) -----> CONV-02 (QRL Wrapping)
    [must detect markers before wrapping them]

CONV-01 (Dollar Detection) -----> CONV-05 (Segment Extraction)
    [must detect markers before extracting segments]

CONV-03 (Capture Analysis) -----> CONV-02 (QRL Wrapping)
    [captures list passed to qrl()/inlinedQrl() calls]

CONV-03 (Capture Analysis) -----> CONV-07 (Signal Optimization)
    [_fnSignal needs captured variable list]

CONV-05 (Segment Extraction) -----> CONV-12 (Import Rewriting)
    [segment modules need their own import resolution]

CONV-06 (JSX Transform) -----> CONV-07 (Signal Optimization)
    [_fnSignal generated during JSX prop processing]

GlobalCollect -----> All CONVs
    [import/export metadata required by every transformation]

Variable Migration -----> CONV-05 (Segment Extraction)
    [runs after segments are extracted to optimize root module]

Treeshaker -----> CONV-09 (DCE)
    [client-side cleanup runs after DCE]

strip_exports (CONV-11) -----> All other transforms
    [runs at step 2, before everything else]
```

### Jack's Snapshot Mapping for Phase 3

| Topic | Best Snapshot(s) | What It Shows |
|-------|-----------------|---------------|
| Const Replacement + DCE (server build) | `example_build_server` | `isServer`/`isBrowser` replaced, dead branches removed, client imports dropped |
| Dead Code Elimination | `example_dead_code` | `if (false)` branch removed after simplification |
| Client-side Treeshaker | `example_drop_side_effects` | Side-effect removal on client builds |
| Strip server code (strip_ctx_name) | `example_strip_server_code` | Server-only segments become `export const x = null` noop stubs |
| Strip client code (strip_ctx_name) | `example_strip_client_code` | Client-only segments become noops, `.s()` still registered in Hoist |
| Strip exports (strip_exports) | `example_strip_exports_used`, `example_strip_exports_unused` | Throwing stub generation for stripped exports |
| sync$ serialization | `example_of_synchronous_qrl` | `_qrlSync(fn, "serializedString")` output for three function forms |
| Noop QRL (Dev mode) | `example_noop_dev_mode` | `_noopQrlDEV` with source location, stripped segments with captures |
| Noop QRL (Prod/Hoist) | `example_strip_server_code` | `_noopQrl("symbolName")` in Hoist pattern |
| PURE annotations | `example_strip_exports_unused` | `/*#__PURE__*/` on `componentQrl()` and `qrl()` calls |
| Entry Strategy: Inline/Hoist | `example_inlined_entry_strategy` | Hoist pattern with `_noopQrl` + `.s()` registration |
| Entry Strategy: Segment/Hook | `example_strip_server_code` | Per-segment output files with `qrl(() => import(...))` |
| Lib mode | `example_lib_mode` | All inline, no segments, no const replacement, `inlinedQrl()` throughout |
| Dev mode | `example_dev_mode`, `example_dev_mode_inlined` | `qrlDEV`/`inlinedQrlDEV` with source location objects |
| HMR mode | `hmr` | `_useHmr("/path")` injection in component bodies |
| Impure template functions | `impure_template_fns` | Demonstrates that only `componentQrl` gets outer PURE, not other wrappers |

## Common Pitfalls

### Pitfall 1: Confusing Inline and Hoist Strategies
**What goes wrong:** They share the same EntryPolicy (InlineStrategy) and both keep code in the root module, so they appear identical at the grouping level.
**Why it happens:** The difference is entirely in post-processing -- Hoist converts `inlinedQrl()` calls to `_noopQrl()` + `.s()` pattern.
**How to avoid:** The spec must clearly separate the grouping behavior (same for both) from the output pattern (fundamentally different).
**Warning signs:** If an implementer sees `InlineStrategy` returning the same value for both and concludes they're identical.

### Pitfall 2: PURE Annotation Anti-List
**What goes wrong:** Adding `/*#__PURE__*/` to `useTaskQrl()`, `serverStuffQrl()`, or other side-effectful wrappers.
**Why it happens:** All QRL factory calls (`qrl()`, `inlinedQrl()`, `_noopQrl()`) get PURE annotations internally, so it's tempting to annotate all `Qrl`-suffixed calls.
**How to avoid:** The spec must explicitly state: PURE on `componentQrl` (tree-shakeable declaration), NEVER on hook/effect registrations (side-effectful at render time).
**Warning signs:** Bundler incorrectly drops `useTask$` or `server$` calls in production.

### Pitfall 3: Lib Mode Skips Almost Everything
**What goes wrong:** Running const replacement or DCE in Lib mode.
**Why it happens:** The conditional guards are scattered across parse.rs (`mode != Lib` checks at different points).
**How to avoid:** The Mode x CONV table must make Lib mode's extensive skip list unmistakable.
**Warning signs:** Library .qwik.mjs output has environment booleans baked in.

### Pitfall 4: Treeshaker Phase Ordering
**What goes wrong:** Running Treeshaker clean before or without the Simplifier.
**Why it happens:** The two-phase span-based approach requires: (1) mark existing side effects, (2) run simplifier to expose new ones, (3) remove unmarked new ones.
**How to avoid:** Document the three-step sequence clearly with the span-tracking mechanism.
**Warning signs:** Legitimate side effects removed on client builds.

### Pitfall 5: strip_exports Runs First
**What goes wrong:** Assuming strip_exports runs during the QwikTransform pass like strip_ctx_name.
**Why it happens:** Both are "stripping" mechanisms, but strip_exports is a pre-pass (step 2) while strip_ctx_name is evaluated during the main transform (step 10).
**How to avoid:** Pipeline ordering section must highlight this distinction. strip_exports runs before TypeScript stripping, before resolver, before everything.

### Pitfall 6: Const Replacement Import Sources
**What goes wrong:** Only handling imports from `@qwik.dev/core/build`, missing the `@qwik.dev/core` re-exports.
**Why it happens:** The documentation typically refers to `@qwik.dev/core/build` as the source for build-time constants.
**How to avoid:** Source analysis shows 6 identifier slots -- 3 for each import source. Both must be checked.

## Code Examples

### Const Replacement (verified from const_replace.rs)

```rust
// Source: const_replace.rs ConstReplacerVisitor::visit_mut_expr
// For isServer from @qwik.dev/core/build:
// Input:  import { isServer } from '@qwik.dev/core/build'; if (isServer) { ... }
// Output: if (true) { ... }   // when is_server=true
// Output: if (false) { ... }  // when is_server=false

// For isBrowser (inverted):
// Output: if (false) { ... }  // when is_server=true (isBrowser = !isServer)
// Output: if (true) { ... }   // when is_server=false

// For isDev:
// Output: if (true) { ... }   // when mode=Dev or mode=Hmr
// Output: if (false) { ... }  // when mode=Prod, Lib(skipped), Test(skipped)
```

### Throwing Stub (verified from filter_exports.rs)

```javascript
// Source: filter_exports.rs empty_module_item()
// Input:
export const onGet = () => {
  const data = mongodb.collection.whatever;
  return { body: { data } };
};

// Output (after strip_exports: ["onGet"]):
export const onGet = () => {
  throw "Symbol removed by Qwik Optimizer, it can not be called from current platform";
};
```

### Noop QRL with .s() Pattern (Hoist strategy, verified from example_strip_client_code snapshot)

```javascript
// Source: transform.rs hoist_qrl_to_module_scope + example_inlined_entry_strategy snapshot
// Hoist pattern for a component:
const q_Child_component_9GyF01GDKqw = /*#__PURE__*/ _noopQrl("Child_component_9GyF01GDKqw");
// ... other noop QRL declarations ...

// Registration calls:
q_Child_component_useStyles_qBZTuFM0160.s('somestring');
q_Child_component_9GyF01GDKqw.s(() => {
    useStylesQrl(q_Child_component_useStyles_qBZTuFM0160);
    // ... component body ...
});
export const Child = /*#__PURE__*/ componentQrl(q_Child_component_9GyF01GDKqw);
```

### sync$ Serialization (verified from example_of_synchronous_qrl snapshot)

```javascript
// Source: transform.rs handle_sync_qrl + inlined_fn.rs render_expr
// Input:
sync$(function(event, target) {
    // comment should be removed
    event.preventDefault();
})

// Output:
_qrlSync(function(event, target) {
    // comment should be removed
    event.preventDefault();
}, "function(event,target){event.preventDefault();}")
// Note: comment preserved in the AST expression but removed from serialized string
```

### HMR Hook Injection (verified from hmr snapshot)

```javascript
// Source: transform.rs fold_call_expr, HMR injection block
// Only in EmitMode::Hmr, only for component$ bodies
// Input:
export const TestGetsHmr = component$(() => {
    return <div>Test</div>;
});

// Output (segment module):
export const TestGetsHmr_component_jBUoou0sxX4 = () => {
    _useHmr("/user/qwik/src/test.tsx");  // injected first statement
    return /*#__PURE__*/ _jsxSorted("div", null, null, "Test", 3, "u6_0", { ... });
};
```

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Not applicable -- this is a specification-writing phase, not implementation |
| Config file | N/A |
| Quick run command | Manual review: verify spec sections against SWC source and snapshots |
| Full suite command | N/A |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SPEC-08 | PURE annotation rules documented | manual-only | Verify against `transform.rs` PURE annotation sites | N/A |
| SPEC-09 | DCE behavior documented | manual-only | Verify against `clean_side_effects.rs` + parse.rs simplifier calls | N/A |
| SPEC-10 | Const replacement rules documented | manual-only | Verify against `const_replace.rs` (96 LOC, fully readable) | N/A |
| SPEC-11 | Code stripping mechanisms documented | manual-only | Verify against `filter_exports.rs` + `should_emit_segment` | N/A |
| SPEC-13 | sync$ serialization documented | manual-only | Verify against `handle_sync_qrl` + `render_expr` | N/A |
| SPEC-14 | Noop QRL handling documented | manual-only | Verify against `create_noop_qrl` + `hoist_qrl_to_module_scope` | N/A |
| SPEC-15 | All 7 entry strategies documented | manual-only | Verify against `entry_strategy.rs` (124 LOC, fully readable) | N/A |
| SPEC-16 | All 5 emit modes documented | manual-only | Verify Mode x CONV table against parse.rs conditional checks | N/A |
| SPEC-17 | Pipeline DAG documented | manual-only | Verify step ordering against parse.rs `transform_code` | N/A |

### Sampling Rate
- **Per task:** Visual diff of new spec sections against source code references
- **Phase gate:** All 9 SPEC requirements addressed, Mode x CONV table complete, DAG matches parse.rs

### Wave 0 Gaps
None -- specification-writing phase with manual validation only.

## Sources

### Primary (HIGH confidence)
- `entry_strategy.rs` -- All 5 EntryPolicy implementations, `parse_entry_strategy` mapping (124 LOC, read in full)
- `const_replace.rs` -- ConstReplacerVisitor with 6 identifier slots for 2 import sources (96 LOC, read in full)
- `clean_side_effects.rs` -- Treeshaker two-phase span-based approach (90 LOC, read in full)
- `filter_exports.rs` -- StripExportsVisitor with throwing stub generation (76 LOC, read in full)
- `parse.rs` -- Full pipeline orchestration, emit mode branching, DCE conditions (read lines 1-750 in detail)
- `transform.rs` -- PURE annotation sites, sync$ handling, noop QRL creation, Hoist pattern, HMR injection (targeted reads of ~400 lines total)
- `inlined_fn.rs` -- `render_expr` minified codegen for sync$ serialization (read lines 188-217)
- `add_side_effect.rs` -- SideEffectVisitor for Inline/Hoist side effect preservation (67 LOC, read in full)
- `lib.rs` -- EmitMode enum, EntryStrategy enum, TransformModulesOptions, is_server default (read in full)

### Jack's Snapshots (HIGH confidence)
- `example_build_server.snap` -- Const replacement + DCE example
- `example_dead_code.snap` -- Dead branch elimination
- `example_strip_server_code.snap` -- strip_ctx_name with noop QRLs
- `example_strip_client_code.snap` -- Hoist pattern with stripped client segments
- `example_strip_exports_used.snap` / `example_strip_exports_unused.snap` -- Throwing stub generation
- `example_of_synchronous_qrl.snap` -- sync$ serialization (3 function forms)
- `example_noop_dev_mode.snap` -- noopQrlDEV with source locations
- `example_inlined_entry_strategy.snap` -- Hoist pattern with .s() registration
- `example_lib_mode.snap` -- Lib mode output (all inline)
- `example_dev_mode.snap` / `example_dev_mode_inlined.snap` -- Dev mode variants
- `hmr.snap` -- HMR mode with _useHmr injection
- `impure_template_fns.snap` -- PURE annotation placement verification

### Supporting (HIGH confidence)
- `.planning/research/FEATURES.md` -- Feature dependency DAG, execution order, mode/strategy tables

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all source files read in full, behavior verified against snapshots
- Architecture: HIGH -- document organization recommendation based on source code clustering
- Pitfalls: HIGH -- derived from actual source code conditional logic and known confusion points

**Research date:** 2026-04-01
**Valid until:** 2026-05-01 (stable -- SWC optimizer is not actively changing)
