/**
 * Build Member Path
 *
 * Build member expression path for structural templates
 */

/**
 * Build member expression path
 * Example: user.profile.name → "user.profile.name"
 */
pub fn build_member_path(expr: &Expression) -> Str {
    let mut parts = vec![];
    let mut current = expr;

    // Traverse the member expression chain
    loop {
        if let Expression::MemberExpression(ref member_expr) = current {
            // Add property name to parts (at the beginning)
            if let Expression::Identifier(ref property) = &member_expr.property {
                parts.insert(0, property.name.clone());
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
    }

    parts.join(".")
}
