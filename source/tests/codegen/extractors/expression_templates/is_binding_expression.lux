/**
 * Is Binding Expression
 *
 * Checks if an expression matches a target binding
 */

use "./extract_binding.rsc" { extract_binding };

/**
 * Check if expression is our target binding
 */
pub fn is_binding_expression(expr: &Expression, target_binding: &Str) -> bool {
    if let Some(binding) = extract_binding(expr) {
        binding == *target_binding
    } else {
        false
    }
}
