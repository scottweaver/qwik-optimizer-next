# Phase 11: Root Module Code Generation - Research

**Researched:** 2026-04-06
**Domain:** OXC AST mutation, import management, dead code elimination, root module codegen
**Confidence:** HIGH

## Summary

Phase 11 must fix 173 root module mismatches (28/201 currently matching, target 201/201). Systematic diff analysis of all 201 fixtures reveals **three dominant mismatch categories**: dead imports not removed (136 fixtures), body structure differences (114 fixtures), and hash/naming mismatches (32 fixtures). Many fixtures have overlapping issues.

The root causes are well-understood by comparing the SWC pipeline with our OXC implementation:
1. **Missing dead code elimination** -- SWC runs `swc_ecma_transforms::simplify::simplifier` (full DCE) multiple times; our OXC code only has shallow import elimination that misses most cases
2. **Missing unused import removal after variable migration** -- SWC's `remove_unused_qrl_declarations` removes both unused vars AND unused imports in a fixpoint loop; our Step 9 only removes `_qrl_`/`i_` prefixed vars
3. **Wrong source module for QRL wrapper imports** -- 10 fixtures have wrapper imports (e.g., `globalActionQrl`) sourced from `@qwik.dev/core` instead of the original marker function's source module (e.g., `@qwik.dev/router`)
4. **Missing `collect.synthetic` import emission** -- SWC emits synthetic imports from `global_collect.synthetic` in `fold_module`; our `exit_program` does not
5. **Hash/display_name computation differences** -- 32 fixtures produce different segment names, affecting root module `const q_...` declarations

**Primary recommendation:** Implement a comprehensive dead import/dead code elimination pass that runs after variable migration, fix QRL wrapper import source tracking, and address display_name computation to achieve full root module parity.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Systematic diff analysis first -- categorize all 173 root mismatches by type before fixing
- **D-02:** Holistic rewrite of import insertion logic in `exit_program` to produce SWC-identical ordering
- **D-03:** Import ordering: framework QRL wrapper imports first, then utility imports (qrl, inlinedQrl), then segment imports, then original source re-exports
- **D-04:** Fix codegen output to match SWC exactly; `normalize()` should only normalize truly cosmetic differences
- **D-05:** Focus formatting fixes on statement ordering, expression structure, and whitespace between logical sections
- **D-06:** Fix declaration generation to match SWC structure, leveraging `dependency_analysis.rs`
- **D-07:** Export structure must match SWC exactly -- ordering and separate vs. combined export statements
- **D-08:** QRL const declarations and `.s()` ref assignments must match SWC naming, placement, and expression structure
- **D-09:** Separator comment pattern between logical sections must match SWC whitespace structure

### Claude's Discretion
- Order of implementation across categories (imports first vs. exports first vs. most impactful first)
- Whether to create a comprehensive diff analysis tool/script or do manual categorization
- Exact number of plans to break the work into
- How to handle the ~14 fixtures with combined root+segment or root+diag issues

### Deferred Ideas (OUT OF SCOPE)
None -- analysis stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ROOT-01 | Root module import statements match SWC ordering and format | Import ordering analysis complete; source module tracking fix identified; synthetic import emission gap found; dead import elimination gap is primary blocker |
| ROOT-02 | Root module variable declarations and expressions match SWC output | Variable migration cleanup gap identified; missing import removal after migration; body structure analysis shows 114 fixtures affected |
| ROOT-03 | Root module export structure matches SWC output | Export handling largely correct; main issues are downstream of dead code elimination (vars not removed -> exports not removed) |
| ROOT-04 | Root module QRL references and hoisted declarations match SWC format | 32 fixtures have hash/naming mismatches affecting `const q_*` declarations; display_name computation differences need investigation |
| ROOT-05 | Root module comment separators and whitespace structure match SWC output | `emit.rs::insert_separator_comments` already implements basic separator logic; may need refinement after other fixes change output structure |
</phase_requirements>

## Architecture Patterns

### Root Module Generation Pipeline (SWC Reference)

The SWC optimizer produces root module output through this pipeline:

