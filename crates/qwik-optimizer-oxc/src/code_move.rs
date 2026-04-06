//! Segment module construction helpers.
//!
//! This module provides string-level building blocks used by the `new_module`
//! 13-step pipeline (Steps 1-13) and `emit_segment` for final codegen.
//!
//! All helpers operate on code strings (not AST nodes) and produce code strings.
//! OXC parsing is used only in `emit_segment` for normalizing the assembled module
//! and in `transform_function_expr` for inspecting the expression type.

use std::collections::{HashMap, HashSet, VecDeque};
use std::path::PathBuf;

use crate::collector::GlobalCollect;
use crate::transform::HoistedConst;

// ---------------------------------------------------------------------------
// emit_segment
// ---------------------------------------------------------------------------

/// Parse `raw_code` via OXC (MJS source type), run Codegen, and return the
/// emitted code (and optionally a source map JSON string).
///
/// On parse failure or panic the raw code is returned unchanged with no map.
///
/// Double-quote normalization comes for free from OXC's Codegen default.
pub(crate) fn emit_segment(
    raw_code: &str,
    filename: &str,
    source_maps: bool,
) -> (String, Option<String>) {
    use oxc::allocator::Allocator;
    use oxc::codegen::CodegenOptions;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    let allocator = Allocator::default();
    let source: &str = allocator.alloc_str(raw_code);
    let ret = Parser::new(&allocator, source, SourceType::mjs()).parse();
    if ret.panicked {
        return (raw_code.to_string(), None);
    }
    let program = ret.program;

    if source_maps {
        let codegen_options = CodegenOptions {
            source_map_path: Some(PathBuf::from(filename)),
            ..Default::default()
        };
        let result = oxc::codegen::Codegen::new()
            .with_options(codegen_options)
            .with_source_text(source)
            .build(&program);
        let map = result.map.map(|sm| sm.to_json_string());
        (result.code, map)
    } else {
        let result = oxc::codegen::Codegen::new()
            .with_source_text(source)
            .build(&program);
        (result.code, None)
    }
}

// ---------------------------------------------------------------------------
// new_module_captures_import
// ---------------------------------------------------------------------------

