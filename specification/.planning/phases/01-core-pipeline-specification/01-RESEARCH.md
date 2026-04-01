# Phase 1: Core Pipeline Specification - Research

**Researched:** 2026-04-01
**Domain:** Behavioral specification of Qwik optimizer core QRL extraction pipeline
**Confidence:** HIGH

## Summary

This phase writes the behavioral specification for the core QRL extraction pipeline: dollar detection, QRL wrapping, capture analysis, segment extraction, import rewriting, and supporting infrastructure (GlobalCollect, variable migration, hash generation, path resolution, source maps). The deliverable is sections of a single comprehensive markdown document -- not code.

Research focused on reading the SWC source code (transform.rs, code_move.rs, collector.rs, dependency_analysis.rs, parse.rs, words.rs) to extract exact behavioral rules, cross-referencing with Jack's snapshot files and OXC implementation for validation. The SWC source reveals a precise 20-step pipeline with specific ordering constraints, a capture analysis system that uses a decl_stack + IdentCollector approach, a DefaultHasher-based hash algorithm with base64 encoding, and a code_move module that resolves imports for segment modules by querying the GlobalCollect metadata.

**Primary recommendation:** Write specification sections in pipeline execution order (parse -> collect -> pre-transforms -> core transform -> emit) per D-01, using the SWC source as ground truth per D-05. Each section needs behavioral rules, 2-3 input/output examples from Jack's snapshots, and SWC source file references for traceability.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Document organized in pipeline execution order (parse -> collect -> pre-transforms -> core transform -> emit), not CONV numbering
- **D-02:** Mermaid flowchart diagram at the top showing full transformation pipeline with data flow
- **D-03:** Cross-referencing approach is Claude's discretion
- **D-04:** Each CONV section: behavioral rules (trigger, behavior, edge cases) + 2-3 input/output examples. Enough detail to implement from without SWC source.
- **D-05:** SWC v2 optimizer is source of truth. When SWC and Jack's OXC disagree, SWC wins.
- **D-06:** SWC source file references for traceability (e.g., "Source: transform.rs:298-350")
- **D-07:** Scott's OXC conversion used for OXC pattern examples in migration notes
- **D-08:** AST JSON dumps at Claude's discretion -- only where they add clarity
- **D-09:** Capture taxonomy as Mermaid decision tree + table with 8 categories
- **D-10:** All 16 of Jack's capture edge cases as named spec test cases (CAPTURE-EDGE-01 through CAPTURE-EDGE-16)
- **D-11:** Self-import reclassification format at Claude's discretion
- **D-12:** Variable migration as separate spec section cross-referencing capture analysis
- **D-13:** Examples show input source code then each output module (root + all segments)
- **D-14:** 2-3 examples per CONV section: basic, captures/nesting, edge case
- **D-15:** Segment metadata (SegmentAnalysis JSON) at Claude's discretion
- **D-16:** Examples named descriptively with Jack's snapshot name in parentheses

### Claude's Discretion
- Cross-referencing style (inline vs dependency table vs both) -- D-03
- AST JSON dump inclusion per example -- D-08
- Self-import reclassification presentation format -- D-11
- Segment metadata inclusion per example -- D-15

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SPEC-01 | Dollar Detection (CONV-01) | SWC transform.rs:189-202 marker_functions construction; words.rs QRL_SUFFIX='$'; both imported markers from @qwik.dev/core AND locally-defined $-suffixed exports are detected |
| SPEC-02 | QRL Wrapping (CONV-02) | SWC transform.rs:1888-2062 create_qrl/create_inline_qrl/create_noop_qrl; qrl()/inlinedQrl() with symbol name + captures; dev mode variants qrlDEV/inlinedQrlDEV with source location |
| SPEC-03 | Capture Analysis (CONV-03) | SWC transform.rs:820-1075 _create_synthetic_qsegment capture flow; collector.rs IdentCollector; compute_scoped_idents at line 4894; PITFALLS.md 8-category taxonomy |
| SPEC-05 | Segment Extraction (CONV-05) | SWC transform.rs:1110-1145 create_segment; code_move.rs:105-450 new_module/NewModuleCtx; parse.rs:446-583 segment module construction loop |
| SPEC-12 | Import Rewriting (CONV-12) | SWC code_move.rs:66-277 resolve_import_for_id + import generation; collector.rs GlobalCollect.import() for synthetic imports; parse.rs:278 RenameTransform |
| SPEC-21 | GlobalCollect | SWC collector.rs:56-528 global_collect function + Visit impl; imports/exports/root IndexMaps; ImportKind enum |
| SPEC-22 | Variable Migration | SWC dependency_analysis.rs full file; analyze_root_dependencies -> build_root_var_usage_map -> find_migratable_vars pipeline; parse.rs:400-435 apply_variable_migration |
| SPEC-23 | Hash Generation | SWC transform.rs:204-210 file_hash (DefaultHasher + scope + rel_path); transform.rs:393-421 register_context_name (hash + base64 encoding); base64 function at line 4725 |
| SPEC-24 | Path Resolution | SWC parse.rs:207-210 parse_path; transform.rs:1127-1131 canonical_filename + import_path construction; get_canonical_filename at line 4910 |
| SPEC-25 | Source Map Generation | SWC parse.rs:701-753 emit_source_code; SWC SourceMap::build_source_map for main module; segment modules get individual source maps via same codegen path |
| SPEC-30 | Capture Taxonomy with Edge Cases | PITFALLS.md 8 categories; Jack's Phase 24 Plans 05/07/10 edge case fixes; example_multi_capture.snap demonstrates captures with _captures[N] destructuring |
</phase_requirements>

