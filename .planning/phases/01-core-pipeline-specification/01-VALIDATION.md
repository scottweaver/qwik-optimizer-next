---
phase: 1
slug: core-pipeline-specification
status: draft
nyquist_compliant: true
wave_0_complete: false
created: 2026-04-01
---

# Phase 1 — Validation Strategy

> Per-phase validation contract for feedback sampling during execution.

---

## Test Infrastructure

| Property | Value |
|----------|-------|
| **Framework** | Manual review (specification document — no automated tests) |
| **Config file** | none |
| **Quick run command** | `grep -c "^##" specification/qwik-optimizer-spec.md` (section count check) |
| **Full suite command** | `grep -cE "^### (CONV|SPEC)" specification/qwik-optimizer-spec.md` (CONV section count) |
| **Estimated runtime** | ~1 second |

---

## Sampling Rate

- **After every task commit:** Verify spec file exists and section count grows
- **After every plan wave:** Check all required CONV/SPEC sections present
- **Before `/gsd:verify-work`:** Full section inventory against requirements
- **Max feedback latency:** 1 second

---

## Per-Task Verification Map

| Task ID | Plan | Wave | Requirement | Test Type | Automated Command | File Exists | Status |
|---------|------|------|-------------|-----------|-------------------|-------------|--------|
| 01-01-01 | 01 | 1 | SPEC-21 | grep | `grep "GlobalCollect" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-01-02 | 01 | 1 | SPEC-23 | grep | `grep "Hash Generation" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-01-03 | 01 | 1 | SPEC-24 | grep | `grep "Path Resolution" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-02-01 | 02 | 1 | SPEC-01 | grep | `grep "Dollar Detection" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-02-02 | 02 | 1 | SPEC-02 | grep | `grep "QRL Wrapping" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-03-01 | 03 | 2 | SPEC-03 | grep | `grep "Capture Analysis" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-03-02 | 03 | 2 | SPEC-30 | grep | `grep "CAPTURE-EDGE" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-04-01 | 04 | 2 | SPEC-05 | grep | `grep "Segment Extraction" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-04-02 | 04 | 2 | SPEC-12 | grep | `grep "Import Rewriting" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-05-01 | 05 | 3 | SPEC-22 | grep | `grep "Variable Migration" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |
| 01-05-02 | 05 | 3 | SPEC-25 | grep | `grep "Source Map" specification/qwik-optimizer-spec.md` | ❌ W0 | ⬜ pending |

*Status: ⬜ pending · ✅ green · ❌ red · ⚠️ flaky*

---

## Wave 0 Requirements

- [ ] `specification/qwik-optimizer-spec.md` — Create initial spec document with title, overview, and Mermaid pipeline diagram

*Existing infrastructure covers remaining needs (markdown file editing).*

---

## Manual-Only Verifications

| Behavior | Requirement | Why Manual | Test Instructions |
|----------|-------------|------------|-------------------|
| Spec completeness | All SPEC-* | Content quality requires human review | Read each section; verify rules are implementable without SWC source |
| Example correctness | SPEC-29 | Input/output examples must match SWC behavior | Cross-reference against SWC snapshot tests |
| Mermaid diagram accuracy | SPEC-17 | Visual correctness | Render in VS Code/GitHub preview |

---

## Validation Sign-Off

- [x] All tasks have `<automated>` verify or Wave 0 dependencies
- [x] Sampling continuity: no 3 consecutive tasks without automated verify
- [x] Wave 0 covers all MISSING references
- [x] No watch-mode flags
- [x] Feedback latency < 1s
- [x] `nyquist_compliant: true` set in frontmatter

**Approval:** pending
