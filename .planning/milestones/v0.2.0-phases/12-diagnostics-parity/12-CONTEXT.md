# Phase 12: Diagnostics Parity - Context

**Gathered:** 2026-04-06 (assumptions mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Error diagnostics match SWC for all 201 fixtures -- same errors reported for invalid references and missing custom inlined functions. This phase fixes 9 diagnostic mismatches (8 false positives + 1 false negative). It does NOT address root module code content (Phase 11, complete), segment code content (out of scope), or final acceptance verification (Phase 13).

Current baseline: 192/201 (95%) diagnostics match. Target: 201/201.

</domain>

<decisions>
## Implementation Decisions

### C02 False Positive Suppression
- **D-01:** Add the export-symbol gate to OXC's C02 (FunctionReference) diagnostic check. SWC only emits C02 when `!global_collect.has_export_symbol(&id.0)` (SWC transform.rs:1024), but OXC unconditionally emits C02 for any function/class in `decl_stack` (OXC transform.rs:905-923). The `has_export_symbol` method already exists in `collector.rs:139`.
- **D-02:** Some false positives in non-default modes (Lib, Hoist) may also be resolved by the export-symbol gate, since SWC checks `should_emit_segment` before entering the C02 loop. If the gate alone doesn't fix mode-specific fixtures, add mode-aware suppression as a follow-up.

### C05 Missing Implementation
- **D-03:** Implement the C05 (MissingQrlImplementation) diagnostic. SWC checks whether a locally-defined `$`-suffixed function has a corresponding `Qrl`-suffixed export in the same file (SWC transform.rs:4078-4088). OXC has zero C05 implementation. Jack's conversion includes C05 tests (lib.rs:1768-1792).
- **D-04:** The `example_missing_custom_inlined_functions` fixture is the known C05 failure. This fixture may also have a segment count mismatch (exp=0, act=1) that gets fixed when C05 prevents the invalid segment extraction.

### Diagnostic Comparison Granularity
- **D-05:** Phase 12 targets error presence/absence parity only (boolean match). The parity comparison at `snapshot_tests.rs:1150-1156` checks whether both SWC and OXC agree on "has errors" vs "no errors". Exact diagnostic message text, error codes, and span highlights are NOT required for Phase 12 parity.

### Scope Update
- **D-06:** The actual mismatch count is 9 (not 4 as originally estimated in the roadmap). 8 are false positives (OXC emits errors where SWC doesn't) and 1 is a false negative (OXC misses C05 where SWC emits it).

### Claude's Discretion
- Order of implementation (C02 gate first vs C05 first)
- Whether to batch all C02 false positives into one fix or handle mode-specific cases separately
- Exact placement of C05 check in the OXC transform pipeline

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### OXC Implementation (current codebase)
- `crates/qwik-optimizer-oxc/src/transform.rs` -- C02 diagnostic emission at lines 905-923, `decl_stack` checks, `is_inline_mode()` at 926-930
- `crates/qwik-optimizer-oxc/src/collector.rs` -- `has_export_symbol` at line 139, `GlobalCollect` export tracking
- `crates/qwik-optimizer-oxc/src/types.rs` -- `DiagnosticCategory` enum, diagnostic types
- `crates/qwik-optimizer-oxc/tests/snapshot_tests.rs` -- Parity comparison logic at lines 1150-1156, diagnostic presence/absence check

### SWC Reference Implementation
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` -- C02 with export-symbol gate at line 1024, `should_emit_segment` before C02 loop at line 1021, C05 MissingQrlImplementation at lines 4072-4088
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/collector.rs` -- SWC's `has_export_symbol` implementation

### Jack's OXC Conversion
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/transform.rs` -- Jack's diagnostic handling approach
- `/Users/scottweaver/Projects/qwik-oxc-optimizer/crates/qwik-optimizer-oxc/src/lib.rs` -- C05 tests at lines 1768-1792

### Test Fixtures (known failures)
- `crates/qwik-optimizer-oxc/fixtures.json` -- Fixture configurations including mode/strategy settings for affected fixtures
- `crates/qwik-optimizer-oxc/tests/swc_expected/` -- SWC reference outputs for diagnostic comparison

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `has_export_symbol()` in `collector.rs:139` -- Already implemented, just needs to be called in the C02 check
- `DiagnosticCategory` enum in `types.rs` -- Existing diagnostic type system, needs C05 variant if not present
- Parity test infrastructure in `snapshot_tests.rs` -- Automated comparison against all 201 fixtures

### Established Patterns
- Diagnostics are emitted via `self.diagnostics.push()` during transform traversal
- C02 checks happen in `enter_call_expression` when processing dollar-sign calls
- SWC uses a two-gate pattern: first check `should_emit_segment`, then check `has_export_symbol`

### Integration Points
- C02 fix: Modify existing `enter_call_expression` in `transform.rs` to add export-symbol gate
- C05 new: Add check during call expression processing when callee is a `$`-suffixed local function
- Both changes integrate into the existing `Traverse` visitor pattern

</code_context>

<specifics>
## Specific Ideas

No specific requirements -- open to standard approaches

</specifics>

<deferred>
## Deferred Ideas

None -- analysis stayed within phase scope

</deferred>

---

*Phase: 12-diagnostics-parity*
*Context gathered: 2026-04-06*
