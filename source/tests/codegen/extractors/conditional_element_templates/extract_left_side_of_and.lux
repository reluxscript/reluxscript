/**
 * Extract Left Side Of And
 *
 * Extracts the left side of a chained AND expression
 */

/**
 * Extract the left side of a chained AND expression
 * Example: myState1 && !myState2 && <div /> → returns myState1 && !myState2
 *
 * Returns the condition part of a logical AND expression that has JSX on the right
 */
pub fn extract_left_side_of_and(expr: &Expression) -> &Expression {
    // Check if it's a logical AND expression
    if let Expression::LogicalExpression(ref logical) = expr {
        if logical.operator == "&&" {
            // Check if right side is JSX
            let right_is_jsx = matches!(&logical.right,
                Expression::JSXElement(_) | Expression::JSXFragment(_)
            );

            if right_is_jsx {
                // Return the left side (the condition)
                return &logical.left;
            }
        }
    }

    // Otherwise, return the expression as-is
    expr
}
