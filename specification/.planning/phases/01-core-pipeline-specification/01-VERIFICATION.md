---
phase: 01-core-pipeline-specification
verified: 2026-04-01T00:00:00Z
status: passed
score: 9/11 requirements verified (SPEC-01, SPEC-02 missing)
re_verification: false
gaps:
  - truth: "The spec document describes dollar detection rules such that a reader can determine whether any given function call triggers QRL extraction without consulting SWC source"
    status: failed
    reason: "Plan 02 was never executed. No '### Dollar Detection' section exists. Line 251 contains only a placeholder note: 'Dollar Detection and QRL Wrapping sections are added by Plan 02.' The ROADMAP success criterion 1 for Phase 1 is unmet."
    artifacts:
      - path: "specification/qwik-optimizer-spec.md"
        issue: "Missing ### Dollar Detection (CONV-01) section — no behavioral rules for marker function identification, QRL_SUFFIX detection, convert_qrl_word callee conversion, or local marker detection"
    missing:
      - "### Dollar Detection (CONV-01) section with: imported marker detection (ends_with '$' from @qwik.dev/core), local marker detection (exported $-suffixed functions), marker_functions HashMap, convert_qrl_word callee conversion, special cases (sync$, component$, bare $), non-marker exclusion rule, 3 input/output examples, SWC source references to transform.rs:179-202"

  - truth: "The spec document describes QRL wrapping — replacement of $-suffixed calls with Qrl counterparts, qrl()/inlinedQrl() reference generation, dev mode variants"
    status: failed
    reason: "Plan 02 was never executed. No '### QRL Wrapping' section exists. Line 938 contains a dangling cross-reference 'See QRL Wrapping section for create_qrl details' pointing to a nonexistent section. SPEC-02 is unmet."
    artifacts:
      - path: "specification/qwik-optimizer-spec.md"
        issue: "Missing ### QRL Wrapping (CONV-02) section — no behavioral rules for three QRL creation paths, dev mode variants (qrlDEV, inlinedQrlDEV), captures emission pattern, PURE annotation rule, or symbol name/import path construction in the wrapping context"
    missing:
      - "### QRL Wrapping (CONV-02) section with: create_qrl() for Segment/Hook/Single/Component/Smart strategies, create_inline_qrl() for Inline/Hoist strategies, create_noop_qrl() for stripped callbacks, dev mode qrlDEV/inlinedQrlDEV variants with source location arg, captures emission (scoped_idents array + .w() call), PURE annotation rule (componentQrl only), 3 input/output examples, SWC source references to transform.rs:1888-2062"
human_verification: []
---

# Phase 1: Core Pipeline Specification — Verification Report

**Phase Goal:** The spec document contains complete behavioral descriptions of the core QRL extraction pipeline — the transformations that every other feature depends on — plus the capture analysis taxonomy that is the single highest-risk area of the entire project
**Verified:** 2026-04-01
**Status:** GAPS FOUND
**Re-verification:** No — initial verification

---

## Goal Achievement

### Observable Truths (from ROADMAP.md Success Criteria)

| # | Truth | Status | Evidence |
|---|-------|--------|----------|
| 1 | The spec document describes dollar detection rules such that a reader can determine whether any given function call triggers QRL extraction without consulting SWC source | ✗ FAILED | No `### Dollar Detection` section exists. Line 251 is a placeholder. Plan 02 was never executed (no 01-02-SUMMARY.md exists, no commits for Dollar Detection). |
| 2 | The spec document contains the complete 8-category capture analysis taxonomy with edge case examples for each category, including the self-import reclassification behavior | ✓ VERIFIED | `### Capture Analysis (CONV-03)` at line 253. 8-category table, Mermaid decision tree, self-import reclassification subsection, and all 16 CAPTURE-EDGE cases (01-16) present with code examples. Mentions 293 deviations / 46 self-import fixes. |
| 3 | The spec document describes segment extraction behavior — filename generation, hash computation, nested segment relationships, and variable migration — with input/output examples | ✓ VERIFIED | `### Segment Extraction (CONV-05)` at line 920. Covers create_segment, new_module 8-step pipeline, segment_stack, parent-child nesting, SegmentAnalysis metadata. Hash/path cross-referenced to Infrastructure sections. `## Variable Migration` at line 1456 with 5-step pipeline. |
| 4 | The spec document describes import rewriting rules (consumed import stripping, synthetic import addition, per-segment resolution) with before/after examples | ✓ VERIFIED | `### Import Rewriting (CONV-12)` at line 1219. Four mechanisms documented: RenameTransform legacy rename, consumed stripping via DCE, ensure_core_import synthetic addition, resolve_import_for_id per-segment resolution. 3 before/after examples. |
| 5 | The spec document describes source map generation contracts for both root and segment modules | ✓ VERIFIED | `## Infrastructure: Source Map Generation` at line 2012. Covers emit_source_code, JsWriter, V3 format, root module (high fidelity) vs segment module (mixed fidelity), sourceRoot, sourcesContent, TransformModule.map field. 2 examples. |

