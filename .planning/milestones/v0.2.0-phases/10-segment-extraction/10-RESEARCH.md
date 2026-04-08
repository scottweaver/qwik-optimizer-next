# Phase 10: Segment Extraction - Research

**Researched:** 2026-04-03
**Domain:** OXC AST traversal, Qwik dollar-sign segment extraction, JSX attribute handler extraction
**Confidence:** HIGH

## Summary

Phase 10 addresses 76 segment count mismatches between the OXC optimizer and SWC reference. Analysis of the parity report reveals a clear pattern: **the OXC implementation only extracts segments from explicit `$()` call expressions** (via `enter_call_expression`), but the SWC implementation ALSO extracts segments from **`$`-suffixed JSX attribute values** (via `handle_jsx_value` and `fold_jsx_attr`). This missing JSX attribute extraction path is the root cause of nearly all segment count mismatches.

The SWC implementation detects `$`-suffixed attribute names (e.g., `onClick$`, `onDblClick$`, `document:onFocus$`), checks if the value is a function/arrow expression, and if so calls `create_synthetic_qsegment` to extract a separate segment. It also renames the attribute (e.g., `onClick$` -> `q-e:click` for HTML elements, `onClick$` -> `onClick` for components). The OXC implementation currently passes these attributes through to `classify_props` in `jsx_transform.rs` without extracting segments, so inline handler functions are never turned into separate QRL segments.

**Primary recommendation:** Add a JSX attribute segment extraction path that intercepts `$`-suffixed attributes during the Traverse pass, extracts segments using the same machinery as `enter_call_expression`/`exit_expression`, and renames attributes to match SWC output.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- D-01: Add a new segment extraction path for `$`-suffixed JSX attribute names (e.g., `onClick$`, `onFocus$`, `onMouseOver$`) that creates segments from the inline function/arrow expression values, mirroring SWC's `handle_jsx_value` approach
- D-02: The current `enter_call_expression` path in `transform.rs` correctly handles explicit `$()` calls (e.g., `component$()`) and must remain unchanged -- the new path supplements it for JSX attribute handlers
- D-03: JSX attribute segment extraction should happen during the JSX transform phase (in or near `classify_props` in `jsx_transform.rs` or via `enter_jsx_attribute`/`exit_jsx_attribute` in `transform.rs`), not after JSX lowering, because attribute names and element tag context are needed for display_name generation
- D-04: The extraction requires passing segment extraction state (segment_stack, stack_ctxt) into or coordinating with the JSX transform module
- D-05: `$`-suffixed JSX attributes on HTML elements must be renamed to `q-e:` prefixed names (e.g., `onClick$` -> `q-e:click`) with QRL references replacing inline function values
- D-06: For custom components (non-HTML elements), the `$`-suffix is stripped but the prop name is kept (e.g., `onClick$` -> `onClick` with QRL value)
- D-07: The renaming logic should match SWC's `jsx_event_to_html_attribute` function
- D-08: Loop-related segment count failures (12+ fixtures) are expected to be fixed by adding JSX attribute extraction since OXC's `Traverse` already visits into loop bodies, ternary branches, and `.map()` callbacks correctly
- D-09: After JSX extraction is implemented, verify that `decl_stack` correctly tracks loop variable bindings (e.g., `for (const val of ...)`) for proper capture analysis -- fix separately if needed

### Claude's Discretion
- Exact integration architecture between `jsx_transform.rs` and `transform.rs` for passing segment state
- Whether to use `enter_jsx_attribute` in `transform.rs` vs extending `classify_props` in `jsx_transform.rs`
- Order of implementation (all-at-once vs incremental by fixture category)

### Deferred Ideas (OUT OF SCOPE)
None -- analysis stayed within phase scope.
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SEG-01 | Dollar-sign expressions inside for/for-of/while loops produce separate segments per iteration handler | JSX attribute extraction combined with existing loop variable tracking in `enter_for_*_statement` handlers |
| SEG-02 | Multiple event handlers on JSX elements each produce separate segments | JSX attribute extraction iterates all attributes; each `$`-suffixed attr with fn value produces a segment |
| SEG-03 | Nested loop and ternary dollar-sign expressions produce correct segment count | OXC `Traverse` already visits nested scopes; JSX extraction at each attribute creates correct segments |
| SEG-04 | Spread props with additional handler props produce correct segments | JSX attribute extraction handles `$`-suffixed attributes regardless of spread prop presence |
| SEG-05 | Inline and inlined QRL strategies produce correct segment counts | Segment creation respects `is_inline_mode()` for inline/hoist strategies; same path as call expression extraction |
</phase_requirements>

