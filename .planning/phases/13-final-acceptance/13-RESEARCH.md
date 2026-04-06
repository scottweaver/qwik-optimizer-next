# Phase 13: Final Acceptance - Research

**Researched:** 2026-04-06
**Domain:** SWC parity -- root module code generation, segment extraction, diagnostic matching
**Confidence:** HIGH

## Summary

The parity report currently shows 79/201 (39%) full match, with 121 root module mismatches and 6 segment count mismatches (diagnostics are 201/201). Automated categorization of all 121 root mismatches reveals they cluster into approximately 10 distinct root cause categories, most of which are overlapping (a single fixture typically has 3-5 issues).

The highest-impact categories by fixture count are: indent style tabs-vs-spaces (62), import count differences (53), missing `/*#__PURE__*/` annotations (48), QRL const declarations missing (35), quote style single-vs-double (33), hash/name differences (28), and missing `_wrapProp` references (16). Many of these are symptoms of deeper structural issues -- for example, the Inline entry strategy being fundamentally wrong (12 fixtures) cascades into import count, PURE, and indent differences for all those fixtures.

**Primary recommendation:** Fix root causes in dependency order: (1) indent/whitespace normalization, (2) quote preservation, (3) QRL const declaration emission, (4) import ordering/counting, (5) hash computation, (6) Inline strategy format, (7) dev mode lo/hi offsets, (8) remaining feature gaps (tagName, _wrapProp, _jsxSplit, TS enum transpilation, dead code stripping, import conflict renaming, Fragment imports). Segment count fixes (6 fixtures) should be addressed after root module issues since some may resolve as side effects.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-01:** Exact match required for all 201 fixtures. The milestone goal (ACC-01) is 201/201 full match. No cosmetic exceptions or "close enough" waivers.
- **D-02:** The existing `parity_report` test in `snapshot_tests.rs` is the acceptance gate. When it reports `Full match: 201/201`, the milestone is complete.
- **D-03:** Diagnose mismatches by category (root cause grouping), then batch fix by shared root cause. Most of the 122 root mismatches likely share a handful of root causes.
- **D-04:** The segment mismatch `should_preserve_non_ident_explicit_captures` (exp=1, act=0) must be investigated separately -- it's a segment extraction bug, not a root module issue.
- **D-05:** After each batch fix, re-run the parity report to track progress and detect regressions.
- **D-06:** Use `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture` as the primary verification tool throughout the phase. No additional test infrastructure needed.
- **D-07:** All existing tests (`cargo test -p qwik-optimizer-oxc`) must continue to pass after each fix (no regressions).
- **D-08:** This phase may require multiple planning iterations. Start with diagnosis to understand the root cause categories, then plan targeted fixes.

### Claude's Discretion
- Ordering of root cause categories to fix
- Whether to split fixes into multiple plans or handle in a single large plan
- Level of detail in diagnostic analysis (aggregate vs per-fixture)

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| ACC-01 | Parity report shows 201/201 full match (root module + segment count + diagnostics) | Comprehensive root cause categorization identifies 10+ categories across 121 root mismatches and 6 segment mismatches. All categories have identified fix strategies. |
</phase_requirements>

## Parity Report Baseline

**Run:** `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture`

```
Full match:          79/201 (39%)
Root module match:   80/201
Segment count match: 195/201
Diagnostics match:   201/201
```

- 121 fixtures have root module mismatches
- 6 fixtures have segment count mismatches (4 also have root mismatches)
- 1 fixture has segment-only mismatch (`relative_paths`)

## Root Cause Categories

### Category 1: Indentation -- Tabs vs Spaces (62 fixtures)

**Impact:** 62/121 root mismatches
**Root cause:** OXC Codegen emits tabs for indentation; SWC emits 4-space indentation.
**Fix strategy:** Configure OXC Codegen `indent` option, or post-process output to replace tabs with 4 spaces.
**Confidence:** HIGH -- this is a codegen configuration issue, not a logic bug.

