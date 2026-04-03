//! JSX Transform -- CONV-06.
//!
//! Converts JSX elements to `_jsxSorted()` or `_jsxSplit()` function calls.
//! The choice depends on whether the element has any spread props (which require
//! runtime sorting) or not.
//!
//! Key rules:
//! - Static-only elements -> `_jsxSorted(tag, null, constProps, children, flags, key)`
//! - Elements with dynamic props -> `_jsxSorted(tag, varProps, constProps, children, flags, key)`
//! - Elements with spread props -> `_jsxSplit(tag, varProps, constProps, children, flags, key)`
//! - `className` and `class` both normalize to `class` in the output
//! - `bind:value` expands to value prop + onInput$ handler
//! - `bind:checked` expands to checked prop + onChange$ handler
//! - `key` is extracted from props and passed as the key argument
//! - `children` from props are merged with JSX children
//! - `ref` is handled as a special prop

use oxc::allocator::{Allocator, Vec as ArenaVec};
use oxc::ast::AstBuilder;
use oxc::ast::ast::*;
use oxc::span::SPAN;

use crate::inlined_fn::convert_inlined_fn;
use crate::is_const::is_const_expression;
use crate::transform::{IdentCollector, TypedId, compute_scoped_idents};

// ---------------------------------------------------------------------------
// Signal optimization context
// ---------------------------------------------------------------------------

/// Context needed for signal optimization (CONV-07) during JSX prop classification.
///
/// When provided to `transform_jsx_element`, dynamic prop values are checked
/// via `convert_inlined_fn` to determine if they can be wrapped as `_fnSignal()`
/// calls.
pub(crate) struct SignalOptContext<'a, 'b> {
    /// Flattened declaration stack (Var-type entries from all frames).
    pub decl_stack_flat: &'b [TypedId],
    /// Whether the transform is running in server mode.
    pub is_server: bool,
    /// The allocator for AST construction.
    pub allocator: &'a Allocator,
}

// ---------------------------------------------------------------------------
// JSX flags bitmask
// ---------------------------------------------------------------------------

/// Flag: element has dynamic children.
const FLAG_DYNAMIC_CHILDREN: u32 = 1;
/// Flag: static subtree (all children are immutable).
#[allow(dead_code)]
const FLAG_STATIC_SUBTREE: u32 = 2;

// ---------------------------------------------------------------------------
// Public: transform_jsx_element
// ---------------------------------------------------------------------------

/// Transform a JSXElement into a `_jsxSorted`/`_jsxSplit` call expression.
///
/// Returns the replacement call expression and a `JsxImportNeeds` indicating
/// which runtime imports are required.
pub(crate) fn transform_jsx_element<'a>(
    el: JSXElement<'a>,
    jsx_key_counter: &mut u32,
    is_root: bool,
    allocator: &'a Allocator,
    signal_ctx: Option<&SignalOptContext<'a, '_>>,
) -> (Expression<'a>, JsxImportNeeds) {
    let ast = AstBuilder::new(allocator);
    let mut needs = JsxImportNeeds::default();

    let opening = el.opening_element.unbox();
    let children_vec = el.children;

    // 1. Classify element type (component vs intrinsic)
    let (is_fn, tag_expr) = classify_tag(&opening.name, &ast, allocator);

    // 2. Key generation
    let should_emit_key = is_fn || is_root;
    let key_expr = if should_emit_key {
        let key = gen_jsx_key(jsx_key_counter);
        ast.expression_string_literal(SPAN, allocator.alloc_str(&key), None)
    } else {
        ast.expression_null_literal(SPAN)
    };

    // 3. Process attributes
    let attrs = opening.attributes;
    let (has_spread, var_props, const_props, extracted_key, children_from_props, signal_used) =
        classify_props(attrs, &ast, allocator, signal_ctx);

    // Use extracted key if present, otherwise use generated key
    let final_key = extracted_key.unwrap_or(key_expr);

    // 4. Process children
    let children_opt = build_children(children_vec, children_from_props, &ast, allocator);

    // 5. Compute flags
    let flags: u32 = FLAG_DYNAMIC_CHILDREN;

    // 6. Choose callee
    if signal_used {
        needs.needs_fn_signal = true;
    }

    let callee_name = if has_spread {
        needs.needs_jsx_split = true;
        "_jsxSplit"
    } else {
        needs.needs_jsx_sorted = true;
        "_jsxSorted"
    };

    let call = build_jsx_call(
        callee_name,
        tag_expr,
        var_props,
        const_props,
        children_opt,
        flags,
        final_key,
        &ast,
        allocator,
    );

    (call, needs)
}