## Architecture Patterns

### Current Architecture (What Exists)

The OXC optimizer uses a single `QwikTransform` struct implementing `Traverse<'a, ()>`. The traversal lifecycle is:

1. `enter_call_expression` -- detects `$()` calls, pushes `SegmentScope` to `segment_stack`
2. Children are visited (nested `$` calls handled recursively)
3. `exit_expression` -- pops `SegmentScope`, computes captures, creates `SegmentRecord`, rewrites call to `qrl()/inlinedQrl()`
4. Later in `exit_expression` -- transforms JSX elements/fragments via `jsx_transform.rs`

**Problem:** JSX attributes like `onClick$={() => {...}}` never go through step 1 because `onClick$` is NOT a call expression -- it's an attribute name. The `$` is part of the attribute name string, not a function being called.

### SWC Reference Architecture

The SWC implementation (in `fold_jsx_attr`) does the following for each JSX attribute:

1. Checks if attribute name ends with `$` (via `convert_qrl_word`)
2. If yes, calls `handle_jsx_value(ctx_name, value)` which:
   a. Checks if value is an arrow/function expression
   b. Determines `SegmentKind` (EventHandler if `jsx_event_to_html_attribute` returns Some, else JSXProp)
   c. Calls `create_synthetic_qsegment(expr, segment_kind, ctx_name, None)` to extract a segment
   d. Hoists the QRL to module scope
   e. Returns the hoisted QRL reference as the new attribute value
3. For HTML elements, renames the attribute (e.g., `onClick$` -> `q-e:click`)
4. For component elements, strips the `$` suffix

### Jack's Reference Architecture

Jack's implementation moves JSX processing into `exit_expression` (for `JSXElement`), where `handle_jsx_props` is a method on `QwikTransform` (not a free function). This gives it direct access to:
- `self.stack_ctxt` -- for display_name generation
- `self.segment_names` -- for collision dedup
- `self.decl_stack` -- for capture analysis
- `self.segments` / `self.create_segment()` -- for segment extraction

Key pattern: Jack re-pushes the tag name to `stack_ctxt` before calling `handle_jsx_props` so that extracted handler segments get correct display names (e.g., `..._div_q_e_click`).

### Recommended Architecture for This Phase

**Option A (Recommended): Inline extraction in `exit_expression` JSX path**

Move JSX transformation from the free function `jsx_transform::transform_jsx_element` to a method on `QwikTransform` (or pass `&mut QwikTransform` to the transform function). During prop classification, detect `$`-suffixed attributes and extract segments using the same `create_segment` / `SegmentRecord` machinery.

This follows Jack's pattern and matches D-03 (extraction during JSX transform phase, before lowering).

**Advantages:**
- Direct access to all transform state (stack_ctxt, segment_stack, decl_stack, segments)
- Single pass -- no deferred processing needed
- Matches both SWC and Jack's approach

**Disadvantages:**
- Increases coupling between `jsx_transform.rs` and `transform.rs`
- `classify_props` signature changes significantly

**Option B: Use `enter_jsx_attribute`/`exit_jsx_attribute` in Traverse**

Push a `SegmentScope` in `enter_jsx_attribute` when the attribute name ends with `$` and its value is a function expression, then handle it in `exit_expression` like regular `$()` calls.

**Advantages:**
- Reuses existing enter/exit machinery
- Minimal new code

**Disadvantages:**
- JSX attribute values aren't `CallExpression` nodes, so `exit_expression` matching on `call_span_start` won't work without adaptation
- Harder to handle attribute renaming (need the JSX context)

**Recommendation:** Option A. Move prop classification into `QwikTransform` methods so that segment extraction happens during the JSX transform phase with full access to transform state. This matches Jack's proven approach.

### Recommended Implementation Structure

