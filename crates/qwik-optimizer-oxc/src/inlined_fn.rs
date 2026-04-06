//! Inlined function signal wrapping utilities -- CONV-07.
//!
//! Implements `convert_inlined_fn` with 6 eligibility checks, `ObjectUsageChecker`
//! (read-only `Visit`), and `ReplaceIdentifiers` (string-based replacement) for
//! building `_fnSignal((p0, p1, ...) => expr, [caps])` calls.
//!
//! Called by the JSX transform after the `_wrapProp` fast path has been tried
//! and did not match.

use std::collections::HashSet;

use oxc::allocator::Allocator;
use oxc::ast::ast::*;
use oxc::ast_visit::Visit;
use oxc::codegen::{Codegen, CodegenOptions, IndentChar};
use oxc::span::{SourceType, SPAN};

// ---------------------------------------------------------------------------
// ObjectUsageChecker -- read-only visitor
// ---------------------------------------------------------------------------

/// Traverses an expression to detect whether any captured identifier appears as
/// the object of a member expression or in a logical-OR expression, and whether
/// any call expression (used_as_call) exists.
pub(crate) struct ObjectUsageChecker<'a> {
    pub used_as_call: bool,
    pub used_as_object: bool,
    scoped_idents: &'a HashSet<String>,
}

impl<'a> ObjectUsageChecker<'a> {
    pub(crate) fn new(scoped_idents: &'a HashSet<String>) -> Self {
        Self {
            used_as_call: false,
            used_as_object: false,
            scoped_idents,
        }
    }

    fn is_captured_ident(expr: &Expression<'_>, scoped_idents: &HashSet<String>) -> bool {
        if let Expression::Identifier(id) = expr {
            scoped_idents.contains(id.name.as_str())
        } else {
            false
        }
    }
}

impl<'a, 'b> Visit<'a> for ObjectUsageChecker<'b> {
    fn visit_call_expression(&mut self, expr: &CallExpression<'a>) {
        self.used_as_call = true;
        oxc::ast_visit::walk::walk_call_expression(self, expr);
    }

    fn visit_static_member_expression(&mut self, expr: &StaticMemberExpression<'a>) {
        if Self::is_captured_ident(&expr.object, self.scoped_idents) {
            self.used_as_object = true;
        }
        oxc::ast_visit::walk::walk_static_member_expression(self, expr);
    }

    fn visit_logical_expression(&mut self, expr: &LogicalExpression<'a>) {
        if expr.operator == LogicalOperator::Or {
            if Self::is_captured_ident(&expr.left, self.scoped_idents)
                || Self::is_captured_ident(&expr.right, self.scoped_idents)
            {
                self.used_as_object = true;
            }
        }
        oxc::ast_visit::walk::walk_logical_expression(self, expr);
    }
}

// ---------------------------------------------------------------------------
// ReplaceIdentifiers -- string-based replacement + abort detection
// ---------------------------------------------------------------------------

/// Serializes `expr` via OXC Codegen, then performs string replacement of captured
/// identifier names with `p{N}`, and sets `abort = true` if the expression contains
/// constructs that cannot be safely re-evaluated.
pub(crate) struct ReplaceIdentifiers {
    /// The serialized expression body string with captured idents replaced by `p{N}`.
    pub rendered: String,
    /// Whether the expression contains any construct that prevents wrapping.
    pub abort: bool,
}

impl ReplaceIdentifiers {
    /// Serialize `expr`, detect abort conditions, then substitute captured idents.
    pub(crate) fn run<'a>(
        expr: &Expression<'a>,
        scoped_idents: &[(String, bool)],
        allocator: &'a Allocator,
    ) -> Self {
        let serialized = serialize_expression_inner(expr, allocator);

        let abort = serialized.contains("=>")
            || serialized.contains("function")
            || serialized.contains("class")
            || serialized.contains('@');

        let mut rendered = serialized;
        for (idx, (name, _is_const)) in scoped_idents.iter().enumerate() {
            rendered = replace_word(&rendered, name, &format!("p{idx}"));
        }

        Self { rendered, abort }
    }
}

/// Replace whole-word occurrences of `from` with `to` in `s`.
fn replace_word(s: &str, from: &str, to: &str) -> String {
    let mut result = String::with_capacity(s.len());
    let mut remaining = s;
    while let Some(pos) = remaining.find(from) {
        let before_ok = pos == 0 || {
            let b = remaining.as_bytes()[pos - 1];
            !b.is_ascii_alphanumeric() && b != b'_'
        };
        let after_pos = pos + from.len();
        let after_ok = after_pos >= remaining.len() || {
            let b = remaining.as_bytes()[after_pos];
            !b.is_ascii_alphanumeric() && b != b'_'
        };

        if before_ok && after_ok {
            result.push_str(&remaining[..pos]);
            result.push_str(to);
            remaining = &remaining[after_pos..];
        } else {
            result.push_str(&remaining[..pos + from.len()]);
            remaining = &remaining[after_pos..];
        }
    }
    result.push_str(remaining);
    result
}

