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
pub(crate) mod parse;
pub(crate) mod collector;
pub(crate) mod entry_strategy;
pub(crate) mod rename_imports;
pub(crate) mod const_replace;
pub(crate) mod filter_exports;
pub(crate) mod props_destructuring;
pub(crate) mod transform;

// Re-export all public types
pub use types::*;
