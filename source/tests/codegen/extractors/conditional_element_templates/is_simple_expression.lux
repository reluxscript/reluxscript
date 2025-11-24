/**
 * Is Simple Expression
 *
 * Checks if an expression is simple (identifier, member, literal)
 */

/**
 * Check if expression is simple (identifier, member, literal)
 * Simple expressions can be easily evaluated or referenced
 */
pub fn is_simple_expression(expr: &Expression) -> bool {
    matches!(expr,
        Expression::Identifier(_) |
        Expression::MemberExpression(_) |
        Expression::StringLiteral(_) |
        Expression::NumericLiteral(_) |
        Expression::BooleanLiteral(_) |
        Expression::NullLiteral
    )
}