**Score:** 4/5 success criteria satisfied

---

## Required Artifacts

| Artifact | Expected | Status | Details |
|----------|----------|--------|---------|
| `specification/qwik-optimizer-spec.md` | Complete spec document with all Phase 1 sections | ⚠️ PARTIAL | File exists (2173 lines, 41 SWC source references). Missing Dollar Detection and QRL Wrapping sections. All other planned sections are substantive and wired. |

---

## Key Link Verification

| From | To | Via | Status | Details |
|------|----|-----|--------|---------|
| Pipeline Overview diagram | Stage groupings (6 stages) | Mermaid flowchart | ✓ WIRED | Mermaid diagram at line 17-69 shows 20-step pipeline in 6 subgraphs. All stages labeled. |
| Dollar Detection section | QRL Wrapping section | Marker identification triggers QRL replacement | ✗ NOT_WIRED | Neither section exists — Plan 02 unexecuted. Placeholder at line 251. |
| Capture Analysis section | GlobalCollect section | is_global() determines capture classification | ✓ WIRED | Line 311-317 references global_collect.is_global(). Line 315 explicitly describes the global classification logic. |
| Segment Extraction section | Capture Analysis section | scoped_idents feeds segment construction | ✓ WIRED | Line 938 cross-references scoped_idents. `_captures[N]` pattern documented at lines 932-960. |
| Segment Extraction section | Hash Generation section | Canonical filenames use hashes | ✓ WIRED | Line 932 references get_canonical_filename and links to Hash Generation section by anchor. |
| Import Rewriting section | GlobalCollect section | Synthetic imports registered in GlobalCollect | ✓ WIRED | Line 1299 references GlobalCollect.exports in per-segment resolution. ensure_core_import documented with synthetic flag. |
| Variable Migration section | Capture Analysis section | scoped_idents/local_idents feed usage analysis | ✓ WIRED | Line 1495-1521 references local_idents and scoped_idents from Capture Analysis. |
| Variable Migration section | Segment Extraction section | migrated_root_vars consumed by new_module | ✓ WIRED | Line 1544 explicitly states migrated_root_vars is consumed by new_module() in code_move.rs. |
| Segment Extraction (line 938) | QRL Wrapping section | "See QRL Wrapping section for create_qrl details" | ✗ DANGLING | Cross-reference at line 938 points to a section that does not exist. |

---

## Requirements Coverage

| Requirement | Source Plan | Description | Status | Evidence |
|-------------|------------|-------------|--------|----------|
| SPEC-01 | 01-02-PLAN.md | Dollar Detection (CONV-01) | ✗ BLOCKED | Plan 02 never executed. No 01-02-SUMMARY.md. No commits for this content. Section absent from spec. |
| SPEC-02 | 01-02-PLAN.md | QRL Wrapping (CONV-02) | ✗ BLOCKED | Same as SPEC-01 — Plan 02 never executed. Dangling reference at line 938. |
| SPEC-03 | 01-03-PLAN.md | Capture Analysis (CONV-03) | ✓ SATISFIED | Complete section at line 253: algorithm (4 steps), 8-category taxonomy, Mermaid decision tree, 3 examples, self-import subsection. |
| SPEC-05 | 01-04-PLAN.md | Segment Extraction (CONV-05) | ✓ SATISFIED | Complete section at line 920: create_segment, new_module 8-step pipeline, Segment struct fields, nested segment relationships, 3 examples with SegmentAnalysis metadata. |
| SPEC-12 | 01-04-PLAN.md | Import Rewriting (CONV-12) | ✓ SATISFIED | Complete section at line 1219: 4 mechanisms, resolve_import_for_id, RenameTransform, ensure_core_import, collision handling, 3 examples. |
| SPEC-21 | 01-01-PLAN.md | GlobalCollect | ✓ SATISFIED | Complete section at line 95: IndexMap data structures, 5 behavioral rules, 6 key methods (is_global documented), 2 examples, SWC source reference to collector.rs:56-528. |
| SPEC-22 | 01-05-PLAN.md | Variable Migration | ✓ SATISFIED | Separate `##` section at line 1456: analyze_root_dependencies, build_root_var_usage_map, build_main_module_usage_set, find_migratable_vars, post-migration cleanup. All 6 migration conditions. 2 examples. |
| SPEC-23 | 01-01-PLAN.md | Hash Generation | ✓ SATISFIED | `## Infrastructure: Hash Generation` at line 1667: DefaultHasher, URL_SAFE_NO_PAD, to_le_bytes(), display name construction, escape_sym, deduplication, Dev vs Prod symbol name format. 2 examples. |
| SPEC-24 | 01-01-PLAN.md | Path Resolution | ✓ SATISFIED | `## Infrastructure: Path Resolution` at line 1862: parse_path(), get_canonical_filename(), explicit_extensions, extension mapping table. 2 examples. |
| SPEC-25 | 01-05-PLAN.md | Source Map Generation | ✓ SATISFIED | `## Infrastructure: Source Map Generation` at line 2012: emit_source_code, JsWriter, V3 format, root vs segment fidelity, sourceRoot, sourcesContent, TransformModule.map. 2 examples. |
| SPEC-30 | 01-03-PLAN.md | Capture analysis taxonomy with edge cases for all 8 categories | ✓ SATISFIED | All 16 CAPTURE-EDGE cases (01-16) at lines 563-916. Each has ID, category reference, input code, expected behavior, rationale. 293/46 deviation counts present. |