**Affected fixtures:** destructure_args_inline_cmp_block_stmt, destructure_args_inline_cmp_block_stmt2, destructure_args_inline_cmp_expr_stmt, example_4, example_custom_inlined_functions, example_default_export_index, example_derived_signals_children, example_derived_signals_cmp, example_derived_signals_complext_children, example_derived_signals_div, example_derived_signals_multiple_children, example_dev_mode_inlined, example_drop_side_effects, example_functional_component_2, example_immutable_function_components, example_inlined_entry_strategy, example_input_bind, example_issue_33443, example_issue_4438, example_jsx, example_lib_mode, example_lightweight_functional, example_missing_custom_inlined_functions, example_mutable_children, example_optimization_issue_3542, example_optimization_issue_3561, example_optimization_issue_3795, example_optimization_issue_4386, example_parsed_inlined_qrls, example_preserve_filenames, example_props_optimization, example_props_wrapping, example_props_wrapping2, example_props_wrapping_children, example_props_wrapping_children2, example_qwik_react, example_qwik_react_inline, example_qwik_router_client, example_reg_ctx_name_segments, example_reg_ctx_name_segments_hoisted, example_reg_ctx_name_segments_inlined, example_skip_transform, example_strip_client_code, example_strip_exports_unused, example_strip_exports_used, example_transpile_ts_only, example_ts_enums, example_ts_enums_issue_1341, example_use_optimization, fun_with_scopes, inlined_qrl_uses_identifier_reference_when_hoisted_snapshot, issue_476, root_level_self_referential_qrl_inline, should_keep_module_level_var_used_in_both_main_and_qrl, should_keep_non_migrated_binding_from_shared_destructuring_declarator, should_keep_non_migrated_binding_from_shared_destructuring_with_rest, should_keep_root_var_used_by_exported_function_and_qrl, should_not_generate_conflicting_props_identifiers, should_not_move_over_side_effects, should_preserve_non_ident_explicit_captures, should_wrap_store_expression, special_jsx

### Category 2: Import Count Differences (53 fixtures)

**Impact:** 53/121 root mismatches
**Root cause:** Multiple sub-issues:
- OXC not emitting certain synthesized imports (e.g., `_captures`, `_wrapProp`, `_jsxSplit`, `_getVarProps`, `_getConstProps`, `Fragment`)
- OXC emitting extra imports that should be stripped (e.g., keeping original user imports that SWC strips)
- Import deduplication differences

**Fix strategy:** Audit the import generation pipeline in `transform.rs` exit_program. The SWC version generates imports based on which runtime helpers are actually referenced in the output body. OXC may be missing some of these helper references or not adding the corresponding imports.
**Confidence:** MEDIUM -- this involves multiple sub-issues requiring individual investigation.

### Category 3: Missing `/*#__PURE__*/` Annotations (48 fixtures)

**Impact:** 48/121 root mismatches
**Root cause:** OXC is not adding `/*#__PURE__*/` annotations to QRL wrapper calls, `componentQrl()` calls, `_jsxSorted()` calls, and `_noopQrl()` calls in the root module output. The SWC version marks these as pure for tree-shaking.
**Fix strategy:** Add `/*#__PURE__*/` comment injection for:
- `qrl()` / `qrlDEV()` calls in the QRL declarations section
- `componentQrl()` calls in the body section
- `_jsxSorted()` / `_jsxSplit()` calls in lightweight component bodies
- `_noopQrl()` calls (Inline strategy)

**Confidence:** HIGH -- the pattern is consistent and the fix is mechanical.

### Category 4: QRL Const Declarations Missing (35 fixtures)

**Impact:** 35/121 root mismatches (fully missing); 11 more have partial mismatches
**Root cause:** When the original code has a dollar-sign expression that is NOT assigned to a variable (e.g., bare `component$(() => ...)` call or nested QRL), the SWC version still emits `const q_NAME = /*#__PURE__*/ qrl(...)` as a separate declaration. The OXC version emits the QRL inline as a bare expression.

**Example (example_functional_component):**
```
SWC: const q_Header_component_J4uyIhaBNR4 = /*#__PURE__*/ qrl(()=>import("./test..."), "Header...");
OXC: /*#__PURE__*/ qrl(()=>import("./test..."), "Header...");  // missing const q_ assignment
```

**Fix strategy:** In the QRL declarations section generation, always emit `const q_NAME = ...` even when the original call site was a bare expression. The QRL declaration section should generate named constants for ALL extracted segments.
**Confidence:** HIGH -- the SWC behavior is unambiguous from the snapshots.

### Category 5: Quote Style -- Single vs Double (33 fixtures)

**Impact:** 33/121 root mismatches
**Root cause:** SWC preserves original quote style for user-written imports (e.g., `import { component } from '@qwik.dev/core'` with single quotes). OXC Codegen normalizes all strings to double quotes.
**Fix strategy:** Either:
1. Post-process to restore original quote style for preserved user imports (complex)
2. Accept this as a cosmetic difference and adjust the normalization function in the parity comparison

**IMPORTANT:** Per D-01, exact match is required with no cosmetic exceptions. This means either the OXC output must use single quotes where SWC does, or the comparison must handle this. However, the `normalize()` function already strips whitespace -- it may need to also normalize quotes for comparison purposes. This needs discussion with the user since D-01 says "no cosmetic exceptions."
**Confidence:** MEDIUM -- fix approach depends on interpretation of D-01.

