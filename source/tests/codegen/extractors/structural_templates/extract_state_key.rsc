/**
 * Extract State Key
 *
 * Extracts the root variable name from expressions
 */

/**
 * Extract state key (root variable name) from expression
 * Example: user.isLoggedIn → "user"
 */
pub fn extract_state_key(expr: &Expression, component: &Component) -> Option<Str> {
    match expr {
        Expression::Identifier(ref id) => {
            Some(id.name.clone())
        }

        Expression::MemberExpression(_) => {
            // Get root object: user.isLoggedIn → "user"
            let mut current = expr;

            loop {
                if let Expression::MemberExpression(ref member) = current {
                    current = &member.object;
                } else {
                    break;
                }
            }

            if let Expression::Identifier(ref id) = current {
                Some(id.name.clone())
            } else {
                None
            }
        }

        Expression::UnaryExpression(ref unary) => {
            extract_state_key(&unary.argument, component)
        }

        _ => None
    }
}
