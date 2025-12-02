/**
 * Build Member Path
 *
 * Build member expression path (user.profile.name → "user.profile.name")
 */

/**
 * Build member expression path
 * Returns the dot-separated path or None if unable to build
 */
pub fn build_member_path(expr: &Expression) -> Option<Str> {
    let mut parts = vec![];
    let mut current = expr;

    // Traverse the member expression chain
    loop {
        if let Expression::MemberExpression(ref member_expr) = current {
            // Add property name to parts (at the beginning)
            if let Expression::Identifier(ref property) = &member_expr.property {
                parts.insert(0, property.name.clone());
            } else {
                return None;
            }

            // Move to the object
            current = &member_expr.object;
        } else {
            break;
        }
    }

    // Add the base identifier
    if let Expression::Identifier(ref id) = current {
        parts.insert(0, id.name.clone());
    } else {
        return None;
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join("."))
    }
}
