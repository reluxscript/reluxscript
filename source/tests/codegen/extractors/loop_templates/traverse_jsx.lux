/**
 * Traverse JSX tree looking for .map() call expressions
 *
 * Recursively walks through JSX elements and their children,
 * searching for .map() calls in attributes and expression containers.
 */

use "./find_map_expressions.rsc" { find_map_expressions };
use "./extract_loop_template.rsc" { LoopTemplate };

/**
 * Traverse JSX tree and collect loop templates
 *
 * Recursively searches through:
 * - JSX element attributes (e.g., className={items.map(...)})
 * - JSX children (e.g., {items.map(...)} inside element)
 * - Nested JSX elements
 * - JSX fragments
 *
 * @param node - The JSX node to traverse
 * @param loop_templates - Mutable vector to collect found templates
 */
pub fn traverse_jsx(node: &JSXNode, loop_templates: &mut Vec<LoopTemplate>) {
    match node {
        JSXNode::JSXElement(ref jsx_element) => {
            traverse_jsx_element(jsx_element, loop_templates);
        }

        JSXNode::JSXFragment(ref jsx_fragment) => {
            traverse_jsx_fragment(jsx_fragment, loop_templates);
        }

        _ => {
            // Other node types - nothing to traverse
        }
    }
}

/**
 * Traverse a JSX element
 */
fn traverse_jsx_element(jsx_element: &JSXElement, loop_templates: &mut Vec<LoopTemplate>) {
    // Check attributes for .map() expressions
    for attr in &jsx_element.opening_element.attributes {
        if let JSXAttribute::JSXAttribute(ref jsx_attr) = attr {
            if let Some(ref value) = jsx_attr.value {
                if let JSXAttributeValue::JSXExpressionContainer(ref container) = value {
                    if let JSXExpression::Expression(ref expr) = container.expression {
                        find_map_expressions(expr, loop_templates);
                    }
                }
            }
        }
    }

    // Check children for .map() expressions
    for child in &jsx_element.children {
        match child {
            JSXChild::JSXExpressionContainer(ref container) => {
                if let JSXExpression::Expression(ref expr) = container.expression {
                    find_map_expressions(expr, loop_templates);
                }
            }

            JSXChild::JSXElement(ref child_element) => {
                traverse_jsx_element(child_element, loop_templates);
            }

            JSXChild::JSXFragment(ref fragment) => {
                traverse_jsx_fragment(fragment, loop_templates);
            }

            _ => {
                // Text nodes, etc. - skip
            }
        }
    }
}

/**
 * Traverse a JSX fragment
 */
fn traverse_jsx_fragment(jsx_fragment: &JSXFragment, loop_templates: &mut Vec<LoopTemplate>) {
    for child in &jsx_fragment.children {
        match child {
            JSXChild::JSXElement(ref child_element) => {
                traverse_jsx_element(child_element, loop_templates);
            }

            JSXChild::JSXFragment(ref child_fragment) => {
                traverse_jsx_fragment(child_fragment, loop_templates);
            }

            JSXChild::JSXExpressionContainer(ref container) => {
                if let JSXExpression::Expression(ref expr) = container.expression {
                    find_map_expressions(expr, loop_templates);
                }
            }

            _ => {
                // Text nodes, etc. - skip
            }
        }
    }
}