## Standard Stack

This is a specification-writing phase. No libraries to install. The "stack" is the source material.

### Source Material (Primary)
| Source | Location | Purpose | Lines |
|--------|----------|---------|-------|
| transform.rs | `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` | Dollar detection, capture analysis, QRL wrapping, segment creation | ~5,157 |
| code_move.rs | `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/code_move.rs` | Segment module construction, import resolution | ~1,521 |
| collector.rs | `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/collector.rs` | GlobalCollect: import/export/root-declaration analysis | ~528 |
| dependency_analysis.rs | `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/dependency_analysis.rs` | Variable migration dependency mapping | ~587 |
| parse.rs | `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/parse.rs` | Pipeline orchestration (20-step), source map generation, types | ~1,798 |
| words.rs | `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/words.rs` | String constants (QRL_SUFFIX, function names) | ~54 |

### Source Material (Reference)
| Source | Location | Purpose |
|--------|----------|---------|
| SWC snapshots | `/Users/scottweaver/Projects/qwik-oxc-optimizer/swc-snapshots/*.snap` | 162 verified input/output pairs for examples |
| OXC snapshots | `/Users/scottweaver/Projects/qwik-oxc-optimizer/oxc-snapshots/*.snap` | Comparison outputs showing cosmetic differences |
| Jack's OXC impl | `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/` | Reference for OXC migration notes |
| PITFALLS.md | `.planning/research/PITFALLS.md` | 15 catalogued pitfalls with capture taxonomy |
| FEATURES.md | `.planning/research/FEATURES.md` | 14 CONVs with dependency DAG and pipeline ordering |

## Architecture Patterns

### Spec Document Structure (per D-01)
```
specification/
  qwik-optimizer-spec.md        # Single comprehensive spec document
    ## Pipeline Overview         # Mermaid diagram (D-02)
    ## Stage 1: Parse            # Parsing, syntax detection
    ## Stage 2: GlobalCollect    # SPEC-21
    ## Stage 3: Pre-Transforms   # Const replacement, export stripping, props destructuring
    ## Stage 4: Core Transform   # The main fold/traverse pass
      ### Dollar Detection       # SPEC-01 / CONV-01
      ### Capture Analysis       # SPEC-03 / CONV-03 + SPEC-30
      ### QRL Wrapping           # SPEC-02 / CONV-02
      ### Segment Extraction     # SPEC-05 / CONV-05
      ### Import Rewriting       # SPEC-12 / CONV-12
    ## Stage 5: Post-Transform   # Variable migration, DCE, hygiene
      ### Variable Migration     # SPEC-22
    ## Infrastructure
      ### GlobalCollect Details  # SPEC-21 (expanded)
      ### Hash Generation        # SPEC-23
      ### Path Resolution        # SPEC-24
      ### Source Map Generation   # SPEC-25
```

### Pipeline Execution Order (from SWC parse.rs)

The 20-step sequence, grouped into logical stages for the spec:

