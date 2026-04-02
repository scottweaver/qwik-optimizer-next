//! Qwik optimizer using OXC for code transformation.
//!
//! This crate provides the type definitions and utility functions needed for
//! the Qwik $-call extraction pipeline.

pub mod types;
pub mod words;
pub mod hash;
pub mod errors;
pub mod is_const;
pub(crate) mod parse;
pub(crate) mod collector;
pub(crate) mod entry_strategy;
pub(crate) mod rename_imports;
pub(crate) mod const_replace;
pub(crate) mod filter_exports;
pub(crate) mod inlined_fn;
pub(crate) mod jsx_transform;
pub(crate) mod props_destructuring;
pub(crate) mod transform;
pub(crate) mod emit;
pub(crate) mod code_move;
pub(crate) mod clean_side_effects;
pub(crate) mod add_side_effect;
pub(crate) mod dependency_analysis;

// Re-export all public types
pub use types::*;

use std::path::Path;

use oxc::semantic::SemanticBuilder;
use oxc_traverse::traverse_mut;
use path_slash::PathBufExt as _;

use emit::EmitOptions;
use types::TransformCodeOptions;

// ---------------------------------------------------------------------------
// transform_code -- per-file transformation pipeline
// ---------------------------------------------------------------------------

fn transform_code(
    config: &TransformCodeOptions,
    input_code: &str,
    input_path: &str,
    _dev_path: Option<&str>,
) -> TransformOutput {
    // Stage 0: Decompose path.
    let path_data = match parse::parse_path(input_path, Path::new(&config.src_dir)) {
        Ok(pd) => pd,
        Err(_) => {
            return TransformOutput::default();
        }
    };

    // Stage 1: Allocate arena and parse source.
    let allocator = oxc::allocator::Allocator::default();
    let source_in_arena: &str = allocator.alloc_str(input_code);

    let (is_type_script, is_jsx, mut program, diagnostics) =
        match parse::parse_module(&allocator, source_in_arena, input_path) {
            Ok((parse_result, diags)) => {
                let is_ts = parse_result.source_type.is_typescript();
                let is_jsx = parse_result.source_type.is_jsx();
                (is_ts, is_jsx, parse_result.program, diags)
            }
            Err(diags) => {
                return TransformOutput {
                    modules: vec![],
                    diagnostics: diags,
                    is_type_script: false,
                    is_jsx: false,
                };
            }
        };

    // Stage 2: Strip exports (conditional).
    if !config.strip_exports.is_empty() {
        filter_exports::filter_exports(&mut program, &config.strip_exports, &allocator);
    }

    // Stage 5: Import rename (always).
    rename_imports::rename_imports(&mut program, &allocator);

    // Stage 7: Global collect (always).
    let mut collect = collector::global_collect(&program);

    // Stage 8: Props destructuring (always, all modes).
    props_destructuring::transform_props_destructuring(
        &mut program, &mut collect, &config.core_module, &allocator,
    );

    // Stage 9: Const replacement.
    const_replace::replace_build_constants(&mut program, config, &collect, &allocator);

    // Stage 10: QwikTransform -- marker detection, capture analysis, QRL wrapping.
    let semantic_ret = SemanticBuilder::new().build(&program);
    let scoping = semantic_ret.semantic.into_scoping();

    let rel_path: String = if path_data.rel_dir == std::path::PathBuf::new() {
        path_data.file_name.clone()
    } else {
        format!("{}/{}", path_data.rel_dir.to_slash_lossy(), path_data.file_name)
    };

    let file_extension = path_data
        .file_name
        .rsplit('.')
        .next()
        .unwrap_or("js")
        .to_string();

    // Stage 11 (pre-pass): mark pre-transform call/new expression spans for
    // Treeshaker DCE.
    let is_inline_strategy = matches!(
        config.entry_strategy,
        EntryStrategy::Inline | EntryStrategy::Hoist
    );
    let run_treeshaker = !is_inline_strategy
        && !matches!(config.minify, MinifyMode::None)
        && !config.is_server
        && !matches!(config.mode, EmitMode::Lib);

    let mut treeshaker_opt = if run_treeshaker {
        let ts = clean_side_effects::Treeshaker::new();
        ts.marker.mark_module(&program);
        Some(ts)
    } else {
        None
    };

    let mut xfrm = transform::QwikTransform::new(
        config,
        &collect,
        &path_data.file_name,
        &rel_path,
        &file_extension,
        source_in_arena,
    );
    let _scoping = traverse_mut(&mut xfrm, &allocator, &mut program, scoping, ());

    // Stage 11: Post-transform DCE (mutually exclusive branches).
    if is_inline_strategy {
        add_side_effect::add_side_effect_imports(
            &mut program,
            &collect,
            &path_data.abs_dir,
            Path::new(&config.src_dir),
            &allocator,
        );
    } else if let Some(ref mut ts) = treeshaker_opt {
        ts.cleaner.clean_module(&mut program);
    }

    // Stage 12: Variable migration pipeline.
    if !matches!(config.mode, EmitMode::Lib) && !xfrm.segments.is_empty() {
        apply_variable_migration(&mut program, &mut xfrm, &collect, &allocator);
    }

    // Resolve deferred parent symbol names.
    xfrm.patch_segment_parents();

    let did_transform = !xfrm.segments.is_empty();

    // Emit: codegen the transformed AST back to JavaScript.
    let emit_result = emit::emit_module(
        &program,
        source_in_arena,
        &EmitOptions {
            source_maps: config.source_maps,
        },
        input_path,
    );

    // Compute output path.
    let output_path = if did_transform && !config.preserve_filenames {
        let ext = parse::output_extension(input_path, config.transpile_ts, config.transpile_jsx);
        let stem = &path_data.file_stem;
        let new_filename = format!("{stem}.{ext}");
        if path_data.rel_dir == std::path::PathBuf::new() {
            new_filename
        } else {
            path_data
                .rel_dir
                .join(new_filename)
                .to_slash_lossy()
                .into_owned()
        }
    } else {
        if path_data.rel_dir == std::path::PathBuf::new() {
            path_data.file_name.clone()
        } else {
            path_data
                .rel_dir
                .join(&path_data.file_name)
                .to_slash_lossy()
                .into_owned()
        }
    };

    // Root module order: DefaultHasher of the output path bytes (OUT-02).
    let root_order = {
        use std::hash::{Hash, Hasher};
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        output_path.hash(&mut hasher);
        hasher.finish()
    };

    let root_module = TransformModule {
        path: output_path,
        is_entry: false,
        code: emit_result.code,
        map: emit_result.map,
        segment: None,
        orig_path: Some(input_path.to_string()),
        order: root_order,
    };

    // --- Segment module generation ---
    let mut segment_modules: Vec<TransformModule> = Vec::new();
    let record_extension = parse::output_extension(input_path, config.transpile_ts, config.transpile_jsx);

    for record in &xfrm.segments {
        // Skip inline segments (they live in the parent module)
        if record.is_inline {
            continue;
        }
        // Skip noop segments (no expression to emit)
        let expr_code = match &record.expr {
            Some(e) => e.as_str(),
            None => continue,
        };

        // Build segment module code via new_module
        let module_code = code_move::new_module(code_move::NewModuleCtx {
            expr: expr_code,
            name: &record.name,
            file_stem: &path_data.file_stem,
            local_idents: &record.local_idents,
            scoped_idents: &record.scoped_idents,
            global: &collect,
            core_module: &config.core_module,
            explicit_extensions: config.explicit_extensions,
            extra_top_items: &xfrm.extra_top_items,
            migrated_root_vars: &record.migrated_root_vars,
        });

        // Parse + codegen for normalization
        let (final_code, map) = code_move::emit_segment(
            &module_code,
            &record.canonical_filename,
            config.source_maps,
        );

        // Build segment path
        let segment_path = if path_data.rel_dir == std::path::PathBuf::new() {
            format!("{}.{}", record.canonical_filename, record_extension)
        } else {
            format!(
                "{}/{}.{}",
                path_data.rel_dir.to_slash_lossy(),
                record.canonical_filename,
                record_extension
            )
        };

        // SPEC Pitfall 1: is_entry = entry.is_none() (inverted semantics)
        let is_entry = record.entry.is_none();

        // Segment order: parse first 8 chars of hash as base36
        let order = u64::from_str_radix(
            &record.hash[..std::cmp::min(8, record.hash.len())],
            36,
        ).unwrap_or(0);

        let seg_path = path_data.rel_dir.to_slash_lossy().to_string();

        let segment_analysis = SegmentAnalysis {
            origin: record.origin.clone(),
            name: record.name.clone(),
            entry: record.entry.clone(),
            display_name: record.display_name.clone(),
            hash: record.hash.clone(),
            canonical_filename: record.canonical_filename.clone(),
            path: seg_path,
            extension: record_extension.to_string(),
            parent: record.parent.clone(),
            ctx_kind: record.ctx_kind.clone(),
            ctx_name: record.ctx_name.clone(),
            captures: !record.scoped_idents.is_empty(),
            // SWC uses 1-based byte offsets; OXC uses 0-based.
            // Add 1 to both to match SWC's golden span format.
            loc: (record.span.0 + 1, record.span.1 + 1),
            param_names: record.param_names.clone(),
            capture_names: if record.scoped_idents.is_empty() {
                None
            } else {
                Some(record.scoped_idents.clone())
            },
        };

        segment_modules.push(TransformModule {
            path: segment_path,
            is_entry,
            code: final_code,
            map,
            segment: Some(segment_analysis),
            orig_path: None,
            order,
        });
    }

    let mut all_modules = vec![root_module];
    all_modules.append(&mut segment_modules);

    let mut all_diagnostics = diagnostics;
    all_diagnostics.extend(xfrm.diagnostics);

    TransformOutput {
        modules: all_modules,
        diagnostics: all_diagnostics,
        is_type_script,
        is_jsx,
    }
}

