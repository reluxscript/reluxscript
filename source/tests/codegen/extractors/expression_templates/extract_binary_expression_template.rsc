/**
 * Extract Binary Expression Template
 *
 * Extracts template from binary expressions (arithmetic, etc.)
 */

use "./extract_identifiers.rsc" { extract_identifiers };
use "./analyze_binary_expression.rsc" { analyze_binary_expression, BinaryExpressionAnalysis };
use "./generate_expression_string.rsc" { generate_expression_string };

/**
 * Binary expression template result
 */
pub struct BinaryExpressionTemplate {
    pub template_type: Str,
    pub state_key: Str,
    pub bindings: Vec<Str>,
    pub transform: Option<BinaryExpressionAnalysis>,
    pub expression: Option<Str>,
    pub path: Str,
}

/**
 * Extract template from binary expression
 *
 * Example: count * 2 + 1
 * Returns: { type: 'binaryExpression', bindings: ['count'], transform: { operations: [...] } }
 */
pub fn extract_binary_expression_template(
    binary_expr: &BinaryExpression,
    component: &Component,
    path: &Str
) -> Option<BinaryExpressionTemplate> {
    // Extract all identifiers
    let mut identifiers = vec![];
    extract_identifiers(&Expression::BinaryExpression(binary_expr.clone()), &mut identifiers);

    if identifiers.is_empty() {
        return None;
    }

    // For simple cases (single identifier with constant), extract transform
    if identifiers.len() == 1 {
        let binding = &identifiers[0];
        let transform = analyze_binary_expression(
            &Expression::BinaryExpression(binary_expr.clone()),
            binding
        );

        if let Some(transform_result) = transform {
            let state_key = binding.split('.').next().unwrap_or(binding).to_string();

            return Some(BinaryExpressionTemplate {
                template_type: String::from("binaryExpression"),
                state_key,
                bindings: identifiers,
                transform: Some(transform_result),
                expression: None,
                path: path.clone(),
            });
        }
    }

    // Complex multi-variable expression - store as formula
    let state_key = identifiers[0].split('.').next().unwrap_or(&identifiers[0]).to_string();
    let expression_string = generate_expression_string(&Expression::BinaryExpression(binary_expr.clone()));

    Some(BinaryExpressionTemplate {
        template_type: String::from("complexExpression"),
        state_key,
        bindings: identifiers,
        transform: None,
        expression: Some(expression_string),
        path: path.clone(),
    })
}