```
STAGE 1: PARSE
  1. Parse source code (detect TS/JSX from extension)
  2. Strip exports (strip_exports config)
  3. TypeScript strip (if transpile_ts)
  4. JSX transpile to jsx() calls (if transpile_jsx, React automatic runtime)
  5. Rename legacy imports (@builder.io/qwik -> @qwik.dev/core)
  6. Resolver (SWC mark-based scope resolution)

STAGE 2: COLLECT
  7. GlobalCollect (import/export/root-declaration analysis)

STAGE 3: PRE-TRANSFORMS
  8. Props Destructuring (all modes including Lib)
  9. Const Replacement (if not Lib/Test mode)

STAGE 4: CORE TRANSFORM (single QwikTransform fold pass)
  10. Dollar detection + QRL wrapping + Capture analysis +
      Segment extraction + Import rewriting + JSX transforms +
      Signal optimization + PURE annotations + sync$ + noop QRL

STAGE 5: POST-TRANSFORM
  11. Treeshaker mark (if minify && client)
  12. Simplifier DCE
  13. Side effect preservation (if Inline/Hoist) OR Treeshaker clean
  14. Variable migration
  15. Export cleanup for migrated vars
  16. Second DCE pass (if vars migrated)
  17. Hygiene + Fixer

STAGE 6: EMIT
  18. Codegen root module (with source maps)
  19. Build segment modules (code_move::new_module per segment)
  20. Codegen segment modules (with individual source maps)
```

**Phase 1 scope covers**: Stages 2, 4 (dollar detection + capture analysis + QRL wrapping + segment extraction + import rewriting only -- NOT JSX/signal/PURE/sync$/noop), 5 (variable migration only), and all Infrastructure sections.

### Example Format Pattern (per D-13, D-14, D-16)

Each CONV section should follow this pattern:

```markdown
### Example: Basic Dollar Extraction (example_6)

**Input:**
\`\`\`typescript
import { $, component$ } from '@qwik.dev/core';
export const sym1 = $((ctx) => console.log("1"));
\`\`\`

**Root module output (test.tsx):**
\`\`\`javascript
import { qrl } from "@qwik.dev/core";
const q_sym1_aXUrPXX5Lak = /*#__PURE__*/ qrl(
  ()=>import("./test.tsx_sym1_aXUrPXX5Lak"), "sym1_aXUrPXX5Lak"
);
export const sym1 = q_sym1_aXUrPXX5Lak;
\`\`\`

**Segment module output (test.tsx_sym1_aXUrPXX5Lak.tsx):**
\`\`\`javascript
export const sym1_aXUrPXX5Lak = (ctx)=>console.log("1");
\`\`\`
```

### Key Behavioral Rules Extracted from SWC Source

#### Dollar Detection (CONV-01) -- Source: transform.rs:189-202

1. **Imported markers**: Any named import from `@qwik.dev/core` whose specifier ends with `$` is a marker function. Detected via `global_collect.imports` iteration with `specifier.ends_with(QRL_SUFFIX)`.
2. **Local markers**: Any locally-defined export whose name ends with `$` is also a marker function. Detected via `global_collect.export_local_ids()` with `id.0.ends_with(QRL_SUFFIX)`.
3. **Special cases**: `sync$` has its own handler (not segment extraction). `$` (bare dollar) is the generic marker (`QSEGMENT`). `component$` triggers PURE annotation.
4. **Callee conversion**: When a marker `foo$` is detected, callee is replaced with `fooQrl`. The `Qrl` suffix replaces the `$` suffix via `convert_qrl_word()` (transform.rs:179-187).

#### Capture Analysis (CONV-03) -- Source: transform.rs:960-1005

The capture analysis algorithm works as follows:

1. **Collect descendant identifiers**: `IdentCollector` visits the `$()` callback body, collecting all referenced identifiers (filtered by SyntaxContext != empty, excluding `undefined`/`NaN`/`Infinity`/`null`).
2. **Partition declaration stack**: `decl_stack` entries are partitioned into `Var`-type (capturable) and non-`Var` (function/class -- reported as errors).
3. **Compute scoped identifiers**: `compute_scoped_idents()` (line 4894) finds identifiers that appear in BOTH the descendant set AND the declaration stack. These are the "captured" variables.
4. **Filter function parameters**: Parameters of the `$()` callback itself are excluded from captures (they are local to the segment).
5. **Global vs local**: Identifiers in `global_collect.imports`, `global_collect.exports`, or `global_collect.root` are NOT captures -- they are available via imports in the segment module (either re-emitted imports or self-imports from the parent module).

**The 8 Capture Categories** (from PITFALLS.md, verified against SWC source):