### Category 6: Hash/Name Differences (28 fixtures)

**Impact:** 28/121 root mismatches
**Root cause:** The segment hash computation produces different hashes for some fixtures. This causes the `const q_NAME_HASH` identifiers to differ between SWC and OXC. Root causes include:
- Different segment content being hashed (body extraction differences)
- Different naming conventions (e.g., `Header_component_1_uWM1kg0IGO0` vs `Header_component_UVBJuFYfvDo`)
- The `_1` suffix appearing in OXC but not SWC (counter-based disambiguation)

**Fix strategy:** Investigate the hash computation in `hash.rs` and the naming convention in `words.rs`. The counter suffix `_1` suggests the OXC implementation is adding disambiguation suffixes that SWC doesn't need, likely due to different scope tracking.
**Confidence:** MEDIUM -- requires understanding why hashes differ per fixture.

### Category 7: Missing `_wrapProp` References (16 fixtures)

**Impact:** 16/121 root mismatches
**Root cause:** The JSX transformation is not generating `_wrapProp()` calls for reactive prop access. SWC wraps signal-like prop accesses in `_wrapProp(props, "propName")` for signal tracking.
**Fix strategy:** Implement `_wrapProp` generation in the JSX transform when a prop is used in a reactive context.
**Confidence:** MEDIUM -- requires understanding signal optimization rules.

### Category 8: Export Count Differences (16 fixtures)

**Impact:** 16/121 root mismatches
**Root cause:** Multiple sub-issues:
- Auto-export of vars referenced by QRLs (`_auto_` exports) not working correctly
- Variable migration stripping exports it shouldn't
- `ensure_export` not firing for all needed bindings

**Fix strategy:** Review the variable migration pipeline and ensure_export logic.
**Confidence:** MEDIUM

### Category 9: Inline Entry Strategy Format Wrong (12 fixtures)

**Impact:** 12/121 root mismatches
**Root cause:** When `entry_strategy: "Inline"`, SWC emits a specific format using `_noopQrl("name")` for stripped QRLs and `inlinedQrl(fn, "name", captures)` for inlined QRLs, with `.s()` and `.w()` method chains. The OXC implementation is producing a fundamentally different format -- it's inlining the function body directly rather than using the `_noopQrl`/`inlinedQrl` wrapper pattern.

**Example (example_inlined_entry_strategy):**
```
SWC: const q_Child_component_9GyF01GDKqw = /*#__PURE__*/ _noopQrl("Child_component_9GyF01GDKqw");
     q_Child_component_9GyF01GDKqw.s(()=>{ ... });
OXC: export const Child = componentQrl(() => { ... }, "Child_component_9GyF01GDKqw");
```

**Fix strategy:** The Inline strategy code generation in `entry_strategy.rs` needs significant rework to match SWC's `_noopQrl`/`.s()`/`.w()` pattern.
**Confidence:** HIGH -- the SWC format is clearly documented in snapshots.

### Category 10: Dev Mode lo/hi Offset Differences (5 fixtures)

**Impact:** 5/121 root mismatches
**Root cause:** The `lo` and `hi` byte offsets in `qrlDEV()` calls differ between SWC and OXC. This is because OXC computes offsets differently -- likely using byte offsets from a different source position (e.g., before vs after TypeScript stripping, or different span recording points).

**Example (example_dev_mode):**
```
SWC: lo: 88, hi: 200
OXC: lo: 75, hi: 199
```

**Fix strategy:** Investigate how SWC computes `lo`/`hi` -- it likely uses the span of the arrow function body within the original source. OXC may be recording the span differently.
**Confidence:** MEDIUM -- requires careful span analysis.

### Category 11: Component tagName Option Missing (6 fixtures)

**Impact:** 6/121 root mismatches
**Root cause:** When `component$` is called with a second argument `{ tagName: "my-foo" }`, SWC passes this through to the `componentQrl()` call. OXC drops it.

**Fix strategy:** Preserve the options argument in the `component$` to `componentQrl` rewrite.
**Confidence:** HIGH -- straightforward to fix.

### Category 12: Import Conflict Renaming (2 fixtures)

**Impact:** 2/121 root mismatches (example_qwik_conflict, hmr)
**Root cause:** When user code has a local variable named `componentQrl` (conflicting with the synthesized import), SWC renames the import to `componentQrl1`. OXC doesn't handle this conflict.
**Fix strategy:** Add import deduplication/renaming logic.
**Confidence:** HIGH

### Category 13: Fragment Import Missing (4 fixtures)

