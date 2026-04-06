//! Code generation.
//!
//! Generate JavaScript source code (and optional source maps) from an OXC
//! `Program` AST. Wraps `oxc::codegen::Codegen` with the crate's options
//! (minification, source maps, etc.) and produces `TransformModule` values.

use std::path::PathBuf;

/// Options controlling code emission.
pub(crate) struct EmitOptions {
    pub source_maps: bool,
}

/// Result of emitting a program to JavaScript source.
pub(crate) struct EmitResult {
    pub code: String,
    pub map: Option<String>,
}

/// Emit a Program AST to JavaScript source code.
///
/// Uses OXC's Codegen to serialize the AST back to JavaScript.
/// When `options.source_maps` is true, generates a v3 source map JSON string
/// via OXC codegen's `source_map_path` option combined with `with_source_text()`.
/// The `source_filename` parameter sets the `"file"` field in the source map JSON.
///
/// OXC Codegen produces double-quoted strings by default (matches qwik-core SWC output).
///
/// Post-processing: OXC codegen emits `/* @__PURE__ */` but SWC uses `/*#__PURE__*/`.
/// We normalize to SWC format for parity.
pub(crate) fn emit_module<'a>(
    program: &oxc::ast::ast::Program<'a>,
    source: &str,
    options: &EmitOptions,
    source_filename: &str,
) -> EmitResult {
    if options.source_maps {
        let codegen_options = oxc::codegen::CodegenOptions {
            source_map_path: Some(PathBuf::from(source_filename)),
            indent_char: oxc::codegen::IndentChar::Space,
            indent_width: 4,
            ..Default::default()
        };
        let codegen_result = oxc::codegen::Codegen::new()
            .with_options(codegen_options)
            .with_source_text(source)
            .build(program);

        let map = codegen_result.map.map(|sm| sm.to_json_string());

        let code = normalize_pure_annotations(&codegen_result.code);
        let code = inject_pure_annotations(&code);
        let code = preserve_original_quotes(&code, source);
        let code = sort_imports(&code, source);
        let code = sort_hoisted_consts(&code);
        let code = insert_separator_comments(&code);
        EmitResult { code, map }
    } else {
        let codegen_options = oxc::codegen::CodegenOptions {
            indent_char: oxc::codegen::IndentChar::Space,
            indent_width: 4,
            ..Default::default()
        };
        let codegen_result = oxc::codegen::Codegen::new()
            .with_options(codegen_options)
            .with_source_text(source)
            .build(program);

        let code = normalize_pure_annotations(&codegen_result.code);
        let code = inject_pure_annotations(&code);
        let code = preserve_original_quotes(&code, source);
        let code = sort_imports(&code, source);
        let code = sort_hoisted_consts(&code);
        let code = insert_separator_comments(&code);
        EmitResult { code, map: None }
    }
}

/// Normalize PURE annotations from OXC format to SWC format.
///
/// OXC codegen emits `/* @__PURE__ */` but SWC uses `/*#__PURE__*/`.
/// Both are valid tree-shaking hints but differ cosmetically.
fn normalize_pure_annotations(code: &str) -> String {
    let code = code.replace("/* @__PURE__ */", "/*#__PURE__*/");
    // Normalize arrow function spacing to match SWC format.
    // OXC codegen emits `) => ` but SWC uses `)=>`. Apply globally.
    normalize_arrow_spacing(&code)
}

/// Public wrapper for normalize_arrow_spacing for use in code_move.rs.
pub(crate) fn normalize_arrow_spacing_pub(code: &str) -> String {
    normalize_arrow_spacing(code)
}

