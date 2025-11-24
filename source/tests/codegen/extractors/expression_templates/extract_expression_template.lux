/**
 * Extract Expression Template
 *
 * Main entry point for extracting expression templates
 */

use "./extract_method_call_template.rsc" { extract_method_call_template, MethodCallTemplate };
use "./extract_binary_expression_template.rsc" { extract_binary_expression_template, BinaryExpressionTemplate };
use "./extract_member_expression_template.rsc" { extract_member_expression_template, MemberExpressionTemplate };
use "./extract_unary_expression_template.rsc" { extract_unary_expression_template, UnaryExpressionTemplate };

/**
 * Expression template result (union of all template types)
 */
pub enum ExpressionTemplate {
    MethodCall(MethodCallTemplate),
    BinaryExpression(BinaryExpressionTemplate),
    MemberExpression(MemberExpressionTemplate),
    UnaryExpression(UnaryExpressionTemplate),
}

/**
 * Extract expression template from expression node
 * Returns None for simple identifiers or expressions handled elsewhere
 */
pub fn extract_expression_template(
    expr: &Expression,
    component: &Component,
    path: &Str
) -> Option<ExpressionTemplate> {
    match expr {
        // Skip simple identifiers (no transformation)
        Expression::Identifier(_) => None,

        // Skip conditionals (handled by structural templates)
        Expression::ConditionalExpression(_) | Expression::LogicalExpression(_) => None,

        // Method call: price.toFixed(2)
        Expression::CallExpression(ref call_expr) => {
            if let Expression::MemberExpression(_) = &call_expr.callee {
                if let Some(template) = extract_method_call_template(call_expr, component, path) {
                    return Some(ExpressionTemplate::MethodCall(template));
                }
            }
            None
        }

        // Binary expression: count * 2 + 1
        Expression::BinaryExpression(ref bin_expr) => {
            if let Some(template) = extract_binary_expression_template(bin_expr, component, path) {
                Some(ExpressionTemplate::BinaryExpression(template))
            } else {
                None
            }
        }

        // Member expression: user.name, items.length
        Expression::MemberExpression(ref member_expr) => {
            if let Some(template) = extract_member_expression_template(member_expr, component, path) {
                Some(ExpressionTemplate::MemberExpression(template))
            } else {
                None
            }
        }

        // Unary expression: -count, +value
        Expression::UnaryExpression(ref unary_expr) => {
            if let Some(template) = extract_unary_expression_template(unary_expr, component, path) {
                Some(ExpressionTemplate::UnaryExpression(template))
            } else {
                None
            }
        }

        _ => None
    }
}
