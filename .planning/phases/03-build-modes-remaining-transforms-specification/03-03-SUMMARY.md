---
phase: 03-build-modes-remaining-transforms-specification
plan: 03
subsystem: specification
tags: [entry-strategies, emit-modes, SPEC-15, SPEC-16, mode-conv-table, smart-strategy]
dependency_graph:
  requires: [03-01-PLAN, 03-02-PLAN]
  provides: [Entry Strategies spec section, Emit Modes spec section with Mode x CONV table]
  affects: [03-04-PLAN]
tech_stack:
  added: []
  patterns: [cross-reference-table, per-implementation-subsections, conditional-grouping-documentation]
key_files:
  created: []
  modified:
    - specification/qwik-optimizer-spec.md
decisions:
  - "Documented all 5 EntryPolicy implementations as separate subsections per D-25 with full Rust source excerpts for SmartStrategy and PerComponentStrategy"
  - "Mode x CONV cross-reference table placed first in Emit Modes section per D-26 for scannability"
  - "Inline/Hoist distinction documented as shared InlineStrategy with fundamentally different output patterns (inlinedQrl vs _noopQrl + .s())"
  - "Lib mode documented with explicit skip list format highlighting 7 skipped/modified behaviors"
metrics:
  duration: "4m"
  completed: "2026-04-01T23:03:13Z"
  tasks_completed: 2
  tasks_total: 2
  files_modified: 1
  lines_added: 469
---

# Phase 03 Plan 03: Entry Strategies and Emit Modes Summary

Entry Strategies (SPEC-15) and Emit Modes (SPEC-16) spec sections with 7 strategy enum variants mapped to 5 EntryPolicy implementations, Mode x CONV cross-reference table covering all 14 CONVs across 5 modes, and per-mode behavioral descriptions including Lib mode's extensive skip list.

## What Was Done

### Task 1: Read SWC source files and snapshots
Read all source files (entry_strategy.rs, lib.rs, parse.rs, transform.rs) and relevant snapshots (example_inlined_entry_strategy, example_strip_server_code, hmr, example_lib_mode, example_dev_mode). Extracted 5 EntryPolicy implementations with exact grouping logic, verified SmartStrategy's three conditional rules against source, and cross-referenced Mode x CONV table from 03-RESEARCH.md against parse.rs conditionals for completeness.

### Task 2: Write Entry Strategies and Emit Modes spec sections
Appended ~469 lines to qwik-optimizer-spec.md after Stage 6, organized as:

**Entry Strategies section:**
- EntryPolicy trait definition with `get_entry_for_sym` signature
- InlineStrategy (Inline/Hoist) with clear distinction: same grouping, different output patterns
- SingleStrategy, PerSegmentStrategy (Hook/Segment), PerComponentStrategy subsections
- SmartStrategy with full Rust source excerpt and three named grouping rules
- 7-row summary table (Strategy Enum | EntryPolicy Impl | Grouping Behavior | Output Pattern)
- Full Hoist example from example_inlined_entry_strategy snapshot
- Per-segment example from example_strip_server_code snapshot

**Emit Modes section:**
- Mode x CONV cross-reference table: 14 CONV rows + 3 "Other" rows x 5 mode columns
- Per-mode descriptions: Prod, Dev, Lib, Test, Hmr
- Lib mode skip list covering 7 skipped/modified behaviors
- HMR example from hmr snapshot showing _useHmr injection contrast
- is_dev derivation documented

## Commits

| Task | Commit | Description |
|------|--------|-------------|
| 1-2 | 022efc6 | feat(03-03): add Entry Strategies and Emit Modes spec sections |

## Verification

- `grep -c "## Entry Strategies"` returns 1
- `grep -c "## Emit Modes"` returns 1
- Mode x CONV table has 14 CONV rows and 5 mode columns
- Entry Strategy summary table has 7 rows (one per enum variant)
- Inline/Hoist distinction explicitly documented with InlineStrategy shared, output patterns different
- SmartStrategy three conditional rules documented with Rust source excerpt

## Deviations from Plan

None -- plan executed exactly as written.

## Known Stubs

None -- all sections contain complete content derived from source analysis.

## Self-Check: PASSED

- FOUND: specification/qwik-optimizer-spec.md
- FOUND: 03-03-SUMMARY.md
- FOUND: commit 022efc6