| Category | Is Capture? | How Resolved in Segment | SWC Mechanism |
|----------|-------------|------------------------|---------------|
| 1. Module-level declarations | NO | Self-import from `./module_stem` | `global_collect.has_export_symbol()` + `ensure_export()` in create_synthetic_qsegment |
| 2. User-code imports | NO | Re-emitted import statement | `resolve_import_for_id()` in code_move.rs |
| 3. Outer-scope local variables | YES | `_captures[N]` destructuring | `compute_scoped_idents()` returns them in `scoped_idents` |
| 4. Loop iteration variables | YES | Same as #3 (captured) | Iteration vars added to `decl_stack` via `iteration_var_stack` |
| 5. Destructured component props | YES (as `_rawProps`) | Captured after props_destructuring pass transforms them | Props destructuring runs BEFORE core transform |
| 6. TypeScript type-only imports | NO | Erased at compile time | TS strip runs before GlobalCollect |
| 7. Shadowed variables | NO (inner wins) | The inner binding is local to the segment | `collect_local_declarations_from_expr()` filters them in `get_local_idents` |
| 8. Function/class declarations in scope | NOT captures but ERROR | Diagnostic emitted: "Reference to identifier X can not be used inside a Qrl($) scope because it's a function" | `invalid_decl` partition in `_create_synthetic_qsegment` |

**Self-import reclassification** (the single most impactful behavior):
When a segment references a module-level declaration (const/function/class/enum at root scope), the optimizer does NOT add it as a `_captures[N]` entry. Instead:
- `ensure_export()` is called (transform.rs:1024-1026), which adds a synthetic `_auto_X` named export to the root module
- `code_move::new_module()` sees this identifier in `local_idents`, resolves it via `resolve_export_for_id()`, and generates `import { _auto_X as X } from "./module_stem"` in the segment
- The `captures` field in SegmentAnalysis is `false` for these segments

This is observable in `example_capturing_fn_class.snap`: `hola()` and `Thing` are referenced in a nested `$()` but NOT captured -- they appear directly in the segment code because they are function/class declarations (which produce errors) while the segment's `captures: false`.

#### QRL Wrapping (CONV-02) -- Source: transform.rs:1888-2062

Three QRL creation paths:

1. **`create_qrl()`** (line 1888): For segment (non-inline) strategy. Produces:
   ```javascript
   qrl(() => import("./segment_path"), "symbol_name", [captures...])
   ```
   In dev mode, uses `qrlDEV` with additional `{file, lo, hi, displayName}` argument.

2. **`create_inline_qrl()`** (line 1945): For Inline/Hoist strategy. Produces:
   ```javascript
   inlinedQrl(fn_expr, "symbol_name", [captures...])
   ```
   In dev mode, uses `inlinedQrlDEV` with source location.

3. **`create_noop_qrl()`** (line 3000): When `should_emit_segment()` returns false (stripped via `strip_ctx_name` or `strip_event_handlers`). Produces:
   ```javascript
   _noopQrl("symbol_name")  // or _noopQrlDEV in dev mode
   ```

**Captures emission** (line 2013-2029): When `scoped_idents` is non-empty, an array literal `[capture1, capture2, ...]` is appended as the last argument to qrl/inlinedQrl.

#### Hash Generation (SPEC-23) -- Source: transform.rs:204-421

Two-level hashing:

1. **File hash** (line 204-210): `DefaultHasher` seeded with optional `scope` + `rel_path` (forward-slashed). Used for JSX key generation prefix.

2. **Symbol hash** (line 393-421): `DefaultHasher` seeded with:
   - `scope` (if set, written first)
   - `rel_path` (forward-slashed local filename)
   - `display_name` (escaped, deduplicated name)

   The hash is converted to a string via `base64()` (line 4725):
   ```rust
   base64::engine::general_purpose::URL_SAFE_NO_PAD
       .encode(nu.to_le_bytes())
       .replace(['-', '_'], "0")
   ```

   Note: The hash uses `u64.to_le_bytes()` (little-endian), then URL_SAFE_NO_PAD base64, then replaces `-` and `_` with `0`.

3. **Symbol name** format:
   - Dev/Test/Hmr/Lib mode: `{display_name}_{hash64}` (e.g., `sym1_aXUrPXX5Lak`)
   - Prod mode: `s_{hash64}` (e.g., `s_aXUrPXX5Lak`)

4. **Display name** construction:
   - `escape_sym()` (line 4612): replaces non-alphanumeric chars with `_`, trims/squashes consecutive underscores
   - `stack_ctxt.join("_")` builds the context hierarchy (function names, JSX element names)
   - Prepends `_` if starts with digit
   - Deduplication: `segment_names` HashMap tracks count; appends `_N` for duplicates
   - Final display_name: `{file_name}_{display_name}` (e.g., `test.tsx_sym1`)

5. **Canonical filename**: `{display_name}_{hash}` where hash is the last `_`-delimited segment of symbol_name (line 4910-4912).

#### Segment Extraction (CONV-05) -- Source: transform.rs:1110-1145, code_move.rs

