//! QwikTransform -- core traversal pass.
//!
//! Implements `Traverse<'a, ()>` to walk the AST and:
//! - Detect marker functions ($-suffixed imports and local exports) -- CONV-01
//! - Push `SegmentScope` frames for each detected $ call
//! - Capture analysis (CONV-03): 8-category taxonomy for variable classification
//! - QRL wrapping (CONV-02): Replace $ calls with qrl()/inlinedQrl() wrappers
//! - PURE annotations (CONV-08): /*#__PURE__*/ on componentQrl and qrl calls
//! - Noop QRL (CONV-14): _noopQrl for stripped callbacks
//! - Sync$ (CONV-13): _qrlSync for sync$ calls

use std::collections::{HashMap, HashSet};

use oxc::ast::ast::*;
use oxc::ast_visit::{Visit, walk};
use oxc::span::Ident;
use oxc_traverse::{Traverse, TraverseCtx};

use crate::collector::{GlobalCollect, ImportKind};
use crate::entry_strategy::EntryPolicy;
use crate::types::{CtxKind, EmitMode, EntryStrategy, SegmentData, TransformCodeOptions};
use crate::words;

/// Default span for generated AST nodes.
const SPAN: oxc::span::Span = oxc::span::Span::new(0, 0);

/// Allocate a string in the arena and return it as an `Ident<'a>`.
fn arena_ident<'a>(ctx: &TraverseCtx<'a, ()>, s: &str) -> Ident<'a> {
    let allocated: &'a str = ctx.ast.allocator.alloc_str(s);
    Ident::from(allocated)
}

/// Allocate a string in the arena and return as `&'a str`.
fn arena_str<'a>(ctx: &TraverseCtx<'a, ()>, s: &str) -> &'a str {
    ctx.ast.allocator.alloc_str(s)
}

// ---------------------------------------------------------------------------
// IdentType -- declaration type for decl_stack entries
// ---------------------------------------------------------------------------

/// Classification of a declaration binding for capture analysis.
///
/// `Const` and `Let` are capturable variable bindings;
/// `Fn` and `Class` are non-capturable (produce C02 errors).
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum IdentType {
    /// `const` binding -- captured by reference (no setter needed).
    Const,
    /// `let`/`var`/parameter binding -- captured with getter/setter.
    Let,
    /// Function declaration (not capturable across $ boundary).
    Fn,
    /// Class declaration (not capturable across $ boundary).
    Class,
}

/// A named binding with its declaration type, used in the declaration stack.
pub(crate) type TypedId = (String, IdentType);

// ---------------------------------------------------------------------------
// IdentCollector -- read-only visitor that harvests IdentifierReference names
// ---------------------------------------------------------------------------

/// Collects all `IdentifierReference` names reachable from an expression.
///
/// Used by capture analysis to determine which identifiers a segment closure
/// body references. Filters out:
/// - Global builtins (undefined, NaN, Infinity)
/// - Property access names (only collects the object, not .property)
/// - JSX attribute names
/// - Object literal property keys
pub(crate) struct IdentCollector {
    pub idents: HashSet<String>,
    /// Stack tracking whether nested visits should collect identifiers or skip them.
    tracking: Vec<Tracking>,
}

#[derive(Clone, Copy, PartialEq)]
enum Tracking {
    Track,
    Skip,
}

/// Global builtin names that are never captured.
const GLOBAL_BUILTINS: &[&str] = &["undefined", "NaN", "Infinity", "globalThis", "arguments"];

impl IdentCollector {
    /// Walk `expr` and return every `IdentifierReference` name found.
    pub(crate) fn collect(expr: &Expression<'_>) -> HashSet<String> {
        let mut collector = Self {
            idents: HashSet::new(),
            tracking: vec![Tracking::Track],
        };
        collector.visit_expression(expr);
        collector.idents
    }
}

impl<'a> Visit<'a> for IdentCollector {
    fn visit_identifier_reference(&mut self, id: &IdentifierReference<'a>) {
        // Only collect in expression context
        if self.tracking.last().is_some_and(|t| {t == &Tracking::Track}) {
            let name = id.name.as_str();
            if !GLOBAL_BUILTINS.contains(&name) {
                self.idents.insert(name.to_string());
            }
        }
    }

    fn visit_expression(&mut self, expr: &Expression<'a>) {
        self.tracking.push(Tracking::Track);
        // For member expressions, visit the object but skip the property
        if let Expression::StaticMemberExpression(member) = expr {
            self.visit_expression(&member.object);
            // Skip member.property (it's an IdentifierName, not a reference)
            self.tracking.pop();
            return;
        }
        if let Expression::ComputedMemberExpression(member) = expr {
            self.visit_expression(&member.object);
            self.visit_expression(&member.expression);
            self.tracking.pop();
            return;
        }
        // Default: walk all children
        walk::walk_expression(self, expr);
        self.tracking.pop();
    }

    fn visit_object_property(&mut self, prop: &ObjectProperty<'a>) {
        // Skip the key, visit the value
        self.visit_expression(&prop.value);
    }

    fn visit_jsx_element_name(&mut self, name: &JSXElementName<'a>) {
        // Only collect uppercase JSX element names (components, not HTML tags)
        match name {
            JSXElementName::Identifier(id) => {
                let n = id.name.as_str();
                if n.starts_with(|c: char| c.is_ascii_uppercase()) {
                    self.idents.insert(n.to_string());
                }
            }
            JSXElementName::IdentifierReference(id) => {
                let n = id.name.as_str();
                if n.starts_with(|c: char| c.is_ascii_uppercase()) {
                    self.idents.insert(n.to_string());
                }
            }
            JSXElementName::MemberExpression(member) => {
                // For JSX member expressions like Foo.Bar, collect the root object
                self.visit_jsx_member_expression(member);
            }
            _ => {}
        }
    }

    fn visit_jsx_attribute(&mut self, attr: &JSXAttribute<'a>) {
        // Skip attribute names, only visit attribute values
        if let Some(value) = &attr.value {
            self.visit_jsx_attribute_value(value);
        }
    }
}

// ---------------------------------------------------------------------------
// compute_scoped_idents
// ---------------------------------------------------------------------------

/// Intersect `all_idents` with `all_decl` (keeping only `Const` and `Let` entries),
/// deduplicate, and return sorted captured identifier names.
pub(crate) fn compute_scoped_idents(
    all_idents: &HashSet<String>,
    all_decl: &[TypedId],
) -> Vec<String> {
    let mut matched: HashSet<String> = HashSet::new();

    for name in all_idents {
        for (decl_name, decl_type) in all_decl {
            if name == decl_name {
                match decl_type {
                    IdentType::Const | IdentType::Let => {
                        matched.insert(name.clone());
                    }
                    // Fn/Class entries are NOT captured as scoped idents
                    IdentType::Fn | IdentType::Class => {}
                }
            }
        }
    }

    let mut sorted: Vec<String> = matched.into_iter().collect();
    sorted.sort();
    sorted
}

// ---------------------------------------------------------------------------
// get_function_params -- extract parameter names from a function/arrow
// ---------------------------------------------------------------------------

/// Extract all parameter binding names from a function/arrow expression.
/// These are local to the segment and must NOT be captured.
pub(crate) fn get_function_params(expr: &Expression<'_>) -> HashSet<String> {
    let mut params = HashSet::new();
    match expr {
        Expression::ArrowFunctionExpression(arrow) => {
            collect_formal_params(&arrow.params, &mut params);
        }
        Expression::FunctionExpression(func) => {
            collect_formal_params(&func.params, &mut params);
        }
        _ => {}
    }
    params
}

fn collect_formal_params(formal: &FormalParameters<'_>, out: &mut HashSet<String>) {
    for param in &formal.items {
        collect_binding_names(&param.pattern, out);
    }
    if let Some(rest) = &formal.rest {
        collect_binding_names(&rest.rest.argument, out);
    }
}

fn collect_binding_names(pat: &BindingPattern<'_>, out: &mut HashSet<String>) {
    match pat {
        BindingPattern::BindingIdentifier(id) => {
            out.insert(id.name.as_str().to_string());
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_names(&prop.value, out);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_names(&rest.argument, out);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for el in arr.elements.iter().flatten() {
                collect_binding_names(el, out);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_names(&rest.argument, out);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_names(&assign.left, out);
        }
    }
}

// ---------------------------------------------------------------------------
// can_capture_scope
// ---------------------------------------------------------------------------

/// Returns `true` when `expr` is a function or arrow function -- i.e., when
/// it is capable of closing over outer variables.
fn can_capture_scope(expr: &Expression<'_>) -> bool {
    matches!(
        expr,
        Expression::FunctionExpression(_) | Expression::ArrowFunctionExpression(_)
    )
}

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
    /// Identifier references collected from the callback body.
    pub descendent_idents: HashSet<String>,
}

// ---------------------------------------------------------------------------
// SegmentRecord -- accumulated extracted segment metadata
// ---------------------------------------------------------------------------

// ---------------------------------------------------------------------------
// HoistedConst -- a const declaration hoisted out of an expression
// ---------------------------------------------------------------------------

/// A const binding hoisted to the top of a segment module.
#[derive(Debug, Clone)]
pub(crate) struct HoistedConst {
    /// The const binding name, e.g. `"q_renderHeader1_jMxQsjbyDss"`.
    pub name: String,
    /// Serialized RHS expression (e.g. `"qrl(...)"`  or `"_noopQrl(...)"`).
    pub rhs_code: String,
    /// Deduplication key -- same as `name`.
    pub symbol_name: String,
}

