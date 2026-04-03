---
phase: 2
slug: jsx-props-signal-specification
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-01
---

# Phase 2 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual review (specification document — no automated tests) |
| **Config file** | none |
| **Quick run command** | `grep -c "^##" specification/qwik-optimizer-spec.md` (section count check) |
| **Full suite command** | `grep -cE "JSX Transform\|Props Destructuring\|Signal Optimization\|_jsxSorted\|_jsxSplit\|_fnSignal\|_wrapProp\|_rawProps\|_restProps" specification/qwik-optimizer-spec.md` |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Verify spec file exists and section count grows
- **After every plan wave:** Check all required sections present
- **Before `/gsd:verify-work`:** Full section inventory against requirements
- **Max feedback latency:** 1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 02-01-01 | 01 | 1 | SPEC-04 | grep | `grep "Props Destructuring" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 02-02-01 | 02 | 1 | SPEC-06 | grep | `grep "_jsxSorted" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 02-02-02 | 02 | 1 | SPEC-06 | grep | `grep "_jsxSplit" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 02-03-01 | 03 | 2 | SPEC-07 | grep | `grep "_fnSignal" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 02-03-02 | 03 | 2 | SPEC-07 | grep | `grep "_wrapProp" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- Spec document already exists from Phase 1: `specification/qwik-optimizer-spec.md`

*Existing infrastructure covers all phase requirements.*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Spec completeness | All SPEC-* | Content quality requires human review | Read each section; verify rules are implementable without SWC source |
| Example correctness | SPEC-06, SPEC-07 | Input/output examples must match SWC behavior | Cross-reference against SWC snapshot tests |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 1s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
