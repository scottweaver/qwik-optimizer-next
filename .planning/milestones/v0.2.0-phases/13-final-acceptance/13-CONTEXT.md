# Phase 13: Final Acceptance - Context

**Gathered:** 2026-04-06 (auto mode)
**Status:** Ready for planning

<domain>
## Phase Boundary

Full SWC parity achieved -- 201/201 fixtures match on root module, segment count, and diagnostics. This phase diagnoses and fixes remaining root module mismatches (122 fixtures), remaining segment count mismatches (6 fixtures), and verifies the final 201/201 target. It does NOT add new transformation types, change the parity comparison criteria, or address segment code content parity (out of scope per REQUIREMENTS.md).

Current baseline: 79/201 (39%) full match | 195/201 segment count | 201/201 diagnostics.
Target: 201/201 full match on all three dimensions.

</domain>

<decisions>
## Implementation Decisions

### Acceptance Threshold
- **D-01:** Exact match required for all 201 fixtures. The milestone goal (ACC-01) is 201/201 full match. No cosmetic exceptions or "close enough" waivers.
- **D-02:** The existing `parity_report` test in `snapshot_tests.rs` is the acceptance gate. When it reports `Full match: 201/201`, the milestone is complete.

### Straggler Triage Strategy
- **D-03:** Diagnose mismatches by category (root cause grouping), then batch fix by shared root cause. Most of the 122 root mismatches likely share a handful of root causes (e.g., import ordering, dead code elimination, expression rewriting patterns). Fix the biggest categories first for maximum impact.
- **D-04:** The segment mismatch `should_preserve_non_ident_explicit_captures` (exp=1, act=0) must be investigated separately -- it's a segment extraction bug, not a root module issue.
- **D-05:** After each batch fix, re-run the parity report to track progress and detect regressions.

### Verification Approach
- **D-06:** Use `cargo test -p qwik-optimizer-oxc --test snapshot_tests parity_report -- --nocapture` as the primary verification tool throughout the phase. No additional test infrastructure needed.
- **D-07:** All existing tests (`cargo test -p qwik-optimizer-oxc`) must continue to pass after each fix (no regressions).

### Phase Structure
- **D-08:** This phase may require multiple planning iterations. Start with diagnosis to understand the root cause categories, then plan targeted fixes. The plan count is TBD because the scope depends on how many distinct root causes exist.

### Claude's Discretion
- Ordering of root cause categories to fix
- Whether to split fixes into multiple plans or handle in a single large plan
- Level of detail in diagnostic analysis (aggregate vs per-fixture)

</decisions>

<canonical_refs>
## Canonical References

**Downstream agents MUST read these before planning or implementing.**

### Parity Test Infrastructure
- `crates/qwik-optimizer-oxc/tests/snapshot_tests.rs` -- Parity report logic, full match comparison, root/segment/diagnostic checks
- `crates/qwik-optimizer-oxc/tests/swc_expected/` -- SWC reference snapshot files for all 201 fixtures

### OXC Implementation
- `crates/qwik-optimizer-oxc/src/transform.rs` -- Main transform logic, code generation
- `crates/qwik-optimizer-oxc/src/code_move.rs` -- Segment extraction and root module assembly
- `crates/qwik-optimizer-oxc/src/collector.rs` -- Import/export collection, symbol tracking
- `crates/qwik-optimizer-oxc/src/words.rs` -- Naming conventions, QRL name computation

### SWC Reference
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/transform.rs` -- SWC transform for comparison
- `/Users/scottweaver/Projects/qwik/packages/optimizer/core/src/code_move.rs` -- SWC root module assembly

### Prior Phase Artifacts
- `.planning/phases/11-root-module-code-generation/11-VERIFICATION.md` -- Phase 11 verification results
- `.planning/phases/12-diagnostics-parity/12-VERIFICATION.md` -- Phase 12 verification results

</canonical_refs>

<code_context>
## Existing Code Insights

### Reusable Assets
- `parity_report` test: Already compares all 201 fixtures across three dimensions (root, segments, diagnostics)
- `swc_expected/*.snap` files: Pre-generated SWC reference outputs for every fixture
- `fixtures.json`: Complete fixture configuration (modes, strategies, options per fixture)

### Established Patterns
- Snapshot-based comparison: OXC output compared against SWC `.snap` files
- Root module comparison: Normalized string comparison of generated root module code
- Segment count comparison: Integer comparison of extracted segment counts

### Integration Points
- `code_move.rs::new_module()` -- Root module assembly (most root mismatches originate here)
- `transform.rs::exit_*` methods -- AST transformation handlers affecting output structure
- `collector.rs` -- Symbol collection feeding into code generation decisions

</code_context>

<specifics>
## Specific Ideas

Current parity breakdown (as of Phase 12 completion):
- Full match: 79/201 (39%) -- 122 fixtures need root module fixes
- Segment count: 195/201 -- 6 fixtures with segment count mismatches
- Diagnostics: 201/201 -- fully resolved in Phase 12

Known segment mismatch: `should_preserve_non_ident_explicit_captures` (exp=1, act=0)

</specifics>

<deferred>
## Deferred Ideas

None -- discussion stayed within phase scope.

</deferred>

---

*Phase: 13-final-acceptance*
*Context gathered: 2026-04-06*
