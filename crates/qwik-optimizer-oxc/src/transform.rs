//! QwikTransform -- core traversal pass.
//!
//! Implements `Traverse<'a, ()>` to walk the AST and:
//! - Detect marker functions ($-suffixed imports and local exports) -- CONV-01
//! - Push `SegmentScope` frames for each detected $ call
//!
//! Later plans add segment extraction, QRL generation, JSX handling, and
//! import rewriting by filling the stubs in `exit_expression` and `exit_program`.

use std::collections::{HashMap, HashSet};

use oxc::ast::ast::*;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::collector::{GlobalCollect, ImportKind};
use crate::types::{CtxKind, EmitMode, EntryStrategy, TransformCodeOptions};
use crate::words;

// ---------------------------------------------------------------------------
// SegmentScope -- per-$ call state carried from enter to exit
// ---------------------------------------------------------------------------

/// State accumulated during `enter_call_expression` that is consumed by the
/// matching `exit_expression` to complete segment extraction.
///
/// Because OXC Traverse visits children *after* `enter_*` returns, we cannot
/// process captures until `exit_*` fires (when all nested `$` calls have
/// already been processed).
#[derive(Debug)]
pub(crate) struct SegmentScope {
    /// The context name (e.g., "component$" for `component$(...)`, "$" for `$(...)`).
    pub ctx_name: String,
    /// Classified context kind (Function vs EventHandler).
    pub ctx_kind: CtxKind,
    /// Byte span start of the call expression.
    pub span_start: u32,
    /// Whether this is a sync$ call (CONV-13, not QRL extraction).
    pub is_sync: bool,
}

// ---------------------------------------------------------------------------
// SegmentRecord -- accumulated extracted segment metadata
// ---------------------------------------------------------------------------

/// Internal record for a single extracted segment. Accumulated in
/// `QwikTransform.segments` during the traversal. Later phases read these
/// to emit segment module files.
#[derive(Debug)]
#[allow(dead_code)]
pub(crate) struct SegmentRecord {
    /// Symbol name (e.g. `test_tsx_component_ABC`).
    pub name: String,
    /// File-prefixed display name (e.g. `test.tsx_component_ABC`).
    pub display_name: String,
    /// The context (marker function) name, e.g. `"component$"`.
    pub ctx_name: String,
    /// The context kind (Function, EventHandler, etc.).
    pub ctx_kind: CtxKind,
    /// Byte span `(start, end)` of the original call expression.
    pub span: (u32, u32),
    /// Variables captured from enclosing scope.
    pub scoped_idents: Vec<String>,
    /// Whether this segment was sync$ (CONV-13).
    pub is_sync: bool,
}

// ---------------------------------------------------------------------------
// QwikTransform struct
// ---------------------------------------------------------------------------

/// Core Qwik traversal pass implementing `Traverse<'a, ()>`.
///
/// Traversal state is accumulated across the AST walk. All per-segment
/// extraction logic (later plans) builds on top of the scaffolding
/// established here.
pub(crate) struct QwikTransform {
    // ---- Marker / special-case function detection -------------------------
    /// Maps local binding name -> imported specifier for all $-suffixed Named
    /// imports AND locally-exported $-suffixed identifiers.
    pub(crate) marker_functions: HashMap<String, String>,

    /// Local name for the bare `$` import from the core module.
    pub(crate) qsegment_fn: Option<String>,
    /// Local name for `sync$`.
    pub(crate) sync_qrl_fn: Option<String>,

    // ---- Traversal state --------------------------------------------------
    /// Context name stack; each entry is pushed when entering a named call or
    /// variable declarator and popped on exit. Used to build `display_name`.
    pub(crate) stack_ctxt: Vec<String>,

    /// Stack of segment scopes -- one per detected $ call.
    pub(crate) segment_stack: Vec<SegmentScope>,

    /// Accumulated extracted segments.
    pub(crate) segments: Vec<SegmentRecord>,

    /// Collision counter for `display_name` deduplication.
    pub(crate) segment_names: HashMap<String, u32>,

    /// Global segment counter for generating unique names.
    pub(crate) segment_counter: u32,

    // ---- Import tracking (accumulated during traversal, applied in exit_program)
    pub(crate) needs_qrl_import: bool,
    pub(crate) needs_inlined_qrl_import: bool,

    // ---- Config (owned copies) --------------------------------------------
    pub(crate) mode: EmitMode,
    pub(crate) is_server: bool,
    pub(crate) file_name: String,
    pub(crate) rel_path: String,
    pub(crate) extension: String,
    pub(crate) core_module: String,
    pub(crate) strip_ctx_name: Vec<String>,
    pub(crate) strip_event_handlers: bool,
}

// ---------------------------------------------------------------------------
// QwikTransform::new
// ---------------------------------------------------------------------------

