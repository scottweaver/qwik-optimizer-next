//! Qwik optimizer using OXC for code transformation.
//!
//! This crate provides the type definitions and utility functions needed for
//! the Qwik $-call extraction pipeline. Full transform logic is added in
//! subsequent phases.

pub mod types;
pub mod words;
pub mod hash;
pub mod errors;
pub mod is_const;

// Re-export all public types
pub use types::*;
// qwik-optimizer-oxc: Core transform engine for the Qwik optimizer
// This crate implements all 14 CONV transformations using idiomatic OXC patterns.

// Placeholder: full module declarations will be added by plan 05-01
