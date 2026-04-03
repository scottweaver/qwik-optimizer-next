# Phase 2: JSX, Props & Signal Specification - Context

**Gathered:** 2026-04-01
**Status:** Ready for planning

<domain>
## Phase Boundary

Write the behavioral specification sections for JSX transformation (CONV-06 / SPEC-06), Props Destructuring (CONV-04 / SPEC-04), and Signal Optimization (CONV-07 / SPEC-07). These sections are appended to the existing `specification/qwik-optimizer-spec.md` document, following the pipeline-ordered structure established in Phase 1.

</domain>

<decisions>
## Implementation Decisions

### JSX Transform Structure
- **D-17:** JSX section broken into subsections: Element Transformation, Prop Classification (static/dynamic), Special Attributes (bind, slot, ref, class), Children Handling, Key Generation. Each subsection gets its own rules + examples.
- **D-18:** _jsxSorted and _jsxSplit documented as one mechanism with a branching condition: _jsxSorted when no spread props, _jsxSplit when spread props exist. Branch point explained first, then each path detailed.

### Props Destructuring
- **D-19:** Props destructuring interaction with capture analysis (Phase 1) handled at Claude's discretion — either cross-reference or brief recap, whichever avoids redundancy while keeping the section usable standalone.
- **D-20:** _restProps() helper documented with full behavioral detail — exact exclusion logic for which props are excluded from rest, with examples.

### Signal Optimization
- **D-21:** _fnSignal application boundaries documented as a decision table: expression type × prop context × captured variables → _fnSignal or plain. Covers all combinations explicitly.
- **D-22:** Both _wrapProp and _fnSignal documented in the same Signal Optimization section. _wrapProp wraps individual signal.value accesses, _fnSignal wraps expressions over captured variables.

### Example Format (Carrying Forward)
- **D-23:** Example count per JSX subsection is Claude's discretion — some subsections (spread props) may need more than 2-3, others (key generation) fewer. Adapts to subsection complexity.
- **D-24:** Each subsection includes inline examples (per D-16 naming convention from Phase 1) PLUS a "See also" list of additional relevant Jack snapshots for thorough testing.

### Carrying Forward from Phase 1
- D-01: Pipeline-ordered document structure
- D-04: Rules + examples per CONV section
- D-05: SWC is source of truth
- D-06: SWC source file references for traceability
- D-13: Examples show input + all output modules
- D-16: Descriptive names with Jack's snapshot name in parentheses

### Claude's Discretion
- Props destructuring ↔ capture analysis interaction format — D-19
- Example count per JSX subsection — D-23

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### SWC Source (Source of Truth)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` — JSX transformation logic: `handle_jsx`, `process_jsx_element`, `handle_jsx_props`, `convert_signal_word`, `_fnSignal` generation (~5,157 LOC total, JSX logic spans multiple functions)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/props_destructuring.rs` — Props destructuring transform: `transform_props_destructuring`, `_rawProps`, `_restProps` (~568 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/inlined_fn.rs` — `_fnSignal` generation: `convert_inlined_fn`, positional parameter creation (~294 LOC)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/test.rs` — Snapshot tests with JSX examples (~7,287 LOC)

### Jack's OXC Implementation (Reference)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/jsx_transform.rs` — OXC JSX handling (~1,654 LOC)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/props_destructuring.rs` — OXC props destructuring (~664 LOC)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/swc-snapshots/` — SWC reference snapshots (~40 JSX-focused .snap files)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/oxc-snapshots/` — OXC output snapshots

### Scott's OXC Conversion (OXC Pattern Reference)
- `/Users/scottweaver/Projects/qwik-optimizer/optimizer/src/transform.rs` — JSX handling patterns in OXC (~1,186 LOC)

### Existing Spec (Phase 1 Output)
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/qwik-optimizer-spec.md` — Current spec document (2,551 lines) with Pipeline Overview, core transforms, and infrastructure sections

### Research
- `/Users/scottweaver/Projects/qwik-optimizer-next/specification/.planning/research/FEATURES.md` — CONV-06 (JSX), CONV-04 (Props), CONV-07 (Signal) descriptions with complexity and dependencies

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- Phase 1 spec document already has the Pipeline Overview Mermaid diagram that shows where JSX transforms fit in the pipeline (Stage 4)
- Phase 1 Capture Analysis section documents the capture system that _fnSignal depends on
- Jack's ~40 JSX-focused snapshots provide verified input/output pairs for all JSX subsections

### Established Patterns
- Spec sections follow Phase 1 conventions: behavioral rules, SWC source references, input/output examples with descriptive + snapshot names
- The spec document grows by appending new sections under the existing Stage 4: Core Transform heading

### Integration Points
- JSX section references Capture Analysis (Phase 1) for captured variable handling in _fnSignal
- Props destructuring must note its pre-pass ordering relative to capture analysis (Phase 1)
- Signal optimization references both JSX prop classification and capture analysis

</code_context>

<specifics>
## Specific Ideas

- JSX subsections should mirror the natural analysis flow: element → props → special attrs → children → key → signal optimization
- The _jsxSorted/_jsxSplit branch point is central — this determines the output call site structure
- Jack's 16 known capture deviations from Phase 1 (CAPTURE-EDGE-*) may intersect with JSX event handler scoping — worth cross-referencing
- STATE.md notes "JSX transform has 16 known edge cases around event handler capture scoping" — research should investigate these

</specifics>

<deferred>
## Deferred Ideas

None — discussion stayed within phase scope

</deferred>

---

*Phase: 02-jsx-props-signal-specification*
*Context gathered: 2026-04-01*
