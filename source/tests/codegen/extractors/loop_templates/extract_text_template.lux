/**
 * Extract text template from expression
 *
 * Handles:
 * - Simple binding: {todo.text} → { template: "{0}", bindings: ["item.text"] }
 * - Conditional: {todo.done ? '✓' : '○'} → conditional template
 * - Binary expressions: {todo.count + 1} → expression template
 * - Method calls: {todo.text.toUpperCase()} → expression template
 * - Logical expressions: {todo.date || 'N/A'} → expression template
 */

use "./build_binding_path.rsc" { build_binding_path };
use "./extract_conditional_template.rsc" { extract_conditional_template, ConditionalTemplate };
use "./extract_template_from_template_literal.rsc" {
    extract_template_from_template_literal,
    TemplateLiteralResult
};

/**
 * Text template result (can be simple binding, conditional, or template literal)
 */
pub enum TextTemplate {
    Simple {
        template: Str,
        bindings: Vec<Str>,
        slots: Vec<i32>,
    },
    Conditional(ConditionalTemplate),
    TemplateLiteral(TemplateLiteralResult),
}

/**
 * Extract text template from an expression within JSX
 *
 * @param expr - The expression to extract from
 * @param item_var - The loop item variable name
 * @param index_var - The loop index variable name
 * @returns TextTemplate or None if not template-able
 */
pub fn extract_text_template(
    expr: &Expression,
    item_var: &Str,
    index_var: Option<&Str>
) -> Option<TextTemplate> {
    match expr {
        // Template literal: {`${user.firstName} ${user.lastName}`}
        Expression::TemplateLiteral(ref template_lit) => {
            if let Some(result) = extract_template_from_template_literal(
                template_lit,
                item_var,
                index_var
            ) {
                return Some(TextTemplate::TemplateLiteral(result));
            }
        }

        // Conditional expression: {todo.done ? '✓' : '○'}
        Expression::ConditionalExpression(ref cond_expr) => {
            if let Some(result) = extract_conditional_template(
                cond_expr,
                item_var,
                index_var
            ) {
                return Some(TextTemplate::Conditional(result));
            }
        }

        _ => {
            // Try to extract binding (handles simple, binary, method calls, etc.)
            if let Some(binding) = build_binding_path(expr, item_var, index_var) {
                return Some(TextTemplate::Simple {
                    template: "{0}".to_string(),
                    bindings: vec![binding],
                    slots: vec![0],
                });
            }
        }
    }

    // No binding found
    None
}
