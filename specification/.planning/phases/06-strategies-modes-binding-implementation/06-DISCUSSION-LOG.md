# Phase 6: Strategies, Modes & Binding Implementation - Discussion Log

> **Audit trail only.**

**Date:** 2026-04-02
**Phase:** 06-strategies-modes-binding-implementation
**Areas discussed:** NAPI crate setup, WASM target strategy, Entry strategy integration, Emit mode wiring

---

## NAPI Crate Setup
| Option | Selected |
|--------|----------|
| Separate napi/ crate | ✓ |
| Feature flag on core | |

| Option | Selected |
|--------|----------|
| NAPI-RS v2 | |
| NAPI-RS v3 | ✓ |

## WASM Target Strategy
| Option | Selected |
|--------|----------|
| wasm-bindgen (match SWC) | |
| NAPI v3 unified (try wasm32-wasip1-threads, fall back to wasm-bindgen) | ✓ |

## Entry Strategy Integration
| Option | Selected |
|--------|----------|
| Wire into transform_modules via code_move.rs | ✓ |

## Deferred Ideas
None
