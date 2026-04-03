# Phase 2: JSX, Props & Signal Specification - Research

**Researched:** 2026-04-01
**Domain:** Specification writing -- JSX transformation, props destructuring, and signal optimization behavioral rules
**Confidence:** HIGH

## Summary

Phase 2 specifies three interconnected subsystems of the Qwik v2 optimizer: JSX transformation (CONV-06), props destructuring (CONV-04), and signal optimization (CONV-07). These subsystems produce the largest behavioral surface area in the optimizer. The JSX transform converts `jsx()` calls into `_jsxSorted()` or `_jsxSplit()` calls with separated static/dynamic props. Props destructuring is a pre-pass that converts component parameter destructuring to `_rawProps` member-access patterns. Signal optimization wraps reactive expressions in `_fnSignal()` or `_wrapProp()` for fine-grained reactivity tracking.

All three subsystems have been thoroughly investigated through the SWC source code (`transform.rs`, `props_destructuring.rs`, `inlined_fn.rs`) and verified against Jack's 201 SWC snapshots (of which ~70 are directly relevant to these three subsystems). The behavioral rules are well-defined and deterministic. The primary challenge for the spec writer is organizing the JSX transform's many interacting rules (prop classification, event name conversion, bind sugar, spread handling, key generation, flag computation) into a clear, navigable structure.

**Primary recommendation:** Structure the JSX section around the branching decision tree: first document the `_jsxSorted` vs `_jsxSplit` branch condition, then document prop classification (static/dynamic/children/special), then document each special attribute subsystem. Use the existing Phase 1 conventions for rules + examples.

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions
- **D-17:** JSX section broken into subsections: Element Transformation, Prop Classification (static/dynamic), Special Attributes (bind, slot, ref, class), Children Handling, Key Generation. Each subsection gets its own rules + examples.
- **D-18:** _jsxSorted and _jsxSplit documented as one mechanism with a branching condition: _jsxSorted when no spread props, _jsxSplit when spread props exist. Branch point explained first, then each path detailed.
- **D-19:** Props destructuring interaction with capture analysis (Phase 1) handled at Claude's discretion -- either cross-reference or brief recap, whichever avoids redundancy while keeping the section usable standalone.
- **D-20:** _restProps() helper documented with full behavioral detail -- exact exclusion logic for which props are excluded from rest, with examples.
- **D-21:** _fnSignal application boundaries documented as a decision table: expression type x prop context x captured variables -> _fnSignal or plain. Covers all combinations explicitly.
- **D-22:** Both _wrapProp and _fnSignal documented in the same Signal Optimization section. _wrapProp wraps individual signal.value accesses, _fnSignal wraps expressions over captured variables.
- **D-23:** Example count per JSX subsection is Claude's discretion -- some subsections (spread props) may need more than 2-3, others (key generation) fewer. Adapts to subsection complexity.
- **D-24:** Each subsection includes inline examples (per D-16 naming convention from Phase 1) PLUS a "See also" list of additional relevant Jack snapshots for thorough testing.
- Carrying forward: D-01 (pipeline-ordered), D-04 (rules + examples), D-05 (SWC source of truth), D-06 (SWC source file references), D-13 (input + all output modules), D-16 (descriptive names with snapshot name).

### Claude's Discretion
- Props destructuring <-> capture analysis interaction format -- D-19
- Example count per JSX subsection -- D-23

### Deferred Ideas (OUT OF SCOPE)
None -- discussion stayed within phase scope
</user_constraints>

<phase_requirements>
## Phase Requirements

| ID | Description | Research Support |
|----|-------------|------------------|
| SPEC-04 | Spec documents CONV-04 (Props Destructuring) -- transformation of destructured component props to `_rawProps` access patterns, `_restProps()` handling, pre-pass ordering requirement | Full behavioral rules extracted from `props_destructuring.rs` (568 LOC). `transform_component_props`, `transform_pat`, `transform_rest`, `create_omit_props` fully analyzed. Rest props exclusion logic documented. Pre-pass ordering verified in `parse.rs` execution order. |
| SPEC-06 | Spec documents CONV-06 (JSX Transform) -- `_jsxSorted()`/`_jsxSplit()` conversion, static/dynamic prop separation, `class`/`className` normalization, `bind:value`/`bind:checked` sugar, `q:slot`, `ref`, children extraction, key counter generation | Full behavioral rules extracted from `transform.rs` `handle_jsx`, `handle_jsx_props_obj`, `internal_handle_jsx_props_obj`, `transform_jsx_prop`, event name conversion, flag computation, key generation. All special attributes cataloged. |
| SPEC-07 | Spec documents CONV-07 (Signal Optimization) -- `_fnSignal()` generation for inline JSX expressions, parameterized function creation with positional params (`p0`, `p1`) | Full behavioral rules extracted from `inlined_fn.rs` `convert_inlined_fn` (294 LOC) and `transform.rs` `create_synthetic_qqsegment`, `convert_to_getter`, `convert_to_signal_item`. Decision boundaries documented: when _fnSignal vs _wrapProp vs plain. |
</phase_requirements>

## Architecture Patterns

### Document Structure for Phase 2 Sections

The three new sections integrate into the existing spec document as follows:

