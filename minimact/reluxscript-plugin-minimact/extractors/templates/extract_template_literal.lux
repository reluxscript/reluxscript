/**
 * Extract Template Literal
 *
 * Extracts binding information from template literals
 */

use "./extract_binding_shared.rsc" { extract_binding_shared, BindingResult };

/**
 * Template literal transform metadata
 */
pub struct TemplateLiteralTransform {
    pub slot_index: i32,
    pub method: Str,
    pub args: Vec<Str>,
}

/**
 * Template literal result
 */
pub struct TemplateLiteralResult {
    pub template: Str,
    pub bindings: Vec<Str>,
    pub slots: Vec<i32>,
    pub template_type: Str,
    pub transforms: Vec<TemplateLiteralTransform>,
    pub conditionals: Vec<String>, // Placeholder for future conditional support
}

/**
 * Extract template literal
 * Handles: `Hello ${name}, you are ${age} years old`
 */
pub fn extract_template_literal(node: &TemplateLiteral, component: &Component) -> TemplateLiteralResult {
    let mut template_str = String::new();
    let mut bindings = vec![];
    let mut slots = vec![];
    let mut transforms = vec![];
    let conditionals = vec![];

    // Process quasis (static parts) and expressions (dynamic parts)
    for i in 0..node.quasis.len() {
        let quasi = &node.quasis[i];
        template_str.push_str(&quasi.value.raw);

        // Add expression if it exists at this position
        if i < node.expressions.len() {
            let expr = &node.expressions[i];
            slots.push(template_str.len() as i32);
            template_str.push_str(&format!("{{{}}}", i));

            if let Some(binding_result) = extract_binding_shared(expr) {
                match binding_result {
                    BindingResult::Transform(transform) => {
                        bindings.push(transform.binding.clone());
                        transforms.push(TemplateLiteralTransform {
                            slot_index: i as i32,
                            method: transform.transform,
                            args: transform.args,
                        });
                    }

                    BindingResult::Simple(s) => {
                        bindings.push(s);
                    }
                }
            } else {
                bindings.push(String::from("__complex__"));
            }
        }
    }

    TemplateLiteralResult {
        template: template_str,
        bindings,
        slots,
        template_type: String::from("attribute"),
        transforms,
        conditionals,
    }
}