```
1. simplify::simplifier (DCE pass 1)        -- strip unused code
2. fold_module (transform)                    -- rewrite $-calls to QRL wrappers
   a. fold each ModuleItem (marker detection, QRL emission)
   b. emit synthetic imports from global_collect.synthetic
   c. collect_needed_extra_top_items (transitive closure of needed consts)
   d. partition ALL items into imports vs non-imports
   e. order non-imports by dependency (topological sort)
   f. deduplicate declarations by symbol name
   g. assemble: imports first, then ordered non-imports, then extra_bottom_items
3. SideEffectVisitor (Inline/Hoist) OR CleanSideEffects (Segment)
4. apply_variable_migration
5. remove_migrated_exports
6. remove_unused_qrl_declarations (fixpoint: remove unused vars AND imports)
7. simplify::simplifier (DCE pass 2)        -- clean up after migration
8. hygiene + fixer passes
```

### OXC Pipeline (Current -- showing gaps)

```
1. (NO equivalent DCE pass)
2. traverse_mut with QwikTransform
   - exit_program:
     a. build imports_to_add list (qrl, inlinedQrl, etc.)
     b. build wrapper_imports from segments  ** USES self.core_module, NOT original source **
     c. insert regular imports at pos 0 (reverse order)
     d. insert wrapper imports at pos 0 (reverse order) -- ends up BEFORE regular
     e. strip marker function imports from core module
     f. emit extra_top_items const declarations
     g. emit ref_assignments
     h. dead import elimination (shallow -- only checks referenced idents)
   ** MISSING: synthetic import emission from collect.synthetic **
3. SideEffectVisitor (Inline/Hoist) OR Treeshaker (Segment)
4. apply_variable_migration (lib.rs)
   a. analyze deps, find migratable vars
   b. remove migrated var declarations
   c. remove _auto_ export specifiers
   d. remove_unused_qrl_declarations (only _qrl_/i_ prefixed -- NO import removal)
   ** MISSING: unused import removal after migration **
   ** MISSING: second DCE pass **
5. emit_module (codegen)
```

### Key Gaps to Fix

| Gap | SWC Behavior | OXC Current | Impact |
|-----|-------------|-------------|--------|
| Dead import elimination | Full fixpoint loop removing both vars AND imports | Only removes core module specifiers by ident reference | 136 fixtures |
| QRL wrapper import source | Uses `import.source` from original marker function | Always uses `self.core_module` | 10 fixtures |
| Synthetic imports | Emits `collect.synthetic` as import declarations | Not emitted in exit_program | Multiple fixtures (props destructuring) |
| Post-migration cleanup | `remove_unused_qrl_declarations` + `simplify::simplifier` | Only removes `_qrl_`/`i_` vars, no import cleanup | ~30+ fixtures |
| Variable migration import retention | Removed by post-migration DCE | Third-party imports for migrated vars stay in root | 59+ lines across many fixtures |
| Build-condition imports | Removed by DCE when code is dead | Stay in root module | 6 fixtures |

### Recommended Fix Architecture

```
exit_program changes:
  1. Track original source module per marker function
  2. Use tracked source for QRL wrapper imports (not always core_module)
  3. Emit collect.synthetic imports
  4. Enhanced import ordering (D-03)

Post-migration changes (lib.rs):
  5. Extend remove_unused_qrl_declarations to also remove unused imports
  6. Implement general dead import elimination (fixpoint)
  7. Remove imports only used by migrated vars
  8. Remove var declarations only used by migrated code
```

### SWC fold_module Assembly Order (Critical Reference)

```rust
// SWC fold_module assembly order:
body = [];
body.extend(synthetic_imports);        // from global_collect.synthetic
extra_top_items = collect_needed_extra_top_items(...);
(extra_imports, extra_non_imports) = partition(extra_top_items);
(module_imports, module_non_imports) = partition(module_body);
body.extend(extra_imports);            // synthetic QRL imports
body.extend(module_imports);           // original source imports (cleaned)
non_imports = order_by_dependency(
    extra_non_imports              // QRL const declarations
    .chain(ref_assignments)        // .s() calls
    .chain(module_non_imports)     // original module body
);
deduplicated = deduplicate_by_symbol(non_imports);
body.extend(deduplicated);
body.extend(extra_bottom_items);       // auto-exports
```

