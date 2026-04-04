---
phase: 10
slug: segment-extraction
status: draft
nyquist_compliant: false
wave_0_complete: false
created: 2026-04-03
---

# Phase 10 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | cargo test (Rust built-in) + insta snapshot testing |
| **Config file** | `crates/qwik-optimizer-oxc/Cargo.toml` |
| **Quick run command** | `cargo test -p qwik-optimizer-oxc -- --test-threads=1 2>&1 \| head -50` |
| **Full suite command** | `cargo test -p qwik-optimizer-oxc` |
| **Estimated runtime** | ~30 seconds |

---

## Sampling Rate

- **After every task commit:** Run `cargo test -p qwik-optimizer-oxc -- --test-threads=1 2>&1 | head -50`
- **After every plan wave:** Run `cargo test -p qwik-optimizer-oxc`
- **Before `/gsd:verify-work`:** Full suite must be green
- **Max feedback latency:** 30 seconds

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 10-01-* | 01 | 1 | SEG-01..05 | snapshot | `cargo test -p qwik-optimizer-oxc` | ✅ | ⬜ pending |
| 10-02-* | 02 | 1 | SEG-01..05 | snapshot | `cargo test -p qwik-optimizer-oxc` | ✅ | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

Existing infrastructure covers all phase requirements. The 201 snapshot fixtures and parity comparison scripts are already in place from v0.1.0.

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Segment count parity | SEG-01..05 | Parity report comparison | Run parity report and verify segment count column matches for all affected fixtures |

---

## Validation Sign-Off

- [ ] All tasks have `<automated>` verify or Wave 0 dependencies
- [ ] Sampling continuity: no 3 consecutive tasks without automated verify
- [ ] Wave 0 covers all MISSING references
- [ ] No watch-mode flags
- [ ] Feedback latency < 30s
- [ ] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