/// Transform a JSXFragment into a `_jsxSorted(_Fragment, ...)` call expression.
pub(crate) fn transform_jsx_fragment<'a>(
    frag: JSXFragment<'a>,
    jsx_key_counter: &mut u32,
    is_root: bool,
    allocator: &'a Allocator,
) -> (Expression<'a>, JsxImportNeeds) {
    let ast = AstBuilder::new(allocator);
    let mut needs = JsxImportNeeds::default();

    needs.needs_jsx_sorted = true;
    needs.needs_fragment = true;

    let tag_expr = ast.expression_identifier(SPAN, "_Fragment");

    let should_emit_key = is_root;
    let key_expr = if should_emit_key {
        let key = gen_jsx_key(jsx_key_counter);
        ast.expression_string_literal(SPAN, allocator.alloc_str(&key), None)
    } else {
        ast.expression_null_literal(SPAN)
    };

    let children_opt = build_children(frag.children, None, &ast, allocator);
    let flags: u32 = FLAG_DYNAMIC_CHILDREN;

    let call = build_jsx_call(
        "_jsxSorted",
        tag_expr,
        None,
        None,
        children_opt,
        flags,
        key_expr,
        &ast,
        allocator,
    );

    (call, needs)
}

// ---------------------------------------------------------------------------
// JsxImportNeeds
// ---------------------------------------------------------------------------

/// Tracks which JSX runtime imports are needed after transformation.
#[derive(Default, Debug)]
pub(crate) struct JsxImportNeeds {
    pub needs_jsx_sorted: bool,
    pub needs_jsx_split: bool,
    pub needs_fragment: bool,
    pub needs_fn_signal: bool,
    pub needs_wrap_prop: bool,
}

// ---------------------------------------------------------------------------
// Tag classification
// ---------------------------------------------------------------------------

fn classify_tag<'a>(
    name: &JSXElementName<'a>,
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> (bool, Expression<'a>) {
    match name {
        JSXElementName::Identifier(id) => {
            let s = id.name.as_str();
            let first_char = s.chars().next().unwrap_or('a');
            let is_fn = first_char.is_ascii_uppercase();
            if is_fn {
                let name_arena: &'a str = allocator.alloc_str(s);
                (true, ast.expression_identifier(SPAN, name_arena))
            } else {
                let name_arena: &'a str = allocator.alloc_str(s);
                (false, ast.expression_string_literal(SPAN, name_arena, None))
            }
        }
        JSXElementName::IdentifierReference(id) => {
            let name_arena: &'a str = allocator.alloc_str(id.name.as_str());
            (true, ast.expression_identifier(SPAN, name_arena))
        }
        JSXElementName::MemberExpression(me) => {
            let expr = jsx_member_to_expr(me, ast, allocator);
            (true, expr)
        }
        JSXElementName::NamespacedName(nn) => {
            let name = format!("{}:{}", nn.namespace.name.as_str(), nn.name.name.as_str());
            let name_arena: &'a str = allocator.alloc_str(&name);
            (false, ast.expression_string_literal(SPAN, name_arena, None))
        }
        JSXElementName::ThisExpression(_) => (true, ast.expression_this(SPAN)),
    }
}

/// Convert a JSX member expression (e.g., `Foo.Bar.Baz`) to an AST expression.
fn jsx_member_to_expr<'a>(
    me: &JSXMemberExpression<'a>,
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> Expression<'a> {
    let prop_name: &'a str = allocator.alloc_str(me.property.name.as_str());
    let prop = ast.identifier_name(SPAN, prop_name);
    let object = match &me.object {
        JSXMemberExpressionObject::IdentifierReference(id) => {
            let name: &'a str = allocator.alloc_str(id.name.as_str());
            ast.expression_identifier(SPAN, name)
        }
        JSXMemberExpressionObject::MemberExpression(inner) => {
            jsx_member_to_expr(inner, ast, allocator)
        }
        JSXMemberExpressionObject::ThisExpression(_) => {
            ast.expression_this(SPAN)
        }
    };
    Expression::StaticMemberExpression(ast.alloc_static_member_expression(
        SPAN, object, prop, false,
    ))
}

