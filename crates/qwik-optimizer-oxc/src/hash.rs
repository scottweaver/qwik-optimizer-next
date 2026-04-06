//! Segment hash computation, symbol escaping, and context naming.
//!
//! Provides:
//! - `compute_segment_hash` -- 11-char SipHash-1-3 hash for a segment
//! - `escape_sym` -- normalize arbitrary strings to valid identifier parts
//! - `register_context_name` -- full 6-step naming pipeline
//! - `get_canonical_filename` -- derive canonical_filename from display_name + symbol_name
//!
//! Algorithm (SPEC Hash):
//!   1. Concatenate raw bytes (no separators): scope?, rel_path, display_name_core
//!   2. SipHash-1-3 with deterministic seed (0, 0)
//!   3. u64 -> 8 little-endian bytes -> base64url -> replace `-` and `_` with `0`
//!
//! CRITICAL: Uses `siphasher::sip::SipHasher13` with seed (0, 0), NOT
//! `std::collections::hash_map::DefaultHasher` which is non-deterministic.

use std::collections::HashMap;
use std::hash::Hasher;

use base64::Engine;
use siphasher::sip::SipHasher13;

use crate::types::EmitMode;

/// Compute the 2-character file hash prefix used for JSX dev keys.
///
/// SWC computes `base64(file_hash)[0..2]` where `file_hash` is derived from
/// `scope? + rel_path` (without display_name). This prefix is prepended to
/// the JSX key counter: e.g., `"u6_0"`, `"u6_1"`.
pub(crate) fn compute_file_hash_prefix(scope: Option<&str>, rel_path: &str) -> String {
    let mut hasher = SipHasher13::new_with_keys(0, 0);
    if let Some(s) = scope {
        hasher.write(s.as_bytes());
    }
    hasher.write(rel_path.as_bytes());
    let hash_value = hasher.finish();
    let encoded =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash_value.to_le_bytes());
    let sanitized = encoded.replace(['-', '_'], "0");
    sanitized[..2.min(sanitized.len())].to_string()
}

/// Result of the `register_context_name` naming pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ContextNameResult {
    /// Symbol name used in generated code.
    /// Prod: `s_{hash}`, others: `{display_name_core}_{hash}`
    pub symbol_name: String,

    /// Display name for the segment (raw file_name prefix + display_name_core).
    /// Format: `{file_name}_{display_name_core}` (e.g., "test.tsx_test_component")
    pub display_name: String,

    /// 11-character hash.
    pub hash: String,

    /// Canonical filename: `{display_name}_{hash}`
    pub canonical_filename: String,
}

// ---------------------------------------------------------------------------
// compute_segment_hash
// ---------------------------------------------------------------------------

/// Compute the 11-character segment hash.
///
/// Algorithm:
///   - SipHasher13 with seed (0, 0)
///   - Write scope bytes (if Some)
///   - Write rel_path bytes
///   - Write display_name bytes (pre-file-prefix portion)
///   - Finish -> u64 -> to_le_bytes -> base64 URL_SAFE_NO_PAD -> replace `-`/`_` with `0`
pub fn compute_segment_hash(
    scope: Option<&str>,
    rel_path: &str,
    display_name: &str,
) -> String {
    let mut hasher = SipHasher13::new_with_keys(0, 0);
    if let Some(s) = scope {
        hasher.write(s.as_bytes());
    }
    hasher.write(rel_path.as_bytes());
    hasher.write(display_name.as_bytes());
    let hash_value = hasher.finish();

    let encoded =
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash_value.to_le_bytes());
    encoded.replace(['-', '_'], "0")
}

/// Compute hash, optionally short-circuiting via `hash_override`.
///
/// When `hash_override` is Some, ONLY its bytes are hashed (scope/rel_path/display_name
/// are ignored). Otherwise delegates to `compute_segment_hash`.
pub(crate) fn compute_segment_hash_with_override(
    scope: Option<&str>,
    rel_path: &str,
    display_name: &str,
    hash_override: Option<&str>,
) -> String {
    if let Some(h) = hash_override {
        let mut hasher = SipHasher13::new_with_keys(0, 0);
        hasher.write(h.as_bytes());
        let hash_value = hasher.finish();
        let encoded =
            base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(hash_value.to_le_bytes());
        return encoded.replace(['-', '_'], "0");
    }
    compute_segment_hash(scope, rel_path, display_name)
}

// ---------------------------------------------------------------------------
// escape_sym
// ---------------------------------------------------------------------------