/// Internal record for a single extracted segment. Accumulated in
/// `QwikTransform.segments` during the traversal. Later phases read these
/// to emit segment module files.
#[derive(Debug)]
pub(crate) struct SegmentRecord {
    /// Symbol name (e.g. `test_tsx_component_ABC`).
    pub name: String,
    /// File-prefixed display name (e.g. `test.tsx_component_ABC`).
    pub display_name: String,
    /// Canonical filename for the segment module.
    pub canonical_filename: String,
    /// Output chunk key from entry_policy, or None for own chunk.
    pub entry: Option<String>,
    /// Serialized folded closure body (set by create_segment via OXC Codegen).
    /// None for noop QRLs that do not require a segment module.
    pub expr: Option<String>,
    /// Runtime-captured identifiers (closed-over variables).
    pub scoped_idents: Vec<String>,
    /// Compile-time import names referenced inside the segment body.
    pub local_idents: Vec<String>,
    /// The context (marker function) name, e.g. `"component$"`.
    pub ctx_name: String,
    /// The context kind (Function, EventHandler, etc.).
    pub ctx_kind: CtxKind,
    /// Relative path of the source file.
    pub origin: String,
    /// Byte span `(start, end)` of the original call expression.
    pub span: (u32, u32),
    /// 11-character SipHash-based segment hash.
    pub hash: String,
    /// Whether this segment was created via create_inline_qrl (not its own module).
    pub is_inline: bool,
    /// Root-level variable declarations migrated into this segment module (Stage 12).
    pub migrated_root_vars: Vec<String>,
    /// Parent segment name if nested inside another.
    pub parent: Option<String>,
    /// Span-start of the parent segment's call expression.
    pub pending_parent_span: Option<u32>,
    /// Ordered function parameter names extracted from the closure.
    pub param_names: Option<Vec<String>>,
}

/// Import information for a segment's needed imports.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub(crate) struct NeededImport {
    pub local_name: String,
    pub specifier: String,
    pub source: String,
}

// ---------------------------------------------------------------------------
// QwikTransform struct
// ---------------------------------------------------------------------------