```
transform.rs:
  exit_expression() {
    // JSXElement handling:
    // 1. Take JSXElement from expr
    // 2. Call self.transform_jsx_element(el, ctx) -- method on QwikTransform
    // 3. Replace expr with result
  }

  transform_jsx_element(&mut self, el, ctx) -> Expression {
    // 1. Classify tag (component vs intrinsic)
    // 2. Push tag name to stack_ctxt
    // 3. Call self.process_jsx_props(attrs, is_fn, ctx) -- handles $-suffixed extraction
    // 4. Pop tag name from stack_ctxt
    // 5. Build _jsxSorted/_jsxSplit call
  }

  process_jsx_props(&mut self, attrs, is_fn, ctx) -> ... {
    // For each attribute:
    //   - If name ends with '$' and value is fn/arrow:
    //     a. Push attr name (e.g. "q-e:click") to stack_ctxt
    //     b. register_context_name to get names
    //     c. Compute scoped_idents via capture analysis
    //     d. Serialize fn body, create SegmentRecord
    //     e. Build qrl() / inlinedQrl() replacement expression
    //     f. Pop stack_ctxt
    //     g. Rename attribute (onClick$ -> q-e:click / onClick)
    //   - Otherwise: normal prop classification (const/var/signal opt)
  }
```

### Event Name Transformation (jsx_event_to_html_attribute)

The SWC implementation converts JSX event names to HTML attribute names:

```
onClick$       -> q-e:click
onDblClick$    -> q-e:dbl-click      (camelCase -> kebab-case)
onInput$       -> q-e:input
onBlur$        -> q-e:blur
on-anotherCustom$  -> q-e:anothercustom  (hyphen prefix = case-sensitive)
document:onFocus$  -> q-d:focus       (document: prefix)
window:onClick$    -> q-w:click       (window: prefix)
onDOMContentLoaded$ -> q-e:d-o-m-content-loaded  (special case)
```

Rules:
1. Must end with `$`
2. Prefix: `on` -> `q-e:`, `window:on` -> `q-w:`, `document:on` -> `q-d:`
3. Name after prefix: lowercase, then camelCase -> kebab-case
4. Hyphen prefix (e.g., `on-foo$`): keep case after hyphen
5. Special case: `DOMContentLoaded` -> `d-o-m-content-loaded`

This function already exists in the SWC codebase (lines 4658-4710 of `transform.rs`). It needs to be ported to our OXC codebase.

### Context Kind Determination for JSX Attributes

From SWC `handle_jsx_value`:
```rust
let segment_kind = if jsx_event_to_html_attribute(&ctx_name).is_some() {
    SegmentKind::EventHandler
} else {
    SegmentKind::JSXProp
};
```

For attributes that match event handler patterns (onClick$, onFocus$, etc.), the ctx_kind is `EventHandler`. For other `$`-suffixed props (e.g., custom component props like `render$`), the ctx_kind is `JSXProp`.

Note: Our `CtxKind` enum already has `EventHandler`, `Function`, and `JSXProp` variants.

### Segment Naming for JSX Handlers

From the parity report fixtures, the display_name includes the full JSX context path:
```
test.tsx_ManyEventsComponent_component_div_button_q_e_click_1
```

This means `stack_ctxt` at extraction time must contain:
```
["ManyEventsComponent", "component", "div", "button", "q-e:click"]
```

The current `enter_jsx_opening_element` already pushes tag names. The current `enter_jsx_attribute` already pushes attribute names. But the attribute name pushed needs to be the **transformed** name (e.g., `q-e:click` not `onClick$`) for the hash to match SWC.

### Segment Hash and display_name

The segment hash is computed from the display_name_core (joined stack_ctxt). For SWC parity, the stack must produce the exact same joined string. The `escape_sym` function normalizes special characters (`:`, `-`, `.`) to `_`, so `q-e:click` becomes `q_e_click` in the symbol name.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Event name transformation | Manual string manipulation | Port SWC's `jsx_event_to_html_attribute` directly | Complex edge cases (DOMContentLoaded, kebab-case, prefixes) |
| Segment hash computation | New hash function | Existing `hash::register_context_name` | Already produces correct hashes for `$()` call segments |
| Capture analysis | New capture logic | Existing `compute_scoped_idents` + `classify_captures` | Same capture analysis needed for JSX handlers as for `$()` calls |
| QRL wrapping | New wrapping code | Existing `qrl()`/`inlinedQrl()` construction in `exit_expression` | Must produce identical output format |

## Common Pitfalls