// ---------------------------------------------------------------------------
// apply_variable_migration -- Stage 12 implementation
// ---------------------------------------------------------------------------

fn apply_variable_migration<'a>(
    program: &mut oxc::ast::ast::Program<'a>,
    xfrm: &mut transform::QwikTransform,
    collect: &collector::GlobalCollect,
    _allocator: &'a oxc::allocator::Allocator,
) {
    use std::collections::HashSet;

    // Step 1: emit root module code for analysis.
    let root_code = emit::emit_module(
        program,
        "",
        &EmitOptions { source_maps: false },
        "",
    ).code;

    // Steps 2-4: analyze and find migratable vars.
    let root_deps = dependency_analysis::analyze_root_dependencies(&root_code, collect);
    let usage_map = dependency_analysis::build_root_var_usage_map(&root_deps, &xfrm.segments);
    let main_usage = dependency_analysis::build_main_module_usage_set(&root_code, &xfrm.segments);
    let migratable = dependency_analysis::find_migratable_vars(&root_deps, &usage_map, &main_usage);

    if migratable.is_empty() {
        return;
    }

    let all_migrated: HashSet<String> = migratable
        .values()
        .flat_map(|vs| vs.iter().cloned())
        .collect();

    // Step 5: ensure_export for deps of migrated vars that remain in root.
    for var_names in migratable.values() {
        for var_name in var_names {
            if let Some(info) = root_deps.get(var_name) {
                for dep in &info.depends_on {
                    if !all_migrated.contains(dep) {
                        xfrm.ensure_export(dep);
                    }
                }
            }
        }
    }

    // Step 6: populate migrated_root_vars on each SegmentRecord.
    for (seg_idx, var_names) in &migratable {
        if let Some(segment) = xfrm.segments.get_mut(*seg_idx) {
            let mut code_items: Vec<String> = Vec::new();
            for name in var_names {
                if let Some(info) = root_deps.get(name) {
                    if !info.code.is_empty() {
                        code_items.push(info.code.clone());
                    }
                }
            }
            segment.migrated_root_vars = code_items;
        }
    }

    // Step 7: strip migrated vars from local_idents and scoped_idents.
    for segment in &mut xfrm.segments {
        segment.local_idents.retain(|id| !all_migrated.contains(id));
        segment.scoped_idents.retain(|id| !all_migrated.contains(id));
    }

    // Step 8a: remove migrated var declarations from root AST.
    {
        let mut to_remove: Vec<usize> = Vec::new();
        for (i, stmt) in program.body.iter().enumerate() {
            let should_remove = match stmt {
                oxc::ast::ast::Statement::VariableDeclaration(decl) => {
                    decl.declarations.iter().all(|d| {
                        if let oxc::ast::ast::BindingPattern::BindingIdentifier(id) = &d.id {
                            all_migrated.contains(id.name.as_str())
                        } else {
                            false
                        }
                    })
                }
                oxc::ast::ast::Statement::FunctionDeclaration(fn_decl) => {
                    fn_decl.id.as_ref().map_or(false, |id| {
                        all_migrated.contains(id.name.as_str())
                    })
                }
                oxc::ast::ast::Statement::ClassDeclaration(cls) => {
                    cls.id.as_ref().map_or(false, |id| {
                        all_migrated.contains(id.name.as_str())
                    })
                }
                _ => false,
            };
            if should_remove {
                to_remove.push(i);
            }
        }
        for idx in to_remove.into_iter().rev() {
            program.body.remove(idx);
        }
    }

    // Step 8b: remove _auto_ export specifiers for migrated vars.
    {
        let mut to_remove: Vec<usize> = Vec::new();
        for (i, stmt) in program.body.iter().enumerate() {
            if let oxc::ast::ast::Statement::ExportNamedDeclaration(export_decl) = stmt {
                if export_decl.declaration.is_none() && !export_decl.specifiers.is_empty() {
                    let all_migrated_spec = export_decl.specifiers.iter().all(|spec| {
                        let local = spec.local.name();
                        all_migrated.contains(local.as_str())
                    });
                    if all_migrated_spec {
                        to_remove.push(i);
                    }
                }
            }
        }
        for idx in to_remove.into_iter().rev() {
            program.body.remove(idx);
        }
    }

    // Step 9: remove_unused_qrl_declarations -- iterative fixpoint.
    loop {
        let referenced = {
            use oxc::ast_visit::Visit;
            let mut collector = dependency_analysis::IdentRefCollector::default();
            for stmt in program.body.iter() {
                collector.visit_statement(stmt);
            }
            collector.names
        };

        let before_len = program.body.len();
        let mut to_remove: Vec<usize> = Vec::new();

        for (i, stmt) in program.body.iter().enumerate() {
            if let oxc::ast::ast::Statement::VariableDeclaration(decl) = stmt {
                if let Some(declarator) = decl.declarations.first() {
                    if let oxc::ast::ast::BindingPattern::BindingIdentifier(id) = &declarator.id {
                        let name = id.name.as_str();
                        if name.starts_with("_qrl_") || name.starts_with("i_") {
                            let ref_count = referenced.iter().filter(|r| r.as_str() == name).count();
                            if ref_count == 0 {
                                to_remove.push(i);
                            }
                        }
                    }
                }
            }
        }

        if to_remove.is_empty() {
            break;
        }
        for idx in to_remove.into_iter().rev() {
            program.body.remove(idx);
        }
        if program.body.len() == before_len {
            break;
        }
    }
}

