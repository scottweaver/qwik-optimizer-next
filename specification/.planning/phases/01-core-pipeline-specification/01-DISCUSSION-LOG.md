# Phase 1: Core Pipeline Specification - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 01-core-pipeline-specification
**Areas discussed:** Spec document structure, Source material strategy, Capture analysis depth, Example selection criteria

---

## Spec Document Structure

| Option | Description | Selected |
|--------|-------------|----------|
| Pipeline-ordered | Sections follow transformation pipeline order (parse → collect → pre-transforms → core → emit) | ✓ |
| CONV-numbered | Sections follow CONV-01 through CONV-14 numbering | |
| Layered by dependency | Start with foundation types, then infrastructure, then transformations | |

**User's choice:** Pipeline-ordered
**Notes:** Matches how the optimizer actually processes code

| Option | Description | Selected |
|--------|-------------|----------|
| Rules + examples | Behavioral rules + 2-3 input/output examples per CONV | ✓ |
| Rules only | Rules without inline examples, examples in appendix | |
| Full reference | Rules + examples + rationale for Qwik's resumability model | |

**User's choice:** Rules + examples

| Option | Description | Selected |
|--------|-------------|----------|
| ASCII diagram | Text-based pipeline diagram | |
| Mermaid diagram | Mermaid flowchart for GitHub/VS Code rendering | ✓ |
| No diagram | Prose description only | |

**User's choice:** Mermaid diagram

| Option | Description | Selected |
|--------|-------------|----------|
| Inline references | "See CONV-01" style within each section | |
| Dependency table | Matrix at start + inline references | |
| You decide | Claude picks best approach | ✓ |

**User's choice:** You decide (Claude's Discretion)

---

## Source Material Strategy

| Option | Description | Selected |
|--------|-------------|----------|
| SWC is truth | SWC v2 defines correct behavior; Jack's deviations noted but SWC wins | ✓ |
| Jack's output is truth | Jack's 162 snapshots are the target, including deviations | |
| Case by case | SWC for runtime-breaking, accept OXC cosmetic style | |

**User's choice:** SWC is truth

| Option | Description | Selected |
|--------|-------------|----------|
| Include SWC source references | Each spec section notes SWC source files for traceability | ✓ |
| No source references | Spec is self-contained | |
| Appendix only | Traceability in appendix, not inline | |

**User's choice:** Include references

| Option | Description | Selected |
|--------|-------------|----------|
| Implementation reference only | Don't reference Scott's conversion in spec | |
| OXC pattern examples | Pull code snippets for OXC migration notes | ✓ |
| Ignore it | Based on outdated SWC, don't use | |

**User's choice:** OXC pattern examples

| Option | Description | Selected |
|--------|-------------|----------|
| Code only, no AST dumps | Spec examples show source code only | |
| Include AST for key cases | AST snippets for capture analysis and segment extraction | |
| You decide | Claude picks based on clarity vs noise | ✓ |

**User's choice:** You decide (Claude's Discretion)

---

## Capture Analysis Depth

| Option | Description | Selected |
|--------|-------------|----------|
| Decision tree + table | Mermaid flowchart + table with 8 categories, rules, examples | ✓ |
| Exhaustive edge case catalog | Every edge case with individual test references | |
| Narrative with examples | Prose description with inline examples | |

**User's choice:** Decision tree + table

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, as spec test cases | Each of 16 deviations becomes named spec example (CAPTURE-EDGE-01 etc.) | ✓ |
| Mention but don't spec | Note 16 exist, reference Jack's Phase 24 | |
| Categorize only | Group into 8 categories, individual edge cases are test-level detail | |

**User's choice:** Yes, as spec test cases

| Option | Description | Selected |
|--------|-------------|----------|
| Dedicated subsection | Self-import reclassification gets its own section | |
| Part of the table | One row with longer description and examples | |
| You decide | Claude picks based on complexity | ✓ |

**User's choice:** You decide (Claude's Discretion)

| Option | Description | Selected |
|--------|-------------|----------|
| Separate section, cross-ref | Variable migration is its own section referencing captures | ✓ |
| Inline after captures | Variable migration as sub-section of capture analysis | |

**User's choice:** Separate section, cross-ref

---

## Example Selection Criteria

| Option | Description | Selected |
|--------|-------------|----------|
| Input + all outputs | Source input + each output module (root + segments) as code blocks | ✓ |
| Input + root output only | Source + root only, segments described | |
| Diff-style | Input + annotated diffs | |

**User's choice:** Input + all outputs

| Option | Description | Selected |
|--------|-------------|----------|
| 2-3 per CONV | Basic, captures/nesting, edge case | ✓ |
| 1 per CONV | One representative example | |
| Match Jack's corpus | All relevant snapshots | |

**User's choice:** 2-3 per CONV

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, always | Segment metadata shown for every example | |
| For segment examples only | Only when demonstrating segment extraction | |
| You decide | Claude picks per example | ✓ |

**User's choice:** You decide (Claude's Discretion)

| Option | Description | Selected |
|--------|-------------|----------|
| Yes, use Jack's names | Direct traceability to 162-test corpus | |
| Rename descriptively | Descriptive names, loses Jack's naming | |
| Both | Descriptive + Jack's name in parentheses | ✓ |

**User's choice:** Both — "Basic Dollar Extraction (example_6)"

---

## Claude's Discretion

- Cross-referencing style (inline references vs dependency table)
- AST JSON dump inclusion per example
- Self-import reclassification presentation format
- Segment metadata inclusion per example

## Deferred Ideas

None — discussion stayed within phase scope