The `create_segment()` function:
1. Computes `canonical_filename` from display_name + symbol_name
2. Gets `entry` from entry_policy for bundler chunking
3. Builds `import_path`: `"./" + canonical_filename` (+ extension if explicit_extensions)
4. Creates the `qrl()` call via `create_qrl()`
5. Pushes a `Segment` struct to `self.segments`

**Segment module construction** (`new_module` in code_move.rs:122+):
1. If segment has `scoped_idents` (captures): adds `import { _captures } from "@qwik.dev/core"` and wraps function body with `const captured = _captures[N]` destructuring
2. Resolves imports for `local_idents`: checks GlobalCollect imports, then falls back to self-import from parent module (via `resolve_export_for_id`)
3. Adds `extra_top_items` (hoisted QRL declarations needed by this segment)
4. Adds `migrated_root_vars` (variable declarations moved from root)
5. Orders items by dependency (topological sort)
6. Deduplicates by symbol name
7. Creates named export: `export const {name} = {expr}`

**Nested segments**: The `segment_stack` tracks nesting. `parent_segment` is set to `segment_stack.last()`. A component's nested segments (event handlers, useTask$, etc.) have `parent` pointing to the component segment.

#### Import Rewriting (CONV-12) -- Source: code_move.rs, collector.rs

1. **Legacy rename**: `RenameTransform` (rename_imports.rs) rewrites `@builder.io/qwik` to `@qwik.dev/core` before GlobalCollect runs.

2. **Consumed import stripping**: After the QwikTransform fold, imports for `$`-suffixed functions that were consumed (converted to `Qrl` equivalents) are no longer referenced. SWC's DCE pass removes them.

3. **Synthetic import addition**: During transform, `ensure_core_import()` registers new imports (qrl, componentQrl, _jsxSorted, etc.) in GlobalCollect's synthetic imports list. These are emitted in the final codegen.

4. **Per-segment import resolution** (code_move.rs:200-276):
   - For each identifier used by a segment: check GlobalCollect imports (exact match first, then unique-by-symbol fallback)
   - If not an import but is an export: generate self-import `import { _auto_X as X } from "./module_stem"`
   - Collision detection: if same local name needed for different sources, rename with `_N` suffix

#### GlobalCollect (SPEC-21) -- Source: collector.rs

A single read-only pass over the AST collecting:
- **imports**: `IndexMap<Id, Import>` -- every import specifier with source, kind (Named/Default/All), and whether synthetic
- **exports**: `IndexMap<Atom, ExportInfo>` -- every exported name with local_id and possible rename
- **root**: `IndexMap<Id, Span>` -- every top-level declaration (var/fn/class/TsEnum)
- **canonical_ids**: `HashMap<Atom, Id>` -- maps symbol names to their canonical Id (first seen)

Key methods used during transform:
- `is_global(id)`: returns true if id is an import, export, or root declaration
- `import(specifier, source)`: ensures an import exists, returns local Id (creates synthetic if needed)
- `add_export(id, exported)`: registers an export (used by ensure_export for self-imports)
- `remove_root_and_exports_for_id(id)`: cleanup after variable migration

#### Variable Migration (SPEC-22) -- Source: dependency_analysis.rs

Post-transform optimization that moves root-level declarations exclusively used by one segment into that segment:

1. `analyze_root_dependencies(module, global_collect)`: Builds a map of root var -> its dependencies (other identifiers referenced in its initializer). Tracks whether each var is user-exported.

2. `build_root_var_usage_map(segments, root_deps)`: For each segment, check which root vars appear in its `local_idents` or `scoped_idents`.

3. `build_main_module_usage_set(module, root_deps)`: Find root vars still referenced by non-declaration, non-import statements in the main module.

4. `find_migratable_vars(segments, root_deps, usage_map, main_usage)`: A var is migratable if:
   - Used by exactly one segment
   - Not exported by user code
   - Not an import
   - Not still referenced by main module
   - Transitive dependencies are also migratable
   - Safety filter: no root declaration outside the target segment depends on it

5. After migration: `remove_migrated_exports()` cleans up synthetic exports, and a second DCE pass removes now-unused imports.

#### Source Map Generation (SPEC-25) -- Source: parse.rs:701-753

SWC's `emit_source_code()` function:
1. Creates a `JsWriter` that records source mappings during serialization
2. Runs `Emitter::emit_module()` which produces JavaScript code + source map buffer
3. Builds V3 source map from the recorded mappings
4. Optionally sets `sourceRoot` from `root_dir`
5. Returns `(code: String, map: Option<String>)` where map is JSON-stringified source map

