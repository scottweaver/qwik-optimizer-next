---
phase: 11
slug: root-module-code-generation
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-06
---

# Phase 11 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + insta snapshot testing |
| **Config file** | `crates/qwik-optimizer-oxc/Cargo.toml` |
| **Quick run command** | `cargo test --package qwik-optimizer-oxc -- parity_report --nocapture` |
| **Full suite command** | `cargo test --package qwik-optimizer-oxc` |
| **Estimated runtime** | ~5 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test --package qwik-optimizer-oxc -- parity_report --nocapture`
- **After every plan wave:** Run `cargo test --package qwik-optimizer-oxc`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 5 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 11-01-01 | 01 | 1 | ROOT-01 | integration | `cargo test --package qwik-optimizer-oxc -- parity_report --nocapture` | ✅ | ⬜ pending |
| 11-01-02 | 01 | 1 | ROOT-01, ROOT-04 | integration | `cargo test --package qwik-optimizer-oxc -- parity_report --nocapture` | ✅ | ⬜ pending |
| 11-02-01 | 02 | 1 | ROOT-02, ROOT-03 | integration | `cargo test --package qwik-optimizer-oxc -- parity_report --nocapture` | ✅ | ⬜ pending |
| 11-02-02 | 02 | 1 | ROOT-04, ROOT-05 | integration | `cargo test --package qwik-optimizer-oxc -- parity_report --nocapture` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. The parity_report test already validates root module output against SWC reference snapshots for all 201 fixtures.

---

## Manual-Only Verifications

All phase behaviors have automated verification. The parity_report test compares root module output character-by-character (after normalization) against SWC reference snapshots.

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 5s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