/// Remove spaces around `=>` in arrow functions to match SWC output.
///
/// OXC codegen always emits `(params) => body` with spaces around `=>`.
/// SWC emits `(params)=>body` (no spaces). We normalize to SWC format.
///
/// This handles all arrow patterns:
/// - `() => {` -> `()=>{`
/// - `(props) => {` -> `(props)=>{`
/// - `() => import(` -> `()=>import(`
fn normalize_arrow_spacing(code: &str) -> String {
    // Replace ") => " with ")=>" and ") =>\n" with ")=>\n".
    // OXC codegen always parenthesizes arrow params, so ") =>" is the only
    // pattern we need to handle.
    //
    // We process line-by-line to avoid cross-line issues and use a simple
    // quote-parity check to avoid replacing inside string literals.
    let mut result = String::with_capacity(code.len());

    for line in code.split('\n') {
        if result.len() > 0 {
            result.push('\n');
        }
        // Quick check: does this line contain ") =>" at all?
        if !line.contains(") =>") {
            result.push_str(line);
            continue;
        }

        // Process the line character by character
        let bytes = line.as_bytes();
        let len = bytes.len();
        let mut i = 0;
        let mut in_string = false;
        let mut string_char: u8 = 0;

        while i < len {
            // Track string state
            if !in_string {
                if bytes[i] == b'"' || bytes[i] == b'\'' || bytes[i] == b'`' {
                    in_string = true;
                    string_char = bytes[i];
                    result.push(bytes[i] as char);
                    i += 1;
                    continue;
                }
            } else {
                if bytes[i] == string_char && (i == 0 || bytes[i - 1] != b'\\') {
                    in_string = false;
                }
                result.push(bytes[i] as char);
                i += 1;
                continue;
            }

            // Check for ") => " pattern (with space after =>)
            if !in_string
                && i + 4 < len
                && bytes[i] == b')'
                && bytes[i + 1] == b' '
                && bytes[i + 2] == b'='
                && bytes[i + 3] == b'>'
                && bytes[i + 4] == b' '
            {
                result.push_str(")=>");
                i += 5; // skip ") => "
                continue;
            }

            // Check for ") =>" at end of meaningful content (no space after)
            if !in_string
                && i + 3 < len
                && bytes[i] == b')'
                && bytes[i + 1] == b' '
                && bytes[i + 2] == b'='
                && bytes[i + 3] == b'>'
            {
                result.push_str(")=>");
                i += 4; // skip ") =>"
                continue;
            }

            result.push(bytes[i] as char);
            i += 1;
        }
    }

    result
}

/// Inject `/*#__PURE__*/` annotations before known wrapper call patterns.
///
/// SWC adds PURE annotations to wrapper calls in the root module body
/// (componentQrl, _jsxSorted, _jsxSplit, _noopQrl, etc.) to enable
/// tree-shaking by bundlers. The OXC transform renames callees (e.g.,
/// component$ -> componentQrl) but doesn't inject PURE comments at the
/// AST level. This post-processing pass adds them.
///
/// Only injects when the annotation is not already present immediately
/// before the call.
fn inject_pure_annotations(code: &str) -> String {
    // Wrapper call patterns that need PURE annotations.
    // qrl/qrlDEV are handled by rhs_code string injection in transform.rs.
    const WRAPPER_CALLS: &[&str] = &[
        "componentQrl(",
        "_jsxSorted(",
        "_jsxSplit(",
        "_noopQrl(",
        "_noopQrlDEV(",
    ];

    let mut result = String::with_capacity(code.len() + 256);
    for line in code.lines() {
        let mut modified = line.to_string();
        for &wrapper in WRAPPER_CALLS {
            // Find all occurrences in this line
            let mut search_from = 0;
            loop {
                if let Some(pos) = modified[search_from..].find(wrapper) {
                    let abs_pos = search_from + pos;
                    // Check if /*#__PURE__*/ already precedes this call
                    let prefix = &modified[..abs_pos];
                    let trimmed_prefix = prefix.trim_end();
                    if trimmed_prefix.ends_with("/*#__PURE__*/") {
                        // Already annotated, skip
                        search_from = abs_pos + wrapper.len();
                        continue;
                    }
                    // Don't inject inside import statements
                    if line.trim().starts_with("import ") {
                        break;
                    }
                    // Don't inject inside string literals (rough check: odd number of quotes before pos)
                    let quote_count = modified[..abs_pos].chars().filter(|&c| c == '"').count();
                    if quote_count % 2 != 0 {
                        search_from = abs_pos + wrapper.len();
                        continue;
                    }
                    // Insert /*#__PURE__*/ before the call
                    modified.insert_str(abs_pos, "/*#__PURE__*/ ");
                    search_from = abs_pos + "/*#__PURE__*/ ".len() + wrapper.len();
                } else {
                    break;
                }
            }
        }
        result.push_str(&modified);
        result.push('\n');
    }
    // Remove trailing newline if original didn't have one
    if !code.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

/// Preserve original quote style for user-written import specifiers.
///
/// SWC preserves the quote style from the original source for import declarations.
/// OXC Codegen normalizes all string literals to double quotes. This function
/// restores single quotes on import specifiers where the original source used them.
///
/// Only applies to `import ... from "..."` lines. Synthesized imports (not present
/// in the original source) keep double quotes, matching SWC behavior.
fn preserve_original_quotes(code: &str, source: &str) -> String {
    let mut result = String::with_capacity(code.len());
    for line in code.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with("import ") {
            if let Some(converted) = try_restore_single_quotes(line, source) {
                result.push_str(&converted);
            } else {
                result.push_str(line);
            }
        } else {
            result.push_str(line);
        }
        result.push('\n');
    }
    // Remove trailing newline if original didn't have one
    if !code.ends_with('\n') && result.ends_with('\n') {
        result.pop();
    }
    result
}

