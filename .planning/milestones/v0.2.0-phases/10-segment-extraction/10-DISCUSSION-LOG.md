# Phase 10: Segment Extraction - Discussion Log (Assumptions Mode)

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions captured in CONTEXT.md -- this log preserves the analysis.

**Date:** 2026-04-03
**Phase:** 10-Segment Extraction
**Mode:** assumptions
**Areas analyzed:** JSX Attribute Segment Extraction, Integration Point, Event Handler Attribute Renaming, Loop and Nested Context Handling

## Assumptions Presented

### JSX Attribute Segment Extraction Path
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Primary cause of segment count mismatches is missing JSX attribute extraction path -- OXC only extracts from explicit `$()` calls, not `$`-suffixed JSX attribute names | Confident | `transform.rs:detect_dollar_call`, SWC `handle_jsx_value`, parity report showing act=1 vs exp=2+ |

### Integration Point for JSX Attribute Extraction
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Extraction must happen during JSX transform phase (near `classify_props` or via `enter_jsx_attribute`), not after JSX lowering | Likely | SWC integrates into `handle_jsx_props_obj` pipeline; `jsx_transform.rs:classify_props` is equivalent; element tag context needed for display_name |

### Event Handler Attribute Renaming
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| `$`-suffixed HTML element attributes renamed to `q-e:` prefix; custom component attributes strip `$` only | Likely | SWC expected snaps show `"q-e:click"` format; SWC `jsx_event_to_html_attribute` function |

### Loop and Nested Context Handling
| Assumption | Confidence | Evidence |
|------------|-----------|----------|
| Loop failures are downstream of missing JSX extraction, not separate traversal bugs | Likely | All loop fixtures show act=1 (component$) vs exp=2+ (component$ + handlers); OXC Traverse already visits loop bodies |

## Corrections Made

No corrections -- all assumptions confirmed (auto mode).

## Auto-Resolved

- Integration Point: auto-confirmed Likely assumption (classify_props integration recommended)
- Event Handler Renaming: auto-confirmed Likely assumption (q-e: prefix pattern from SWC)
- Loop Context: auto-confirmed Likely assumption (downstream of JSX extraction)
