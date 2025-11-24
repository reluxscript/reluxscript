/**
 * Is Condition Evaluable Client Side
 *
 * Checks if a condition can be evaluated client-side
 * Only simple boolean logic is supported (&&, ||, !, comparisons)
 */

use "./is_simple_expression.rsc" { is_simple_expression };

/**
 * Check if condition can be evaluated client-side
 * Only simple boolean logic is supported (&&, ||, !, comparisons)
 *
 * @param expr - Expression to check
 * @param bindings - Available bindings (for context)
 * @returns true if the condition can be evaluated client-side
 */
pub fn is_condition_evaluable_client_side(expr: &Expression, bindings: &Vec<Str>) -> bool {
    match expr {
        // Simple identifier: myState1
        Expression::Identifier(_) => true,

        // Unary: !myState1
        Expression::UnaryExpression(ref unary) => {
            if unary.operator == "!" {
                is_condition_evaluable_client_side(&unary.argument, bindings)
            } else {
                false
            }
        }

        // Logical: myState1 && myState2, myState1 || myState2
        Expression::LogicalExpression(ref logical) => {
            is_condition_evaluable_client_side(&logical.left, bindings) &&
            is_condition_evaluable_client_side(&logical.right, bindings)
        }

        // Binary comparisons: count > 0, name === "admin"
        Expression::BinaryExpression(ref binary) => {
            // Simple comparisons are evaluable
            let operators = vec!["==", "===", "!=", "!==", "<", ">", "<=", ">="];
            if operators.contains(&binary.operator.as_str()) {
                is_simple_expression(&binary.left) && is_simple_expression(&binary.right)
            } else {
                false
            }
        }

        // Member expressions: user.isAdmin
        Expression::MemberExpression(_) => true,

        // Complex expressions require server evaluation
        _ => false,
    }
}
