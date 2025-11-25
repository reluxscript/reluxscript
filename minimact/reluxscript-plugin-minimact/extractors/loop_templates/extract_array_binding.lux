/**
 * Extract array binding from member expression
 *
 * Examples:
 * - todos.map(...) → "todos"
 * - this.state.items.map(...) → "items"
 * - [...todos].map(...) → "todos"
 */

/**
 * Extract the array variable name from various expression forms
 *
 * Handles:
 * - Direct identifiers: todos → "todos"
 * - Member expressions: this.state.items → "items" (last property)
 * - Call expressions: todos.reverse() → "todos" (recursive)
 * - Array expressions with spread: [...todos] → "todos"
 *
 * @param expr - The expression to extract from
 * @returns The array binding name or None
 */
pub fn extract_array_binding(expr: &Expression) -> Option<Str> {
    match expr {
        // Direct identifier: todos
        Expression::Identifier(ref id) => {
            Some(id.name.clone())
        }

        // Member expression: this.state.items → "items"
        Expression::MemberExpression(ref member_expr) => {
            // Get the last property name
            if let Expression::Identifier(ref prop) = member_expr.property {
                Some(prop.name.clone())
            } else {
                None
            }
        }

        // Call expression: todos.reverse(), todos.slice()
        Expression::CallExpression(ref call_expr) => {
            // Handle array methods - recurse on the object
            if let Expression::MemberExpression(ref member_expr) = call_expr.callee {
                extract_array_binding(&member_expr.object)
            } else {
                None
            }
        }

        // Array expression with spread: [...todos]
        Expression::ArrayExpression(ref arr_expr) => {
            if !arr_expr.elements.is_empty() {
                if let Some(ref first_elem) = arr_expr.elements.get(0) {
                    if let ArrayExpressionElement::SpreadElement(ref spread) = first_elem {
                        return extract_array_binding(&spread.argument);
                    }
                }
            }
            None
        }

        _ => None,
    }
}
