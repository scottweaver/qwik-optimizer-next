# Phase 10: Segment Extraction - Context

**Gathered:** 2026-04-03 (assumptions mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

The optimizer correctly extracts separate segments for every dollar-sign expression regardless of AST position -- loops, JSX handlers, ternaries, spread props, and inline strategies. This phase fixes ~76 segment count mismatches. It does NOT address root module code generation (Phase 11) or diagnostics (Phase 12).

</domain>

<decisions>
## Implementation Decisions

### JSX Attribute Segment Extraction
- **D-01:** Add a new segment extraction path for `$`-suffixed JSX attribute names (e.g., `onClick$`, `onFocus$`, `onMouseOver$`) that creates segments from the inline function/arrow expression values, mirroring SWC's `handle_jsx_value` approach
- **D-02:** The current `enter_call_expression` path in `transform.rs` correctly handles explicit `$()` calls (e.g., `component$()`) and must remain unchanged -- the new path supplements it for JSX attribute handlers

### Integration Point
- **D-03:** JSX attribute segment extraction should happen during the JSX transform phase (in or near `classify_props` in `jsx_transform.rs` or via `enter_jsx_attribute`/`exit_jsx_attribute` in `transform.rs`), not after JSX lowering, because attribute names and element tag context are needed for display_name generation (e.g., `test.tsx_Foo_component_div_div_q_e_click`)
- **D-04:** The extraction requires passing segment extraction state (segment_stack, stack_ctxt) into or coordinating with the JSX transform module

### Event Handler Attribute Renaming
- **D-05:** `$`-suffixed JSX attributes on HTML elements must be renamed to `q-e:` prefixed names (e.g., `onClick$` -> `q-e:click`) with QRL references replacing inline function values
- **D-06:** For custom components (non-HTML elements), the `$`-suffix is stripped but the prop name is kept (e.g., `onClick$` -> `onClick` with QRL value)
- **D-07:** The renaming logic should match SWC's `jsx_event_to_html_attribute` function

### Loop and Nested Context Handling
- **D-08:** Loop-related segment count failures (12+ fixtures) are expected to be fixed by adding JSX attribute extraction since OXC's `Traverse` already visits into loop bodies, ternary branches, and `.map()` callbacks correctly
- **D-09:** After JSX extraction is implemented, verify that `decl_stack` correctly tracks loop variable bindings (e.g., `for (const val of ...)`) for proper capture analysis -- fix separately if needed

### Claude's Discretion
- Exact integration architecture between `jsx_transform.rs` and `transform.rs` for passing segment state
- Whether to use `enter_jsx_attribute` in `transform.rs` vs extending `classify_props` in `jsx_transform.rs`
- Order of implementation (all-at-once vs incremental by fixture category)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### OXC Implementation (current codebase)
- `crates/qwik-optimizer-oxc/src/transform.rs` -- Main transform logic, `enter_call_expression`, `detect_dollar_call`, segment stack management
- `crates/qwik-optimizer-oxc/src/jsx_transform.rs` -- JSX lowering, `classify_props` (lines 261-378), prop classification
- `crates/qwik-optimizer-oxc/src/code_move.rs` -- Segment extraction and code movement logic
- `crates/qwik-optimizer-oxc/src/types.rs` -- `SegmentAnalysis`, `HookData`, segment-related types
- `crates/qwik-optimizer-oxc/src/entry_strategy.rs` -- Entry strategy types and segment grouping
- `crates/qwik-optimizer-oxc/src/collector.rs` -- Variable capture and scope analysis

### SWC Reference Implementation
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` -- `handle_jsx_value` (around line 1226), `jsx_event_to_html_attribute`, segment extraction from JSX attributes
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/code_move.rs` -- SWC's code movement for comparison

### Jack's OXC Conversion
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/transform.rs` -- Jack's approach to JSX attribute handling, `handle_jsx_props`

### Test Infrastructure
- `crates/qwik-optimizer-oxc/tests/swc_expected/` -- SWC reference snapshots (201 fixtures)
- `crates/qwik-optimizer-oxc/tests/` -- Test harness and parity comparison logic

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `transform.rs:enter_call_expression` -- Existing segment extraction for `$()` calls; pattern for how segments are created
- `jsx_transform.rs:classify_props` -- Already processes all JSX attributes; natural integration point for `$`-suffix detection
- `types.rs:SegmentAnalysis` -- Existing segment type that new segments must conform to
- `hash.rs` -- Segment hash computation used for segment naming

### Established Patterns
- Two-phase analyze-then-emit architecture -- semantic analysis must complete before AST mutation
- `segment_stack` and `stack_ctxt` in transform state -- used for tracking segment nesting and context
- `detect_dollar_call` pattern -- identifies dollar-sign function calls; needs equivalent for JSX attributes
- OXC `Traverse` trait -- visitor pattern with enter/exit methods; already visits into loops and nested scopes

### Integration Points
- `jsx_transform.rs` <-> `transform.rs` -- Need coordination for segment extraction during JSX processing
- `code_move.rs` -- Extracted segments are moved here; must accept segments from both `$()` calls and JSX attributes
- `entry_strategy.rs` -- Segment grouping logic must handle inline/inlined strategies for JSX-sourced segments

</code_context>

<specifics>
## Specific Ideas

No specific requirements -- open to standard approaches following SWC reference patterns.

</specifics>

<deferred>
## Deferred Ideas

None -- analysis stayed within phase scope.

</deferred>

---

*Phase: 10-segment-extraction*
*Context gathered: 2026-04-03*
