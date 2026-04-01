# Domain Pitfalls: Qwik Optimizer Specification & OXC Implementation

**Domain:** Behavioral specification of a multi-output JS code transformer + OXC-based Rust implementation
**Researched:** 2026-04-01
**Confidence:** HIGH (derived from Jack Shelton's 11-plan bug-fix campaign on the same codebase, 162 spec files, OXC official docs, and direct codebase analysis)

---

## Critical Pitfalls

Mistakes that cause specification gaps, implementation rewrites, or runtime failures in the optimizer.

---

### Pitfall 1: Specifying Capture Semantics Without Exhaustive Edge Case Taxonomy

**What goes wrong:**
The specification describes capture analysis as "variables referenced inside `$()` that are declared outside it" and the implementation treats it as a single algorithm. In reality, Jack's Phase 24 revealed at least 8 distinct capture categories, each with different behavior. Conflating them produces a spec that appears complete but generates runtime `ReferenceError` failures in the 30-40% of specs that exercise non-trivial capture patterns.

The categories that MUST be individually specified:

1. **Module-level declarations used in nested segments** -- NOT captures; they become self-imports (`import { X } from "./module"`). Jack's Plan 05 fixed 46 runtime-breaking deviations from this single category alone. The original implementation silently dropped these references.
2. **User-code imports** -- NOT captures; re-emitted as import statements in the segment module. Requires tracking import kind (default/namespace/named/aliased).
3. **Loop iteration variables** -- ARE captures with special serialization via `_captures[N]`.
4. **Destructured component props** -- Require a dedicated transformation pass (props destructuring) that runs BEFORE capture analysis, changing variable references.
5. **TypeScript type-only imports** -- NOT captures; erased at compile time.
6. **Shadowed variables** -- Inner binding wins; NOT a capture.
7. **Function declarations (hoisted)** -- May or may not be captures depending on reference location.
8. **Pattern destructuring defaults** (e.g., `{I5 = v2}`) -- ARE module-level declarations. Jack's Plan 07 found that `AssignmentPattern` was missed in all three `BindingPattern` match sites.

**Why it happens:**
The SWC source code handles these implicitly through its `SyntaxContext`-based identity system. When reading SWC code, the cases are not explicitly separated -- they emerge from the interaction of multiple passes. A behavioral spec derived from reading SWC code will miss the taxonomy because SWC does not explicitly document it.

**Consequences:**
- 293 runtime-breaking deviations at the start of Jack's Phase 24 -- the majority were capture-related
- Missing captures cause `ReferenceError` at runtime (segment code references undefined variables)
- Extra captures cause unnecessary serialization overhead and larger bundles
- Module-level declarations silently dropped (the worst kind: tests pass syntactically but code crashes at runtime)

**Prevention:**
The specification MUST include an explicit capture classification table with a dedicated section per category, including:
- Whether the symbol becomes a capture, a self-import, a re-emitted import, or is dropped
- Input/output example pairs for each category
- The ordering constraint (props destructuring BEFORE capture analysis)

Write the capture taxonomy section BEFORE writing any other transformation spec. It is the foundation that most other transformations depend on.

**Detection (during spec writing):**
- Count capture categories in the spec. If fewer than 8, categories are missing.
- Check that module-level declarations have a dedicated subsection (this was the single largest bug class).
- Verify that the spec addresses `AssignmentPattern`, `TSEnumDeclaration`, and named default exports as module-level declarations.

**Phase to address:** Phase 1 (core behavioral specification). Capture taxonomy must be in the first draft.

---

### Pitfall 2: Under-Specifying the Module-Level Declaration to Self-Import Reclassification

**What goes wrong:**
When a nested segment (a `$()` inside another `$()`) references a module-level declaration (a `const`, `function`, `class`, or `enum` at the top level of the source file), the SWC optimizer does NOT pass it as a `_captures[]` entry. Instead, it generates a self-import: `import { X } from "./module_stem"`. Jack discovered this mid-implementation (Plan 05) -- his initial approach added module-level declarations as captures, which caused 3 spec regressions because SWC expects `captures: false` for those segments.

This is the single most impactful behavioral distinction in the entire optimizer. Getting it wrong produced 46 of the 52 missing-import-used deviations.

**Why it happens:**
The SWC source code does not have a function called `reclassify_module_level_decl_captures`. The behavior emerges from how SWC's code_move module generates import statements for new modules. Without reading the SWC output carefully (not just the code), this distinction is invisible.

**Consequences:**
- Segments reference symbols that do not exist in their scope (no import, no capture)
- Runtime crashes in lazy-loaded segments
- Test suite may show all metadata "passing" (capture counts match) while generated code is semantically wrong

**Prevention:**
The specification must explicitly state:
- Top-level segments: captures are zeroed out (module-level declarations are available via standard module semantics)
- Nested segments: module-level declarations are converted from capture candidates to `needed_imports` with source `"./module_stem"`
- The reclassification step is a post-processing pass on capture analysis results, NOT part of the core capture algorithm

Include the self-import pattern as a first-class concept in the spec with its own section, not buried in capture analysis.

**Detection:**
- Search the spec for "self-import" or "module_level_decls". If absent, this pitfall is active.
- Check that the spec distinguishes top-level vs nested segment capture behavior.

**Phase to address:** Phase 1 (core behavioral specification), capture analysis section.

---

### Pitfall 3: Specifying OXC Patterns by Translating SWC Idioms

**What goes wrong:**
The specification describes transformations using SWC concepts (fold, ownership transfer, `std::mem::replace`, `SyntaxContext`-based identity) and the implementation ports those patterns to OXC APIs. The result compiles but fights OXC's architecture at every turn: arena lifetime infections spread through all helper functions, borrow conflicts block multi-output module generation, and semantic info invalidation produces silent capture bugs.

Jack's key decisions document records the correct OXC-native patterns that emerged through painful discovery:
- **Two-phase analyze-then-emit** (not SWC's fold-based single-pass)
- **String-based segment code construction** (not AST construction during traversal)
- **Parse-roundtrip for JSX lambda serialization** (because OXC's `inherit_variants!` makes direct Expression/JSXExpression casting unsound)
- **`exit_program` for deferred import insertion** (because Traverse does not allow sibling insertion)
- **Stack-based capture analysis** (not SWC's implicit scope tracking)
- **Span-based tracking with `HashSet<u32>`** (not node identity)

**Why it happens:**
Natural tendency when writing a "behavioral spec derived from SWC" to describe the behavior in SWC's terms. The spec says "the optimizer folds the expression into a QRL wrapper" and the implementer writes a fold-like pattern that hits OXC's borrow checker.

**Consequences:**
- Arena lifetime `'a` infects every helper function signature (dozens of compile errors)
- Cannot build output module ASTs during input traversal (double-borrow on Allocator)
- `move_expression` leaves NullExpression garbage that appears as `null` in output
- Semantic info (ScopeId, SymbolId) goes stale after AST mutation

**Prevention:**
The specification must be behavior-only with explicit OXC migration notes where SWC and OXC diverge. Structure:

1. **Behavioral contract**: "Given input X, the optimizer produces output Y" (no implementation hints)
2. **OXC migration note** (per-transformation): "In OXC, this requires [specific pattern] because [OXC constraint]"

Mandatory migration notes for:
- Arena allocation and lifetime propagation
- Two-phase architecture (analyze-then-mutate)
- Deferred statement insertion via `exit_program`
- String-based segment body construction (avoids separate Allocator per output module)
- Parse-roundtrip for JSX attribute lambda serialization
- `TraverseCtx` as the universal context parameter for AST construction

**Detection:**
- Grep the spec for SWC-specific terms: "fold", "SyntaxContext", "ownership transfer", "std::mem::replace". These indicate SWC-think leaking into the spec.
- Check that every transformation section has both a behavioral contract AND an OXC migration note.

**Phase to address:** All phases. Establish the behavioral-contract + migration-note format in Phase 1. Review for SWC-isms at every phase transition.

---

### Pitfall 4: Parse Error Bailout in Capture Analysis

**What goes wrong:**
The capture analysis function parses extracted lambda bodies to find identifier references. When the parser returns errors, the function bails out and returns empty captures. But OXC's parser reports SEMANTIC errors (like `await` in a non-async function) alongside STRUCTURAL errors (like missing brackets). Semantic errors produce a valid AST that can still be analyzed for captures. Bailing on any error silently drops captures for valid code patterns.

Jack discovered this in Plan 10: the lambda `() => await api()` produced a semantic parse error ("await in non-async function"), causing `analyze_lambda_captures` to return empty results, which meant the `api` symbol was never imported in the segment module.

**Why it happens:**
Defensive programming -- "if the parser returned errors, the AST might be garbage, so don't trust it." This is correct for structural errors but wrong for semantic errors.

**Consequences:**
- Single missing import causes runtime crash in the affected segment
- Bug is invisible in testing unless the specific code pattern (await in non-async, etc.) appears in test cases
- Difficult to debug: the segment compiles fine, but at runtime the symbol is undefined

**Prevention:**
The specification must state: "Capture analysis proceeds on parse results regardless of diagnostic errors. Only bail out if the parsed program body is empty (structural parse failure). Semantic errors (type errors, scope errors) do not invalidate identifier reference extraction."

Include this as a behavioral contract with a test case: `() => await someFunction()` must still capture `someFunction`.

**Detection:**
- Search the spec for parse error handling in capture analysis. If it says "bail on errors" without distinguishing structural vs semantic, this pitfall is active.
- Include the `await` test case in the spec's capture analysis examples.

**Phase to address:** Phase 1 (capture analysis specification). Add as an explicit edge case with test vector.

---

### Pitfall 5: Display Name Collisions in Nested Dollar Calls

**What goes wrong:**
When multiple `$()` calls exist inside the same function (e.g., `component$` wrapping a function that contains event handlers), the display name derivation produces duplicate names. If `renderHeader` contains both a `component$()` wrapper and an `onClick$` handler, both segments may get the name `renderHeader`, causing hash collisions and duplicate `const` declarations in the output.

Jack fixed this in Plan 10 by adding `wrapper_callee_name` context: when `$()` appears as an argument to a non-dollar function like `component(...)`, the wrapper function name is included in the display name (e.g., `renderHeader_component` vs `renderHeader_div_q_e_click`).

**Why it happens:**
The display name algorithm derives names from the enclosing function declaration. Without context about the call site (is this `$()` inside `component()` or standalone?), nested `$()` calls in the same function get identical base names.

**Consequences:**
- Duplicate segment names cause hash collisions (segments overwrite each other)
- Duplicate `const` declarations are invalid JavaScript
- Affects common patterns (every component with event handlers)

**Prevention:**
The specification must document the complete display name derivation algorithm, including:
- Base name from enclosing function declaration
- `scope_prefix` for nested `$()` calls (parent context hierarchy)
- `wrapper_callee_name` for `$()` as argument to non-dollar functions
- JSX element/attribute context for event handler segments (e.g., `div_q_e_click`)

Include test cases with multiple `$()` calls in the same function to validate uniqueness.

**Detection:**
- Check the spec's display name section. If it only covers the "enclosing function name" case, nesting/wrapper cases are missing.
- Verify test vectors include at least: standalone `$()`, `component$(() => ...)`, and `onClick$={...}` within the same source file.

**Phase to address:** Phase 1 (segment extraction specification). Display name uniqueness is a correctness requirement, not cosmetic.

---

### Pitfall 6: Incomplete BindingPattern Exhaustiveness

**What goes wrong:**
OXC's `BindingPattern` has 4 variants: `BindingIdentifier`, `ObjectPattern`, `ArrayPattern`, and `AssignmentPattern`. Code that matches on binding patterns but uses a wildcard `_ => {}` catch-all silently ignores `AssignmentPattern` (destructuring defaults like `{I5 = v2}`). This means variables declared via pattern defaults are not collected as module-level declarations, not tracked for capture analysis, and not available for self-import generation.

Jack found this in Plan 07: the same bug existed in THREE separate pattern-matching sites (one in collector.rs, two in transform.rs). Fixing one site but not the others would have left the bug partially active.

**Why it happens:**
The `AssignmentPattern` variant is less common than the other three. When writing match arms, developers handle the obvious cases and use `_ => {}` for "everything else." Rust's exhaustiveness checking would catch this IF there were no wildcard, but the wildcard suppresses the warning.

**Consequences:**
- Variables declared with destructuring defaults are invisible to the optimizer
- Missing self-imports for those variables cause runtime errors
- Bug is replicated across every site that matches on `BindingPattern`

**Prevention:**
The specification should note that all BindingPattern matching must be exhaustive (no wildcard). The OXC migration notes should explicitly list all 4 variants and state that `AssignmentPattern` recursion is required (it wraps another `BindingPattern`).

For the implementation, the coding standard should mandate: "Never use `_ => {}` on `BindingPattern`. Always match all 4 variants explicitly."

**Detection:**
- Grep implementation for `BindingPattern` match arms with wildcard `_`.
- Check that every BindingPattern match site handles `AssignmentPattern` by recursing into its inner pattern.

**Phase to address:** Implementation phase (coding standard). Specification should mention all 4 variants in the binding analysis section.

---

## Moderate Pitfalls

---

### Pitfall 7: Ordering Constraint Between Transformations Not Explicit in Spec

**What goes wrong:**
The optimizer's 14 transformation types (CONV-01 through CONV-14) have ordering dependencies that, if violated, produce wrong output. Key constraints from Jack's implementation:

- **Props destructuring BEFORE capture analysis** -- destructuring changes variable references that capture analysis reads
- **CONV-10 (const_replace) BEFORE CONV-09 (code stripping)** -- `isServer` replacement creates dead branches that stripping eliminates
- **const_replace as pre-pass** -- runs before `traverse_mut` so segment body serialization sees replaced boolean literals
- **JSX event handler replacement BEFORE JSX transform** -- pre-register replacement info during segment creation, apply before JSX transform runs

**Why it happens:**
Each transformation in isolation is well-defined. The ordering constraints only emerge when you consider data flow between transformations. A spec that documents each transformation independently (as the SWC code is organized) will not surface these.

**Consequences:**
- Dead branches survive in output (larger bundles, possible runtime side effects)
- Capture analysis reads pre-destructuring variable names (wrong captures)
- JSX event handler segments emit raw lambdas instead of `qrl()` calls

**Prevention:**
The specification must include a transformation ordering section that explicitly states:
1. The execution order of all 14 CONVs
2. Which CONVs must run before which others and why
3. Which are pre-passes (before main traversal) vs main-pass vs post-passes

Model this as a dependency DAG, not a flat list.

**Detection:**
- Check the spec for a transformation ordering section. If absent, ordering constraints are implicit and likely wrong.
- Verify the spec states props_destructuring -> capture_analysis ordering.
- Verify CONV-10 -> CONV-09 ordering.

**Phase to address:** Phase 1 (transformation catalog). Include the ordering DAG alongside the individual transformation specs.

---

### Pitfall 8: JSX Expression/Expression Type Casting Is Unsound in OXC

**What goes wrong:**
OXC's `inherit_variants!` macro makes `JSXExpression` and `Expression` share variant types (e.g., both have `ArrowFunctionExpression`), but they are different enum layouts. Attempting to cast between them via raw pointer is unsound and produces undefined behavior. Jack discovered this in Plan 01 and abandoned the approach.

The correct pattern is parse-roundtrip: extract the lambda source by span, wrap as `var x = <lambda>`, parse with OXC parser, and codegen the init expression.

**Why it happens:**
The shared variant types create the illusion that `JSXExpression` and `Expression` are interchangeable. The types even have the same `Box<ArrowFunctionExpression>` inner type. But Rust enum layout is not guaranteed to be the same between different enums with the same variants.

**Consequences:**
- Undefined behavior (use-after-free, wrong variant tag)
- Silent data corruption in generated output
- Unsafe code that appears to work in tests but fails on different inputs

**Prevention:**
The specification's OXC migration notes must explicitly state: "JSXExpression and Expression are NOT castable despite shared variant types. Use parse-roundtrip for JSX attribute lambda serialization."

The spec should include the parse-roundtrip pattern as the canonical approach for extracting expression code from JSX attribute positions.

**Detection:**
- Any `unsafe` block in the implementation related to Expression/JSXExpression conversion
- Any `transmute` or pointer cast between these types

**Phase to address:** JSX transformation specification. Include as an OXC migration note.

---

### Pitfall 9: String-Based Segment Construction vs AST Construction

**What goes wrong:**
The natural approach for building segment output modules is AST construction: create a new `Program`, add import declarations, add the segment body as statements, run codegen. But OXC requires a separate `Allocator` per `Program`, and building complex AST subtrees with `AstBuilder` is extremely verbose (20+ lines per import declaration).

Jack's key decision: use string-based segment code construction. Build the segment module as a string (`code_move` builds JS as string), then run `normalize_code` (parse + codegen) to format it. This is simpler, produces identity-like source maps, and avoids the separate-allocator-per-output constraint.

**Why it happens:**
AST construction seems "more correct" and is the approach used in SWC (which has no arena allocation constraint). But in OXC, the cost is enormous: verbose `AstBuilder` calls, separate allocators, lifetime management for each output module.

**Consequences (if AST construction is chosen):**
- 5-10x more code for segment module generation
- Separate Allocator per output module adds complexity
- `AstBuilder` calls are so verbose they obscure the intent
- Lifetime errors when output modules reference input allocator

**Prevention:**
The specification should describe segment output modules in terms of their string representation (the expected JavaScript output), not as AST construction recipes. The OXC migration notes should explicitly recommend string-based construction with parse-roundtrip normalization.

The spec's input/output examples already use string form (JavaScript code). Maintain this as the canonical representation.

**Detection:**
- Check if the spec describes segment output as AST construction steps. If so, it's guiding toward the wrong implementation pattern.
- The spec should describe output as "the segment module contains this JavaScript code" not "construct these AST nodes."

**Phase to address:** Segment extraction specification. Describe output as code strings with parse-roundtrip normalization noted in OXC migration section.

---

### Pitfall 10: PURE Annotation Scope -- Only componentQrl Is Tree-Shakeable

**What goes wrong:**
The optimizer marks calls with `/*#__PURE__*/` annotations so bundlers can tree-shake unused code. The temptation is to annotate ALL `*Qrl()` wrapper calls. But Jack's key decision states: "componentQrl-only PURE. Only component$ is tree-shakeable; all other Qrl wrappers are side-effectful."

Annotating non-component QRL calls (like `useTask$`, `useVisibleTask$`, `server$`) as PURE tells the bundler they have no side effects, which causes the bundler to drop them. These hooks DO have side effects (registering tasks, server endpoints) and must not be removed.

**Why it happens:**
SWC's PURE annotation logic is buried in a conditional check that is easy to miss when reading the source. The behavioral difference between `component$` and other dollar functions is not obvious from the API surface.

**Consequences:**
- Hooks silently removed by bundler in production builds
- `useTask$` callbacks never execute
- Server endpoints disappear
- Bugs only manifest in production builds with tree-shaking enabled

**Prevention:**
The specification must explicitly list which QRL wrappers receive PURE annotations. A table:

| Wrapper | PURE | Reason |
|---------|------|--------|
| `componentQrl` | YES | Components are declarative, tree-shakeable |
| `useTaskQrl` | NO | Registers side-effectful task |
| `useVisibleTaskQrl` | NO | Registers side-effectful task |
| `serverQrl` / `server$` | NO | Registers server endpoint |
| All other `*Qrl` | NO | Side-effectful by default |

**Detection:**
- Check the spec's PURE annotation section. If it says "annotate all QRL wrappers," this pitfall is active.
- Verify the spec has a whitelist (not blacklist) approach: only listed wrappers get PURE.

**Phase to address:** Phase 1 (transformation catalog, PURE annotation section).

---

## Minor Pitfalls

---

### Pitfall 11: OXC Parser Is Stricter Than SWC on Invalid Source

**What goes wrong:**
3 of Jack's 162 spec files had invalid input code that SWC accepted but OXC rejected. Examples: trailing `});` instead of `};`, placeholder `{...}` in JSX attributes, bare `[].map(...)` as JSX child without expression container braces. The OXC parser fails on these, producing 0 output modules.

**Prevention:**
The specification should note that all input examples must be valid JavaScript/TypeScript/JSX/TSX. Do not copy SWC test inputs verbatim without validation. Run every spec input through the OXC parser during spec generation to catch syntax issues early.

**Detection:**
- Any spec input that produces parser errors when fed to OXC.
- Run `oxc_parser::parse()` on every spec input as a validation step.

**Phase to address:** Spec generation (Phase 1). Validate inputs during spec creation.

---

### Pitfall 12: Import Deduplication and Grouping Differences

**What goes wrong:**
SWC and OXC produce cosmetically different import groupings and deduplication. SWC may merge `import { a } from 'x'` and `import { b } from 'x'` into `import { a, b } from 'x'` while OXC emits them separately. These differences are cosmetic but cause test failures if the spec uses byte-for-byte comparison.

**Prevention:**
The specification must use semantic comparison for output validation, not string equality. Define "functionally equivalent output" as: same imports (regardless of grouping), same exports, same code semantics, same segment metadata. Jack's project locked this decision early: "Fix only runtime-breaking deviations; accept benign differences."

**Detection:**
- Test failures on import ordering or grouping that do not affect runtime behavior.
- 287 "codegen-style" cosmetic deviations in Jack's final audit -- all acceptable.

**Phase to address:** Test infrastructure (Phase 1). Define semantic comparison criteria in the spec.

---

### Pitfall 13: Naming Convention Differences in JSX Element Path Depth

**What goes wrong:**
SWC includes the full JSX element hierarchy in display names (e.g., `Foo_component_div_button_q_e_click`) while OXC may produce shorter paths (e.g., `Foo_component_button_q_e_click`). This affects 63 naming-convention pairs in Jack's audit. The modules themselves are correct -- only the display name path depth differs, which changes hashes.

**Prevention:**
The specification should define the exact display name derivation algorithm including JSX element path traversal depth. State whether intermediate container elements (like wrapping `<div>`) are included or skipped.

If the spec matches SWC exactly, implement full JSX hierarchy traversal. If cosmetic differences are acceptable, document which depth is used and accept the hash differences.

**Detection:**
- Display name mismatches between expected and actual output.
- Hash differences that trace back to display name differences.

**Phase to address:** Display name specification section. Decide on the canonical depth algorithm.

---

## Specification-Writing Pitfalls

---

### Pitfall 14: Spec Describes "What SWC Does" Instead of "What the Optimizer Must Do"

**What goes wrong:**
The specification is derived from SWC source code and describes SWC's implementation choices as behavioral requirements. Example: "The optimizer folds the call expression" (SWC implementation detail) vs "The optimizer replaces the `$()` call with a `qrl()` reference" (behavioral contract). The implementation team then ports SWC patterns instead of building OXC-native solutions.

**Prevention:**
Every spec section must pass the "implementation-agnostic test": could this section be implemented in a completely different language/framework without referencing SWC? If not, it contains implementation leakage.

Format: Input code -> Expected output code -> Behavioral rules -> OXC migration notes (separate section).

**Detection:**
- Grep the spec for SWC-specific terms: "fold", "visit_mut_", "Mark", "SyntaxContext", "hygiene".
- Any reference to SWC module names (transform.rs, code_move.rs) in the behavioral section.

**Phase to address:** Phase 1, continuously through all phases.

---

### Pitfall 15: Single-Boundary Test Cases Hide Multi-Boundary Interaction Bugs

**What goes wrong:**
Most simple test cases have one `$()` boundary. The optimizer works perfectly for single-boundary cases but fails when multiple `$()` calls interact: nested `$()` inside `$()`, multiple event handlers in the same component, `$()` calls referencing the same module-level declarations. Jack's capture_stack design specifically handles nested `$()` scopes by merging ALL frames, not just the innermost.

**Prevention:**
The specification must include multi-boundary test vectors as first-class examples, not just edge cases:
- Component with 3+ event handlers referencing shared state
- Nested `$()` calls (component -> task -> event handler)
- Multiple segments referencing the same module-level declaration
- `$()` as argument to both dollar-suffixed and non-dollar-suffixed functions in the same file

**Detection:**
- Count `$()` occurrences in spec test vectors. If most have only 1, multi-boundary cases are under-represented.
- Check for nested `$()` test cases specifically.

**Phase to address:** Phase 1 (test vector selection). Prioritize multi-boundary cases.

---

## Phase-Specific Warnings

| Phase Topic | Likely Pitfall | Mitigation |
|-------------|---------------|------------|
| Capture analysis spec | Pitfall 1, 2, 4 | Write capture taxonomy FIRST with all 8 categories and self-import reclassification. Include parse-error tolerance rule. |
| Transformation catalog | Pitfall 7, 10, 14 | Include ordering DAG. Whitelist PURE annotations. Use behavioral language, not SWC terms. |
| Segment extraction spec | Pitfall 5, 9 | Document full display name algorithm with wrapper_callee_name. Describe output as code strings. |
| JSX transformation spec | Pitfall 8 | Document parse-roundtrip as canonical JSX lambda serialization pattern. |
| OXC migration notes | Pitfall 3, 6 | Two-phase architecture. Exhaustive BindingPattern matching. Arena lifetime conventions. |
| Test infrastructure | Pitfall 11, 12, 15 | Validate inputs with OXC parser. Semantic comparison. Multi-boundary test vectors. |
| Implementation coding standard | Pitfall 6 | No wildcard on BindingPattern. Separate analysis functions (no `'a`) from construction functions (require `'a`). |

---

## Sources

- Jack Shelton's qwik-oxc-optimizer PROJECT.md -- Key Decisions table (28 validated decisions)
- Phase 24 FINAL-AUDIT.md -- 293 -> 4 runtime-breaking deviations across 11 plans
- Phase 24 Plan 01 SUMMARY -- Parse-roundtrip discovery, JSX Expression casting unsoundness
- Phase 24 Plan 02 SUMMARY -- ImportKind/ReemittedImport infrastructure for import re-emission
- Phase 24 Plan 03 SUMMARY -- Capture stack frame merging, UpdateExpression member walking
- Phase 24 Plan 05 SUMMARY -- Module-level declaration self-import reclassification (46 deviations fixed)
- Phase 24 Plan 07 SUMMARY -- Exhaustive BindingPattern matching, TSEnumDeclaration collection
- Phase 24 Plan 10 SUMMARY -- Parse error bailout fix, display name collision fix
- Phase 24 VERIFICATION.md -- 4/4 success criteria, regression threshold assertion
- [OXC AST Design](https://oxc.rs/docs/contribute/parser/ast) -- BindingIdentifier vs IdentifierReference distinction
- [OXC Transformer Alpha](https://oxc.rs/blog/2024-09-29-transformer-alpha) -- Babel-modeled transforms, cosmetic difference constraints
- [OXC AST Tools](https://oxc.rs/docs/learn/architecture/ast-tools) -- Code generation, no build.rs policy
- [OXC Transformer Documentation](https://oxc.rs/docs/guide/usage/transformer.html) -- Usage patterns
- [OXC GitHub](https://github.com/oxc-project/oxc) -- Issue tracker for traverse API patterns
