//! Dependency analysis for variable migration (Stage 12).
//!
//! This module implements the first four steps of the 10-step variable
//! migration pipeline (SPEC lines 2884-3110):
//!
//! 1. `analyze_root_dependencies` -- extract root-level var/fn/class declarations
//!    with their dependencies and import/export status.
//! 2. `build_root_var_usage_map` -- which segments reference which root vars.
//! 3. `build_main_module_usage_set` -- vars still used by the root module's
//!    non-segment runtime items.
//! 4. `find_migratable_vars` -- 4-condition check + safety fixpoint loop.

use std::collections::{BTreeMap, HashMap, HashSet};

use oxc::ast::ast::*;
use oxc::ast_visit::Visit;
use oxc::span::GetSpan;

use crate::collector::GlobalCollect;
use crate::transform::SegmentRecord;

// ---------------------------------------------------------------------------
// RootVarInfo
// ---------------------------------------------------------------------------

/// Metadata about a single root-level declaration in the parent module.
#[derive(Debug, Clone)]
pub(crate) struct RootVarInfo {
    pub code: String,
    pub is_imported: bool,
    pub is_exported: bool,
    pub depends_on: Vec<String>,
}

// ---------------------------------------------------------------------------
// analyze_root_dependencies
// ---------------------------------------------------------------------------

