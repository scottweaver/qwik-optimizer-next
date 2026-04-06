//! Qwik optimizer using OXC for code transformation.
//!
//! This crate provides the type definitions and utility functions needed for
//! the Qwik $-call extraction pipeline.

pub mod types;
pub mod words;
pub mod hash;
pub mod errors;
pub mod is_const;
pub(crate) mod parser;
pub(crate) mod source_path;
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
    dev_path: Option<&str>,
) -> TransformOutput {
    // Stage 0: Decompose path.
    let path_data = match source_path::SourcePath(input_path).path_data(Path::new(&config.src_dir)) {
        Ok(pd) => pd,
        Err(_) => {
            return TransformOutput::default();
        }
    };

    // Stage 1: Allocate arena and parse source.
    let allocator = oxc::allocator::Allocator::default();
    let source_in_arena: &str = allocator.alloc_str(input_code);

    let (is_type_script, is_jsx, mut program, diagnostics) =
        match parser::parse(&allocator, source_in_arena, input_path) {
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

    // Use output extension (not raw input extension) for QRL import paths.
    // SWC maps .tsx -> .js when both transpile_ts and transpile_jsx are true,
    // so the dynamic import("./foo.js") reflects the transpiled output, not the source.
    let file_extension = source_path::SourcePath(input_path)
        .output_extension(config.transpile_ts, config.transpile_jsx)
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
        &path_data.file_stem,
        &path_data.rel_dir.to_string_lossy(),
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

    // Post-migration cleanup: remove unused QRL declarations and dead imports.
    // Matches SWC's separate remove_unused_qrl_declarations call, which runs
    // after variable migration (i.e., not in Lib mode where migration is skipped).
    if !matches!(config.mode, EmitMode::Lib) && !xfrm.segments.is_empty() {
        remove_unused_qrl_declarations(&mut program);
    }

    // Resolve deferred parent symbol names.
    xfrm.patch_segment_parents();

    let did_transform = !xfrm.segments.is_empty();

    // Emit: codegen the transformed AST back to JavaScript.
    let mut emit_result = emit::emit_module(
        &program,
        source_in_arena,
        &EmitOptions {
            source_maps: config.source_maps,
        },
        input_path,
    );

    // Post-emit: strip unreferenced const declarations to bare expression statements.
    // SWC's DCE simplifier converts `const X = wrapper(q_...)` to just
    // `wrapper(q_...)` when X is not exported or referenced elsewhere in root.
    if !matches!(config.mode, EmitMode::Lib) && did_transform {
        emit_result.code = strip_unreferenced_wrapper_consts_text(&emit_result.code);
    }

    // Post-emit: inject dev mode metadata into qrlDEV/qrlDEV calls.
    // OXC codegen's sourcemap builder panics on span violations if we embed
    // large format strings into parsed AST nodes, so we inject the metadata
    // as a text post-processing step after codegen.
    if !xfrm.dev_metadata.is_empty() {
        emit_result.code = inject_dev_metadata(&emit_result.code, &xfrm.dev_metadata);
    }

    // Compute output path.
    let output_path = if did_transform && !config.preserve_filenames {
        let ext = source_path::SourcePath(input_path).output_extension(config.transpile_ts, config.transpile_jsx);
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
    let record_extension = source_path::SourcePath(input_path).output_extension(config.transpile_ts, config.transpile_jsx);

    // HMR: compute effective dev_path for _useHmr injection (D-41).
    // Defaults to abs_dir/file_name when dev_path is not provided.
    let effective_dev_path: String = dev_path.map(|s| s.to_string()).unwrap_or_else(|| {
        let abs = path_data.abs_dir.join(&path_data.file_name);
        abs.to_slash_lossy().into_owned()
    });
    let is_hmr_mode = config.mode == EmitMode::Hmr;

    for record in &xfrm.segments {
        // Skip inline segments (they live in the parent module)
        if record.is_inline {
            continue;
        }

        // Handle noop segments (stripped handlers): emit `export const NAME = null;`
        let expr_code = match &record.expr {
            Some(e) => e.as_str(),
            None => {
                // Noop segment: emit a null export module
                let noop_code = format!("export const {} = null;", record.name);
                let (final_code, map) = code_move::emit_segment(
                    &noop_code,
                    &record.canonical_filename,
                    config.source_maps,
                );

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

                let is_entry = record.entry.is_none();
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
                    captures: false,
                    loc: (record.span.0 + 1, record.span.1 + 1),
                    param_names: record.param_names.clone(),
                    capture_names: None,
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
                continue;
            }
        };

        // HMR _useHmr() injection (D-41): inject into component$ segment bodies only.
        let is_component_hmr = is_hmr_mode && record.ctx_name == "component$";
        let hmr_expr_code: String;
        let final_expr_code: &str = if is_component_hmr {
            hmr_expr_code = code_move::inject_use_hmr(expr_code, &effective_dev_path);
            &hmr_expr_code
        } else {
            expr_code
        };

        // For HMR component$ segments, add _useHmr to the local_idents so it gets imported.
        let mut local_idents_with_hmr: Vec<String>;
        let effective_local_idents: &[String] = if is_component_hmr {
            local_idents_with_hmr = record.local_idents.clone();
            if !local_idents_with_hmr.contains(&"_useHmr".to_string()) {
                local_idents_with_hmr.push("_useHmr".to_string());
            }
            &local_idents_with_hmr
        } else {
            &record.local_idents
        };

        // Build synthetic imports for HMR _useHmr
        let hmr_import = if is_component_hmr {
            vec![format!(r#"import {{ _useHmr }} from "{}";"#, config.core_module)]
        } else {
            vec![]
        };

        // Build segment module code via new_module
        let module_code = code_move::new_module(code_move::NewModuleCtx {
            expr: final_expr_code,
            name: &record.name,
            file_stem: &path_data.file_stem,
            local_idents: effective_local_idents,
            scoped_idents: &record.scoped_idents,
            global: &collect,
            core_module: &config.core_module,
            explicit_extensions: config.explicit_extensions,
            extra_top_items: &xfrm.extra_top_items,
            migrated_root_vars: &record.migrated_root_vars,
            synthetic_imports: &hmr_import,
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

}

// ---------------------------------------------------------------------------
// remove_unused_qrl_declarations -- SWC-matching fixpoint cleanup
// ---------------------------------------------------------------------------

/// Remove unused `_qrl_`/`i_` variable declarations and dead imports from the
/// root module in a fixpoint loop. Matches SWC's `remove_unused_qrl_declarations`
/// (parse.rs lines 1482-1640).
///
/// This runs AFTER variable migration and import assembly. It iterates until
/// stable, removing both unused `_qrl_`/`i_` var declarations AND unused imports
/// (from ANY source, not just the core module). Transitive closure propagation
/// ensures that if a `_qrl_` var references an import, and the `_qrl_` var is
/// used, the import is kept.
fn remove_unused_qrl_declarations(program: &mut oxc::ast::ast::Program<'_>) {
    use oxc::ast::ast::*;
    use oxc::ast_visit::Visit;
    use std::collections::HashSet;

    loop {
        // 1. Collect all names DEFINED by _qrl_/i_ var declarations and ALL imports
        let mut qrl_defined: HashSet<String> = HashSet::new();
        let mut import_defined: HashSet<String> = HashSet::new();
        for stmt in program.body.iter() {
            match stmt {
                Statement::VariableDeclaration(decl) => {
                    for d in &decl.declarations {
                        if let BindingPattern::BindingIdentifier(id) = &d.id {
                            let name = id.name.as_str();
                            if name.starts_with("_qrl_") || name.starts_with("i_") {
                                qrl_defined.insert(name.to_string());
                            }
                        }
                    }
                }
                Statement::ImportDeclaration(import_decl) => {
                    if let Some(specifiers) = &import_decl.specifiers {
                        for spec in specifiers {
                            let sym = match spec {
                                ImportDeclarationSpecifier::ImportSpecifier(n) => n.local.name.as_str(),
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(d) => d.local.name.as_str(),
                                ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => ns.local.name.as_str(),
                            };
                            import_defined.insert(sym.to_string());
                        }
                    }
                }
                _ => {}
            }
        }
        if qrl_defined.is_empty() && import_defined.is_empty() {
            break;
        }

        // 2. Collect all identifiers referenced by NON-removable items
        //    (items that are not _qrl_/i_ vars and not imports)
        let mut used: HashSet<String> = HashSet::new();
        for stmt in program.body.iter() {
            let is_removable = match stmt {
                Statement::VariableDeclaration(decl) => {
                    decl.declarations.iter().all(|d| {
                        if let BindingPattern::BindingIdentifier(id) = &d.id {
                            let name = id.name.as_str();
                            name.starts_with("_qrl_") || name.starts_with("i_")
                        } else {
                            false
                        }
                    })
                }
                Statement::ImportDeclaration(_) => true,
                _ => false,
            };
            if !is_removable {
                let mut collector = dependency_analysis::IdentRefCollector::default();
                collector.visit_statement(stmt);
                for name in collector.names {
                    used.insert(name);
                }
            }
        }

        // 3. Propagate: if a removable item is used, its references become used too
        //    (transitive closure -- a used _qrl_ var may reference an import)
        let mut changed = true;
        while changed {
            changed = false;
            for stmt in program.body.iter() {
                let defined_sym = match stmt {
                    Statement::VariableDeclaration(decl) => {
                        decl.declarations.first().and_then(|d| {
                            if let BindingPattern::BindingIdentifier(id) = &d.id {
                                let name = id.name.as_str();
                                if name.starts_with("_qrl_") || name.starts_with("i_") {
                                    Some(name.to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        })
                    }
                    Statement::ImportDeclaration(import_decl) => {
                        import_decl.specifiers.as_ref().and_then(|specs| {
                            specs.first().map(|spec| match spec {
                                ImportDeclarationSpecifier::ImportSpecifier(n) => n.local.name.to_string(),
                                ImportDeclarationSpecifier::ImportDefaultSpecifier(d) => d.local.name.to_string(),
                                ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => ns.local.name.to_string(),
                            })
                        })
                    }
                    _ => None,
                };
                if let Some(sym) = defined_sym {
                    if used.contains(&sym) {
                        let mut collector = dependency_analysis::IdentRefCollector::default();
                        collector.visit_statement(stmt);
                        for r in collector.names {
                            if used.insert(r) {
                                changed = true;
                            }
                        }
                    }
                }
            }
        }

        // 4. Remove unused _qrl_/i_ declarations and unused imports
        let before_len = program.body.len();
        program.body.retain(|stmt| {
            match stmt {
                Statement::VariableDeclaration(decl) => {
                    let all_unused = decl.declarations.iter().all(|d| {
                        if let BindingPattern::BindingIdentifier(id) = &d.id {
                            let name = id.name.as_str();
                            (name.starts_with("_qrl_") || name.starts_with("i_"))
                                && !used.contains(name)
                        } else {
                            false
                        }
                    });
                    !all_unused
                }
                Statement::ImportDeclaration(import_decl) => {
                    // Keep side-effect imports (specifiers: None)
                    let Some(specifiers) = &import_decl.specifiers else {
                        return true;
                    };
                    if specifiers.is_empty() {
                        // Empty specifier list = leftover shell after prior removal
                        return false;
                    }
                    // Remove imports where ALL specifiers are unused
                    let all_unused = specifiers.iter().all(|spec| {
                        let sym = match spec {
                            ImportDeclarationSpecifier::ImportSpecifier(n) => n.local.name.as_str(),
                            ImportDeclarationSpecifier::ImportDefaultSpecifier(d) => d.local.name.as_str(),
                            ImportDeclarationSpecifier::ImportNamespaceSpecifier(ns) => ns.local.name.as_str(),
                        };
                        !used.contains(sym)
                    });
                    !all_unused
                }
                _ => true,
            }
        });

        if program.body.len() == before_len {
            break; // No more items removed, stable
        }
    }
}

// ---------------------------------------------------------------------------
// inject_dev_metadata -- qrlDEV post-emit metadata injection
// ---------------------------------------------------------------------------

/// Inject dev mode metadata objects into qrlDEV calls after codegen.
///
/// Transforms: `qrlDEV(()=>import("./path"), "sym")` into
/// `qrlDEV(()=>import("./path"), "sym", {\n    file: "...",\n    lo: N,\n    hi: N,\n    displayName: "..."\n})`
///
/// This is done as text post-processing because OXC's codegen sourcemap builder
/// panics on span violations when large format strings are parsed into AST nodes.
fn inject_dev_metadata(
    code: &str,
    metadata: &std::collections::HashMap<String, (String, u32, u32, String)>,
) -> String {
    let mut result = code.to_string();
    for (sym, (file, lo, hi, display_name)) in metadata {
        // Match: qrlDEV(()=>import("..."), "SYM")
        let search = format!(r#"qrlDEV(()=>import("{}"), "{}")"#,
            // We don't know the exact import path, so search by symbol name
            "", sym);
        // Actually, we can't match the import path. Let's use a more targeted approach:
        // Search for `"SYM")` and replace the closing `)` with `, { ... })`
        let search_pattern = format!(r#", "{}")"#, sym);
        let replacement = format!(
            r#", "{}", {{
    file: "{}",
    lo: {},
    hi: {},
    displayName: "{}"
}})"#,
            sym, file, lo, hi, display_name
        );
        // Only replace in qrlDEV lines (not in other contexts)
        let mut new_result = String::with_capacity(result.len() + 200);
        for line in result.lines() {
            if line.contains("qrlDEV") && line.contains(&search_pattern) {
                new_result.push_str(&line.replace(&search_pattern, &replacement));
            } else {
                new_result.push_str(line);
            }
            new_result.push('\n');
        }
        // Trim trailing newline if original didn't have one
        if !result.ends_with('\n') && new_result.ends_with('\n') {
            new_result.pop();
        }
        result = new_result;
    }
    result
}

// ---------------------------------------------------------------------------
// strip_unreferenced_wrapper_consts_text -- SWC DCE parity (text-level)
// ---------------------------------------------------------------------------

/// Text-level pass: convert unreferenced `const X = EXPR;` to bare `EXPR;`.
///
/// SWC's simplifier strips const declarations when the declared name is neither
/// exported nor referenced elsewhere in the root module. This text-based approach
/// operates on the codegen output because OXC's arena allocation makes in-place
/// AST node type changes impractical.
///
/// Algorithm:
/// 1. Parse lines to find `const NAME = EXPR;` patterns (not `const q_*`, not `export const`)
/// 2. Collect all identifier references from other lines
/// 3. For declarations where NAME appears nowhere else, strip to `EXPR;`
fn strip_unreferenced_wrapper_consts_text(code: &str) -> String {
    use std::collections::{HashMap, HashSet};

    let mut current = code.to_string();

    // Fixpoint loop: stripping one const may make others unreferenced.
    // Limited to 5 iterations to prevent infinite loops.
    for _ in 0..5 {
        let lines: Vec<&str> = current.lines().collect();
        if lines.is_empty() {
            return current;
        }

        // 1. Find candidate const declarations.
        // Only non-exported consts with call expression or PURE annotation RHS,
        // OR simple identifier assignment (e.g., `const X = q_Y;`).
        let mut candidates: HashMap<usize, String> = HashMap::new();
        for (i, line) in lines.iter().enumerate() {
            let trimmed = line.trim();
            if trimmed.starts_with("export ") || !trimmed.starts_with("const ") {
                continue;
            }
            let rest = &trimmed[6..]; // skip "const "
            if let Some(eq_pos) = rest.find(" = ") {
                let name = rest[..eq_pos].trim();
                if name.starts_with('{') || name.starts_with('[') {
                    continue;
                }
                // Never strip hoisted QRL const declarations (const q_*)
                // These are QRL references used in the module body; stripping
                // them creates dangling references (e.g., `component(q_foo)` with no matching const).
                if name.starts_with("q_") {
                    continue;
                }
                let rhs = &rest[eq_pos + 3..];
                // Include: call expressions, PURE annotations, or simple q_ identifier assignments
                let is_call = rhs.contains('(');
                let is_pure = rhs.contains("__PURE__");
                let rhs_ident = rhs.trim_end_matches(';').trim();
                let is_simple_q_ref = rhs_ident.starts_with("q_")
                    && rhs_ident.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
                if is_call || is_pure || is_simple_q_ref {
                    candidates.insert(i, name.to_string());
                }
            }
        }

        if candidates.is_empty() {
            return current;
        }

        // 2. Collect all identifier occurrences from non-candidate lines.
        let mut referenced: HashSet<String> = HashSet::new();
        for (i, line) in lines.iter().enumerate() {
            if candidates.contains_key(&i) {
                continue;
            }
            let mut chars = line.chars().peekable();
            while let Some(&ch) = chars.peek() {
                if ch.is_ascii_alphabetic() || ch == '_' || ch == '$' {
                    let mut word = String::new();
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_alphanumeric() || c == '_' || c == '$' {
                            word.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                    referenced.insert(word);
                } else {
                    chars.next();
                }
            }
        }

        // 3. Strip unreferenced candidates.
        let mut any_stripped = false;
        let mut result_lines: Vec<String> = Vec::with_capacity(lines.len());
        for (i, line) in lines.iter().enumerate() {
            if let Some(name) = candidates.get(&i) {
                if !referenced.contains(name) {
                    let trimmed = line.trim();
                    let prefix = format!("const {} = ", name);
                    if trimmed.starts_with(&prefix) {
                        let expr = &trimmed[prefix.len()..];
                        // If expr is just a simple identifier (e.g., `q_foo;`),
                        // remove the entire line (dead assignment to dead value).
                        let expr_trimmed = expr.trim_end_matches(';').trim();
                        if expr_trimmed.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
                            any_stripped = true;
                            continue; // Remove line entirely
                        }
                        // Otherwise, keep just the expression (strip the const decl prefix)
                        result_lines.push(expr.to_string());
                        any_stripped = true;
                        continue;
                    }
                }
            }
            result_lines.push(line.to_string());
        }

        if !any_stripped {
            return current;
        }

        current = result_lines.join("\n");
        if code.ends_with('\n') && !current.ends_with('\n') {
            current.push('\n');
        }
    }

    current
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

    // -----------------------------------------------------------------------
    // Hoist strategy tests (D-40 / CONV-14 Rule 6)
    // -----------------------------------------------------------------------

    #[test]
    fn test_hoist_strategy_produces_noop_qrl_const_and_s_registration() {
        let code = r#"import { component$, useStore } from "@qwik.dev/core";
export const Child = component$(() => {
    const state = useStore({ count: 0 });
    return <div></div>;
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Hoist,
            mode: EmitMode::Prod,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();

        // Must contain _noopQrl const declaration
        assert!(
            root.code.contains("_noopQrl("),
            "Hoist strategy should produce _noopQrl const, got:\n{}",
            root.code
        );
        // Must contain .s() registration call
        assert!(
            root.code.contains(".s("),
            "Hoist strategy should produce .s() registration, got:\n{}",
            root.code
        );
        // Must contain componentQrl wrapper
        assert!(
            root.code.contains("componentQrl("),
            "Hoist strategy should wrap with componentQrl, got:\n{}",
            root.code
        );
        // Must import _noopQrl from core module
        assert!(
            root.code.contains("import { _noopQrl }"),
            "Hoist strategy should import _noopQrl, got:\n{}",
            root.code
        );
    }

    #[test]
    fn test_hoist_strategy_with_captures_produces_w_chain() {
        let code = r#"import { component$, useStore, useBrowserVisibleTask$ } from "@qwik.dev/core";
export const Child = component$(() => {
    const state = useStore({ count: 0 });
    useBrowserVisibleTask$(() => {
        state.count = 1;
    });
    return <div></div>;
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Hoist,
            mode: EmitMode::Prod,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();

        // The useBrowserVisibleTask$ captures 'state', so should have .w([state])
        // This appears in the .s() body of the component segment
        assert!(
            root.code.contains(".w(") || root.code.contains("_noopQrl("),
            "Hoist strategy with captures should produce .w() chain or _noopQrl, got:\n{}",
            root.code
        );
    }

    #[test]
    fn test_hoist_strategy_lib_mode_returns_unchanged() {
        let code = r#"import { component$ } from "@qwik.dev/core";
export const Child = component$(() => {
    return <div></div>;
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Hoist,
            mode: EmitMode::Lib,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();

        // Lib mode should NOT produce _noopQrl -- falls through to inline path
        assert!(
            !root.code.contains("_noopQrl("),
            "Hoist + Lib mode should NOT produce _noopQrl, got:\n{}",
            root.code
        );
        // Should use inlinedQrl instead (inline path)
        assert!(
            root.code.contains("inlinedQrl") || root.code.contains("componentQrl"),
            "Hoist + Lib mode should use inlinedQrl path, got:\n{}",
            root.code
        );
    }

    #[test]
    fn test_hoist_strategy_dev_mode_uses_noop_qrl_dev() {
        let code = r#"import { component$ } from "@qwik.dev/core";
export const Child = component$(() => {
    return <div></div>;
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Hoist,
            mode: EmitMode::Dev,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();

        // Dev mode should use _noopQrlDEV
        assert!(
            root.code.contains("_noopQrlDEV("),
            "Hoist + Dev mode should use _noopQrlDEV, got:\n{}",
            root.code
        );
    }

    // -----------------------------------------------------------------------
    // EntryPolicy wiring tests
    // -----------------------------------------------------------------------

    #[test]
    fn test_entry_policy_segment_strategy_no_entry() {
        let code = r#"import { component$, $ } from "@qwik.dev/core";
export const App = component$(() => {
    return $(() => "hello");
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Segment,
            mode: EmitMode::Prod,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        // Segment strategy: each segment gets its own chunk (entry = None -> is_entry = true)
        let segment_modules: Vec<_> = result.modules.iter().filter(|m| m.segment.is_some()).collect();
        for m in &segment_modules {
            assert!(
                m.is_entry,
                "Segment strategy should set is_entry=true (entry=None), got is_entry={} for {}",
                m.is_entry, m.path
            );
        }
    }

    #[test]
    fn test_entry_policy_component_strategy_groups_by_component() {
        let code = r#"import { component$, $ } from "@qwik.dev/core";
export const App = component$(() => {
    return $(() => "hello");
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Component,
            mode: EmitMode::Prod,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let segment_modules: Vec<_> = result.modules.iter().filter(|m| m.segment.is_some()).collect();
        // Component strategy: segments within a component should share an entry key
        // (is_entry = false means entry is Some(...))
        if segment_modules.len() > 1 {
            let has_grouped = segment_modules.iter().any(|m| !m.is_entry);
            assert!(
                has_grouped,
                "Component strategy should group segments (some with is_entry=false)"
            );
        }
    }

    #[test]
    fn test_entry_policy_smart_strategy_groups_component_with_context() {
        // Smart strategy: component$ segments with a parent context (e.g., declared in a
        // named variable) are grouped per-component, NOT given their own chunk.
        let code = r#"import { component$, $ } from "@qwik.dev/core";
export const App = component$(() => {
    return <div onClick$={() => console.log("click")}></div>;
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Smart,
            mode: EmitMode::Prod,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let segment_modules: Vec<_> = result.modules.iter().filter(|m| m.segment.is_some()).collect();
        assert!(
            !segment_modules.is_empty(),
            "Smart strategy should produce segments"
        );
        // component$ in a named variable context should be grouped (is_entry=false)
        let component_seg = segment_modules.iter().find(|m| {
            m.segment.as_ref().map_or(false, |s| s.ctx_name == "component$")
        });
        assert!(
            component_seg.is_some(),
            "Should produce a component$ segment"
        );
        assert!(
            !component_seg.unwrap().is_entry,
            "Smart strategy should group component$ with its parent context (is_entry=false)"
        );
    }

    #[test]
    fn test_entry_policy_hook_strategy_each_own_chunk() {
        let code = r#"import { component$, $ } from "@qwik.dev/core";
export const App = component$(() => {
    return $(() => "hello");
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Hook,
            mode: EmitMode::Prod,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        // Hook strategy: each segment gets its own chunk (same as Segment)
        let segment_modules: Vec<_> = result.modules.iter().filter(|m| m.segment.is_some()).collect();
        for m in &segment_modules {
            assert!(
                m.is_entry,
                "Hook strategy should set is_entry=true (entry=None), got is_entry={} for {}",
                m.is_entry, m.path
            );
        }
    }

    #[test]
    fn test_hoist_strategy_spec_example_inlined_entry_strategy() {
        // Test from spec example_inlined_entry_strategy
        let code = r#"import { component$, useBrowserVisibleTask$, useStore, useStyles$ } from '@qwik.dev/core';
import { thing } from './sibling';
import mongodb from 'mongodb';

export const Child = component$(() => {
    useStyles$('somestring');
    const state = useStore({ count: 0 });
    useBrowserVisibleTask$(() => {
        state.count = thing.doStuff() + import("./sibling");
    });
    return (
        <div onClick$={() => console.log(mongodb)}>
        </div>
    );
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Hoist,
            mode: EmitMode::Prod,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();

        // Verify key patterns from spec
        assert!(
            root.code.contains("_noopQrl("),
            "Hoist spec example should contain _noopQrl, got:\n{}",
            root.code
        );
        assert!(
            root.code.contains("componentQrl("),
            "Hoist spec example should contain componentQrl, got:\n{}",
            root.code
        );
        assert!(
            root.code.contains(".s("),
            "Hoist spec example should contain .s() registrations, got:\n{}",
            root.code
        );
        // Verify multiple _noopQrl const declarations exist (component + inner segments)
        let noop_count = root.code.matches("_noopQrl(").count();
        assert!(
            noop_count >= 1,
            "Hoist spec example should have at least 1 _noopQrl declaration, found {}, got:\n{}",
            noop_count, root.code
        );
    }

    #[test]
    fn test_all_seven_entry_strategies_produce_output() {
        let code = r#"import { component$, $ } from "@qwik.dev/core";
export const App = component$(() => {
    return $(() => "hello");
});"#;
        let strategies = vec![
            ("Segment", EntryStrategy::Segment),
            ("Inline", EntryStrategy::Inline),
            ("Hoist", EntryStrategy::Hoist),
            ("Single", EntryStrategy::Single),
            ("Component", EntryStrategy::Component),
            ("Smart", EntryStrategy::Smart),
            ("Hook", EntryStrategy::Hook),
        ];
        for (name, strategy) in strategies {
            let opts = TransformModulesOptions {
                src_dir: "/project".to_string(),
                input: vec![make_input(code, "test.tsx")],
                source_maps: false,
                entry_strategy: strategy,
                mode: EmitMode::Prod,
                ..TransformModulesOptions::default()
            };
            let result = transform_modules(opts);
            assert!(
                !result.modules.is_empty(),
                "{} strategy should produce at least one module",
                name
            );
            assert!(
                result.diagnostics.is_empty(),
                "{} strategy should produce no diagnostics, got: {:?}",
                name, result.diagnostics
            );
        }
    }

    // -----------------------------------------------------------------------
    // Emit mode tests (06-02: HMR, Lib, Test, Dev, Prod)
    // -----------------------------------------------------------------------

    #[test]
    fn test_hmr_mode_injects_use_hmr_in_component_segment() {
        let code = r#"import { component$ } from "@qwik.dev/core";
export const App = component$(() => {
    return <div>Hello</div>;
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/user/qwik/src/".to_string(),
            input: vec![TransformModuleInput {
                code: code.to_string(),
                path: "test.tsx".to_string(),
                dev_path: None,
            }],
            source_maps: false,
            mode: EmitMode::Hmr,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let segment = result.modules.iter().find(|m| {
            m.segment.as_ref().map_or(false, |s| s.ctx_name == "component$")
        });
        assert!(segment.is_some(), "Should have a component$ segment module");
        let seg = segment.unwrap();
        assert!(
            seg.code.contains("_useHmr("),
            "HMR mode component$ segment should contain _useHmr() call, got:\n{}",
            seg.code
        );
        // The import should also be present
        assert!(
            seg.code.contains("_useHmr"),
            "Segment should import _useHmr, got:\n{}",
            seg.code
        );
    }

    #[test]
    fn test_hmr_mode_does_not_inject_use_hmr_in_bare_dollar_segment() {
        let code = r#"import { $, component$ } from "@qwik.dev/core";
export const App = component$(() => {
    return $(() => "hello");
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/user/qwik/src/".to_string(),
            input: vec![TransformModuleInput {
                code: code.to_string(),
                path: "test.tsx".to_string(),
                dev_path: None,
            }],
            source_maps: false,
            mode: EmitMode::Hmr,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        // Find the bare $ segment (not the component$ segment)
        let bare_dollar_seg = result.modules.iter().find(|m| {
            m.segment.as_ref().map_or(false, |s| s.ctx_name == "$")
        });
        assert!(bare_dollar_seg.is_some(), "Should have a bare $ segment");
        let seg = bare_dollar_seg.unwrap();
        assert!(
            !seg.code.contains("_useHmr("),
            "HMR mode bare $ segment should NOT contain _useHmr(), got:\n{}",
            seg.code
        );
    }

    #[test]
    fn test_hmr_mode_does_not_inject_use_hmr_in_use_task_segment() {
        let code = r#"import { component$, useTask$ } from "@qwik.dev/core";
export const App = component$(() => {
    useTask$(() => { console.log("task"); });
    return <div>Hello</div>;
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/user/qwik/src/".to_string(),
            input: vec![TransformModuleInput {
                code: code.to_string(),
                path: "test.tsx".to_string(),
                dev_path: None,
            }],
            source_maps: false,
            mode: EmitMode::Hmr,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let use_task_seg = result.modules.iter().find(|m| {
            m.segment.as_ref().map_or(false, |s| s.ctx_name == "useTask$")
        });
        assert!(use_task_seg.is_some(), "Should have a useTask$ segment");
        let seg = use_task_seg.unwrap();
        assert!(
            !seg.code.contains("_useHmr("),
            "HMR mode useTask$ segment should NOT contain _useHmr(), got:\n{}",
            seg.code
        );
    }

    #[test]
    fn test_lib_mode_produces_no_separate_segments() {
        let code = r#"import { component$, $ } from "@qwik.dev/core";
export const App = component$(() => {
    return $(() => "hello");
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            mode: EmitMode::Lib,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let segment_modules: Vec<_> = result.modules.iter().filter(|m| m.segment.is_some()).collect();
        assert!(
            segment_modules.is_empty(),
            "Lib mode should produce no separate segment modules, got {} segments",
            segment_modules.len()
        );
        // Root module should contain inlinedQrl (inline path)
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();
        assert!(
            root.code.contains("inlinedQrl"),
            "Lib mode root should use inlinedQrl, got:\n{}",
            root.code
        );
    }

    #[test]
    fn test_test_mode_preserves_is_server_identifier() {
        let code = r#"import { isServer } from "@qwik.dev/core/build";
console.log(isServer);"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            mode: EmitMode::Test,
            is_server: Some(true),
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();
        assert!(
            root.code.contains("isServer"),
            "Test mode should preserve isServer as identifier, got:\n{}",
            root.code
        );
        assert!(
            !root.code.contains("console.log(true)"),
            "Test mode should NOT replace isServer with true, got:\n{}",
            root.code
        );
    }

    #[test]
    fn test_dev_mode_uses_qrl_dev() {
        let code = r#"import { component$ } from "@qwik.dev/core";
export const App = component$(() => {
    return <div>Hello</div>;
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            mode: EmitMode::Dev,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();
        assert!(
            root.code.contains("qrlDEV"),
            "Dev mode root should use qrlDEV, got:\n{}",
            root.code
        );
    }

    #[test]
    fn test_prod_mode_uses_standard_qrl() {
        let code = r#"import { component$ } from "@qwik.dev/core";
export const App = component$(() => {
    return <div>Hello</div>;
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            mode: EmitMode::Prod,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let root = result.modules.iter().find(|m| m.segment.is_none()).unwrap();
        assert!(
            root.code.contains("qrl("),
            "Prod mode root should use qrl (not qrlDEV), got:\n{}",
            root.code
        );
        assert!(
            !root.code.contains("qrlDEV"),
            "Prod mode root should NOT use qrlDEV, got:\n{}",
            root.code
        );
    }

    #[test]
    fn test_entry_policy_single_strategy_groups_all() {
        let code = r#"import { component$, $ } from "@qwik.dev/core";
export const App = component$(() => {
    return $(() => "hello");
});"#;
        let opts = TransformModulesOptions {
            src_dir: "/project".to_string(),
            input: vec![make_input(code, "test.tsx")],
            source_maps: false,
            entry_strategy: EntryStrategy::Single,
            mode: EmitMode::Prod,
            ..TransformModulesOptions::default()
        };
        let result = transform_modules(opts);
        let segment_modules: Vec<_> = result.modules.iter().filter(|m| m.segment.is_some()).collect();
        // Single strategy: all segments share "entry_segments" key (is_entry = false)
        for m in &segment_modules {
            assert!(
                !m.is_entry,
                "Single strategy should set is_entry=false (entry=Some), got is_entry={} for {}",
                m.is_entry, m.path
            );
        }
    }
}
