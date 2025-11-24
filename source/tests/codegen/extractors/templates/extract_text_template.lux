/**
 * Extract Text Template
 *
 * Extracts template from mixed text/expression children in JSX
 */

use "./extract_binding.rsc" { extract_binding, Binding };
use "./extract_template_literal.rsc" { extract_template_literal };

/**
 * Conditional template values
 */
pub struct ConditionalTemplates {
    pub true_value: Str,
    pub false_value: Str,
}

/**
 * Transform metadata
 */
pub struct TransformMetadata {
    pub method: Str,
    pub args: Vec<Str>,
}

/**
 * Text template result
 */
pub struct TextTemplate {
    pub template: Str,
    pub bindings: Vec<Str>,
    pub slots: Vec<i32>,
    pub path: Str,
    pub template_type: Str,
    pub conditional_templates: Option<ConditionalTemplates>,
    pub transform: Option<TransformMetadata>,
    pub nullable: bool,
}

/**
 * Extract template from mixed text/expression children
 * Example: <h1>Count: {count}</h1> → "Count: {0}"
 */
pub fn extract_text_template(
    children: &Vec<JSXChild>,
    current_path: &Str,
    text_index: i32,
    component: &Component
) -> Option<TextTemplate> {
    let mut template_str = String::new();
    let mut bindings = vec![];
    let mut slots = vec![];
    let mut param_index = 0;
    let mut has_expressions = false;
    let mut conditional_templates: Option<ConditionalTemplates> = None;
    let mut transform_metadata: Option<TransformMetadata> = None;
    let mut nullable_metadata = false;

    for child in children {
        match child {
            JSXChild::JSXText(ref jsx_text) => {
                template_str.push_str(&jsx_text.value);
            }

            JSXChild::JSXExpressionContainer(ref expr_container) => {
                has_expressions = true;

                // Special case: Template literal inside JSX expression container
                // Example: {`${(discount * 100).toFixed(0)}%`}
                if let Expression::TemplateLiteral(ref template_lit) = &expr_container.expression {
                    let template_result = extract_template_literal(template_lit, component);

                    // Merge the template literal's content into the current template
                    template_str.push_str(&template_result.template);

                    // Add the template literal's bindings
                    for binding in template_result.bindings {
                        bindings.push(binding);
                    }

                    // Store transforms if present
                    if !template_result.transforms.is_empty() {
                        let first_transform = &template_result.transforms[0];
                        transform_metadata = Some(TransformMetadata {
                            method: first_transform.method.clone(),
                            args: first_transform.args.clone(),
                        });
                    }

                    param_index += 1;
                    continue; // Skip normal binding extraction
                }

                // Extract binding from expression
                if let Some(binding_result) = extract_binding(&expr_container.expression, component) {
                    match binding_result {
                        Binding::Conditional(cond) => {
                            // Conditional binding (ternary)
                            slots.push(template_str.len() as i32);
                            template_str.push_str(&format!("{{{}}}", param_index));
                            bindings.push(cond.conditional.clone());

                            // Store conditional template values
                            conditional_templates = Some(ConditionalTemplates {
                                true_value: cond.true_value,
                                false_value: cond.false_value,
                            });

                            param_index += 1;
                        }

                        Binding::MethodCall(method) => {
                            // Transform binding (method call)
                            slots.push(template_str.len() as i32);
                            template_str.push_str(&format!("{{{}}}", param_index));
                            bindings.push(method.binding.clone());

                            // Store transform metadata
                            transform_metadata = Some(TransformMetadata {
                                method: method.transform,
                                args: method.args,
                            });

                            param_index += 1;
                        }

                        Binding::OptionalChain(opt) => {
                            // Nullable binding (optional chaining)
                            slots.push(template_str.len() as i32);
                            template_str.push_str(&format!("{{{}}}", param_index));
                            bindings.push(opt.binding);

                            // Mark as nullable
                            nullable_metadata = true;

                            param_index += 1;
                        }

                        Binding::Simple(s) => {
                            // Simple binding (string)
                            slots.push(template_str.len() as i32);
                            template_str.push_str(&format!("{{{}}}", param_index));
                            bindings.push(s);
                            param_index += 1;
                        }
                    }
                } else {
                    // Complex expression - can't template it
                    template_str.push_str(&format!("{{{}}}", param_index));
                    bindings.push(String::from("__complex__"));
                    param_index += 1;
                }
            }

            _ => {
                // Other JSX child types (elements, fragments, etc.)
                // Skip for now
            }
        }
    }

    // Clean up whitespace
    template_str = template_str.trim().to_string();

    if !has_expressions {
        return None;
    }

    // Determine template type
    let template_type = if conditional_templates.is_some() {
        "conditional"
    } else if transform_metadata.is_some() {
        "transform"
    } else if nullable_metadata {
        "nullable"
    } else {
        "dynamic"
    };

    Some(TextTemplate {
        template: template_str,
        bindings,
        slots,
        path: current_path.clone(),
        template_type: String::from(template_type),
        conditional_templates,
        transform: transform_metadata,
        nullable: nullable_metadata,
    })
}
