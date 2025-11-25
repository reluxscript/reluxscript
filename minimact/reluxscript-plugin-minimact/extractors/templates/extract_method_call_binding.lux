/**
 * Extract Method Call Binding
 *
 * Extracts binding from method call expressions with transformations
 */

use "./build_member_path.rsc" { build_member_path, build_optional_member_path };
use "./extract_identifiers.rsc" { extract_identifiers };

/**
 * Method call binding result
 */
pub struct MethodCallBinding {
    pub transform: Str,
    pub binding: Str,
    pub args: Vec<Str>,
}

/**
 * Extract method call binding
 * Handles: price.toFixed(2), text.toLowerCase(), etc.
 * Returns: { transform: 'toFixed', binding: 'price', args: [2] }
 */
pub fn extract_method_call_binding(expr: &CallExpression) -> Option<MethodCallBinding> {
    let callee = &expr.callee;

    // Get the method name from member or optional member expression
    let method_name = match callee {
        Expression::MemberExpression(ref member) => {
            if let Expression::Identifier(ref property) = &member.property {
                property.name.clone()
            } else {
                return None;
            }
        }

        Expression::OptionalMemberExpression(ref opt_member) => {
            if let Expression::Identifier(ref property) = &opt_member.property {
                property.name.clone()
            } else {
                return None;
            }
        }

        _ => return None, // Not a method call
    };

    // Supported transformation methods
    let transform_methods = vec![
        "toFixed", "toString", "toLowerCase", "toUpperCase",
        "trim", "trimStart", "trimEnd"
    ];

    if !transform_methods.contains(&method_name.as_str()) {
        return None; // Unsupported method
    }

    // Extract the object being called (price from price.toFixed(2))
    let binding = match callee {
        Expression::MemberExpression(ref member) => {
            match &member.object {
                Expression::MemberExpression(_) => {
                    build_member_path(&member.object)
                }

                Expression::Identifier(ref id) => {
                    id.name.clone()
                }

                Expression::BinaryExpression(_) => {
                    // Handle expressions like (discount * 100).toFixed(0)
                    let mut identifiers = vec![];
                    extract_identifiers(&member.object, &mut identifiers);
                    format!("__expr__:{}", identifiers.join(","))
                }

                _ => return None,
            }
        }

        Expression::OptionalMemberExpression(ref opt_member) => {
            match &opt_member.object {
                Expression::OptionalMemberExpression(_) | Expression::MemberExpression(_) => {
                    build_optional_member_path(&opt_member.object)?
                }

                Expression::Identifier(ref id) => {
                    id.name.clone()
                }

                _ => return None,
            }
        }

        _ => return None,
    };

    // Extract method arguments (e.g., 2 from toFixed(2))
    let args: Vec<Str> = expr.arguments.iter().filter_map(|arg| {
        match arg {
            Expression::NumericLiteral(ref num) => Some(num.value.to_string()),
            Expression::StringLiteral(ref str_lit) => Some(str_lit.value.clone()),
            Expression::BooleanLiteral(ref bool_lit) => Some(bool_lit.value.to_string()),
            _ => None,
        }
    }).collect();

    Some(MethodCallBinding {
        transform: method_name,
        binding,
        args,
    })
}
