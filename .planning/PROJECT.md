# Qwik Optimizer Specification & OXC Implementation

## What This Is

A comprehensive behavioral specification and feature-complete OXC implementation of the Qwik v2 optimizer, derived from the SWC-based Rust implementation and validated against 201 behavioral test fixtures. The specification lives in `specification/` and the implementation in `crates/` of the `qwik-optimizer-next` Rust workspace.

## Core Value

The specification must be comprehensive and precise enough that an OXC implementation can be built from it without referencing the SWC source code — capturing every transformation behavior, edge case, and output contract.

## Requirements

### Validated

- [x] Behavioral spec covers all 14 transformation types — Validated in Phases 1-4, 7
- [x] Spec documents the public API contract (TransformModulesOptions, TransformOutput) — Validated in Phase 4
- [x] Spec covers all 7 entry strategies with behavioral descriptions — Validated in Phase 3
- [x] Spec includes NAPI and WASM binding contracts — Validated in Phase 4
- [x] Spec includes 24+ representative input/output examples covering all 14 CONVs — Validated in Phase 4, 7
- [x] Spec includes OXC migration notes per transformation section — Validated in Phase 4
- [x] Feature-complete OXC implementation passes 201 behavioral tests — Validated in Phases 5, 8
- [x] OXC implementation uses Traverse trait, arena allocators, SemanticBuilder, Codegen — Validated in Phase 5

### Active

- [ ] Achieve 201/201 full SWC parity — every fixture's root module, segment count, and diagnostics match the SWC reference
- [ ] Fix root module code generation mismatches (95 fixtures)
- [x] Fix segment extraction for dollar-sign expressions in loops, ternaries, nested scopes, and spread props — Validated in Phase 10 (195/201 segment count parity, 97%)
- [ ] Fix diagnostics parity (4 fixtures)

### Out of Scope

- Duplicating SWC internal implementation details — spec describes behavior, not SWC's specific code structure
- TypeScript plugin integration (platform.ts binding swap) — separate concern from core optimizer
- Qwik build pipeline integration — the optimizer is a standalone transform, not a bundler plugin
- Performance benchmarking against SWC — correctness first, optimization later
- Parallel module processing (rayon) — deferred to post-parity milestone

## Context

## Current Milestone: v0.2.0 Full SWC Parity

**Goal:** Achieve 201/201 full SWC parity — every OXC-transformed fixture produces output matching the SWC reference.

**Target features:**
- Fix 95 root-module-only mismatches (code generation differences)
- Fix ~76 segment count mismatches (missing segment extraction)
- Fix 4 diagnostics mismatches (error reporting parity)

**Baseline:** 28/201 (14%) full match | 57/201 root match | 125/201 segment count match | 197/201 diagnostics match

**Current state (Phase 10 complete 2026-04-04):**
- ~190K LOC Rust across 3 crates (qwik-optimizer-oxc, qwik-optimizer-napi, qwik-optimizer-wasm)
- 511 tests passing (264 unit + 223 snapshot + 24 spec)
- 195/201 segment count parity (97%) — up from 125/201 (62%) at v0.1.0
- Comprehensive spec document covering all 14 CONVs with 24+ examples

**Source material:**
- SWC-based Qwik v2 optimizer: `/Users/scottweaver/Projects/qwik/packages/optimizer` (~18.5k LOC Rust, 18 modules)
- Jack Shelton's OXC conversion: `/Users/scottweaver/Projects/qwik-oxc-optimizer` (96% complete, 13k LOC, 162 spec files)

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
| Behavioral spec, not technical | Spec describes what the optimizer does, not how SWC implements it — enables idiomatic OXC rewrite | Validated |
| Build on Jack's 162 spec files | Near-complete test corpus already exists with input/output pairs and metadata | Validated — 201 fixtures + 24 spec examples |
| Include OXC migration notes | Explicit guidance on SWC→OXC pattern differences prevents porting anti-patterns | Validated |
| Single comprehensive document | Easier to reference and maintain than scattered docs | Validated |
| Core + NAPI + WASM bindings | Full coverage needed for drop-in replacement | Validated — NAPI v3 + wasm-bindgen |

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
*Last updated: 2026-04-04 after Phase 10 completion*
