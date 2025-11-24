/**
 * Extract Style Object Template
 *
 * Extracts template information from style object expressions
 */

use "./extract_binding_shared.rsc" { extract_binding_shared, BindingResult };
use "./style_helpers.rsc" { camel_to_kebab, convert_style_value };

/**
 * Style template result
 */
pub struct StyleTemplate {
    pub template: Str,
    pub bindings: Vec<Str>,
    pub slots: Vec<i32>,
    pub path: Str,
    pub attribute: Str,
    pub template_type: Str,
}

/**
 * Extract template from style object
 * Handles: { fontSize: '32px', opacity: isVisible ? 1 : 0.5 }
 */
pub fn extract_style_object_template(
    object_expr: &ObjectExpression,
    tag_name: &Str,
    element_index: i32,
    parent_path: &Str,
    current_path: &Str,
    component: &Component
) -> StyleTemplate {
    let mut has_bindings = false;
    let mut css_properties = vec![];
    let mut bindings = vec![];
    let mut slots = vec![];
    let mut slot_index = 0;

    // Check each property for dynamic values
    for prop in &object_expr.properties {
        if let ObjectProperty::Property(ref obj_prop) = prop {
            if !obj_prop.computed {
                // Get property key
                let key = match &obj_prop.key {
                    Expression::Identifier(ref id) => id.name.clone(),
                    Expression::StringLiteral(ref str_lit) => str_lit.value.clone(),
                    _ => continue,
                };

                let css_key = camel_to_kebab(&key);
                let value = &obj_prop.value;

                // Check if value is dynamic (expression, conditional, etc.)
                let is_dynamic = matches!(value,
                    Expression::ConditionalExpression(_) |
                    Expression::Identifier(_) |
                    Expression::MemberExpression(_)
                );

                if is_dynamic {
                    // Dynamic value - extract binding
                    has_bindings = true;

                    if let Some(binding_result) = extract_binding_shared(value) {
                        let binding_str = match binding_result {
                            BindingResult::Simple(s) => s,
                            BindingResult::Transform(t) => t.binding,
                        };

                        bindings.push(binding_str);
                        css_properties.push(format!("{}: {{{}}}", css_key, slot_index));

                        // Calculate slot position
                        let joined = css_properties.join("; ");
                        if let Some(last_brace) = joined.rfind('{') {
                            slots.push(last_brace as i32);
                        }

                        slot_index += 1;
                    } else {
                        // Complex expression - fall back to static
                        let css_value = convert_style_value(value);
                        css_properties.push(format!("{}: {}", css_key, css_value));
                    }
                } else {
                    // Static value
                    let css_value = convert_style_value(value);
                    css_properties.push(format!("{}: {}", css_key, css_value));
                }
            }
        }
    }

    let css_string = css_properties.join("; ");
    let template_type = if has_bindings {
        "attribute-dynamic"
    } else {
        "attribute-static"
    };

    StyleTemplate {
        template: css_string,
        bindings,
        slots,
        path: current_path.clone(),
        attribute: String::from("style"),
        template_type: String::from(template_type),
    }
}