/// Produce the captures import statement for a segment module.
///
/// Example: `import { _captures } from "@qwik.dev/core";`
pub(crate) fn new_module_captures_import(core_module: &str) -> String {
    format!(r#"import {{ _captures }} from "{}";"#, core_module)
}

// ---------------------------------------------------------------------------
// read_captures
// ---------------------------------------------------------------------------

/// Produce individual `const` destructuring statements for captured idents.
///
/// SPEC Pitfall 5: uses individual index access, NOT array destructuring.
///
/// Example for `["count", "signal"]`:
/// ```text
/// const count = _captures[0];
/// const signal = _captures[1];
/// ```
pub(crate) fn read_captures(scoped_idents: &[String]) -> String {
    scoped_idents
        .iter()
        .enumerate()
        .map(|(i, name)| format!("const {} = _captures[{}];\n", name, i))
        .collect()
}

// ---------------------------------------------------------------------------
// inject_use_hmr -- HMR hook injection (D-41)
// ---------------------------------------------------------------------------

/// Inject `_useHmr("dev_path");` as the first statement in a component$
/// function body for HMR mode.
///
/// Only called for `component$` segments when `EmitMode::Hmr` is active.
/// The injection statement is prepended after the opening brace of the
/// function/arrow body.
pub(crate) fn inject_use_hmr(expr_code: &str, dev_path: &str) -> String {
    let hmr_stmt = format!("\n_useHmr(\"{}\");\n", dev_path);

    // Try arrow function with block body first: () => { ... }
    if let Some(arrow_pos) = expr_code.find("=>") {
        let after_arrow = expr_code[arrow_pos + 2..].trim_start();
        if after_arrow.starts_with('{') {
            return prepend_into_block_arrow(expr_code, &hmr_stmt);
        }
        // Concise arrow: () => expr -- wrap in block with _useHmr + return
        let body_expr = after_arrow;
        let params_code = extract_arrow_params(expr_code);
        return format!(
            "({}) => {{\n_useHmr(\"{}\");\nreturn {};\n}}",
            params_code, dev_path, body_expr
        );
    }

    // Try function expression: function(...) { ... }
    prepend_into_function_body(expr_code, &hmr_stmt)
}

// ---------------------------------------------------------------------------
// transform_function_expr
// ---------------------------------------------------------------------------

/// Prepend `read_captures` statements into a function expression or arrow body.
///
/// If `scoped_idents` is empty, returns `expr_code` unchanged.
pub(crate) fn transform_function_expr(expr_code: &str, scoped_idents: &[String]) -> String {
    if scoped_idents.is_empty() {
        return expr_code.to_string();
    }

    let captures = read_captures(scoped_idents);

    // Wrap as a var declaration so we can parse expr_code as an expression.
    let wrapped = format!("var _x = {};", expr_code);

    use oxc::allocator::Allocator;
    use oxc::ast::ast::{Expression, Statement};
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    let allocator = Allocator::default();
    let src: &str = allocator.alloc_str(&wrapped);
    let ret = Parser::new(&allocator, src, SourceType::mjs()).parse();
    if ret.panicked || ret.program.body.is_empty() {
        return expr_code.to_string();
    }

    // SAFETY: program borrows from allocator; allocator lives for this function.
    let program: oxc::ast::ast::Program<'static> = unsafe {
        std::mem::transmute::<oxc::ast::ast::Program<'_>, oxc::ast::ast::Program<'static>>(
            ret.program,
        )
    };

    let stmt = match program.body.first() {
        Some(s) => s,
        None => return expr_code.to_string(),
    };

    // Extract the initializer expression from `var _x = <expr>`.
    let init_expr = match stmt {
        Statement::VariableDeclaration(var_decl) => match var_decl.declarations.first() {
            Some(decl) => decl.init.as_ref(),
            None => return expr_code.to_string(),
        },
        _ => return expr_code.to_string(),
    };

    let expr = match init_expr {
        Some(e) => e,
        None => return expr_code.to_string(),
    };

    match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            if arrow.expression {
                // Concise body: () => expr
                let body_expr_code = if arrow.body.statements.first().is_some() {
                    extract_arrow_concise_body(expr_code)
                } else {
                    return expr_code.to_string();
                };

                // Build new arrow: (params) => { read_captures; return body; }
                let params_code = extract_arrow_params(expr_code);
                format!(
                    "({}) => {{\n{}return {};\n}}",
                    params_code, captures, body_expr_code
                )
            } else {
                // Block body: () => { ... }
                prepend_into_block_arrow(expr_code, &captures)
            }
        }
        Expression::FunctionExpression(_) => {
            // function(...) { ... }
            prepend_into_function_body(expr_code, &captures)
        }
        _ => expr_code.to_string(),
    }
}

/// Extract the parameter string from an arrow like `(a, b) => ...` -> `"a, b"`.
fn extract_arrow_params(expr_code: &str) -> String {
    let arrow_pos = match expr_code.find("=>") {
        Some(p) => p,
        None => return String::new(),
    };
    let params_part = expr_code[..arrow_pos].trim();
    if params_part.starts_with('(') && params_part.ends_with(')') {
        params_part[1..params_part.len() - 1].to_string()
    } else {
        params_part.to_string()
    }
}

/// Extract the concise body from an arrow like `(a) => expr` -> `"expr"`.
fn extract_arrow_concise_body(expr_code: &str) -> String {
    let arrow_pos = match expr_code.find("=>") {
        Some(p) => p,
        None => return expr_code.to_string(),
    };
    expr_code[arrow_pos + 2..].trim().to_string()
}

/// Prepend `captures` text after the `{` that opens the arrow body.
fn prepend_into_block_arrow(expr_code: &str, captures: &str) -> String {
    if let Some(arrow_pos) = expr_code.find("=>") {
        let after_arrow = expr_code[arrow_pos + 2..].trim_start();
        if after_arrow.starts_with('{') {
            let before_brace = &expr_code[..arrow_pos + 2 + (expr_code[arrow_pos + 2..].len() - after_arrow.len())];
            let after_brace = &after_arrow[1..]; // skip the `{`
            return format!("{}{{ {}{}", before_brace, captures, after_brace);
        }
    }
    expr_code.to_string()
}