**Impact:** 4/121 root mismatches
**Root cause:** JSX Fragment (`<>...</>`) requires `import { Fragment as _Fragment } from "@qwik.dev/core/jsx-runtime"` but OXC is not generating this import.
**Fix strategy:** Add Fragment import when JSX fragments are used.
**Confidence:** HIGH

### Category 14: Dead Code Not Stripped (1 fixture)

**Impact:** 1/121 root mismatches (example_build_server)
**Root cause:** `is_server: true` mode should strip client-only code and replace `if (typeof window !== "undefined")` blocks with `if (false) {}` (SWC further strips these). OXC keeps the full dead code.
**Fix strategy:** Enhance server-mode dead code elimination.
**Confidence:** MEDIUM

### Category 15: TS Enum Not Transpiled (1 fixture)

**Impact:** 1/121 root mismatches (example_ts_enums)
**Root cause:** SWC transpiles TypeScript enums to IIFE form (`function(Thing) { ... }({})`). OXC preserves the `enum` keyword.
**Fix strategy:** Since `transpile_ts: true` for this fixture, the TS enum transpilation needs to be implemented. OXC's transformer feature handles this but we excluded it per CLAUDE.md. May need a targeted enum transpiler.
**Confidence:** MEDIUM -- implementing enum transpilation without OXC's full transformer is non-trivial.

### Category 16: `@jsxImportSource` Not Respected (1 fixture)

**Impact:** 1 root mismatch + 1 segment count mismatch (example_jsx_import_source)
**Root cause:** The `/* @jsxImportSource react */` pragma tells the optimizer to use React's JSX runtime instead of Qwik's. SWC respects this and imports `jsx` from `react/jsx-runtime`, leaving non-Qwik `onClick$` as-is. OXC ignores the pragma and transforms everything through Qwik's JSX pipeline.
**Fix strategy:** Parse `@jsxImportSource` pragma and skip Qwik JSX transforms when a non-Qwik import source is specified.
**Confidence:** HIGH

## Segment Count Mismatches (6 fixtures)

| Fixture | Expected | Actual | Likely Root Cause |
|---------|----------|--------|-------------------|
| example_3 | 2 | 0 | Not extracting segments from nested QRL expressions |
| example_immutable_analysis | 5 | 0 | Immutable analysis features not implemented |
| example_jsx_import_source | 1 | 3 | `@jsxImportSource` not respected, over-extracting |
| example_qwik_react | 2 | 0 | `qwikify$` integration not extracting segments |
| relative_paths | 3 | 1 | Multi-input file handling incomplete |
| should_preserve_non_ident_explicit_captures | 1 | 0 | Non-ident capture preservation bug |

## Overlap Analysis

Most fixtures have multiple overlapping issues. Fixing high-impact categories will cascade:

| Fix Order | Category | Direct Impact | Cascade Effect |
|-----------|----------|---------------|----------------|
| 1 | Indent (tabs->spaces) | 62 fixtures | Reduces noise in all other comparisons |
| 2 | Quote style | 33 fixtures | Depends on D-01 interpretation |
| 3 | QRL const declarations | 35 fixtures | May fix some PURE and import issues |
| 4 | PURE annotations | 48 fixtures | Mechanical fix after QRL consts |
| 5 | Hash computation | 28 fixtures | Independent fix |
| 6 | Inline strategy | 12 fixtures | Fixes import count for those 12 |
| 7 | Import generation | 53 fixtures | Many will be fixed by steps 3-6 |
| 8 | Feature gaps | ~30 fixtures | tagName, _wrapProp, Fragment, etc. |
| 9 | Dev lo/hi | 5 fixtures | Independent fix |
| 10 | Segment extraction | 6 fixtures | Independent investigation |

**Key insight:** After fixing indent (62), quotes (33), QRL consts (35), and PURE (48), the remaining unique fixtures should drop significantly due to overlap. Estimated that 40-50 fixtures will be fixed by indent+quote normalization alone.

## Architecture Patterns

### Root Module Emission Pipeline

The root module is generated through this pipeline:

1. **Stage 10:** `QwikTransform` traverse rewrites AST (dollar-sign detection, QRL generation)
2. **Stage 11:** `clean_side_effects` removes side-effect-free statements
3. **Stage 12:** Variable migration moves root vars to segments
4. **emit_module:** OXC Codegen produces code string from modified AST
5. **Post-processing:** `strip_const_declarations` removes unused QRL consts
6. **Post-processing:** Tab-to-space conversion needed (currently missing)
7. **Post-processing:** PURE annotation injection (partially implemented)

### Key Source Files

