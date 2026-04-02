# Phase 5: Core OXC Implementation - Research

**Researched:** 2026-04-01
**Domain:** Rust OXC-based JavaScript code transformer -- Qwik optimizer implementing 14 CONV transformations
**Confidence:** HIGH

## Summary

Phase 5 implements the core `qwik-optimizer-oxc` Rust crate that performs all 14 CONV transformations specified in the 8,091-line behavioral specification. The crate is a spec-driven fresh build at `crates/qwik-optimizer-oxc/` in the existing `qwik-optimizer-next` workspace. It must pass all 201 behavioral tests from the SWC snapshot corpus (grown from the original 162) plus spec-derived tests from Appendix B's 24 curated examples.

The technology stack is well-established from prior research: OXC 0.123 umbrella crate with `oxc_traverse` for AST traversal, arena allocation via `Allocator`, `SemanticBuilder` for scope/symbol analysis, and `Codegen` for JavaScript output. The architecture follows a three-stage pipeline (Parse, Transform, Emit) with a single `traverse_mut` pass, pre-traverse mutations for const replacement and export stripping, and string-based segment module construction. Jack Shelton's working implementation (16,577 lines of Rust across 21 source files) serves as OXC API reference but code is written fresh from the spec.

The highest risk area is capture analysis -- 82% of Jack's runtime bugs originated there, with 8 distinct capture categories requiring exhaustive handling. The snapshot corpus provides the gating mechanism: all 201 `.snap` files define input/output pairs that the implementation must match (semantic equivalence, not byte-for-byte). The `fixtures.json` file (201 entries) provides the complete test configuration for each snapshot including mode, entry strategy, and options.

**Primary recommendation:** Build in dependency order (Layer 1 foundation through Layer 6 integration) with the 201-test insta snapshot suite running from the earliest possible moment. Prioritize capture analysis correctness over feature breadth.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-33:** The new crate lives at `crates/qwik-optimizer-oxc/` in the existing `qwik-optimizer-next` workspace (alongside `specification/`). Add to the root `Cargo.toml` workspace members.
- **D-34:** Spec-driven fresh build. Write from the spec document (`specification/qwik-optimizer-spec.md`). Consult Jack's implementation and Scott's conversion when stuck on OXC API usage, but don't copy-paste. The goal is idiomatic design driven by the spec, not inherited patterns.
- **D-35:** Both test sources: (1) Copy Jack's 201 SWC snapshots (`swc-snapshots/*.snap`) as the full regression suite using insta for snapshot testing. (2) Derive focused behavioral tests from the 24+ spec examples in Appendix B.
- **D-36:** Target the latest stable OXC release at build time. Pin with exact version in Cargo.toml. Upgrade deliberately with the 201-test suite as the gating mechanism.
- **D-05:** SWC is source of truth for behavioral correctness.
- **D-08:** Idiomatic OXC patterns (Traverse trait, arena allocators, SemanticBuilder, Codegen).
- **D-09:** OXC Scoping for capture analysis where it improves correctness over manual approaches.

### Claude's Discretion
- Internal module organization within the crate (how to split transform.rs, collector.rs, etc.)
- Whether to use a single `traverse_mut` pass or two-phase analyze-then-emit (both are valid OXC approaches per research)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| IMPL-01 | OXC implementation passes all 201 behavioral tests from snapshot corpus | 201 `.snap` files at `swc-snapshots/` with matching `fixtures.json`; insta snapshot testing framework; semantic equivalence comparison |
| IMPL-02 | OXC implementation supports all 14 CONV transformation types | Spec document covers all 14 CONVs (CONV-01 through CONV-14); architecture research maps each CONV to pipeline stages; snapshot corpus has coverage for all 14 |
| IMPL-05 | OXC implementation produces functionally equivalent output to SWC version | SWC snapshots ARE the SWC output; passing snapshots = semantic equivalence; cosmetic differences acceptable per project constraints |
| IMPL-08 | Uses idiomatic OXC patterns (Traverse, arena allocators, SemanticBuilder, Codegen) | `traverse_mut` API verified at v0.123.0; three-stage architecture (Parse/Transform/Emit) documented; Appendix A migration guide maps all SWC patterns to OXC equivalents |
| IMPL-09 | Uses OXC Scoping for capture analysis | `SemanticBuilder` produces `Scoping` with `ScopeTree`, `SymbolTable`, `ReferenceTable`; accessible via `TraverseCtx::scoping()` during traversal; 8-category capture taxonomy documented |
</phase_requirements>

## Project Constraints (from CLAUDE.md)