/// Prepend `captures` text after the first `{` in a function expression body.
fn prepend_into_function_body(expr_code: &str, captures: &str) -> String {
    if let Some(brace_pos) = find_function_body_brace(expr_code) {
        let before = &expr_code[..brace_pos + 1];
        let after = &expr_code[brace_pos + 1..];
        format!("{}{}{}", before, captures, after)
    } else {
        expr_code.to_string()
    }
}

/// Find the opening brace of the function body in a function expression.
fn find_function_body_brace(code: &str) -> Option<usize> {
    let mut depth = 0i32;
    let mut in_params = false;
    let bytes = code.as_bytes();
    let mut paren_close: Option<usize> = None;

    for (i, &b) in bytes.iter().enumerate() {
        if b == b'(' {
            in_params = true;
            depth += 1;
        } else if b == b')' && in_params {
            depth -= 1;
            if depth == 0 {
                paren_close = Some(i);
                break;
            }
        }
    }

    let close_paren = paren_close?;
    for (offset, &b) in bytes[close_paren + 1..].iter().enumerate() {
        if b == b'{' {
            return Some(close_paren + 1 + offset);
        }
    }
    None
}

// ---------------------------------------------------------------------------
// hoist_qrls_from_expr
// ---------------------------------------------------------------------------

/// Scan `expr_code` for `qrl(` and `inlinedQrl(` call patterns.
///
/// For each found, extracts the symbol name (second string argument) and
/// returns:
/// - `modified_expr`: the expression with QRL calls replaced by symbol references
/// - `hoisted_pairs`: `(symbol_name, full_call_code)` -- each becomes `var {name} = {call};`
pub(crate) fn hoist_qrls_from_expr(expr_code: &str) -> (String, Vec<(String, String)>) {
    let mut result = expr_code.to_string();
    let mut hoisted: Vec<(String, String)> = Vec::new();

    for prefix in &["inlinedQrl(", "qrl("] {
        loop {
            let start = match result.find(prefix) {
                Some(p) => p,
                None => break,
            };
            let call_start = start;
            let inner_start = start + prefix.len();
            let mut depth = 1i32;
            let mut end_pos = inner_start;
            let bytes = result.as_bytes();
            for (i, &b) in bytes[inner_start..].iter().enumerate() {
                match b {
                    b'(' => depth += 1,
                    b')' => {
                        depth -= 1;
                        if depth == 0 {
                            end_pos = inner_start + i;
                            break;
                        }
                    }
                    _ => {}
                }
            }
            let full_call = &result[call_start..end_pos + 1];
            let inner_args = &result[inner_start..end_pos];

            let sym_name = extract_second_string_arg(inner_args);
            match sym_name {
                Some(name) => {
                    hoisted.push((name.clone(), full_call.to_string()));
                    result = format!("{}{}{}", &result[..call_start], name, &result[end_pos + 1..]);
                }
                None => break,
            }
        }
    }

    (result, hoisted)
}

