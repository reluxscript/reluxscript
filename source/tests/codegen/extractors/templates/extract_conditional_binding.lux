/**
 * Extract Conditional Binding
 *
 * Extracts conditional binding from ternary expressions
 */

use "./extract_literal_value.rsc" { extract_literal_value };

/**
 * Conditional binding result
 */
pub struct ConditionalBinding {
    pub conditional: Str,
    pub true_value: Str,
    pub false_value: Str,
}

/**
 * Extract conditional binding from ternary expression
 * Returns object with test identifier and consequent/alternate values
 *
 * Example: isExpanded ? 'Hide' : 'Show'
 * Returns: { conditional: 'isExpanded', trueValue: 'Hide', falseValue: 'Show' }
 */
pub fn extract_conditional_binding(expr: &ConditionalExpression) -> Option<ConditionalBinding> {
    // Check if test is a simple identifier
    if let Expression::Identifier(ref test_id) = &expr.test {
        // Check if consequent and alternate are literals
        let true_value = extract_literal_value(&expr.consequent)?;
        let false_value = extract_literal_value(&expr.alternate)?;

        Some(ConditionalBinding {
            conditional: test_id.name.clone(),
            true_value,
            false_value,
        })
    } else {
        // Complex test condition - mark as complex
        None
    }
}
