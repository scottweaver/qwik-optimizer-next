//! All public and internal type definitions for the qwik-optimizer-oxc crate.
//!
//! This is a pure data module with no logic -- only structs, enums, derives,
//! and serde attributes. Separating types into their own module prevents
//! circular dependencies since every other module can import from `types`
//! without importing logic.