- Single spec document (`specification/qwik-optimizer-spec.md`) is the primary reference
- Behavioral fidelity: functionally equivalent output for all test cases (cosmetic differences acceptable)
- OXC idioms mandatory: Traverse trait, arena allocators, SemanticBuilder, Codegen -- not SWC patterns translated
- Jack's 201 spec files are the behavioral test corpus
- GSD workflow enforcement: changes through GSD commands

## Standard Stack

### Core

| Library | Version | Purpose | Why Standard |
|---------|---------|---------|--------------|
| `oxc` | `0.123.0` | Umbrella: parser, AST, codegen, semantic | De facto Rust JS toolchain. Single dep gates parser + codegen + semantic. Verified on crates.io 2026-04-01. |
| `oxc_traverse` | `0.123.0` | AST traversal with `Traverse` trait | Required separate crate. Provides enter/exit visitor, ancestor access, `TraverseCtx` for scope queries. Must match `oxc` version. |
| `serde` | `1` (features: `["derive"]`) | Serialize/deserialize all public types | Industry standard. Required for JSON interface with JS callers. |
| `serde_json` | `1` | JSON encoding for options/output | Pairs with serde. Options arrive as JSON from NAPI/WASM. |
| `anyhow` | `1.0.102` | Error handling with context chains | Application-level error handling with `.context()` chains. |
| `base64` | `0.22.1` | Segment hash computation | Used for hash encoding in canonical filenames. |
| `siphasher` | `1.0.2` | Deterministic SipHash 1-3 with seed (0,0) | Required for byte-identical hashes matching SWC. `std::collections::DefaultHasher` uses randomized seed since Rust 1.36. |
| `indexmap` | `2.13.0` | Ordered maps with insertion order | Used by GlobalCollect for stable import/export ordering. |
| `pathdiff` | `0.2.3` | Relative path computation | Computes `./foo` relative imports between root and segments. |
| `path-slash` | `0.2.1` | Cross-platform path normalization | Windows backslash to forward slash for import paths. |
| `rayon` | `1` (optional, `parallel` feature) | Parallel module transformation | Optional. WASM targets cannot use threads. Feature-gated. |

### Testing

| Library | Version | Purpose | When to Use |
|---------|---------|---------|-------------|
| `insta` | `1.47.2` (features: `["json"]`) | Snapshot testing | All 201 regression tests + JSON structured comparison for `TransformOutput`. |

### OXC Feature Flags

```toml
oxc = { version = "0.123", features = [
    "codegen",     # oxc_codegen: AST -> JavaScript source
    "semantic",    # oxc_semantic: SemanticBuilder for scope/symbol analysis
    "serialize",   # ESTree JSON serialization (testing/debugging)
    "ast_visit",   # VisitMut trait for const_replace pre-pass
] }
```

**Excluded flags:** `transformer` (Babel compat, unneeded), `minifier`, `mangler`, `full`, `cfg`, `isolated_declarations`.

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| `siphasher` | `std::collections::DefaultHasher` | DefaultHasher is randomized since Rust 1.36 -- produces different hashes on each run. Must use siphasher for deterministic output. |
| `indexmap` | `HashMap` | HashMap has random iteration order, producing non-deterministic import ordering in output. IndexMap preserves insertion order. |
| `anyhow` | `thiserror` | thiserror for typed errors in library code. The optimizer is an application (callers get strings via NAPI/WASM). anyhow's context chains better for debugging. |
| Single `traverse_mut` | Two-phase analyze-then-emit | Two passes double traversal time. Single pass with enter/exit ordering proven sufficient by Jack's implementation. |

**Installation:**
```bash
# From workspace root
mkdir -p crates/qwik-optimizer-oxc
cd crates/qwik-optimizer-oxc
cargo init --lib

cargo add oxc@0.123 --features codegen,semantic,serialize,ast_visit
cargo add oxc_traverse@0.123
cargo add serde@1 --features derive
cargo add serde_json@1
cargo add anyhow@1
cargo add base64@0.22
cargo add siphasher@1
cargo add indexmap@2
cargo add pathdiff@0.2
cargo add path-slash@0.2
cargo add rayon@1 --optional

# Dev dependencies
cargo add --dev insta@1 --features json
```

## Architecture Patterns

### Recommended Project Structure

