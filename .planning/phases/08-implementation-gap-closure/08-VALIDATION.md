---
phase: 8
slug: implementation-gap-closure
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-03
---

# Phase 8 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + insta 1.47.2 (snapshot testing) |
| **Config file** | `crates/qwik-optimizer-oxc/Cargo.toml` |
| **Quick run command** | `cargo test -p qwik-optimizer-oxc --lib` |
| **Full suite command** | `cargo test -p qwik-optimizer-oxc --test snapshot_tests --test spec_examples` |
| **Estimated runtime** | ~2 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p qwik-optimizer-oxc --lib`
- **After every plan wave:** Run `cargo test -p qwik-optimizer-oxc --test snapshot_tests --test spec_examples`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 08-01-01 | 01 | 1 | IMPL-02 | unit + snapshot | `cargo test -p qwik-optimizer-oxc --lib convert_inlined_fn` | ✅ | ⬜ pending |
| 08-01-02 | 01 | 1 | IMPL-02 | snapshot | `cargo test -p qwik-optimizer-oxc --test snapshot_tests -- derived_signals` | ✅ | ⬜ pending |
| 08-02-01 | 02 | 1 | IMPL-05 | snapshot | `cargo test -p qwik-optimizer-oxc --test snapshot_tests -- example_functional_component` | ✅ | ⬜ pending |
| 08-03-01 | 03 | 2 | IMPL-05 | snapshot | `cargo test -p qwik-optimizer-oxc --test snapshot_tests swc_parity -- --nocapture` | ✅ | ⬜ pending |
| 08-03-01 | 03 | 3 | IMPL-02 | integration | `cargo test -p qwik-optimizer-oxc --test spec_examples` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. The 201 snapshot tests and 24 spec_examples tests already exist — this phase activates and fixes them.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| SWC parity report | IMPL-05 | Parity % is a summary metric | Run `cargo test -p qwik-optimizer-oxc --test snapshot_tests swc_parity -- --nocapture` and verify root match count ≥ 50 |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
