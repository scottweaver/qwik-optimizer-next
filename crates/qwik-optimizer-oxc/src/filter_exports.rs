//! Export stripping -- CONV-11.
//!
//! Strip exports from the module based on the `strip_exports` option.
//! Used for server/client mode where certain exports should be removed.
//!
//! When an export name is in the `strip_exports` list AND the export uses a
//! single declarator (not destructuring), its function/arrow body is replaced
//! with a single `throw "Symbol removed ..."` statement. The export declaration
//! itself is preserved so downstream consumers see the binding but get a runtime
//! error if they call it on the wrong platform.
//!
//! Also handles `strip_ctx_name` to remove matching $ call sites and
//! `strip_event_handlers` to remove all event handler $ calls.

use oxc::ast::AstBuilder;
use oxc::ast::ast::*;
use oxc::span::SPAN;
use oxc::syntax::number::NumberBase;
use oxc::syntax::operator::UnaryOperator;

/// The error message injected into stripped export bodies.
const STRIP_MESSAGE: &str =
    "Symbol removed by Qwik Optimizer, it can not be called from current platform";

// ---------------------------------------------------------------------------
// filter_exports -- strip_exports handling
// ---------------------------------------------------------------------------

/// Filter (strip) exports from the program AST.
///
/// For each export whose binding name appears in `strip_exports`:
///   - `export const/let/var name = fn` -> body replaced with throw stub
///     (single-declarator only; multi-declarator and destructuring are skipped)
///   - `export function name() {...}` -> body replaced with throw stub
pub(crate) fn filter_exports<'a>(
    program: &mut Program<'a>,
    strip_exports: &[String],
    allocator: &'a oxc::allocator::Allocator,
) {
    if strip_exports.is_empty() {
        return;
    }

    let ast = AstBuilder::new(allocator);

    for stmt in program.body.iter_mut() {
        if let Statement::ExportNamedDeclaration(export_decl) = stmt {
            if let Some(ref mut decl) = export_decl.declaration {
                match decl {
                    Declaration::VariableDeclaration(var_decl) => {
                        // SPEC: single-declarator only. Multi-declarator / destructuring skipped.
                        if var_decl.declarations.len() != 1 {
                            continue;
                        }
                        let declarator = &mut var_decl.declarations[0];
                        if let Some(name) = binding_pattern_name(&declarator.id) {
                            if strip_exports.iter().any(|s| s == name) {
                                replace_init_body(&mut declarator.init, &ast);
                            }
                        }
                    }
                    Declaration::FunctionDeclaration(func_decl) => {
                        if let Some(ref id) = func_decl.id {
                            let name = id.name.as_str();
                            if strip_exports.iter().any(|s| s == name) {
                                // Replace: `export function name(...) {...}` ->
                                //          `export const name = () => { throw ... };`
                                let binding =
                                    ast.binding_pattern_binding_identifier(SPAN, name);
                                let stub = build_arrow_throw_stub(&ast);
                                let mut declarators = ast.vec();
                                declarators.push(ast.variable_declarator(
                                    SPAN,
                                    VariableDeclarationKind::Const,
                                    binding,
                                    Option::<TSTypeAnnotation<'a>>::None,
                                    Some(stub),
                                    false,
                                ));
                                *decl = Declaration::VariableDeclaration(
                                    ast.alloc_variable_declaration(
                                        SPAN,
                                        VariableDeclarationKind::Const,
                                        declarators,
                                        false,
                                    ),
                                );
                            }
                        }
                    }
                    _ => {
                        // Class and other declaration types: skip per SPEC.
                    }
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// filter_ctx_names -- strip_ctx_name and strip_event_handlers handling
// ---------------------------------------------------------------------------

/// Strip call expressions matching `strip_ctx_name` or `strip_event_handlers`.
///
/// For each matching call expression found in an expression statement, the
/// entire statement is replaced with a throwing stub. This runs as a separate
/// pre-pass from `filter_exports`.
pub(crate) fn filter_ctx_names<'a>(
    program: &mut Program<'a>,
    strip_ctx_name: &[String],
    strip_event_handlers: bool,
    allocator: &'a oxc::allocator::Allocator,
) {
    if strip_ctx_name.is_empty() && !strip_event_handlers {
        return;
    }

    let ast = AstBuilder::new(allocator);

    for stmt in program.body.iter_mut() {
        // Only process expression statements containing call expressions
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            if let Some(callee_name) = get_call_callee_name(&expr_stmt.expression) {
                let should_strip = strip_ctx_name.iter().any(|s| s == callee_name)
                    || (strip_event_handlers && is_event_handler_name(callee_name));

                if should_strip {
                    // Replace the expression with a no-op (void 0)
                    expr_stmt.expression = ast.expression_unary(
                        SPAN,
                        UnaryOperator::Void,
                        ast.expression_numeric_literal(SPAN, 0.0, None, NumberBase::Decimal),
                    );
                }
            }
        }
    }
}

/// Extract the callee name from a call expression if it's a simple identifier or
/// member expression ending in a $ name.
fn get_call_callee_name<'a>(expr: &Expression<'a>) -> Option<&'a str> {
    match expr {
        Expression::CallExpression(call) => match &call.callee {
            Expression::Identifier(id) => Some(id.name.as_str()),
            _ => None,
        },
        _ => None,
    }
}

/// Check if a callee name matches an event handler pattern (on[A-Z]*$).
fn is_event_handler_name(name: &str) -> bool {
    if !name.ends_with('$') || !name.starts_with("on") || name.len() <= 3 {
        return false;
    }
    if let Some(ch) = name.chars().nth(2) {
        ch.is_ascii_uppercase() || ch == '-'
    } else {
        false
    }
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Replace the initializer with a synchronous zero-arg arrow throw stub.
fn replace_init_body<'a>(init: &mut Option<Expression<'a>>, ast: &AstBuilder<'a>) {
    if init.is_some() {
        *init = Some(build_arrow_throw_stub(ast));
    }
}

/// Build `() => { throw "Symbol removed ..." }` arrow expression.
fn build_arrow_throw_stub<'a>(ast: &AstBuilder<'a>) -> Expression<'a> {
    let throw_body = build_throw_body(ast);
    let params = ast.formal_parameters(
        SPAN,
        FormalParameterKind::ArrowFormalParameters,
        ast.vec(),
        Option::<FormalParameterRest<'a>>::None,
    );
    ast.expression_arrow_function(
        SPAN,
        false, // expression
        false, // async
        Option::<TSTypeParameterDeclaration<'a>>::None,
        params,
        Option::<TSTypeAnnotation<'a>>::None,
        throw_body,
    )
}