```
crates/qwik-optimizer-oxc/
  Cargo.toml
  src/
    lib.rs                  # Public API: transform_modules() + per-file orchestration
    types.rs                # All public/internal types (pure data, no logic)
    words.rs                # String constants, dollar_to_qrl_name(), classify_ctx_kind()
    hash.rs                 # Deterministic SipHash + base64 encoding for segment names
    errors.rs               # Diagnostic creation helpers
    is_const.rs             # Expression constness analysis for signal optimization
    parse.rs                # Parser + SemanticBuilder wrapper
    collector.rs            # GlobalCollect: import/export/root-decl indexing
    entry_strategy.rs       # EntryStrategy enum, grouping, should_inline()
    const_replace.rs        # Pre-traverse: isServer/isDev/isBrowser -> bool literals
    filter_exports.rs       # Pre-traverse: strip_exports removal
    rename_imports.rs       # Legacy @builder.io -> @qwik.dev rename
    props_destructuring.rs  # Props reconstruction for signal forwarding
    transform.rs            # QwikTransform: impl Traverse (the core)
    jsx_transform.rs        # JSX rewriting (_jsxSorted/_jsxSplit, signals)
    inlined_fn.rs           # _fnSignal generation for inline JSX expressions
    code_move.rs            # Segment module construction from strings
    emit.rs                 # Codegen wrapper with source map support
    dependency_analysis.rs  # Variable migration dependency analysis
    clean_side_effects.rs   # Treeshaker side effect cleanup
    add_side_effect.rs      # Side effect import preservation
  tests/
    snapshots/              # 201 .snap files (copied from swc-snapshots/)
    snapshot_tests.rs       # Insta test harness driving fixtures.json
```

### Pattern 1: Three-Stage Pipeline (Parse, Transform, Emit)

**What:** Each input file is processed through three sequential stages: (1) Parse + SemanticBuilder + pre-traverse mutations, (2) single `traverse_mut` pass for core transformation, (3) codegen for root module + string-based segment module construction.

**When to use:** Always. This is the fundamental architecture.

**Example:**
```rust
// Source: ARCHITECTURE.md research + OXC docs.rs v0.123
pub fn transform_code(options: TransformCodeOptions) -> TransformOutput {
    // Stage 1: Parse
    let allocator = Allocator::default();
    let ret = Parser::new(&allocator, &source, source_type).parse();
    let scoping = SemanticBuilder::new().build(&ret.program).scoping;
    let collect = global_collect(&ret.program);

    // Pre-traverse mutations
    const_replace::replace(&mut program, &collect, is_server);
    filter_exports::filter(&mut program, &strip_exports);

    // Stage 2: Transform (single pass)
    let mut transformer = QwikTransform::new(collect, options);
    let scoping = traverse_mut(&mut transformer, &allocator, &mut program, scoping, ());

    // Stage 3: Emit
    let root_code = Codegen::new().build(&program);
    let segments = transformer.segments.iter().map(|seg| {
        code_move::build_segment_module(seg, &collect)
    });
    // ...
}
```

### Pattern 2: Span-Based Body Extraction (Not AST Cloning)

**What:** Extract segment function bodies by slicing the original source code at recorded spans, not by cloning AST subtrees. Assemble segment modules as strings, re-parse with OXC for source map generation.

**When to use:** Always for segment module construction. Avoids separate-allocator-per-output complexity.

**Example:**
```rust
// During traverse: record the span
let body_code = &self.source_code[span.start as usize..span.end as usize];
self.segment_body_codes.push((seg_id, body_code.to_string()));

// Post-traverse: build segment module as string
let segment_code = format!(
    "{imports}\nexport const {name} = {body};\n",
    imports = resolved_imports,
    name = segment.name,
    body = body_code,
);
// Re-parse + codegen for normalized output and source map
let (code, map) = emit_segment(&segment_code, &filename, source_maps);
```

### Pattern 3: ImportTracker Accumulator

**What:** Accumulate import needs as boolean flags during traversal. Batch-apply all import mutations in `exit_program`.

**When to use:** Always. Import needs are discovered incrementally (this $-call needs `qrl`, this JSX needs `_jsxSorted`).

### Pattern 4: Pre-Traverse Mutations

**What:** Run const replacement and export stripping BEFORE `traverse_mut`. The main traversal sees a stable, simplified AST.

**When to use:** For CONV-10 (const replacement) and CONV-11 (strip_exports). These use `VisitMut` trait, not `Traverse`.

### Anti-Patterns to Avoid

