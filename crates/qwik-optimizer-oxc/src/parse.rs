//! Module parsing.
//!
//! Parse a single source file (JS/TS/JSX/TSX) into an OXC `Program` AST
//! with semantic scoping from `SemanticBuilder`. Reports parse errors as
//! `Diagnostic` values. Also provides path decomposition for input modules.
//!
//! Source type detection and output extension logic live in `source_path.rs`.

use std::path::{Path, PathBuf};

use oxc::semantic::Scoping;

use crate::errors;
use crate::source_path::SourcePath;
use crate::types::Diagnostic;

/// Decomposed path data for a single input module.
#[derive(Debug, Clone, PartialEq)]
pub(crate) struct PathData {
    /// Filename without extension (e.g., "index" for "src/routes/index.tsx").
    pub file_stem: String,

    /// Filename with extension (e.g., "index.tsx").
    pub file_name: String,

    /// Directory portion of the relative path (e.g., "src/routes").
    /// Empty PathBuf when the file is in the root.
    pub rel_dir: PathBuf,

    /// Absolute directory path = src_dir.join(rel_dir).
    pub abs_dir: PathBuf,
}

/// Result of parsing a single source file.
pub(crate) struct ParseResult<'a> {
    pub program: oxc::ast::ast::Program<'a>,
    pub source_type: oxc::span::SourceType,
    pub scoping: Scoping,
}

// Manual Debug impl because oxc::ast::ast::Program does not derive Debug
impl std::fmt::Debug for ParseResult<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ParseResult")
            .field("source_type", &self.source_type)
            .field("program.body.len", &self.program.body.len())
            .finish()
    }
}


/// Decompose a relative file path into its constituent parts.
///
/// `relative_path` is a slash-separated path relative to `src_dir`, e.g.
/// `"src/routes/index.tsx"`. `src_dir` is the absolute root directory.
///
/// Returns a `PathData` with:
/// - `file_stem`: filename without extension
/// - `file_name`: filename with extension
/// - `rel_dir`:   parent directory of relative_path (empty PathBuf when no parent)
/// - `abs_dir`:   `src_dir.join(rel_dir)`
pub(crate) fn parse_path(
    relative_path: &str,
    src_dir: &Path,
) -> Result<PathData, anyhow::Error> {
    let rel_path = Path::new(relative_path);

    let file_name = rel_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("path has no filename: {relative_path}"))?
        .to_string();

    let file_stem = rel_path
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| anyhow::anyhow!("path has no file stem: {relative_path}"))?
        .to_string();

    // rel_dir is the parent of the relative path, or empty if there is none.
    let rel_dir = match rel_path.parent() {
        Some(p) if p != Path::new("") => p.to_path_buf(),
        _ => PathBuf::new(),
    };

    let abs_dir = src_dir.join(&rel_dir);

    Ok(PathData {
        file_stem,
        file_name,
        rel_dir,
        abs_dir,
    })
}

