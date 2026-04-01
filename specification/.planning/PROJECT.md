# Qwik Optimizer Specification & OXC Implementation

## What This Is

A comprehensive behavioral specification of the Qwik v2 optimizer (build/v2 branch), derived from the existing SWC-based Rust implementation and Jack Shelton's 162 OXC spec files. The spec serves as the authoritative reference for building a feature-complete, idiomatic OXC-based Qwik optimizer. This project lives in the `specification/` subdirectory of the `qwik-optimizer-next` Rust workspace.

## Core Value

The specification must be comprehensive and precise enough that an OXC implementation can be built from it without referencing the SWC source code — capturing every transformation behavior, edge case, and output contract.

## Requirements

### Validated

(None yet — ship to validate)

### Active

- [ ] Behavioral spec covers all 14 transformation types (dollar detection, QRL wrapping, capture analysis, props destructuring, segment extraction, JSX transforms, signal optimization, PURE annotations, const replacement, dead branch elimination, code stripping, import rewriting, sync$ serialization)
- [ ] Spec documents the public API contract (TransformModulesOptions input, TransformOutput output, all config variants)
- [ ] Spec covers all 6 entry strategies (Inline, Hoist, Single, Hook/Segment, Component, Smart) with behavioral descriptions
- [ ] Spec includes NAPI and WASM binding contracts
- [ ] Spec includes representative input/output examples extracted from Jack's 162 spec files
- [ ] Spec includes OXC migration notes where SWC and OXC patterns diverge (visitor vs traverse, fold vs mutation, arena allocation, semantic analysis)
- [ ] Feature-complete OXC implementation passes all behavioral tests derived from spec
- [ ] OXC implementation is idiomatic — not a port of SWC patterns, but native OXC architecture (two-phase analyze-then-emit, arena allocators, TraverseCtx)

### Out of Scope

- Duplicating SWC internal implementation details — spec describes behavior, not SWC's specific code structure
- TypeScript plugin integration (platform.ts binding swap) — separate concern from core optimizer
- Qwik build pipeline integration — the optimizer is a standalone transform, not a bundler plugin
- Performance benchmarking against SWC — correctness first, optimization later

## Context

**Source material:**
- SWC-based Qwik v2 optimizer: `/Users/scottweaver/Projects/qwik/packages/optimizer` (~18.5k LOC Rust, 18 modules)
- Jack Shelton's OXC conversion: `/Users/scottweaver/Projects/qwik-oxc-optimizer` (96% complete, 13k LOC, 162 spec files)
- Jack's AI-generated spec: referenced in README but currently inaccessible (Hashnode 403)

**Key architectural insight from Jack's work:** OXC requires a two-phase approach (analyze-then-emit) unlike SWC's ownership-based fold model. Semantic analysis must complete before any AST mutation to avoid scope/symbol invalidation.

**The optimizer's role in Qwik:** Enables resumability by extracting lazy-loadable segments from `$()` marker functions, generating QRL references for on-demand loading, and tracking captured variables across segment boundaries. It transforms a single input module into multiple output modules (root + N segments) with source maps.

## Constraints

- **Behavioral fidelity**: The OXC implementation must produce functionally equivalent output to the SWC version for all 162 test cases (cosmetic differences in formatting/naming are acceptable per Jack's precedent)
- **OXC idioms**: Implementation must use OXC's `Traverse` trait, arena allocators, `SemanticBuilder`, and `Codegen` — not SWC patterns translated to OXC APIs
- **Single spec document**: The specification is one comprehensive markdown file, not split across multiple docs
- **Foundation**: Jack's 162 spec files (`.planning/spec/*.md` in `qwik-oxc-optimizer`) are the behavioral test corpus

## Key Decisions

| Decision | Rationale | Outcome |
|----------|-----------|---------|
| Behavioral spec, not technical | Spec describes what the optimizer does, not how SWC implements it — enables idiomatic OXC rewrite | — Pending |
| Build on Jack's 162 spec files | Near-complete test corpus already exists with input/output pairs and metadata | — Pending |
| Include OXC migration notes | Explicit guidance on SWC→OXC pattern differences prevents porting anti-patterns | — Pending |
| Single comprehensive document | Easier to reference and maintain than scattered docs | — Pending |
| Core + NAPI + WASM bindings | Full coverage needed for drop-in replacement | — Pending |

## Evolution

This document evolves at phase transitions and milestone boundaries.

**After each phase transition** (via `/gsd:transition`):
1. Requirements invalidated? → Move to Out of Scope with reason
2. Requirements validated? → Move to Validated with phase reference
3. New requirements emerged? → Add to Active
4. Decisions to log? → Add to Key Decisions
5. "What This Is" still accurate? → Update if drifted

**After each milestone** (via `/gsd:complete-milestone`):
1. Full review of all sections
2. Core Value check — still the right priority?
3. Audit Out of Scope — reasons still valid?
4. Update Context with current state

---
*Last updated: 2026-04-01 after initialization*