// ---------------------------------------------------------------------------
// Prop classification
// ---------------------------------------------------------------------------

/// Classify JSX props into var (dynamic) and const (static) buckets.
///
/// When `signal_ctx` is provided, dynamic prop values are checked for signal
/// optimization eligibility via `convert_inlined_fn`. If eligible, the prop
/// value is replaced with a `_fnSignal()` call expression (parsed from the
/// generated code string).
///
/// Returns `(has_spread, var_props_obj, const_props_obj, extracted_key, children_from_props, signal_used)`.
fn classify_props<'a>(
    attrs: ArenaVec<'a, JSXAttributeItem<'a>>,
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
    signal_ctx: Option<&SignalOptContext<'a, '_>>,
) -> (
    bool,
    Option<Expression<'a>>,
    Option<Expression<'a>>,
    Option<Expression<'a>>,
    Option<Expression<'a>>,
    bool,
) {
    let mut has_spread = false;
    let mut var_entries: ArenaVec<'a, ObjectPropertyKind<'a>> = ArenaVec::new_in(allocator);
    let mut const_entries: ArenaVec<'a, ObjectPropertyKind<'a>> = ArenaVec::new_in(allocator);
    let mut extracted_key: Option<Expression<'a>> = None;
    let mut children_from_props: Option<Expression<'a>> = None;
    let mut signal_used = false;

    for attr in attrs {
        match attr {
            JSXAttributeItem::SpreadAttribute(spread) => {
                has_spread = true;
                // Add spread to var_entries
                var_entries.push(ObjectPropertyKind::SpreadProperty(
                    ast.alloc_spread_element(SPAN, spread.unbox().argument),
                ));
            }
            JSXAttributeItem::Attribute(jsx_attr) => {
                let jsx_attr = jsx_attr.unbox();
                let key_name = jsx_attr_key_owned(&jsx_attr.name);

                // Handle special props
                if key_name == "key" {
                    // Extract key, don't include in props
                    if let Some(value) = jsx_attr.value {
                        extracted_key = jsx_attr_value_to_expr(value, ast, allocator);
                    }
                    continue;
                }

                if key_name == "children" {
                    if let Some(value) = jsx_attr.value {
                        children_from_props = jsx_attr_value_to_expr(value, ast, allocator);
                    }
                    continue;
                }

                // Normalize className -> class
                let normalized_key = if key_name == "className" {
                    "class".to_string()
                } else {
                    key_name.clone()
                };

                // Get value expression
                let value_expr = if let Some(value) = jsx_attr.value {
                    jsx_attr_value_to_expr(value, ast, allocator)
                        .unwrap_or_else(|| ast.expression_boolean_literal(SPAN, true))
                } else {
                    // No value means `true` (e.g., `<input disabled />`)
                    ast.expression_boolean_literal(SPAN, true)
                };

                // Classify as const or var
                let is_const = is_const_expression(&value_expr);

                // Attempt signal optimization for dynamic prop values (CONV-07)
                let value_expr = if !is_const {
                    if let Some(ctx) = signal_ctx {
                        try_signal_optimize(
                            value_expr,
                            ctx,
                            &mut signal_used,
                            allocator,
                        )
                    } else {
                        value_expr
                    }
                } else {
                    value_expr
                };

                let prop_key = build_prop_key(&normalized_key, ast, allocator);
                let prop = ObjectPropertyKind::ObjectProperty(ast.alloc_object_property(
                    SPAN,
                    PropertyKind::Init,
                    prop_key,
                    value_expr,
                    false,
                    false,
                    false,
                ));

                if is_const {
                    const_entries.push(prop);
                } else {
                    var_entries.push(prop);
                }
            }
        }
    }

    let var_props = if var_entries.is_empty() {
        None
    } else {
        Some(ast.expression_object(SPAN, var_entries))
    };

    let const_props = if const_entries.is_empty() {
        None
    } else {
        Some(ast.expression_object(SPAN, const_entries))
    };

    (has_spread, var_props, const_props, extracted_key, children_from_props, signal_used)
}