/// Extract the second string argument from a function call's argument list.
fn extract_second_string_arg(args: &str) -> Option<String> {
    let mut depth = 0i32;
    let mut first_comma: Option<usize> = None;
    let bytes = args.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        match b {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                first_comma = Some(i);
                break;
            }
            _ => {}
        }
    }

    let second_arg_start = first_comma? + 1;
    let second_arg = args[second_arg_start..].trim();

    if (second_arg.starts_with('"') && second_arg.contains('"'))
        || (second_arg.starts_with('\'') && second_arg.contains('\''))
    {
        let quote = second_arg.chars().next()?;
        let inner = &second_arg[1..];
        let close = inner.find(quote)?;
        Some(inner[..close].to_string())
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// fix_self_referential_vars
// ---------------------------------------------------------------------------

/// Rewrite `const X = ...X...` patterns where the initializer references X.
pub(crate) fn fix_self_referential_vars(body_code: &str) -> String {
    let mut lines: Vec<String> = body_code.lines().map(|l| l.to_string()).collect();
    let mut i = 0;
    while i < lines.len() {
        let line = lines[i].trim();
        if let Some(rest) = line.strip_prefix("const ") {
            if let Some(eq_pos) = rest.find(" = ") {
                let name = rest[..eq_pos].trim();
                if name.chars().all(|c| c.is_alphanumeric() || c == '_' || c == '$') {
                    let init_with_semi = rest[eq_pos + 3..].trim();
                    let init = init_with_semi.trim_end_matches(';').trim();
                    if contains_word(init, name) {
                        let new_init = replace_word(init, name, &format!("_ref.{}", name));
                        let replacement = vec![
                            format!("{}const _ref = {{}};", &lines[i][..lines[i].len() - line.len()]),
                            format!("{}_ref.{} = {};", &lines[i][..lines[i].len() - line.len()], name, new_init),
                            format!("{}const {{ {} }} = _ref;", &lines[i][..lines[i].len() - line.len()], name),
                        ];
                        lines.splice(i..=i, replacement);
                        i += 3;
                        continue;
                    }
                }
            }
        }
        i += 1;
    }
    lines.join("\n")
}

/// Check if `word` appears as a whole word in `text`.
fn contains_word(text: &str, word: &str) -> bool {
    let mut start = 0;
    while let Some(pos) = text[start..].find(word) {
        let abs_pos = start + pos;
        let before_ok = abs_pos == 0
            || !text
                .chars()
                .nth(abs_pos.saturating_sub(1))
                .map(|c| c.is_alphanumeric() || c == '_' || c == '$')
                .unwrap_or(false);
        let after_ok = abs_pos + word.len() >= text.len()
            || !text
                .chars()
                .nth(abs_pos + word.len())
                .map(|c| c.is_alphanumeric() || c == '_' || c == '$')
                .unwrap_or(false);
        if before_ok && after_ok {
            return true;
        }
        start = abs_pos + 1;
        if start >= text.len() {
            break;
        }
    }
    false
}

/// Replace all whole-word occurrences of `word` in `text` with `replacement`.
fn replace_word(text: &str, word: &str, replacement: &str) -> String {
    let mut result = String::new();
    let mut start = 0;
    while let Some(pos) = text[start..].find(word) {
        let abs_pos = start + pos;
        let before_ok = abs_pos == 0
            || !text
                .chars()
                .nth(abs_pos.saturating_sub(1))
                .map(|c| c.is_alphanumeric() || c == '_' || c == '$')
                .unwrap_or(false);
        let after_ok = abs_pos + word.len() >= text.len()
            || !text
                .chars()
                .nth(abs_pos + word.len())
                .map(|c| c.is_alphanumeric() || c == '_' || c == '$')
                .unwrap_or(false);
        if before_ok && after_ok {
            result.push_str(&text[start..abs_pos]);
            result.push_str(replacement);
            start = abs_pos + word.len();
        } else {
            result.push_str(&text[start..abs_pos + 1]);
            start = abs_pos + 1;
        }
        if start >= text.len() {
            break;
        }
    }
    result.push_str(&text[start..]);
    result
}

// ---------------------------------------------------------------------------
// create_named_export
// ---------------------------------------------------------------------------

/// Produce a named export statement: `export const name = <expr>;`.
pub(crate) fn create_named_export(name: &str, expr_code: &str) -> String {
    format!("export const {} = {};", name, expr_code)
}

// ---------------------------------------------------------------------------
// word_idents_in -- extract identifiers from code string
// ---------------------------------------------------------------------------

/// Extract all identifiers from a code string using word-boundary scanning.
fn word_idents_in(code: &str) -> HashSet<String> {
    let mut result = HashSet::new();
    let bytes = code.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        let b = bytes[i];
        if b.is_ascii_alphabetic() || b == b'_' || b == b'$' {
            let start = i;
            while i < bytes.len()
                && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'$')
            {
                i += 1;
            }
            let ident = &code[start..i];
            result.insert(ident.to_string());
        } else {
            i += 1;
        }
    }
    result
}

// ---------------------------------------------------------------------------
// generate_imports -- SPEC Step 7: 4-priority cascade import generation
// ---------------------------------------------------------------------------

