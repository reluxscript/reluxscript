/**
 * Extract Binding Shared
 *
 * Shared binding extraction utilities for template literals and expressions
 */

use "./build_member_path.rsc" { build_member_path };
use "./extract_identifiers.rsc" { extract_identifiers };

/**
 * Transform method binding
 */
pub struct TransformBinding {
    pub transform: Str,
    pub binding: Str,
    pub args: Vec<Str>,
}

/**
 * Binding result - can be either a simple binding string or a transform
 */
pub enum BindingResult {
    Simple(Str),
    Transform(TransformBinding),
}

/**
 * Extract method call binding (shared version)
 * Handles: price.toFixed(2), text.toLowerCase(), etc.
 */
fn extract_method_call_binding_shared(expr: &CallExpression) -> Option<TransformBinding> {
    let callee = &expr.callee;

    // Get method name
    let method_name = match callee {
        Expression::MemberExpression(ref member) | Expression::OptionalMemberExpression(ref member) => {
            if let Expression::Identifier(ref property) = &member.property {
                property.name.clone()
            } else {
                return None;
            }
        }
        _ => return None,
    };

    let transform_methods = vec![
        "toFixed", "toString", "toLowerCase", "toUpperCase",
        "trim", "trimStart", "trimEnd"
    ];

    if !transform_methods.contains(&method_name.as_str()) {
        return None;
    }

    // Extract binding
    let binding = match callee {
        Expression::MemberExpression(ref member) | Expression::OptionalMemberExpression(ref member) => {
            match &member.object {
                Expression::MemberExpression(_) => build_member_path(&member.object),
                Expression::Identifier(ref id) => id.name.clone(),
                Expression::BinaryExpression(_) => {
                    let mut identifiers = vec![];
                    extract_identifiers(&member.object, &mut identifiers);
                    format!("__expr__:{}", identifiers.join(","))
                }
                _ => return None,
            }
        }
        _ => return None,
    };

    // Extract arguments
    let args: Vec<Str> = expr.arguments.iter().filter_map(|arg| {
        match arg {
            Expression::NumericLiteral(ref num) => Some(num.value.to_string()),
            Expression::StringLiteral(ref str_lit) => Some(str_lit.value.clone()),
            Expression::BooleanLiteral(ref bool_lit) => Some(bool_lit.value.to_string()),
            _ => None,
        }
    }).collect();

    Some(TransformBinding {
        transform: method_name,
        binding,
        args,
    })
}

/**
 * Extract binding from binary expression
 * Examples: todo.priority + 1, price * quantity, index * 2 + 1
 */
fn extract_binary_expression_binding(expr: &BinaryExpression) -> Str {
    let mut identifiers = vec![];
    extract_identifiers(&Expression::BinaryExpression(expr.clone()), &mut identifiers);

    // Use __expr__ prefix to indicate this is a computed expression
    format!("__expr__:{}", identifiers.join(","))
}

/**
 * Extract binding from logical expression
 * Examples: todo.dueDate || 'No due date', condition && value
 */
fn extract_logical_expression_binding(expr: &LogicalExpression) -> Str {
    let mut identifiers = vec![];
    extract_identifiers(&Expression::LogicalExpression(expr.clone()), &mut identifiers);

    // Use __expr__ prefix to indicate this is a computed expression
    format!("__expr__:{}", identifiers.join(","))
}

/**
 * Extract binding from unary expression
 * Examples: !todo.completed, -value
 */
fn extract_unary_expression_binding(expr: &UnaryExpression) -> Str {
    let mut identifiers = vec![];
    extract_identifiers(&Expression::UnaryExpression(expr.clone()), &mut identifiers);

    // Use __expr__ prefix to indicate this is a computed expression
    format!("__expr__:{}", identifiers.join(","))
}

/**
 * Extract binding from complex call expression (non-transform methods)
 * Examples: todo.text.substring(0, 10).toUpperCase(), array.concat(other)
 */
fn extract_complex_call_expression(expr: &CallExpression) -> Option<Str> {
    let mut identifiers = vec![];
    extract_identifiers(&Expression::CallExpression(expr.clone()), &mut identifiers);

    if identifiers.is_empty() {
        return None;
    }

    // Use __expr__ prefix to indicate this is a computed expression
    Some(format!("__expr__:{}", identifiers.join(",")))
}

/**
 * Shared helper: Extract binding from expression
 * Returns either a simple binding string or a transform binding
 */
pub fn extract_binding_shared(expr: &Expression) -> Option<BindingResult> {
    match expr {
        Expression::Identifier(ref id) => {
            Some(BindingResult::Simple(id.name.clone()))
        }

        Expression::MemberExpression(_) => {
            Some(BindingResult::Simple(build_member_path(expr)))
        }

        Expression::CallExpression(ref call_expr) => {
            // First try method call binding (toFixed, etc.)
            if let Some(method_binding) = extract_method_call_binding_shared(call_expr) {
                return Some(BindingResult::Transform(method_binding));
            }

            // Otherwise, handle chained method calls
            if let Some(complex) = extract_complex_call_expression(call_expr) {
                return Some(BindingResult::Simple(complex));
            }

            None
        }

        Expression::BinaryExpression(ref bin_expr) => {
            Some(BindingResult::Simple(extract_binary_expression_binding(bin_expr)))
        }

        Expression::LogicalExpression(ref log_expr) => {
            Some(BindingResult::Simple(extract_logical_expression_binding(log_expr)))
        }

        Expression::UnaryExpression(ref unary_expr) => {
            Some(BindingResult::Simple(extract_unary_expression_binding(unary_expr)))
        }

        _ => None,
    }
}