/// Normalize a string to a valid identifier fragment.
///
/// Steps:
///   1. Replace non-ASCII-alphanumeric chars with `_`
///   2. Squash consecutive underscores into one
///   3. Trim leading and trailing underscores
///   4. Prepend `_` if the result starts with a digit
///   5. Return empty string if input is empty or result is all underscores
pub(crate) fn escape_sym(s: &str) -> String {
    if s.is_empty() {
        return String::new();
    }

    // Step 1: Replace non-alphanumeric (ASCII) with '_'
    let replaced: String = s
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();

    // Step 2: Squash consecutive underscores
    let mut squashed = String::with_capacity(replaced.len());
    let mut prev_underscore = false;
    for c in replaced.chars() {
        if c == '_' {
            if !prev_underscore {
                squashed.push(c);
            }
            prev_underscore = true;
        } else {
            squashed.push(c);
            prev_underscore = false;
        }
    }

    // Step 3: Trim leading and trailing underscores
    let trimmed = squashed.trim_matches('_');

    if trimmed.is_empty() {
        return String::new();
    }

    // Step 4: Digit prefix guard
    if trimmed.starts_with(|c: char| c.is_ascii_digit()) {
        format!("_{trimmed}")
    } else {
        trimmed.to_string()
    }
}

// ---------------------------------------------------------------------------
// get_canonical_filename
// ---------------------------------------------------------------------------

/// Derive `canonical_filename` from `display_name` and `symbol_name`.
///
/// Extracts the hash suffix (last `_`-delimited token of symbol_name)
/// and returns `{display_name}_{hash_suffix}`.
pub(crate) fn get_canonical_filename(display_name: &str, symbol_name: &str) -> String {
    let hash_suffix = symbol_name
        .rsplit('_')
        .next()
        .unwrap_or(symbol_name);
    format!("{display_name}_{hash_suffix}")
}

// ---------------------------------------------------------------------------
// register_context_name
// ---------------------------------------------------------------------------

/// Full 6-step naming pipeline for a segment.
///
/// # Steps
/// 1. Custom symbol short-circuit (if custom_symbol is Some)
/// 1b. Join stack_ctxt with `_`, run through escape_sym
/// 2. Collision counter -- append `_{n}` suffix for duplicate names (none for first)
/// 3. Compute hash via compute_segment_hash_with_override
/// 4. Build symbol_name based on mode (Prod: `s_{hash}`, others: `{core}_{hash}`)
/// 5. Build display_name = `{file_name}_{display_name_core}`
/// 6. Build canonical_filename via get_canonical_filename
pub(crate) fn register_context_name(
    stack_ctxt: &[String],
    segment_names: &mut HashMap<String, u32>,
    scope: Option<&str>,
    rel_path: &str,
    file_name: &str,
    mode: &EmitMode,
    custom_symbol: Option<&str>,
    display_name_override: Option<&str>,
    hash_override: Option<&str>,
) -> ContextNameResult {
    // Step 1: custom_symbol short-circuit
    if let Some(sym) = custom_symbol {
        let hash = compute_segment_hash_with_override(scope, rel_path, sym, hash_override);
        let symbol_name = match mode {
            EmitMode::Prod => format!("s_{hash}"),
            _ => format!("{sym}_{hash}"),
        };
        let display_name = format!("{file_name}_{sym}");
        let canonical_filename = get_canonical_filename(&display_name, &symbol_name);
        return ContextNameResult {
            symbol_name,
            display_name,
            hash,
            canonical_filename,
        };
    }

    // Step 1b: join stack_ctxt and escape
    let joined = stack_ctxt.join("_");
    let base_core = escape_sym(&joined);

    // Step 2: collision counter
    // first=none, second=_1, third=_2
    let counter = segment_names.entry(base_core.clone()).or_insert(0);
    let display_name_core = if *counter == 0 {
        base_core.clone()
    } else {
        format!("{base_core}_{}", *counter)
    };
    *counter += 1;

    // Step 3: hash
    let hash_input = display_name_override.unwrap_or(&display_name_core);
    let hash = compute_segment_hash_with_override(scope, rel_path, hash_input, hash_override);

    // Step 4: symbol_name
    let symbol_name = match mode {
        EmitMode::Prod => format!("s_{hash}"),
        _ => format!("{display_name_core}_{hash}"),
    };

    // Step 5: display_name -- file_name is RAW (not escape_sym'd)
    let display_name = format!("{file_name}_{display_name_core}");

    // Step 6: canonical_filename
    let canonical_filename = get_canonical_filename(&display_name, &symbol_name);

    ContextNameResult {
        symbol_name,
        display_name,
        hash,
        canonical_filename,
    }
}