/// Generate import statements for a list of identifiers used in a segment module.
pub(crate) fn generate_imports(
    combined_local_idents: &[String],
    global: &GlobalCollect,
    file_stem: &str,
    explicit_extensions: bool,
    _core_module: &str,
) -> Vec<String> {
    let mut seen_import_names: HashMap<String, Vec<(String, String)>> = HashMap::new();

    for ident in combined_local_idents {
        // Priority 1: exact match in global.imports by local name
        if let Some(import) = global.imports.get(ident.as_str()) {
            seen_import_names
                .entry(ident.clone())
                .or_default()
                .push((import.specifier.clone(), import.source.clone()));
            continue;
        }

        // Priority 3: scan for specifier == ident (single match)
        let specifier_matches: Vec<(&String, &crate::collector::Import)> = global
            .imports
            .iter()
            .filter(|(_local, imp)| imp.specifier == *ident)
            .collect();
        if specifier_matches.len() == 1 {
            let (_, imp) = specifier_matches[0];
            seen_import_names
                .entry(ident.clone())
                .or_default()
                .push((imp.specifier.clone(), imp.source.clone()));
            continue;
        }

        // Priority 4: ident is exported from the parent file
        if global.has_export_symbol(ident) {
            let export_name = global
                .resolve_export_for_id(ident)
                .unwrap_or_else(|| ident.clone());
            let source = if explicit_extensions {
                format!("./{}.js", file_stem)
            } else {
                format!("./{}", file_stem)
            };
            seen_import_names
                .entry(ident.clone())
                .or_default()
                .push((export_name, source));
            continue;
        }
    }

    // Pass 2: emit import statements sorted by local name for stability.
    let mut spec_source_to_local: HashMap<(String, String), String> = HashMap::new();
    let mut result: Vec<String> = Vec::new();

    let mut keys: Vec<String> = seen_import_names.keys().cloned().collect();
    keys.sort();

    for local in &keys {
        let entries = &seen_import_names[local];
        if entries.is_empty() {
            continue;
        }

        if entries.len() == 1 {
            let (specifier, source) = &entries[0];
            let key = (specifier.clone(), source.clone());
            if spec_source_to_local.contains_key(&key) {
                let suffix_local = format!("_{}_1", local);
                let stmt = format_import_stmt(specifier, &suffix_local, source);
                result.push(stmt);
            } else {
                spec_source_to_local.insert(key, local.clone());
                let stmt = format_import_stmt(specifier, local, source);
                result.push(stmt);
            }
        } else {
            for (i, (specifier, source)) in entries.iter().enumerate() {
                let effective_local = if i == 0 {
                    local.clone()
                } else {
                    format!("_{}_{}", local, i)
                };
                let key = (specifier.clone(), source.clone());
                spec_source_to_local.insert(key, effective_local.clone());
                let stmt = format_import_stmt(specifier, &effective_local, source);
                result.push(stmt);
            }
        }
    }

    result
}