- **Cloning arena-allocated ASTs for segments:** Every node is arena-allocated. Building segments from AST clones wastes memory. Use string-based construction.
- **Manual scope tracking duplicating SemanticBuilder:** `oxc_semantic` builds a complete scope tree. Don't maintain a hand-rolled `decl_stack` when `Scoping` provides the same information.
- **Casting between JSXExpression and Expression:** OXC's `inherit_variants!` macro makes these look interchangeable but they have different enum layouts. Use parse-roundtrip instead.
- **Using SWC `Fold` patterns:** No ownership transfer. Use `enter_*`/`exit_*` with mutable references. Analyze in enter, mutate in exit.
- **Wildcard `_ => {}` on BindingPattern:** Must handle all 4 variants (BindingIdentifier, ObjectPattern, ArrayPattern, AssignmentPattern) explicitly. Jack found the same bug in 3 separate match sites.
- **Two separate traverse passes:** Single `traverse_mut` with enter/exit ordering is sufficient and proven.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| AST parsing | Custom parser | `oxc::parser::Parser` | Full JS/TS/JSX/TSX support, error recovery |
| Scope/symbol analysis | Manual scope stack | `oxc::semantic::SemanticBuilder` -> `Scoping` | Handles all scope types (catch, for-in/of, class fields, etc.) |
| JavaScript output | String concatenation | `oxc::codegen::Codegen` | Source map support, proper formatting, spec-compliant output |
| Deterministic hashing | `std::hash::DefaultHasher` | `siphasher::sip::SipHasher13` with seed (0,0) | DefaultHasher is randomized. SipHash 1-3 with fixed seed matches SWC behavior. |
| Ordered maps | `HashMap` with sort step | `indexmap::IndexMap` | Insertion-order iteration without explicit sorting |
| Relative paths | Manual string slicing | `pathdiff::diff_paths()` | Edge cases: Windows, trailing slashes, relative roots |
| Snapshot testing | Manual assert_eq on multi-line strings | `insta` with `.snap` files | 201 test cases, `cargo insta review` workflow |

**Key insight:** The OXC ecosystem provides parser, semantic analysis, traversal, and codegen as a complete pipeline. The optimizer's job is the transformation logic between parse and codegen -- do not rebuild any of the infrastructure OXC already provides.

## Snapshot Corpus Analysis (201 Tests)

### Corpus Growth

The snapshot corpus has grown from 162 to **201** `.snap` files. All 201 have matching entries in `fixtures.json` with complete test configurations (mode, entry strategy, strip options, etc.).

### Snapshot File Format

Each `.snap` file follows this structure:
```
---
source: packages/optimizer/core/src/test.rs
assertion_line: NNNN
expression: output
---
==INPUT==

[input source code]

============================= [segment_filename] (ENTRY POINT)==

[segment output code]

Some("[source_map_json]")
/*
{
  [SegmentAnalysis JSON metadata]
}
*/
============================= [root_module_filename] ==

[root module output code]

Some("[source_map_json]")
== DIAGNOSTICS ==

[JSON array of diagnostics]
```

Key sections per snapshot:
1. **INPUT** -- the source TypeScript/JSX/TSX code
2. **One or more segment modules** -- each with code, source map, and SegmentAnalysis JSON
3. **Root module** -- the transformed main file with code and source map
4. **DIAGNOSTICS** -- warnings/errors as JSON array

### CONV Coverage by Snapshot Count

| CONV | Indicator | Snapshot Count | Coverage Quality |
|------|-----------|---------------|-----------------|
| CONV-01 Dollar Detection | `qrl(` present | 152 | Excellent |
| CONV-02 QRL Wrapping | `qrl(` / `inlinedQrl` | 152+12 | Excellent (segment + inline strategies) |
| CONV-03 Capture Analysis | `.w([` captures | 40 | Good (includes loop, nested, multi-capture) |
| CONV-04 Props Destructuring | `_rawProps` | 17 | Good (colon, rest, nested variants) |
| CONV-05 Segment Extraction | segment modules | ~190 | Excellent (nearly all tests produce segments) |
| CONV-06 JSX Transform | `_jsxSorted`/`_jsxSplit` | 137 | Excellent |
| CONV-07 Signal Optimization | `_fnSignal`/`_wrapProp` | 59 | Excellent |
| CONV-08 PURE Annotations | `#__PURE__` | 196 | Excellent |
| CONV-09 Dead Branch Elimination | dead code removal | ~5 | Adequate (limited scenarios) |
| CONV-10 Const Replacement | `isServer`/`isBrowser`/`isDev` | 8 | Adequate |
| CONV-11 Code Stripping | `strip_*` filenames | 4 | Adequate |
| CONV-12 Import Rewriting | import mutations | ~190 | Excellent (present in nearly all tests) |
| CONV-13 sync$ Serialization | `_qrlSync` | 2 | Minimal but covered |
| CONV-14 Noop QRL Handling | `_noopQrl` | 38 | Good |

Additional coverage areas: loop captures (13 files), destructuring (13 files), scopes (8 files), self-referential QRLs (4 files), bind directives (12 files), dev mode (6 files), variable migration (2 files), HMR (1 file), lib mode (2 files).

### New Snapshots (39 additions beyond original 162)

The 39 new snapshots include tests for:
- Component-level self-referential QRLs
- Destructuring edge cases (colon props, inline component variants)
- Hoisted function signals in loops
- Fun with scopes (scope isolation tests)
- Issue-specific regression tests (issue_117, issue_150, issue_476, issue_5008, issue_7216, issue_964)
- Root-level self-referential QRL (inline + segment variants)
- Impure template function handling
- Inlined QRL uses identifier reference when hoisted
- Moves captures when possible
- Lib mode fn_signal
- Transform QRL in regular prop

