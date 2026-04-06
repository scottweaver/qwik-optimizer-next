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

        EmitResult {
            code: collapse_single_prop_objects(&codegen_result.code),
            map,
        }
    } else {
        let codegen_result = oxc::codegen::Codegen::new()
            .with_source_text(source)
            .build(program);

        EmitResult {
            code: collapse_single_prop_objects(&codegen_result.code),
            map: None,
        }
    }
}

/// Strip TypeScript type annotations from emitted code using OXC's
/// isolated declarations transform.
///
/// Since we cannot use OXC's `transformer` feature (per project constraints),
/// TS type stripping is deferred to a future plan that implements manual
/// AST mutation for type annotation removal.
///
/// Placeholder: returns the code unchanged for now.
#[allow(dead_code)]
pub(crate) fn strip_typescript_types(code: &str) -> String {
    code.to_string()
}

/// Post-process emitted code to collapse single-property object literals to one line.
///
/// OXC codegen renders `{ key: value }` as:
/// ```text
/// {
///     key: value
/// }
/// ```
/// SWC renders them inline: `{ key: value }`.
/// This function collapses such patterns for parity.
pub(crate) fn collapse_single_prop_objects(code: &str) -> String {
    let lines: Vec<&str> = code.lines().collect();
    let mut result = Vec::new();
    let mut i = 0;

    while i < lines.len() {
        // Look for pattern: line ending with `{`, next line with single prop, next line with `}`
        if i + 2 < lines.len() {
            let line1 = lines[i];
            let line2 = lines[i + 1];
            let line3 = lines[i + 2];

            let trimmed1 = line1.trim_end();
            let trimmed2 = line2.trim();
            let trimmed3 = line3.trim();

            // Pattern: `...{` / `key: value` / `}`  or  `...{` / `key: value,` / `}`
            if trimmed1.ends_with('{')
                && (trimmed3 == "}" || trimmed3 == "},")
                && !trimmed2.is_empty()
                && !trimmed2.contains('{')
                && !trimmed2.contains('}')
                && !trimmed2.starts_with("//")
            {
                // Remove trailing comma from the property if closing has no comma
                let prop = if trimmed2.ends_with(',') {
                    &trimmed2[..trimmed2.len() - 1]
                } else {
                    trimmed2
                };
                let suffix = if trimmed3 == "}," { "," } else { "" };
                let prefix = &trimmed1[..trimmed1.len() - 1]; // remove `{`
                result.push(format!("{}{{ {} }}{}", prefix, prop, suffix));
                i += 3;
                continue;
            }
        }
        result.push(lines[i].to_string());
        i += 1;
    }

    result.join("\n")
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

    #[test]
    fn test_strip_typescript_types_placeholder() {
        // Placeholder: strip_typescript_types is a no-op pending manual AST mutation impl
        let input = "export const foo = (ref: SeenRef) => {};";
        let result = strip_typescript_types(input);
        assert_eq!(result, input, "Placeholder should return unchanged");
    }
}