/// Parse an existing inlinedQrl symbol name into its components.
pub(crate) fn parse_symbol_name(
    symbol_name: &str,
    mode: &EmitMode,
    file_name: &str,
) -> (String, String, String) {
    let (prefix, hash) = match symbol_name.rsplit_once('_') {
        Some((p, h)) => (p, h),
        None => (symbol_name, ""),
    };

    let display_name = format!("{file_name}_{prefix}");

    let new_symbol_name = match mode {
        EmitMode::Prod => format!("s_{hash}"),
        _ => symbol_name.to_string(),
    };

    (new_symbol_name, display_name, hash.to_string())
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // compute_segment_hash -- golden snapshot vectors
    // -----------------------------------------------------------------------

    #[test]
    fn hash_golden_test_component() {
        let h = compute_segment_hash(None, "test.tsx", "test_component");
        assert_eq!(h, "LUXeXe0DQrg", "test_component hash mismatch");
    }

    #[test]
    fn hash_golden_foo_component() {
        let h = compute_segment_hash(None, "test.tsx", "Foo_component");
        assert_eq!(h, "HTDRsvUbLiE", "Foo_component hash mismatch");
    }

    #[test]
    fn hash_golden_foo_component_sig_use_async() {
        let h = compute_segment_hash(None, "test.tsx", "Foo_component_sig_useAsync");
        assert_eq!(h, "f0BGwWm4eeY", "Foo_component_sig_useAsync hash mismatch");
    }

    #[test]
    fn hash_golden_foo_component_other_use_async() {
        let h = compute_segment_hash(None, "test.tsx", "Foo_component_other_useAsync");
        assert_eq!(h, "fsHooibmyyE", "Foo_component_other_useAsync hash mismatch");
    }

    #[test]
    fn hash_always_11_chars() {
        let h = compute_segment_hash(None, "test.tsx", "test_component");
        assert_eq!(h.len(), 11, "hash must be 11 chars, got {h:?}");
    }

    #[test]
    fn hash_alphanumeric_only() {
        let h = compute_segment_hash(None, "test.tsx", "test_component");
        assert!(
            h.chars().all(|c| c.is_ascii_alphanumeric()),
            "hash must be alphanumeric (no - or _), got {h:?}"
        );
    }

    #[test]
    fn hash_scope_changes_output() {
        let without_scope = compute_segment_hash(None, "test.tsx", "test_component");
        let with_scope = compute_segment_hash(Some("my-pkg"), "test.tsx", "test_component");
        assert_ne!(without_scope, with_scope, "scope must change the hash");
    }

    #[test]
    fn hash_deterministic() {
        let h1 = compute_segment_hash(None, "test.tsx", "test_component");
        let h2 = compute_segment_hash(None, "test.tsx", "test_component");
        assert_eq!(h1, h2, "hash must be deterministic");
    }

    #[test]
    fn hash_override_bypasses_normal_inputs() {
        let normal = compute_segment_hash_with_override(
            None,
            "test.tsx",
            "test_component",
            None,
        );
        let overridden = compute_segment_hash_with_override(
            Some("some-scope"),
            "completely-different.tsx",
            "different_name",
            Some("my-override"),
        );
        let overridden2 = compute_segment_hash_with_override(
            Some("some-scope"),
            "completely-different.tsx",
            "different_name",
            Some("other-override"),
        );
        assert_ne!(normal, overridden);
        assert_ne!(overridden, overridden2);
        let overridden3 = compute_segment_hash_with_override(
            None,
            "test.tsx",
            "test_component",
            Some("my-override"),
        );
        assert_eq!(overridden, overridden3);
    }

    // -----------------------------------------------------------------------
    // escape_sym
    // -----------------------------------------------------------------------

    #[test]
    fn escape_sym_replaces_non_alnum_with_underscore() {
        assert_eq!(escape_sym("my-component.handler"), "my_component_handler");
    }

    #[test]
    fn escape_sym_trims_leading_underscores() {
        assert_eq!(escape_sym("---foo"), "foo");
    }

    #[test]
    fn escape_sym_digit_prefix() {
        assert_eq!(escape_sym("123click"), "_123click");
    }

    #[test]
    fn escape_sym_already_valid() {
        assert_eq!(escape_sym("already_valid"), "already_valid");
    }

    #[test]
    fn escape_sym_empty() {
        assert_eq!(escape_sym(""), "");
    }

    #[test]
    fn escape_sym_all_special_chars() {
        assert_eq!(escape_sym("---"), "");
    }

    // -----------------------------------------------------------------------
    // get_canonical_filename
    // -----------------------------------------------------------------------

    #[test]
    fn canonical_filename_extracts_hash_from_symbol() {
        let cf = get_canonical_filename(
            "test.tsx_test_component",
            "test_component_LUXeXe0DQrg",
        );
        assert_eq!(cf, "test.tsx_test_component_LUXeXe0DQrg");
    }

    #[test]
    fn canonical_filename_handles_nested_segment() {
        let cf = get_canonical_filename(
            "test.tsx_Foo_component_sig_useAsync",
            "Foo_component_sig_useAsync_f0BGwWm4eeY",
        );
        assert_eq!(cf, "test.tsx_Foo_component_sig_useAsync_f0BGwWm4eeY");
    }

    // -----------------------------------------------------------------------
    // register_context_name
    // -----------------------------------------------------------------------

    #[test]
    fn register_context_name_basic_dev_mode() {
        let mut names: HashMap<String, u32> = HashMap::new();
        let stack = vec!["test".to_string(), "component".to_string()];
        let result = register_context_name(
            &stack,
            &mut names,
            None,
            "test.tsx",
            "test.tsx",
            &EmitMode::Lib,
            None,
            None,
            None,
        );
        assert_eq!(result.hash, "LUXeXe0DQrg");
        assert_eq!(result.display_name, "test.tsx_test_component");
        assert_eq!(result.symbol_name, "test_component_LUXeXe0DQrg");
        assert_eq!(result.canonical_filename, "test.tsx_test_component_LUXeXe0DQrg");
    }

    #[test]
    fn register_context_name_prod_mode_uses_mangled_symbol() {
        let mut names: HashMap<String, u32> = HashMap::new();
        let stack = vec!["test".to_string(), "component".to_string()];
        let result = register_context_name(
            &stack,
            &mut names,
            None,
            "test.tsx",
            "test.tsx",
            &EmitMode::Prod,
            None,
            None,
            None,
        );
        assert_eq!(result.symbol_name, format!("s_{}", result.hash));
        assert_eq!(result.display_name, "test.tsx_test_component");
    }

    #[test]
    fn register_context_name_collision_counter() {
        let mut names: HashMap<String, u32> = HashMap::new();
        let stack = vec!["test".to_string(), "component".to_string()];

        let r1 = register_context_name(
            &stack, &mut names, None, "test.tsx", "test.tsx",
            &EmitMode::Lib, None, None, None,
        );
        let r2 = register_context_name(
            &stack, &mut names, None, "test.tsx", "test.tsx",
            &EmitMode::Lib, None, None, None,
        );
        let r3 = register_context_name(
            &stack, &mut names, None, "test.tsx", "test.tsx",
            &EmitMode::Lib, None, None, None,
        );

        assert!(r1.display_name.ends_with("_test_component"));
        assert!(r2.display_name.ends_with("_test_component_1"));
        assert!(r3.display_name.ends_with("_test_component_2"));
    }

    #[test]
    fn register_context_name_file_name_is_raw() {
        let mut names: HashMap<String, u32> = HashMap::new();
        let stack = vec!["component".to_string()];
        let result = register_context_name(
            &stack, &mut names, None, "test.tsx", "test.tsx",
            &EmitMode::Lib, None, None, None,
        );
        assert!(result.display_name.starts_with("test.tsx_"));
    }

    #[test]
    fn register_context_name_canonical_filename_is_display_name_plus_hash() {
        let mut names: HashMap<String, u32> = HashMap::new();
        let stack = vec!["test".to_string(), "component".to_string()];
        let result = register_context_name(
            &stack, &mut names, None, "test.tsx", "test.tsx",
            &EmitMode::Lib, None, None, None,
        );
        assert_eq!(
            result.canonical_filename,
            format!("{}_{}", result.display_name, result.hash)
        );
    }

    // -----------------------------------------------------------------------
    // parse_symbol_name
    // -----------------------------------------------------------------------

    #[test]
    fn parse_symbol_name_prod_mode() {
        let (sym, display, hash) = parse_symbol_name("test_component_ABC", &EmitMode::Prod, "test.tsx");
        assert_eq!(sym, "s_ABC");
        assert_eq!(display, "test.tsx_test_component");
        assert_eq!(hash, "ABC");
    }

    #[test]
    fn parse_symbol_name_dev_mode_unchanged() {
        let (sym, display, hash) = parse_symbol_name("test_component_ABC", &EmitMode::Dev, "test.tsx");
        assert_eq!(sym, "test_component_ABC");
        assert_eq!(display, "test.tsx_test_component");
        assert_eq!(hash, "ABC");
    }

    #[test]
    fn parse_symbol_name_no_underscore() {
        let (sym, display, hash) = parse_symbol_name("singleword", &EmitMode::Prod, "f.tsx");
        assert_eq!(sym, "s_");
        assert_eq!(display, "f.tsx_singleword");
        assert_eq!(hash, "");
    }
}