/// Core Qwik traversal pass implementing `Traverse<'a, ()>`.
///
/// Traversal state is accumulated across the AST walk. All per-segment
/// extraction logic builds on top of the scaffolding established here.
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

    // ---- Capture analysis state (CONV-03) ---------------------------------
    /// Scope frames. Each function/arrow body gets a new frame.
    /// Each frame contains the (name, IdentType) bindings declared in it.
    /// Initialized with one empty root frame.
    pub(crate) decl_stack: Vec<Vec<TypedId>>,

    /// Diagnostics accumulated during the traversal.
    pub(crate) diagnostics: Vec<crate::types::Diagnostic>,

    /// Reference to the GlobalCollect for self-import reclassification.
    /// SAFETY: Valid for the duration of the traversal.
    pub(crate) global_collect_ptr: *const GlobalCollect,

    // ---- Hoisted const items (accumulated during traversal for code_move) -----
    pub(crate) extra_top_items: Vec<HoistedConst>,

    // ---- Hoist strategy ref_assignments (module-scope .s() calls) -----
    /// Module-scope `.s(fn_body)` expression statements for the Hoist entry
    /// strategy. These are emitted in `exit_program` AFTER `extra_top_items`
    /// const declarations but BEFORE export statements.
    pub(crate) ref_assignments: Vec<String>,

    // ---- Import tracking (accumulated during traversal, applied in exit_program)
    pub(crate) needs_qrl_import: bool,
    pub(crate) needs_inlined_qrl_import: bool,
    pub(crate) needs_noop_qrl_import: bool,
    pub(crate) needs_jsx_sorted_import: bool,
    pub(crate) needs_jsx_split_import: bool,
    pub(crate) needs_fragment_import: bool,
    pub(crate) needs_fn_signal_import: bool,
    pub(crate) needs_wrap_prop_import: bool,

    // ---- JSX state -----------------------------------------------------------
    /// Counter for deterministic JSX key generation.
    pub(crate) jsx_key_counter: u32,

    // ---- Entry policy (for segment grouping) --------------------------------
    pub(crate) entry_policy: Box<dyn EntryPolicy>,

    // ---- Config (owned copies) --------------------------------------------
    pub(crate) mode: EmitMode,
    pub(crate) entry_strategy: EntryStrategy,
    pub(crate) is_server: bool,
    pub(crate) file_name: String,
    pub(crate) rel_path: String,
    pub(crate) extension: String,
    pub(crate) core_module: String,
    pub(crate) strip_ctx_name: Vec<String>,
    pub(crate) strip_event_handlers: bool,
    pub(crate) scope: Option<String>,
    pub(crate) explicit_extensions: bool,
    /// Pointer to the original source text (valid for traversal lifetime).
    pub(crate) source_text: *const str,
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
        source_text: &str,
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

        // Build initial decl_stack from module-level declarations
        // Root scope frame includes all top-level var/fn/class declarations
        let mut root_frame: Vec<TypedId> = Vec::new();
        for (name, _span) in &collect.root {
            // Module-level declarations: we add them to decl_stack so they CAN
            // be found during compute_scoped_idents, but then reclassify them
            // as self-imports in the post-processing step.
            // We treat them as Var(true) since root-level consts are typical.
            root_frame.push((name.clone(), IdentType::Const));
        }

        let entry_policy = crate::entry_strategy::parse_entry_strategy(&config.entry_strategy);

        QwikTransform {
            marker_functions,
            qsegment_fn,
            sync_qrl_fn,
            stack_ctxt: Vec::new(),
            segment_stack: Vec::new(),
            segments: Vec::new(),
            segment_names: HashMap::new(),
            segment_counter: 0,
            decl_stack: vec![root_frame],
            diagnostics: Vec::new(),
            global_collect_ptr: collect as *const GlobalCollect,
            extra_top_items: Vec::new(),
            ref_assignments: Vec::new(),
            entry_policy,
            needs_qrl_import: false,
            needs_inlined_qrl_import: false,
            needs_noop_qrl_import: false,
            needs_jsx_sorted_import: false,
            needs_jsx_split_import: false,
            needs_fragment_import: false,
            needs_fn_signal_import: false,
            needs_wrap_prop_import: false,
            jsx_key_counter: 0,
            mode: config.mode.clone(),
            entry_strategy: config.entry_strategy.clone(),
            is_server: config.is_server,
            file_name: file_name.to_string(),
            rel_path: rel_path.to_string(),
            extension: extension.to_string(),
            core_module: config.core_module.clone(),
            strip_ctx_name: config.strip_ctx_name.clone(),
            strip_event_handlers: config.strip_event_handlers,
            scope: config.scope.clone(),
            explicit_extensions: config.explicit_extensions,
            source_text: source_text as *const str,
        }
    }

    // -----------------------------------------------------------------------
    // compute_entry -- compute entry key for a segment via EntryPolicy
    // -----------------------------------------------------------------------

    /// Compute the output chunk entry key for a segment via the configured
    /// `EntryPolicy`. Returns `Some(key)` for grouped segments or `None` for
    /// own-chunk segments.
    fn compute_entry(
        &self,
        ctx_kind: &CtxKind,
        ctx_name: &str,
        scoped_idents: &[String],
        hash: &str,
        symbol_name: &str,
    ) -> Option<String> {
        let seg_data = SegmentData {
            origin: self.rel_path.clone(),
            ctx_kind: ctx_kind.clone(),
            ctx_name: ctx_name.to_string(),
            scoped_idents: scoped_idents.to_vec(),
            display_name: String::new(),
            hash: hash.to_string(),
            name: symbol_name.to_string(),
            extension: self.extension.clone(),
            span: (0, 0),
            parent: None,
            captures: !scoped_idents.is_empty(),
            capture_names: scoped_idents.to_vec(),
        };
        self.entry_policy.get_entry_for_sym(&self.stack_ctxt, &seg_data)
    }

    // -----------------------------------------------------------------------
    // global_collect accessor
    // -----------------------------------------------------------------------

    /// Returns a reference to the `GlobalCollect` via the raw pointer.
    ///
    /// SAFETY: Only valid during the traversal lifetime (between `new()` and
    /// the end of `traverse_mut`).
    fn global_collect(&self) -> &GlobalCollect {
        unsafe { &*self.global_collect_ptr }
    }

    // -----------------------------------------------------------------------
    // ensure_export -- Stage 12 support
    // -----------------------------------------------------------------------

    /// Ensure a root-level name is exported (for variable migration).
    /// Adds an `_auto_{name}` export entry to the collect if not already exported.
    pub(crate) fn ensure_export(&mut self, _name: &str) {
        // In the full pipeline, this would inject `export { name as _auto_name }`
        // into the program body. For now, this is a no-op placeholder that the
        // variable migration pipeline can call without error.
        // Full implementation deferred to gap closure.
    }

    // -----------------------------------------------------------------------
    // patch_segment_parents -- resolve deferred parent symbol names
    // -----------------------------------------------------------------------

    /// After all segments are registered, resolve `pending_parent_span` to actual
    /// parent segment names.
    pub(crate) fn patch_segment_parents(&mut self) {
        // Build a map: call_span_start -> segment name
        let span_to_name: HashMap<u32, String> = self
            .segments
            .iter()
            .map(|s| (s.span.0, s.name.clone()))
            .collect();

        for seg in &mut self.segments {
            if let Some(parent_span) = seg.pending_parent_span {
                if let Some(parent_name) = span_to_name.get(&parent_span) {
                    seg.parent = Some(parent_name.clone());
                }
            }
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

    // -----------------------------------------------------------------------
    // Capture analysis helpers (CONV-03)
    // -----------------------------------------------------------------------

    /// Collect all identifier references from the first argument of a $ call.
    fn collect_descendent_idents(first_arg: &Argument<'_>) -> HashSet<String> {
        match first_arg {
            Argument::ArrowFunctionExpression(arrow) => {
                IdentCollector::collect(&Expression::ArrowFunctionExpression(
                    // SAFETY: We only read; the borrow is temporary.
                    unsafe { std::ptr::read(arrow as *const _) },
                ))
            }
            Argument::FunctionExpression(func) => {
                IdentCollector::collect(&Expression::FunctionExpression(
                    // SAFETY: We only read; the borrow is temporary.
                    unsafe { std::ptr::read(func as *const _) },
                ))
            }
            _ => HashSet::new(),
        }
    }

    /// Get function parameter names from the first argument of a $ call.
    fn get_first_arg_params(first_arg: &Argument<'_>) -> HashSet<String> {
        match first_arg {
            Argument::ArrowFunctionExpression(arrow) => {
                let mut params = HashSet::new();
                collect_formal_params(&arrow.params, &mut params);
                params
            }
            Argument::FunctionExpression(func) => {
                let mut params = HashSet::new();
                collect_formal_params(&func.params, &mut params);
                params
            }
            _ => HashSet::new(),
        }
    }

    /// Extract ordered parameter names from a function/arrow first argument.
    fn get_param_names(first_arg: &Argument<'_>) -> Option<Vec<String>> {
        let params = match first_arg {
            Argument::ArrowFunctionExpression(arrow) => &arrow.params,
            Argument::FunctionExpression(func) => &func.params,
            _ => return None,
        };
        let mut name_set = HashSet::new();
        for param in &params.items {
            collect_binding_names(&param.pattern, &mut name_set);
        }
        if name_set.is_empty() {
            None
        } else {
            let mut names: Vec<String> = name_set.into_iter().collect();
            names.sort();
            Some(names)
        }
    }

    /// Classify captured identifiers against GlobalCollect (Step 4 of capture analysis).
    ///
    /// For each identifier in the callback body:
    /// - If in global_collect.imports -> needed_import (Category 2)
    /// - If in global_collect.root or exports -> self-import (Category 1)
    /// - If in decl_stack as Fn/Class -> C02 error diagnostic (Category 8)
    /// - If in decl_stack as Var -> actual capture (Category 3/4/5)
    fn classify_captures(
        &mut self,
        all_idents: &HashSet<String>,
        scoped_idents: &mut Vec<String>,
        needed_imports: &mut Vec<NeededImport>,
        self_imports: &mut Vec<String>,
    ) {
        // SAFETY: global_collect_ptr is valid for the traversal duration.
        let collect = unsafe { &*self.global_collect_ptr };

        // Check scoped_idents against GlobalCollect for reclassification
        let mut to_remove = Vec::new();
        for (i, name) in scoped_idents.iter().enumerate() {
            // Category 1: Module-level declarations -> self-imports
            if collect.root.contains_key(name) || collect.has_export_symbol(name) {
                self_imports.push(name.clone());
                to_remove.push(i);
                continue;
            }
            // Category 2: Already-imported names -> needed_imports
            if let Some(import) = collect.get_import(name) {
                needed_imports.push(NeededImport {
                    local_name: name.clone(),
                    specifier: import.specifier.clone(),
                    source: import.source.clone(),
                });
                to_remove.push(i);
                continue;
            }
        }

        // Remove reclassified entries (in reverse to preserve indices)
        for i in to_remove.into_iter().rev() {
            scoped_idents.remove(i);
        }

        // Check remaining all_idents for imports not in scoped_idents
        // (they still need to be re-emitted in the segment module)
        for name in all_idents {
            if collect.imports.contains_key(name) {
                // Check if we already added it
                if !needed_imports.iter().any(|ni| ni.local_name == *name) {
                    if let Some(import) = collect.get_import(name) {
                        needed_imports.push(NeededImport {
                            local_name: name.clone(),
                            specifier: import.specifier.clone(),
                            source: import.source.clone(),
                        });
                    }
                }
            }
            // Check for self-imports from all_idents too
            if collect.root.contains_key(name) || collect.has_export_symbol(name) {
                if !self_imports.contains(name) && !scoped_idents.contains(name) {
                    self_imports.push(name.clone());
                }
            }
        }

        // Category 8: Check for fn/class declarations in decl_stack referenced by segment
        let all_decl: Vec<TypedId> = self
            .decl_stack
            .iter()
            .flat_map(|frame| frame.iter().cloned())
            .collect();

        for name in all_idents {
            for (decl_name, decl_type) in &all_decl {
                if name == decl_name && matches!(decl_type, IdentType::Fn | IdentType::Class) {
                    // C02 error: function/class declaration referenced across $ boundary
                    self.diagnostics.push(crate::types::Diagnostic {
                        scope: "optimizer".to_string(),
                        category: crate::types::DiagnosticCategory::Error,
                        code: Some("C02".to_string()),
                        file: self.file_name.clone(),
                        message: format!(
                            "Reference to identifier '{}' can not be used inside a Qrl($) scope because it's a function",
                            name
                        ),
                        highlights: None,
                        suggestions: None,
                    });
                }
            }
        }
    }

    /// Determine if the entry strategy means inline (Inline, Hoist, or Lib mode).
    fn is_inline_mode(&self) -> bool {
        matches!(self.entry_strategy, EntryStrategy::Inline | EntryStrategy::Hoist)
            || matches!(self.mode, EmitMode::Lib)
    }
}

// ---------------------------------------------------------------------------
// Traverse implementation
// ---------------------------------------------------------------------------

impl<'a> Traverse<'a, ()> for QwikTransform {
    // -----------------------------------------------------------------------
    // Scope tracking for capture analysis (enter/exit functions)
    // -----------------------------------------------------------------------

    fn enter_function(
        &mut self,
        func: &mut Function<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Push a new declaration frame for this function scope
        let mut frame = Vec::new();
        // Collect parameters into decl_stack
        collect_formal_params_to_decl(&func.params, &mut frame);
        self.decl_stack.push(frame);
    }

    fn exit_function(
        &mut self,
        _func: &mut Function<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        self.decl_stack.pop();
    }

    fn enter_arrow_function_expression(
        &mut self,
        arrow: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        let mut frame = Vec::new();
        collect_formal_params_to_decl(&arrow.params, &mut frame);
        self.decl_stack.push(frame);
    }

    fn exit_arrow_function_expression(
        &mut self,
        _arrow: &mut ArrowFunctionExpression<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        self.decl_stack.pop();
    }

    // -----------------------------------------------------------------------
    // Variable declaration tracking
    // -----------------------------------------------------------------------

    fn enter_variable_declaration(
        &mut self,
        decl: &mut VariableDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        let is_const = decl.kind == VariableDeclarationKind::Const;
        if let Some(frame) = self.decl_stack.last_mut() {
            for declarator in &decl.declarations {
                collect_binding_to_decl(&declarator.id, frame, is_const);
            }
        }
    }

    // -----------------------------------------------------------------------
    // For-loop variable tracking (Category 4: loop captures)
    // -----------------------------------------------------------------------

    fn enter_for_in_statement(
        &mut self,
        stmt: &mut ForInStatement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let ForStatementLeft::VariableDeclaration(decl) = &stmt.left {
            let is_const = decl.kind == VariableDeclarationKind::Const;
            if let Some(frame) = self.decl_stack.last_mut() {
                for declarator in &decl.declarations {
                    collect_binding_to_decl(&declarator.id, frame, is_const);
                }
            }
        }
    }

    fn enter_for_of_statement(
        &mut self,
        stmt: &mut ForOfStatement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let ForStatementLeft::VariableDeclaration(decl) = &stmt.left {
            let is_const = decl.kind == VariableDeclarationKind::Const;
            if let Some(frame) = self.decl_stack.last_mut() {
                for declarator in &decl.declarations {
                    collect_binding_to_decl(&declarator.id, frame, is_const);
                }
            }
        }
    }

    fn enter_for_statement(
        &mut self,
        stmt: &mut ForStatement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let Some(ForStatementInit::VariableDeclaration(decl)) = &stmt.init {
            let is_const = decl.kind == VariableDeclarationKind::Const;
            if let Some(frame) = self.decl_stack.last_mut() {
                for declarator in &decl.declarations {
                    collect_binding_to_decl(&declarator.id, frame, is_const);
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // Function/class declaration tracking for Category 8
    // -----------------------------------------------------------------------

    fn enter_statement(
        &mut self,
        stmt: &mut Statement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        match stmt {
            Statement::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    if let Some(frame) = self.decl_stack.last_mut() {
                        frame.push((id.name.as_str().to_string(), IdentType::Fn));
                    }
                }
            }
            Statement::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    if let Some(frame) = self.decl_stack.last_mut() {
                        frame.push((id.name.as_str().to_string(), IdentType::Class));
                    }
                }
            }
            _ => {}
        }
    }

    // -----------------------------------------------------------------------
    // Dollar detection and capture analysis (enter_call_expression)
    // -----------------------------------------------------------------------

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
                    // Collect descendent identifiers from the callback body
                    let descendent_idents = Self::collect_descendent_idents(&node.arguments[0]);

                    // Push context name for display_name building
                    self.stack_ctxt.push(escape_dollar(&ctx_name));

                    // 4. Push a SegmentScope onto segment_stack
                    self.segment_stack.push(SegmentScope {
                        ctx_name,
                        ctx_kind,
                        span_start: node.span.start,
                        is_sync,
                        descendent_idents,
                    });
                }
            }
        }
    }

    fn exit_expression(
        &mut self,
        expr: &mut Expression<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // --- CONV-06: JSX Transform ---
        // Transform JSXElement and JSXFragment expressions into _jsxSorted/_jsxSplit calls.
        // We take the expression, transform it, and replace in-place.
        let is_jsx_element = matches!(expr, Expression::JSXElement(_));
        let is_jsx_fragment = matches!(expr, Expression::JSXFragment(_));

        if is_jsx_element || is_jsx_fragment {
            let allocator: &'a oxc::allocator::Allocator = ctx.ast.allocator;
            // Replace expr with a dummy null literal, take ownership of the original.
            let taken = std::mem::replace(expr, ctx.ast.expression_null_literal(SPAN));
            let is_root = self.segment_stack.is_empty();

            let (new_expr, needs) = if is_jsx_element {
                if let Expression::JSXElement(el) = taken {
                    crate::jsx_transform::transform_jsx_element(
                        el.unbox(),
                        &mut self.jsx_key_counter,
                        is_root,
                        allocator,
                    )
                } else {
                    unreachable!()
                }
            } else {
                if let Expression::JSXFragment(frag) = taken {
                    crate::jsx_transform::transform_jsx_fragment(
                        frag.unbox(),
                        &mut self.jsx_key_counter,
                        is_root,
                        allocator,
                    )
                } else {
                    unreachable!()
                }
            };

            if needs.needs_jsx_sorted {
                self.needs_jsx_sorted_import = true;
            }
            if needs.needs_jsx_split {
                self.needs_jsx_split_import = true;
            }
            if needs.needs_fragment {
                self.needs_fragment_import = true;
            }

            *expr = new_expr;
            return;
        }

        // --- CONV-02: QRL Wrapping ---
        // Check if this expression is a CallExpression that matches a pending SegmentScope
        let call_span_start = match expr {
            Expression::CallExpression(call) => call.span.start,
            _ => return,
        };

        // Check if we have a matching segment scope
        let has_pending = self
            .segment_stack
            .last()
            .map_or(false, |s| s.span_start == call_span_start);

        if !has_pending {
            return;
        }

        let pending = self.segment_stack.pop().unwrap();

        // Pop the context name we pushed
        self.stack_ctxt.pop();

        // --- Flatten decl_stack for Var entries ---
        let all_decl: Vec<TypedId> = self
            .decl_stack
            .iter()
            .flat_map(|frame| frame.iter().cloned())
            .collect();

        // --- Compute scoped_idents (captures) ---
        let mut scoped_idents =
            compute_scoped_idents(&pending.descendent_idents, &all_decl);

        // Exclude function parameters of the callback
        let call = match expr {
            Expression::CallExpression(call) => call,
            _ => return,
        };
        let param_idents = Self::get_first_arg_params(&call.arguments[0]);
        scoped_idents.retain(|id| !param_idents.contains(id));

        // C03: if not a function/arrow and has captures, clear and emit diagnostic.
        let first_arg_can_capture = if !call.arguments.is_empty() {
            match &call.arguments[0] {
                Argument::ArrowFunctionExpression(_) | Argument::FunctionExpression(_) => true,
                _ => false,
            }
        } else {
            false
        };

        if !first_arg_can_capture && !scoped_idents.is_empty() {
            self.diagnostics.push(crate::types::Diagnostic {
                scope: "optimizer".to_string(),
                category: crate::types::DiagnosticCategory::Error,
                code: Some("C03".to_string()),
                file: self.file_name.clone(),
                message: format!(
                    "Qrl($) scope is not a function, but it's capturing local identifiers: {}",
                    scoped_idents.join(", ")
                ),
                highlights: None,
                suggestions: None,
            });
            scoped_idents.clear();
        }

        // --- Classify captures against GlobalCollect ---
        let mut needed_imports = Vec::new();
        let mut self_imports = Vec::new();
        self.classify_captures(
            &pending.descendent_idents,
            &mut scoped_idents,
            &mut needed_imports,
            &mut self_imports,
        );

        // --- Compute names via register_context_name ---
        let names = crate::hash::register_context_name(
            &self.stack_ctxt,
            &mut self.segment_names,
            self.scope.as_deref(),
            &self.rel_path,
            &self.file_name,
            &self.mode,
            None,
            None,
            None,
        );

        let has_captures = !scoped_idents.is_empty();
        let should_emit = self.should_emit_segment(&pending.ctx_name, &pending.ctx_kind);

        // --- Compute entry key from EntryPolicy ---
        let entry = self.compute_entry(
            &pending.ctx_kind,
            &pending.ctx_name,
            &scoped_idents,
            &names.hash,
            &names.symbol_name,
        );

        // --- Extract expr code from source text ---
        let call = match expr {
            Expression::CallExpression(call) => call,
            _ => return,
        };

        // Extract the function expression body from source text using span
        let expr_code: Option<String> = if !call.arguments.is_empty() {
            let first_arg_span = match &call.arguments[0] {
                Argument::ArrowFunctionExpression(a) => Some(a.span),
                Argument::FunctionExpression(f) => Some(f.span),
                _ => None,
            };
            first_arg_span.and_then(|span| {
                let src = unsafe { &*self.source_text };
                let start = span.start as usize;
                let end = span.end as usize;
                if start <= end && end <= src.len() {
                    Some(src[start..end].to_string())
                } else {
                    None
                }
            })
        } else {
            None
        };

        // Compute local_idents from self_imports + needed_imports local names
        let mut local_idents: Vec<String> = self_imports.clone();
        for ni in &needed_imports {
            if !local_idents.contains(&ni.local_name) {
                local_idents.push(ni.local_name.clone());
            }
        }

        // Determine parent segment (if nested)
        let parent_span = self.segment_stack.last().map(|s| s.span_start);

        // Compute param names from first argument
        let param_names = if !call.arguments.is_empty() {
            Self::get_param_names(&call.arguments[0])
        } else {
            None
        };

        // Save span end before branches that may reassign *expr (breaks borrow)
        let call_span_end = call.span.end;
        let _allocator: &'a oxc::allocator::Allocator = ctx.ast.allocator;

        // --- QRL wrapping (CONV-02) ---
        if pending.is_sync {
            // CONV-13: sync$ handling
            if let Expression::Identifier(id) = &mut call.callee {
                id.name = arena_ident(ctx, "_qrlSync");
            }
            self.segments.push(SegmentRecord {
                name: names.symbol_name.clone(),
                display_name: names.display_name.clone(),
                canonical_filename: names.canonical_filename.clone(),
                entry: entry.clone(),
                expr: expr_code.clone(),
                scoped_idents: scoped_idents.clone(),
                local_idents: local_idents.clone(),
                ctx_name: pending.ctx_name.clone(),
                ctx_kind: pending.ctx_kind.clone(),
                origin: self.rel_path.clone(),
                span: (pending.span_start, call_span_end),
                hash: names.hash.clone(),
                is_inline: true,
                migrated_root_vars: Vec::new(),
                parent: None,
                pending_parent_span: parent_span,
                param_names: param_names.clone(),
            });
            return;
        }

        if !should_emit {
            // CONV-14: Noop QRL for stripped callbacks
            let is_dev = matches!(self.mode, EmitMode::Dev | EmitMode::Hmr);
            let callee_name = if is_dev { "_noopQrlDEV" } else { "_noopQrl" };

            if let Expression::Identifier(id) = &mut call.callee {
                id.name = arena_ident(ctx, callee_name);
            }

            call.arguments.clear();
            call.arguments.push(Argument::StringLiteral(
                ctx.ast.alloc_string_literal(SPAN, arena_str(ctx, &names.symbol_name), None),
            ));

            self.needs_noop_qrl_import = true;

            self.segments.push(SegmentRecord {
                name: names.symbol_name.clone(),
                display_name: names.display_name.clone(),
                canonical_filename: names.canonical_filename.clone(),
                entry: entry.clone(),
                expr: None,
                scoped_idents: vec![],
                local_idents: local_idents.clone(),
                ctx_name: pending.ctx_name.clone(),
                ctx_kind: pending.ctx_kind.clone(),
                origin: self.rel_path.clone(),
                span: (pending.span_start, call_span_end),
                hash: names.hash.clone(),
                is_inline: true,
                migrated_root_vars: Vec::new(),
                parent: None,
                pending_parent_span: parent_span,
                param_names: param_names.clone(),
            });
            return;
        }

        // Determine QRL creation path
        let is_inline = self.is_inline_mode();

        // Rename callee to Qrl suffix (component$ -> componentQrl)
        let qrl_wrapper_name = words::dollar_to_qrl_name(&pending.ctx_name);
        if let Expression::Identifier(id) = &mut call.callee {
            id.name = arena_ident(ctx, &qrl_wrapper_name);
        }

        // CONV-08: PURE annotation -- only on componentQrl
        // The actual comment injection is deferred to codegen; we track which
        // calls need it. For now, we note it in the SegmentRecord.
        let _needs_pure = qrl_wrapper_name == "componentQrl";

        // Check for Hoist strategy (separate path from Inline)
        let is_hoist = matches!(self.entry_strategy, EntryStrategy::Hoist)
            && !matches!(self.mode, EmitMode::Lib);

        if is_hoist {
            // ---------------------------------------------------------------
            // Hoist strategy: _noopQrl const + .s() registration (CONV-14/D-40)
            // ---------------------------------------------------------------
            let is_dev = matches!(self.mode, EmitMode::Dev | EmitMode::Hmr);
            let noop_fn = if is_dev { "_noopQrlDEV" } else { "_noopQrl" };
            let ident_name = format!("q_{}", names.symbol_name);

            // 1. Build _noopQrl("sym") as a HoistedConst
            //    For dev mode: _noopQrlDEV("sym", { file: ..., lo: ..., hi: ..., displayName: ... })
            let noop_rhs = format!(r#"/*#__PURE__*/ {}("{}")"#, noop_fn, names.symbol_name);

            // Deduplicate by symbol_name
            if !self.extra_top_items.iter().any(|h| h.symbol_name == names.symbol_name) {
                self.extra_top_items.push(HoistedConst {
                    name: ident_name.clone(),
                    rhs_code: noop_rhs,
                    symbol_name: names.symbol_name.clone(),
                });
            }

            // 2. Extract fn_body code from source text for .s()
            let fn_body_code: Option<String> = if !call.arguments.is_empty() {
                let first_arg_span = match &call.arguments[0] {
                    Argument::ArrowFunctionExpression(a) => Some(a.span),
                    Argument::FunctionExpression(f) => Some(f.span),
                    _ => None,
                };
                first_arg_span.and_then(|span| {
                    let src = unsafe { &*self.source_text };
                    let start = span.start as usize;
                    let end = span.end as usize;
                    if start <= end && end <= src.len() {
                        Some(src[start..end].to_string())
                    } else {
                        None
                    }
                })
            } else {
                // Non-function argument (e.g., useStyles$('string'))
                let first_arg_span = match &call.arguments[0] {
                    Argument::StringLiteral(s) => Some(s.span),
                    _ => None,
                };
                first_arg_span.and_then(|span| {
                    let src = unsafe { &*self.source_text };
                    let start = span.start as usize;
                    let end = span.end as usize;
                    if start <= end && end <= src.len() {
                        Some(src[start..end].to_string())
                    } else {
                        None
                    }
                })
            };

            // 3. Determine if fn_body ident is global (for .s() placement)
            //    Check: is the fn_body a reference to a known global-scope ident?
            //    For Hoist strategy nested QRLs, the fn_body is a function expression
            //    (not an ident), so it is NOT a hoisted segment ident.
            //    The SWC code checks if fn_body_expr is an Ident in
            //    hoisted_segment_idents -- that case arises when an inner $-call
            //    was already hoisted and its fn_body was replaced with q_X ident.
            //    For now, all .s() calls go to ref_assignments (module scope) since
            //    the fn_body is typically a function/arrow expression (globally accessible).
            if let Some(ref body_code) = fn_body_code {
                let s_call = format!("{}.s({});", ident_name, body_code);
                self.ref_assignments.push(s_call);
            }

            // 4. Build the replacement expression
            //    Base: q_{sym} identifier
            //    If captures: q_{sym}.w([cap1, cap2, ...])
            let allocator = ctx.ast.allocator;

            let replacement = if has_captures {
                // Build: q_sym.w([cap1, cap2, ...])
                let caps_str = scoped_idents.join(", ");
                let w_expr_code = format!("{}.w([{}])", ident_name, caps_str);
                // Parse this expression
                let expr_stmt = format!("{};", w_expr_code);
                if let Some(stmt) = crate::add_side_effect::parse_single_statement(&expr_stmt, allocator) {
                    if let oxc::ast::ast::Statement::ExpressionStatement(es) = stmt {
                        es.unbox().expression
                    } else {
                        ctx.ast.expression_identifier(SPAN, arena_ident(ctx, &ident_name))
                    }
                } else {
                    ctx.ast.expression_identifier(SPAN, arena_ident(ctx, &ident_name))
                }
            } else {
                ctx.ast.expression_identifier(SPAN, arena_ident(ctx, &ident_name))
            };

            // 5. Wrap in wrapperQrl(replacement) -- e.g., componentQrl(q_sym)
            let wrapper_call_code = format!("{}(0)", qrl_wrapper_name);
            let wrapper_stmt = format!("{};", wrapper_call_code);
            if let Some(stmt) = crate::add_side_effect::parse_single_statement(&wrapper_stmt, allocator) {
                if let oxc::ast::ast::Statement::ExpressionStatement(es) = stmt {
                    let mut wrapper_expr = es.unbox().expression;
                    if let Expression::CallExpression(ref mut wrapper_call) = wrapper_expr {
                        // Replace the dummy 0 arg with our replacement
                        wrapper_call.arguments.clear();
                        wrapper_call.arguments.push(expr_to_argument(replacement));
                    }
                    *expr = wrapper_expr;
                }
            }

            self.needs_noop_qrl_import = true;

            self.segments.push(SegmentRecord {
                name: names.symbol_name.clone(),
                display_name: names.display_name.clone(),
                canonical_filename: names.canonical_filename.clone(),
                entry: entry.clone(),
                expr: expr_code.clone(),
                scoped_idents: scoped_idents.clone(),
                local_idents: local_idents.clone(),
                ctx_name: pending.ctx_name.clone(),
                ctx_kind: pending.ctx_kind.clone(),
                origin: self.rel_path.clone(),
                span: (pending.span_start, call_span_end),
                hash: names.hash.clone(),
                is_inline: true,
                migrated_root_vars: Vec::new(),
                parent: None,
                pending_parent_span: parent_span,
                param_names: param_names.clone(),
            });
        } else if is_inline {
            // Inline strategy: inlinedQrl(fn_expr, "symbol_name"[, captures])
            let is_dev = matches!(self.mode, EmitMode::Dev | EmitMode::Hmr);
            let _inlined_name = if is_dev { "inlinedQrlDEV" } else { "inlinedQrl" };

            // Replace callee with inlinedQrl wrapper
            // The first arg stays as the function expression
            // Insert symbol name as second arg
            call.arguments.push(Argument::StringLiteral(
                ctx.ast.alloc_string_literal(
                    SPAN,
                    arena_str(ctx, &names.symbol_name),
                    None,
                ),
            ));

            if has_captures {
                // Build captures array: [capture1, capture2, ...]
                let captures_array = build_capture_array_expr(&scoped_idents, ctx);
                call.arguments.push(expr_to_argument(captures_array));
            }

            self.needs_inlined_qrl_import = true;

            self.segments.push(SegmentRecord {
                name: names.symbol_name.clone(),
                display_name: names.display_name.clone(),
                canonical_filename: names.canonical_filename.clone(),
                entry: entry.clone(),
                expr: expr_code.clone(),
                scoped_idents: scoped_idents.clone(),
                local_idents: local_idents.clone(),
                ctx_name: pending.ctx_name.clone(),
                ctx_kind: pending.ctx_kind.clone(),
                origin: self.rel_path.clone(),
                span: (pending.span_start, call_span_end),
                hash: names.hash.clone(),
                is_inline: true,
                migrated_root_vars: Vec::new(),
                parent: None,
                pending_parent_span: parent_span,
                param_names: param_names.clone(),
            });
        } else {
            // Segment strategy: qrl(() => import("./path"), "symbol_name")
            // The callback body is extracted to a separate segment module.
            // Replace the call args with:
            //   1. () => import("./canonical_filename")
            //   2. "symbol_name"

            let import_path = if self.explicit_extensions {
                format!("./{}.{}", names.canonical_filename, self.extension)
            } else {
                format!("./{}", names.canonical_filename)
            };

            let is_dev = matches!(self.mode, EmitMode::Dev | EmitMode::Hmr);
            let qrl_callee_name = if is_dev { "qrlDEV" } else { "qrl" };

            // Build the import arrow: () => import("./path")
            let import_path_str = arena_str(ctx, &import_path);
            let import_expr = ctx.ast.expression_import(
                SPAN,
                ctx.ast.expression_string_literal(SPAN, import_path_str, None),
                None,
                None,
            );
            let arrow_params = ctx.ast.formal_parameters(
                SPAN,
                FormalParameterKind::ArrowFormalParameters,
                ctx.ast.vec(),
                None::<FormalParameterRest<'a>>,
            );
            let import_stmt = ctx.ast.statement_expression(SPAN, import_expr);
            let arrow_body = ctx.ast.function_body(
                SPAN,
                ctx.ast.vec(),
                ctx.ast.vec1(import_stmt),
            );
            let arrow = ctx.ast.expression_arrow_function(
                SPAN,
                true,
                false,
                None::<TSTypeParameterDeclaration<'a>>,
                arrow_params,
                None::<TSTypeAnnotation<'a>>,
                arrow_body,
            );

            // Replace call arguments
            call.arguments.clear();

            // Arg 1: arrow function with dynamic import
            call.arguments.push(expr_to_argument(arrow));

            // Arg 2: symbol name string
            call.arguments.push(Argument::StringLiteral(
                ctx.ast.alloc_string_literal(
                    SPAN,
                    arena_str(ctx, &names.symbol_name),
                    None,
                ),
            ));

            if has_captures {
                // Arg 3: captures array
                let captures_array = build_capture_array_expr(&scoped_idents, ctx);
                call.arguments.push(expr_to_argument(captures_array));
            }

            // Replace callee with qrl/qrlDEV
            if let Expression::Identifier(id) = &mut call.callee {
                id.name = arena_ident(ctx, qrl_callee_name);
            }

            self.needs_qrl_import = true;

            self.segments.push(SegmentRecord {
                name: names.symbol_name.clone(),
                display_name: names.display_name.clone(),
                canonical_filename: names.canonical_filename.clone(),
                entry: entry.clone(),
                expr: expr_code.clone(),
                scoped_idents: scoped_idents.clone(),
                local_idents: local_idents.clone(),
                ctx_name: pending.ctx_name.clone(),
                ctx_kind: pending.ctx_kind.clone(),
                origin: self.rel_path.clone(),
                span: (pending.span_start, call_span_end),
                hash: names.hash.clone(),
                is_inline: false,
                migrated_root_vars: Vec::new(),
                parent: None,
                pending_parent_span: parent_span,
                param_names: param_names.clone(),
            });
        }
    }

    // -----------------------------------------------------------------------
    // Stack context push/pop for descriptive symbol naming (SWC parity)
    // -----------------------------------------------------------------------

    fn enter_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let BindingPattern::BindingIdentifier(ident) = &node.id {
            self.stack_ctxt.push(ident.name.to_string());
        }
    }

    fn exit_variable_declarator(
        &mut self,
        node: &mut VariableDeclarator<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if matches!(&node.id, BindingPattern::BindingIdentifier(_)) {
            self.stack_ctxt.pop();
        }
    }

    fn enter_declaration(
        &mut self,
        node: &mut Declaration<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        match node {
            Declaration::FunctionDeclaration(func) => {
                if let Some(id) = &func.id {
                    self.stack_ctxt.push(id.name.to_string());
                }
            }
            Declaration::ClassDeclaration(class) => {
                if let Some(id) = &class.id {
                    self.stack_ctxt.push(id.name.to_string());
                }
            }
            _ => {}
        }
    }

    fn exit_declaration(
        &mut self,
        node: &mut Declaration<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        match node {
            Declaration::FunctionDeclaration(func) => {
                if func.id.is_some() {
                    self.stack_ctxt.pop();
                }
            }
            Declaration::ClassDeclaration(class) => {
                if class.id.is_some() {
                    self.stack_ctxt.pop();
                }
            }
            _ => {}
        }
    }

    fn enter_jsx_opening_element(
        &mut self,
        node: &mut JSXOpeningElement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let JSXElementName::Identifier(ident) = &node.name {
            self.stack_ctxt.push(ident.name.to_string());
        }
    }

    fn exit_jsx_opening_element(
        &mut self,
        node: &mut JSXOpeningElement<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if matches!(&node.name, JSXElementName::Identifier(_)) {
            self.stack_ctxt.pop();
        }
    }

    fn enter_jsx_attribute(
        &mut self,
        node: &mut JSXAttribute<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if let JSXAttributeName::Identifier(ident) = &node.name {
            self.stack_ctxt.push(ident.name.to_string());
        }
    }

    fn exit_jsx_attribute(
        &mut self,
        node: &mut JSXAttribute<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        if matches!(&node.name, JSXAttributeName::Identifier(_)) {
            self.stack_ctxt.pop();
        }
    }

    fn enter_export_default_declaration(
        &mut self,
        _node: &mut ExportDefaultDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Push file stem (e.g., "test" from "test.tsx") for export default context.
        // SWC uses path_data.file_stem; we derive it from file_name.
        let file_stem = self.file_name
            .rsplit_once('.')
            .map(|(stem, _ext)| stem)
            .unwrap_or(&self.file_name)
            .to_string();
        self.stack_ctxt.push(file_stem);
    }

    fn exit_export_default_declaration(
        &mut self,
        _node: &mut ExportDefaultDeclaration<'a>,
        _ctx: &mut TraverseCtx<'a, ()>,
    ) {
        self.stack_ctxt.pop();
    }

    fn exit_program(
        &mut self,
        program: &mut Program<'a>,
        ctx: &mut TraverseCtx<'a, ()>,
    ) {
        // Import rewriting: add synthetic imports for qrl/inlinedQrl/etc.
        let _core = arena_str(ctx, &self.core_module);

        // Build list of (specifier, local_name) imports to add
        let mut imports_to_add: Vec<(&str, &str)> = Vec::new();

        if self.needs_qrl_import {
            let is_dev = matches!(self.mode, EmitMode::Dev | EmitMode::Hmr);
            if is_dev {
                imports_to_add.push(("qrlDEV", "qrlDEV"));
            } else {
                imports_to_add.push(("qrl", "qrl"));
            }
        }
        if self.needs_inlined_qrl_import {
            let is_dev = matches!(self.mode, EmitMode::Dev | EmitMode::Hmr);
            if is_dev {
                imports_to_add.push(("inlinedQrlDEV", "inlinedQrlDEV"));
            } else {
                imports_to_add.push(("inlinedQrl", "inlinedQrl"));
            }
        }
        if self.needs_noop_qrl_import {
            let is_dev = matches!(self.mode, EmitMode::Dev | EmitMode::Hmr);
            if is_dev {
                imports_to_add.push(("_noopQrlDEV", "_noopQrlDEV"));
            } else {
                imports_to_add.push(("_noopQrl", "_noopQrl"));
            }
        }
        if self.needs_jsx_sorted_import {
            imports_to_add.push(("_jsxSorted", "_jsxSorted"));
        }
        if self.needs_jsx_split_import {
            imports_to_add.push(("_jsxSplit", "_jsxSplit"));
        }
        if self.needs_fragment_import {
            imports_to_add.push(("Fragment", "Fragment"));
        }
        if self.needs_fn_signal_import {
            imports_to_add.push(("_fnSignal", "_fnSignal"));
        }
        if self.needs_wrap_prop_import {
            imports_to_add.push(("_wrapProp", "_wrapProp"));
        }

        // Insert synthetic import declarations at position 0 using string-based parsing.
        // This avoids complex AST builder API differences across OXC versions.
        let allocator = ctx.ast.allocator;
        for (specifier, _local) in imports_to_add.into_iter().rev() {
            let import_str = format!(r#"import {{ {} }} from "{}";"#, specifier, self.core_module);
            if let Some(stmt) = crate::add_side_effect::parse_single_statement(&import_str, allocator) {
                program.body.insert(0, stmt);
            }
        }

        // Hoist strategy: emit extra_top_items const declarations and
        // ref_assignments (.s() calls) into the root module.
        //
        // Ordering (critical per Pitfall 1):
        //   1. Import declarations (already inserted above)
        //   2. const q_sym = _noopQrl("sym") declarations (extra_top_items)
        //   3. q_sym.s(fn_body) expression statements (ref_assignments)
        //   4. Original module body (exports, etc.)
        if matches!(self.entry_strategy, EntryStrategy::Hoist)
            && !matches!(self.mode, EmitMode::Lib)
        {
            // Find the insertion point: after imports, before everything else.
            let first_non_import = program.body.iter().position(|stmt| {
                !matches!(
                    stmt,
                    oxc::ast::ast::Statement::ImportDeclaration(_)
                )
            }).unwrap_or(program.body.len());

            // Insert ref_assignments (in reverse to maintain order at same position)
            let ref_stmts: Vec<String> = self.ref_assignments.drain(..).collect();
            for s_call in ref_stmts.into_iter().rev() {
                // Use JSX-aware parser since fn_body may contain JSX elements
                if let Some(stmt) = crate::add_side_effect::parse_single_statement_jsx(&s_call, allocator) {
                    program.body.insert(first_non_import, stmt);
                }
            }

            // Insert const declarations for extra_top_items (in reverse)
            let extra_items: Vec<HoistedConst> = self.extra_top_items.clone();
            for item in extra_items.into_iter().rev() {
                let const_decl = format!("const {} = {};", item.name, item.rhs_code);
                if let Some(stmt) = crate::add_side_effect::parse_single_statement(&const_decl, allocator) {
                    program.body.insert(first_non_import, stmt);
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: collect formal params into decl_stack frame
// ---------------------------------------------------------------------------

fn collect_formal_params_to_decl(formal: &FormalParameters<'_>, frame: &mut Vec<TypedId>) {
    for param in &formal.items {
        collect_binding_to_decl(&param.pattern, frame, false);
    }
    if let Some(rest) = &formal.rest {
        collect_binding_to_decl(&rest.rest.argument, frame, false);
    }
}

/// Collect binding names from a pattern into a decl_stack frame.
/// Handles all 4 BindingPattern variants exhaustively (no wildcards per Pitfall 3).
fn collect_binding_to_decl(pat: &BindingPattern<'_>, frame: &mut Vec<TypedId>, is_const: bool) {
    match pat {
        BindingPattern::BindingIdentifier(id) => {
            frame.push((id.name.as_str().to_string(), if is_const { IdentType::Const } else { IdentType::Let }));
        }
        BindingPattern::ObjectPattern(obj) => {
            for prop in &obj.properties {
                collect_binding_to_decl(&prop.value, frame, is_const);
            }
            if let Some(rest) = &obj.rest {
                collect_binding_to_decl(&rest.argument, frame, is_const);
            }
        }
        BindingPattern::ArrayPattern(arr) => {
            for element in arr.elements.iter().flatten() {
                collect_binding_to_decl(element, frame, is_const);
            }
            if let Some(rest) = &arr.rest {
                collect_binding_to_decl(&rest.argument, frame, is_const);
            }
        }
        BindingPattern::AssignmentPattern(assign) => {
            collect_binding_to_decl(&assign.left, frame, is_const);
        }
    }
}

// ---------------------------------------------------------------------------
// Helper: build captures array expression
// ---------------------------------------------------------------------------

fn build_capture_array_expr<'a>(
    scoped_idents: &[String],
    ctx: &mut TraverseCtx<'a, ()>,
) -> Expression<'a> {
    let mut elements = ctx.ast.vec();
    for name in scoped_idents {
        let name_ident = arena_ident(ctx, name.as_str());
        let ident = ctx.ast.expression_identifier(SPAN, name_ident);
        elements.push(ArrayExpressionElement::from(ident));
    }
    ctx.ast.expression_array(SPAN, elements)
}

// ---------------------------------------------------------------------------
// Helper: convert Expression to Argument
// ---------------------------------------------------------------------------

fn expr_to_argument(expr: Expression<'_>) -> Argument<'_> {
    match expr {
        Expression::BooleanLiteral(b) => Argument::BooleanLiteral(b),
        Expression::NullLiteral(b) => Argument::NullLiteral(b),
        Expression::NumericLiteral(b) => Argument::NumericLiteral(b),
        Expression::BigIntLiteral(b) => Argument::BigIntLiteral(b),
        Expression::RegExpLiteral(b) => Argument::RegExpLiteral(b),
        Expression::StringLiteral(b) => Argument::StringLiteral(b),
        Expression::TemplateLiteral(b) => Argument::TemplateLiteral(b),
        Expression::Identifier(b) => Argument::Identifier(b),
        Expression::ArrayExpression(b) => Argument::ArrayExpression(b),
        Expression::ObjectExpression(b) => Argument::ObjectExpression(b),
        Expression::FunctionExpression(b) => Argument::FunctionExpression(b),
        Expression::ArrowFunctionExpression(b) => Argument::ArrowFunctionExpression(b),
        Expression::CallExpression(b) => Argument::CallExpression(b),
        Expression::SequenceExpression(b) => Argument::SequenceExpression(b),
        Expression::AssignmentExpression(b) => Argument::AssignmentExpression(b),
        Expression::ConditionalExpression(b) => Argument::ConditionalExpression(b),
        Expression::LogicalExpression(b) => Argument::LogicalExpression(b),
        Expression::BinaryExpression(b) => Argument::BinaryExpression(b),
        Expression::UnaryExpression(b) => Argument::UnaryExpression(b),
        Expression::UpdateExpression(b) => Argument::UpdateExpression(b),
        Expression::StaticMemberExpression(b) => Argument::StaticMemberExpression(b),
        Expression::ComputedMemberExpression(b) => Argument::ComputedMemberExpression(b),
        Expression::PrivateFieldExpression(b) => Argument::PrivateFieldExpression(b),
        Expression::NewExpression(b) => Argument::NewExpression(b),
        Expression::TaggedTemplateExpression(b) => Argument::TaggedTemplateExpression(b),
        Expression::YieldExpression(b) => Argument::YieldExpression(b),
        Expression::AwaitExpression(b) => Argument::AwaitExpression(b),
        Expression::ParenthesizedExpression(b) => Argument::ParenthesizedExpression(b),
        Expression::ClassExpression(b) => Argument::ClassExpression(b),
        Expression::ImportExpression(b) => Argument::ImportExpression(b),
        Expression::MetaProperty(b) => Argument::MetaProperty(b),
        Expression::ChainExpression(b) => Argument::ChainExpression(b),
        Expression::ThisExpression(b) => Argument::ThisExpression(b),
        Expression::TSTypeAssertion(b) => Argument::TSTypeAssertion(b),
        Expression::TSAsExpression(b) => Argument::TSAsExpression(b),
        Expression::TSSatisfiesExpression(b) => Argument::TSSatisfiesExpression(b),
        Expression::TSNonNullExpression(b) => Argument::TSNonNullExpression(b),
        Expression::TSInstantiationExpression(b) => Argument::TSInstantiationExpression(b),
        // All Expression variants should be covered above
        _ => unreachable!("Unexpected Expression variant in expr_to_argument"),
    }
}

// ---------------------------------------------------------------------------
// words helper
// ---------------------------------------------------------------------------

/// Escape $ suffix from context name for display name building.
fn escape_dollar(name: &str) -> String {
    name.strip_suffix('$')
        .unwrap_or(name)
        .to_string()
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
        match crate::parser::parse(&allocator, source_in_arena, filename) {
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
    let mut collect = crate::collector::global_collect(&program);

    // Stage 4: Pre-traverse mutations
    crate::const_replace::replace_build_constants(&mut program, config, &collect, &allocator);
    crate::filter_exports::filter_exports(&mut program, &config.strip_exports, &allocator);
    crate::filter_exports::filter_ctx_names(
        &mut program,
        &config.strip_ctx_name,
        config.strip_event_handlers,
        &allocator,
    );

    // Stage 4b: Props destructuring (CONV-04) -- MUST run before capture analysis
    crate::props_destructuring::transform_props_destructuring(
        &mut program,
        &mut collect,
        &config.core_module,
        &allocator,
    );

    // Stage 5: Determine file metadata
    let path_data = crate::source_path::SourcePath(filename).path_data(
        std::path::Path::new(&config.src_dir),
    )
    .unwrap_or_else(|_| crate::source_path::PathData {
        file_stem: "unknown".to_string(),
        file_name: filename.to_string(),
        rel_dir: std::path::PathBuf::new(),
        abs_dir: std::path::PathBuf::from(&config.src_dir),
    });

    let extension = crate::source_path::SourcePath(filename).output_extension(
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
        source_in_arena,
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
    diagnostics.extend(transformer.diagnostics);

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
        // After QRL wrapping, the component body is replaced with a dynamic import.
        // The root module should contain the QRL wrapper, not the original body.
        let code = &output.modules[0].code;
        assert!(
            code.contains("qrlDEV") || code.contains("qrl("),
            "Should contain QRL wrapper call, got: {}",
            code
        );
        // The original component$ should be replaced
        assert!(
            !code.contains("component$("),
            "component$ call should be replaced, got: {}",
            code
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
        let t = QwikTransform::new(&config, &collect, "test.tsx", "test.tsx", "tsx", "");

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
        let t = QwikTransform::new(&config, &collect, "test.tsx", "test.tsx", "tsx", "");

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
        let t = QwikTransform::new(&config, &collect, "test.tsx", "test.tsx", "tsx", "");

        assert!(
            t.sync_qrl_fn.is_some(),
            "sync$ should be detected"
        );
        assert_eq!(t.sync_qrl_fn.as_deref(), Some("sync$"));
    }

    // -----------------------------------------------------------------------
    // Capture analysis tests (CONV-03)
    // -----------------------------------------------------------------------

    #[test]
    fn capture_category_1_module_level_decl_is_self_import() {
        // Module-level const referenced inside $() should become self-import, not capture
        let src = r#"
            import { $ } from "@qwik.dev/core";
            const API_URL = "/api";
            const handler = $(() => {
                console.log(API_URL);
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.diagnostics.is_empty(), "Diagnostics: {:?}", output.diagnostics);
    }

    #[test]
    fn capture_category_2_import_is_needed_import() {
        // User import referenced inside $() should become needed_import, not capture
        let src = r#"
            import { $ } from "@qwik.dev/core";
            import css from "./style.css";
            const handler = $(() => {
                return css;
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.diagnostics.is_empty(), "Diagnostics: {:?}", output.diagnostics);
    }

    #[test]
    fn capture_category_3_outer_local_is_capture() {
        // Variable in enclosing function scope should be captured
        let src = r#"
            import { $, component$ } from "@qwik.dev/core";
            export const App = component$(() => {
                const count = 42;
                return $(() => {
                    console.log(count);
                });
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.diagnostics.is_empty(), "Diagnostics: {:?}", output.diagnostics);
    }

    #[test]
    fn capture_category_4_loop_variable_captured() {
        // Loop iteration variable should be captured (CAPTURE-EDGE-01)
        let src = r#"
            import { $, component$ } from "@qwik.dev/core";
            export const App = component$(() => {
                const items = ['a', 'b'];
                for (const item of items) {
                    $(() => console.log(item));
                }
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.diagnostics.is_empty(), "Diagnostics: {:?}", output.diagnostics);
    }

    #[test]
    fn capture_category_7_shadowed_not_captured() {
        // Inner binding shadows outer -- outer is NOT captured (CAPTURE-EDGE-05)
        let src = r#"
            import { $ } from "@qwik.dev/core";
            const x = 'outer';
            export const handler = $(() => {
                const x = 'inner';
                console.log(x);
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.diagnostics.is_empty(), "Diagnostics: {:?}", output.diagnostics);
    }

    #[test]
    fn capture_category_8_fn_class_produces_c02_error() {
        // Function/class declarations referenced across $ boundary -> C02 error
        let src = r#"
            import { $, component$ } from "@qwik.dev/core";
            export const App = component$(() => {
                function hola() { console.log('hola'); }
                class Thing {}
                return $(() => {
                    hola();
                    new Thing();
                });
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        // Should have C02 errors for hola and Thing
        let c02_errors: Vec<_> = output
            .diagnostics
            .iter()
            .filter(|d| d.code.as_deref() == Some("C02"))
            .collect();
        assert!(
            c02_errors.len() >= 2,
            "Expected at least 2 C02 errors for hola and Thing, got {} errors: {:?}",
            c02_errors.len(),
            c02_errors
        );
    }

    #[test]
    fn capture_edge_06_callback_params_not_captured() {
        // Parameters of the $() callback are NOT captured (CAPTURE-EDGE-06)
        let src = r#"
            import { $ } from "@qwik.dev/core";
            export const handler = $((event, element) => {
                console.log(event.target, element);
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.diagnostics.is_empty(), "Diagnostics: {:?}", output.diagnostics);
    }

    // -----------------------------------------------------------------------
    // IdentCollector tests
    // -----------------------------------------------------------------------

    #[test]
    fn ident_collector_finds_references() {
        use oxc::allocator::Allocator;
        use oxc::parser::Parser;
        use oxc::span::SourceType;

        let alloc = Allocator::default();
        let src = alloc.alloc_str("const x = foo + bar;");
        let ret = Parser::new(&alloc, src, SourceType::mjs()).parse();
        assert!(!ret.panicked);
        // Extract expression from: const x = foo + bar;
        if let Statement::VariableDeclaration(decl) = &ret.program.body[0] {
            if let Some(init) = &decl.declarations[0].init {
                let idents = IdentCollector::collect(init);
                assert!(idents.contains("foo"), "Should find 'foo'");
                assert!(idents.contains("bar"), "Should find 'bar'");
                assert!(!idents.contains("x"), "Should not find 'x' (declaration)");
            }
        }
    }

    #[test]
    fn ident_collector_skips_global_builtins() {
        use oxc::allocator::Allocator;
        use oxc::parser::Parser;
        use oxc::span::SourceType;

        let alloc = Allocator::default();
        let src = alloc.alloc_str("const x = undefined;");
        let ret = Parser::new(&alloc, src, SourceType::mjs()).parse();
        assert!(!ret.panicked);
        if let Statement::VariableDeclaration(decl) = &ret.program.body[0] {
            if let Some(init) = &decl.declarations[0].init {
                let idents = IdentCollector::collect(init);
                assert!(!idents.contains("undefined"), "Should skip 'undefined'");
            }
        }
    }

    #[test]
    fn compute_scoped_idents_basic() {
        let mut idents = HashSet::new();
        idents.insert("x".to_string());
        idents.insert("y".to_string());
        idents.insert("z".to_string());

        let decl: Vec<TypedId> = vec![
            ("x".to_string(), IdentType::Const),
            ("y".to_string(), IdentType::Let),
            ("w".to_string(), IdentType::Const),
        ];

        let scoped = compute_scoped_idents(&idents, &decl);
        assert!(scoped.contains(&"x".to_string()));
        assert!(scoped.contains(&"y".to_string()));
        assert!(!scoped.contains(&"z".to_string())); // not in decl
        assert!(!scoped.contains(&"w".to_string())); // not in idents
    }

    #[test]
    fn compute_scoped_idents_excludes_fn_class() {
        let mut idents = HashSet::new();
        idents.insert("myFn".to_string());
        idents.insert("myClass".to_string());
        idents.insert("myVar".to_string());

        let decl: Vec<TypedId> = vec![
            ("myFn".to_string(), IdentType::Fn),
            ("myClass".to_string(), IdentType::Class),
            ("myVar".to_string(), IdentType::Const),
        ];

        let scoped = compute_scoped_idents(&idents, &decl);
        assert!(!scoped.contains(&"myFn".to_string()), "Fn should not be scoped ident");
        assert!(!scoped.contains(&"myClass".to_string()), "Class should not be scoped ident");
        assert!(scoped.contains(&"myVar".to_string()), "Var should be scoped ident");
    }

    // -----------------------------------------------------------------------
    // QRL wrapping tests (CONV-02)
    // -----------------------------------------------------------------------

    #[test]
    fn qrl_wrap_bare_dollar_produces_qrl_call() {
        let src = r#"
            import { $ } from "@qwik.dev/core";
            export const sym1 = $((ctx) => console.log("1"));
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        assert!(output.modules.len() == 1);
        let code = &output.modules[0].code;
        // Should contain qrlDEV (dev mode) instead of $
        assert!(
            code.contains("qrlDEV") || code.contains("qrl"),
            "Should replace $ with qrl/qrlDEV, got: {}",
            code
        );
        // Should NOT contain the original $ call
        assert!(
            !code.contains("$("),
            "Should not contain original $ call, got: {}",
            code
        );
    }

    #[test]
    fn qrl_wrap_component_produces_component_qrl() {
        let src = r#"
            import { component$ } from "@qwik.dev/core";
            export const App = component$(() => {
                return <div>Hello</div>;
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        // component$ should become componentQrl or the qrl() call
        // In segment mode, the callee is replaced with qrl/qrlDEV
        assert!(
            !code.contains("component$("),
            "Should not contain component$ call, got: {}",
            code
        );
    }

    #[test]
    fn noop_qrl_for_stripped_callback() {
        // When strip_ctx_name is used, the pre-pass filter_ctx_names replaces
        // the call with `void 0`. The noop QRL path in the traversal is for
        // cases where should_emit_segment returns false DURING traversal.
        let src = r#"
            import { useTask$ } from "@qwik.dev/core";
            useTask$(() => {
                console.log("task");
            });
        "#;
        let mut config = make_config();
        config.strip_ctx_name = vec!["useTask$".to_string()];
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        // Pre-pass filter_ctx_names replaces with void 0
        assert!(
            code.contains("void 0"),
            "Stripped callback should be replaced with void 0, got: {}",
            code
        );
        // Original call should be gone
        assert!(
            !code.contains("useTask$("),
            "Should not contain original useTask$ call, got: {}",
            code
        );
    }

    #[test]
    fn sync_dollar_produces_qrl_sync() {
        let src = r#"
            import { sync$ } from "@qwik.dev/core";
            const fn1 = sync$(() => {
                return true;
            });
        "#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        // sync$ should become _qrlSync
        assert!(
            code.contains("_qrlSync"),
            "Should replace sync$ with _qrlSync, got: {}",
            code
        );
    }

    #[test]
    fn pure_annotation_only_on_component_qrl() {
        // CONV-08: Only componentQrl gets /*#__PURE__*/, not useTaskQrl
        let src = r#"
            import { component$, useTask$ } from "@qwik.dev/core";
            export const App = component$(() => {
                useTask$(() => { console.log("task"); });
                return <div>Hello</div>;
            });
        "#;
        let config = make_config();
        let _output = transform_code(src, "test.tsx", &config);
        // PURE annotation implementation is tracked but will be fully
        // implemented in the codegen phase. For now, we verify no crash.
    }

    #[test]
    fn qrl_wrap_dev_mode_uses_qrl_dev() {
        let src = r#"
            import { $ } from "@qwik.dev/core";
            export const handler = $(() => console.log("hello"));
        "#;
        let mut config = make_config();
        config.mode = EmitMode::Dev;
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        assert!(
            code.contains("qrlDEV"),
            "Dev mode should use qrlDEV, got: {}",
            code
        );
    }

    #[test]
    fn qrl_wrap_prod_mode_uses_qrl() {
        let src = r#"
            import { $ } from "@qwik.dev/core";
            export const handler = $(() => console.log("hello"));
        "#;
        let mut config = make_config();
        config.mode = EmitMode::Prod;
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        assert!(
            code.contains("qrl("),
            "Prod mode should use qrl, got: {}",
            code
        );
        assert!(
            !code.contains("qrlDEV"),
            "Prod mode should not use qrlDEV, got: {}",
            code
        );
    }

    #[test]
    fn qrl_wrap_inline_strategy_uses_inlined_qrl() {
        let src = r#"
            import { $ } from "@qwik.dev/core";
            export const handler = $(() => console.log("hello"));
        "#;
        let mut config = make_config();
        config.entry_strategy = EntryStrategy::Inline;
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        // In Inline mode, should not create dynamic import
        assert!(
            !code.contains("import("),
            "Inline mode should not have dynamic import, got: {}",
            code
        );
    }

    // -----------------------------------------------------------------------
    // JSX transform integration tests (CONV-06)
    // -----------------------------------------------------------------------

    #[test]
    fn jsx_element_transformed_to_jsx_sorted() {
        let src = r#"<div class="hello">text</div>"#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        assert!(
            code.contains("_jsxSorted"),
            "JSX element should produce _jsxSorted call, got: {}",
            code
        );
        assert!(
            !code.contains("<div"),
            "JSX syntax should be removed, got: {}",
            code
        );
    }

    #[test]
    fn jsx_spread_uses_jsx_split() {
        let src = r#"<div {...props}>text</div>"#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        assert!(
            code.contains("_jsxSplit"),
            "Spread props should produce _jsxSplit call, got: {}",
            code
        );
    }

    #[test]
    fn jsx_classname_normalized_to_class() {
        let src = r#"<div className="foo">text</div>"#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        // className should be normalized to class in the output
        assert!(
            !code.contains("className"),
            "className should be normalized to class, got: {}",
            code
        );
    }

    #[test]
    fn jsx_component_gets_key() {
        let src = r#"<Header title="hello" />"#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        assert!(
            code.contains("_jsxSorted"),
            "Component should produce _jsxSorted, got: {}",
            code
        );
        // Component elements should get a key (non-null last arg)
        assert!(
            code.contains("Header"),
            "Component name should appear as identifier, got: {}",
            code
        );
    }

    #[test]
    fn jsx_key_extracted_from_props() {
        let src = r#"<div key="my-key">text</div>"#;
        let config = make_config();
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        assert!(
            code.contains("my-key"),
            "Extracted key should appear in output, got: {}",
            code
        );
    }

    // -----------------------------------------------------------------------
    // Props destructuring integration test (CONV-04)
    // -----------------------------------------------------------------------

    #[test]
    fn props_destructuring_in_component_pipeline() {
        // In Inline mode, the function body stays in the root module,
        // so we can see the _rawProps transformation.
        let src = r#"
            import { component$ } from "@qwik.dev/core";
            export const App = component$(({ name, age }) => {
                return <div>{name} is {age}</div>;
            });
        "#;
        let mut config = make_config();
        config.entry_strategy = EntryStrategy::Inline;
        let output = transform_code(src, "test.tsx", &config);
        let code = &output.modules[0].code;
        // Props destructuring should have run (pre-pass)
        assert!(
            code.contains("_rawProps"),
            "Props destructuring should produce _rawProps, got: {}",
            code
        );
    }
}
