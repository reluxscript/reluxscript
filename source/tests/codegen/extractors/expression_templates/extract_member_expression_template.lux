/**
 * Extract Member Expression Template
 *
 * Extracts template from member expressions (property access)
 */

use "./build_member_path.rsc" { build_member_path };
use "./extract_state_key.rsc" { extract_state_key };
use "./supported_transforms.rsc" { get_transform_info };

/**
 * Transform metadata
 */
pub struct MemberTransform {
    pub transform_type: Str,
    pub property: Str,
}

/**
 * Member expression template result
 */
pub struct MemberExpressionTemplate {
    pub template_type: Str,
    pub state_key: Str,
    pub binding: Str,
    pub property: Str,
    pub transform: MemberTransform,
    pub path: Str,
    pub is_computed: bool,
    pub requires_runtime_eval: bool,
}

/**
 * Extract template from member expression
 *
 * Example: items.length
 * Returns: { type: 'memberExpression', binding: 'items.length', transform: { type: 'property', property: 'length' } }
 */
pub fn extract_member_expression_template(
    member_expr: &MemberExpression,
    component: &Component,
    path: &Str
) -> Option<MemberExpressionTemplate> {
    // Check for computed property access: item[field]
    if member_expr.computed {
        // Computed properties require runtime evaluation
        return Some(MemberExpressionTemplate {
            template_type: String::from("computedMemberExpression"),
            state_key: String::new(),
            binding: String::new(),
            property: String::new(),
            transform: MemberTransform {
                transform_type: String::from("computed"),
                property: String::new(),
            },
            path: path.clone(),
            is_computed: true,
            requires_runtime_eval: true,
        });
    }

    let binding = build_member_path(&Expression::MemberExpression(member_expr.clone()));

    // Get property name (only for non-computed properties)
    let property_name = if let Expression::Identifier(ref prop_id) = &member_expr.property {
        prop_id.name.clone()
    } else {
        return None;
    };

    // Check if it's a supported property transform
    let transform_info = get_transform_info(&property_name)?;

    let state_key = extract_state_key(&Expression::MemberExpression(member_expr.clone()), component)
        .unwrap_or_else(|| binding.split('.').next().unwrap_or("").to_string());

    Some(MemberExpressionTemplate {
        template_type: String::from("memberExpression"),
        state_key,
        binding,
        property: property_name.clone(),
        transform: MemberTransform {
            transform_type: transform_info.transform_type,
            property: property_name,
        },
        path: path.clone(),
        is_computed: false,
        requires_runtime_eval: false,
    })
}