```
qwik-optimizer-spec.md (existing)
  ## Stage 3: Pre-Transforms              <-- currently a placeholder
    ### Props Destructuring (CONV-04)      <-- NEW (Phase 2)
  ## Stage 4: Core Transform               <-- existing with CONV-01,02,03,05,12
    ### JSX Transform (CONV-06)            <-- NEW (Phase 2)
    ### Signal Optimization (CONV-07)      <-- NEW (Phase 2)
```

### JSX Transform Internal Structure (per D-17)

The JSX Transform section should follow this subsection structure, mirroring the code's analysis flow:

```
### JSX Transform (CONV-06)
  #### _jsxSorted vs _jsxSplit Branch Point     (D-18: branch first)
  #### Element Transformation                   (jsx() -> _jsxSorted/_jsxSplit)
  #### Prop Classification                      (static vs dynamic separation)
  #### Special Attributes                       (bind, slot, ref, class, events)
  #### Children Handling                        (extraction, text-only, arrays)
  #### Key Generation                           (auto-generated keys)
  #### Flag Computation                         (static_listeners, static_subtree, moved_captures)
  #### Spread Props Handling                    (_getVarProps/_getConstProps)
```

### Subsection Pattern (Carrying Forward D-04, D-06, D-16, D-24)

Each subsection follows this template:
1. One-paragraph behavioral summary
2. Source reference (file:line range)
3. Numbered behavioral rules
4. Inline examples with descriptive names + snapshot names
5. "See also" snapshot list for additional test coverage

## JSX Transform -- Behavioral Rules Extracted from SWC

This section catalogs the complete behavioral rules discovered in the SWC source. The spec writer should organize these into the subsection structure above.

### Core Mechanism: jsx() to _jsxSorted/_jsxSplit

**Source:** transform.rs:1149-1224 (`handle_jsx`)

