//! Global Collector -- Stage 2.
//!
//! Performs a single read-only pass over the parsed AST to build an index of:
//! - `imports`: all import declarations (keyed by local name)
//! - `exports`: all export declarations (keyed by exported name)
//! - `root`: all top-level var/fn/class declarations that are NOT import or
//!   export statements (keyed by binding name)
//!
//! The resulting `GlobalCollect` is consumed by later stages (const replacement,
//! capture analysis, etc.).

use indexmap::IndexMap;
use oxc::ast::ast::*;
use oxc::span::Span;
use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// The kind of a JavaScript import binding.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum ImportKind {
    /// `import { foo } from "bar"`
    Named,
    /// `import Foo from "bar"` (default binding)
    Default,
    /// `import * as ns from "bar"` (namespace binding)
    Namespace,
}

/// Information about a single import binding.
#[derive(Debug, Clone)]
pub(crate) struct Import {
    /// The module specifier string (the `"bar"` in `import { foo } from "bar"`).
    pub source: String,
    /// The imported name from the source module (e.g. `"foo"` in `import { foo }`).
    /// For default imports this is `"default"`.
    /// For namespace imports this is `"*"`.
    pub specifier: String,
    /// How the binding was imported.
    pub kind: ImportKind,
    /// True if this import was synthetically injected (not from the original source).
    pub synthetic: bool,
}

/// Placeholder for export metadata. Extended in future phases.
#[derive(Debug, Clone, Default)]
pub(crate) struct ExportInfo {
    // Future phases will add re-export source, export kind, etc.
}

/// Centralized index of all imports, exports, and top-level declarations in a module.
///
/// Built by [`global_collect`] via a single read-only pass.
pub(crate) struct GlobalCollect {
    /// Synthetically-inserted imports (not from parsed source). Phase 10+ populates this.
    pub synthetic: Vec<(String, Import)>,
    /// All import bindings, keyed by local binding name.
    pub imports: IndexMap<String, Import>,
    /// All exported names, keyed by exported name.
    pub exports: IndexMap<String, ExportInfo>,
    /// Top-level declarations that are NOT imports or exports,
    /// keyed by binding name and valued by the declaration span.
    pub root: IndexMap<String, Span>,
    /// Reverse lookup: (specifier, source) -> local name.
    rev_imports: HashMap<(String, String), String>,
}

impl GlobalCollect {
    fn new() -> Self {
        Self {
            synthetic: Vec::new(),
            imports: IndexMap::with_capacity(16),
            exports: IndexMap::with_capacity(16),
            root: IndexMap::with_capacity(16),
            rev_imports: HashMap::with_capacity(16),
        }
    }

    /// Create an empty `GlobalCollect` for use in tests.
    #[allow(dead_code)]
    pub(crate) fn new_empty() -> Self {
        Self::new()
    }

    /// Insert an import binding, updating both `imports` and `rev_imports`.
    fn insert_import(&mut self, local: String, import: Import) {
        let key = (import.specifier.clone(), import.source.clone());
        self.rev_imports.insert(key, local.clone());
        self.imports.insert(local, import);
    }

    // -----------------------------------------------------------------------
    // Query methods
    // -----------------------------------------------------------------------

    /// Resolve `(specifier, source)` -> local binding name.
    ///
    /// Returns `Some(local)` when an `import { specifier } from "source"` exists.
    pub(crate) fn get_imported_local(&self, specifier: &str, source: &str) -> Option<&str> {
        self.rev_imports
            .get(&(specifier.to_string(), source.to_string()))
            .map(|s| s.as_str())
    }

    /// Returns `true` if `name` appears in imports.
    #[allow(dead_code)]
    pub(crate) fn is_import(&self, name: &str) -> bool {
        self.imports.contains_key(name)
    }

    /// Get import info for a given local name.
    #[allow(dead_code)]
    pub(crate) fn get_import(&self, name: &str) -> Option<&Import> {
        self.imports.get(name)
    }

