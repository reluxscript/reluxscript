/**
 * Build full path from member expression
 *
 * Example: todo.author.name → "todo.author.name"
 */

/**
 * Build full dotted path from a member expression
 *
 * Recursively walks up the member expression chain to build the full path.
 * Only supports non-computed properties (identifier properties).
 *
 * @param expr - The member expression to convert
 * @returns Full path string or None if computed properties are found
 */
pub fn build_member_expression_path(expr: &Expression) -> Option<Str> {
    let mut parts: Vec<Str> = vec![];
    let mut current = expr.clone();

    loop {
        match &current {
            Expression::MemberExpression(ref member_expr) => {
                // Only support non-computed properties
                if member_expr.computed {
                    return None;
                }

                // Extract property name
                if let Expression::Identifier(ref prop) = member_expr.property {
                    parts.insert(0, prop.name.clone());
                } else {
                    return None;
                }

                // Move to the object
                current = member_expr.object.clone();
            }

            Expression::Identifier(ref id) => {
                // Base case: found the root identifier
                parts.insert(0, id.name.clone());
                break;
            }

            _ => {
                // Unsupported expression type
                return None;
            }
        }
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("."))
    }
}
