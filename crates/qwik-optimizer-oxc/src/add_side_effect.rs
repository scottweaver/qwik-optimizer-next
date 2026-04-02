//! Stage 11 post-transform DCE -- SideEffectVisitor.
//!
//! For `Inline` and `Hoist` entry strategies, relative imports that were
//! collected in `GlobalCollect` should be preserved as bare side-effect
//! imports in the root module so bundlers see the dependency edge.

use std::path::{Path, PathBuf};

use oxc::allocator::Allocator;
use oxc::ast::ast::Statement;

use crate::collector::GlobalCollect;

// ---------------------------------------------------------------------------
// normalize_path
// ---------------------------------------------------------------------------

fn normalize_path(path: &Path) -> PathBuf {
    let mut components = Vec::new();
    for component in path.components() {
        use std::path::Component;
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if matches!(components.last(), Some(Component::Normal(_))) {
                    components.pop();
                } else {
                    components.push(component);
                }
            }
            other => components.push(other),
        }
    }
    components.iter().collect()
}

// ---------------------------------------------------------------------------
// add_side_effect_imports
// ---------------------------------------------------------------------------

/// Inject bare `import './source';` statements at position 0 of `program.body`
/// for each relative import in `global_collect` whose resolved path is inside
/// `src_dir`.
pub(crate) fn add_side_effect_imports<'a>(
    program: &mut oxc::ast::ast::Program<'a>,
    global_collect: &GlobalCollect,
    abs_dir: &Path,
    src_dir: &Path,
    allocator: &'a Allocator,
) {
    let mut existing: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    for stmt in program.body.iter() {
        if let Statement::ImportDeclaration(import_decl) = stmt {
            existing.insert(import_decl.source.value.as_str().to_string());
        }
    }

    let mut to_inject: Vec<String> = Vec::new();
    for (_local, import_info) in global_collect.imports.iter() {
        let src = &import_info.source;
        if !src.starts_with('.') {
            continue;
        }
        let raw = abs_dir.join(src);
        let resolved = normalize_path(&raw);
        let normalized_src_dir = normalize_path(src_dir);
        if !resolved.starts_with(&normalized_src_dir) {
            continue;
        }
        if existing.contains(src.as_str()) {
            continue;
        }
        if !to_inject.contains(src) {
            to_inject.push(src.clone());
        }
    }

    for source in to_inject.iter().rev() {
        let import_str = format!("import \"{source}\";");
        if let Some(stmt) = parse_single_statement(&import_str, allocator) {
            program.body.insert(0, stmt);
        }
    }
}

/// Parse a single statement from a string.
pub(crate) fn parse_single_statement<'a>(src: &str, allocator: &'a Allocator) -> Option<Statement<'a>> {
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    let src_arena: &str = allocator.alloc_str(src);
    let ret = Parser::new(allocator, src_arena, SourceType::mjs()).parse();
    if ret.panicked || ret.program.body.is_empty() {
        return None;
    }
    // SAFETY: program borrows from allocator; we transmute lifetime to match.
    let program: oxc::ast::ast::Program<'a> = unsafe {
        std::mem::transmute(ret.program)
    };
    program.body.into_iter().next()
}

/// Like `parse_single_statement` but uses JSX-aware source type (`.tsx`).
/// Needed for Hoist strategy `.s()` calls whose body may contain JSX.
pub(crate) fn parse_single_statement_jsx<'a>(src: &str, allocator: &'a Allocator) -> Option<Statement<'a>> {
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    let src_arena: &str = allocator.alloc_str(src);
    let source_type = SourceType::tsx();
    let ret = Parser::new(allocator, src_arena, source_type).parse();
    if ret.panicked || ret.program.body.is_empty() {
        return None;
    }
    let program: oxc::ast::ast::Program<'a> = unsafe {
        std::mem::transmute(ret.program)
    };
    program.body.into_iter().next()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::{Import, ImportKind};
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    fn parse_program<'a>(allocator: &'a Allocator, src: &str) -> oxc::ast::ast::Program<'a> {
        let src = allocator.alloc_str(src);
        let ret = Parser::new(allocator, src, SourceType::default()).parse();
        assert!(!ret.panicked, "parse failed");
        ret.program
    }

    fn make_collect(sources: &[&str]) -> GlobalCollect {
        let mut collect = GlobalCollect::new_empty();
        for (i, src) in sources.iter().enumerate() {
            collect.imports.insert(
                format!("local_{i}"),
                Import {
                    source: src.to_string(),
                    specifier: "default".to_string(),
                    kind: ImportKind::Default,
                    synthetic: false,
                },
            );
        }
        collect
    }

    #[test]
    fn test_inject_relative_import_within_src_dir() {
        let alloc = Allocator::default();
        let mut program = parse_program(&alloc, r#"export const x = 1;"#);

        let collect = make_collect(&["./utils"]);
        let abs_dir = PathBuf::from("/project/src");
        let src_dir = PathBuf::from("/project/src");

        add_side_effect_imports(&mut program, &collect, &abs_dir, &src_dir, &alloc);

        assert_eq!(program.body.len(), 2);
    }

    #[test]
    fn test_no_injection_for_non_relative_import() {
        let alloc = Allocator::default();
        let mut program = parse_program(&alloc, r#"export const x = 1;"#);
        let original_len = program.body.len();

        let collect = make_collect(&["@qwik.dev/core"]);
        let abs_dir = PathBuf::from("/project/src");
        let src_dir = PathBuf::from("/project/src");

        add_side_effect_imports(&mut program, &collect, &abs_dir, &src_dir, &alloc);

        assert_eq!(program.body.len(), original_len);
    }
}
