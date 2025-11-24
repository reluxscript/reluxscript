/**
 * Extract identifiers from expression, converting item references to "item" prefix
 */

use "./build_member_expression_path.rsc" { build_member_expression_path };

/**
 * Extract all identifiers from an expression within a loop context
 *
 * Recursively traverses the expression tree and collects identifier names,
 * converting item variable references to use the "item" prefix.
 *
 * @param expr - The expression to extract from
 * @param result - Mutable vector to collect identifiers
 * @param item_var - The loop item variable name (e.g., "todo")
 * @param index_var - The loop index variable name (e.g., "i")
 */
pub fn extract_loop_identifiers(
    expr: &Expression,
    result: &mut Vec<Str>,
    item_var: &Str,
    index_var: Option<&Str>
) {
    match expr {
        Expression::Identifier(ref id) => {
            // Skip raw item variable
            if id.name == *item_var {
                return;
            }

            // Check for index variable
            if id.name == "index" || (index_var.is_some() && id.name == *index_var.unwrap()) {
                result.push("index".to_string());
            } else {
                result.push(id.name.clone());
            }
        }

        Expression::BinaryExpression(ref bin_expr) => {
            extract_loop_identifiers(&bin_expr.left, result, item_var, index_var);
            extract_loop_identifiers(&bin_expr.right, result, item_var, index_var);
        }

        Expression::LogicalExpression(ref log_expr) => {
            extract_loop_identifiers(&log_expr.left, result, item_var, index_var);
            extract_loop_identifiers(&log_expr.right, result, item_var, index_var);
        }

        Expression::UnaryExpression(ref unary_expr) => {
            extract_loop_identifiers(&unary_expr.argument, result, item_var, index_var);
        }

        Expression::MemberExpression(ref member_expr) => {
            if let Some(path) = build_member_expression_path(expr) {
                // Check if it starts with the item variable
                let item_prefix = format!("{}.", item_var);
                if path.starts_with(&item_prefix) {
                    // Replace item variable with "item" prefix
                    let item_path = format!("item{}", &path[item_var.len()..]);
                    result.push(item_path);
                } else {
                    result.push(path);
                }
            } else {
                // Complex member expression (e.g., (a + b).toFixed())
                // Extract from both object and property
                extract_loop_identifiers(&member_expr.object, result, item_var, index_var);
                if let Expression::Identifier(ref prop) = member_expr.property {
                    result.push(prop.name.clone());
                }
            }
        }

        Expression::CallExpression(ref call_expr) => {
            // Extract from callee
            extract_loop_identifiers(&call_expr.callee, result, item_var, index_var);

            // Extract from arguments
            for arg in &call_expr.arguments {
                if let CallExpressionArgument::Expression(ref arg_expr) = arg {
                    extract_loop_identifiers(arg_expr, result, item_var, index_var);
                }
            }
        }

        _ => {
            // Other expression types - no identifiers to extract
        }
    }
}