/// Format a single import statement.
fn format_import_stmt(specifier: &str, local: &str, source: &str) -> String {
    if specifier == local {
        format!(r#"import {{ {} }} from "{}";"#, specifier, source)
    } else {
        format!(r#"import {{ {} as {} }} from "{}";"#, specifier, local, source)
    }
}

// ---------------------------------------------------------------------------
// collect_needed_extra_top_items -- SPEC Steps 5/8
// ---------------------------------------------------------------------------

/// Transitively expand the set of needed `HoistedConst` items from a seed set.
pub(crate) fn collect_needed_extra_top_items(
    extra_top_items: &[HoistedConst],
    seed_idents: &HashSet<String>,
) -> Vec<HoistedConst> {
    let mut needed: HashSet<String> = seed_idents.clone();

    loop {
        let before = needed.len();
        for item in extra_top_items {
            if needed.contains(&item.name) {
                let rhs_idents = word_idents_in(&item.rhs_code);
                for ident in rhs_idents {
                    needed.insert(ident);
                }
            }
        }
        if needed.len() == before {
            break;
        }
    }

    extra_top_items
        .iter()
        .filter(|item| needed.contains(&item.name))
        .map(|item| HoistedConst {
            name: item.name.clone(),
            rhs_code: item.rhs_code.clone(),
            symbol_name: item.symbol_name.clone(),
        })
        .collect()
}

// ---------------------------------------------------------------------------
// order_items_by_dependency -- SPEC Step 11: Kahn's topological sort
// ---------------------------------------------------------------------------

/// Sort (sym_name, code_string) pairs by their dependency order using Kahn's BFS.
pub(crate) fn order_items_by_dependency(items: Vec<(String, String)>) -> Vec<(String, String)> {
    if items.is_empty() {
        return items;
    }

    let sym_to_idx: HashMap<&str, usize> = items
        .iter()
        .enumerate()
        .map(|(i, (sym, _))| (sym.as_str(), i))
        .collect();

    let n = items.len();
    let mut in_degree = vec![0usize; n];
    let mut adjacency: Vec<Vec<usize>> = vec![Vec::new(); n];

    for (i, (sym, code)) in items.iter().enumerate() {
        let used = word_idents_in(code);
        for dep_sym in &used {
            if dep_sym == sym {
                continue;
            }
            if let Some(&j) = sym_to_idx.get(dep_sym.as_str()) {
                in_degree[i] += 1;
                adjacency[j].push(i);
            }
        }
    }

    let mut queue: VecDeque<usize> = {
        let mut zero: Vec<usize> = (0..n).filter(|&i| in_degree[i] == 0).collect();
        zero.sort_by_key(|&i| &items[i].0);
        zero.into_iter().collect()
    };

    let mut sorted: Vec<(String, String)> = Vec::with_capacity(n);
    let mut visited = vec![false; n];

    while let Some(idx) = queue.pop_front() {
        if visited[idx] {
            continue;
        }
        visited[idx] = true;
        sorted.push(items[idx].clone());

        let mut newly_zero: Vec<usize> = Vec::new();
        for &dep_idx in &adjacency[idx] {
            if !visited[dep_idx] {
                in_degree[dep_idx] -= 1;
                if in_degree[dep_idx] == 0 {
                    newly_zero.push(dep_idx);
                }
            }
        }
        newly_zero.sort_by_key(|&i| &items[i].0);
        for idx in newly_zero {
            queue.push_back(idx);
        }
    }

    // Cycle handling
    if sorted.len() < n {
        for (i, (sym, code)) in items.iter().enumerate() {
            if visited[i] {
                continue;
            }
            if code.contains("qrl(") || code.contains("inlinedQrl(") {
                sorted.insert(0, (sym.clone(), format!("let {};", sym)));
                sorted.push((sym.clone(), format!("{} = {};", sym, code)));
            } else {
                sorted.push((sym.clone(), code.clone()));
            }
        }
    }

    sorted
}

// ---------------------------------------------------------------------------
// dedup_by_sym -- SPEC Step 12: final symbol deduplication
// ---------------------------------------------------------------------------

/// Remove duplicate items by their defined symbol name.
pub(crate) fn dedup_by_sym(items: Vec<String>) -> Vec<String> {
    let mut seen: HashSet<String> = HashSet::new();
    let mut result: Vec<String> = Vec::with_capacity(items.len());

    for item in items {
        if let Some(sym) = extract_defined_sym(&item) {
            if seen.contains(&sym) {
                continue;
            }
            seen.insert(sym);
        }
        result.push(item);
    }

    result
}

/// Extract the first defined symbol from a code item string.
fn extract_defined_sym(item: &str) -> Option<String> {
    let s = item.trim();

    if let Some(rest) = s.strip_prefix("import {") {
        let inner = rest.trim();
        let sym = inner
            .split(|c: char| !c.is_alphanumeric() && c != '_' && c != '$')
            .next()
            .filter(|s| !s.is_empty())
            .map(|s| s.to_string());
        return sym;
    }

    if let Some(rest) = s.strip_prefix("export const ") {
        return extract_first_ident(rest);
    }
    if let Some(rest) = s.strip_prefix("export let ") {
        return extract_first_ident(rest);
    }
    if let Some(rest) = s.strip_prefix("export var ") {
        return extract_first_ident(rest);
    }
    if let Some(rest) = s.strip_prefix("const ") {
        return extract_first_ident(rest);
    }
    if let Some(rest) = s.strip_prefix("var ") {
        return extract_first_ident(rest);
    }
    if let Some(rest) = s.strip_prefix("let ") {
        return extract_first_ident(rest);
    }

    None
}

fn extract_first_ident(s: &str) -> Option<String> {
    let trimmed = s.trim();
    let sym: String = trimmed
        .chars()
        .take_while(|c| c.is_alphanumeric() || *c == '_' || *c == '$')
        .collect();
    if sym.is_empty() {
        None
    } else {
        Some(sym)
    }
}

// ---------------------------------------------------------------------------
// NewModuleCtx + new_module -- SPEC 13-step pipeline
// ---------------------------------------------------------------------------

/// Context passed to `new_module` to generate a complete segment module string.
pub(crate) struct NewModuleCtx<'a> {
    pub expr: &'a str,
    pub name: &'a str,
    pub file_stem: &'a str,
    pub local_idents: &'a [String],
    pub scoped_idents: &'a [String],
    pub global: &'a GlobalCollect,
    pub core_module: &'a str,
    pub explicit_extensions: bool,
    pub extra_top_items: &'a [HoistedConst],
    pub migrated_root_vars: &'a [String],
    /// Additional synthetic import statements to prepend (e.g., HMR _useHmr import).
    pub synthetic_imports: &'a [String],
}

/// Build a complete segment module string from the 13-step pipeline.
pub(crate) fn new_module(ctx: NewModuleCtx<'_>) -> String {
    let mut header_items: Vec<String> = Vec::new();

    // Step 1: captures import
    if !ctx.scoped_idents.is_empty() {
        header_items.push(new_module_captures_import(ctx.core_module));
    }

    // Step 2: inject read_captures into function expression
    let transformed_expr = transform_function_expr(ctx.expr, ctx.scoped_idents);

    // Step 3: hoist QRL calls out of the expression
    let (expr_after_hoist, hoisted_pairs) = hoist_qrls_from_expr(&transformed_expr);

    // Step 4: fix self-referential variables
    let final_expr = fix_self_referential_vars(&expr_after_hoist);

    // Step 5: collect needed extra_top_items
    let mut seed: HashSet<String> = HashSet::new();
    for ident in ctx.local_idents {
        seed.insert(ident.clone());
    }
    for ident in ctx.scoped_idents {
        seed.insert(ident.clone());
    }
    for ident in word_idents_in(&final_expr) {
        seed.insert(ident);
    }
    for (sym, code) in &hoisted_pairs {
        seed.insert(sym.clone());
        for ident in word_idents_in(code) {
            seed.insert(ident);
        }
    }

    let needed_extras = collect_needed_extra_top_items(ctx.extra_top_items, &seed);

    // Step 6: build combined_local_idents
    let mut combined_local_idents: Vec<String> = ctx.local_idents.to_vec();
    for (sym, _code) in &hoisted_pairs {
        if !combined_local_idents.contains(sym) {
            combined_local_idents.push(sym.clone());
        }
    }
    for item in &needed_extras {
        for ident in word_idents_in(&item.rhs_code) {
            if !combined_local_idents.contains(&ident) {
                combined_local_idents.push(ident);
            }
        }
    }

    // Step 7: generate imports
    let import_stmts = generate_imports(
        &combined_local_idents,
        ctx.global,
        ctx.file_stem,
        ctx.explicit_extensions,
        ctx.core_module,
    );

    // Step 8: second collect_needed_extra_top_items
    let needed_extras_2 = collect_needed_extra_top_items(ctx.extra_top_items, &seed);

    // Build set of already-imported symbols for dedup
    let mut imported_syms: HashSet<String> = HashSet::new();
    for stmt in &import_stmts {
        if let Some(sym) = extract_defined_sym(stmt) {
            imported_syms.insert(sym);
        }
    }
    for stmt in &header_items {
        if let Some(sym) = extract_defined_sym(stmt) {
            imported_syms.insert(sym);
        }
    }

    // Step 9: dedup extra_top_items against already-imported symbols
    let deduped_extras: Vec<&HoistedConst> = needed_extras_2
        .iter()
        .filter(|item| !imported_syms.contains(&item.name))
        .collect();

    // Step 10: separate extra_top_items into imports vs non-imports
    let mut extra_imports: Vec<String> = Vec::new();
    let mut extra_non_imports: Vec<(String, String)> = Vec::new();
    for item in &deduped_extras {
        let rhs = item.rhs_code.trim();
        if rhs.starts_with("import ") || rhs.starts_with("import{") {
            extra_imports.push(rhs.to_string());
        } else {
            extra_non_imports.push((item.name.clone(), format!("const {} = {};", item.name, rhs)));
        }
    }

    // Step 11: order hoisted_pairs + extra_non_imports by dependency
    let mut items_to_sort: Vec<(String, String)> = hoisted_pairs
        .iter()
        .map(|(sym, code)| (sym.clone(), format!("var {} = {};", sym, code)))
        .collect();
    items_to_sort.extend(extra_non_imports);

    let sorted_items = order_items_by_dependency(items_to_sort);

    // Step 12: combine all items and dedup
    let mut all_items: Vec<String> = Vec::new();
    all_items.extend(header_items);
    all_items.extend(import_stmts);
    // Synthetic imports (e.g., HMR _useHmr)
    for si in ctx.synthetic_imports {
        all_items.push(si.clone());
    }
    all_items.extend(extra_imports);
    all_items.extend(sorted_items.into_iter().map(|(_sym, code)| code));
    let deduped = dedup_by_sym(all_items);

    // Step 13: append migrated root var declarations
    let mut result_parts = deduped;
    for migrated_stmt in ctx.migrated_root_vars {
        let stmt = migrated_stmt.trim().to_string();
        if !stmt.is_empty() {
            result_parts.push(stmt);
        }
    }

    // Step 14: append named export
    let export_stmt = create_named_export(ctx.name, &final_expr);
    result_parts.push(export_stmt);

    result_parts.join("\n")
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captures_import_generates_correct_import_statement() {
        let result = new_module_captures_import("@qwik.dev/core");
        assert_eq!(result, r#"import { _captures } from "@qwik.dev/core";"#);
    }

    #[test]
    fn read_captures_generates_individual_consts() {
        let idents = vec!["count".to_string(), "signal".to_string()];
        let result = read_captures(&idents);
        assert_eq!(result, "const count = _captures[0];\nconst signal = _captures[1];\n");
    }

    #[test]
    fn transform_function_expr_no_captures_unchanged() {
        let expr = "() => 42";
        let result = transform_function_expr(expr, &[]);
        assert_eq!(result, "() => 42");
    }

    #[test]
    fn hoist_qrls_from_expr_basic() {
        let expr = r#"qrl(() => import("./x"), "s_abc")"#;
        let (modified, hoisted) = hoist_qrls_from_expr(expr);
        assert_eq!(modified, "s_abc");
        assert_eq!(hoisted.len(), 1);
        assert_eq!(hoisted[0].0, "s_abc");
    }

    #[test]
    fn fix_self_referential_var() {
        let body = "const x = fn(x);";
        let result = fix_self_referential_vars(body);
        assert!(result.contains("const _ref = {}"));
        assert!(result.contains("_ref.x = fn(_ref.x)"));
        assert!(result.contains("const { x } = _ref"));
    }

    #[test]
    fn create_named_export_basic() {
        let result = create_named_export("myComp", "componentQrl(...)");
        assert_eq!(result, "export const myComp = componentQrl(...);");
    }

    #[test]
    fn emit_segment_round_trips_code() {
        let code = "const x = 1;\nexport const y = 2;\n";
        let (out, map) = emit_segment(code, "test.js", false);
        assert!(out.contains("x"));
        assert!(out.contains("y"));
        assert!(map.is_none());
    }

    #[test]
    fn new_module_basic_no_captures() {
        let global = GlobalCollect::new_empty();
        let ctx = NewModuleCtx {
            expr: "() => 42",
            name: "s_test",
            file_stem: "app",
            local_idents: &[],
            scoped_idents: &[],
            global: &global,
            core_module: "@qwik.dev/core",
            explicit_extensions: false,
            extra_top_items: &[],
            migrated_root_vars: &[],
            synthetic_imports: &[],
        };
        let result = new_module(ctx);
        assert!(result.contains("export const s_test"));
        assert!(!result.contains("_captures"));
    }

    #[test]
    fn new_module_with_captures() {
        let global = GlobalCollect::new_empty();
        let ctx = NewModuleCtx {
            expr: "() => count",
            name: "s_abc",
            file_stem: "app",
            local_idents: &[],
            scoped_idents: &["count".to_string()],
            global: &global,
            core_module: "@qwik.dev/core",
            explicit_extensions: false,
            extra_top_items: &[],
            migrated_root_vars: &[],
            synthetic_imports: &[],
        };
        let result = new_module(ctx);
        assert!(result.contains(r#"import { _captures } from "@qwik.dev/core";"#));
        assert!(result.contains("export const s_abc"));
    }
}