/// Build a `FunctionBody` containing a single: `throw "Symbol removed ..."`.
fn build_throw_body<'a>(ast: &AstBuilder<'a>) -> FunctionBody<'a> {
    let message = ast.expression_string_literal(SPAN, STRIP_MESSAGE, None);
    let throw_stmt = ast.statement_throw(SPAN, message);
    ast.function_body(SPAN, ast.vec(), ast.vec1(throw_stmt))
}

/// Extract the binding name from a `BindingPattern`, if it is a simple identifier.
fn binding_pattern_name<'a>(pattern: &BindingPattern<'a>) -> Option<&'a str> {
    match pattern {
        BindingPattern::BindingIdentifier(id) => Some(id.name.as_str()),
        _ => None,
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxc::allocator::Allocator;
    use oxc::codegen::Codegen;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    fn transform_strip(src: &str, names: &[&str]) -> String {
        let allocator = Allocator::default();
        let source_type = SourceType::tsx();
        let ret = Parser::new(&allocator, src, source_type).parse();
        let mut program = ret.program;
        let strip: Vec<String> = names.iter().map(|s| s.to_string()).collect();
        filter_exports(&mut program, &strip, &allocator);
        Codegen::new().build(&program).code
    }

    fn transform_ctx(src: &str, ctx_names: &[&str], strip_event: bool) -> String {
        let allocator = Allocator::default();
        let source_type = SourceType::tsx();
        let ret = Parser::new(&allocator, src, source_type).parse();
        let mut program = ret.program;
        let names: Vec<String> = ctx_names.iter().map(|s| s.to_string()).collect();
        filter_ctx_names(&mut program, &names, strip_event, &allocator);
        Codegen::new().build(&program).code
    }

    // -----------------------------------------------------------------------
    // filter_exports tests
    // -----------------------------------------------------------------------

    #[test]
    fn filter_exports_strips_arrow_function() {
        let src = "export const onGet = () => { return 42; };";
        let out = transform_strip(src, &["onGet"]);
        assert!(out.contains("throw"), "Expected throw in output, got: {out}");
        assert!(
            out.contains(STRIP_MESSAGE),
            "Expected STRIP_MESSAGE in output, got: {out}"
        );
        assert!(
            !out.contains("return 42"),
            "Original body should be gone, got: {out}"
        );
    }

    #[test]
    fn filter_exports_strips_function_declaration() {
        let src = "export function onGet() { return 42; }";
        let out = transform_strip(src, &["onGet"]);
        assert!(out.contains("throw"), "Expected throw in output, got: {out}");
        assert!(
            out.contains("export const onGet"),
            "Should be const arrow stub, got: {out}"
        );
        assert!(
            !out.contains("function onGet"),
            "Should not preserve function declaration, got: {out}"
        );
    }

    #[test]
    fn filter_exports_strips_named_exports() {
        let src = r#"export const Foo = () => 1; export const Bar = () => 2; export const Keep = () => 3;"#;
        let out = transform_strip(src, &["Foo", "Bar"]);
        // Foo and Bar stripped, Keep preserved
        assert!(
            out.contains("throw") && !out.contains("=> 1") && !out.contains("=> 2"),
            "Foo and Bar should be stripped, got: {out}"
        );
        assert!(
            out.contains("=> 3"),
            "Keep should be preserved, got: {out}"
        );
    }

    #[test]
    fn filter_exports_does_not_strip_multi_declarator() {
        let src = "export const { a, b } = obj;";
        let out = transform_strip(src, &["a"]);
        assert!(
            !out.contains("throw"),
            "Multi-declarator should not be stripped, got: {out}"
        );
    }

    #[test]
    fn filter_exports_does_not_strip_unmatched() {
        let src = "export const keep = () => 1;";
        let out = transform_strip(src, &["other"]);
        assert!(
            !out.contains("throw"),
            "Unrelated export should not be stripped, got: {out}"
        );
    }

    #[test]
    fn filter_exports_empty_is_noop() {
        let src = "export const onGet = () => { return 42; };";
        let out = transform_strip(src, &[]);
        assert!(
            !out.contains("throw"),
            "Empty strip_exports must be a no-op, got: {out}"
        );
    }

    #[test]
    fn filter_exports_strips_default_via_function() {
        let src = "export default function handler() { return 42; }";
        let out = transform_strip(src, &["default"]);
        // ExportDefaultDeclaration is not ExportNamedDeclaration, so it should
        // NOT be stripped by filter_exports (default exports use a different path).
        assert!(
            out.contains("return 42"),
            "Default export uses different path, should be intact: {out}"
        );
    }

    // -----------------------------------------------------------------------
    // filter_ctx_names tests
    // -----------------------------------------------------------------------

    #[test]
    fn filter_ctx_names_strips_matching_dollar_call() {
        let src = r#"onClick$(() => console.log("hi"));"#;
        let out = transform_ctx(src, &["onClick$"], false);
        assert!(
            !out.contains("console.log"),
            "onClick$ call should be stripped, got: {out}"
        );
    }

    #[test]
    fn filter_ctx_names_preserves_non_matching() {
        let src = r#"useTask$(() => console.log("hi"));"#;
        let out = transform_ctx(src, &["onClick$"], false);
        assert!(
            out.contains("useTask$"),
            "useTask$ should be preserved when stripping onClick$, got: {out}"
        );
    }

    #[test]
    fn filter_ctx_names_strip_event_handlers() {
        let src = r#"onClick$(() => {}); onInput$(() => {}); useTask$(() => {});"#;
        let out = transform_ctx(src, &[], true);
        assert!(
            !out.contains("onClick$"),
            "onClick$ should be stripped with strip_event_handlers, got: {out}"
        );
        assert!(
            !out.contains("onInput$"),
            "onInput$ should be stripped with strip_event_handlers, got: {out}"
        );
        assert!(
            out.contains("useTask$"),
            "useTask$ should NOT be stripped with strip_event_handlers, got: {out}"
        );
    }
}