/// Try to restore single quotes on an import line's `from` specifier.
///
/// Returns `Some(modified_line)` if the original source used single quotes
/// for the same module specifier AND the same import bindings, `None` otherwise.
///
/// We match by checking that at least one imported name from the output line
/// also appears in a source import line that uses single quotes for the same
/// module specifier. This distinguishes user-written imports (which share
/// bindings with the source) from synthesized imports (which have new names
/// like `componentQrl`, `qrl`, etc. not in the source).
fn try_restore_single_quotes(line: &str, source: &str) -> Option<String> {
    // Find the `from "specifier"` portion
    let from_idx = line.rfind("from \"")?;
    let spec_start = from_idx + 6; // after `from "`
    let spec_end = line[spec_start..].find('"')? + spec_start;
    let specifier = &line[spec_start..spec_end];

    // Extract imported names from the output line (between { and })
    let output_names = extract_import_names(line);

    // Check if the original source has a matching import with single quotes
    let single_quoted = format!("'{}'", specifier);
    for src_line in source.lines() {
        let t = src_line.trim();
        if !t.starts_with("import ") || !t.contains(&single_quoted) {
            continue;
        }
        // Found a source import with single quotes for this specifier.
        // Check if ANY of the output import names appear in this source line.
        let src_names = extract_import_names(t);
        let has_overlap = output_names.iter().any(|n| src_names.contains(n));
        if has_overlap {
            // This output import line corresponds to a user-written import
            let mut result = String::with_capacity(line.len());
            result.push_str(&line[..from_idx]);
            result.push_str("from '");
            result.push_str(specifier);
            result.push('\'');
            let after = &line[spec_end + 1..];
            result.push_str(after);
            return Some(result);
        }
    }
    None
}

/// Extract named import identifiers from an import line.
/// E.g., `import { $, component$, useStore } from '...'` -> ["$", "component$", "useStore"]
fn extract_import_names(line: &str) -> Vec<&str> {
    let open = match line.find('{') {
        Some(i) => i,
        None => return Vec::new(),
    };
    let close = match line[open..].find('}') {
        Some(i) => open + i,
        None => return Vec::new(),
    };
    line[open + 1..close]
        .split(',')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect()
}

