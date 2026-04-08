---
phase: 11-root-module-code-generation
verified: 2026-04-06T15:25:06Z
status: gaps_found
score: 3/5 must-haves verified
gaps:
  - truth: "Root module match count reaches 160+ out of 201"
    status: failed
    reason: "Actual root module match is 80/201 (40%). Plan 04 acceptance criterion of 160+ was not met. The const stripping pass (strip_unreferenced_wrapper_consts_text) introduces a correctness bug visible in example_1: the q_* const binding is stripped but the reference to it in the module body is left dangling, creating invalid output. Import ordering within the same source module also still differs from SWC (OXC inserts by category, SWC inserts alphabetically within category)."
    artifacts:
      - path: "crates/qwik-optimizer-oxc/src/lib.rs"
        issue: "strip_unreferenced_wrapper_consts_text: const q_renderHeader2_component_Ay6ibkfFYsw stripped from example_1 but component(q_renderHeader2_component_Ay6ibkfFYsw) reference retained — broken output. Root cause: q_* consts ARE included as candidates (is_pure=true) but are referenced by a line that may itself be a candidate, so the reference is missed."
      - path: "crates/qwik-optimizer-oxc/src/transform.rs"
        issue: "Import ordering: within the same source module, OXC inserts wrapper imports (componentQrl before globalActionQrl) but SWC produces alphabetical order. Affects 'should_work' and similar fixtures with multiple wrapper imports."
    missing:
      - "Fix strip_unreferenced_wrapper_consts_text to exclude const q_* lines from the candidate set (SWC DCE strips non-hoisted consts, NOT the q_* QRL references themselves)"
      - "Fix wrapper import ordering to match SWC alphabetical ordering or first-use ordering"

  - truth: "Variable declarations unreferenced in root module are stripped or simplified (without introducing invalid references)"
    status: failed
    reason: "The text-level const stripping pass strips const bindings but leaves dangling references. In example_1: const q_renderHeader2_component_Ay6ibkfFYsw is removed but component(q_renderHeader2_component_Ay6ibkfFYsw) is kept, producing invalid JS. The referenced-set collection only looks at non-candidate lines — if the reference-bearing line (e.g., a bare call expression) is itself a candidate, the reference is missed."
    artifacts:
      - path: "crates/qwik-optimizer-oxc/src/lib.rs"
        issue: "strip_unreferenced_wrapper_consts_text at line 814 — const q_* lines (which have PURE annotations) are incorrectly included as stripping candidates. These are the hoisted QRL declarations that MUST be kept."
    missing:
      - "Exclude lines matching `const q_` from the candidate set in strip_unreferenced_wrapper_consts_text (q_* lines are hoisted QRL refs, not user const declarations)"
      - "Add a check: before stripping a const, verify the referenced set includes ALL lines (including other candidate lines that reference it)"
---

# Phase 11: Root Module Code Generation Verification Report

**Phase Goal:** The root module output for every fixture matches SWC in import ordering, variable declarations, export structure, QRL references, and comment separators
**Verified:** 2026-04-06T15:25:06Z
**Status:** gaps_found
**Re-verification:** No — initial verification

## Goal Achievement

### Observable Truths

| # | Truth | Status | Evidence |
|---|-------|--------|---------|
| 1 | QRL wrapper imports use original marker function source module | ✓ VERIFIED | `find_wrapper_source()` at transform.rs:586, `marker_fn_sources` field at :384, populated at :507. `should_work` fixture shows `globalActionQrl` from `@qwik.dev/router`. |
| 2 | Synthetic imports from collect.synthetic emitted as import declarations | ✓ VERIFIED | transform.rs:2717 emits synthetic imports in exit_program loop over `collect.synthetic`. |
| 3 | Fixpoint dead code elimination removes unused _qrl_/i_ vars AND imports | ✓ VERIFIED | `remove_unused_qrl_declarations` at lib.rs:588 — standalone function with fixpoint loop, transitive closure propagation, runs after variable migration. 36→76 parity jump confirms. |
| 4 | Comment separators between import block and QRL const block match SWC | ✓ VERIFIED | `insert_separator_comments` at emit.rs:81 — detects import→non-import and hoisted→non-hoisted transitions, inserts `//` lines. Used in both code paths of `emit_module`. |
| 5 | Root module match count reaches 160+ out of 201 | ✗ FAILED | Actual: 80/201 (40%). Plan 04 acceptance criterion explicitly states 160+. Const stripping pass introduces broken output (dangling references in example_1). Import ordering within same source module still differs. 122 fixtures remain mismatched. |

