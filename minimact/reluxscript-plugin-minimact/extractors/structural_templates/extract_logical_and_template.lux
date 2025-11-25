/**
 * Extract Logical And Template
 *
 * Extracts structural template from logical AND expressions
 */

use "./extract_binding.rsc" { extract_binding };
use "./extract_state_key.rsc" { extract_state_key };
use "./extract_element_or_fragment_template.rsc" { extract_element_or_fragment_template, ElementOrFragmentTemplate };

/**
 * Logical AND template result
 */
pub struct LogicalAndTemplate {
    pub template_type: Str,
    pub state_key: Str,
    pub condition_binding: Str,
    pub truthy_branch: ElementOrFragmentTemplate,
    pub falsy_branch: NullTemplate,
    pub path: Str,
}

/**
 * Null template (represents null/nothing)
 */
pub struct NullTemplate {
    pub template_type: Str,
}

/**
 * Extract structural template from logical AND
 *
 * Example: {error && <ErrorMessage />}
 * Returns: { type: 'logicalAnd', conditionBinding: 'error', branches: {...} }
 */
pub fn extract_logical_and_template(
    logical_expr: &LogicalExpression,
    component: &Component,
    path: &Str
) -> Option<LogicalAndTemplate> {
    // Check operator
    if logical_expr.operator != "&&" {
        return None;
    }

    let left = &logical_expr.left;
    let right = &logical_expr.right;

    // Extract condition binding from left side
    let condition_binding = extract_binding(left, component)?;

    // Check if right side is JSX element (structural change)
    let right_is_jsx = matches!(right,
        Expression::JSXElement(_) | Expression::JSXFragment(_)
    );

    if !right_is_jsx {
        return None;
    }

    // Extract template for truthy case
    let truthy_branch = if let Expression::JSXElement(ref elem) = right {
        extract_element_or_fragment_template(&JSXElementOrFragment::JSXElement(elem.clone()), component)?
    } else if let Expression::JSXFragment(ref frag) = right {
        extract_element_or_fragment_template(&JSXElementOrFragment::JSXFragment(frag.clone()), component)?
    } else {
        return None;
    };

    let state_key = extract_state_key(left, component)
        .unwrap_or_else(|| condition_binding.clone());

    Some(LogicalAndTemplate {
        template_type: String::from("logicalAnd"),
        state_key,
        condition_binding,
        truthy_branch,
        falsy_branch: NullTemplate {
            template_type: String::from("Null"),
        },
        path: path.clone(),
    })
}
