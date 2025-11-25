/**
 * Extract conditional template from ternary expression
 *
 * Example: todo.done ? 'completed' : 'pending'
 * →
 * {
 *   template: "{0}",
 *   bindings: ["item.done"],
 *   conditional_templates: { "true": "completed", "false": "pending" },
 *   conditional_binding_index: 0
 * }
 */

use "./build_binding_path.rsc" { build_binding_path };
use "./extract_literal_value.rsc" { extract_literal_value };

/**
 * Template result for conditional expressions
 */
pub struct ConditionalTemplate {
    pub template: Str,
    pub bindings: Vec<Str>,
    pub slots: Vec<i32>,
    pub conditional_templates: HashMap<Str, Str>,
    pub conditional_binding_index: i32,
    pub template_type: Str,
}

/**
 * Extract conditional template from a ternary expression
 *
 * Handles: test ? consequent : alternate
 * Only works if consequent and alternate are literal values.
 *
 * @param conditional_expr - The conditional expression
 * @param item_var - The loop item variable name
 * @param index_var - The loop index variable name
 * @returns ConditionalTemplate or None if not template-able
 */
pub fn extract_conditional_template(
    conditional_expr: &ConditionalExpression,
    item_var: &Str,
    index_var: Option<&Str>
) -> Option<ConditionalTemplate> {
    let test = &conditional_expr.test;
    let consequent = &conditional_expr.consequent;
    let alternate = &conditional_expr.alternate;

    // Extract binding from test expression
    let binding = build_binding_path(test, item_var, index_var)?;

    // Extract literal values from consequent and alternate
    let true_value = extract_literal_value(consequent)?;
    let false_value = extract_literal_value(alternate)?;

    // Build conditional templates map
    let mut conditional_templates = HashMap::new();
    conditional_templates.insert("true".to_string(), true_value);
    conditional_templates.insert("false".to_string(), false_value);

    Some(ConditionalTemplate {
        template: "{0}".to_string(),
        bindings: vec![binding],
        slots: vec![0],
        conditional_templates,
        conditional_binding_index: 0,
        template_type: "conditional".to_string(),
    })
}