### Pitfall 1: Stack Context Ordering
**What goes wrong:** The display_name hash doesn't match SWC because stack_ctxt entries are in the wrong order or contain the wrong names.
**Why it happens:** The OXC Traverse fires `enter_jsx_opening_element` (pushes tag), then visits attributes. If we push the attribute name before extracting, the stack order matches SWC. But if the JSX transform happens in `exit_expression` (after attributes are visited), the attribute-level stack entries from `enter_jsx_attribute` have already been popped.
**How to avoid:** When processing attributes during JSX transformation in `exit_expression`, manually push/pop the transformed attribute name (e.g., `q-e:click`) to `stack_ctxt` around each extraction call, matching Jack's approach.
**Warning signs:** Hash mismatches in parity report despite correct segment counts.

### Pitfall 2: Attribute Name vs ctx_name
**What goes wrong:** SegmentAnalysis `ctx_name` field contains the wrong value.
**Why it happens:** SWC uses the original `$`-suffixed name (e.g., `onClick$`) as ctx_name in the segment metadata, but uses the transformed name (e.g., `q-e:click`) for the display_name path. Confusing the two produces wrong metadata.
**How to avoid:** `ctx_name` = original attribute name with `$` (e.g., `onClick$`). `stack_ctxt` push = transformed HTML attribute name (e.g., `q-e:click`).
**Warning signs:** Segment metadata `ctxName` field doesn't match SWC snapshots.

### Pitfall 3: Component vs HTML Element Attribute Handling
**What goes wrong:** Event handler attributes on custom components get `q-e:` prefixed names, or HTML element handlers don't get renamed.
**Why it happens:** The transform needs to know whether the parent element is an HTML intrinsic (lowercase tag) or a component (uppercase tag).
**How to avoid:** The `is_fn` flag from `classify_tag` determines this. HTML elements (is_fn=false) get `jsx_event_to_html_attribute` renaming. Components (is_fn=true) get `$` suffix stripped only.
**Warning signs:** Root module output shows wrong attribute names in `_jsxSorted` calls.

### Pitfall 4: Loop Variable Capture with JSX Handlers
**What goes wrong:** Handlers inside loops don't capture loop variables, or capture too many variables.
**Why it happens:** Loop variable declarations (e.g., `for (const val of ...)`) need to be in `decl_stack` when capture analysis runs for the handler. If the JSX transform runs after the loop exit (which pops the scope), captures are missed.
**How to avoid:** Since OXC's `Traverse` visits attributes and their values while INSIDE the loop body, the `decl_stack` still has the loop frame. Capture analysis using the current `decl_stack` state at the point of JSX transformation (in `exit_expression`) will correctly include loop variables. The existing `enter_for_of_statement`/`enter_for_in_statement` handlers already push loop variables to `decl_stack`.
**Warning signs:** Loop handler fixtures show `captures: false` when they should be `true`, or scoped_idents is empty.