**Root module**: Gets source map computed from the mutated AST (spans reference original source positions tracked by SWC's SourceMap).

**Segment modules**: Each segment module is independently constructed via `new_module()`, then run through hygiene + fixer + codegen with source map generation. The segment AST is built from scratch (not cloned from original), so source maps map back to original positions only for the function body (via preserved spans from the original parse).

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Snapshot-based examples | Manual input/output pairs | Jack's 162 .snap files | Already verified against SWC output; traceability to test corpus |
| Pipeline ordering | Guess from CONV descriptions | SWC parse.rs transform_code() function | The 20-step sequence has subtle ordering dependencies |
| Capture categories | Derive from first principles | PITFALLS.md taxonomy + Jack's Phase 24 bug fixes | 8 categories discovered through 293 runtime failures |
| Hash algorithm | Describe abstractly | Exact SWC DefaultHasher + base64 algorithm | Hash must be deterministic and match SWC output for behavioral fidelity |
| Self-import pattern | Describe as "module semantics" | SWC's ensure_export + code_move resolve_export_for_id flow | The _auto_ prefix convention and self-import generation is SWC-specific behavior |

## Common Pitfalls

### Pitfall 1: Under-specifying the 8 Capture Categories
**What goes wrong:** Spec says "captures are variables from outer scope" and the implementer treats it as one algorithm. Results in 293+ runtime failures per Jack's experience.
**Why it happens:** SWC handles categories implicitly through SyntaxContext. The distinction is invisible when reading SWC code.
**How to avoid:** Write the capture taxonomy FIRST. Use the Mermaid decision tree (D-09). Verify all 8 categories have explicit rules and examples.
**Warning signs:** Fewer than 8 categories in the spec. No mention of self-import reclassification.

### Pitfall 2: Missing Self-Import Reclassification
**What goes wrong:** Module-level declarations treated as captures (added to `_captures[]`) instead of resolved via self-imports. Causes 46+ deviations.
**Why it happens:** There is no function called "reclassify_module_level_decl_captures" in SWC. The behavior emerges from how code_move resolves identifiers.
**How to avoid:** Dedicate a subsection to self-import reclassification. Show the ensure_export + resolve_export_for_id flow explicitly.
**Warning signs:** Spec has no mention of `_auto_` prefixed exports or self-imports.

### Pitfall 3: Wrong Hash Algorithm Description
**What goes wrong:** Spec describes "content hash" or "SHA-based hash" when SWC actually uses `DefaultHasher` (SipHash-based) with specific byte ordering and base64 encoding.
**Why it happens:** Assumption that "hash" means content hash. SWC hashes the file path + display name, NOT the segment content.
**How to avoid:** Document the exact algorithm: DefaultHasher, le_bytes, URL_SAFE_NO_PAD base64, dash/underscore replacement with '0'.
**Warning signs:** Spec mentions "content-based hash" for segments.

### Pitfall 4: Ignoring Pipeline Ordering Constraints
**What goes wrong:** Spec documents each CONV independently without stating which must run before which.
**Why it happens:** Each transformation in isolation is well-defined. Ordering constraints only emerge from data flow.
**How to avoid:** Include the 20-step pipeline diagram (D-02) with explicit arrows showing data dependencies between stages.
**Warning signs:** No transformation ordering section. No mention of props_destructuring running before capture analysis.

### Pitfall 5: Display Name Collisions in Nested Dollar Calls
**What goes wrong:** Multiple `$()` calls in the same function get identical display names, causing hash collisions.
**Why it happens:** Display name derived from `stack_ctxt.join("_")` without sufficient context about call site.
**How to avoid:** Document the complete display name algorithm including: stack_ctxt hierarchy, segment_names deduplication HashMap (appends `_N`), wrapper_callee_name context, JSX element path depth.
**Warning signs:** Spec only covers "enclosing function name" case for display names.

### Pitfall 6: Parse Error Bailout in Capture Analysis
**What goes wrong:** OXC parser reports semantic errors (e.g., `await` in non-async) that should not prevent capture analysis.
**Why it happens:** Defensive "bail on any error" logic. Semantic errors produce valid ASTs that can still be analyzed.
**How to avoid:** Spec must state: "Capture analysis proceeds regardless of diagnostic errors. Only bail if parsed body is empty."
**Warning signs:** Spec says "bail on parse errors" without distinguishing structural vs semantic.

## Code Examples

Verified patterns from SWC snapshots:

### Basic Dollar Extraction (example_6)
```
INPUT:
  import { $, component$ } from '@qwik.dev/core';
  export const sym1 = $((ctx) => console.log("1"));

ROOT OUTPUT:
  import { qrl } from "@qwik.dev/core";
  const q_sym1_aXUrPXX5Lak = /*#__PURE__*/ qrl(
    ()=>import("./test.tsx_sym1_aXUrPXX5Lak"), "sym1_aXUrPXX5Lak"
  );
  export const sym1 = q_sym1_aXUrPXX5Lak;

SEGMENT OUTPUT (test.tsx_sym1_aXUrPXX5Lak.tsx):
  export const sym1_aXUrPXX5Lak = (ctx)=>console.log("1");
```

Key observations:
- `$` import stripped from root; `qrl` import added
- QRL const named `q_{symbol_name}` with `/*#__PURE__*/` annotation (note: this PURE is on the qrl() call itself, not componentQrl)
- Segment exports a named const matching the symbol_name
- `captures: false` -- no outer variables referenced

### Nested Segments with Captures (example_multi_capture)
```
INPUT:
  import { $, component$ } from '@qwik.dev/core';
  export const Foo = component$(({foo}) => {
    const arg0 = 20;
    return $(() => {
      const fn = ({aaa}) => aaa;
      return (<div>{foo}{fn()}{arg0}</div>)
    });
  })

ROOT:
  componentQrl(q_Foo_component_HTDRsvUbLiE)

COMPONENT SEGMENT (Foo_component):
  Foo_component_HTDRsvUbLiE = (_rawProps) => {
    return q_Foo_component_1_DvU6FitWglY.w([_rawProps]);
  };
  // captures: false (no scoped_idents -- _rawProps is a param)

NESTED SEGMENT (Foo_component_1):
  import { _captures } from "@qwik.dev/core";
  Foo_component_1_DvU6FitWglY = () => {
    const _rawProps = _captures[0];
    const fn = ({aaa}) => aaa;
    return <div>{_rawProps.foo}{fn()}{20}</div>;
  };
  // captures: true, captureNames: ["_rawProps"]
```

Key observations:
- Props destructuring transformed `{foo}` to `_rawProps` before capture analysis
- `_rawProps` is captured (scoped_ident) because it's an outer-scope local variable
- `arg0` (const 20) is inlined as literal `20` in the segment -- NOT captured
- `.w([_rawProps])` is the "with captures" call on the QRL reference
- The `_captures` import + destructuring pattern is the segment-side capture resolution

### Import Re-emission in Segments (example_capture_imports)
```
INPUT:
  import { component$, useStyles$ } from '@qwik.dev/core';
  import css1 from './global.css';
  import css2 from './style.css';
  export const App = component$(() => {
    useStyles$(`${css1}${css2}`);
    useStyles$(css3);
  })

SEGMENT (App_component_useStyles):
  import css1 from "./global.css";
  import css2 from "./style.css";
  export const App_component_useStyles_t35nSa5UV7U = `${css1}${css2}`;
  // css1, css2 are re-emitted imports, NOT captures
```

Key observations:
- `css1` and `css2` are user-code imports -- they are re-emitted in the segment as import statements
- The segment directly imports from the same source (`./global.css`, `./style.css`)
- `captures: false` because all referenced identifiers resolve to imports, not locals

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `@builder.io/qwik` imports | `@qwik.dev/core` imports | Qwik v2 | RenameTransform handles legacy; spec should document both |
| `Hook` entry strategy name | `Segment` entry strategy name | Qwik v2 | Both are aliases; `Hook` maps to `Segment` |
| No variable migration | Full dependency-based migration | Late SWC development | Smaller root modules; segments contain their exclusive dependencies |
| Manual capture stack | IdentCollector + compute_scoped_idents | SWC architecture | Single-algorithm capture detection with decl_stack partitioning |

## Open Questions

1. **IdentCollector JSX attribute behavior**
   - What we know: IdentCollector pushes `ExprOrSkip::Skip` for `visit_jsx_attr` (line 452-455), meaning identifiers inside JSX attribute names are not collected. But JSX expression containers are still collected because `visit_expr` pushes `Expr`.
   - What's unclear: The exact boundary between "attribute name" and "attribute value expression" in the collector. Need to verify with specific examples.
   - Recommendation: Use snapshot examples with JSX event handlers to validate behavior.

2. **Exact _captures destructuring pattern**
   - What we know: `transform_function_expr()` (in code_move.rs) injects `const captured = _captures[N]` at the top of the function body.
   - What's unclear: The exact function signature and whether it wraps arrow expressions differently from function expressions.
   - Recommendation: Read `transform_function_expr` in code_move.rs for the exact pattern. Include in SPEC-03/SPEC-30.

3. **Source map fidelity for segment modules**
   - What we know: Segment modules are constructed as new AST, run through codegen with source map enabled. The source map JSON is stringified and included in TransformModule.map.
   - What's unclear: How accurately segment source maps point back to original source positions, given that the segment AST is built from scratch (not cloned).
   - Recommendation: Document as a behavioral contract ("segment source maps SHOULD reference original file positions") without guaranteeing byte-level accuracy.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual review (specification document, not code) |
| Config file | N/A |
| Quick run command | Visual inspection of spec sections against SWC source |
| Full suite command | Cross-reference all spec examples against SWC snapshot outputs |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SPEC-01 | Dollar detection rules complete | manual | Verify spec covers both imported + local markers | N/A |
| SPEC-02 | QRL wrapping rules complete | manual | Verify qrl/inlinedQrl/noopQrl paths documented | N/A |
| SPEC-03 | Capture analysis taxonomy complete | manual | Count 8 categories + 16 edge cases in spec | N/A |
| SPEC-05 | Segment extraction behavior documented | manual | Verify filename gen + hash + nesting + migration examples | N/A |
| SPEC-12 | Import rewriting rules documented | manual | Verify consumed stripping + synthetic addition + per-segment resolution | N/A |
| SPEC-21 | GlobalCollect behavior documented | manual | Verify imports/exports/root collection rules | N/A |
| SPEC-22 | Variable migration documented | manual | Verify 5-step pipeline + migratable conditions | N/A |
| SPEC-23 | Hash algorithm documented | manual | Verify DefaultHasher + base64 + exact encoding | N/A |
| SPEC-24 | Path resolution documented | manual | Verify canonical filename + relative path + extension | N/A |
| SPEC-25 | Source map contracts documented | manual | Verify root + segment source map generation | N/A |
| SPEC-30 | Capture taxonomy with all 8 categories | manual | Count categories + verify edge case coverage | N/A |

### Sampling Rate
- **Per task commit:** Review spec section against SWC source + snapshots
- **Per wave merge:** Full spec review for internal consistency
- **Phase gate:** All 11 requirements addressed with examples; cross-reference against snapshot corpus

### Wave 0 Gaps
None -- this is a specification phase, not a code phase. No test infrastructure needed.

## Sources

### Primary (HIGH confidence)
- SWC optimizer source at `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/` -- direct reading of transform.rs, code_move.rs, collector.rs, dependency_analysis.rs, parse.rs, words.rs
- Jack's SWC snapshots at `/Users/scottweaver/Projects/qwik-oxc-optimizer/swc-snapshots/` -- verified input/output pairs (example_6.snap, example_capture_imports.snap, example_multi_capture.snap, example_capturing_fn_class.snap)

### Secondary (HIGH confidence)
- PITFALLS.md at `.planning/research/PITFALLS.md` -- 15 catalogued pitfalls derived from Jack's Phase 24 bug-fix campaign
- FEATURES.md at `.planning/research/FEATURES.md` -- 14 CONVs with dependency DAG and pipeline ordering
- ARCHITECTURE.md at `.planning/research/ARCHITECTURE.md` -- Pipeline architecture patterns, SWC vs OXC comparison

## Metadata

**Confidence breakdown:**
- Pipeline ordering: HIGH -- directly read from SWC parse.rs transform_code() function
- Dollar detection: HIGH -- straightforward pattern matching in transform.rs constructor
- Capture analysis: HIGH -- full algorithm traced through transform.rs + collector.rs + compute_scoped_idents
- QRL wrapping: HIGH -- all three creation paths (qrl/inlinedQrl/noopQrl) traced with exact arguments
- Hash generation: HIGH -- exact algorithm including DefaultHasher seed order, base64 encoding, character replacement
- Segment extraction: HIGH -- create_segment + new_module + import resolution traced end-to-end
- Variable migration: HIGH -- full 5-step pipeline in dependency_analysis.rs with safety filter
- Source maps: MEDIUM -- general contract understood but segment source map fidelity details are implicit in SWC
- Import rewriting: HIGH -- GlobalCollect synthetic imports + code_move per-segment resolution traced

**Research date:** 2026-04-01
**Valid until:** 2026-05-01 (stable -- SWC source is the ground truth and does not change frequently)

## Project Constraints (from CLAUDE.md)

- Single spec document: The specification is one comprehensive markdown file
- Behavioral fidelity: OXC implementation must produce functionally equivalent output to SWC for all 162 test cases
- OXC idioms: Implementation must use OXC's Traverse trait, arena allocators, SemanticBuilder, and Codegen -- not SWC patterns translated to OXC APIs
- Foundation: Jack's 162 spec files are the behavioral test corpus
- GSD workflow: Use GSD entry points for all file changes