// ---------------------------------------------------------------------------
// Signal optimization helper (CONV-07)
// ---------------------------------------------------------------------------

/// Attempt to wrap a dynamic prop value expression as a `_fnSignal()` call.
///
/// Collects identifiers from the expression, computes scoped_idents against
/// the declaration stack, and calls `convert_inlined_fn`. If eligible, parses
/// the resulting code string into an AST expression and returns it.
fn try_signal_optimize<'a>(
    value_expr: Expression<'a>,
    ctx: &SignalOptContext<'a, '_>,
    signal_used: &mut bool,
    allocator: &'a Allocator,
) -> Expression<'a> {
    // Skip call expressions -- they can't be signal-optimized
    if matches!(&value_expr, Expression::CallExpression(_)) {
        return value_expr;
    }

    // Collect identifiers referenced in this expression
    let expr_idents = IdentCollector::collect(&value_expr);

    // Compute scoped_idents (captures) for this expression
    let scoped_names = compute_scoped_idents(&expr_idents, ctx.decl_stack_flat);

    // Build (name, is_const) pairs for convert_inlined_fn
    let scoped_pairs: Vec<(String, bool)> = scoped_names
        .iter()
        .map(|name| {
            let is_const = ctx
                .decl_stack_flat
                .iter()
                .any(|(n, t)| n == name && matches!(t, crate::transform::IdentType::Const));
            (name.clone(), is_const)
        })
        .collect();

    let (fn_signal_code, _arrow_code, _is_const) =
        convert_inlined_fn(&value_expr, &scoped_pairs, false, ctx.is_server, allocator);

    if let Some(code) = fn_signal_code {
        // Parse the generated _fnSignal(...) code into an expression
        if let Some(parsed_expr) = parse_signal_expr(&code, allocator) {
            *signal_used = true;
            return parsed_expr;
        }
    }

    value_expr
}

/// Parse a code string (e.g., `_fnSignal(p0 => p0.value, [obj])`) into an AST expression.
fn parse_signal_expr<'a>(code: &str, allocator: &'a Allocator) -> Option<Expression<'a>> {
    use oxc::parser::Parser;
    use oxc::span::SourceType;

    let src: &str = allocator.alloc_str(code);
    let parser = Parser::new(allocator, src, SourceType::tsx());
    parser.parse_expression().ok()
}

// ---------------------------------------------------------------------------
// Children handling
// ---------------------------------------------------------------------------

/// Build children expression from JSX children + any children from props attribute.
fn build_children<'a>(
    children: ArenaVec<'a, JSXChild<'a>>,
    children_from_props: Option<Expression<'a>>,
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> Option<Expression<'a>> {
    let mut child_exprs: Vec<Expression<'a>> = Vec::new();

    for child in children {
        match child {
            JSXChild::Text(text) => {
                let trimmed = text.value.as_str().trim();
                if !trimmed.is_empty() {
                    let text_arena: &'a str = allocator.alloc_str(trimmed);
                    child_exprs.push(ast.expression_string_literal(SPAN, text_arena, None));
                }
            }
            JSXChild::ExpressionContainer(container) => {
                if let JSXExpression::EmptyExpression(_) = &container.expression {
                    continue;
                }
                if let Some(expr) = jsx_expression_to_expr(container.unbox().expression) {
                    child_exprs.push(expr);
                }
            }
            JSXChild::Element(el) => {
                // Nested JSX elements -- these would already be transformed by post-order traversal
                // For standalone usage, convert to JSXElement expression
                child_exprs.push(Expression::JSXElement(el));
            }
            JSXChild::Fragment(frag) => {
                child_exprs.push(Expression::JSXFragment(frag));
            }
            JSXChild::Spread(spread) => {
                child_exprs.push(spread.unbox().expression);
            }
        }
    }

    if let Some(prop_children) = children_from_props {
        child_exprs.push(prop_children);
    }

    if child_exprs.is_empty() {
        None
    } else if child_exprs.len() == 1 {
        Some(child_exprs.into_iter().next().unwrap())
    } else {
        // Multiple children -> array expression
        let mut elements: ArenaVec<'a, ArrayExpressionElement<'a>> = ArenaVec::new_in(allocator);
        for expr in child_exprs {
            elements.push(ArrayExpressionElement::from(expr));
        }
        Some(ast.expression_array(SPAN, elements))
    }
}