// ---------------------------------------------------------------------------
// transform_modules -- public entry point
// ---------------------------------------------------------------------------

/// Transform one or more input modules.
///
/// Iterates all inputs, calls `transform_code` per file, merges results via
/// `TransformOutput::append`, and sorts output modules by their `order` field.
///
/// `is_server` defaults to `true` when `None` per SPEC line 259.
pub fn transform_modules(config: TransformModulesOptions) -> TransformOutput {
    let code_config = TransformCodeOptions {
        src_dir: config.src_dir.clone(),
        root_dir: config.root_dir.clone(),
        source_maps: config.source_maps,
        minify: config.minify.clone(),
        transpile_ts: config.transpile_ts,
        transpile_jsx: config.transpile_jsx,
        preserve_filenames: config.preserve_filenames,
        entry_strategy: config.entry_strategy.clone(),
        explicit_extensions: config.explicit_extensions,
        mode: config.mode.clone(),
        scope: config.scope.clone(),
        core_module: config
            .core_module
            .clone()
            .unwrap_or_else(|| "@qwik.dev/core".to_string()),
        strip_exports: config.strip_exports.clone().unwrap_or_default(),
        strip_ctx_name: config.strip_ctx_name.clone().unwrap_or_default(),
        strip_event_handlers: config.strip_event_handlers,
        reg_ctx_name: config.reg_ctx_name.clone().unwrap_or_default(),
        is_server: config.is_server.unwrap_or(true),
    };

    let mut result = TransformOutput::default();

    for input in &config.input {
        let mut file_output = transform_code(
            &code_config,
            &input.code,
            &input.path,
            input.dev_path.as_deref(),
        );
        result.append(&mut file_output);
    }

    // OUT-02: sort by order for deterministic output.
    result.modules.sort_unstable_by_key(|m| m.order);

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    fn make_input(code: &str, path: &str) -> TransformModuleInput {
        TransformModuleInput {
            code: code.to_string(),
            path: path.to_string(),
            dev_path: None,
        }
    }

    fn opts_with_inputs(src_dir: &str, inputs: Vec<TransformModuleInput>) -> TransformModulesOptions {
        TransformModulesOptions {
            src_dir: src_dir.to_string(),
            input: inputs,
            source_maps: false,
            ..TransformModulesOptions::default()
        }
    }

    #[test]
    fn test_transform_tsx_single_module_flags() {
        let opts = opts_with_inputs(
            "/project",
            vec![make_input("export const x = 1;", "component.tsx")],
        );
        let result = transform_modules(opts);
        assert_eq!(result.modules.len(), 1);
        assert!(result.is_type_script);
        assert!(result.is_jsx);
    }

    #[test]
    fn test_transform_js_single_module_flags() {
        let opts = opts_with_inputs(
            "/project",
            vec![make_input("export const x = 1;", "utils.js")],
        );
        let result = transform_modules(opts);
        assert_eq!(result.modules.len(), 1);
        assert!(!result.is_type_script);
        assert!(!result.is_jsx);
    }

    #[test]
    fn test_transform_empty_input() {
        let opts = opts_with_inputs("/project", vec![]);
        let result = transform_modules(opts);
        assert_eq!(result.modules.len(), 0);
    }

    #[test]
    fn test_transform_simple_dollar() {
        let code = r#"import { component$, $ } from "@qwik.dev/core";
export const App = component$(() => {
    return $(() => "hello");
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            mode: EmitMode::Test,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        // Should have root module + segment modules
        assert!(result.modules.len() >= 1, "Should have at least root module, got {}", result.modules.len());
        // Root module should contain qrl or componentQrl
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();
        assert!(
            root.code.contains("Qrl") || root.code.contains("qrl"),
            "Root module should contain QRL wrappers, got: {}",
            root.code
        );
    }
}
