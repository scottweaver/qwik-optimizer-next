# Phase 12: Diagnostics Parity - Discussion Log (Assumptions Mode)

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in CONTEXT.md -- this log preserves the analysis.

**Date:** 2026-04-06
**Phase:** 12-diagnostics-parity
**Mode:** assumptions
**Areas analyzed:** C02 False Positive Suppression, C05 Missing Implementation, Diagnostic Comparison Granularity, Inline/Hoist Mode Interaction

## Assumptions Presented

### C02 False Positive Suppression
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Most false-positive mismatches caused by missing export-symbol gate in C02 check | Likely | SWC transform.rs:1024 vs OXC transform.rs:905-923; has_export_symbol exists in collector.rs:139 |

### C05 Missing Implementation
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| OXC has zero C05 (MissingQrlImplementation) implementation | Confident | Grep for C05/MissingQrl returns 0 results; SWC emits at transform.rs:4078-4088; Jack includes C05 tests |

### Diagnostic Comparison Granularity
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Parity comparison checks error presence/absence only (boolean), sufficient for Phase 12 | Confident | snapshot_tests.rs:1150-1156 uses boolean check |

### Inline/Hoist Mode Interaction
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Non-default modes (Lib, Hoist) cause false positives because SWC skips segment extraction and thus C02 | Likely | fixtures.json mode configs; SWC checks should_emit_segment before C02 loop |

## Corrections Made

No corrections -- all assumptions confirmed.

## Scope Update

Analysis revealed 9 diagnostic mismatches (not 4 as originally estimated): 8 false positives + 1 false negative (missing C05).