This produces the observed SWC output order:
1. QRL wrapper imports (`componentQrl`, `globalActionQrl`, etc.) -- from original source modules
2. Utility imports (`qrl`, `inlinedQrl`) -- from core module
3. Remaining original imports (non-marker, not dead)
4. `//` separator
5. `const q_*` declarations (QRL references)
6. `//` separator
7. Module body (exports, expressions, etc.)

## Mismatch Categorization (Quantified)

### By Category (lines affected, with fixture overlap)

| Category | Fixtures | Lines | Root Cause |
|----------|----------|-------|------------|
| Dead imports not removed | 136 | 277+ | Missing comprehensive DCE |
| Body structure differences | 114 | 181+ | Vars not stripped, expressions not simplified |
| Hash/naming differences | 32 | 64+ | display_name computation divergence |
| Wrong import source | 10 | 10 | QRL wrappers always from core_module |

### Dead Import Breakdown (by import type)

| Import Type | Line Count | Fix Strategy |
|-------------|------------|-------------|
| QRL wrapper imports (e.g., `useStylesQrl`) | 152 | These appear when segment code uses them but root doesn't -- remove via dead import elimination |
| Third-party imports (e.g., `deps`, `mongodb`) | 59 | Only used by migrated vars -- remove after variable migration |
| Core module imports (migrated var deps) | 44 | Same as above |
| Marker functions not stripped (e.g., `globalAction$`) | 10 | Fix marker stripping to include non-core sources |
| CSS imports | 6 | Only used by migrated useStyles code |
| Build-condition imports | 6 | Removed by SWC's DCE when wrapped in `if (false)` blocks |

### Fixture Overlap Analysis

| Pattern | Count | Strategy |
|---------|-------|----------|
| Only dead imports | 52 | Dead import elimination alone fixes these |
| Only body differences | 30 | Variable declaration cleanup, expression structure |
| Both dead imports + body | 84 | Need both DCE and body fixes |
| Only hash differences | 2 | Fix display_name computation |

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Full JS DCE | Complete dead code elimination from scratch | Targeted fixpoint loop matching SWC's `remove_unused_qrl_declarations` pattern | SWC uses its own `simplify::simplifier` which is thousands of lines; we only need the import/var removal subset |
| Import deduplication | Custom import merge logic | Extend existing `program.body.retain_mut` pattern | Already have the pattern in exit_program |
| Topological sort | Custom dependency ordering | Port SWC's `order_items_by_dependency` approach | ~70 lines, well-defined algorithm |

## Common Pitfalls

### Pitfall 1: Incomplete Dead Import Elimination
**What goes wrong:** Removing vars but not their imports, or removing imports but not vars that depend on them.
**Why it happens:** Single-pass elimination misses transitive dependencies.
**How to avoid:** Implement fixpoint loop: remove unused vars -> check if any imports became unused -> repeat until stable.
**Warning signs:** Test fixtures still have extra import lines after fix.

### Pitfall 2: QRL Wrapper Source Module
**What goes wrong:** `globalActionQrl` imported from `@qwik.dev/core` instead of `@qwik.dev/router`.
**Why it happens:** `exit_program` hardcodes `self.core_module` for all wrapper imports.
**How to avoid:** Track the original import source for each marker function. When emitting the QRL wrapper, use the tracked source.
**Warning signs:** Fixtures with non-core marker functions (globalAction$, serverAuth$, qwikify$, formAction$) show wrong import source.

### Pitfall 3: Import Ordering Sensitivity
**What goes wrong:** Same imports in different order fails normalized comparison.
**Why it happens:** BTreeSet ordering vs. SWC's insertion-order-preserving approach.
**How to avoid:** Match SWC's exact ordering: extra_imports first, then module_imports. Within each group, maintain original source order.
**Warning signs:** Fixtures where all imports are present but in wrong order.

### Pitfall 4: Display Name / Hash Computation Divergence
**What goes wrong:** 32 fixtures produce different segment hashes, causing different `const q_*` names.
**Why it happens:** The `stack_ctxt` at the point of segment registration differs between SWC and OXC, or the `display_name_core` computation differs.
**How to avoid:** Compare stack_ctxt values at registration time for specific failing fixtures. The hash function itself is verified correct (golden tests pass). The difference is in the INPUT to the hash.
**Warning signs:** `const q_renderHeader2_component_Ay6ibkfFYsw` (SWC has `component_` prefix) vs `const q_renderHeader2_PJjSfD4ZE08` (OXC missing `component_` in stack).

