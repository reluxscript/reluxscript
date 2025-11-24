/**
 * Extract Binding
 *
 * Main binding extraction logic for JSX expressions
 */

use "./build_member_path.rsc" { build_member_path };
use "./extract_identifiers.rsc" { extract_identifiers };
use "./extract_method_call_binding.rsc" { extract_method_call_binding, MethodCallBinding };
use "./extract_conditional_binding.rsc" { extract_conditional_binding, ConditionalBinding };
use "./extract_optional_chain_binding.rsc" { extract_optional_chain_binding, OptionalChainBinding };

/**
 * Binding extraction result
 */
pub enum Binding {
    Simple(Str),
    MethodCall(MethodCallBinding),
    Conditional(ConditionalBinding),
    OptionalChain(OptionalChainBinding),
}

/**
 * Extract binding name from expression
 * Supports:
 * - Identifiers: {count}
 * - Member expressions: {user.name}
 * - Simple operations: {count + 1}
 * - Conditionals: {isExpanded ? 'Hide' : 'Show'}
 * - Method calls: {price.toFixed(2)}
 * - Optional chaining: {viewModel?.userEmail}
 */
pub fn extract_binding(expr: &Expression, component: &Component) -> Option<Binding> {
    match expr {
        Expression::Identifier(ref id) => {
            Some(Binding::Simple(id.name.clone()))
        }

        Expression::MemberExpression(_) => {
            Some(Binding::Simple(build_member_path(expr)))
        }

        Expression::OptionalMemberExpression(_) => {
            // Optional chaining (viewModel?.userEmail)
            if let Some(opt_binding) = extract_optional_chain_binding(expr) {
                Some(Binding::OptionalChain(opt_binding))
            } else {
                None
            }
        }

        Expression::CallExpression(ref call_expr) => {
            // Method calls (price.toFixed(2))
            if let Some(method_binding) = extract_method_call_binding(call_expr) {
                Some(Binding::MethodCall(method_binding))
            } else {
                None
            }
        }

        Expression::BinaryExpression(ref bin_expr) | Expression::UnaryExpression(ref unary_expr) => {
            // Simple operations - extract all identifiers
            let mut identifiers = vec![];
            extract_identifiers(expr, &mut identifiers);
            Some(Binding::Simple(identifiers.join(".")))
        }

        Expression::ConditionalExpression(ref cond_expr) => {
            // Ternary expression: {isExpanded ? 'Hide' : 'Show'}
            if let Some(cond_binding) = extract_conditional_binding(cond_expr) {
                Some(Binding::Conditional(cond_binding))
            } else {
                None
            }
        }

        _ => {
            // Complex expression
            None
        }
    }
}