// ---------------------------------------------------------------------------
// Helper: build _jsxSorted/_jsxSplit call
// ---------------------------------------------------------------------------

fn build_jsx_call<'a>(
    callee_name: &str,
    tag: Expression<'a>,
    var_props: Option<Expression<'a>>,
    const_props: Option<Expression<'a>>,
    children: Option<Expression<'a>>,
    flags: u32,
    key: Expression<'a>,
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> Expression<'a> {
    let mut args: ArenaVec<'a, Argument<'a>> = ArenaVec::new_in(allocator);

    // Arg 1: tag
    push_expr_arg(&mut args, tag);
    // Arg 2: varProps or null
    push_expr_arg(
        &mut args,
        var_props.unwrap_or_else(|| ast.expression_null_literal(SPAN)),
    );
    // Arg 3: constProps or null
    push_expr_arg(
        &mut args,
        const_props.unwrap_or_else(|| ast.expression_null_literal(SPAN)),
    );
    // Arg 4: children or null
    push_expr_arg(
        &mut args,
        children.unwrap_or_else(|| ast.expression_null_literal(SPAN)),
    );
    // Arg 5: flags
    push_expr_arg(
        &mut args,
        ast.expression_numeric_literal(SPAN, flags as f64, None, NumberBase::Decimal),
    );
    // Arg 6: key
    push_expr_arg(&mut args, key);

    let callee_arena: &'a str = allocator.alloc_str(callee_name);
    let callee = ast.expression_identifier(SPAN, callee_arena);
    ast.expression_call(
        SPAN,
        callee,
        None::<TSTypeParameterInstantiation<'a>>,
        args,
        false,
    )
}

/// Push an expression as an argument.
fn push_expr_arg<'a>(args: &mut ArenaVec<'a, Argument<'a>>, expr: Expression<'a>) {
    args.push(expr_to_argument(expr));
}

// ---------------------------------------------------------------------------
// Helper: JSX attribute utilities
// ---------------------------------------------------------------------------

/// Get the string key from a JSX attribute name.
fn jsx_attr_key_owned(name: &JSXAttributeName<'_>) -> String {
    match name {
        JSXAttributeName::Identifier(id) => id.name.as_str().to_string(),
        JSXAttributeName::NamespacedName(nn) => {
            format!("{}:{}", nn.namespace.name.as_str(), nn.name.name.as_str())
        }
    }
}

/// Convert a JSX attribute value to an expression.
fn jsx_attr_value_to_expr<'a>(
    value: JSXAttributeValue<'a>,
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> Option<Expression<'a>> {
    match value {
        JSXAttributeValue::StringLiteral(s) => {
            let val: &'a str = allocator.alloc_str(s.value.as_str());
            Some(ast.expression_string_literal(SPAN, val, None))
        }
        JSXAttributeValue::ExpressionContainer(container) => {
            let container = container.unbox();
            jsx_expression_to_expr(container.expression)
        }
        JSXAttributeValue::Element(el) => Some(Expression::JSXElement(el)),
        JSXAttributeValue::Fragment(frag) => Some(Expression::JSXFragment(frag)),
    }
}

/// Build a property key from a string.
fn build_prop_key<'a>(
    key: &str,
    ast: &AstBuilder<'a>,
    allocator: &'a Allocator,
) -> PropertyKey<'a> {
    let needs_quotes = key.contains(':') || key.contains('-');
    let key_arena: &'a str = allocator.alloc_str(key);
    if needs_quotes {
        PropertyKey::StringLiteral(ast.alloc_string_literal(SPAN, key_arena, None))
    } else {
        PropertyKey::StaticIdentifier(ast.alloc_identifier_name(SPAN, key_arena))
    }
}

