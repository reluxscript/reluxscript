/**
 * Extract Method Call Template
 *
 * Extracts template from method call expressions
 */

use "./extract_binding.rsc" { extract_binding };
use "./extract_state_key.rsc" { extract_state_key };
use "./supported_transforms.rsc" { get_transform_info };

/**
 * Method call transform
 */
pub struct MethodCallTransform {
    pub transform_type: Str,
    pub method: Str,
    pub args: Vec<String>,
}

/**
 * Method call template result
 */
pub struct MethodCallTemplate {
    pub template_type: Str,
    pub state_key: Str,
    pub binding: Str,
    pub method: Str,
    pub args: Vec<String>,
    pub transform: MethodCallTransform,
    pub path: Str,
}

/**
 * Extract template from method call
 *
 * Example: price.toFixed(2)
 * Returns: { type: 'methodCall', binding: 'price', method: 'toFixed', args: [2], transform: {...} }
 */
pub fn extract_method_call_template(
    call_expr: &CallExpression,
    component: &Component,
    path: &Str
) -> Option<MethodCallTemplate> {
    let callee = &call_expr.callee;
    let args = &call_expr.arguments;

    // Check if callee is a member expression
    if let Expression::MemberExpression(ref member_expr) = callee {
        // Get binding (e.g., 'price' from price.toFixed())
        let binding = extract_binding(&member_expr.object)?;

        // Get method name
        let method_name = if let Expression::Identifier(ref prop_id) = &member_expr.property {
            prop_id.name.clone()
        } else {
            return None;
        };

        // Check if this is a supported transformation
        let transform_info = get_transform_info(&method_name)?;

        // Extract arguments
        let extracted_args: Vec<String> = args.iter().filter_map(|arg| {
            match arg {
                Expression::NumericLiteral(ref num) => Some(num.value.to_string()),
                Expression::StringLiteral(ref str_lit) => Some(str_lit.value.clone()),
                Expression::BooleanLiteral(ref bool_lit) => Some(bool_lit.value.to_string()),
                _ => None,
            }
        }).collect();

        // Determine state key
        let state_key = extract_state_key(&member_expr.object, component)
            .unwrap_or_else(|| binding.clone());

        Some(MethodCallTemplate {
            template_type: String::from("methodCall"),
            state_key,
            binding,
            method: method_name.clone(),
            args: extracted_args.clone(),
            transform: MethodCallTransform {
                transform_type: transform_info.transform_type,
                method: method_name,
                args: extracted_args,
            },
            path: path.clone(),
        })
    } else {
        None
    }
}
