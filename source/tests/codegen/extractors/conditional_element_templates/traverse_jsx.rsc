/**
 * Traverse JSX
 *
 * Traverses JSX tree to find conditional expressions
 */

use "./extract_logical_and_element_template.rsc" { extract_logical_and_element_template };
use "./extract_ternary_element_template.rsc" { extract_ternary_element_template, ConditionalElementTemplate };

/**
 * Traverse JSX tree to find conditional expressions
 *
 * @param node - JSX node to traverse
 * @param parent_path - Hex path of parent conditional (for nesting)
 * @param conditional_templates - HashMap to store found templates
 * @param state_key_map - Map of variable names to state keys
 */
pub fn traverse_jsx(
    node: &JSXElementOrFragment,
    parent_path: Option<Str>,
    conditional_templates: &mut HashMap<Str, ConditionalElementTemplate>,
    state_key_map: &HashMap<Str, Str>
) {
    match node {
        JSXElementOrFragment::JSXElement(ref element) => {
            traverse_jsx_element(element, parent_path, conditional_templates, state_key_map);
        }

        JSXElementOrFragment::JSXFragment(ref fragment) => {
            traverse_jsx_fragment(fragment, parent_path, conditional_templates, state_key_map);
        }
    }
}

/**
 * Traverse JSX element
 */
fn traverse_jsx_element(
    node: &JSXElement,
    parent_path: Option<Str>,
    conditional_templates: &mut HashMap<Str, ConditionalElementTemplate>,
    state_key_map: &HashMap<Str, Str>
) {
    // Process children
    for child in &node.children {
        match child {
            JSXChild::JSXExpressionContainer(ref expr_container) => {
                let expr = &expr_container.expression;

                // Logical AND: {condition && <Element />}
                if let Expression::LogicalExpression(ref logical) = expr {
                    if logical.operator == "&&" {
                        if let Some(template) = extract_logical_and_element_template(
                            logical,
                            node,
                            parent_path.clone(),
                            state_key_map
                        ) {
                            if let Some(path) = expr_container.get_custom_property("__minimactPath") {
                                // Recursively find nested conditionals inside this template
                                traverse_jsx_expression(&logical.right, Some(path.clone()), conditional_templates, state_key_map);

                                conditional_templates.insert(path, template);
                            }
                        }
                    }
                }

                // Ternary: {condition ? <A /> : <B />}
                if let Expression::ConditionalExpression(ref conditional) = expr {
                    if let Some(template) = extract_ternary_element_template(
                        conditional,
                        node,
                        parent_path.clone(),
                        state_key_map
                    ) {
                        if let Some(path) = expr_container.get_custom_property("__minimactPath") {
                            // Recursively find nested conditionals in both branches
                            traverse_jsx_expression(&conditional.consequent, Some(path.clone()), conditional_templates, state_key_map);
                            traverse_jsx_expression(&conditional.alternate, Some(path.clone()), conditional_templates, state_key_map);

                            conditional_templates.insert(path, template);
                        }
                    }
                }
            }

            JSXChild::JSXElement(ref child_element) => {
                traverse_jsx_element(child_element, parent_path.clone(), conditional_templates, state_key_map);
            }

            JSXChild::JSXFragment(ref child_fragment) => {
                traverse_jsx_fragment(child_fragment, parent_path.clone(), conditional_templates, state_key_map);
            }

            _ => {
                // Other child types (text, etc.)
            }
        }
    }
}

/**
 * Traverse JSX fragment
 */
fn traverse_jsx_fragment(
    node: &JSXFragment,
    parent_path: Option<Str>,
    conditional_templates: &mut HashMap<Str, ConditionalElementTemplate>,
    state_key_map: &HashMap<Str, Str>
) {
    for child in &node.children {
        match child {
            JSXChild::JSXElement(ref element) => {
                traverse_jsx_element(element, parent_path.clone(), conditional_templates, state_key_map);
            }

            JSXChild::JSXFragment(ref fragment) => {
                traverse_jsx_fragment(fragment, parent_path.clone(), conditional_templates, state_key_map);
            }

            _ => {}
        }
    }
}

/**
 * Helper: Traverse expression that might contain JSX
 */
fn traverse_jsx_expression(
    expr: &Expression,
    parent_path: Option<Str>,
    conditional_templates: &mut HashMap<Str, ConditionalElementTemplate>,
    state_key_map: &HashMap<Str, Str>
) {
    match expr {
        Expression::JSXElement(ref element) => {
            traverse_jsx_element(element, parent_path, conditional_templates, state_key_map);
        }

        Expression::JSXFragment(ref fragment) => {
            traverse_jsx_fragment(fragment, parent_path, conditional_templates, state_key_map);
        }

        _ => {
            // Not JSX
        }
    }
}
