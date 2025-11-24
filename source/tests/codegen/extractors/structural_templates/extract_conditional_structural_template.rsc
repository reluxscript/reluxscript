/**
 * Extract Conditional Structural Template
 *
 * Extracts structural template from ternary conditionals
 */

use "./extract_binding.rsc" { extract_binding };
use "./extract_state_key.rsc" { extract_state_key };
use "./extract_element_or_fragment_template.rsc" { extract_element_or_fragment_template, ElementOrFragmentTemplate };
use "./extract_logical_and_template.rsc" { NullTemplate };

/**
 * Conditional branches
 */
pub enum ConditionalBranch {
    Template(ElementOrFragmentTemplate),
    Null(NullTemplate),
}

/**
 * Conditional structural template result
 */
pub struct ConditionalStructuralTemplate {
    pub template_type: Str,
    pub state_key: Str,
    pub condition_binding: Str,
    pub true_branch: Option<ConditionalBranch>,
    pub false_branch: Option<ConditionalBranch>,
    pub path: Str,
}

/**
 * Extract structural template from ternary conditional
 *
 * Example: {isLoggedIn ? <Dashboard /> : <LoginForm />}
 * Returns: { type: 'conditional', conditionBinding: 'isLoggedIn', branches: {...} }
 */
pub fn extract_conditional_structural_template(
    conditional_expr: &ConditionalExpression,
    component: &Component,
    path: &Str
) -> Option<ConditionalStructuralTemplate> {
    let test = &conditional_expr.test;
    let consequent = &conditional_expr.consequent;
    let alternate = &conditional_expr.alternate;

    // Extract condition binding
    let condition_binding = extract_binding(test, component)?;

    // Check if both branches are JSX elements (structural change)
    let has_true_branch = matches!(consequent,
        Expression::JSXElement(_) | Expression::JSXFragment(_)
    );

    let has_false_branch = matches!(alternate,
        Expression::JSXElement(_) | Expression::JSXFragment(_) | Expression::NullLiteral
    );

    if !has_true_branch && !has_false_branch {
        // Not a structural template (probably just conditional text)
        return None;
    }

    // Extract templates for both branches
    let true_branch = if has_true_branch {
        let template = match consequent {
            Expression::JSXElement(ref elem) => {
                extract_element_or_fragment_template(&JSXElementOrFragment::JSXElement(elem.clone()), component)?
            }
            Expression::JSXFragment(ref frag) => {
                extract_element_or_fragment_template(&JSXElementOrFragment::JSXFragment(frag.clone()), component)?
            }
            _ => return None,
        };
        Some(ConditionalBranch::Template(template))
    } else {
        None
    };

    let false_branch = if has_false_branch {
        if matches!(alternate, Expression::NullLiteral) {
            Some(ConditionalBranch::Null(NullTemplate {
                template_type: String::from("Null"),
            }))
        } else {
            let template = match alternate {
                Expression::JSXElement(ref elem) => {
                    extract_element_or_fragment_template(&JSXElementOrFragment::JSXElement(elem.clone()), component)?
                }
                Expression::JSXFragment(ref frag) => {
                    extract_element_or_fragment_template(&JSXElementOrFragment::JSXFragment(frag.clone()), component)?
                }
                _ => return None,
            };
            Some(ConditionalBranch::Template(template))
        }
    } else {
        None
    };

    // Determine state key (for C# attribute)
    let state_key = extract_state_key(test, component)
        .unwrap_or_else(|| condition_binding.clone());

    Some(ConditionalStructuralTemplate {
        template_type: String::from("conditional"),
        state_key,
        condition_binding,
        true_branch,
        false_branch,
        path: path.clone(),
    })
}
