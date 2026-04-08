# Milestones

## v0.2.0 Full SWC Parity (Shipped: 2026-04-08)

**Delivered:** Full SWC parity for the OXC-based Qwik optimizer — 200/201 fixtures produce output matching the SWC reference across root module, segment count, and diagnostics.

**Stats:** 4 phases, 18 plans, 27 tasks | 40+ commits | 383 files changed (+32,718 / -5,619) | ~14.8K LOC Rust | 5 days (2026-04-03 → 2026-04-07)

**Key accomplishments:**

1. **Segment extraction parity** — from 125/201 (62%) to 195/201 (97%) by implementing JSX dollar-attr extraction, strip_ctx_name fixes, noop segment generation, and C05 diagnostics
2. **Diagnostics parity** — from 197/201 to 201/201 (100%) via C02 export-symbol gate, C03 ordering fix, and C05 MissingQrlImplementation
3. **Root module code generation** — import assembly rewrite, dead code elimination, stack_ctxt/hash computation, PURE annotations, quote style preservation
4. **Final acceptance sweep** — 11 gap-closure plans bringing full parity from 28/201 (14%) to 200/201 (99.5%), fixing indentation, QRL naming, inline strategy, dev mode metadata, arrow spacing, const sorting, JSX key prefixes, signal optimization body emission
5. **Universal QRL hoisting** — all entry strategies (Hoist, Inline, Hook) use unified _noopQrl/.s()/.w() code path with correct body codegen and capture lists

**Known gaps:**

- 1 failing snapshot (example_1): spurious `$` import leaking into segment output

**Archive:** [v0.2.0-ROADMAP.md](milestones/v0.2.0-ROADMAP.md) | [v0.2.0-REQUIREMENTS.md](milestones/v0.2.0-REQUIREMENTS.md)

---

## v0.1.0 — Qwik Optimizer Spec & OXC Implementation (Shipped: 2026-04-03)

**Delivered:** Comprehensive behavioral specification of the Qwik v2 optimizer plus a feature-complete OXC-based implementation passing all behavioral tests.

**Stats:** 9 phases, 35 plans, 58 tasks | 192 commits | ~190K LOC Rust | 4 days (2026-03-31 → 2026-04-03)

**Key accomplishments:**

1. **Complete behavioral specification** covering all 14 CONV transformations, 7 entry strategies, 5 emit modes, and public API contracts in a single comprehensive document
2. **Working OXC implementation** using idiomatic Traverse trait, arena allocators, SemanticBuilder, and Codegen — passing 444 tests (211 snapshots + 233 unit tests)
3. **NAPI and WASM bindings** for Node.js and browser consumers with same JSON interface as SWC version
4. **57/201 SWC root module parity** with descriptive symbol naming alignment, consumed import stripping, and separator comments
5. **All 35 v1 requirements satisfied** with verification reports for all 9 phases
6. **24 spec examples activated** (previously ignored) — all passing with documented gap analysis

**Known gaps carried forward:**

- SWC parity at 57/201 (28%) — remaining mismatches are structural (import ordering, whitespace, comment placement)
- Phase 1 plans 01-02 and 01-04 superseded by gap-closure Phase 7 (content written, plans not formally executed)
- Phase 2 plans created retroactively to match disk state

**Archive:** [v0.1.0-ROADMAP.md](milestones/v0.1.0-ROADMAP.md) | [v0.1.0-REQUIREMENTS.md](milestones/v0.1.0-REQUIREMENTS.md) | [v0.1.0-MILESTONE-AUDIT.md](milestones/v0.1.0-MILESTONE-AUDIT.md)

---
