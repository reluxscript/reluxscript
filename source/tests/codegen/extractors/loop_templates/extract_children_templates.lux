/**
 * Extract children templates from JSX children
 *
 * Returns array of templates (text or element)
 * Note: This has a circular dependency with extract_element_template.rsc
 */

use "./extract_text_template.rsc" { extract_text_template, TextTemplate };
use "./extract_element_template.rsc" { extract_element_template, ElementTemplate };

/**
 * Child template (can be text or element)
 */
pub enum ChildTemplate {
    Text {
        template: Str,
        bindings: Vec<Str>,
        slots: Vec<i32>,
    },
    TextConditional(TextTemplate),
    Element(ElementTemplate),
}

/**
 * Extract children templates from JSX children array
 *
 * Processes each child and extracts templates for:
 * - Static text
 * - Expression containers (bindings, conditionals, etc.)
 * - Nested JSX elements
 *
 * @param children - The JSX element's children
 * @param item_var - The loop item variable name
 * @param index_var - The loop index variable name
 * @returns Vector of child templates
 */
pub fn extract_children_templates(
    children: &Vec<JSXChild>,
    item_var: &Str,
    index_var: Option<&Str>
) -> Vec<ChildTemplate> {
    let mut templates: Vec<ChildTemplate> = vec![];

    for child in children {
        match child {
            // Static text: <li>Static text</li>
            JSXChild::JSXText(ref jsx_text) => {
                let text = jsx_text.value.trim();
                if !text.is_empty() {
                    templates.push(ChildTemplate::Text {
                        template: text.to_string(),
                        bindings: vec![],
                        slots: vec![],
                    });
                }
            }

            // Expression: <li>{todo.text}</li>
            JSXChild::JSXExpressionContainer(ref container) => {
                if let JSXExpression::Expression(ref expr) = container.expression {
                    if let Some(text_template) = extract_text_template(
                        expr,
                        item_var,
                        index_var
                    ) {
                        match text_template {
                            TextTemplate::Simple { template, bindings, slots } => {
                                templates.push(ChildTemplate::Text {
                                    template,
                                    bindings,
                                    slots,
                                });
                            }
                            TextTemplate::Conditional(_) | TextTemplate::TemplateLiteral(_) => {
                                templates.push(ChildTemplate::TextConditional(text_template));
                            }
                        }
                    }
                }
            }

            // Nested element: <li><span>{todo.text}</span></li>
            JSXChild::JSXElement(ref jsx_element) => {
                let element_template = extract_element_template(
                    jsx_element,
                    item_var,
                    index_var
                );
                templates.push(ChildTemplate::Element(element_template));
            }

            _ => {
                // Other child types (JSX fragments, etc.) - skip for now
            }
        }
    }

    templates
}
