/**
 * Extract Unary Expression Template
 *
 * Extracts template from unary expressions (-count, +value, etc.)
 */

use "./extract_binding.rsc" { extract_binding };
use "./extract_state_key.rsc" { extract_state_key };

/**
 * Unary transform metadata
 */
pub struct UnaryTransform {
    pub transform_type: Str,
    pub operator: Str,
}

/**
 * Unary expression template result
 */
pub struct UnaryExpressionTemplate {
    pub template_type: Str,
    pub state_key: Str,
    pub binding: Str,
    pub operator: Str,
    pub transform: UnaryTransform,
    pub path: Str,
}

/**
 * Extract template from unary expression
 *
 * Example: -count, +value
 */
pub fn extract_unary_expression_template(
    unary_expr: &UnaryExpression,
    component: &Component,
    path: &Str
) -> Option<UnaryExpressionTemplate> {
    let operator = &unary_expr.operator;
    let argument = &unary_expr.argument;

    let binding = extract_binding(argument)?;

    // Only handle numeric unary operators
    if operator != "-" && operator != "+" {
        return None;
    }

    let state_key = extract_state_key(argument, component)
        .unwrap_or_else(|| binding.clone());

    Some(UnaryExpressionTemplate {
        template_type: String::from("unaryExpression"),
        state_key,
        binding,
        operator: operator.clone(),
        transform: UnaryTransform {
            transform_type: String::from("unary"),
            operator: operator.clone(),
        },
        path: path.clone(),
    })
}