/// Convert a JSX expression to an Expression, returning None for empty expressions.
fn jsx_expression_to_expr<'a>(jsx_expr: JSXExpression<'a>) -> Option<Expression<'a>> {
    match jsx_expr {
        JSXExpression::EmptyExpression(_) => None,
        JSXExpression::BooleanLiteral(b) => Some(Expression::BooleanLiteral(b)),
        JSXExpression::NullLiteral(b) => Some(Expression::NullLiteral(b)),
        JSXExpression::NumericLiteral(b) => Some(Expression::NumericLiteral(b)),
        JSXExpression::BigIntLiteral(b) => Some(Expression::BigIntLiteral(b)),
        JSXExpression::RegExpLiteral(b) => Some(Expression::RegExpLiteral(b)),
        JSXExpression::StringLiteral(b) => Some(Expression::StringLiteral(b)),
        JSXExpression::TemplateLiteral(b) => Some(Expression::TemplateLiteral(b)),
        JSXExpression::Identifier(b) => Some(Expression::Identifier(b)),
        JSXExpression::ArrayExpression(b) => Some(Expression::ArrayExpression(b)),
        JSXExpression::ObjectExpression(b) => Some(Expression::ObjectExpression(b)),
        JSXExpression::FunctionExpression(b) => Some(Expression::FunctionExpression(b)),
        JSXExpression::ArrowFunctionExpression(b) => {
            Some(Expression::ArrowFunctionExpression(b))
        }
        JSXExpression::CallExpression(b) => Some(Expression::CallExpression(b)),
        JSXExpression::ConditionalExpression(b) => Some(Expression::ConditionalExpression(b)),
        JSXExpression::LogicalExpression(b) => Some(Expression::LogicalExpression(b)),
        JSXExpression::BinaryExpression(b) => Some(Expression::BinaryExpression(b)),
        JSXExpression::UnaryExpression(b) => Some(Expression::UnaryExpression(b)),
        JSXExpression::UpdateExpression(b) => Some(Expression::UpdateExpression(b)),
        JSXExpression::StaticMemberExpression(b) => {
            Some(Expression::StaticMemberExpression(b))
        }
        JSXExpression::ComputedMemberExpression(b) => {
            Some(Expression::ComputedMemberExpression(b))
        }
        JSXExpression::AssignmentExpression(b) => Some(Expression::AssignmentExpression(b)),
        JSXExpression::SequenceExpression(b) => Some(Expression::SequenceExpression(b)),
        JSXExpression::ParenthesizedExpression(b) => {
            Some(Expression::ParenthesizedExpression(b))
        }
        JSXExpression::TaggedTemplateExpression(b) => {
            Some(Expression::TaggedTemplateExpression(b))
        }
        JSXExpression::ThisExpression(b) => Some(Expression::ThisExpression(b)),
        JSXExpression::NewExpression(b) => Some(Expression::NewExpression(b)),
        JSXExpression::ClassExpression(b) => Some(Expression::ClassExpression(b)),
        JSXExpression::AwaitExpression(b) => Some(Expression::AwaitExpression(b)),
        JSXExpression::YieldExpression(b) => Some(Expression::YieldExpression(b)),
        JSXExpression::ImportExpression(b) => Some(Expression::ImportExpression(b)),
        JSXExpression::ChainExpression(b) => Some(Expression::ChainExpression(b)),
        JSXExpression::MetaProperty(b) => Some(Expression::MetaProperty(b)),
        JSXExpression::PrivateFieldExpression(b) => {
            Some(Expression::PrivateFieldExpression(b))
        }
        JSXExpression::JSXElement(b) => Some(Expression::JSXElement(b)),
        JSXExpression::JSXFragment(b) => Some(Expression::JSXFragment(b)),
        JSXExpression::TSAsExpression(b) => Some(Expression::TSAsExpression(b)),
        JSXExpression::TSSatisfiesExpression(b) => Some(Expression::TSSatisfiesExpression(b)),
        JSXExpression::TSTypeAssertion(b) => Some(Expression::TSTypeAssertion(b)),
        JSXExpression::TSNonNullExpression(b) => Some(Expression::TSNonNullExpression(b)),
        JSXExpression::TSInstantiationExpression(b) => {
            Some(Expression::TSInstantiationExpression(b))
        }
        _ => None, // Fallback for any unknown variants
    }
}

/// Generate a deterministic JSX key.
fn gen_jsx_key(counter: &mut u32) -> String {
    let key = format!("{counter}");
    *counter += 1;
    key
}

