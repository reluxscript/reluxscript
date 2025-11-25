/**
 * Build Member Path
 *
 * Builds string paths from member expressions (e.g., user.name → "user.name")
 */

/**
 * Build member expression path: user.name → "user.name"
 *
 * Recursively traverses the member expression tree and builds a dot-separated path
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

/**
 * Build optional member expression path: viewModel?.userEmail → "viewModel.userEmail"
 *
 * Handles both regular and optional chaining expressions
 * Returns None if the path contains computed properties
 */
pub fn build_optional_member_path(expr: &Expression) -> Option<Str> {
    let mut parts = vec![];
    let mut current = expr;

    // Traverse the member/optional member expression chain
    loop {
        match current {
            Expression::OptionalMemberExpression(ref opt_member) => {
                if let Expression::Identifier(ref property) = &opt_member.property {
                    parts.insert(0, property.name.clone());
                } else {
                    return None; // Computed property
                }
                current = &opt_member.object;
            }

            Expression::MemberExpression(ref member_expr) => {
                if let Expression::Identifier(ref property) = &member_expr.property {
                    parts.insert(0, property.name.clone());
                } else {
                    return None; // Computed property
                }
                current = &member_expr.object;
            }

            _ => break
        }
    }

    // Add the base identifier
    if let Expression::Identifier(ref id) = current {
        parts.insert(0, id.name.clone());
        Some(parts.join("."))
    } else {
        None
    }
}
