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
            ..Default::default()
        };
        let codegen_result = oxc::codegen::Codegen::new()
            .with_options(codegen_options)
            .with_source_text(source)
            .build(program);

        let map = codegen_result.map.map(|sm| sm.to_json_string());

        let code = normalize_pure_annotations(&codegen_result.code);
        let code = insert_separator_comments(&code);
        EmitResult { code, map }
    } else {
        let codegen_result = oxc::codegen::Codegen::new()
            .with_source_text(source)
            .build(program);

        let code = normalize_pure_annotations(&codegen_result.code);
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
    // Normalize arrow function spacing in dynamic imports to match SWC format.
    // OXC codegen emits `() => import(...)` but SWC uses `()=>import(...)`.
    code.replace("() => import(", "()=>import(")
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
