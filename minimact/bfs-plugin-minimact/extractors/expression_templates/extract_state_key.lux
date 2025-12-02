/**
 * Extract State Key
 *
 * Extracts the root variable name from an expression
 */

/**
 * Extract state key (root variable)
 * Example: user.profile.name → "user"
 */
pub fn extract_state_key(expr: &Expression, component: &Component) -> Option<Str> {
    match expr {
        Expression::Identifier(ref id) => {
            Some(id.name.clone())
        }

        Expression::MemberExpression(ref member_expr) => {
            // Traverse to the root
            let mut current: &Expression = expr;

            loop {
                if let Expression::MemberExpression(ref member) = current {
                    current = &member.object;
                } else {
                    break;
                }
            }

            // Return the root identifier
            if let Expression::Identifier(ref id) = current {
                Some(id.name.clone())
            } else {
                None
            }
        }

        _ => None
    }
}
