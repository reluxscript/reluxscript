/**
 * Extract loop template from .map() call expression
 *
 * Example:
 * todos.map((todo, index) => <li key={todo.id}>{todo.text}</li>)
 *
 * Extracts all the information needed to render a loop at runtime:
 * - Array binding (which array to iterate)
 * - Item/index variable names
 * - Key binding (for React keys)
 * - Item template (how to render each element)
 */

use "./extract_array_binding.rsc" { extract_array_binding };
use "./extract_jsx_from_callback.rsc" { extract_jsx_from_callback };
use "./extract_element_template.rsc" { extract_element_template, ElementTemplate };
use "./extract_key_binding.rsc" { extract_key_binding };

/**
 * Loop template structure
 */
pub struct LoopTemplate {
    pub state_key: Str,
    pub array_binding: Str,
    pub item_var: Str,
    pub index_var: Option<Str>,
    pub key_binding: Option<Str>,
    pub item_template: ElementTemplate,
}

/**
 * Extract loop template from a .map() call expression
 *
 * @param map_call_expr - The CallExpression for .map()
 * @returns LoopTemplate or None if extraction fails
 */
pub fn extract_loop_template(map_call_expr: &CallExpression) -> Option<LoopTemplate> {
    // Get array binding (the object being mapped)
    // e.g., todos.map(...) → "todos"
    if let Expression::MemberExpression(ref member_expr) = map_call_expr.callee {
        let array_binding = extract_array_binding(&member_expr.object)?;

        // Get callback function (arrow function or function expression)
        if map_call_expr.arguments.is_empty() {
            return None;
        }

        let callback_arg = &map_call_expr.arguments[0];
        let callback = match callback_arg {
            CallExpressionArgument::Expression(Expression::ArrowFunctionExpression(ref arrow)) => {
                Function::ArrowFunctionExpression(arrow.clone())
            }
            CallExpressionArgument::Expression(Expression::FunctionExpression(ref func)) => {
                Function::FunctionExpression(func.clone())
            }
            _ => {
                return None;
            }
        };

        // Get item and index parameter names
        let (item_var, index_var) = extract_param_names(&callback);

        // Get JSX element returned by callback
        let jsx_element = extract_jsx_from_callback(&callback)?;

        // Extract item template from JSX element
        let item_template = extract_element_template(
            &jsx_element,
            &item_var,
            index_var.as_ref()
        );

        // Extract key binding
        let key_binding = extract_key_binding(
            &jsx_element,
            &item_var,
            index_var.as_ref()
        );

        return Some(LoopTemplate {
            state_key: array_binding.clone(),
            array_binding,
            item_var,
            index_var,
            key_binding,
            item_template,
        });
    }

    None
}

/**
 * Extract parameter names from callback function
 */
fn extract_param_names(callback: &Function) -> (Str, Option<Str>) {
    let params = match callback {
        Function::ArrowFunctionExpression(ref arrow) => &arrow.params,
        Function::FunctionExpression(ref func) => &func.params,
        _ => return ("item".to_string(), None),
    };

    let item_var = if !params.is_empty() {
        match &params[0] {
            Pattern::Identifier(ref id) => id.name.clone(),
            _ => "item".to_string(),
        }
    } else {
        "item".to_string()
    };

    let index_var = if params.len() > 1 {
        match &params[1] {
            Pattern::Identifier(ref id) => Some(id.name.clone()),
            _ => None,
        }
    } else {
        None
    };

    (item_var, index_var)
}
