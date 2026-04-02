//! Const evaluation utilities.
//!
//! Determine whether an expression is a compile-time constant for JSX prop
//! classification. Used by the JSX transform to decide whether a prop value
//! goes into const props or var props.

/// Determine whether a JSX prop value expression is a compile-time constant.
///
/// Returns `true` for literals, template literals with no expressions,
/// typeof expressions, and other statically-known values. Returns `false`
/// for identifiers, member expressions, calls, and other dynamic values.
///
/// This is used for the var/const prop split in JSX transformation.
/// Signal wrapping logic (_wrapProp, _fnSignal) handles reactive cases
/// independently and may promote expressions from var to const after wrapping.
pub(crate) fn is_const_expression(expr: &oxc::ast::ast::Expression<'_>) -> bool {
    use oxc::ast::ast::*;
    match expr {
        // Literals are always const
        Expression::StringLiteral(_)
        | Expression::NumericLiteral(_)
        | Expression::BooleanLiteral(_)
        | Expression::NullLiteral(_)
        | Expression::BigIntLiteral(_)
        | Expression::RegExpLiteral(_) => true,

        // Template literals are const only if they have no expressions
        Expression::TemplateLiteral(tpl) => {
            tpl.expressions.is_empty() || tpl.expressions.iter().all(|e| is_const_expression(e))
        }

        // typeof is always a string
        Expression::UnaryExpression(unary) => {
            matches!(unary.operator, UnaryOperator::Typeof) || is_const_expression(&unary.argument)
        }

        // Ternary: const if all three parts are const
        Expression::ConditionalExpression(cond) => {
            is_const_expression(&cond.test)
                && is_const_expression(&cond.consequent)
                && is_const_expression(&cond.alternate)
        }

        // Binary expressions: const if both sides are const
        Expression::BinaryExpression(bin) => {
            is_const_expression(&bin.left) && is_const_expression(&bin.right)
        }

        // Object expressions: const if all property values are const
        Expression::ObjectExpression(obj) => obj.properties.iter().all(|prop| match prop {
            ObjectPropertyKind::ObjectProperty(p) => is_const_expression(&p.value),
            ObjectPropertyKind::SpreadProperty(_) => false,
        }),

        // Array expressions: const if all elements are const
        Expression::ArrayExpression(arr) => arr.elements.iter().all(|elem| match elem {
            ArrayExpressionElement::SpreadElement(_) => false,
            ArrayExpressionElement::Elision(_) => true,
            ArrayExpressionElement::BooleanLiteral(_)
            | ArrayExpressionElement::NullLiteral(_)
            | ArrayExpressionElement::NumericLiteral(_)
            | ArrayExpressionElement::BigIntLiteral(_)
            | ArrayExpressionElement::RegExpLiteral(_)
            | ArrayExpressionElement::StringLiteral(_) => true,
            _ => false,
        }),

        // Parenthesized expressions: const if inner is const
        Expression::ParenthesizedExpression(paren) => is_const_expression(&paren.expression),

        // Everything else (identifiers, member exprs, calls, arrows, etc.) is NOT const
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_expr<'a>(
        allocator: &'a oxc::allocator::Allocator,
        src: &'a str,
    ) -> oxc::ast::ast::Expression<'a> {
        let source_type = oxc::span::SourceType::mjs();
        let parser = oxc::parser::Parser::new(allocator, src, source_type);
        parser.parse_expression().unwrap()
    }

    #[test]
    fn test_numeric_literal_is_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "42");
        assert!(is_const_expression(&expr));
    }

    #[test]
    fn test_string_literal_is_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "\"hello\"");
        assert!(is_const_expression(&expr));
    }

    #[test]
    fn test_boolean_literal_is_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "true");
        assert!(is_const_expression(&expr));
    }

    #[test]
    fn test_null_literal_is_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "null");
        assert!(is_const_expression(&expr));
    }

    #[test]
    fn test_identifier_is_not_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "foo");
        assert!(!is_const_expression(&expr));
    }

    #[test]
    fn test_call_expression_is_not_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "foo()");
        assert!(!is_const_expression(&expr));
    }

    #[test]
    fn test_member_expression_is_not_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "a.b");
        assert!(!is_const_expression(&expr));
    }

    #[test]
    fn test_typeof_is_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "typeof foo");
        assert!(is_const_expression(&expr));
    }

    #[test]
    fn test_binary_of_literals_is_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "1 + 2");
        assert!(is_const_expression(&expr));
    }

    #[test]
    fn test_binary_with_identifier_is_not_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "1 + x");
        assert!(!is_const_expression(&expr));
    }

    #[test]
    fn test_template_literal_no_expressions_is_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "`hello`");
        assert!(is_const_expression(&expr));
    }

    #[test]
    fn test_ternary_all_const_is_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "true ? 1 : 2");
        assert!(is_const_expression(&expr));
    }

    #[test]
    fn test_ternary_with_identifier_is_not_const() {
        let allocator = oxc::allocator::Allocator::default();
        let expr = parse_expr(&allocator, "x ? 1 : 2");
        assert!(!is_const_expression(&expr));
    }
}
