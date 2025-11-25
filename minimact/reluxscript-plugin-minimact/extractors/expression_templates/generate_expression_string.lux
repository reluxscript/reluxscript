/**
 * Generate Expression String
 *
 * Generates expression string for complex expressions
 */

use "./build_member_path.rsc" { build_member_path };

/**
 * Generate expression string for complex expressions
 * Recursively builds a string representation of the expression
 */
pub fn generate_expression_string(expr: &Expression) -> Str {
    match expr {
        Expression::Identifier(ref id) => {
            id.name.clone()
        }

        Expression::NumericLiteral(ref num_lit) => {
            num_lit.value.to_string()
        }

        Expression::BinaryExpression(ref bin_expr) => {
            let left = generate_expression_string(&bin_expr.left);
            let right = generate_expression_string(&bin_expr.right);
            format!("{} {} {}", left, bin_expr.operator, right)
        }

        Expression::UnaryExpression(ref unary_expr) => {
            let arg = generate_expression_string(&unary_expr.argument);
            format!("{}{}", unary_expr.operator, arg)
        }

        Expression::MemberExpression(_) => {
            build_member_path(expr)
        }

        _ => String::from("?")
    }
}