/// Parse a single source file into an OXC Program AST with semantic scoping.
///
/// The `source` must have lifetime `'a` tied to the allocator so the AST
/// can reference it. Returns `Err(diagnostics)` only if the parser panicked
/// (unrecoverable error with empty AST). For recoverable parse errors,
/// the partial AST is returned along with diagnostics (OXC guarantees a
/// structurally valid AST even with syntax errors when `panicked == false`).
pub(crate) fn parse_module<'a>(
    allocator: &'a oxc::allocator::Allocator,
    source: &'a str,
    filename: &str,
) -> Result<(ParseResult<'a>, Vec<Diagnostic>), Vec<Diagnostic>> {
    let source_type = SourcePath(filename).source_type();

    // Parse source into AST
    let ret = oxc::parser::Parser::new(allocator, source, source_type).parse();

    // Only bail on unrecoverable parser panics (empty AST).
    // When panicked == false, OXC guarantees a structurally valid partial AST
    // even when there are syntax errors, so we can proceed with transformation.
    if ret.panicked {
        let diagnostics: Vec<Diagnostic> = ret
            .errors
            .iter()
            .map(|err| errors::create_source_error(&err.to_string(), filename))
            .collect();
        return Err(diagnostics);
    }

    // Collect any non-fatal parse errors as diagnostics
    let parse_diagnostics: Vec<Diagnostic> = ret
        .errors
        .iter()
        .map(|err| errors::create_source_error(&err.to_string(), filename))
        .collect();

    let program = ret.program;

    // Build semantic scoping
    let semantic_ret = oxc::semantic::SemanticBuilder::new()
        .with_excess_capacity(2.0)
        .build(&program);
    let scoping = semantic_ret.semantic.into_scoping();

    Ok((
        ParseResult {
            program,
            source_type,
            scoping,
        },
        parse_diagnostics,
    ))
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxc::allocator::Allocator;
    use std::path::Path;

    // ---- parse_module tests -----------------------------------------------

    #[test]
    fn test_parse_simple_const() {
        let allocator = Allocator::default();
        let source = "const x = 1;";
        let result = parse_module(&allocator, source, "test.js");
        assert!(result.is_ok(), "Expected successful parse");
        let (parsed, diags) = result.unwrap();
        assert!(diags.is_empty());
        assert_eq!(parsed.program.body.len(), 1, "Expected 1 statement");
    }

    #[test]
    fn test_parse_import_produces_ast() {
        let allocator = Allocator::default();
        let source = r#"import { $ } from '@qwik.dev/core';"#;
        let result = parse_module(&allocator, source, "test.tsx");
        assert!(result.is_ok());
        let (parsed, _) = result.unwrap();
        assert!(!parsed.program.body.is_empty());
    }

    #[test]
    fn test_parse_export_const() {
        let allocator = Allocator::default();
        let source = "export const Foo = 1;";
        let result = parse_module(&allocator, source, "test.tsx");
        assert!(result.is_ok());
        let (parsed, _) = result.unwrap();
        assert_eq!(parsed.program.body.len(), 1);
    }

    #[test]
    fn test_parse_tsx_source() {
        let allocator = Allocator::default();
        let source = r#"import { component$ } from '@qwik.dev/core';
export const App = component$(() => {
    return <div>Hello</div>;
});"#;

        let result = parse_module(&allocator, source, "app.tsx");
        assert!(result.is_ok(), "Expected successful parse of TSX source");

        let (parsed, diags) = result.unwrap();
        assert!(diags.is_empty());
        assert!(parsed.source_type.is_typescript());
        assert!(parsed.source_type.is_jsx());
        assert!(!parsed.program.body.is_empty());
    }

    #[test]
    fn test_parse_ts_source() {
        let allocator = Allocator::default();
        let source = r#"const x: number = 42;"#;

        let result = parse_module(&allocator, source, "utils.ts");
        assert!(result.is_ok());

        let (parsed, _diags) = result.unwrap();
        assert!(parsed.source_type.is_typescript());
        // JSX is enabled for .ts in Qwik
        assert!(parsed.source_type.is_jsx());
    }

    #[test]
    fn test_parse_jsx_source() {
        let allocator = Allocator::default();
        let source = r#"export const App = () => <div>Hello</div>;"#;

        let result = parse_module(&allocator, source, "app.jsx");
        assert!(result.is_ok());

        let (parsed, _diags) = result.unwrap();
        assert!(!parsed.source_type.is_typescript());
        assert!(parsed.source_type.is_jsx());
    }

    #[test]
    fn test_parse_js_source() {
        let allocator = Allocator::default();
        let source = r#"export const x = 1;"#;

        let result = parse_module(&allocator, source, "utils.js");
        assert!(result.is_ok());

        let (parsed, _diags) = result.unwrap();
        assert!(!parsed.source_type.is_typescript());
    }

    #[test]
    fn test_parse_error_does_not_panic() {
        let allocator = Allocator::default();
        // Syntax error: const without initializer, but recoverable
        let source = r#"const x = 1; const = ; const y = 2;"#;

        let result = parse_module(&allocator, source, "bad.tsx");
        // Should succeed with partial AST (recoverable error)
        // OR fail with panicked (unrecoverable) -- depends on OXC
        match result {
            Ok((parsed, diags)) => {
                assert!(!diags.is_empty(), "Expected parse diagnostics");
                assert!(!parsed.program.body.is_empty(), "Expected partial AST");
            }
            Err(diags) => {
                assert!(!diags.is_empty());
            }
        }
    }

    #[test]
    fn test_parse_returns_scoping() {
        let allocator = Allocator::default();
        let source = r#"
import { $ } from '@qwik.dev/core';
const x = $(() => {
    const inner = 1;
    return inner;
});
"#;

        let result = parse_module(&allocator, source, "test.tsx");
        assert!(result.is_ok());
        let (parsed, _diags) = result.unwrap();
        // Just verify we can access the scoping -- if this compiles and runs, scoping is valid
        let _scoping = &parsed.scoping;
    }

    // source_type and output_extension tests live in source_path.rs

    // ---- parse_path tests ------------------------------------------------

    #[test]
    fn test_parse_path_nested() {
        let src_dir = Path::new("/project");
        let result = parse_path("src/routes/index.tsx", src_dir).unwrap();
        assert_eq!(result.file_stem, "index");
        assert_eq!(result.file_name, "index.tsx");
        assert_eq!(result.rel_dir, PathBuf::from("src/routes"));
        assert_eq!(result.abs_dir, PathBuf::from("/project/src/routes"));
    }

    #[test]
    fn test_parse_path_root_level() {
        let src_dir = Path::new("/project");
        let result = parse_path("component.tsx", src_dir).unwrap();
        assert_eq!(result.file_stem, "component");
        assert_eq!(result.file_name, "component.tsx");
        assert_eq!(result.rel_dir, PathBuf::new());
        assert_eq!(result.abs_dir, PathBuf::from("/project"));
    }

    #[test]
    fn test_parse_path_one_level() {
        let src_dir = Path::new("/app");
        let result = parse_path("routes/index.ts", src_dir).unwrap();
        assert_eq!(result.file_stem, "index");
        assert_eq!(result.file_name, "index.ts");
        assert_eq!(result.rel_dir, PathBuf::from("routes"));
        assert_eq!(result.abs_dir, PathBuf::from("/app/routes"));
    }

    // output_extension tests live in source_path.rs
}