    /// Returns `true` if `name` appears in imports, exports, or root.
    #[allow(dead_code)]
    pub(crate) fn is_global(&self, name: &str) -> bool {
        self.imports.contains_key(name)
            || self.exports.contains_key(name)
            || self.root.contains_key(name)
    }

    /// Returns the names of top-level declarations that are also exported.
    #[allow(dead_code)]
    pub(crate) fn export_local_ids(&self) -> Vec<String> {
        self.exports
            .keys()
            .filter(|name| self.root.contains_key(name.as_str()))
            .cloned()
            .collect()
    }

    /// Returns true if `sym` appears as a key in `self.exports`.
    #[allow(dead_code)]
    pub(crate) fn has_export_symbol(&self, sym: &str) -> bool {
        self.exports.contains_key(sym)
    }

    /// Register a synthetic import binding.
    ///
    /// Inserts into `imports`, `rev_imports`, AND `synthetic`.
    /// No-op if the specifier+source pair is already imported.
    #[allow(dead_code)]
    pub(crate) fn add_synthetic_import(&mut self, local: String, import: Import) {
        if self
            .get_imported_local(&import.specifier, &import.source)
            .is_some()
        {
            return;
        }
        self.synthetic.push((local.clone(), import.clone()));
        self.insert_import(local, import);
    }
}

// ---------------------------------------------------------------------------
// Collector pass (iterates program.body directly -- top-level only)
// ---------------------------------------------------------------------------

/// Build a `GlobalCollect` for `program` by scanning all top-level statements.
pub(crate) fn global_collect(program: &Program<'_>) -> GlobalCollect {
    let mut collect = GlobalCollect::new();

    for stmt in &program.body {
        match stmt {
            // ----------------------------------------------------------------
            // Import declarations
            // ----------------------------------------------------------------
            Statement::ImportDeclaration(import_decl) => {
                let source = import_decl.source.value.as_str().to_string();
                if let Some(specifiers) = &import_decl.specifiers {
                    for spec in specifiers {
                        match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(s) => {
                                let local = s.local.name.as_str().to_string();
                                let imported = match &s.imported {
                                    ModuleExportName::IdentifierName(id) => {
                                        id.name.as_str().to_string()
                                    }
                                    ModuleExportName::IdentifierReference(id) => {
                                        id.name.as_str().to_string()
                                    }
                                    ModuleExportName::StringLiteral(s) => {
                                        s.value.as_str().to_string()
                                    }
                                };
                                collect.insert_import(
                                    local,
                                    Import {
                                        source: source.clone(),
                                        specifier: imported,
                                        kind: ImportKind::Named,
                                        synthetic: false,
                                    },
                                );
                            }
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(s) => {
                                let local = s.local.name.as_str().to_string();
                                collect.insert_import(
                                    local,
                                    Import {
                                        source: source.clone(),
                                        specifier: "default".to_string(),
                                        kind: ImportKind::Default,
                                        synthetic: false,
                                    },
                                );
                            }
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(s) => {
                                let local = s.local.name.as_str().to_string();
                                collect.insert_import(
                                    local,
                                    Import {
                                        source: source.clone(),
                                        specifier: "*".to_string(),
                                        kind: ImportKind::Namespace,
                                        synthetic: false,
                                    },
                                );
                            }
                        }
                    }
                }
            }

            // ----------------------------------------------------------------
            // Export named declarations: `export { x }` or `export const x = ...`
            // ----------------------------------------------------------------
            Statement::ExportNamedDeclaration(export_decl) => {
                // Re-exports from another module or local re-exports: track exported names.
                for spec in &export_decl.specifiers {
                    let exported_name = match &spec.exported {
                        ModuleExportName::IdentifierName(id) => id.name.as_str().to_string(),
                        ModuleExportName::IdentifierReference(id) => {
                            id.name.as_str().to_string()
                        }
                        ModuleExportName::StringLiteral(s) => s.value.as_str().to_string(),
                    };
                    collect
                        .exports
                        .insert(exported_name, ExportInfo::default());
                }

                // Inline declaration: `export const x = 1;` or `export function f() {}`
                if let Some(decl) = &export_decl.declaration {
                    collect_decl_names(decl, &mut collect.exports, Some(&mut collect.root));
                }
            }

            // ----------------------------------------------------------------
            // Default exports: `export default function Foo() {}` or `export default expr`
            // ----------------------------------------------------------------
            Statement::ExportDefaultDeclaration(export_default) => {
                collect
                    .exports
                    .insert("default".to_string(), ExportInfo::default());
                // If a named function/class is default-exported, also add to root.
                match &export_default.declaration {
                    ExportDefaultDeclarationKind::FunctionDeclaration(f) => {
                        if let Some(id) = &f.id {
                            collect
                                .root
                                .insert(id.name.as_str().to_string(), id.span);
                        }
                    }
                    ExportDefaultDeclarationKind::ClassDeclaration(c) => {
                        if let Some(id) = &c.id {
                            collect
                                .root
                                .insert(id.name.as_str().to_string(), id.span);
                        }
                    }
                    _ => {}
                }
            }

            // ----------------------------------------------------------------
            // Export all: `export * from "mod"` -- no local bindings to index.
            // ----------------------------------------------------------------
            Statement::ExportAllDeclaration(_) => {}

            // ----------------------------------------------------------------
            // Top-level declarations (NOT exported, NOT imported)
            // ----------------------------------------------------------------
            other => {
                collect_stmt_root(other, &mut collect.root);
            }
        }
    }

    collect
}