**Orphaned requirements check:** REQUIREMENTS.md Traceability maps SPEC-01, SPEC-02, SPEC-03, SPEC-05, SPEC-12, SPEC-21, SPEC-22, SPEC-23, SPEC-24, SPEC-25, SPEC-30 to Phase 1 (11 total). All 11 are claimed by plans in this phase. SPEC-01 and SPEC-02 are claimed by 01-02-PLAN.md but that plan was never executed — they are not orphaned (claimed) but their deliverables are absent (blocked).

---

## Anti-Patterns Found

| File | Line | Pattern | Severity | Impact |
|------|------|---------|----------|--------|
| `specification/qwik-optimizer-spec.md` | 251 | Placeholder note: "Dollar Detection and QRL Wrapping sections are added by Plan 02." | Blocker | The two sections are absent. Plan 02 was never executed. |
| `specification/qwik-optimizer-spec.md` | 938 | Dangling cross-reference: "See QRL Wrapping section for `create_qrl` details" | Blocker | The referenced section does not exist. The Segment Extraction section's QRL creation step is incompletely specified as a result. |
| `specification/qwik-optimizer-spec.md` | 89 | Phase Coverage text claims "Dollar Detection" is specified in Phase 1 | Warning | The text at line 89 states Phase 1 specifies Dollar Detection and QRL Wrapping, but neither section exists. The Phase Coverage description is inaccurate. |

---

## Behavioral Spot-Checks

Step 7b: SKIPPED — this phase produces a specification document (markdown), not runnable code. No entry points exist to test.

---

## Human Verification Required

None. All gaps are structural (missing sections) and verifiable programmatically.

---

## Gaps Summary

**Root cause:** Plan 02 (Dollar Detection SPEC-01 + QRL Wrapping SPEC-02) was skipped during phase execution. Plans 01, 03, 04, and 05 were all executed successfully. The ROADMAP shows "1/5 plans executed" as the phase completion state — this is slightly misleading since Plans 01, 03, 04, and 05 are complete (4/5), but Plan 02 remains unexecuted.

The missing content represents the entry point of the entire core transform pipeline. Dollar detection is what gates QRL extraction — everything else (capture analysis, QRL wrapping, segment extraction) depends on it. The spec references these concepts throughout but provides no behavioral specification for how marker functions are identified or how the QRL replacement calls are constructed.

**Impact on goal:** The phase goal states "complete behavioral descriptions of the core QRL extraction pipeline." With Dollar Detection and QRL Wrapping absent, the pipeline description has a structural gap: a reader cannot determine whether any function call triggers QRL extraction (ROADMAP success criterion 1 is unmet), and the Segment Extraction section contains a dangling reference ("See QRL Wrapping section for create_qrl details") pointing to nonexistent content.

**What is working well (9 of 11 requirements satisfied):**

- Capture analysis (SPEC-03, SPEC-30): The most complex section is complete and precise, with the full 8-category taxonomy, Mermaid decision tree, self-import reclassification documented as a first-class concern, and all 16 named edge cases.
- Supporting infrastructure (SPEC-21, SPEC-23, SPEC-24): GlobalCollect, Hash Generation, and Path Resolution sections are complete with behavioral rules and examples.
- Pipeline output (SPEC-05, SPEC-12): Segment Extraction and Import Rewriting are fully specified.
- Post-transform (SPEC-22, SPEC-25): Variable Migration and Source Map Generation are complete.

---

_Verified: 2026-04-01_
_Verifier: Claude (gsd-verifier)_
