---
phase: 5
slug: core-oxc-implementation
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-02
---

# Phase 5 — Validation Strategy

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Custom snapshot test harness + cargo test |
| **Config file** | crates/qwik-optimizer-oxc/Cargo.toml |
| **Quick run command** | `cargo test -p qwik-optimizer-oxc 2>&1 \| tail -5` |
| **Full suite command** | `cargo test -p qwik-optimizer-oxc -- --nocapture 2>&1` |
| **Estimated runtime** | ~30 seconds |

## Wave 0 Requirements

- [ ] `crates/qwik-optimizer-oxc/` — Rust crate with Cargo.toml, OXC dependencies
- [ ] `crates/qwik-optimizer-oxc/tests/` — Snapshot test harness with 201 .snap files
- [ ] `cargo test -p qwik-optimizer-oxc` exits 0 (even if most tests are marked ignored/pending)

## Validation Sign-Off

- [x] All tasks have automated verify
- [x] Sampling continuity satisfied
- [ ] Wave 0 covered
- [x] Feedback latency < 30s
- [x] `nyquist_compliant: true`

**Approval:** pending
