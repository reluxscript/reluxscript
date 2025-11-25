/**
 * Extract template from template literal
 *
 * Example: `item-${todo.id}`
 * →
 * {
 *   template: "item-{0}",
 *   bindings: ["item.id"],
 *   slots: [5]
 * }
 */

use "./build_binding_path.rsc" { build_binding_path };

/**
 * Template literal result
 */
pub struct TemplateLiteralResult {
    pub template: Str,
    pub bindings: Vec<Str>,
    pub slots: Vec<i32>,
    pub template_type: Str,
}

/**
 * Extract template from a JavaScript template literal
 *
 * Converts template literals like `item-${todo.id}` to a format
 * suitable for runtime template rendering.
 *
 * @param template_literal - The template literal expression
 * @param item_var - The loop item variable name
 * @param index_var - The loop index variable name
 * @returns TemplateLiteralResult or None if contains complex expressions
 */
pub fn extract_template_from_template_literal(
    template_literal: &TemplateLiteral,
    item_var: &Str,
    index_var: Option<&Str>
) -> Option<TemplateLiteralResult> {
    let mut template_str = String::new();
    let mut bindings: Vec<Str> = vec![];
    let mut slots: Vec<i32> = vec![];

    // Process each quasi (template string part) and expression
    for i in 0..template_literal.quasis.len() {
        let quasi = &template_literal.quasis[i];
        template_str.push_str(&quasi.value.raw);

        // Check if there's a corresponding expression
        if i < template_literal.expressions.len() {
            let expr = &template_literal.expressions[i];

            // Try to extract binding from expression
            if let Some(binding) = build_binding_path(expr, item_var, index_var) {
                // Record the slot position (current length of template string)
                slots.push(template_str.len() as i32);

                // Add placeholder {N} where N is the binding index
                template_str.push_str(&format!("{{{}}}", bindings.len()));

                // Add binding
                bindings.push(binding);
            } else {
                // Complex expression - can't template it
                return None;
            }
        }
    }

    Some(TemplateLiteralResult {
        template: template_str,
        bindings,
        slots,
        template_type: "template-literal".to_string(),
    })
}