/// Parse `root_code` and return a map of `var_name -> RootVarInfo` for every
/// top-level declaration.
pub(crate) fn analyze_root_dependencies(
    root_code: &str,
    global: &GlobalCollect,
) -> HashMap<String, RootVarInfo> {
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    let allocator = Allocator::default();
    let src: &str = allocator.alloc_str(root_code);
    let ret = Parser::new(&allocator, src, SourceType::mjs()).parse();
    if ret.panicked {
        return HashMap::new();
    }
    let program = ret.program;

    let mut all_root_names: HashSet<String> = HashSet::new();
    for stmt in &program.body {
        collect_decl_names_stmt(stmt, &mut all_root_names);
    }

    let real_export_locals: HashSet<String> = global
        .exports
        .keys()
        .filter(|k| !k.starts_with("_auto_"))
        .cloned()
        .collect();

    let mut result: HashMap<String, RootVarInfo> = HashMap::new();

    for stmt in &program.body {
        match stmt {
            Statement::VariableDeclaration(decl) => {
                process_var_decl(decl, src, &all_root_names, global, &real_export_locals, &mut result, false);
            }
            Statement::FunctionDeclaration(fn_decl) => {
                let name = match fn_decl.id.as_ref() {
                    Some(id) => id.name.to_string(),
                    None => continue,
                };
                let code = span_to_str(src, fn_decl.span);
                let depends_on = match &fn_decl.body {
                    Some(body) => {
                        let mut c = IdentRefCollector::default();
                        c.visit_function_body(body);
                        c.names.into_iter().filter(|n| all_root_names.contains(n)).collect()
                    }
                    None => Vec::new(),
                };
                result.insert(
                    name.clone(),
                    RootVarInfo {
                        code,
                        is_imported: global.imports.contains_key(&name),
                        is_exported: real_export_locals.contains(&name),
                        depends_on,
                    },
                );
            }
            Statement::ClassDeclaration(cls) => {
                let name = match cls.id.as_ref() {
                    Some(id) => id.name.to_string(),
                    None => continue,
                };
                let code = span_to_str(src, cls.span);
                let depends_on = {
                    let mut c = IdentRefCollector::default();
                    c.visit_class(cls);
                    c.names.into_iter().filter(|n| all_root_names.contains(n)).collect()
                };
                result.insert(
                    name.clone(),
                    RootVarInfo {
                        code,
                        is_imported: global.imports.contains_key(&name),
                        is_exported: real_export_locals.contains(&name),
                        depends_on,
                    },
                );
            }
            Statement::ExportNamedDeclaration(export_decl) => {
                if let Some(decl) = &export_decl.declaration {
                    match decl {
                        Declaration::VariableDeclaration(var_decl) => {
                            process_var_decl(var_decl, src, &all_root_names, global, &real_export_locals, &mut result, true);
                        }
                        Declaration::FunctionDeclaration(fn_decl) => {
                            if let Some(id) = &fn_decl.id {
                                let name = id.name.to_string();
                                let code = span_to_str(src, export_decl.span);
                                let depends_on = match &fn_decl.body {
                                    Some(body) => {
                                        let mut c = IdentRefCollector::default();
                                        c.visit_function_body(body);
                                        c.names.into_iter().filter(|n| all_root_names.contains(n)).collect()
                                    }
                                    None => Vec::new(),
                                };
                                result.insert(name.clone(), RootVarInfo {
                                    code,
                                    is_imported: global.imports.contains_key(&name),
                                    is_exported: true,
                                    depends_on,
                                });
                            }
                        }
                        Declaration::ClassDeclaration(cls) => {
                            if let Some(id) = &cls.id {
                                let name = id.name.to_string();
                                let code = span_to_str(src, export_decl.span);
                                let depends_on = {
                                    let mut c = IdentRefCollector::default();
                                    c.visit_class(cls);
                                    c.names.into_iter().filter(|n| all_root_names.contains(n)).collect()
                                };
                                result.insert(name.clone(), RootVarInfo {
                                    code,
                                    is_imported: global.imports.contains_key(&name),
                                    is_exported: true,
                                    depends_on,
                                });
                            }
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    result
}

fn process_var_decl<'a>(
    decl: &VariableDeclaration<'a>,
    src: &str,
    all_root_names: &HashSet<String>,
    global: &GlobalCollect,
    real_export_locals: &HashSet<String>,
    result: &mut HashMap<String, RootVarInfo>,
    force_exported: bool,
) {
    let kind_str = match decl.kind {
        VariableDeclarationKind::Const => "const",
        VariableDeclarationKind::Let => "let",
        VariableDeclarationKind::Var => "var",
        VariableDeclarationKind::Using | VariableDeclarationKind::AwaitUsing => "const",
    };
    for declarator in &decl.declarations {
        let name = match &declarator.id {
            BindingPattern::BindingIdentifier(id) => id.name.to_string(),
            _ => continue,
        };
        let code = if let Some(init) = &declarator.init {
            let init_code = span_to_str(src, init.span());
            if init_code.is_empty() {
                format!("{} {};", kind_str, name)
            } else {
                format!("{} {} = {};", kind_str, name, init_code)
            }
        } else {
            format!("{} {};", kind_str, name)
        };

        let depends_on = if let Some(init) = &declarator.init {
            let mut c = IdentRefCollector::default();
            c.visit_expression(init);
            c.names.into_iter().filter(|n| all_root_names.contains(n)).collect()
        } else {
            Vec::new()
        };

        let is_exported = force_exported || real_export_locals.contains(&name);

        result.insert(
            name.clone(),
            RootVarInfo {
                code,
                is_imported: global.imports.contains_key(&name),
                is_exported,
                depends_on,
            },
        );
    }
}

// ---------------------------------------------------------------------------
// build_root_var_usage_map
// ---------------------------------------------------------------------------

/// For each root var, find which segments (by index) reference it.
pub(crate) fn build_root_var_usage_map(
    root_deps: &HashMap<String, RootVarInfo>,
    segments: &[SegmentRecord],
) -> HashMap<String, Vec<usize>> {
    let mut map: HashMap<String, Vec<usize>> = HashMap::new();

    for var_name in root_deps.keys() {
        let mut seg_indices: Vec<usize> = Vec::new();

        for (idx, seg) in segments.iter().enumerate() {
            let referenced_in_local = seg.local_idents.contains(var_name);
            let referenced_in_expr = seg.expr.as_ref().map_or(false, |expr_code| {
                contains_whole_word(expr_code, var_name)
            });
            if referenced_in_local || referenced_in_expr {
                seg_indices.push(idx);
            }
        }

        map.insert(var_name.clone(), seg_indices);
    }

    map
}

// ---------------------------------------------------------------------------
// build_main_module_usage_set
// ---------------------------------------------------------------------------

/// Return the set of root-level var names still referenced by the root module's
/// non-segment code.
pub(crate) fn build_main_module_usage_set(
    root_code: &str,
    _segments: &[SegmentRecord],
) -> HashSet<String> {
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    let allocator = Allocator::default();
    let src: &str = allocator.alloc_str(root_code);
    let ret = Parser::new(&allocator, src, SourceType::mjs()).parse();
    if ret.panicked {
        return HashSet::new();
    }
    let program = ret.program;

    let mut usage: HashSet<String> = HashSet::new();

    for stmt in &program.body {
        let is_qrl_call_stmt = match stmt {
            Statement::ExpressionStatement(es) => match &es.expression {
                Expression::CallExpression(call) => match &call.callee {
                    Expression::Identifier(id) => {
                        let name = id.name.as_str();
                        name.ends_with("Qrl") || name.ends_with("QrlDEV")
                    }
                    _ => false,
                },
                _ => false,
            },
            _ => false,
        };

        if !is_qrl_call_stmt {
            let mut collector = IdentRefCollector::default();
            collector.visit_statement(stmt);
            for name in collector.names {
                usage.insert(name);
            }
        }
    }

    usage
}

// ---------------------------------------------------------------------------
// find_migratable_vars
// ---------------------------------------------------------------------------

/// Apply the 4-condition check + safety fixpoint to determine which vars can
/// be migrated into which segments.
pub(crate) fn find_migratable_vars(
    root_deps: &HashMap<String, RootVarInfo>,
    usage_map: &HashMap<String, Vec<usize>>,
    main_usage: &HashSet<String>,
) -> BTreeMap<usize, Vec<String>> {
    let mut candidates: HashMap<String, usize> = HashMap::new();

    for (var_name, info) in root_deps {
        if info.is_imported {
            continue;
        }
        if info.is_exported {
            continue;
        }
        let seg_usages = match usage_map.get(var_name) {
            Some(v) => v,
            None => continue,
        };
        if seg_usages.len() != 1 {
            continue;
        }
        if main_usage.contains(var_name) {
            continue;
        }

        candidates.insert(var_name.clone(), seg_usages[0]);
    }

    // Safety fixpoint
    loop {
        let mut removed: Vec<String> = Vec::new();

        for (var_name, seg_idx) in &candidates {
            let info = match root_deps.get(var_name) {
                Some(i) => i,
                None => {
                    removed.push(var_name.clone());
                    continue;
                }
            };

            for dep in &info.depends_on {
                match candidates.get(dep) {
                    Some(&dep_seg) if dep_seg == *seg_idx => {}
                    Some(_) => {
                        removed.push(var_name.clone());
                        break;
                    }
                    None => {
                        let dep_info = root_deps.get(dep);
                        let dep_is_safe = dep_info.map_or(true, |di| {
                            di.is_imported || di.is_exported
                        });
                        if !dep_is_safe {
                            removed.push(var_name.clone());
                            break;
                        }
                    }
                }
            }
        }

        if removed.is_empty() {
            break;
        }
        for name in removed {
            candidates.remove(&name);
        }
    }

    let mut result: BTreeMap<usize, Vec<String>> = BTreeMap::new();
    for (var_name, seg_idx) in candidates {
        result.entry(seg_idx).or_default().push(var_name);
    }
    for names in result.values_mut() {
        names.sort();
    }

    result
}

// ---------------------------------------------------------------------------
// Internal helpers
// ---------------------------------------------------------------------------

fn collect_decl_names_stmt(stmt: &Statement<'_>, out: &mut HashSet<String>) {
    match stmt {
        Statement::VariableDeclaration(decl) => {
            for d in &decl.declarations {
                if let BindingPattern::BindingIdentifier(id) = &d.id {
                    out.insert(id.name.to_string());
                }
            }
        }
        Statement::FunctionDeclaration(fn_decl) => {
            if let Some(id) = &fn_decl.id {
                out.insert(id.name.to_string());
            }
        }
        Statement::ClassDeclaration(cls) => {
            if let Some(id) = &cls.id {
                out.insert(id.name.to_string());
            }
        }
        Statement::ExportNamedDeclaration(export_decl) => {
            if let Some(decl) = &export_decl.declaration {
                match decl {
                    Declaration::VariableDeclaration(var_decl) => {
                        for d in &var_decl.declarations {
                            if let BindingPattern::BindingIdentifier(id) = &d.id {
                                out.insert(id.name.to_string());
                            }
                        }
                    }
                    Declaration::FunctionDeclaration(fn_decl) => {
                        if let Some(id) = &fn_decl.id {
                            out.insert(id.name.to_string());
                        }
                    }
                    Declaration::ClassDeclaration(cls) => {
                        if let Some(id) = &cls.id {
                            out.insert(id.name.to_string());
                        }
                    }
                    _ => {}
                }
            }
        }
        _ => {}
    }
}

fn span_to_str(src: &str, span: oxc::span::Span) -> String {
    let start = span.start as usize;
    let end = span.end as usize;
    if start <= end && end <= src.len() {
        src[start..end].to_string()
    } else {
        String::new()
    }
}

/// Check whether `text` contains `word` as a whole-word match.
pub(crate) fn contains_whole_word(text: &str, word: &str) -> bool {
    let wlen = word.len();
    let tlen = text.len();
    if wlen > tlen {
        return false;
    }
    let bytes = text.as_bytes();
    let wbytes = word.as_bytes();
    let mut start = 0;
    while start + wlen <= tlen {
        if bytes[start..start + wlen] == *wbytes {
            let pre_ok = start == 0 || !is_ident_char(bytes[start - 1]);
            let post_ok = start + wlen == tlen || !is_ident_char(bytes[start + wlen]);
            if pre_ok && post_ok {
                return true;
            }
        }
        start += 1;
    }
    false
}

fn is_ident_char(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

// ---------------------------------------------------------------------------
// IdentRefCollector
// ---------------------------------------------------------------------------

#[derive(Default)]
pub(crate) struct IdentRefCollector {
    pub names: Vec<String>,
}

impl<'a> Visit<'a> for IdentRefCollector {
    fn visit_identifier_reference(&mut self, ident: &IdentifierReference<'a>) {
        self.names.push(ident.name.to_string());
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collector::global_collect_from_str;
    use crate::types::CtxKind;

    fn empty_global() -> GlobalCollect {
        GlobalCollect::new_empty()
    }

    fn make_segment(name: &str, local_idents: Vec<&str>, expr: Option<&str>) -> SegmentRecord {
        SegmentRecord {
            name: name.to_string(),
            display_name: name.to_string(),
            canonical_filename: name.to_string(),
            entry: None,
            expr: expr.map(|s| s.to_string()),
            scoped_idents: Vec::new(),
            local_idents: local_idents.into_iter().map(|s| s.to_string()).collect(),
            ctx_name: "component$".to_string(),
            ctx_kind: CtxKind::Function,
            origin: "test.tsx".to_string(),
            span: (0, 0),
            hash: "abc12345678".to_string(),
            is_inline: false,
            migrated_root_vars: Vec::new(),
            parent: None,
            pending_parent_span: None,
            param_names: None,
        }
    }

    #[test]
    fn analyze_root_dependencies_finds_const_decl() {
        let code = "const THRESHOLD = 100;";
        let global = empty_global();
        let result = analyze_root_dependencies(code, &global);
        assert!(result.contains_key("THRESHOLD"));
        let info = &result["THRESHOLD"];
        assert!(!info.is_imported);
        assert!(!info.is_exported);
    }

    #[test]
    fn build_root_var_usage_map_finds_segment_using_var() {
        let code = "const THRESHOLD = 100;";
        let global = empty_global();
        let root_deps = analyze_root_dependencies(code, &global);
        let segments = vec![make_segment("s_abc", vec!["THRESHOLD"], Some("() => THRESHOLD > 50"))];
        let map = build_root_var_usage_map(&root_deps, &segments);
        let usages = map.get("THRESHOLD").expect("THRESHOLD should be in map");
        assert_eq!(usages, &[0usize]);
    }

    #[test]
    fn find_migratable_vars_basic_single_segment() {
        let code = "const THRESHOLD = 100;";
        let global = empty_global();
        let root_deps = analyze_root_dependencies(code, &global);
        let segments = vec![make_segment("s_a", vec!["THRESHOLD"], Some("() => THRESHOLD"))];
        let usage_map = build_root_var_usage_map(&root_deps, &segments);
        let main_usage = HashSet::new();
        let result = find_migratable_vars(&root_deps, &usage_map, &main_usage);
        assert!(result.contains_key(&0));
        assert!(result[&0].contains(&"THRESHOLD".to_string()));
    }

    #[test]
    fn find_migratable_vars_shared_var_not_migrated() {
        let code = "const SHARED = 42;";
        let global = empty_global();
        let root_deps = analyze_root_dependencies(code, &global);
        let segments = vec![
            make_segment("s_a", vec!["SHARED"], Some("() => SHARED")),
            make_segment("s_b", vec!["SHARED"], Some("() => SHARED * 2")),
        ];
        let usage_map = build_root_var_usage_map(&root_deps, &segments);
        let main_usage = HashSet::new();
        let result = find_migratable_vars(&root_deps, &usage_map, &main_usage);
        for vars in result.values() {
            assert!(!vars.contains(&"SHARED".to_string()));
        }
    }

    #[test]
    fn contains_whole_word_basic() {
        assert!(contains_whole_word("THRESHOLD > 50", "THRESHOLD"));
        assert!(!contains_whole_word("THRESHOLD > 50", "THRESH"));
        assert!(!contains_whole_word("MY_THRESHOLD > 50", "THRESHOLD"));
    }
}