/// Convenience: parse `source` and return a `GlobalCollect`.
///
/// Used in tests and by `dependency_analysis` helpers to build a collect from
/// raw source strings. On parse failure returns an empty `GlobalCollect`.
#[allow(dead_code)]
pub(crate) fn global_collect_from_str(source: &str) -> GlobalCollect {
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    let allocator = Allocator::default();
    let src: &str = allocator.alloc_str(source);
    let ret = Parser::new(&allocator, src, SourceType::mjs()).parse();
    if ret.panicked {
        return GlobalCollect::new_empty();
    }
    global_collect(&ret.program)
}

/// Collect binding names from a declaration, inserting into `exports` and
/// optionally into `root`.
fn collect_decl_names(
    decl: &Declaration<'_>,
    exports: &mut IndexMap<String, ExportInfo>,
    mut root: Option<&mut IndexMap<String, Span>>,
) {
    match decl {
        Declaration::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                collect_binding_pattern_names(&declarator.id, exports, root.as_deref_mut());
            }
        }
        Declaration::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                let name = id.name.as_str().to_string();
                exports.insert(name.clone(), ExportInfo::default());
                if let Some(r) = root {
                    r.insert(name, id.span);
                }
            }
        }
        Declaration::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                let name = id.name.as_str().to_string();
                exports.insert(name.clone(), ExportInfo::default());
                if let Some(r) = root {
                    r.insert(name, id.span);
                }
            }
        }
        _ => {}
    }
}

/// Collect binding names from a `BindingPattern`, inserting into exports and/or root.
fn collect_binding_pattern_names(
    pattern: &BindingPattern<'_>,
    exports: &mut IndexMap<String, ExportInfo>,
    mut root: Option<&mut IndexMap<String, Span>>,
) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            let name = id.name.as_str().to_string();
            exports.insert(name.clone(), ExportInfo::default());
            if let Some(r) = root {
                r.insert(name, id.span);
            }
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_pattern_names(&prop.value, exports, root.as_deref_mut());
            }
            if let Some(rest) = &obj.rest {
                collect_binding_pattern_names(&rest.argument, exports, root.as_deref_mut());
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for element in arr.elements.iter().flatten() {
                collect_binding_pattern_names(element, exports, root.as_deref_mut());
            }
            if let Some(rest) = &arr.rest {
                collect_binding_pattern_names(&rest.argument, exports, root.as_deref_mut());
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_pattern_names(&assign.left, exports, root.as_deref_mut());
        }
    }
}

