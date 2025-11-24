/**
 * Traverse JSX
 *
 * Traverses JSX tree looking for expression containers
 */

use "./extract_expression_template.rsc" { extract_expression_template, ExpressionTemplate };

/**
 * Traverse JSX tree looking for expression containers
 */
pub fn traverse_jsx(
    node: &JSXElement,
    path: &Vec<i32>,
    expression_templates: &mut Vec<ExpressionTemplate>,
    component: &Component
) {
    // Check children for expressions
    for (i, child) in node.children.iter().enumerate() {
        match child {
            JSXChild::JSXExpressionContainer(ref expr_container) => {
                let mut child_path = path.clone();
                child_path.push(i as i32);
                let path_str = child_path.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<Str>>()
                    .join(".");

                if let Some(template) = extract_expression_template(
                    &expr_container.expression,
                    component,
                    &path_str
                ) {
                    expression_templates.push(template);
                }
            }

            JSXChild::JSXElement(ref child_element) => {
                let mut child_path = path.clone();
                child_path.push(i as i32);
                traverse_jsx(child_element, &child_path, expression_templates, component);
            }

            _ => {
                // Other child types
            }
        }
    }

    // Check attributes for expressions
    for attr in &node.opening_element.attributes {
        if let JSXAttributeOrSpread::JSXAttribute(ref jsx_attr) = attr {
            if let Some(JSXAttributeValue::JSXExpressionContainer(ref expr_container)) = &jsx_attr.value {
                let path_str = path.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<Str>>()
                    .join(".");

                if let Some(mut template) = extract_expression_template(
                    &expr_container.expression,
                    component,
                    &path_str
                ) {
                    // Add attribute name to template
                    let attr_name = match &jsx_attr.name {
                        JSXAttributeName::Identifier(ref id) => Some(id.name.clone()),
                        _ => None,
                    };

                    // Store attribute name in template (implementation depends on template type)
                    // For now, just add to list
                    expression_templates.push(template);
                }
            }
        }
    }
}