impl QwikTransform {
    /// Create a new `QwikTransform` from the given config and collect.
    ///
    /// - Scans `collect.imports` for `Named` entries whose specifier ends with `$`
    ///   and inserts them into `marker_functions`.
    /// - Scans `collect.export_local_ids()` for names ending with `$` and inserts
    ///   them as self-referential entries in `marker_functions`.
    /// - Resolves special-case functions (`$`, `sync$`) via `get_imported_local`.
    pub(crate) fn new(
        config: &TransformCodeOptions,
        collect: &GlobalCollect,
        file_name: &str,
        rel_path: &str,
        extension: &str,
    ) -> Self {
        let mut marker_functions: HashMap<String, String> = HashMap::new();

        // --- Named imports whose specifier ends with `$` ---
        for (local, import) in &collect.imports {
            if import.kind == ImportKind::Named && import.specifier.ends_with('$') {
                marker_functions.insert(local.clone(), import.specifier.clone());
            }
        }

        // --- Locally-exported names ending with `$` ---
        for name in collect.export_local_ids() {
            if name.ends_with('$') {
                marker_functions.insert(name.clone(), name.clone());
            }
        }

        // --- Special-case function resolution ---
        let qsegment_fn = collect
            .get_imported_local("$", &config.core_module)
            .map(|s| s.to_string());
        let sync_qrl_fn = collect
            .get_imported_local("sync$", &config.core_module)
            .map(|s| s.to_string());

        QwikTransform {
            marker_functions,
            qsegment_fn,
            sync_qrl_fn,
            stack_ctxt: Vec::new(),
            segment_stack: Vec::new(),
            segments: Vec::new(),
            segment_names: HashMap::new(),
            segment_counter: 0,
            needs_qrl_import: false,
            needs_inlined_qrl_import: false,
            mode: config.mode.clone(),
            is_server: config.is_server,
            file_name: file_name.to_string(),
            rel_path: rel_path.to_string(),
            extension: extension.to_string(),
            core_module: config.core_module.clone(),
            strip_ctx_name: config.strip_ctx_name.clone(),
            strip_event_handlers: config.strip_event_handlers,
        }
    }

    // -----------------------------------------------------------------------
    // Dollar detection helpers (CONV-01)
    // -----------------------------------------------------------------------

    /// Check if a call expression's callee is a known $ marker function.
    ///
    /// Returns `Some((specifier_name, is_sync))` if the callee resolves to a
    /// known marker, `None` otherwise.
    fn detect_dollar_call(&self, callee: &Expression<'_>) -> Option<(String, bool)> {
        match callee {
            Expression::Identifier(ident) => {
                let local_name = ident.name.as_str();

                // Check sync$ first (CONV-13: sync serialization, not QRL extraction)
                if self.sync_qrl_fn.as_deref() == Some(local_name) {
                    return Some(("sync$".to_string(), true));
                }

                // Check marker_functions (all $-suffixed imports from core module)
                if let Some(specifier) = self.marker_functions.get(local_name) {
                    return Some((specifier.clone(), false));
                }

                // Check bare $ (qsegment_fn)
                if self.qsegment_fn.as_deref() == Some(local_name) {
                    return Some(("$".to_string(), false));
                }

                None
            }
            _ => None,
        }
    }

    /// Check if the first argument to a $ call is a function/arrow expression.
    fn first_arg_is_function(args: &[Argument<'_>]) -> bool {
        if args.is_empty() {
            return false;
        }
        matches!(
            &args[0],
            Argument::ArrowFunctionExpression(_) | Argument::FunctionExpression(_)
        )
    }

    /// Determine if a $ call should be emitted as a segment
    /// (not stripped by strip_ctx_name or strip_event_handlers).
    fn should_emit_segment(&self, ctx_name: &str, ctx_kind: &CtxKind) -> bool {
        // Check strip_ctx_name
        if self.strip_ctx_name.iter().any(|s| s == ctx_name) {
            return false;
        }

        // Check strip_event_handlers
        if self.strip_event_handlers && *ctx_kind == CtxKind::EventHandler {
            return false;
        }

        true
    }
}

// ---------------------------------------------------------------------------
// Traverse implementation
// ---------------------------------------------------------------------------

impl<'a> Traverse<'a, ()> for QwikTransform {
    fn enter_call_expression(
        &mut self,
        node: &mut CallExpression<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // CONV-01: Dollar detection
        // 1. Check if callee is a known $ marker function
        if let Some((ctx_name, is_sync)) = self.detect_dollar_call(&node.callee) {
            // 2. Verify first argument is a function/arrow expression
            if Self::first_arg_is_function(&node.arguments) {
                let ctx_kind = words::classify_ctx_kind(&ctx_name);

                // 3. Check if this segment should be emitted (not stripped)
                if self.should_emit_segment(&ctx_name, &ctx_kind) {
                    // 4. Push a SegmentScope onto segment_stack
                    self.segment_stack.push(SegmentScope {
                        ctx_name,
                        ctx_kind,
                        span_start: node.span.start,
                        is_sync,
                    });
                }
            }
        }
    }

