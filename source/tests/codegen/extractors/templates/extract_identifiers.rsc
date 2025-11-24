/**
 * Extract Identifiers
 *
 * Extracts all identifiers from an expression recursively
 */

use "./build_member_path.rsc" { build_member_path };

/**
 * Extract all identifiers from expression
 *
 * Recursively traverses the expression tree and collects all identifier names
 * and member expression paths
 *
 * @param expr - Expression to extract identifiers from
 * @param result - Mutable vector to accumulate identifiers
 */
pub fn extract_identifiers(expr: &Expression, result: &mut Vec<Str>) {
    match expr {
        Expression::Identifier(ref id) => {
            result.push(id.name.clone());
        }

        Expression::BinaryExpression(ref bin_expr) => {
            extract_identifiers(&bin_expr.left, result);
            extract_identifiers(&bin_expr.right, result);
        }

        Expression::LogicalExpression(ref log_expr) => {
            extract_identifiers(&log_expr.left, result);
            extract_identifiers(&log_expr.right, result);
        }

        Expression::UnaryExpression(ref unary_expr) => {
            extract_identifiers(&unary_expr.argument, result);
        }

        Expression::MemberExpression(_) => {
            result.push(build_member_path(expr));
        }

        _ => {
            // Other expression types don't contain simple identifiers we track
        }
    }
}
