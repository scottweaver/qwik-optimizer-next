# Phase 5: Core OXC Implementation - Discussion Log

> **Audit trail only.**

**Date:** 2026-04-02
**Phase:** 05-core-oxc-implementation
**Areas discussed:** Crate structure, Reference code reuse, Test strategy, OXC version

---

## Crate Structure & Workspace
| Option | Selected |
|--------|----------|
| In qwik-optimizer-next root (`crates/qwik-optimizer-oxc/`) | ✓ |
| Inside specification/ | |
| Separate repo | |

## Reference Code Reuse
| Option | Selected |
|--------|----------|
| Spec-driven fresh build, reference as needed | ✓ |
| Fork Jack's implementation | |
| Fork Scott's conversion | |

## Test Strategy
| Option | Selected |
|--------|----------|
| Copy Jack's SWC snapshots | |
| Derive from spec examples | |
| Both | ✓ |

## OXC Version
| Option | Selected |
|--------|----------|
| Latest stable | ✓ |
| Match Jack's 0.113 | |

## Deferred Ideas
None