The JSX transform intercepts calls to `jsx()`, `jsxs()`, and `jsxDEV()` (generated by SWC's JSX transpiler in Stage 1, Step 4) and converts them to Qwik's internal `_jsxSorted()` or `_jsxSplit()` calls.

**Input call signature:** `jsx(type, props_object, key?)`

**Output call signature:** `_jsxSorted(type, varProps, constProps, children, flags, key)` or `_jsxSplit(type, varProps, constProps, children, flags, key)` -- same 6-argument signature for both.

**Branch condition (D-18):**
- `_jsxSplit` when: spread props exist (`has_spread_props`) OR component has `bind:*` props (`has_component_bind_props`)
- `_jsxSorted` when: neither condition holds (no spreads, no component bind directives)
- In the code, the variable `should_sort` indicates `_jsxSplit` path (confusingly named -- `should_sort = true` means "should runtime sort" which means `_jsxSplit`). When `should_sort = false`, props are sorted at compile time and `_jsxSorted` is used.

**Var props sorting:** When using `_jsxSorted` (no spreads), var props are sorted alphabetically by key name at compile time (transform.rs:2624-2648). When using `_jsxSplit`, props are NOT sorted (runtime handles ordering).

**Dev mode:** In Dev or Hmr emit modes, a 7th argument is added containing source location info (transform.rs:1211-1213).

### Element Type Detection

**Source:** transform.rs:1158-1174

| Expression Type | `is_fn` | `is_text_only` | `name_token` | Mutability |
|----------------|---------|----------------|--------------|------------|
| String literal (e.g., `"div"`) | false | checked via `is_text_only()` | true | no change |
| Identifier (e.g., `MyComponent`) | true | false | true | mutable unless in `immutable_function_cmp` |
| Other expression | true | false | false | always mutable |

**Text-only elements** (transform.rs:5099-5103): `text`, `textarea`, `title`, `option`, `script`, `style`, `noscript`. These elements force children to be treated as text content (no JSX children optimization).

### Prop Classification: Static vs Dynamic

**Source:** transform.rs:2066-2653 (`handle_jsx_props_obj`, `internal_handle_jsx_props_obj`)

The core of JSX transformation is splitting props into two buckets:

1. **const_props (static):** Values known at compile time -- literals, template literals with only literal parts, imported identifiers, variables declared as `const` with static initializers. These go into the third argument.
2. **var_props (dynamic):** Everything else -- signal accesses, computed expressions, function calls. These go into the second argument.

**Static determination** uses `is_const_expr()` which checks:
- Literal values (string, number, boolean, null)
- Identifiers that are in the `const_idents` list (variables marked as `IdentType::Var(true)`)
- Global identifiers (imports, etc.)
- Template literals where all expressions are const

**Empty buckets:** If var_props is empty, the second argument is `null`. If const_props is empty, the third argument is `null`. (transform.rs:2081-2098)

**Spread props interaction:** When spread props exist (before the last spread), ALL regular props that appear before the last spread go to var_props regardless of their static nature (transform.rs:2270-2276). Props after the last spread follow normal static/dynamic classification unless there are non-const props after the last spread (`has_var_prop_after_last_spread`).

### Special Attributes Catalog

**Source:** transform.rs:1744-1866, words.rs

| Attribute | Behavior | Native Elements | Components |
|-----------|----------|-----------------|------------|
| `children` | Extracted to 4th argument, not placed in props | Same | Same |
| `ref` | Always goes to var_props (not optimized) | Yes | N/A |
| `q:slot` | Always goes to var_props (not optimized) | Yes | N/A |
| `className` | Renamed to `class` for native elements; kept as `className` for components | Renamed | Kept |
| `class` | Kept as-is | Yes | Yes |
| `bind:value` | Expands to `value` prop + `q-e:input` handler using `_val` | Expanded (see below) | Forces `_jsxSplit` |
| `bind:checked` | Expands to `checked` prop + `q-e:input` handler using `_chk` | Expanded (see below) | Forces `_jsxSplit` |
| `bind:*` (other) | Kept as-is in const_props (unrecognized bind) | Kept | Forces `_jsxSplit` |
| `on*$` (event handlers) | Arrow/function expressions extracted as QRL segments; event name converted to `q-e:` prefix | Converted | Kept (component props use `$`-suffix as-is for Qrl conversion) |
| `q:p` / `q:ps` | Injected for iteration variable lifting (single/multiple) | Injected | N/A |

### Event Name Conversion

**Source:** transform.rs:4639-4694 (`jsx_event_to_html_attribute`, `get_event_scope_data_from_jsx_event`, `create_event_name`)

Event handler prop names ending with `$` on native elements are converted to HTML attribute format:

| JSX Name Pattern | Prefix | Example Input | Example Output |
|-----------------|--------|---------------|----------------|
| `on*$` (e.g., `onClick$`) | `q-e:` | `onClick$` | `q-e:click` |
| `document:on*$` | `q-d:` | `document:onScroll$` | `q-d:scroll` |
| `window:on*$` | `q-w:` | `window:onResize$` | `q-w:resize` |

**Case conversion rules:**
1. Remove the `$` suffix
2. Strip the scope prefix (`on`, `document:on`, `window:on`) to get the event name
3. If the event name starts with `-`, it's a case-sensitive marker -- strip the `-` and use as-is
4. Otherwise, lowercase the event name
5. Convert camelCase to kebab-case (uppercase letters become `-` + lowercase)
6. Special case: `DOMContentLoaded` becomes `d-o-m-content-loaded`

**For components (`is_fn = true`):** Event names are NOT converted. They are processed through `convert_qrl_word` which replaces the `$` suffix with `Qrl` (e.g., `onClick$` -> `onClickQrl`). The arrow/function expression values are still extracted as QRL segments.

**`host:` prefix:** Event handlers prefixed with `host:` (e.g., `host:onClick$`, `host:onDocumentScroll$`) are kept as-is -- they are NOT converted to `q-e:` format. The `host:` prefix tells the runtime to register the handler on the host element directly.

**Handler merging:** When multiple handlers share the same converted key (e.g., two `onDocumentScroll$` handlers), they are merged into an array (transform.rs:1670-1742).

### bind:value and bind:checked Sugar

**Source:** transform.rs:1779-1862

`bind:value` and `bind:checked` are syntactic sugar that expand into two props:

For `bind:value={signal}`:
- A `value` prop with the signal reference in const_props
- A `q-e:input` event handler: `inlinedQrl(_val, "_val", [signal])` in const_props

For `bind:checked={checked}`:
- A `checked` prop with the signal reference in const_props
- A `q-e:input` event handler: `inlinedQrl(_chk, "_chk", [signal])` in const_props

**Behavior differs by branch:**
- **`_jsxSorted` path (no spreads):** bind:value/checked are expanded (value prop + handler)
- **`_jsxSplit` path (spreads exist):** bind:value/checked in var_props are left untouched; the runtime handles expansion. Only bind props targeting const_props are still expanded.
- **Components with bind:* props:** Force `_jsxSplit` (transform.rs:1872-1886, 2186-2188). The component_has_bind_props check triggers `should_runtime_sort = true`.

**Merging with existing handlers:** If `q-e:input` already exists (e.g., from an explicit `onInput$` handler), the bind handler is merged into an array with the existing handler (transform.rs:1852-1856, using `merge_or_add_event_handler`).

### Children Handling

**Source:** transform.rs:2327-2366

Children are extracted from the props object and placed as the 4th argument to `_jsxSorted`/`_jsxSplit`:

- **Single child:** Passed directly as the 4th argument
- **Array of children:** Passed as a JS array literal
- **No children:** 4th argument is `null`
- **Text-only elements:** Children forced to be treated as raw content (the `is_text_only` flag applies to `text`, `textarea`, `title`, `option`, `script`, `style`, `noscript`)
- **Children in spread context:** When spread props exist, children that appear before a spread go to var_props as a regular `children` key-value prop instead of being extracted to the 4th argument (transform.rs:2354-2364)

**Children signal optimization:** Child expressions that reference signals are processed through `convert_to_signal_item` which may wrap them in `_fnSignal` or `_wrapProp`.

### Key Generation

**Source:** transform.rs:1182-1197

Keys serve as stable identity markers for JSX elements:

**Explicit keys:** If the original `jsx()` call has a 3rd argument (explicit key), it is used as-is.

**Auto-generated keys:** When no explicit key exists AND the element should emit a key:
- Format: `"{base64_prefix}_{counter}"` where:
  - `base64_prefix` = first 2 characters of `base64(file_hash)` (transform.rs:4725-4728)
  - `counter` = `jsx_key_counter`, incremented for each key generated in the file
- Example: `"u6_0"`, `"u6_1"`, `"u6_2"`

**When keys are emitted:**
- Components (`is_fn = true`): Always get a key
- Root-level JSX (`root_jsx_mode = true`): Gets a key
- Nested native elements in non-root position: Key is `null`

The `root_jsx_mode` flag is set to `true` initially and reset to `false` after the first JSX element is processed (transform.rs:1176-1177). It is restored after processing (transform.rs:1218).

### Flag Computation

**Source:** transform.rs:2613-2622

The 5th argument is a numeric flags bitmask:

| Bit | Value | Name | Meaning |
|-----|-------|------|---------|
| 0 | 1 | `static_listeners` | All event handlers are const (no captured mutable state) |
| 1 | 2 | `static_subtree` | Children subtree is fully static |
| 2 | 4 | `moved_captures` | Element has `q:p` or `q:ps` props (lifted iteration variables) |

Common flag values:
- `3` = static_listeners + static_subtree (fully static element)
- `1` = static_listeners only (dynamic children)
- `0` = fully dynamic

**Flag defaults change with spreads:** When spreads exist, `static_listeners` and `static_subtree` both start as `false` (transform.rs:2191-2192).

### Spread Props Handling

**Source:** transform.rs:2570-2602, 2655-2700

When a JSX element has spread props (`{...expr}`), special handling occurs:

**For identifier-based spreads** (e.g., `{...props}`):
- The spread is split using `_getVarProps(props)` and `_getConstProps(props)` helpers (imported from `@qwik.dev/core`)
- `_getVarProps(ident)` spread goes to var_props
- `_getConstProps(ident)` spread goes to const_props (or var_props if multiple spreads or var props follow the last spread)

**For non-identifier spreads** (e.g., `{...getProps()}`):
- The entire spread goes to var_props as-is

**Multiple spreads:** When multiple spreads exist, all const_props spreads after the first spread go to var_props to maintain correct override order (transform.rs:2587-2595).

### Iteration Variable Lifting (q:p / q:ps)

**Source:** transform.rs:2196-2265, 2430-2476

When event handlers inside JSX elements capture loop variables or external variables:

1. **In loop context:** Iteration variables from `iteration_var_stack` that are used by event handlers are collected
2. **Outside loop context:** The union of all captures from all event handlers on the element is computed via `compute_handler_captures`
3. These variables are injected as extra parameters to the event handler functions
4. A `q:p` (single variable) or `q:ps` (multiple variables) prop is added to var_props
5. Props that reference iteration variables are moved to var_props even if they would otherwise be const

**q:p vs q:ps:**
- `q:p`: Single captured variable, value is the variable directly
- `q:ps`: Multiple captured variables, value is an array of the variables

## Props Destructuring -- Behavioral Rules Extracted from SWC

### Core Mechanism

**Source:** props_destructuring.rs:22-37 (`transform_props_destructuring`)

Props destructuring is a **pre-pass** that runs at Step 8 in the pipeline -- BEFORE the main QwikTransform (Step 10). This ordering is critical because it changes variable references that capture analysis later reads.

**Trigger condition:** The transform looks for `component$()` calls and arrow functions that:
1. Have exactly one parameter
2. That parameter is an object destructuring pattern
3. The function body is either a block statement with a return, or a single expression

**What it does:** Converts `({prop1, prop2, ...rest}) => { ... }` to `(_rawProps) => { const rest = _restProps(_rawProps, ["prop1", "prop2"]); ... }` where all references to `prop1` are replaced with `_rawProps.prop1`.

### _rawProps Replacement

**Source:** props_destructuring.rs:62-88

When a destructured parameter is detected:
1. A new identifier `_rawProps` is created
2. Each destructured prop binding becomes a member access: `prop` -> `_rawProps.prop`
3. Renamed props (`{ count: c }`) use the original prop name for access: `c` -> `_rawProps.count`
4. Default values with const expressions use nullish coalescing: `{ x = 5 }` -> `_rawProps.x ?? 5`
5. Default values with non-const expressions cause the transform to bail out (skip)
6. The replacements are stored in an `identifiers: HashMap<Id, Expr>` map
7. All subsequent references to the original identifiers in the function body are replaced via `visit_mut_expr`

### _restProps() Handling

**Source:** props_destructuring.rs:480-522 (`transform_rest`)

When a rest pattern exists (`{...rest}`):
- `_restProps` is imported from `@qwik.dev/core`
- A const declaration is inserted at the top of the function body:
  - With exclusions: `const rest = _restProps(_rawProps, ["prop1", "prop2", ...])`
  - Without exclusions (rest-only pattern): `const rest = _restProps(_rawProps)`
- The exclusion list contains the **original prop names** (not renamed aliases)

**Rest-only destructuring** (`({...props}) => {}`):
- Transforms to: `(_rawProps) => { const props = _restProps(_rawProps); ... }`
- No exclusion array is passed

### Inlining Optimizations in Component Body

**Source:** props_destructuring.rs:89-284 (`transform_component_body`)

After the parameter transformation, the function body is scanned for `const` declarations that can be further inlined:

1. **Literal initializers:** `const x = 5` -> removed (x is inlined at use sites as the replacement expression)
2. **Member access on known identifiers:** `const y = _rawProps.something.count` -> inlined
3. **Computed member access:** `const z = obj['key']` -> inlined
4. **`use*` hook results with member access:** `const x = useStore({}).thing` -> kept as `const store = useStore({}); ... store.thing`

**Skip condition:** If the function body starts with a `_captures` destructuring (indicating pre-compiled library code from Lib mode), the entire body transformation is skipped (transform.rs:93-106).

**Unused declaration cleanup:** After inlining, declarations like `const _unused;` (with no init) are removed to avoid invalid JS output (transform.rs:274-283).

### Shorthand Property Handling

**Source:** props_destructuring.rs:347-357

When props are used in shorthand object properties (e.g., `{name}` in JSX), the shorthand is expanded to a key-value pair: `{name}` -> `{name: _rawProps.name}`.

## Signal Optimization -- Behavioral Rules Extracted from SWC

### Core Mechanism: create_synthetic_qqsegment

**Source:** transform.rs:737-817

This is the gateway function for signal optimization. Called from `convert_to_getter` (for JSX props) and `convert_to_signal_item` (for children). It determines whether an expression should be wrapped in `_fnSignal` or `_wrapProp`.

**Decision flow:**

1. Collect all identifiers used in the expression (`IdentCollector`)
2. Partition declaration stack into scope variables (`Var`) and invalid declarations
3. Check for disqualifying conditions:
   - If any identifier is a **global** (import/external) -> `contains_side_effect = true`
   - If any identifier is an **invalid declaration** (function, class) -> bail out, return `(None, false)`
   - If any identifier is **not in declaration stack at all** -> bail out, return `(None, false)`
4. Compute scoped identifiers and const-ness
5. If `contains_side_effect` -> bail out, return `(None, scoped_idents.is_empty())`
6. If expression is a simple `Ident` -> bail out, return `(None, is_const)` -- no wrapping needed
7. If expression is `Call` or `Tpl` (template literal) and not const -> bail out, return `(None, false)`
8. **Member expression on ident** (e.g., `signal.value`): use `_wrapProp(signal)` -- a simpler wrapper
9. **Everything else:** attempt `convert_inlined_fn` to generate `_fnSignal`

### _wrapProp: Simple Signal Property Access

**Source:** transform.rs:791-804

When the expression is `ident.prop` (member expression where object is a simple identifier):
- Generates: `_wrapProp(ident)` (without the `.value` -- just the signal object)
- The property name is extracted but not used in the simple form
- Works for `signal.value`, `store.address`, etc.

**Note on _wrapProp without property argument:** Looking at the snapshot evidence, `_wrapProp(signal)` is the standard form -- it wraps the signal object itself, and the runtime knows to access `.value`. The `_wrapProp(obj, "propName")` form (with two arguments) is used for props destructuring access patterns (e.g., `_wrapProp(_rawProps, "id")`).

### _fnSignal: Computed Expression Wrapping

**Source:** inlined_fn.rs:24-115 (`convert_inlined_fn`)

When `create_synthetic_qqsegment` reaches step 9, it calls `convert_inlined_fn` which attempts to create a `_fnSignal()` call.

**Bail-out conditions (returns None):**
1. Expression is an `ArrowExpr` -> bail out (arrow functions are not optimized this way)
2. Expression is used as a **call** (contains a `CallExpr` node) -> bail out, mark as non-const
3. Expression identifiers are NOT used as **objects** (no member access on them) -> bail out
4. After identifier replacement, the `ReplaceIdentifiers` visitor sets `abort = true` if it encounters:
   - A nested `ArrowExpr`
   - A nested `Function`
   - A nested `ClassExpr`
   - A `Decorator`
   - A `Stmt` (statement node)
   - A `Callee` that is an `Import` or when `accept_call_expr = false`
5. Rendered expression exceeds 150 characters -> bail out (transform.rs:67-70)
6. `scoped_idents` is empty (no captured variables) -> return `(None, true)` -- expression is const, no need to wrap

**Successful generation:**

When none of the bail-out conditions trigger:
1. Create positional parameter names: `p0`, `p1`, `p2`, ... (one per scoped identifier)
2. Replace all captured identifier references with positional params
3. Render the transformed expression to a minified string (the "stringified version")
4. Wrap in an arrow function: `(p0, p1) => <expression with p0, p1>`
5. Generate the call: `_fnSignal(arrowFn, [capturedVar0, capturedVar1], "stringified")`

**Arguments to _fnSignal:**
1. Arrow function with positional params
2. Array of captured variable references (the actual runtime values)
3. (Optional, when `serialize_fn` is true / server mode) Stringified version of the expression for SSR

### _fnSignal Hoisting

**Source:** transform.rs:2763-2872 (`hoist_fn_signal_call`)

After `_fnSignal` is generated, the arrow function (1st argument) and stringified expression (3rd argument) are hoisted to module-level const declarations for deduplication:

- Arrow function: `const _hf0 = (p0) => 12 + p0.value;`
- Stringified version: `const _hf0_str = "12+p0.value";`
- The `_fnSignal` call is rewritten to reference these: `_fnSignal(_hf0, [signal], _hf0_str)`
- Counter-based naming: `_hf0`, `_hf1`, `_hf2`, etc.
- Deduplication: If the same arrow function body has been seen before, the existing hoisted reference is reused

### Decision Table (D-21)

This table covers all expression type x context combinations for signal optimization:

| Expression | Captured Vars | Side Effects | Result | Example |
|-----------|--------------|--------------|--------|---------|
| Simple identifier | any | any | No wrapping | `signal` -> `signal` |
| `ident.prop` (member on ident) | yes | no | `_wrapProp(ident)` | `signal.value` -> `_wrapProp(signal)` |
| `ident.prop` (member on ident) | yes | yes (global ident) | No wrapping | `globalThing.thing` -> `globalThing.thing` |
| Computed expression | yes, non-empty | no | `_fnSignal(fn, [...])` | `12 + signal.value` -> `_fnSignal(_hf0, [signal], _hf0_str)` |
| Computed expression | empty (all const) | no | No wrapping, mark const | `1 + 2` -> `1 + 2` (const) |
| Computed expression | yes | yes (contains global) | No wrapping | `signal.value + globalFn()` -> as-is |
| Arrow/function expression | any | any | No wrapping (bail out) | `() => {}` -> as-is |
| Call expression | any | any | No wrapping, mark non-const | `fn()` -> `fn()` |
| Template literal (non-const) | yes | no | No wrapping, mark non-const | `` `${signal.value}` `` -> as-is |
| Expression > 150 chars rendered | yes | no | No wrapping, mark non-const | Long expression -> as-is |
| Ternary / binary | yes, non-empty | no | `_fnSignal(fn, [...])` | `a.value ? 'yes' : 'no'` -> `_fnSignal(...)` |
| Ternary containing call | yes | no | No wrapping (abort: callee) | `a.value ? fn() : b` -> as-is |

**Context-dependent behavior:**
- In `convert_to_getter` (JSX props): `accept_call_expr = true` -- call expressions within the expression are allowed (but `is_used_as_call` still bails out)
- In `convert_to_signal_item` (children): `accept_call_expr = false` -- stricter, call expressions cause abort

### _wrapProp with Props Destructuring

When props are destructured and inlined to `_rawProps.propName`, signal optimization sees this as a member access on `_rawProps` and generates `_wrapProp(_rawProps, "propName")` -- the two-argument form. This preserves signal reactivity tracking for component props.

Example from `should_destructure_args` snapshot:
```
Input:  ({ message, id, count: c }) => { return <div id={id}> ... }
Output: (_rawProps) => { return <div id={_wrapProp(_rawProps, "id")}> ... }
```

## Relevant Snapshot Catalog

### JSX Transform Snapshots (~40 files)

**Core JSX:**
- `example_jsx.snap` -- Basic JSX with fragments, nested elements, spread props on component
- `example_jsx_keyed.snap` -- Explicit key handling
- `example_jsx_keyed_dev.snap` -- Key + dev mode (7th argument)
- `example_jsx_listeners.snap` -- Full event name conversion catalog (onClick$, document:, window:, host:, case-sensitive)
- `example_jsx_import_source.snap` -- JSX import source handling
- `special_jsx.snap` -- Special JSX patterns

**Spread Props:**
- `should_split_spread_props.snap` -- Basic spread -> _jsxSplit + _getVarProps/_getConstProps
- `should_split_spread_props_with_additional_prop.snap` through `...5.snap` -- Spread with additional props (5 variations)
- `should_merge_attributes_with_spread_props.snap` -- Spread + regular attributes
- `should_merge_attributes_with_spread_props_before_and_after.snap` -- Props before and after spread

**Class/ClassName:**
- `example_class_name.snap` -- className -> class normalization (native vs component behavior)

**bind:value/checked:**
- `example_input_bind.snap` -- Full bind:value/checked expansion with _val/_chk
- `should_merge_bind_value_and_on_input.snap` -- bind:value merged with explicit onInput$
- `should_merge_bind_checked_and_on_input.snap` -- bind:checked merged with explicit onInput$
- `should_merge_on_input_and_bind_value.snap` -- Order: onInput$ first, then bind:value
- `should_merge_on_input_and_bind_checked.snap` -- Order: onInput$ first, then bind:checked
- `should_make_component_jsx_split_with_bind.snap` -- Component + bind forces _jsxSplit
- `should_not_transform_bind_value_in_var_props_for_jsx_split.snap` -- _jsxSplit leaves bind in var_props
- `should_not_transform_bind_checked_in_var_props_for_jsx_split.snap` -- Same for checked
- `should_move_bind_value_to_var_props.snap` -- bind in spread context moves to var_props

**Signal Optimization:**
- `example_derived_signals_div.snap` -- Full _wrapProp/_fnSignal showcase (16+ prop types)
- `example_derived_signals_cmp.snap` -- Component prop optimization
- `example_derived_signals_children.snap` -- Children signal optimization
- `example_derived_signals_complext_children.snap` -- Complex children patterns
- `example_derived_signals_multiple_children.snap` -- Multiple children with signals
- `example_getter_generation.snap` -- Getter generation patterns
- `example_immutable_analysis.snap` -- Immutability detection
- `example_immutable_function_components.snap` -- Immutable function components
- `example_props_wrapping.snap` / `example_props_wrapping2.snap` -- Props wrapping patterns
- `example_props_wrapping_children.snap` / `...2.snap` -- Props wrapping in children
- `example_props_optimization.snap` -- Props optimization patterns
- `should_wrap_object_with_fn_signal.snap` -- Object expression _fnSignal wrapping
- `should_wrap_logical_expression_in_template.snap` -- Logical expressions
- `should_wrap_store_expression.snap` -- Store expression wrapping
- `should_wrap_type_asserted_variables_in_template.snap` -- Type assertion handling
- `should_wrap_prop_from_destructured_array.snap` -- Array destructuring
- `should_wrap_inner_inline_component_prop.snap` -- Inner component prop wrapping
- `should_not_wrap_fn.snap` -- Arrow function bail-out
- `should_not_wrap_ternary_function_operator_with_fn.snap` -- Ternary with function
- `should_not_wrap_var_template_string.snap` -- Template string bail-out
- `ternary_prop.snap` -- Ternary expression optimization
- `hoisted_fn_signal_in_loop.snap` -- _fnSignal hoisting in loop context
- `lib_mode_fn_signal.snap` -- _fnSignal in lib mode
- `transform_qrl_in_regular_prop.snap` -- QRL in regular prop context

**Props Destructuring:**
- `should_destructure_args.snap` -- Full destructuring + _restProps + _wrapProp interaction
- `should_convert_rest_props.snap` -- Rest-only destructuring pattern
- `destructure_args_colon_props.snap` through `...3.snap` -- Renamed prop destructuring (3 variations)
- `destructure_args_inline_cmp_block_stmt.snap` / `...2.snap` -- Inline component block statement
- `destructure_args_inline_cmp_expr_stmt.snap` -- Inline component expression
- `should_not_generate_conflicting_props_identifiers.snap` -- Name collision avoidance
- `example_functional_component_capture_props.snap` -- Props capture interaction

**Loop/Iteration:**
- `example_component_with_event_listeners_inside_loop.snap` -- Loop iteration variable lifting
- `should_move_props_related_to_iteration_variables_to_var_props.snap` -- Iteration vars force var_props

**Event Conversion:**
- `should_convert_jsx_events.snap` -- JSX event conversion patterns
- `should_transform_event_names_without_jsx_transpile.snap` -- Event names without JSX transpile

## Common Pitfalls

### Pitfall 1: Confusing `should_sort` with `_jsxSorted`
**What goes wrong:** The SWC code variable `should_sort` is true when runtime sorting is needed, which means `_jsxSplit`. When `should_sort = false`, compile-time sorting is used, which means `_jsxSorted`. This is counterintuitive.
**How to avoid:** The spec should use clear terminology: "compile-time sorted path (_jsxSorted)" vs "runtime-sorted path (_jsxSplit)". Avoid using the internal variable name `should_sort` in behavioral rules.

### Pitfall 2: bind:* Behavior Varies by Context
**What goes wrong:** bind:value expansion (to value + q-e:input handler) only happens in certain contexts. On native elements without spreads it expands. On native elements with spreads, only const_props-targeted bind props expand. On components, bind props force _jsxSplit but don't expand at all.
**How to avoid:** Document the context-dependent behavior explicitly in the bind:* subsection with a truth table.

### Pitfall 3: Props Destructuring Must Run Before Capture Analysis
**What goes wrong:** If capture analysis runs on the original destructured parameter, it sees different variable names than the transformed `_rawProps.propName` accesses.
**How to avoid:** The spec must clearly state the ordering requirement and cross-reference the Phase 1 pipeline ordering.

### Pitfall 4: _fnSignal 150-Character Limit
**What goes wrong:** Expressions that render to >150 characters in minified form silently fall back to non-optimized (non-reactive) mode. This is not an error -- it's by design, but can cause subtle reactivity bugs.
**How to avoid:** Document this limit explicitly with a note about the behavioral consequence (loss of fine-grained reactivity tracking).

### Pitfall 5: _wrapProp Two Forms
**What goes wrong:** `_wrapProp(signal)` (1-arg form for signal.value access) and `_wrapProp(_rawProps, "propName")` (2-arg form for destructured props) look similar but serve different purposes.
**How to avoid:** Document both forms explicitly with when each is generated.

### Pitfall 6: Event Handler Host Prefix
**What goes wrong:** `host:onClick$` handlers are NOT converted to `q-e:` format. They stay as `host:onClick$` in the output. This is easy to miss since all other event handlers are converted.
**How to avoid:** Explicitly catalog `host:` as a non-converting prefix in the event name conversion rules.

## Code Examples

### JSX Transform: _jsxSorted (No Spreads)

Source: `example_derived_signals_div.snap`

```javascript
// INPUT
<div
  staticText="text"
  signalValue={signal.value}
  signalComputedValue={12 + signal.value}
  store={store.address.city.name}
/>

// OUTPUT
_jsxSorted("div", {
  // var_props (dynamic)
}, {
  // const_props (static)
  staticText: "text",
  signalValue: _wrapProp(signal),
  signalComputedValue: _fnSignal(_hf0, [signal], _hf0_str),
  store: _fnSignal(_hf1, [store], _hf1_str),
}, null, 3, "u6_0")

// Hoisted declarations:
const _hf0 = (p0) => 12 + p0.value;
const _hf0_str = "12+p0.value";
const _hf1 = (p0) => p0.address.city.name;
const _hf1_str = "p0.address.city.name";
```

### JSX Transform: _jsxSplit (With Spreads)

Source: `should_split_spread_props.snap`

```javascript
// INPUT
<div {...props}></div>

// OUTPUT
_jsxSplit("div", {
  ..._getVarProps(props)
}, _getConstProps(props), null, 0, "u6_0")
```

### Props Destructuring: _rawProps + _restProps

Source: `should_destructure_args.snap`

```javascript
// INPUT
component$(({ message, id, count: c, ...rest }) => {
  return <div id={id}><span {...rest}>{message} {c}</span></div>;
})

// OUTPUT (inside segment)
(_rawProps) => {
  const rest = _restProps(_rawProps, ["message", "id", "count"]);
  return _jsxSorted("div", {
    id: _wrapProp(_rawProps, "id")
  }, null, [
    _jsxSplit("span", {
      ..._getVarProps(rest)
    }, _getConstProps(rest), [
      _wrapProp(_rawProps, "message"), " ", _wrapProp(_rawProps, "count")
    ], 0, null),
  ], 1, "u6_0");
}
```

### bind:value Expansion

Source: `example_input_bind.snap`

```javascript
// INPUT
<input bind:value={value} />
<input bind:checked={checked} />
<input bind:stuff={stuff} />

// OUTPUT
_jsxSorted("input", null, {
  "value": value,
  "q-e:input": inlinedQrl(_val, "_val", [value])
}, null, 3, null)

_jsxSorted("input", null, {
  "checked": checked,
  "q-e:input": inlinedQrl(_chk, "_chk", [checked])
}, null, 3, null)

// Unrecognized bind:stuff -- kept as-is
_jsxSorted("input", null, {
  "bind:stuff": stuff
}, null, 3, null)
```

### className Normalization

Source: `example_class_name.snap`

```javascript
// INPUT
<div className="hola" />
<div className={signal.value} />
<Foo className="hola" />
<Foo className={signal.value} />

// OUTPUT -- native elements: className -> class
_jsxSorted("div", null, { "class": "hola" }, null, 3, null)
_jsxSorted("div", null, { "class": _wrapProp(signal) }, null, 3, null)

// OUTPUT -- components: className kept as-is
_jsxSorted(Foo, null, { className: "hola" }, null, 3, "u6_0")
_jsxSorted(Foo, null, { className: _wrapProp(signal) }, null, 3, "u6_1")
```

## Validation Architecture

### Test Framework
| Property | Value |
|----------|-------|
| Framework | Manual spec review (no automated testing -- this is a specification document, not code) |
| Config file | N/A |
| Quick run command | Visual review of spec sections |
| Full suite command | Cross-reference spec rules against SWC snapshot outputs |

### Phase Requirements -> Test Map
| Req ID | Behavior | Test Type | Automated Command | File Exists? |
|--------|----------|-----------|-------------------|-------------|
| SPEC-04 | Props destructuring rules documented | manual-only | Review spec section against `props_destructuring.rs` and 7+ destructuring snapshots | N/A |
| SPEC-06 | JSX transform rules documented | manual-only | Review spec section against `transform.rs` JSX functions and 40+ JSX snapshots | N/A |
| SPEC-07 | Signal optimization rules documented | manual-only | Review spec section against `inlined_fn.rs`, `create_synthetic_qqsegment`, and 20+ signal snapshots | N/A |

### Sampling Rate
- **Per task commit:** Visual review of new spec sections for accuracy
- **Per wave merge:** Cross-reference all rules against SWC source line numbers
- **Phase gate:** All 3 CONV sections present with rules + examples + snapshot references

### Wave 0 Gaps
None -- existing spec document infrastructure from Phase 1 is sufficient.

## Sources

### Primary (HIGH confidence)
- SWC `transform.rs` -- JSX handling functions (handle_jsx, handle_jsx_props_obj, internal_handle_jsx_props_obj, transform_jsx_prop, jsx_event_to_html_attribute, create_event_name, build_children, build_flags, convert_to_getter, create_synthetic_qqsegment) -- lines 737-2900, 4639-4728
- SWC `props_destructuring.rs` -- Complete file (568 LOC): transform_props_destructuring, transform_component_props, transform_component_body, transform_pat, transform_rest, create_omit_props
- SWC `inlined_fn.rs` -- Complete file (294 LOC): convert_inlined_fn, render_expr, ReplaceIdentifiers, ObjectUsageChecker
- SWC `words.rs` -- Complete file: all constant definitions (_JSX_SORTED, _JSX_SPLIT, _WRAP_PROP, _INLINED_FN, BIND_VALUE, BIND_CHECKED, CLASS_NAME, CLASS, REF, QSLOT, CHILDREN, _REST_PROPS, _GET_VAR_PROPS, _GET_CONST_PROPS, _CHK, _VAL, ON_INPUT)
- Jack's SWC snapshots (201 files) -- Verified input/output pairs for all behavioral rules

### Secondary (MEDIUM confidence)
- FEATURES.md -- CONV-04, CONV-06, CONV-07 descriptions and dependency graph
- Phase 1 spec document -- Existing Pipeline Overview, Capture Analysis cross-references

## Metadata

**Confidence breakdown:**
- JSX Transform: HIGH -- Complete source code analysis of all JSX functions, verified against 40+ snapshots
- Props Destructuring: HIGH -- Complete source code analysis of props_destructuring.rs, verified against 7+ snapshots
- Signal Optimization: HIGH -- Complete source code analysis of inlined_fn.rs + create_synthetic_qqsegment, verified against 20+ snapshots
- Decision table (D-21): HIGH -- All combination rules derived from source code control flow

**Research date:** 2026-04-01
**Valid until:** Stable -- SWC source is frozen (Qwik v2 build/v2 branch). No expected changes.