/// Sort import declarations to match SWC output ordering.
///
/// SWC places synthesized imports (those NOT present in the original source)
/// before user-written imports. Within synthesized imports, they are ordered
/// alphabetically by the first binding name (BTreeMap order).
///
/// User-written imports retain their original source order.
fn sort_imports(code: &str, source: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    if lines.is_empty() {
        return code.to_string();
    }

    // Find the contiguous block of import lines at the top
    let mut import_end = 0;
    for (i, line) in lines.iter().enumerate() {
        if line.trim().starts_with("import ") {
            import_end = i + 1;
        } else if !line.trim().is_empty() && import_end > 0 {
            break;
        }
    }

    if import_end < 2 {
        return code.to_string(); // 0 or 1 imports, nothing to sort
    }

    // Separate synthesized vs user imports
    let mut synthesized: Vec<&str> = Vec::new();
    let mut user: Vec<&str> = Vec::new();

    for i in 0..import_end {
        let line = lines[i];
        if !line.trim().starts_with("import ") {
            continue;
        }
        // Check if this import line (or its module specifier) exists in the source
        if is_user_import(line, source) {
            user.push(line);
        } else {
            synthesized.push(line);
        }
    }

    // Sort synthesized imports alphabetically by first binding name
    synthesized.sort_by(|a, b| {
        let key_a = import_sort_key(a);
        let key_b = import_sort_key(b);
        key_a.cmp(&key_b)
    });

    // Rebuild: synthesized first, then user
    let mut result: Vec<&str> = Vec::with_capacity(lines.len());
    result.extend_from_slice(&synthesized);
    result.extend_from_slice(&user);
    if import_end < lines.len() {
        result.extend_from_slice(&lines[import_end..]);
    }

    let mut output = result.join("\n");
    if code.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Check if an import line is user-written (exists in original source).
fn is_user_import(line: &str, source: &str) -> bool {
    // Extract binding names from the import line
    let names = extract_import_names(line);
    if names.is_empty() {
        // Default or namespace import -- check if module specifier appears in source
        if let Some(spec) = extract_module_specifier(line) {
            return source.contains(&spec);
        }
        return false;
    }

    // Check if ANY imported name appears in a source import line
    for src_line in source.lines() {
        let t = src_line.trim();
        if !t.starts_with("import ") {
            continue;
        }
        let src_names = extract_import_names(t);
        if names.iter().any(|n| src_names.contains(n)) {
            return true;
        }
    }
    false
}

/// Extract the module specifier from an import line.
fn extract_module_specifier(line: &str) -> Option<String> {
    // Match from "..." or from '...'
    if let Some(idx) = line.rfind("from ") {
        let after = &line[idx + 5..];
        let quote = after.chars().next()?;
        if quote == '"' || quote == '\'' {
            let end = after[1..].find(quote)?;
            return Some(after[1..1 + end].to_string());
        }
    }
    None
}

/// Extract the sort key from an import line (first binding name).
fn import_sort_key(line: &str) -> String {
    let names = extract_import_names(line);
    if let Some(first) = names.first() {
        return first.to_string();
    }
    // For default/namespace imports, use the local name
    let trimmed = line.trim();
    if let Some(rest) = trimmed.strip_prefix("import ") {
        if let Some(name) = rest.split_whitespace().next() {
            if name != "{" && name != "*" {
                return name.to_string();
            }
        }
    }
    line.to_string()
}

/// Sort hoisted QRL const declarations alphabetically by name to match SWC output.
///
/// SWC stores hoisted consts in a `BTreeMap<Id, _>` which produces alphabetical
/// ordering by identifier name. OXC accumulates them in DFS pre-order which may
/// differ. This post-processing step sorts the `const q_*` block alphabetically.
fn sort_hoisted_consts(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    if lines.is_empty() {
        return code.to_string();
    }

    // Find the contiguous block of hoisted const lines
    let mut hoisted_start = None;
    let mut hoisted_end = None;

    for (i, line) in lines.iter().enumerate() {
        let t = line.trim();
        let is_hoisted = t.starts_with("const q_") && t.contains("/*#__PURE__*/");
        if is_hoisted {
            if hoisted_start.is_none() {
                hoisted_start = Some(i);
            }
            hoisted_end = Some(i);
        }
    }

    let (start, end) = match (hoisted_start, hoisted_end) {
        (Some(s), Some(e)) if e > s => (s, e),
        _ => return code.to_string(), // 0 or 1 hoisted consts, nothing to sort
    };

    // Extract hoisted const lines (filtering out any non-const lines in the range)
    let mut hoisted_lines: Vec<&str> = Vec::new();
    let mut other_lines_in_range: Vec<(usize, &str)> = Vec::new();

    for i in start..=end {
        let t = lines[i].trim();
        if t.starts_with("const q_") && t.contains("/*#__PURE__*/") {
            hoisted_lines.push(lines[i]);
        } else {
            other_lines_in_range.push((i, lines[i]));
        }
    }

    // Sort alphabetically
    hoisted_lines.sort();

    // Rebuild the output
    let mut result: Vec<&str> = Vec::with_capacity(lines.len());
    result.extend_from_slice(&lines[..start]);
    result.extend_from_slice(&hoisted_lines);
    // Re-insert any non-hoisted lines that were in the range
    for (_, line) in &other_lines_in_range {
        result.push(line);
    }
    if end + 1 < lines.len() {
        result.extend_from_slice(&lines[end + 1..]);
    }

    let mut output = result.join("\n");
    if code.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Insert `//` separator comments between code sections to match SWC output format.
///
/// SWC emits empty single-line comments (`//`) as separators between:
/// 1. Import block and hoisted QRL const declarations (`const q_...`)
/// 2. Hoisted QRL const declarations and the module body
///
/// Only inserts separators when hoisted QRL declarations exist.
fn insert_separator_comments(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    if lines.is_empty() {
        return code.to_string();
    }

    // Check if there are any hoisted QRL const declarations
    let has_hoisted = lines.iter().any(|l| {
        let t = l.trim();
        t.starts_with("const q_") && t.contains("/*#__PURE__*/")
    });
    if !has_hoisted {
        return code.to_string();
    }

    let mut result = Vec::with_capacity(lines.len() + 4);
    let mut last_was_import = false;
    let mut in_hoisted_section = false;

    for line in &lines {
        let trimmed = line.trim();

        let is_import = trimmed.starts_with("import ");
        let is_hoisted_const = trimmed.starts_with("const q_") && trimmed.contains("/*#__PURE__*/");

        // Transition: imports -> non-imports (insert //)
        if last_was_import && !is_import && !trimmed.is_empty() {
            result.push("//");
            if is_hoisted_const {
                in_hoisted_section = true;
            }
        }

        // Transition: hoisted consts -> non-hoisted (insert //)
        if in_hoisted_section && !is_hoisted_const && !trimmed.is_empty() {
            result.push("//");
            in_hoisted_section = false;
        }

        result.push(line);
        last_was_import = is_import;
        if is_hoisted_const {
            in_hoisted_section = true;
        }
    }

    // Preserve trailing newline if original had one
    let mut output = result.join("\n");
    if code.ends_with('\n') && !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_emit_with_source_maps() {
        let source = "const x = 1;\nconst y = x + 2;\n";
        let allocator = oxc::allocator::Allocator::default();
        let source_in_arena = allocator.alloc_str(source);
        let source_type = oxc::span::SourceType::mjs();
        let ret = oxc::parser::Parser::new(&allocator, source_in_arena, source_type).parse();
        assert!(!ret.panicked, "Parse should not panic");

        let options = EmitOptions { source_maps: true };

        let result = emit_module(&ret.program, source_in_arena, &options, "test.js");

        assert!(
            result.code.contains("const x = 1"),
            "Expected code output: {}",
            result.code
        );

        assert!(result.map.is_some(), "Expected source map to be Some");
        let map_json = result.map.unwrap();
        assert!(
            map_json.contains("\"version\""),
            "Expected version field in source map: {}",
            map_json
        );
    }

    #[test]
    fn test_emit_without_source_maps() {
        let source = "const x = 1;\n";
        let allocator = oxc::allocator::Allocator::default();
        let source_in_arena = allocator.alloc_str(source);
        let source_type = oxc::span::SourceType::mjs();
        let ret = oxc::parser::Parser::new(&allocator, source_in_arena, source_type).parse();
        assert!(!ret.panicked, "Parse should not panic");

        let options = EmitOptions { source_maps: false };

        let result = emit_module(&ret.program, source_in_arena, &options, "test.js");

        assert!(
            result.code.contains("const x = 1"),
            "Expected code output: {}",
            result.code
        );

        assert!(
            result.map.is_none(),
            "Expected source map to be None when source_maps is false"
        );
    }

    #[test]
    fn test_preserve_original_quotes() {
        // User-written import with single quotes in source -> should restore single quotes
        let code = r#"import { $, component$, useStore } from "@qwik.dev/core";
import { componentQrl } from "@qwik.dev/core";
"#;
        let source = r#"import { $, component$, useStore } from '@qwik.dev/core';
export const App = component$(() => {});
"#;
        let result = preserve_original_quotes(code, source);
        assert!(
            result.contains("from '@qwik.dev/core'"),
            "User-written import should use single quotes: {}",
            result
        );
        // Synthesized import (componentQrl not in source) should keep double quotes
        assert!(
            result.contains("import { componentQrl } from \"@qwik.dev/core\""),
            "Synthesized import should keep double quotes: {}",
            result
        );
    }

    #[test]
    fn test_preserve_original_quotes_no_source_match() {
        // No matching import in source -> keep double quotes
        let code = r#"import { qrl } from "@qwik.dev/core";
"#;
        let source = "const x = 1;\n";
        let result = preserve_original_quotes(code, source);
        assert!(
            result.contains("from \"@qwik.dev/core\""),
            "Should keep double quotes when source has no matching import: {}",
            result
        );
    }

    #[test]
    fn test_emit_double_quoted_strings() {
        let source = "const x = 'hello';\n";
        let allocator = oxc::allocator::Allocator::default();
        let source_in_arena = allocator.alloc_str(source);
        let source_type = oxc::span::SourceType::mjs();
        let ret = oxc::parser::Parser::new(&allocator, source_in_arena, source_type).parse();
        assert!(!ret.panicked, "Parse should not panic");

        let options = EmitOptions { source_maps: false };
        let result = emit_module(&ret.program, source_in_arena, &options, "test.js");

        assert!(
            result.code.contains("\"hello\""),
            "Expected double-quoted string in output: {}",
            result.code
        );
    }
}