// ---------------------------------------------------------------------------
// serialize_expression_inner
// ---------------------------------------------------------------------------

fn serialize_expression_inner<'a>(expr: &Expression<'a>, allocator: &'a Allocator) -> String {
    use oxc::allocator::{CloneIn, Vec as ArenaVec};
    use oxc::ast::AstBuilder;

    let ast = AstBuilder::new(allocator);
    let cloned: Expression<'a> = expr.clone_in(allocator);
    let binding = ast.binding_pattern_binding_identifier(SPAN, "_x");
    let mut declarators: ArenaVec<VariableDeclarator<'_>> = ArenaVec::new_in(allocator);
    declarators.push(ast.variable_declarator(
        SPAN,
        VariableDeclarationKind::Const,
        binding,
        None::<TSTypeAnnotation<'_>>,
        Some(cloned),
        false,
    ));
    let var_decl = ast.alloc_variable_declaration(
        SPAN,
        VariableDeclarationKind::Const,
        declarators,
        false,
    );
    let mut body: ArenaVec<Statement<'_>> = ArenaVec::new_in(allocator);
    body.push(Statement::VariableDeclaration(var_decl));
    let directives: ArenaVec<Directive<'_>> = ArenaVec::new_in(allocator);
    let comments: ArenaVec<Comment> = ArenaVec::new_in(allocator);
    let prog = ast.program(SPAN, SourceType::tsx(), "", comments, None, directives, body);
    let codegen_options = CodegenOptions {
        indent_char: IndentChar::Space,
        indent_width: 4,
        ..Default::default()
    };
    let raw = Codegen::new().with_options(codegen_options).build(&prog).code;
    raw.trim_start_matches("const _x = ")
        .trim_end_matches(';')
        .trim()
        .to_string()
}

// ---------------------------------------------------------------------------
// convert_inlined_fn -- 6-check eligibility + _fnSignal construction
// ---------------------------------------------------------------------------

