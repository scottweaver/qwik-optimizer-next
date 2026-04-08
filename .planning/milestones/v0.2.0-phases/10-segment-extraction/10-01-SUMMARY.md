---
phase: 10-segment-extraction
plan: 01
subsystem: optimizer
tags: [oxc, qwik, jsx, segment-extraction, qrl, event-handlers]

requires:
  - phase: v0.1.0
    provides: "Working OXC optimizer with $() call segment extraction and JSX transform"
provides:
  - "JSX dollar-attr segment extraction (onClick$, onFocus$, etc. produce SegmentRecords)"
  - "jsx_event_to_html_attribute function for event name conversion"
  - "DollarAttr struct and classify_jsx_element/build_jsx_call_from_parts APIs"
  - "Recursive JSX child processing for nested element dollar-attr extraction"
  - "Noop segment module emission for stripped handlers"
affects: [11-root-module, 12-diagnostics, 13-acceptance]

tech-stack:
  added: []
  patterns:
    - "Recursive JSX transform with segment extraction at each tree level"
    - "Split classify/build pattern for JSX element transformation"
    - "Dollar-attr detection in classify_props with deferred QRL injection"

key-files:
  created: []
  modified:
    - "crates/qwik-optimizer-oxc/src/transform.rs"
    - "crates/qwik-optimizer-oxc/src/jsx_transform.rs"
    - "crates/qwik-optimizer-oxc/src/lib.rs"

key-decisions:
  - "Recursive JSX processing in transform_jsx_with_segments instead of relying on OXC Traverse exit_expression for nested JSXChild::Element nodes"
  - "Split classify_jsx_element from build_jsx_call_from_parts to allow QRL prop injection between classification and call construction"
  - "Tag name pushed to stack_ctxt around both dollar-attr processing AND children processing for correct nested display_name paths"
  - "Noop segments emitted as export const NAME = null modules instead of being silently skipped"

patterns-established:
  - "transform_jsx_with_segments: recursive JSX tree walk with segment extraction at each level"
  - "DollarAttr: intermediate representation of $-suffixed attributes extracted from classify_props"
  - "process_jsx_dollar_attr: per-attribute segment creation handling all entry strategies"

requirements-completed: [SEG-01, SEG-02, SEG-03, SEG-04, SEG-05]

duration: 30min
completed: 2026-04-04
---

# Phase 10 Plan 01: JSX Attribute Segment Extraction Summary

**JSX dollar-attr segment extraction producing SegmentRecords for onClick$, onFocus$, etc. with QRL replacement props, improving segment count parity from 125/201 to 180/201 (+55 fixtures)**

## Performance

- **Duration:** 30 min
- **Started:** 2026-04-04T02:01:36Z
- **Completed:** 2026-04-04T02:31:50Z
- **Tasks:** 2
- **Files modified:** 3 source files + 77 snapshot files

## Accomplishments
- Segment count parity improved from 125/201 (62%) to 180/201 (90%) -- 55 fixtures fixed
- All 7 plan-specified acceptance criteria fixtures now pass segment count matching
- jsx_event_to_html_attribute ported from SWC with all edge cases (onClick$ -> q-e:click, etc.)
- Recursive JSX child processing handles nested elements that OXC Traverse doesn't visit via exit_expression
- Fragment children processed for dollar-attr extraction
- Noop segments emitted as proper module files for stripped handlers

## Task Commits

1. **Task 1: Port jsx_event_to_html_attribute and restructure classify_props** - `78207b3` (feat)
2. **Task 2: Implement JSX attribute segment extraction in exit_expression** - `68ad3c9` (feat)

## Files Created/Modified
- `crates/qwik-optimizer-oxc/src/transform.rs` - Added jsx_event_to_html_attribute, transform_jsx_with_segments, transform_children_recursive, process_jsx_dollar_attr methods; 9 unit tests
- `crates/qwik-optimizer-oxc/src/jsx_transform.rs` - Added DollarAttr struct, classify_jsx_element, build_jsx_call_from_parts; modified classify_props to extract dollar-suffixed attributes
- `crates/qwik-optimizer-oxc/src/lib.rs` - Noop segment emission (export const NAME = null) instead of skipping segments with no expression

## Decisions Made
- Chose recursive JSX processing approach because OXC Traverse does not call exit_expression for JSXChild::Element nodes (only for Expression::JSXElement in expression context). This required building transform_jsx_with_segments as a recursive method on QwikTransform.
- Split classify_jsx_element from build_jsx_call_from_parts to enable QRL prop injection between classification and final call construction. This avoids modifying CallExpression nodes after construction.
- Tag name pushed to stack_ctxt around BOTH dollar-attr processing AND children processing, matching Jack's approach for correct nested display_name paths (e.g., Cmp_p_q_e_click).

## Deviations from Plan

### Auto-fixed Issues

**1. [Rule 1 - Bug] Recursive JSX child processing for nested elements**
- **Found during:** Task 2
- **Issue:** OXC Traverse does not call exit_expression for JSXChild::Element nodes, so button elements nested inside div/Cmp elements were never processed for dollar-attr extraction
- **Fix:** Added transform_children_recursive method that walks Expression trees looking for JSXElement/JSXFragment nodes and processes them via transform_jsx_with_segments
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** 68ad3c9

**2. [Rule 1 - Bug] Fragment children dollar-attr processing**
- **Found during:** Task 2 (impure_template_fns fixture failure)
- **Issue:** JSXFragment path in exit_expression used transform_jsx_fragment (free function) which doesn't process children for dollar-attrs
- **Fix:** Added post-transform child processing for fragment call expressions, scanning children args for JSXElement nodes
- **Files modified:** crates/qwik-optimizer-oxc/src/transform.rs
- **Committed in:** 68ad3c9

**3. [Rule 2 - Missing Critical] Noop segment module emission**
- **Found during:** Task 2 (example_noop_dev_mode fixture)
- **Issue:** lib.rs skipped segments with expr: None (noop/stripped handlers), but SWC emits them as `export const NAME = null;` modules that count in parity
- **Fix:** Added noop segment emission path in lib.rs that generates proper TransformModule with SegmentAnalysis for stripped handlers
- **Files modified:** crates/qwik-optimizer-oxc/src/lib.rs
- **Committed in:** 68ad3c9

---

**Total deviations:** 3 auto-fixed (2 bugs, 1 missing critical)
**Impact on plan:** All fixes necessary for correct segment extraction. No scope creep.

## Issues Encountered
- test expectations for jsx_event_to_html_attribute("onDblClick$") needed correction: SWC lowercases the full name first (producing "dblclick"), so camelCase-to-kebab doesn't insert hyphens

## Known Stubs
None -- all code paths produce real segment extraction and QRL replacement.

## Next Phase Readiness
- Segment extraction is functional for all JSX dollar-attr patterns
- 21 remaining segment mismatches are pre-existing issues (useStyles$ with string args, foreign JSX import source, non-JSX patterns) -- these may be addressed in Plan 10-02 or Phase 13
- Root module code generation (Phase 11) is the next priority -- 171 root module mismatches remain
- Display name hashes may need verification against SWC reference after root module work

---
*Phase: 10-segment-extraction*
*Completed: 2026-04-04*
