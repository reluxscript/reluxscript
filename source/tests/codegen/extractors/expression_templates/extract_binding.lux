/**
 * Extract Binding
 *
 * Extracts binding name from simple expressions
 */

use "./build_member_path.rsc" { build_member_path };

/**
 * Extract binding from expression
 * Returns the binding name or None if not extractable
 */
pub fn extract_binding(expr: &Expression) -> Option<Str> {
    match expr {
        Expression::Identifier(ref id) => {
            Some(id.name.clone())
        }

        Expression::MemberExpression(_) => {
            Some(build_member_path(expr))
        }

        _ => None
    }
}
