/**
 * Extract key binding from JSX element
 *
 * Example: <li key={todo.id}> → "item.id"
 */

use "./build_binding_path.rsc" { build_binding_path };

/**
 * Extract the key binding from a JSX element's attributes
 *
 * Looks for the "key" attribute and extracts the binding path from it.
 *
 * @param jsx_element - The JSX element to extract from
 * @param item_var - The loop item variable name (e.g., "todo")
 * @param index_var - The loop index variable name (e.g., "i")
 * @returns The key binding path or None if no key or static key
 */
pub fn extract_key_binding(
    jsx_element: &JSXElement,
    item_var: &Str,
    index_var: Option<&Str>
) -> Option<Str> {
    // Find the "key" attribute
    for attr in &jsx_element.opening_element.attributes {
        if let JSXAttribute::JSXAttribute(ref jsx_attr) = attr {
            // Check if this is the "key" attribute
            if let JSXAttributeName::Identifier(ref name) = jsx_attr.name {
                if name.name == "key" {
                    // Found the key attribute
                    if let Some(ref value) = jsx_attr.value {
                        match value {
                            JSXAttributeValue::JSXExpressionContainer(ref container) => {
                                // Extract binding from the expression
                                if let JSXExpression::Expression(ref expr) = container.expression {
                                    return build_binding_path(expr, item_var, index_var);
                                }
                            }

                            JSXAttributeValue::StringLiteral(_) => {
                                // Static key (not based on item data)
                                return None;
                            }

                            _ => {
                                return None;
                            }
                        }
                    }
                }
            }
        }
    }

    None
}