## Common Pitfalls

### Pitfall 1: Capture Analysis Category Confusion
**What goes wrong:** Treating capture analysis as a single algorithm when there are 8 distinct categories with different behavior (module-level decls become self-imports, user imports become re-emitted imports, loop vars are actual captures, etc.).
**Why it happens:** SWC handles these implicitly through SyntaxContext. The categories are not explicitly separated in SWC source.
**How to avoid:** Follow the spec's capture taxonomy (8 categories). Module-level declarations are NOT captures -- they become self-imports. Test with the 40 capture-specific snapshots first.
**Warning signs:** `captures: false` in SegmentAnalysis when captures were expected, or vice versa. Runtime `ReferenceError` in segment code.

### Pitfall 2: Module-Level Declaration Self-Import Reclassification
**What goes wrong:** Module-level declarations (const/function/class at file root) referenced by nested segments get treated as captures instead of being converted to self-imports (`import { X } from "./module_stem"`).
**Why it happens:** This is the single most impactful behavioral distinction in the optimizer. Jack's Plan 05 fixed 46 runtime-breaking deviations from this one issue.
**How to avoid:** Post-process capture analysis results: for each capture candidate, check if it exists in `GlobalCollect.root` -- if yes, reclassify as needed_import with source `./module_stem`.
**Warning signs:** Segments with missing imports for module-level variables. `captures: true` when SWC expects `captures: false`.

### Pitfall 3: BindingPattern Exhaustiveness
**What goes wrong:** `AssignmentPattern` variant (destructuring defaults like `{I5 = v2}`) missed in pattern matching. Bug replicated across every site that matches on `BindingPattern`.
**Why it happens:** Wildcard `_ => {}` suppresses Rust's exhaustiveness warning.
**How to avoid:** Never use wildcard on `BindingPattern`. Match all 4 variants: `BindingIdentifier`, `ObjectPattern`, `ArrayPattern`, `AssignmentPattern` (recurse into inner pattern).
**Warning signs:** Variables from destructuring defaults missing from collector or capture analysis.

### Pitfall 4: Parse Error Bailout in Capture Analysis
**What goes wrong:** Bailing on ANY parse error when analyzing lambda captures. OXC reports semantic errors (like `await` in non-async) that still produce valid ASTs.
**Why it happens:** Defensive "if parser errored, don't trust the AST" assumption.
**How to avoid:** Only bail if `ret.panicked` (structural failure). Proceed with capture extraction despite semantic diagnostics.
**Warning signs:** Empty captures for valid-but-semantic-error code patterns.

### Pitfall 5: Display Name Collisions in Nested Dollar Calls
**What goes wrong:** Multiple `$()` calls in the same function produce duplicate display names, causing hash collisions.
**Why it happens:** Display name derives from enclosing function without accounting for wrapper context.
**How to avoid:** Include `wrapper_callee_name` context (e.g., `renderHeader_component` vs `renderHeader_div_q_e_click`). Follow spec's display name construction algorithm exactly.
**Warning signs:** Duplicate `const` declarations in output. Hash collisions between segments.

### Pitfall 6: PURE Annotations on Non-Component QRLs
**What goes wrong:** Annotating `useTaskQrl`, `serverQrl`, etc. with `/*#__PURE__*/`. These have side effects and get tree-shaken away in production.
**Why it happens:** Temptation to annotate all `*Qrl()` calls.
**How to avoid:** Only `componentQrl` gets PURE annotation. Follow the explicit whitelist in spec CONV-08.
**Warning signs:** Hooks silently removed in production builds.

### Pitfall 7: JSXExpression/Expression Type Casting
**What goes wrong:** Attempting to cast between `JSXExpression` and `Expression` via pointer manipulation. Unsound due to different enum layouts despite shared variant types.
**Why it happens:** OXC's `inherit_variants!` makes them look interchangeable.
**How to avoid:** Use parse-roundtrip: extract lambda by span, wrap as `var x = <lambda>`, parse with OXC, codegen the init expression.
**Warning signs:** Any `unsafe` block involving Expression/JSXExpression conversion.

## Code Examples

### Example 1: Public API Entry Point

```rust
// Source: spec lines 3671-3677 (TransformOutput::append) + ARCHITECTURE.md
pub fn transform_modules(options: TransformModulesOptions) -> TransformOutput {
    let mut output = TransformOutput::default();

    let process_file = |input: &TransformModuleInput| -> TransformOutput {
        transform_code(TransformCodeOptions {
            path: &input.path,
            code: &input.code,
            // ... options from TransformModulesOptions
        })
    };

    #[cfg(feature = "parallel")]
    {
        use rayon::prelude::*;
        let results: Vec<_> = options.input.par_iter().map(process_file).collect();
        for mut r in results { output.append(&mut r); }
    }

    #[cfg(not(feature = "parallel"))]
    {
        for input in &options.input {
            let mut r = process_file(input);
            output.append(&mut r);
        }
    }

    output
}
```