| File | Responsibility | Relevant Categories |
|------|---------------|---------------------|
| `src/transform.rs` | AST transformation, QRL generation | 3, 4, 7, 11, 12, 13 |
| `src/code_move.rs` | Segment module assembly | Segment fixes |
| `src/emit.rs` | OXC Codegen wrapper | 1 (indent), 5 (quotes) |
| `src/hash.rs` | Segment hash computation | 6 |
| `src/words.rs` | QRL naming conventions | 6 |
| `src/entry_strategy.rs` | Inline/Hoist/Segment strategy | 9 |
| `src/lib.rs` | Pipeline orchestration, post-processing | 14 (dead code) |
| `src/clean_side_effects.rs` | Tree-shaking | 14 |

### Anti-Patterns to Avoid
- **Fixing symptoms instead of root causes:** Don't patch individual fixture outputs; fix the underlying generation logic
- **Regex-based string manipulation for structural changes:** Use proper AST modification or at minimum structured string assembly
- **Fixing categories in isolation without regression testing:** Each fix must run full test suite

## Common Pitfalls

### Pitfall 1: Quote Normalization vs Exact Match
**What goes wrong:** Changing quote handling in Codegen might affect segment output, not just root module
**Why it happens:** Codegen options are global -- changing indent or quote style affects ALL output
**How to avoid:** Apply post-processing only to root module output, or use per-module Codegen options
**Warning signs:** Segment snapshot tests breaking after indent/quote fixes

### Pitfall 2: Hash Instability
**What goes wrong:** Fixing one hash computation issue causes different hashes everywhere, breaking currently-passing fixtures
**Why it happens:** Hash is computed from segment content -- any content change cascades to hash
**How to avoid:** Understand exactly what goes into the hash before changing anything; run full parity report after each change
**Warning signs:** Previously-passing fixtures starting to fail

### Pitfall 3: Inline Strategy Rework Scope Creep
**What goes wrong:** The Inline strategy format difference is fundamental, not cosmetic. Attempting incremental fixes leads to half-working state
**Why it happens:** SWC's Inline format (`_noopQrl` + `.s()` + `.w()`) is architecturally different from what OXC currently emits
**How to avoid:** Treat Inline strategy as a discrete work item; implement the full SWC Inline format pattern rather than patching
**Warning signs:** Inline fixtures partially matching but with subtle method chain differences

### Pitfall 4: PURE Annotation Placement
**What goes wrong:** `/*#__PURE__*/` must appear immediately before the call expression, not on a separate line
**Why it happens:** OXC Codegen has specific rules about comment placement
**How to avoid:** Insert PURE annotations via string-level injection in the emit pipeline, matching the `/* @__PURE__ */` -> `/*#__PURE__*/` replacement pattern already in code_move.rs
**Warning signs:** Tree-shaking tests failing because PURE annotations are in wrong position

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Rust built-in `#[test]` + custom snapshot harness |
| Config file | `crates/qwik-optimizer-oxc/tests/snapshot_tests.rs` |
| Quick run command | `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture` |
| Full suite command | `cargo test -p qwik-optimizer-oxc` |

### Phase Requirements to Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| ACC-01 | 201/201 full match | integration | `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture` | Yes |

### Sampling Rate
- **Per task commit:** `cargo test -p qwik-optimizer-oxc` (full suite, ~0.35s)
- **Per wave merge:** `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture`
- **Phase gate:** Parity report shows `Full match: 201/201`

### Wave 0 Gaps
None -- existing test infrastructure covers all phase requirements.

## Sources

### Primary (HIGH confidence)
- Parity report output: live test run against all 201 fixtures (2026-04-06)
- Automated categorization of all 121 root mismatches via temporary analysis test
- Manual diff inspection of 18 representative fixtures across all categories
- SWC expected snapshots: `crates/qwik-optimizer-oxc/tests/swc_expected/*.snap`

### Secondary (MEDIUM confidence)
- Source code analysis of `transform.rs`, `code_move.rs`, `emit.rs`, `hash.rs`, `entry_strategy.rs`
- `fixtures.json` configuration analysis (159 Segment, 22 Inline, 13 Hoist, 4 Single, 3 Smart strategies)

## Metadata

**Confidence breakdown:**
- Root cause categorization: HIGH -- based on automated analysis of all 121 mismatches
- Fix strategies: MEDIUM -- strategies are based on pattern analysis but implementation complexity varies
- Impact estimates: MEDIUM -- overlap effects make precise counting difficult
- Segment mismatch analysis: LOW -- only surface-level investigation; deeper analysis needed during planning

**Research date:** 2026-04-06
**Valid until:** 2026-04-20 (stable -- no external dependencies changing)