### Pitfall 5: Variable Declaration Retention
**What goes wrong:** `const renderHeader2 = component(q_...)` stays in root when SWC strips it to just `component(q_...)`.
**Why it happens:** SWC's DCE removes the variable declaration when the var is unused; our code keeps it.
**How to avoid:** After migration, check if non-exported variable declarations are referenced elsewhere in the root module. If not, either strip the `const name =` prefix (keeping the call expression) or remove entirely.
**Warning signs:** Fixtures showing `const name = wrapper(q_...)` where SWC shows just `wrapper(q_...)`.

## Code Examples

### Fix 1: Track Marker Function Source Module

The QwikTransform should track the source module for each marker function, not just its name.

```rust
// In QwikTransform struct, add:
/// Maps marker function ctx_name (e.g., "globalAction$") to its import source module
marker_fn_sources: HashMap<String, String>,

// When detecting a marker function call, record its source:
if let Some(import) = self.collect.imports.get(callee_name) {
    self.marker_fn_sources.insert(
        callee_name.to_string(),
        import.source.clone(),
    );
}

// In exit_program, use tracked source for wrapper imports:
for seg in &self.segments {
    let wrapper_name = words::dollar_to_qrl_name(&seg.ctx_name);
    let source = self.marker_fn_sources
        .get(&seg.ctx_name)
        .unwrap_or(&self.core_module);
    let import_str = format!(r#"import {{ {} }} from "{}";"#, wrapper_name, source);
    // ...
}
```

### Fix 2: Comprehensive Dead Import Elimination (Fixpoint)

After variable migration, run a fixpoint loop that removes both unused vars AND unused imports:

```rust
// In lib.rs, after Step 8b (remove _auto_ exports), replace Step 9:
loop {
    // Collect all ident references from non-removable statements
    let mut used: HashSet<String> = HashSet::new();
    for stmt in program.body.iter() {
        let is_removable = match stmt {
            Statement::VariableDeclaration(decl) => {
                decl.declarations.iter().all(|d| {
                    if let BindingPattern::BindingIdentifier(id) = &d.id {
                        let name = id.name.as_str();
                        name.starts_with("_qrl_") || name.starts_with("i_")
                    } else { false }
                })
            }
            Statement::ImportDeclaration(_) => true,
            _ => false,
        };
        if !is_removable {
            // Visit and collect all identifier references
            let mut collector = IdentRefCollector::default();
            collector.visit_statement(stmt);
            used.extend(collector.names);
        }
    }

    // Propagate: if a removable item is used, its references are also used
    let mut changed = true;
    while changed {
        changed = false;
        for stmt in program.body.iter() {
            // Check if this item defines something that's used
            // If so, mark its references as used too
            // ... (transitive closure)
        }
    }

    // Remove unused
    let before_len = program.body.len();
    program.body.retain(|stmt| {
        match stmt {
            Statement::VariableDeclaration(decl) => {
                // Remove _qrl_/i_ declarations where defined name is not in `used`
                // ...
            }
            Statement::ImportDeclaration(import_decl) => {
                // Remove imports where ALL specifiers are unused
                // ...
            }
            _ => true,
        }
    });

    if program.body.len() == before_len { break; }
}
```

### Fix 3: Emit Synthetic Imports

```rust
// In exit_program, before inserting wrapper imports:
for (local_name, import) in &self.collect.synthetic {
    let import_str = format!(
        r#"import {{ {} }} from "{}";"#,
        local_name, import.source
    );
    if let Some(stmt) = parse_single_statement(&import_str, allocator) {
        program.body.insert(0, stmt);
    }
}
```

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| Shallow ident-based import elimination | Need fixpoint dead import elimination | This phase | 136 fixtures |
| Hardcoded core_module for wrapper imports | Need per-marker-function source tracking | This phase | 10 fixtures |
| No post-migration import cleanup | Need import removal after variable migration | This phase | 30+ fixtures |

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in Rust testing) + insta 1.47.2 (snapshot testing) |
| Config file | `Cargo.toml` (workspace level) |
| Quick run command | `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture` |
| Full suite command | `cargo test -p qwik-optimizer-oxc --test snapshot_tests` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ROOT-01 | Import ordering matches SWC | integration | `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture` (check "Root module match" count) | Yes |
| ROOT-02 | Variable declarations match SWC | integration | Same parity report | Yes |
| ROOT-03 | Export structure matches SWC | integration | Same parity report | Yes |
| ROOT-04 | QRL references match SWC | integration | Same parity report | Yes |
| ROOT-05 | Comment separators match SWC | integration | Same parity report | Yes |

