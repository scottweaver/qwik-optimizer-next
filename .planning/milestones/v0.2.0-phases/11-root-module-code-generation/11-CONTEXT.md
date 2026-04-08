# Phase 11: Root Module Code Generation - Context

**Gathered:** 2026-04-06 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

The root module output for every fixture matches SWC in import ordering, variable declarations, export structure, QRL references, and comment separators. This phase fixes ~159 root-only mismatches (plus ~14 root+segment/diag combined). It does NOT address segment code content (out of scope), diagnostics (Phase 12), or new transformation types.

Current baseline: 28/201 (14%) root module match. Target: 201/201.

</domain>

<decisions>
## Implementation Decisions

### Mismatch Categorization Strategy
- **D-01:** Systematic diff analysis first — categorize all 173 root mismatches by type (import ordering, declaration structure, export format, QRL references, comment/whitespace) before fixing. This gives the planner clear categories to create targeted plans rather than fixture-by-fixture debugging.

### Import Ordering Approach
- **D-02:** Holistic rewrite of import insertion logic in `exit_program` to produce SWC-identical ordering. The current approach inserts at position 0 in reverse order which may not match SWC's ordering semantics. Patching individual ordering bugs would be fragile.
- **D-03:** Import ordering must respect SWC's ordering: framework QRL wrapper imports first, then utility imports (qrl, inlinedQrl), then segment imports, then original source re-exports.

### Codegen Formatting Parity
- **D-04:** Fix codegen output to match SWC exactly where possible. The existing `normalize()` function in the parity test should only normalize truly cosmetic/non-semantic differences (trailing whitespace, final newlines). Actual code structure differences (semicolons, expression ordering, parenthesization) must be fixed in the codegen/transform output.
- **D-05:** OXC Codegen double-quoted strings already match SWC. Focus formatting fixes on: statement ordering, expression structure, and whitespace between logical sections.

### Variable Declaration & Export Handling
- **D-06:** Fix declaration generation to match SWC structure, leveraging the existing `dependency_analysis.rs` infrastructure for analyzing root module variable usage and migration.
- **D-07:** Export structure (default exports, re-exports, named exports) must match SWC format exactly — this includes export ordering and whether items are separate export statements vs. combined.

### QRL References & Hoisted Constants
- **D-08:** QRL const declarations (`const q_sym = ...`) and `.s()` ref assignments must match SWC's naming convention, placement, and expression structure.
- **D-09:** The separator comment pattern between logical sections (imports → QRL consts → body) must match SWC whitespace structure.

### Claude's Discretion
- Order of implementation across categories (imports first vs. exports first vs. most impactful first)
- Whether to create a comprehensive diff analysis tool/script or do manual categorization
- Exact number of plans to break the work into
- How to handle the ~14 fixtures with combined root+segment or root+diag issues

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### OXC Implementation (current codebase)
- `crates/qwik-optimizer-oxc/src/transform.rs` — Main transform logic, `exit_program` (root module generation at ~line 2500+), import insertion, marker stripping, QRL const emission
- `crates/qwik-optimizer-oxc/src/lib.rs` — `transform_module` pipeline, root module construction (~line 200-220), `apply_variable_migration` (~line 428)
- `crates/qwik-optimizer-oxc/src/emit.rs` — OXC Codegen wrapper, `emit_module` function
- `crates/qwik-optimizer-oxc/src/dependency_analysis.rs` — Root module dependency analysis, variable migration
- `crates/qwik-optimizer-oxc/src/code_move.rs` — Segment extraction, `generate_imports` function
- `crates/qwik-optimizer-oxc/src/clean_side_effects.rs` — Post-transform root module cleanup
- `crates/qwik-optimizer-oxc/src/add_side_effect.rs` — Side effect import injection
- `crates/qwik-optimizer-oxc/src/filter_exports.rs` — Export filtering/stripping logic

### SWC Reference Implementation
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` — SWC's root module generation, import ordering, QRL emission
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/code_move.rs` — SWC's segment extraction and code movement

### Jack's OXC Conversion
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/transform.rs` — Jack's approach to root module generation

### Test Infrastructure
- `crates/qwik-optimizer-oxc/tests/swc_expected/` — SWC reference snapshots (201 fixtures with expected root module output)
- `crates/qwik-optimizer-oxc/tests/snapshot_tests.rs` — Parity report test (~line 1057), `normalize()` function, snapshot comparison logic

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `transform.rs:exit_program` (~line 2500+): Already handles import insertion, marker stripping, QRL const emission, ref_assignments — this is the primary code to modify
- `dependency_analysis.rs`: Variable migration infrastructure already analyzes which vars are used by root module vs. segments
- `emit.rs:emit_module`: OXC Codegen wrapper producing double-quoted strings (matches SWC)
- `clean_side_effects.rs`: Removes dead code from root module after segment extraction
- `add_side_effect.rs:parse_single_statement`: Utility for parsing synthetic import statements

### Established Patterns
- Two-phase analyze-then-emit: semantic analysis before AST mutation
- Import insertion via `program.body.insert(0, ...)` in reverse order
- Marker function stripping via `program.body.retain_mut`
- QRL const declarations via `extra_top_items` and `ref_assignments` vectors
- `normalize()` in tests: trims whitespace and normalizes line endings for comparison

### Integration Points
- `exit_program` is the main integration point — all root module generation happens here
- `lib.rs:transform_module` calls `emit::emit_module` after transform completes — the Program AST must be correct before this call
- `apply_variable_migration` runs after exit_program, moving declarations between root and segments

### Current Parity State (from parity_report test)
- 28/201 full match (14%)
- 28/201 root module match (14%)
- 195/201 segment count match (97%)
- 193/201 diagnostics match (96%)
- 159 fixtures with root-only mismatches
- 14 fixtures with root + segment/diag combined issues

</code_context>

<specifics>
## Specific Ideas

No specific requirements — open to standard approaches following SWC reference patterns. The key insight from Phase 10 is that OXC's Traverse correctly visits all AST positions, so the focus should be on what the exit_program and codegen pipeline produce, not on traversal coverage.

</specifics>

<deferred>
## Deferred Ideas

None — analysis stayed within phase scope.

</deferred>

---

*Phase: 11-root-module-code-generation*
*Context gathered: 2026-04-06*
