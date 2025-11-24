/**
 * Build binding path from expression relative to item variable
 *
 * Examples:
 * - todo → null (just the item itself)
 * - todo.text → "item.text"
 * - todo.author.name → "item.author.name"
 * - index → "index"
 * - todo.priority + 1 → "__expr__:item.priority"
 * - todo.text.toUpperCase() → "__expr__:item.text"
 * - index * 2 + 1 → "__expr__:index"
 */

use "./build_member_expression_path.rsc" { build_member_expression_path };
use "./extract_loop_expressions.rsc" {
    extract_loop_binary_expression,
    extract_loop_logical_expression,
    extract_loop_unary_expression,
    extract_loop_call_expression
};

/**
 * Build a binding path from an expression within a loop context
 *
 * Converts expressions to template-able paths by identifying which parts
 * of the expression depend on the loop item or index.
 *
 * @param expr - The expression to build a path from
 * @param item_var - The loop item variable name (e.g., "todo")
 * @param index_var - The loop index variable name (e.g., "i")
 * @returns Binding path string or None
 */
pub fn build_binding_path(
    expr: &Expression,
    item_var: &Str,
    index_var: Option<&Str>
) -> Option<Str> {
    match expr {
        Expression::Identifier(ref id) => {
            // Just the item variable itself
            if id.name == *item_var {
                // Can't template the entire item object
                return None;
            }

            // Index variable
            if id.name == "index" || (index_var.is_some() && id.name == *index_var.unwrap()) {
                return Some("index".to_string());
            }

            // Other identifier (likely a closure variable)
            None
        }

        Expression::MemberExpression(_) => {
            if let Some(path) = build_member_expression_path(expr) {
                let item_prefix = format!("{}.", item_var);
                if path.starts_with(&item_prefix) {
                    // Replace item variable with "item" prefix
                    let item_path = format!("item{}", &path[item_var.len()..]);
                    return Some(item_path);
                }
            }
            None
        }

        Expression::BinaryExpression(ref bin_expr) => {
            // Handle binary expressions: todo.priority + 1, price * quantity, etc.
            extract_loop_binary_expression(bin_expr, item_var, index_var)
        }

        Expression::LogicalExpression(ref log_expr) => {
            // Handle logical expressions: todo.dueDate || 'No due date'
            extract_loop_logical_expression(log_expr, item_var, index_var)
        }

        Expression::UnaryExpression(ref unary_expr) => {
            // Handle unary expressions: !todo.completed, -value
            extract_loop_unary_expression(unary_expr, item_var, index_var)
        }

        Expression::CallExpression(ref call_expr) => {
            // Handle call expressions: todo.text.toUpperCase(), array.concat()
            extract_loop_call_expression(call_expr, item_var, index_var)
        }

        _ => None,
    }
}
