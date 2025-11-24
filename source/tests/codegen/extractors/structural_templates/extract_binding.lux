/**
 * Extract Binding
 *
 * Extracts binding from expressions for structural templates
 */

use "./build_member_path.rsc" { build_member_path };

/**
 * Extract binding from expression
 * Handles identifiers, member expressions, and negation (!isLoading)
 */
pub fn extract_binding(expr: &Expression, component: &Component) -> Option<Str> {
    match expr {
        Expression::Identifier(ref id) => {
            Some(id.name.clone())
        }

        Expression::MemberExpression(_) => {
            Some(build_member_path(expr))
        }

        Expression::UnaryExpression(ref unary) => {
            // Handle !isLoading
            if unary.operator == "!" {
                if let Some(binding) = extract_binding(&unary.argument, component) {
                    Some(format!("!{}", binding))
                } else {
                    None
                }
            } else {
                None
            }
        }

        _ => None
    }
}