### Pitfall 5: Segment Source Text Extraction
**What goes wrong:** The extracted function body code is wrong or empty.
**Why it happens:** In the current `exit_expression` for `$()` calls, the function body is extracted from source text using spans. For JSX attribute values, the expression span points to the arrow/function within the attribute value, which should work the same way.
**How to avoid:** Use the same span-based source text extraction (`self.source_text[start..end]`) for JSX handler function bodies. Alternatively, serialize the expression via OXC Codegen (Jack's approach).
**Warning signs:** Segment module files contain wrong code or are empty.

### Pitfall 6: Inline/Hoist Strategy Segments Still Need Records
**What goes wrong:** Inline/Hoist mode doesn't create segment records for JSX handlers.
**Why it happens:** The `is_inline_mode()` check changes HOW the QRL is created (inlinedQrl vs qrl), but segments still need SegmentRecord entries for the parity report to count them.
**How to avoid:** Always push a `SegmentRecord` regardless of inline/hoist mode. The `is_inline` flag on the record controls whether a separate module file is emitted.
**Warning signs:** Segment counts for Inline/Hoist fixtures are wrong.

## Code Examples

### SWC's handle_jsx_value (Reference Pattern)
```rust
// Source: /Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs:1226-1259
fn handle_jsx_value(
    &mut self,
    ctx_name: Atom,
    value: Option<ast::JSXAttrValue>,
) -> Option<ast::JSXAttrValue> {
    if let Some(ast::JSXAttrValue::JSXExprContainer(container)) = value {
        if let ast::JSXExpr::Expr(expr) = container.expr {
            let is_fn = matches!(*expr, ast::Expr::Arrow(_) | ast::Expr::Fn(_));
            if is_fn {
                let segment_kind = if jsx_event_to_html_attribute(&ctx_name).is_some() {
                    SegmentKind::EventHandler
                } else {
                    SegmentKind::JSXProp
                };
                let qrl = self.create_synthetic_qsegment(*expr, segment_kind, ctx_name, None);
                let hoisted = self.hoist_qrl_to_module_scope(qrl);
                Some(ast::JSXAttrValue::JSXExprContainer(ast::JSXExprContainer {
                    span: DUMMY_SP,
                    expr: ast::JSXExpr::Expr(Box::new(hoisted)),
                }))
            } else {
                Some(ast::JSXAttrValue::JSXExprContainer(container))
            }
        } else {
            Some(ast::JSXAttrValue::JSXExprContainer(container))
        }
    } else {
        value
    }
}
```

### Jack's JSX Handler Extraction (Reference Pattern)
```rust
// Source: /Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/transform.rs:1453-1516
// Inside handle_jsx_props, for event handler attributes on native elements:
if matches!(
    &value_expr,
    Expression::ArrowFunctionExpression(_) | Expression::FunctionExpression(_)
) {
    let ctx_kind = CtxKind::EventHandler;
    let ctx_name_for_seg = key.clone();  // e.g., "onClick$"
    let descendent_idents = IdentCollector::collect(&value_expr);
    let all_decl: Vec<IdPlusType> = self.decl_stack
        .iter()
        .flat_map(|frame| frame.iter().cloned())
        .collect();
    let (mut scoped_idents, _) = compute_scoped_idents(&descendent_idents, &all_decl);
    let fn_params = get_function_params(&value_expr);
    scoped_idents.retain(|id| !fn_params.contains(id));
    
    // Push HTML attr name for correct display_name
    self.stack_ctxt.push(html_attr.clone());
    let names = hash::register_context_name(
        &self.stack_ctxt, &mut self.segment_names,
        self.scope.as_deref(), &self.rel_path, &self.file_name, &self.mode,
        None, None, None,
    );
    self.stack_ctxt.pop();
    
    let qrl_expr = self.create_segment(
        value_expr, &names, scoped_idents, local_idents,
        &ctx_name_for_seg, ctx_kind, fn_span_tuple, allocator,
    );
    self.hoist_qrl_to_module_scope(qrl_expr, &scoped_for_hoist, &sym_for_hoist, allocator)
}
```

### jsx_event_to_html_attribute (Must Port)
```rust
// Source: /Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs:4658-4710
fn jsx_event_to_html_attribute(jsx_event: &str) -> Option<String> {
    if !jsx_event.ends_with('$') {
        return None;
    }
    let (prefix, idx) = get_event_scope_data_from_jsx_event(jsx_event);
    if idx == usize::MAX {
        return None;
    }
    let name = &jsx_event[idx..jsx_event.len() - 1];
    if name == "DOMContentLoaded" {
        return Some(format!("{}-d-o-m-content-loaded", prefix));
    }
    let processed_name = if let Some(stripped) = name.strip_prefix('-') {
        stripped.to_string()  // marker for case sensitive event name
    } else {
        name.to_lowercase()
    };
    Some(create_event_name(&processed_name, prefix))
}

fn get_event_scope_data_from_jsx_event(jsx_event: &str) -> (&str, usize) {
    if jsx_event.starts_with("window:on") { ("q-w:", 9) }
    else if jsx_event.starts_with("document:on") { ("q-d:", 11) }
    else if jsx_event.starts_with("on") { ("q-e:", 2) }
    else { ("", usize::MAX) }
}

fn create_event_name(name: &str, prefix: &str) -> String {
    let mut result = String::from(prefix);
    for c in name.chars() {
        if c.is_ascii_uppercase() || c == '-' {
            result.push('-');
            result.push(c.to_ascii_lowercase());
        } else {
            result.push(c);
        }
    }
    result
}
```

## Parity Report Analysis

Current baseline: 125/201 segment count match (62%).

### Fixture Categories Affected

**Category 1: JSX Event Handlers (33+ fixtures)**
Fixtures like `should_convert_jsx_events`, `should_transform_multiple_event_handlers`, `moves_captures_when_possible`. Pattern: `<button onClick$={() => {...}} />` -- the `onClick$` inline handler is never extracted.

**Category 2: Loop + JSX Handler (12+ fixtures)**
Fixtures like `should_transform_handler_in_for_of_loop`, `should_transform_block_scoped_variables_in_loop`, `should_transform_nested_loops`. Pattern: `for (const val of arr) { items.push(<div onClick$={() => use(val)} />) }` -- handlers inside loops need individual segments with proper loop variable captures.

**Category 3: bind:value / bind:checked (4 fixtures)**
Fixtures like `should_merge_bind_value_and_on_input`, `should_move_bind_value_to_var_props`. Pattern: `<input bind:value={signal} />` expands to include an onInput$ handler segment.

**Category 4: Component Props (5+ fixtures)**
Fixtures like `should_not_transform_events_on_non_elements`, `should_transform_component_with_normal_function`. Pattern: `<CustomCmp onClick$={() => {...}} />` -- needs segment extraction but NO `q-e:` renaming.

**Category 5: Spread + Handler (3+ fixtures)**
Fixtures like `should_split_spread_props_with_additional_prop4`. Pattern: `<div {...spread} onClick$={() => {...}} />` -- spread presence doesn't prevent handler extraction.

**Category 6: Ternary/Conditional (2+ fixtures)**
Fixtures like `should_transform_qrls_in_ternary_expression`. Pattern: `{condition ? <div onClick$={...} /> : null}` -- handlers inside ternary branches.

**Category 7: Inline/Inlined Strategy (5+ fixtures)**
Fixtures like `example_prod_node`, `example_strip_server_code`. Pattern: Different entry strategies still need correct segment records.

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | cargo test (built-in Rust test framework) |
| Config file | `Cargo.toml` test configuration |
| Quick run command | `cargo test -p qwik-optimizer-oxc --test snapshot_tests swc_parity -- --nocapture` |
| Full suite command | `cargo test -p qwik-optimizer-oxc` |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SEG-01 | Loop handlers produce separate segments | integration | `cargo test -p qwik-optimizer-oxc --test snapshot_tests should_transform_handler_in_for_of_loop -- --nocapture` | Existing snapshot test |
| SEG-02 | Multiple event handlers produce separate segments | integration | `cargo test -p qwik-optimizer-oxc --test snapshot_tests should_convert_jsx_events -- --nocapture` | Existing snapshot test |
| SEG-03 | Nested loops/ternaries produce correct segment count | integration | `cargo test -p qwik-optimizer-oxc --test snapshot_tests should_transform_nested_loops -- --nocapture` | Existing snapshot test |
| SEG-04 | Spread + handler props produce correct segments | integration | `cargo test -p qwik-optimizer-oxc --test snapshot_tests should_split_spread_props_with_additional_prop4 -- --nocapture` | Existing snapshot test |
| SEG-05 | Inline/inlined strategies produce correct segment counts | integration | `cargo test -p qwik-optimizer-oxc --test snapshot_tests example_prod_node -- --nocapture` | Existing snapshot test |

### Sampling Rate
- **Per task commit:** `cargo test -p qwik-optimizer-oxc --test snapshot_tests swc_parity -- --nocapture` (parity report)
- **Per wave merge:** `cargo test -p qwik-optimizer-oxc` (full suite)
- **Phase gate:** Parity report shows >= 195/201 segment count match (from 125/201 baseline)

### Wave 0 Gaps
None -- existing test infrastructure (snapshot_tests with parity_report) covers all phase requirements. The 201 SWC reference snapshots serve as the test corpus.

## Key Implementation Details

### What Needs to Change in `transform.rs`

1. **Move JSX element transformation into QwikTransform methods** -- Currently calls free function `jsx_transform::transform_jsx_element`. Needs to become a method that can access `self.stack_ctxt`, `self.segments`, `self.decl_stack`, etc.

2. **Add `jsx_event_to_html_attribute` function** -- Port from SWC. Needed for both attribute renaming and ctx_kind determination.

3. **Add JSX attribute segment extraction in prop processing** -- During attribute iteration, detect `$`-suffixed names with fn/arrow values, extract segments, replace values with QRL references.

4. **Handle attribute renaming** -- Transform `onClick$` to `q-e:click` (HTML) or `onClick` (component) in the final prop output.

5. **Ensure `stack_ctxt` is correct** -- The JSX opening element tag name must already be on the stack. The transformed attribute name (e.g., `q-e:click`) must be pushed before `register_context_name` and popped after.

### What Stays Unchanged

- `enter_call_expression` / `exit_expression` for `$()` calls (D-02)
- `SegmentRecord` struct and fields
- `hash::register_context_name`
- `compute_scoped_idents` and capture analysis
- `code_move.rs` segment emission
- Test infrastructure

### Interaction with `jsx_transform.rs`

The current `classify_props` free function returns prop classification data but has no access to transform state. Options:

1. **Replace with method on QwikTransform** -- Move prop classification logic into `transform.rs` as a method. Keep `jsx_transform.rs` for utilities (tag classification, JSX call building, etc.)

2. **Pass callback/closure** -- Pass a closure to `classify_props` that handles `$`-suffixed attributes. Complex and ergonomic issues with Rust lifetimes.

3. **Two-pass approach** -- First pass: classify_props returns `$`-suffixed attributes unprocessed. Second pass: QwikTransform handles extraction. Adds complexity.

**Recommendation:** Option 1. The prop classification with segment extraction is inherently coupled to transform state. Jack's implementation confirms this -- his `handle_jsx_props` is a method on QwikTransform.

## Open Questions

1. **Hoist strategy `.s()` calls for JSX handlers**
   - What we know: The Hoist strategy uses `.s(fn_body)` ref_assignments instead of separate module files. The existing code handles this for `$()` calls.
   - What's unclear: Whether JSX handler segments need the same `.s()` treatment for Hoist strategy.
   - Recommendation: Implement the same Hoist path for JSX handlers. If it works for `$()` calls, it should work identically for JSX handler segments.

2. **bind:value / bind:checked expansion**
   - What we know: SWC expands `bind:value={signal}` to a value prop + an onInput$ handler. The handler is a fixed function (`_val` / `_chk`) wrapped in inlinedQrl.
   - What's unclear: Whether bind expansion needs to happen in this phase or is already partially handled.
   - Recommendation: Include bind expansion as it creates segments that are counted in parity.

3. **q:p / q:ps prop injection for loop captures**
   - What we know: SWC injects `q:p` (single capture) or `q:ps` (multiple captures) props when event handlers inside loops capture iteration variables.
   - What's unclear: Exact behavior and when this injection is needed.
   - Recommendation: Implement after basic JSX extraction works. If parity mismatches remain for loop fixtures after basic extraction, add q:p/q:ps injection.

## Project Constraints (from CLAUDE.md)

- **Behavioral fidelity**: OXC must produce functionally equivalent output to SWC for all test cases
- **OXC idioms**: Use `Traverse` trait, arena allocators, `Codegen` -- not SWC patterns translated to OXC APIs
- **No SWC crates**: Do not import any SWC crate even for utilities
- **No lazy_static**: Use `std::sync::LazyLock` instead
- **Acceptance criteria**: All `*.snap` files under `tests/swc_expected/` must match
- **Reference repos**: Pull all 3 reference repos before meaningful work (SWC qwik, Jack's qwik-oxc-optimizer, kunai-consulting qwik-optimizer)

## Sources

### Primary (HIGH confidence)
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` -- SWC reference: `handle_jsx_value` (L1226-1259), `fold_jsx_attr` (L3870-3930), `jsx_event_to_html_attribute` (L4658-4710)
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/transform.rs` -- Jack's reference: `handle_jsx_props` (L1122-1534), JSX stack_ctxt management (L3030-3090)
- `/Users/scottweaver/Projects/qwik-optimizer-next/crates/qwik-optimizer-oxc/src/transform.rs` -- Current implementation: `enter_call_expression` (L1004), `exit_expression` (L1043), JSX handling (L1707-1745)
- `/Users/scottweaver/Projects/qwik-optimizer-next/crates/qwik-optimizer-oxc/src/jsx_transform.rs` -- Current JSX transform: `classify_props` (L261-378)
- Parity report output -- 76 segment count mismatches analyzed

### Secondary (MEDIUM confidence)
- `/Users/scottweaver/Projects/qwik-optimizer-next/crates/qwik-optimizer-oxc/tests/swc_expected/` -- 201 SWC reference snapshots

## Metadata

**Confidence breakdown:**
- Standard stack: HIGH - No new dependencies needed, all work within existing OXC crate
- Architecture: HIGH - Both SWC and Jack's implementations provide clear reference patterns
- Pitfalls: HIGH - Identified through analysis of actual code and parity mismatches

**Research date:** 2026-04-03
**Valid until:** 2026-05-03 (stable -- OXC API at 0.123, unlikely to break Traverse trait)
