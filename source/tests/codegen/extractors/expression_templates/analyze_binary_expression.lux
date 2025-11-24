/**
 * Analyze Binary Expression
 *
 * Analyzes binary expressions to extract arithmetic operations
 */

use "./is_binding_expression.rsc" { is_binding_expression };

/**
 * Arithmetic operation
 */
pub struct ArithmeticOperation {
    pub op: Str,
    pub value: f64,
    pub side: Str,
}

/**
 * Binary expression analysis result
 */
pub struct BinaryExpressionAnalysis {
    pub analysis_type: Str,
    pub operations: Vec<ArithmeticOperation>,
}

/**
 * Analyze binary expression to extract arithmetic operations
 *
 * Example: count * 2 + 1 with binding="count"
 * Returns operations: [{ op: '*', value: 2, side: 'right' }, { op: '+', value: 1, side: 'right' }]
 */
pub fn analyze_binary_expression(expr: &Expression, target_binding: &Str) -> Option<BinaryExpressionAnalysis> {
    let mut operations = vec![];

    analyze_node(expr, target_binding, &mut operations);

    if !operations.is_empty() {
        Some(BinaryExpressionAnalysis {
            analysis_type: String::from("arithmetic"),
            operations,
        })
    } else {
        None
    }
}

/**
 * Internal: Recursively analyze node
 */
fn analyze_node(node: &Expression, target_binding: &Str, operations: &mut Vec<ArithmeticOperation>) {
    if let Expression::BinaryExpression(ref bin_expr) = node {
        let left = &bin_expr.left;
        let operator = &bin_expr.operator;
        let right = &bin_expr.right;

        // Check if one side is our target binding
        let left_is_target = is_binding_expression(left, target_binding);
        let right_is_target = is_binding_expression(right, target_binding);

        if left_is_target {
            if let Expression::NumericLiteral(ref num_lit) = right {
                operations.push(ArithmeticOperation {
                    op: operator.clone(),
                    value: num_lit.value,
                    side: String::from("right"),
                });
                return;
            }
        }

        if right_is_target {
            if let Expression::NumericLiteral(ref num_lit) = left {
                operations.push(ArithmeticOperation {
                    op: operator.clone(),
                    value: num_lit.value,
                    side: String::from("left"),
                });
                return;
            }
        }

        // Recurse on both sides
        analyze_node(left, target_binding, operations);
        analyze_node(right, target_binding, operations);
    }
}
