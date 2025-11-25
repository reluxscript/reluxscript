/**
 * Map Call Expression Detection
 *
 * Checks if an expression is a .map() call (including chained calls)
 */

/**
 * Check if expression is a .map() call (including chained calls like .filter().map())
 *
 * Returns true if the expression is:
 * - A direct .map() call: items.map(...)
 * - A chained call ending in .map(): items.filter(...).map(...)
 */
pub fn is_map_call_expression(expr: &Expression) -> bool {
    // Check if it's a call expression at all
    if let Expression::CallExpression(ref call_expr) = expr {
        // Check if it's a direct .map() call
        if let Expression::MemberExpression(ref member_expr) = &call_expr.callee {
            if let Expression::Identifier(ref property) = &member_expr.property {
                if property.name == "map" {
                    return true;
                }
            }
        }

        // Check if it's a chained call ending in .map()
        // e.g., items.filter(...).map(...), items.slice(0, 10).map(...)
        let mut current = expr;

        loop {
            if let Expression::CallExpression(ref call) = current {
                if let Expression::MemberExpression(ref member) = &call.callee {
                    if let Expression::Identifier(ref property) = &member.property {
                        if property.name == "map" {
                            return true;
                        }
                    }

                    // Move to the next call in the chain
                    current = &member.object;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
    }

    false
}
