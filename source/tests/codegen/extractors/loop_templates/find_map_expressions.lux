/**
 * Find .map() call expressions recursively
 *
 * Searches through expressions to find all `.map()` calls and extract
 * loop templates from them.
 */

use "./extract_loop_template.rsc" { extract_loop_template, LoopTemplate };

/**
 * Find all .map() call expressions in an expression tree
 *
 * Recursively searches for `.map()` calls and extracts loop templates.
 * Handles:
 * - Direct .map() calls: items.map(...)
 * - Chained operations: items.filter(...).map(...)
 * - Wrapped in logical/conditional expressions
 *
 * @param expr - The expression to search
 * @param loop_templates - Mutable vector to collect found loop templates
 */
pub fn find_map_expressions(expr: &Expression, loop_templates: &mut Vec<LoopTemplate>) {
    match expr {
        // Direct .map() call: items.map(...)
        Expression::CallExpression(ref call_expr) => {
            // Check if this is a .map() call
            if let Expression::MemberExpression(ref member_expr) = call_expr.callee {
                if let Expression::Identifier(ref prop) = member_expr.property {
                    if prop.name == "map" {
                        // Found a .map() call - extract template
                        if let Some(loop_template) = extract_loop_template(call_expr) {
                            loop_templates.push(loop_template);
                        }
                    }
                }

                // Also check for chained operations: items.filter(...).map(...)
                find_map_expressions(&member_expr.object, loop_templates);
            }
        }

        // Wrapped in logical expression: condition && items.map(...)
        Expression::LogicalExpression(ref log_expr) => {
            find_map_expressions(&log_expr.left, loop_templates);
            find_map_expressions(&log_expr.right, loop_templates);
        }

        // Wrapped in conditional: test ? items.map(...) : other.map(...)
        Expression::ConditionalExpression(ref cond_expr) => {
            find_map_expressions(&cond_expr.test, loop_templates);
            find_map_expressions(&cond_expr.consequent, loop_templates);
            find_map_expressions(&cond_expr.alternate, loop_templates);
        }

        _ => {
            // Other expression types - no .map() calls to find
        }
    }
}
