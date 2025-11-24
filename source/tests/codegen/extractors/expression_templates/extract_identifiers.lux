/**
 * Extract Identifiers
 *
 * Extracts all identifiers from an expression recursively
 */

use "./build_member_path.rsc" { build_member_path };

/**
 * Extract all identifiers from expression
 * Recursively traverses the expression tree
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

        Expression::UnaryExpression(ref unary_expr) => {
            extract_identifiers(&unary_expr.argument, result);
        }

        Expression::MemberExpression(_) => {
            let path = build_member_path(expr);
            result.push(path);
        }

        _ => {
            // Other expression types don't contribute identifiers
        }
    }
}