/// Collect top-level binding names from a statement into `root`.
/// Only handles var/fn/class declarations -- other statement kinds are skipped.
fn collect_stmt_root(stmt: &Statement<'_>, root: &mut IndexMap<String, Span>) {
    match stmt {
        Statement::VariableDeclaration(var_decl) => {
            for declarator in &var_decl.declarations {
                collect_binding_into_root(&declarator.id, root);
            }
        }
        Statement::FunctionDeclaration(func) => {
            if let Some(id) = &func.id {
                root.insert(id.name.as_str().to_string(), id.span);
            }
        }
        Statement::ClassDeclaration(class) => {
            if let Some(id) = &class.id {
                root.insert(id.name.as_str().to_string(), id.span);
            }
        }
        _ => {}
    }
}

/// Recursively collect binding names from a `BindingPattern` into `root`.
fn collect_binding_into_root(pattern: &BindingPattern<'_>, root: &mut IndexMap<String, Span>) {
    match pattern {
        BindingPattern::BindingIdentifier(id) => {
            root.insert(id.name.as_str().to_string(), id.span);
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_into_root(&prop.value, root);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_into_root(&rest.argument, root);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for element in arr.elements.iter().flatten() {
                collect_binding_into_root(element, root);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_into_root(&rest.argument, root);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_into_root(&assign.left, root);
        }
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // Helper to run global_collect on a source string
    fn collect(src: &str) -> GlobalCollect {
        global_collect_from_str(src)
    }

    // -----------------------------------------------------------------------
    // Import tests
    // -----------------------------------------------------------------------

    #[test]
    fn named_import_populates_imports() {
        let gc = collect(r#"import { foo } from "bar";"#);
        let imp = gc.imports.get("foo").expect("foo not found in imports");
        assert_eq!(imp.source, "bar");
        assert_eq!(imp.specifier, "foo");
        assert_eq!(imp.kind, ImportKind::Named);
        assert!(!imp.synthetic);
    }

    #[test]
    fn import_dollar_from_qwik_core() {
        let gc = collect(r#"import { $ } from '@qwik.dev/core';"#);
        let imp = gc.imports.get("$").expect("$ not found in imports");
        assert_eq!(imp.source, "@qwik.dev/core");
        assert_eq!(imp.specifier, "$");
        assert_eq!(imp.kind, ImportKind::Named);
    }

    #[test]
    fn default_import_populates_imports() {
        let gc = collect(r#"import Def from "bar";"#);
        let imp = gc.imports.get("Def").expect("Def not found in imports");
        assert_eq!(imp.source, "bar");
        assert_eq!(imp.specifier, "default");
        assert_eq!(imp.kind, ImportKind::Default);
    }

    #[test]
    fn namespace_import_populates_imports() {
        let gc = collect(r#"import * as ns from "bar";"#);
        let imp = gc.imports.get("ns").expect("ns not found in imports");
        assert_eq!(imp.source, "bar");
        assert_eq!(imp.specifier, "*");
        assert_eq!(imp.kind, ImportKind::Namespace);
    }

    #[test]
    fn import_does_not_appear_in_root() {
        let gc = collect(r#"import { foo } from "bar";"#);
        assert!(
            !gc.root.contains_key("foo"),
            "Import bindings must NOT appear in root"
        );
    }

    // -----------------------------------------------------------------------
    // Export tests
    // -----------------------------------------------------------------------

    #[test]
    fn named_export_specifier_populates_exports() {
        let gc = collect(r#"const foo = 1; export { foo };"#);
        assert!(
            gc.exports.contains_key("foo"),
            "export {{ foo }} must populate exports"
        );
    }

    #[test]
    fn export_const_populates_exports_and_root() {
        let gc = collect(r#"export const Foo = 1;"#);
        assert!(gc.exports.contains_key("Foo"), "export const Foo must populate exports");
        assert!(gc.root.contains_key("Foo"), "export const Foo must also populate root");
    }

    #[test]
    fn export_function_populates_exports() {
        let gc = collect(r#"export function f() {}"#);
        assert!(gc.exports.contains_key("f"), "export function f must populate exports");
    }

    #[test]
    fn export_default_function_populates_exports() {
        let gc = collect(r#"export default function Foo() {}"#);
        assert!(
            gc.exports.contains_key("default"),
            "export default must insert 'default' key into exports"
        );
    }

    // -----------------------------------------------------------------------
    // Root tests
    // -----------------------------------------------------------------------

    #[test]
    fn plain_const_populates_root() {
        let gc = collect(r#"const bar = 2;"#);
        assert!(gc.root.contains_key("bar"), "const bar must appear in root");
    }

    #[test]
    fn plain_const_and_function_populate_root() {
        let gc = collect(r#"const y = 2; function g() {}"#);
        assert!(gc.root.contains_key("y"), "const y must appear in root");
        assert!(gc.root.contains_key("g"), "function g must appear in root");
    }

    #[test]
    fn import_declaration_does_not_appear_in_root() {
        let gc = collect(r#"import { foo } from "bar"; const x = 1;"#);
        assert!(
            !gc.root.contains_key("foo"),
            "Import 'foo' must NOT appear in root"
        );
        assert!(
            gc.root.contains_key("x"),
            "Plain const 'x' must appear in root"
        );
    }

    // -----------------------------------------------------------------------
    // get_imported_local (reverse lookup) tests
    // -----------------------------------------------------------------------

    #[test]
    fn get_imported_local_returns_local_name() {
        let gc = collect(r#"import { foo } from "bar";"#);
        let local = gc.get_imported_local("foo", "bar");
        assert_eq!(local, Some("foo"), "get_imported_local should return 'foo'");
    }

    #[test]
    fn get_imported_local_returns_none_for_missing() {
        let gc = collect(r#"import { foo } from "bar";"#);
        let local = gc.get_imported_local("missing", "bar");
        assert_eq!(
            local, None,
            "get_imported_local should return None for unknown specifier"
        );
    }

    #[test]
    fn get_imported_local_wrong_source_returns_none() {
        let gc = collect(r#"import { foo } from "bar";"#);
        let local = gc.get_imported_local("foo", "wrong-source");
        assert_eq!(
            local, None,
            "get_imported_local should return None for wrong source"
        );
    }

    // -----------------------------------------------------------------------
    // is_import and get_import tests
    // -----------------------------------------------------------------------

    #[test]
    fn is_import_true_for_imported_name() {
        let gc = collect(r#"import { $ } from '@qwik.dev/core';"#);
        assert!(gc.is_import("$"), "$ should be recognized as an import");
    }

    #[test]
    fn is_import_false_for_root_decl() {
        let gc = collect(r#"const x = 1;"#);
        assert!(!gc.is_import("x"), "x should NOT be recognized as an import");
    }

    #[test]
    fn get_import_returns_info() {
        let gc = collect(r#"import { $ } from '@qwik.dev/core';"#);
        let imp = gc.get_import("$").expect("$ should be found");
        assert_eq!(imp.source, "@qwik.dev/core");
    }

    #[test]
    fn get_import_returns_none_for_non_import() {
        let gc = collect(r#"const x = 1;"#);
        assert!(gc.get_import("x").is_none());
    }

    // -----------------------------------------------------------------------
    // Synthetic import tests
    // -----------------------------------------------------------------------

    #[test]
    fn add_synthetic_import_adds_to_imports() {
        let mut gc = GlobalCollect::new_empty();
        gc.add_synthetic_import(
            "_restProps".to_string(),
            Import {
                source: "@qwik.dev/core".to_string(),
                specifier: "_restProps".to_string(),
                kind: ImportKind::Named,
                synthetic: true,
            },
        );
        assert!(gc.is_import("_restProps"));
        assert_eq!(gc.synthetic.len(), 1);
    }

    #[test]
    fn add_synthetic_import_deduplicates() {
        let mut gc = GlobalCollect::new_empty();
        let imp = Import {
            source: "@qwik.dev/core".to_string(),
            specifier: "_restProps".to_string(),
            kind: ImportKind::Named,
            synthetic: true,
        };
        gc.add_synthetic_import("_restProps".to_string(), imp.clone());
        gc.add_synthetic_import("_restProps2".to_string(), imp);
        // Second call is no-op because (specifier, source) already exists
        assert_eq!(gc.synthetic.len(), 1);
    }
}