### Sampling Rate
- **Per task commit:** `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture`
- **Per wave merge:** `cargo test -p qwik-optimizer-oxc --test snapshot_tests`
- **Phase gate:** Full parity report shows 201/201 root module match

### Wave 0 Gaps
None -- existing test infrastructure (parity report + 201 insta snapshots) covers all phase requirements. The parity report test prints exact match counts and per-fixture mismatch categories.

## Open Questions

1. **Display name computation differences (32 fixtures)**
   - What we know: Hash function is verified correct. The difference is in the stack_ctxt input to `register_context_name`. SWC includes the marker function name (e.g., `component`) in the display name for some segments; OXC does not.
   - What's unclear: Exactly which stack_ctxt entries differ for each failing fixture.
   - Recommendation: Add debug logging to `register_context_name` for failing fixtures to compare stack_ctxt. The `component_` prefix in display names like `renderHeader2_component_Ay6ibkfFYsw` suggests SWC pushes the marker function name onto the stack.

2. **Build-condition import removal (6 fixtures)**
   - What we know: SWC's `simplify::simplifier` evaluates `isServer`/`isBrowser` constants and removes dead branches, making some imports unused.
   - What's unclear: Whether our `const_replace` pass handles this correctly. If the branches are correctly replaced with `if (false)`, DCE should handle the rest.
   - Recommendation: Check if `const_replace` is producing `if (false)` blocks, then verify DCE removes the dead imports.

3. **Interaction between fixes**
   - What we know: Fixing dead import elimination should cascade -- removing imports makes some var declarations unused, which makes more imports unused, etc.
   - What's unclear: Exact fixture count improvement from each individual fix.
   - Recommendation: Fix dead import elimination first (highest impact), measure, then address remaining issues.

## Project Constraints (from CLAUDE.md)

- **Behavioral fidelity**: OXC must produce functionally equivalent output to SWC for all 201 test cases
- **OXC idioms**: Use OXC's Traverse trait, arena allocators, SemanticBuilder, Codegen
- **Single spec document**: Specification is one comprehensive markdown file
- **Foundation**: 162 spec files are the behavioral test corpus (+ additional fixtures to 201)
- **No SWC crates**: Do not import any SWC crate
- **No lazy_static**: Use `std::sync::LazyLock`
- **Testing with insta**: Snapshot testing for all fixtures
- **Acceptance**: Must match all `*.snap` files under `tests/swc_expected/`

## Sources

### Primary (HIGH confidence)
- SWC reference: `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` (fold_module at L3329, collect_needed_extra_top_items at L4182, order_items_by_dependency at L4253)
- SWC reference: `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/parse.rs` (remove_unused_qrl_declarations at L1482, apply_variable_migration pipeline at L399-435)
- OXC implementation: `crates/qwik-optimizer-oxc/src/transform.rs` (exit_program at L2464)
- OXC implementation: `crates/qwik-optimizer-oxc/src/lib.rs` (apply_variable_migration at L428)
- Parity report: 28/201 full match, 136 dead import fixtures, 114 body diff fixtures, 32 hash diff fixtures
- Diff analysis: Automated comparison of all 201 fixture pairs (SWC expected vs OXC actual)

### Secondary (MEDIUM confidence)
- Jack's OXC conversion: `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/transform.rs` (reference for OXC-idiomatic patterns)

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH -- no new dependencies needed, all fixes are in existing code
- Architecture: HIGH -- SWC pipeline fully mapped, gaps precisely identified with line numbers
- Pitfalls: HIGH -- mismatch categories quantified from automated analysis of all 201 fixtures

**Research date:** 2026-04-06
**Valid until:** 2026-05-06 (stable -- core optimizer logic is not changing)