    fn exit_expression(
        &mut self,
        _expr: &mut Expression<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Will be filled in Plan 05 with QRL wrapping and capture analysis
    }

    fn exit_program(
        &mut self,
        _program: &mut Program<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Will be filled in later plans with import rewriting
    }
}

// ---------------------------------------------------------------------------
// transform_code -- pipeline orchestration
// ---------------------------------------------------------------------------

/// Orchestrate the full transform pipeline for a single module.
///
/// Pipeline stages:
/// 1. Parse (from parse.rs)
/// 2. GlobalCollect (from collector.rs)
/// 3. Pre-traverse mutations (rename_imports, const_replace, filter_exports)
/// 4. `traverse_mut(&mut transformer, allocator, &mut program, scoping, ())`
/// 5. Return TransformOutput (segment emission comes in Plan 07)
pub(crate) fn transform_code(
    source: &str,
    filename: &str,
    config: &TransformCodeOptions,
) -> crate::types::TransformOutput {
    use oxc::allocator::Allocator;
    use oxc::codegen::Codegen;

    let allocator = Allocator::default();
    let source_in_arena: &str = allocator.alloc_str(source);

    // Stage 1: Parse
    let (parse_result, parse_diagnostics) =
        match crate::parse::parse_module(&allocator, source_in_arena, filename) {
            Ok(result) => result,
            Err(diagnostics) => {
                return crate::types::TransformOutput {
                    modules: vec![],
                    diagnostics,
                    is_type_script: false,
                    is_jsx: false,
                };
            }
        };

    let mut program = parse_result.program;
    let scoping = parse_result.scoping;
    let source_type = parse_result.source_type;

    // Stage 2: Pre-traverse rename imports
    crate::rename_imports::rename_imports(&mut program, &allocator);

    // Stage 3: GlobalCollect
    let collect = crate::collector::global_collect(&program);

    // Stage 4: Pre-traverse mutations
    crate::const_replace::replace_build_constants(&mut program, config, &collect, &allocator);
    crate::filter_exports::filter_exports(&mut program, &config.strip_exports, &allocator);
    crate::filter_exports::filter_ctx_names(
        &mut program,
        &config.strip_ctx_name,
        config.strip_event_handlers,
        &allocator,
    );

    // Stage 5: Determine file metadata
    let path_data = crate::parse::parse_path(
        filename,
        std::path::Path::new(&config.src_dir),
    )
    .unwrap_or_else(|_| crate::parse::PathData {
        file_stem: "unknown".to_string(),
        file_name: filename.to_string(),
        rel_dir: std::path::PathBuf::new(),
        abs_dir: std::path::PathBuf::from(&config.src_dir),
    });

    let extension = crate::parse::output_extension(
        filename,
        config.transpile_ts,
        config.transpile_jsx,
    );

    // Stage 6: Create QwikTransform and traverse
    let mut transformer = QwikTransform::new(
        config,
        &collect,
        &path_data.file_name,
        filename,
        extension,
    );

    oxc_traverse::traverse_mut(
        &mut transformer,
        &allocator,
        &mut program,
        scoping,
        (),
    );

    // Stage 7: Generate output (segment emission comes in Plan 07)
    let code = Codegen::new().build(&program).code;

    let mut diagnostics = parse_diagnostics;
    // Future: append transformer diagnostics

    crate::types::TransformOutput {
        modules: vec![crate::types::TransformModule {
            path: filename.to_string(),
            is_entry: false,
            code,
            map: None,
            segment: None,
            orig_path: Some(filename.to_string()),
            order: 0,
        }],
        diagnostics,
        is_type_script: source_type.is_typescript(),
        is_jsx: source_type.is_jsx(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EmitMode, MinifyMode};

    fn make_config() -> TransformCodeOptions {
        TransformCodeOptions {
            src_dir: "/project".to_string(),
            root_dir: None,
            source_maps: false,
            minify: MinifyMode::None,
            transpile_ts: false,
            transpile_jsx: false,
            preserve_filenames: false,
            entry_strategy: EntryStrategy::default(),
            explicit_extensions: false,
            mode: EmitMode::Dev,
            scope: None,
            core_module: "@qwik.dev/core".to_string(),
            strip_exports: vec![],
            strip_ctx_name: vec![],
            strip_event_handlers: false,
            reg_ctx_name: vec![],
            is_server: true,
        }
    }

    // -----------------------------------------------------------------------
    // Dollar detection tests
    // -----------------------------------------------------------------------

    #[test]
    fn dollar_detection_identifies_component_dollar() {
        let src = r#"
            import { component$ } from "@qwik.dev/core";
            export const App = component$(() => {
                return <div>Hello</div>;
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(
            output.modules.len() == 1,
            "Expected 1 output module, got: {}",
            output.modules.len()
        );
        assert!(
            output.diagnostics.is_empty(),
            "Expected no diagnostics, got: {:?}",
            output.diagnostics
        );
    }

    #[test]
    fn dollar_detection_identifies_bare_dollar() {
        let src = r#"
            import { $ } from "@qwik.dev/core";
            const handler = $(() => {
                console.log("hello");
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.modules.len() == 1);
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn dollar_detection_identifies_use_task() {
        let src = r#"
            import { useTask$ } from "@qwik.dev/core";
            useTask$(() => {
                console.log("task");
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.modules.len() == 1);
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn dollar_detection_identifies_sync_dollar() {
        let src = r#"
            import { sync$ } from "@qwik.dev/core";
            const fn1 = sync$(() => {
                return true;
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.modules.len() == 1);
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn dollar_detection_ignores_non_dollar_calls() {
        let src = r#"
            import { component$ } from "@qwik.dev/core";
            const x = someFunction(() => {});
            console.log("not a dollar call");
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.modules.len() == 1);
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn dollar_detection_ignores_non_qwik_dollar() {
        // A function named component$ but NOT imported from @qwik.dev/core
        let src = r#"
            import { component$ } from "other-lib";
            const App = component$(() => {});
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.modules.len() == 1);
    }

    #[test]
    fn transform_code_pipeline_compiles_end_to_end() {
        let src = r#"
            import { component$, useTask$ } from "@qwik.dev/core";
            import { isServer } from "@qwik.dev/core/build";

            export const App = component$(() => {
                useTask$(() => {
                    if (isServer) {
                        console.log("server only");
                    }
                });
                return <div>Hello</div>;
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.modules.len() == 1);
        assert!(output.diagnostics.is_empty());
        assert!(output.is_type_script);
        assert!(output.is_jsx);
        // isServer should be replaced with true (is_server=true, mode=Dev)
        assert!(
            output.modules[0].code.contains("true"),
            "isServer should be replaced with true, got: {}",
            output.modules[0].code
        );
    }

    #[test]
    fn dollar_detection_strip_ctx_name() {
        let src = r#"
            import { useTask$ } from "@qwik.dev/core";
            useTask$(() => {
                console.log("task");
            });
        "#;
        let mut config = make_config();
        config.strip_ctx_name = vec!["useTask$".to_string()];
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.modules.len() == 1);
        assert!(output.diagnostics.is_empty());
    }

    #[test]
    fn dollar_detection_event_handler_classification() {
        let src = r#"
            import { component$ } from "@qwik.dev/core";
            export const App = component$(() => {
                return <button onClick$={() => console.log("click")}>Hi</button>;
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.modules.len() == 1);
        assert!(output.diagnostics.is_empty());
    }

    // -----------------------------------------------------------------------
    // Marker function detection tests
    // -----------------------------------------------------------------------

    #[test]
    fn transform_marker_detection_includes_dollar_imports() {
        use crate::collector::global_collect_from_str;

        let src = r#"
            import { component$, useTask$, $ } from "@qwik.dev/core";
        "#;
        let collect = global_collect_from_str(src);
        let config = make_config();
        let t = QwikTransform::new(&config, &collect, "test.tsx", "test.tsx", "tsx");

        assert!(
            t.marker_functions.contains_key("component$"),
            "component$ should be in marker_functions"
        );
        assert!(
            t.marker_functions.contains_key("useTask$"),
            "useTask$ should be in marker_functions"
        );
        assert!(
            t.qsegment_fn.is_some(),
            "$ should be detected as qsegment_fn"
        );
    }

    #[test]
    fn transform_marker_detection_includes_local_exports() {
        use crate::collector::global_collect_from_str;

        let src = r#"
            export function myHelper$() {}
        "#;
        let collect = global_collect_from_str(src);
        let config = make_config();
        let t = QwikTransform::new(&config, &collect, "test.tsx", "test.tsx", "tsx");

        assert!(
            t.marker_functions.contains_key("myHelper$"),
            "myHelper$ should be in marker_functions"
        );
    }

    #[test]
    fn transform_sync_dollar_detection() {
        use crate::collector::global_collect_from_str;

        let src = r#"
            import { sync$ } from "@qwik.dev/core";
        "#;
        let collect = global_collect_from_str(src);
        let config = make_config();
        let t = QwikTransform::new(&config, &collect, "test.tsx", "test.tsx", "tsx");

        assert!(
            t.sync_qrl_fn.is_some(),
            "sync$ should be detected"
        );
        assert_eq!(t.sync_qrl_fn.as_deref(), Some("sync$"));
    }
}
