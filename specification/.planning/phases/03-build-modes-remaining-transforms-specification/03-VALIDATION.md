---
phase: 3
slug: build-modes-remaining-transforms-specification
status: draft
nyquist_compliant: true
wave_0_complete: true
created: 2026-04-01
---

# Phase 3 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual review (specification document) |
| **Config file** | none |
| **Quick run command** | `grep -c "^##" specification/qwik-optimizer-spec.md` |
| **Full suite command** | `grep -cE "Entry Strateg|Emit Mode|PURE Annotation|Const Replacement|Dead Branch|Code Stripping|sync\\$|Noop QRL|Pipeline Ordering" specification/qwik-optimizer-spec.md` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Verify section count grows
- **After every plan wave:** Check all required sections present
- **Max feedback latency:** 1 second

---

## Wave 0 Requirements

- Spec document already exists from Phases 1-2: `specification/qwik-optimizer-spec.md` (3,817 lines)

*Existing infrastructure covers all phase requirements.*

---

## Validation Sign-Off

- [x] All tasks have automated verify
- [x] Sampling continuity satisfied
- [x] Wave 0 covered
- [x] No watch-mode flags
- [x] Feedback latency < 1s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