### Example 2: traverse_mut Setup

```rust
// Source: docs.rs/oxc_traverse/0.123.0 + ARCHITECTURE.md
use oxc::allocator::Allocator;
use oxc::parser::Parser;
use oxc::semantic::SemanticBuilder;
use oxc::span::SourceType;
use oxc_traverse::{Traverse, TraverseCtx, traverse_mut};

struct QwikTransform<'a> {
    source_code: &'a str,
    collect: GlobalCollect,
    segments: Vec<SegmentRecord>,
    // ...
}

impl<'a> Traverse<'a, ()> for QwikTransform<'a> {
    fn enter_call_expression(
        &mut self,
        node: &mut CallExpression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Dollar detection, capture tracking push
    }

    fn exit_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // QRL wrapping, segment extraction
    }

    fn exit_program(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Import rewriting, hoisted const insertion
    }
}
```

### Example 3: GlobalCollect Pattern

```rust
// Source: spec lines 95-158 (Stage 2: GlobalCollect)
pub(crate) struct GlobalCollect {
    pub imports: IndexMap<String, Import>,
    pub exports: IndexMap<String, ExportInfo>,
    pub root: IndexMap<String, Span>,
    rev_imports: HashMap<(String, String), String>,
}

pub(crate) fn global_collect(program: &Program) -> GlobalCollect {
    let mut collect = GlobalCollect::new();
    for stmt in &program.body {
        match stmt {
            Statement::ImportDeclaration(decl) => { /* catalog imports */ }
            Statement::ExportNamedDeclaration(decl) => { /* catalog exports */ }
            Statement::ExportDefaultDeclaration(decl) => { /* catalog default export */ }
            _ => { /* catalog root declarations */ }
        }
    }
    collect
}
```

### Example 4: Segment Module Construction (String-Based)

```rust
// Source: ARCHITECTURE.md Pattern 2 + Jack's code_move.rs
pub(crate) fn emit_segment(
    raw_code: &str,
    filename: &str,
    source_maps: bool,
) -> (String, Option<String>) {
    let allocator = Allocator::default();
    let source: &str = allocator.alloc_str(raw_code);
    let ret = Parser::new(&allocator, source, SourceType::mjs()).parse();
    if ret.panicked {
        return (raw_code.to_string(), None);
    }
    let result = Codegen::new()
        .with_source_text(source)
        .build(&ret.program);
    (result.code, /* optional source map */)
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| SWC `Fold` (ownership) | OXC `Traverse` (mutable ref) | OXC migration | Analyze in enter_*, mutate in exit_* |
| SWC `SyntaxContext` | OXC `SemanticBuilder` -> `Scoping` | OXC migration | Separate pre-pass for scope analysis |
| AST cloning for segments | String-based segment construction | Jack's discovery | 5-10x less code, avoids allocator conflicts |
| `lazy_static!` | `std::sync::LazyLock` | Rust 1.80+ | Zero-dep replacement |
| OXC 0.113 (Jack's pin) | OXC 0.123 (latest stable) | 2026-03-30 | ~10 minor versions newer, API stable for our usage |
| 162 snapshot corpus | 201 snapshot corpus | Recent growth | 39 additional edge cases covered |

**Deprecated/outdated:**
- SWC `Fold`/`VisitMut` patterns -- do not use in OXC code
- `derivative` proc macro -- use standard `#[derive()]`
- `lazy_static` -- use `std::sync::LazyLock`
- `relative-path` crate -- `pathdiff` + `path-slash` cover all needs

## Build Order (Layer Dependencies)

The implementation should follow this dependency order, which enables incremental testing:

```
Layer 1: Foundation (no internal dependencies) -- ~30% of code
    types.rs         -- All shared data structures, options, output types
    words.rs         -- String constants, dollar_to_qrl_name, classify_ctx_kind
    hash.rs          -- SipHash + base64 for segment names
    errors.rs        -- Diagnostic helpers
    is_const.rs      -- Expression constness for signal optimization

Layer 2: Parse + Collect -- ~15% of code
    parse.rs         -- Parser + SemanticBuilder wrapper
    collector.rs     -- GlobalCollect: import/export/root declaration indexing
    entry_strategy.rs -- EntryStrategy enum, grouping logic

Layer 3: Pre-Traverse Mutations -- ~10% of code
    const_replace.rs    -- isServer/isDev/isBrowser replacement
    filter_exports.rs   -- strip_exports removal
    rename_imports.rs   -- @builder.io -> @qwik.dev legacy rename

Layer 4: Core Transform (depends on all above) -- ~30% of code
    props_destructuring.rs -- Props reconstruction
    jsx_transform.rs      -- JSX rewriting (_jsxSorted/_jsxSplit)
    inlined_fn.rs         -- _fnSignal generation
    transform.rs          -- QwikTransform: impl Traverse (the heart)

Layer 5: Emit -- ~10% of code
    code_move.rs    -- Segment module string construction (13-step pipeline)
    emit.rs         -- Codegen wrapper with source map support

Layer 6: Integration -- ~5% of code
    lib.rs          -- Public API, per-file orchestration loop
    dependency_analysis.rs -- Variable migration
    clean_side_effects.rs  -- Treeshaker cleanup
    add_side_effect.rs     -- Side effect import preservation
```

