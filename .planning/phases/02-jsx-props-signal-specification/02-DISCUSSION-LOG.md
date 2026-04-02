# Phase 2: JSX, Props & Signal Specification - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-04-01
**Phase:** 02-jsx-props-signal-specification
**Areas discussed:** JSX transform scope, Props destructuring ordering, Signal optimization boundaries, Example selection for JSX

---

## JSX Transform Scope

| Option | Description | Selected |
|--------|-------------|----------|
| Subsectioned | Break into subsections: Element, Prop Classification, Special Attrs, Children, Key Gen | ✓ |
| Flat with rules | One section with long rules list | |
| By output function | Organize around _jsxSorted/_jsxSplit/_jsxC output | |

**User's choice:** Subsectioned

| Option | Description | Selected |
|--------|-------------|----------|
| Branching condition | One mechanism with branch: _jsxSorted (no spread) vs _jsxSplit (spread) | ✓ |
| Separate paths | Two parallel sections for each function | |
| You decide | | |

**User's choice:** Branching condition

---

## Props Destructuring Ordering

| Option | Description | Selected |
|--------|-------------|----------|
| Cross-reference to Phase 1 | Reference Capture Analysis section, focus on _rawProps | |
| Inline recap | Brief summary of capture interaction, self-contained | |
| You decide | Claude picks to avoid redundancy | ✓ |

**User's choice:** You decide (Claude's Discretion)

| Option | Description | Selected |
|--------|-------------|----------|
| Full behavior | Document exact exclusion logic for _restProps() with examples | ✓ |
| Brief mention | Just note _restProps() exists | |
| You decide | | |

**User's choice:** Full behavior

---

## Signal Optimization Boundaries

| Option | Description | Selected |
|--------|-------------|----------|
| Decision table | Expression type × prop context × captured variables → _fnSignal or plain | ✓ |
| Rule-based | If/then rules in narrative style | |
| You decide | | |

**User's choice:** Decision table

| Option | Description | Selected |
|--------|-------------|----------|
| Both _wrapProp and _fnSignal | Same section, covers both signal optimization mechanisms | ✓ |
| _fnSignal only | _wrapProp described as part of JSX prop classification | |
| You decide | | |

**User's choice:** Both

---

## Example Selection for JSX

| Option | Description | Selected |
|--------|-------------|----------|
| Up to 4 per subsection | Extra examples for combinatorial space | |
| Stick with 2-3 | Consistent with Phase 1 convention | |
| You decide per subsection | Adapt to subsection complexity | ✓ |

**User's choice:** You decide per subsection (Claude's Discretion)

| Option | Description | Selected |
|--------|-------------|----------|
| List recommended snapshots | Each subsection ends with test vector list | |
| Just inline examples | Spec examples are self-sufficient | |
| Both | Inline examples + "See also" snapshot list | ✓ |

**User's choice:** Both — inline examples plus "See also" snapshot references

---

## Claude's Discretion

- Props destructuring ↔ capture analysis interaction format (D-19)
- Example count per JSX subsection (D-23)

## Deferred Ideas

None — discussion stayed within phase scope
