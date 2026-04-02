---
phase: 04-public-api-bindings-cross-cutting-specification
plan: 01
subsystem: public-api-types
tags: [spec, api, types, diagnostics, serde]
dependency_graph:
  requires: [phase-01, phase-02, phase-03]
  provides: [SPEC-18, SPEC-19, SPEC-20]
  affects: [04-02, 04-03]
tech_stack:
  added: []
  patterns: [rust-struct-doc-comments, json-wire-format-examples, serde-annotation-documentation]
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - "D-31 applied: All types documented as Rust struct definitions with doc comments and JSON wire format examples"
  - "SegmentKind: documented all 3 SWC variants (Function, EventHandler, JSXProp) despite TypeScript contract only exposing 2"
  - "EntryStrategy: documented tagged object serialization format matching TypeScript types.ts contract, not SWC bare string form"
  - "Column indexing asymmetry: documented start_col as 1-indexed and end_col as 0-indexed with SWC source explanation"
metrics:
  duration: 4m
  completed: 2026-04-02T00:34:00Z
  tasks: 2
  files: 1
---

# Phase 04 Plan 01: Public API Types Summary

Complete public API type definitions for the Qwik optimizer spec -- TransformModulesOptions (18 fields), all input/output/enum/diagnostic types with Rust struct definitions, doc comments, serde annotations, and JSON wire format examples per D-31.

## What Was Done

### Task 1: Input Types and Enums (94d48c3)
Added the `## Public API Types` section header and 6 subsections:
- **TransformModulesOptions**: 18-field struct with types, defaults, doc comments; JSON wire format example
- **TransformModuleInput**: 3-field input struct (path, dev_path, code)
- **EntryStrategy**: 7-variant enum with `#[serde(tag = "type")]` tagged object serialization; EntryPolicy mapping table
- **EmitMode**: 5 variants (Prod, Dev, Lib, Test, Hmr) with note about Test not in TypeScript contract
- **MinifyMode**: 2 variants (Simplify, None)
- **SegmentKind**: 3 variants (Function, EventHandler, JSXProp) with classification rules and SmartStrategy interaction

### Task 2: Output Types and Diagnostics (bc57561)
Added 7 subsections completing the Public API Types section:
- **TransformOutput**: 4-field return type with merging behavior documentation
- **TransformModule**: 6 fields, `order` documented with `#[serde(skip_serializing)]` annotation
- **SegmentAnalysis**: All 15 fields with serialization notes (loc as array, skip_serializing_if for optional fields)
- **Diagnostic**: 7 fields with complete C02 error JSON example
- **DiagnosticCategory**: 3 variants with SourceError dual-behavior semantics (error in project, warning in node_modules)
- **SourceLocation**: 6 fields with column indexing asymmetry fully documented from SWC source comments
- **Error Codes**: 3 sparse codes (C02, C03, C05) with enum discriminants, trigger conditions, and message patterns

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 2 - Accuracy] SegmentKind variant count corrected**
- **Found during:** Task 1
- **Issue:** Plan stated "SWC has 2 variants (Function, EventHandler)" with "Jack's JSXProp addition as an OXC extension." However, the SWC source (`transform.rs`) contains all 3 variants including JSXProp.
- **Fix:** Documented all 3 variants as present in SWC, noted that TypeScript contract only exposes 2 (`function`, `eventHandler`).
- **Files modified:** specification/qwik-optimizer-spec.md

## Verification

- `## Public API Types` heading count: 1
- Total `### ` subsections: 75 (62 baseline + 13 new)
- All required type names present: TransformModulesOptions, TransformOutput, TransformModule, SegmentAnalysis, SegmentKind, Diagnostic, DiagnosticCategory, SourceLocation
- SPEC-18 satisfied: TransformModulesOptions with all 18 config fields, types, defaults
- SPEC-19 satisfied: TransformOutput, TransformModule, SegmentAnalysis, SegmentKind fully documented
- SPEC-20 satisfied: Diagnostic with all 7 fields, 3 DiagnosticCategory variants, 3 error codes

## Known Stubs

None -- all types are fully documented with complete field definitions and wire format examples.