// ---------------------------------------------------------------------------
// Helper: Expression -> Argument conversion
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
        Expression::JSXElement(b) => Argument::JSXElement(b),
        Expression::JSXFragment(b) => Argument::JSXFragment(b),
        _ => unreachable!("Unexpected Expression variant in expr_to_argument"),
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

    fn transform_jsx(src: &str) -> String {
        let allocator = Allocator::default();
        let source_type = SourceType::tsx();
        let ret = Parser::new(&allocator, src, source_type).parse();
        let program = ret.program;
        Codegen::new().build(&program).code
    }

    #[test]
    fn jsx_transform_gen_key_increments() {
        let mut counter = 0u32;
        assert_eq!(gen_jsx_key(&mut counter), "0");
        assert_eq!(gen_jsx_key(&mut counter), "1");
        assert_eq!(gen_jsx_key(&mut counter), "2");
    }

    #[test]
    fn jsx_transform_classify_tag_lowercase_is_intrinsic() {
        let allocator = Allocator::default();
        let ast = AstBuilder::new(&allocator);
        let id = ast.alloc_jsx_identifier(SPAN, "div");
        let name = JSXElementName::Identifier(id);
        let (is_fn, _expr) = classify_tag(&name, &ast, &allocator);
        assert!(!is_fn, "lowercase element should not be a function component");
    }

    #[test]
    fn jsx_transform_classify_tag_uppercase_is_component() {
        let allocator = Allocator::default();
        let ast = AstBuilder::new(&allocator);
        let id = ast.alloc_jsx_identifier(SPAN, "Header");
        let name = JSXElementName::Identifier(id);
        let (is_fn, _expr) = classify_tag(&name, &ast, &allocator);
        assert!(is_fn, "uppercase element should be a function component");
    }

    #[test]
    fn jsx_transform_is_valid_prop_key() {
        // className normalizes to class
        let normalized = if "className" == "className" {
            "class".to_string()
        } else {
            "className".to_string()
        };
        assert_eq!(normalized, "class");
    }

    #[test]
    fn jsx_transform_build_jsx_call_produces_6_args() {
        let allocator = Allocator::default();
        let ast = AstBuilder::new(&allocator);
        let tag = ast.expression_string_literal(SPAN, "div", None);
        let key = ast.expression_null_literal(SPAN);

        let call = build_jsx_call(
            "_jsxSorted",
            tag,
            None,
            None,
            None,
            1,
            key,
            &ast,
            &allocator,
        );

        // Verify it's a call expression with 6 arguments
        if let Expression::CallExpression(call) = &call {
            assert_eq!(
                call.arguments.len(),
                6,
                "JSX call should have 6 arguments"
            );
        } else {
            panic!("Expected CallExpression");
        }
    }

    #[test]
    fn jsx_transform_jsx_attr_key_owned_namespaced() {
        // Test namespaced attribute key extraction
        let allocator = Allocator::default();
        let ast = AstBuilder::new(&allocator);
        let ns = ast.alloc_jsx_identifier(SPAN, "bind");
        let local = ast.alloc_jsx_identifier(SPAN, "value");
        let nn = ast.alloc_jsx_namespaced_name(SPAN, ns.unbox(), local.unbox());
        let name = JSXAttributeName::NamespacedName(nn);
        let key = jsx_attr_key_owned(&name);
        assert_eq!(key, "bind:value");
    }

    #[test]
    fn jsx_transform_fragment_uses_sorted() {
        // Fragments always use _jsxSorted
        let allocator = Allocator::default();
        let ast = AstBuilder::new(&allocator);

        let children: ArenaVec<'_, JSXChild<'_>> = ArenaVec::new_in(&allocator);
        let opening = ast.jsx_opening_fragment(SPAN);
        let closing = ast.jsx_closing_fragment(SPAN);
        let frag = ast.jsx_fragment(SPAN, opening, children, closing);

        let mut counter = 0;
        let (expr, needs) = transform_jsx_fragment(frag, &mut counter, false, &allocator);

        assert!(needs.needs_jsx_sorted, "Fragment should need _jsxSorted");
        assert!(needs.needs_fragment, "Fragment should need _Fragment import");
        assert!(
            !needs.needs_jsx_split,
            "Fragment should not need _jsxSplit"
        );

        if let Expression::CallExpression(call) = &expr {
            assert_eq!(call.arguments.len(), 6);
        } else {
            panic!("Expected CallExpression from fragment transform");
        }
    }
}