**Critical path:** transform.rs (6,493 lines in Jack's impl) is by far the largest module. It integrates all Layer 4 modules and contains the core `Traverse` implementation. Building it incrementally (dollar detection first, then capture analysis, then QRL wrapping, etc.) is essential.

## Reference Code Analysis

### Jack's Implementation (16,577 total lines)

| Module | Lines | Complexity | Notes |
|--------|-------|-----------|-------|
| transform.rs | 6,493 | Very High | Core traverse, all CONV logic interleaved |
| code_move.rs | 1,568 | High | 13-step segment module pipeline |
| lib.rs | 1,660 | High | Orchestration + tests |
| types.rs | 1,036 | Medium | Pure data definitions |
| dependency_analysis.rs | 842 | Medium | Variable migration |
| hash.rs | 694 | Medium | SipHash + base64 + display name |
| collector.rs | 616 | Medium | GlobalCollect |
| props_destructuring.rs | 577 | Medium | Props reconstruction |
| parse.rs | 422 | Low | Parser wrapper |
| inlined_fn.rs | 399 | Medium | _fnSignal generation |
| entry_strategy.rs | 350 | Low | Strategy enum + grouping |
| is_const.rs | 307 | Low | Constness checks |
| filter_exports.rs | 304 | Low | Export stripping |
| const_replace.rs | 287 | Low | Const replacement |
| clean_side_effects.rs | 282 | Low | Treeshaker cleanup |
| add_side_effect.rs | 258 | Low | Side effect preservation |
| rename_imports.rs | 189 | Low | Legacy import rename |
| emit.rs | 152 | Low | Codegen wrapper |
| words.rs | 121 | Low | String constants |
| errors.rs | 20 | Trivial | Error helpers |

### Key Differences from Our Build

Our implementation will differ from Jack's in these ways:
1. **Spec-driven design:** Module organization follows the spec's CONV grouping, not Jack's organic growth
2. **OXC 0.123 vs 0.113:** 10 minor versions newer; verify API compatibility for `traverse_mut` signature, `Scoping`, `SemanticBuilder`
3. **OXC Scoping for capture analysis:** Jack uses a simplified `capture_stack` -- we should leverage `TraverseCtx::scoping()` where it improves correctness (D-09)
4. **201 snapshots vs 162:** Our test corpus is larger from the start

## Validation Architecture

### Test Framework

| Property | Value |
|----------|-------|
| Framework | `insta` 1.47.2 with `json` feature |
| Config file | None initially -- Wave 0 creates test harness |
| Quick run command | `cargo test -p qwik-optimizer-oxc -- --test-threads=1` |
| Full suite command | `cargo test -p qwik-optimizer-oxc` |

### Phase Requirements to Test Map

| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| IMPL-01 | Pass all 201 behavioral tests | snapshot | `cargo test -p qwik-optimizer-oxc snapshot_tests` | Wave 0 |
| IMPL-02 | All 14 CONV types | snapshot (subset) | `cargo test -p qwik-optimizer-oxc -- conv` | Wave 0 |
| IMPL-05 | Semantic equivalence to SWC | snapshot | Same as IMPL-01 (passing snapshots = equivalence) | Wave 0 |
| IMPL-08 | Idiomatic OXC patterns | code review + compile | `cargo build -p qwik-optimizer-oxc` (no SWC deps) | Wave 0 |
| IMPL-09 | OXC Scoping for captures | unit + snapshot | `cargo test -p qwik-optimizer-oxc -- capture` | Wave 0 |

### Sampling Rate

- **Per task commit:** `cargo test -p qwik-optimizer-oxc -- --test-threads=1 -q` (quick subset)
- **Per wave merge:** `cargo test -p qwik-optimizer-oxc` (full 201-snapshot suite)
- **Phase gate:** Full suite green + `cargo clippy -p qwik-optimizer-oxc` clean

### Wave 0 Gaps

- [ ] `crates/qwik-optimizer-oxc/` -- crate does not exist yet, needs creation
- [ ] `Cargo.toml` workspace member addition
- [ ] `tests/snapshot_tests.rs` -- test harness that parses `fixtures.json` and runs each fixture through `transform_modules()`, comparing output against `.snap` files
- [ ] `tests/snapshots/` -- copy of 201 `.snap` files from `swc-snapshots/`
- [ ] `fixtures.json` -- copy from Jack's repo (201 fixture configs)
- [ ] `tests/spec_examples.rs` -- behavioral tests derived from spec Appendix B (24 examples)

### Test Harness Design

The snapshot test harness needs to:
1. Read `fixtures.json` to get test configs (mode, entry strategy, strip options, input code)
2. Call `transform_modules()` with the fixture config
3. Format the `TransformOutput` into the `.snap` format (INPUT + segments + root + diagnostics)
4. Compare against the corresponding `.snap` file using insta

This is not a standard insta usage (where insta manages the snapshot files). The `.snap` files use a **custom format** specific to the SWC test harness. The test harness must either:
- (a) Parse the `.snap` file format and compare structurally, or
- (b) Generate the same format and use string comparison with tolerance for cosmetic differences

Option (a) is recommended: parse `.snap` files into structured data (input, segments, root module, diagnostics), run transform, compare output structures semantically.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| Rust toolchain | All | Yes | 1.94.1 (2026-03-25) | -- |
| Cargo | Build | Yes | 1.94.1 | -- |
| OXC 0.123 | Core | Yes (crates.io) | 0.123.0 | -- |
| oxc_traverse 0.123 | Traversal | Yes (crates.io) | 0.123.0 | -- |

**Rust 1.94.1 exceeds MSRV 1.88.** All required crates verified on crates.io as of 2026-04-01. No missing dependencies.

## Open Questions

1. **Snapshot comparison strategy**
   - What we know: `.snap` files use a custom multi-section format, not standard insta format
   - What's unclear: Whether to use insta's native snapshot management or build a custom comparison harness
   - Recommendation: Build a custom test harness that parses `.snap` format and compares structurally. Use insta for any new spec-derived tests that don't need the SWC format.

2. **OXC 0.123 API changes from 0.113**
   - What we know: Jack's code compiles against 0.113. The `traverse_mut` signature is the same at 0.123.
   - What's unclear: Whether any internal API changes (e.g., `SemanticBuilder`, `AstBuilder` methods) require adaptation
   - Recommendation: Start with 0.123 from day one. Fix compile errors if any arise. The 201-test suite will catch behavioral regressions.

3. **Cosmetic differences tolerance**
   - What we know: SWC and OXC codegen produce cosmetically different output (import grouping, whitespace)
   - What's unclear: Which of the 201 snapshots will need "expected output" adjustments for OXC codegen
   - Recommendation: First pass: get all snapshots to compile and run. Second pass: triage failures into semantic (must fix) vs cosmetic (update snapshot). Use `cargo insta review` for mass updates.

## Sources

### Primary (HIGH confidence)
- OXC crates.io: `oxc` 0.123.0, `oxc_traverse` 0.123.0 -- verified 2026-04-01
- [oxc_traverse docs.rs](https://docs.rs/oxc_traverse/0.123.0/oxc_traverse/) -- `traverse_mut` signature verified
- Jack's `qwik-oxc-optimizer` codebase -- 16,577 lines of working reference at OXC 0.113
- `fixtures.json` -- 201 fixture configurations verified
- `swc-snapshots/` -- 201 `.snap` files verified
- Specification: `qwik-optimizer-spec.md` -- 8,091 lines covering all 14 CONVs
- `.planning/research/STACK.md` -- Stack research with version verification
- `.planning/research/ARCHITECTURE.md` -- Three-stage pipeline, component boundaries
- `.planning/research/PITFALLS.md` -- 15 domain pitfalls with prevention strategies

### Secondary (MEDIUM confidence)
- [OXC crates.io page](https://crates.io/crates/oxc) -- version verification
- [OXC Semantic Analysis docs](https://oxc.rs/docs/learn/parser_in_rust/semantic_analysis) -- SemanticBuilder usage

### Tertiary (LOW confidence)
- None. All findings verified against primary sources.

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- all versions verified on crates.io, Jack's working impl validates compatibility
- Architecture: HIGH -- proven by Jack's 16K-line working implementation, documented in ARCHITECTURE.md research
- Pitfalls: HIGH -- derived from Jack's 11-plan bug-fix campaign with specific deviations counted
- Snapshot corpus: HIGH -- all 201 files verified, format examined, CONV coverage catalogued

**Research date:** 2026-04-01
**Valid until:** 2026-05-01 (OXC releases 2-3x/week but core API stable; re-verify version before implementation starts)
