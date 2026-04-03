---
phase: 02-jsx-props-signal-specification
plan: 02
subsystem: JSX Transform (CONV-06)
tags: [jsx, transform, props, signals, specification]
dependency_graph:
  requires: [01-04-PLAN (Import Rewriting section placement)]
  provides: [JSX Transform CONV-06 section, prop classification rules, event name conversion, bind expansion, flag computation, spread props handling]
  affects: [02-03-PLAN (Signal Optimization references prop classification)]
tech_stack:
  added: []
  patterns: [_jsxSorted/_jsxSplit branch point, static/dynamic prop classification, event name conversion, bind:value/bind:checked expansion, iteration variable lifting]
key_files:
  created: []
  modified: [specification/qwik-optimizer-spec.md]
decisions:
  - D-17 applied: JSX section broken into 8 subsections per plan
  - D-18 applied: _jsxSorted/_jsxSplit documented as one mechanism with branching condition
  - D-23 applied: 8 examples across subsections adapted to complexity
  - D-24 applied: inline examples plus See also snapshot lists
metrics:
  duration: ~5m
  completed: 2026-04-01
---

# Phase 2 Plan 02: JSX Transform (CONV-06) Specification Summary

Complete JSX Transform specification section with 8 subsections covering the full _jsxSorted/_jsxSplit pipeline, prop classification, special attributes, and all edge cases verified against SWC snapshots.

## Tasks Completed

### Task 1: JSX Transform branch point, element detection, prop classification, and special attributes
- **Commit:** 3a7f88f
- **What:** Inserted `### JSX Transform (CONV-06)` section after Import Rewriting (CONV-12) under Stage 4: Core Transform. Wrote 4 core subsections: _jsxSorted vs _jsxSplit Branch Point (D-18), Element Transformation, Prop Classification (Static vs Dynamic), Special Attributes with Event Name Conversion and bind:value/bind:checked Expansion sub-subsections. Added 5 inline examples with snapshot references.
- **Key files:** specification/qwik-optimizer-spec.md (+425 lines)

### Task 2: Children, Key Generation, Flag Computation, Spread Props, Iteration Variable Lifting
- **Commit:** 4152d87
- **What:** Added 4 remaining subsections: Children Handling, Key Generation (root_jsx_mode, auto-key format), Flag Computation (bitmask table), Spread Props Handling (_getVarProps/_getConstProps), plus Iteration Variable Lifting (q:p/q:ps) sub-subsection. Added 3 additional examples with snapshot references.
- **Key files:** specification/qwik-optimizer-spec.md (+243 lines)

## Verification Results

All acceptance criteria passed:
- 1 match for `### JSX Transform (CONV-06)` heading
- 8 main subsections present (Branch Point, Element Transformation, Prop Classification, Special Attributes, Children Handling, Key Generation, Flag Computation, Spread Props Handling)
- 30+ references to `_jsxSorted`, 17+ to `_jsxSplit`
- 33+ references to `q-e:` event conversion prefix
- 13+ references to `bind:value`/`bind:checked`
- 8 inline examples with snapshot source attribution
- 4 "See also" snapshot lists with 20+ additional snapshot references

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None -- all content is fully specified with behavioral rules, examples, and SWC source references.
