/**
 * Extract prop templates from JSX attributes
 *
 * Handles:
 * - Simple bindings: checked={todo.done} → { template: "{0}", bindings: ["item.done"] }
 * - Conditionals: className={todo.done ? 'done' : 'pending'} → conditional template
 * - Template literals: className={`item-${todo.id}`} → template with placeholder
 */

use "./build_binding_path.rsc" { build_binding_path };
use "./extract_conditional_template.rsc" { extract_conditional_template };
use "./extract_template_from_template_literal.rsc" { extract_template_from_template_literal };

/**
 * Prop template structure
 */
pub struct PropTemplate {
    pub template: Str,
    pub bindings: Vec<Str>,
    pub slots: Vec<i32>,
    pub template_type: Str,
    pub conditional_templates: Option<HashMap<Str, Str>>,
    pub conditional_binding_index: Option<i32>,
}

/**
 * Extract prop templates from JSX element attributes
 *
 * Processes each attribute and attempts to extract a template from its value.
 * Skips the "key" attribute as it's handled separately.
 *
 * @param attributes - The JSX element's attributes
 * @param item_var - The loop item variable name
 * @param index_var - The loop index variable name
 * @returns HashMap mapping attribute names to their templates
 */
pub fn extract_prop_templates(
    attributes: &Vec<JSXAttribute>,
    item_var: &Str,
    index_var: Option<&Str>
) -> HashMap<Str, PropTemplate> {
    let mut templates: HashMap<Str, PropTemplate> = HashMap::new();

    for attr in attributes {
        match attr {
            JSXAttribute::JSXAttribute(ref jsx_attr) => {
                // Get the attribute name
                let prop_name = match &jsx_attr.name {
                    JSXAttributeName::Identifier(ref id) => id.name.clone(),
                    _ => continue, // Skip namespace names
                };

                // Skip key attribute (handled separately)
                if prop_name == "key" {
                    continue;
                }

                // Get the attribute value
                if let Some(ref prop_value) = jsx_attr.value {
                    match prop_value {
                        // Static string: className="static"
                        JSXAttributeValue::StringLiteral(ref str_lit) => {
                            templates.insert(prop_name, PropTemplate {
                                template: str_lit.value.clone(),
                                bindings: vec![],
                                slots: vec![],
                                template_type: "static".to_string(),
                                conditional_templates: None,
                                conditional_binding_index: None,
                            });
                        }

                        // Expression: {todo.done}, {todo.done ? 'yes' : 'no'}
                        JSXAttributeValue::JSXExpressionContainer(ref container) => {
                            if let JSXExpression::Expression(ref expr) = container.expression {
                                // Try conditional expression
                                if let Expression::ConditionalExpression(ref cond_expr) = expr {
                                    if let Some(cond_template) = extract_conditional_template(
                                        cond_expr,
                                        item_var,
                                        index_var
                                    ) {
                                        templates.insert(prop_name, PropTemplate {
                                            template: cond_template.template,
                                            bindings: cond_template.bindings,
                                            slots: cond_template.slots,
                                            template_type: cond_template.template_type,
                                            conditional_templates: Some(cond_template.conditional_templates),
                                            conditional_binding_index: Some(cond_template.conditional_binding_index),
                                        });
                                        continue;
                                    }
                                }

                                // Try template literal
                                if let Expression::TemplateLiteral(ref template_lit) = expr {
                                    if let Some(template) = extract_template_from_template_literal(
                                        template_lit,
                                        item_var,
                                        index_var
                                    ) {
                                        templates.insert(prop_name, PropTemplate {
                                            template: template.template,
                                            bindings: template.bindings,
                                            slots: template.slots,
                                            template_type: template.template_type,
                                            conditional_templates: None,
                                            conditional_binding_index: None,
                                        });
                                        continue;
                                    }
                                }

                                // Simple binding: {todo.text}, {todo.done}
                                if let Some(binding) = build_binding_path(expr, item_var, index_var) {
                                    templates.insert(prop_name, PropTemplate {
                                        template: "{0}".to_string(),
                                        bindings: vec![binding],
                                        slots: vec![0],
                                        template_type: "binding".to_string(),
                                        conditional_templates: None,
                                        conditional_binding_index: None,
                                    });
                                }
                            }
                        }

                        _ => {
                            // Other value types (JSX elements, fragments) - skip
                        }
                    }
                }
            }

            _ => {
                // Skip spread attributes
            }
        }
    }

    templates
}