**Score:** 3/5 truths verified

### Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `crates/qwik-optimizer-oxc/src/transform.rs` | marker_fn_sources field, find_wrapper_source helper, stack_ctxt push for marker names | ✓ VERIFIED | marker_fn_sources at :384, find_wrapper_source at :592, stack_ctxt.push(escape_dollar) at :1840, pushed_ctx_name tracking at :291 |
| `crates/qwik-optimizer-oxc/src/lib.rs` | remove_unused_qrl_declarations fixpoint, strip_unreferenced_wrapper_consts_text | ✓ WIRED (with bug) | remove_unused_qrl_declarations at :588 — correct. strip_unreferenced_wrapper_consts_text at :814 — present but includes q_* lines as stripping candidates, causing broken output. |
| `crates/qwik-optimizer-oxc/src/dependency_analysis.rs` | IdentRefCollector with visit_export_named_declaration | ✓ VERIFIED | IdentRefCollector at :489, visit_export_named_declaration at :498. |
| `crates/qwik-optimizer-oxc/src/emit.rs` | insert_separator_comments | ✓ VERIFIED | insert_separator_comments at :81 — full implementation detecting import/hoisted-const transitions. |

### Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| transform.rs::QwikTransform::new | marker_fn_sources HashMap | collect.imports source field | ✓ WIRED | :501-507: `marker_fn_sources.insert(import.specifier, import.source)` inside marker function detection loop |
| transform.rs::exit_program | collect.synthetic | synthetic import emission loop | ✓ WIRED | :2717 loop over `collect.synthetic` emitting import declarations |
| lib.rs::transform_code | remove_unused_qrl_declarations | post-migration standalone call | ✓ WIRED | :165-168: called after apply_variable_migration with non-Lib mode guard |
| lib.rs::transform_code | strip_unreferenced_wrapper_consts_text | post-emit text pass | ✓ WIRED (buggy) | :186-190: called on emit_result.code. Present and called, but strips q_* hoisted const bindings it should preserve. |
| lib.rs::apply_variable_migration Step 9→ | IdentRefCollector | fixpoint reference collection | ✓ WIRED | :647, :689: IdentRefCollector used in remove_unused_qrl_declarations |
| transform.rs::enter_call_expression | stack_ctxt.push(escape_dollar) | pushed_ctx_name conditional | ✓ WIRED | :1838-1850: conditional push for marker names, skips bare "$" and "sync$" |

### Data-Flow Trace (Level 4)

| Artifact | Data Variable | Source | Produces Real Data | Status |
|----------|--------------|--------|-------------------|--------|
| emit.rs::insert_separator_comments | code (root module string) | emit_module codegen output | Yes — processes actual AST output | ✓ FLOWING |
| lib.rs::remove_unused_qrl_declarations | program.body | transformed OXC AST | Yes — iterates real AST statements | ✓ FLOWING |
| lib.rs::strip_unreferenced_wrapper_consts_text | code string | codegen output from emit_module | Yes — processes real codegen output, but has inclusion bug for q_* | ⚠️ HOLLOW (bug) |

### Behavioral Spot-Checks

| Behavior | Command | Result | Status |
|----------|---------|--------|--------|
| All tests pass (264 lib + 223 snapshot) | `cargo test -p qwik-optimizer-oxc` | 264 passed; 0 failed / 223 passed; 0 failed | ✓ PASS |
| Parity report runs without test failure | `cargo test parity_report` | 1 passed; 0 failed | ✓ PASS |
| Root module match 80/201 | `parity_report` | 80/201 (40%) | ✗ FAIL — Plan 04 required 160+ |
| Segment count not regressed | `parity_report` | 194/201 (was 195/201 before phase) | ⚠️ MINOR REGRESSION — -1 from phase start |
| Diagnostics not regressed | `parity_report` | 192/201 (was 193/201 before phase) | ⚠️ MINOR REGRESSION — -1 from phase start |
| No compilation errors | `cargo build -p qwik-optimizer-oxc` | Compiles with warnings only | ✓ PASS |

### Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|---------|
| ROOT-01 | 11-01 | Root module import statements match SWC ordering and format for all fixtures | ✗ PARTIAL | Import source module tracking works (find_wrapper_source); synthetic imports emitted. BUT within-category ordering differs from SWC (componentQrl before globalActionQrl vs SWC alphabetical). 122 root mismatches remain. |
| ROOT-02 | 11-02, 11-04 | Root module variable declarations and expressions match SWC output | ✗ PARTIAL | remove_unused_qrl_declarations fixpoint works correctly (+40 parity). strip_unreferenced_wrapper_consts_text has a correctness bug that strips q_* hoisted refs, creating dangling references. |
| ROOT-03 | 11-02 | Root module export structure matches SWC output | ✓ PARTIAL | Step 8b in apply_variable_migration removes migrated export specifiers. IdentRefCollector visits export_named_declaration. Export stripping for migrated vars is functional. |
| ROOT-04 | 11-01, 11-03, 11-04 | Root module QRL references and hoisted declarations match SWC format | ✗ PARTIAL | Stack_ctxt push/pop for display_name fixed (+40 indirect via fixpoint). QRL const declarations present. BUT q_* ordering within root module differs from SWC, and const stripping bug affects some fixtures. |
| ROOT-05 | 11-01, 11-04 | Root module comment separators and whitespace structure match SWC output | ✓ VERIFIED | insert_separator_comments in emit.rs fully implemented and called for all emit paths. Correctly inserts `//` at import→QRL-const and QRL-const→body transitions. |

**Orphaned Requirements Check:** No additional ROOT-* requirements appear in REQUIREMENTS.md for Phase 11 beyond the 5 declared. No orphaned requirements found.

### Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `crates/qwik-optimizer-oxc/src/lib.rs` | 844-849 | `strip_unreferenced_wrapper_consts_text` includes `const q_*` lines as stripping candidates due to `is_pure=true` match on `/*#__PURE__*/` annotations | Blocker | Strips hoisted QRL const bindings and leaves dangling variable references — broken JavaScript output in example_1 and likely other fixtures with unreferenced wrapper assignments |
| `crates/qwik-optimizer-oxc/src/types.rs` | 8 | `unused import: std::collections::HashMap` (compile warning) | Info | Non-functional warning |
| `crates/qwik-optimizer-oxc/src/transform.rs` | 1026, 1045, 1269, 1969 | Unused variables with compiler warnings (el_box, frag_box, qrl_wrapper_name, el) | Info | Non-functional warnings |

### Human Verification Required

None — all items verified programmatically.

### Gaps Summary

**Gap 1 — Const stripping bug (blocker):** The `strip_unreferenced_wrapper_consts_text` function in `lib.rs` treats `const q_*` lines as stripping candidates because they contain `/*#__PURE__*/` annotations (triggering the `is_pure` check). These are hoisted QRL constant declarations that must never be stripped. When stripped, their binding names disappear but the code that references them (e.g., `component(q_renderHeader2_...)` in example_1) remains, producing invalid JavaScript. Fix: add `name.starts_with("q_")` exclusion to the candidate filter at line 832-850.

**Gap 2 — Plan 04 target not met (80 vs 160+):** Root module match is 80/201, not the 160+ target. The 11-04 SUMMARY documents this and attributes remaining mismatches to: hash/display_name deduplication (counter `_1` suffix, ~40 fixtures), inline entry strategy (inlinedQrl missing, ~26 fixtures), import ordering (~10 fixtures), dev mode span values (~5), signal optimization (~4), and other structural issues (~31). These are deferred to Phase 13.

**Gap 3 — Minor segment/diagnostics regression:** Segment count match dropped from 195/201 to 194/201 and diagnostics from 193/201 to 192/201. The summaries do not explain these regressions; they may be caused by the const stripping bug affecting segment extraction in affected fixtures.

**Root Cause Grouping:** Gaps 1 and 3 likely share the same root cause — the const stripping pass creating invalid root module output, which may also affect fixture snapshot comparisons used for segment counts and diagnostics.

---

_Verified: 2026-04-06T15:25:06Z_
_Verifier: Claude (gsd-verifier)_