/// Try to wrap `expr` as a `_fnSignal((p0, p1, ...) => <body>, [cap0, cap1, ...])` call.
///
/// Returns `(Some(fn_signal_call_code), arrow_code, is_const)` when eligible,
/// or `(None, String::new(), is_const/false)` when any eligibility check fires.
///
/// # Eligibility checks (in order)
///
/// 1. Expression is `ArrowFunctionExpression` -> `(None, "", is_const)`
/// 2. `ObjectUsageChecker::used_as_call == true` -> `(None, "", false)`
/// 3. `!ObjectUsageChecker::used_as_object` -> `(None, "", is_const)`
/// 4. `ReplaceIdentifiers::abort == true` -> `(None, "", is_const)`
/// 5. Rendered expression length > 150 chars -> `(None, "", false)`
/// 6. `scoped_idents.is_empty()` -> `(None, "", true)`
pub(crate) fn convert_inlined_fn<'a>(
    expr: &Expression<'a>,
    scoped_idents: &[(String, bool)],
    is_const: bool,
    is_server: bool,
    allocator: &'a Allocator,
) -> (Option<String>, String, bool) {
    // Check 1: ArrowFunctionExpression -- already a QRL boundary.
    if matches!(expr, Expression::ArrowFunctionExpression(_)) {
        return (None, String::new(), is_const);
    }

    let captured_names: HashSet<String> =
        scoped_idents.iter().map(|(n, _)| n.clone()).collect();

    // Check 2 + 3: ObjectUsageChecker.
    let mut checker = ObjectUsageChecker::new(&captured_names);
    checker.visit_expression(expr);

    if checker.used_as_call {
        return (None, String::new(), false);
    }
    if !checker.used_as_object {
        return (None, String::new(), is_const);
    }

    // Check 4: ReplaceIdentifiers abort detection.
    let replaced = ReplaceIdentifiers::run(expr, scoped_idents, allocator);
    if replaced.abort {
        return (None, String::new(), is_const);
    }

    // Check 5: rendered expression length > 150.
    if replaced.rendered.len() > 150 {
        return (None, String::new(), false);
    }

    // Check 6: no captures.
    if scoped_idents.is_empty() {
        return (None, String::new(), true);
    }

    // All checks passed -- build the _fnSignal call string.
    let params: Vec<String> = (0..scoped_idents.len()).map(|i| format!("p{i}")).collect();
    let params_str = if scoped_idents.len() == 1 {
        "p0".to_string()
    } else {
        format!("({})", params.join(", "))
    };

    let arrow_code = format!("{} => {}", params_str, replaced.rendered);

    let caps: Vec<String> = scoped_idents.iter().map(|(n, _)| n.clone()).collect();
    let caps_str = format!("[{}]", caps.join(", "));

    let fn_signal_code = if is_server {
        let server_str = format!(
            "\"{}\"",
            arrow_code.replace('\\', "\\\\").replace('"', "\\\"")
        );
        format!("_fnSignal({arrow_code}, {caps_str}, {server_str})")
    } else {
        format!("_fnSignal({arrow_code}, {caps_str})")
    };

    (Some(fn_signal_code), arrow_code, false)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use oxc::allocator::Allocator;
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    fn parse_expr<'a>(allocator: &'a Allocator, src: &str) -> Expression<'a> {
        let src_arena: &str = allocator.alloc_str(src);
        let parser = Parser::new(allocator, src_arena, SourceType::tsx());
        parser.parse_expression().unwrap()
    }

    #[test]
    fn fn_signal_check1_arrow_returns_none() {
        let alloc = Allocator::default();
        let expr = parse_expr(&alloc, "() => x");
        let scoped = vec![("x".to_string(), false)];
        let (result, _arrow, _is_const) =
            convert_inlined_fn(&expr, &scoped, false, false, &alloc);
        assert!(result.is_none(), "Arrow expr should return None");
    }

    #[test]
    fn fn_signal_check2_call_returns_none() {
        let alloc = Allocator::default();
        let expr = parse_expr(&alloc, "signal.doSomething()");
        let scoped = vec![("signal".to_string(), false)];
        let (result, _arrow, _is_const) =
            convert_inlined_fn(&expr, &scoped, false, false, &alloc);
        assert!(
            result.is_none(),
            "Call expr should return None (used_as_call)"
        );
    }

    #[test]
    fn fn_signal_check3_no_object_usage_returns_none() {
        let alloc = Allocator::default();
        let expr = parse_expr(&alloc, "x + 1");
        let scoped = vec![("x".to_string(), false)];
        let (result, _arrow, _is_const) =
            convert_inlined_fn(&expr, &scoped, false, false, &alloc);
        assert!(result.is_none(), "No object usage should return None");
    }

    #[test]
    fn fn_signal_check6_no_captures_returns_none() {
        let alloc = Allocator::default();
        let expr = parse_expr(&alloc, "a.value");
        let scoped: Vec<(String, bool)> = vec![];
        let (result, _arrow, _is_const) =
            convert_inlined_fn(&expr, &scoped, false, false, &alloc);
        assert!(result.is_none(), "No captures -> None");
    }

    #[test]
    fn fn_signal_eligible_member_produces_fn_signal() {
        let alloc = Allocator::default();
        let expr = parse_expr(&alloc, "signal.color");
        let scoped = vec![("signal".to_string(), false)];
        let (result, arrow_code, _is_const) =
            convert_inlined_fn(&expr, &scoped, false, false, &alloc);
        assert!(
            result.is_some(),
            "Eligible member expr should produce _fnSignal"
        );
        let code = result.unwrap();
        assert!(code.starts_with("_fnSignal("), "Should start with _fnSignal");
        assert!(code.contains("p0.color"), "Replaced ident should appear as p0");
        assert!(
            arrow_code.contains("p0.color"),
            "arrow_code should contain p0.color"
        );
    }

    #[test]
    fn fn_signal_server_mode_adds_third_arg() {
        let alloc = Allocator::default();
        let expr = parse_expr(&alloc, "signal.color");
        let scoped = vec![("signal".to_string(), false)];
        let (result, _arrow, _is_const) =
            convert_inlined_fn(&expr, &scoped, false, true, &alloc);
        assert!(result.is_some());
        let code = result.unwrap();
        assert!(
            code.matches(',').count() >= 2,
            "Server mode should have 3 args (2 commas): {code}"
        );
    }

    #[test]
    fn fn_signal_replace_word_whole_boundary() {
        assert_eq!(replace_word("signal.value", "signal", "p0"), "p0.value");
        assert_eq!(
            replace_word("mysignal.value", "signal", "p0"),
            "mysignal.value"
        );
        assert_eq!(replace_word("x + x", "x", "p0"), "p0 + p0");
    }
}
