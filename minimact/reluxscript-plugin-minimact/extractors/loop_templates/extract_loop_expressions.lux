/**
 * Extract bindings from various expression types in loop contexts
 *
 * Examples:
 * - Binary: todo.priority + 1, price * quantity, index * 2 + 1
 * - Logical: todo.dueDate || 'No due date', condition && value
 * - Unary: !todo.completed, -value
 * - Call: todo.text.toUpperCase(), todo.text.substring(0, 10)
 */

use "./extract_loop_identifiers.rsc" { extract_loop_identifiers };

/**
 * Extract binding from binary expression in loop
 *
 * Examples: todo.priority + 1, price * quantity, index * 2 + 1
 *
 * @returns Expression binding with __expr__ prefix, or None
 */
pub fn extract_loop_binary_expression(
    expr: &BinaryExpression,
    item_var: &Str,
    index_var: Option<&Str>
) -> Option<Str> {
    let mut identifiers: Vec<Str> = vec![];
    let expr_node = Expression::BinaryExpression(expr.clone());
    extract_loop_identifiers(&expr_node, &mut identifiers, item_var, index_var);

    if identifiers.is_empty() {
        return None;
    }

    // Use __expr__ prefix to indicate this is a computed expression
    Some(format!("__expr__:{}", identifiers.join(",")))
}

/**
 * Extract binding from logical expression in loop
 *
 * Examples: todo.dueDate || 'No due date', condition && value
 *
 * @returns Expression binding with __expr__ prefix, or None
 */
pub fn extract_loop_logical_expression(
    expr: &LogicalExpression,
    item_var: &Str,
    index_var: Option<&Str>
) -> Option<Str> {
    let mut identifiers: Vec<Str> = vec![];
    let expr_node = Expression::LogicalExpression(expr.clone());
    extract_loop_identifiers(&expr_node, &mut identifiers, item_var, index_var);

    if identifiers.is_empty() {
        return None;
    }

    // Use __expr__ prefix to indicate this is a computed expression
    Some(format!("__expr__:{}", identifiers.join(",")))
}

/**
 * Extract binding from unary expression in loop
 *
 * Examples: !todo.completed, -value
 *
 * @returns Expression binding with __expr__ prefix, or None
 */
pub fn extract_loop_unary_expression(
    expr: &UnaryExpression,
    item_var: &Str,
    index_var: Option<&Str>
) -> Option<Str> {
    let mut identifiers: Vec<Str> = vec![];
    let expr_node = Expression::UnaryExpression(expr.clone());
    extract_loop_identifiers(&expr_node, &mut identifiers, item_var, index_var);

    if identifiers.is_empty() {
        return None;
    }

    // Use __expr__ prefix to indicate this is a computed expression
    Some(format!("__expr__:{}", identifiers.join(",")))
}

/**
 * Extract binding from call expression in loop
 *
 * Examples: todo.text.toUpperCase(), todo.text.substring(0, 10)
 *
 * @returns Expression binding with __expr__ prefix, or None
 */
pub fn extract_loop_call_expression(
    expr: &CallExpression,
    item_var: &Str,
    index_var: Option<&Str>
) -> Option<Str> {
    let mut identifiers: Vec<Str> = vec![];
    let expr_node = Expression::CallExpression(expr.clone());
    extract_loop_identifiers(&expr_node, &mut identifiers, item_var, index_var);

    if identifiers.is_empty() {
        return None;
    }

    // Use __expr__ prefix to indicate this is a computed expression
    Some(format!("__expr__:{}", identifiers.join(",")))
}
