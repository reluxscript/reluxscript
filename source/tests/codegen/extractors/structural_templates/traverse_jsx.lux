/**
 * Traverse JSX
 *
 * Traverses JSX tree looking for conditional expressions that affect structure
 */

use "./extract_conditional_structural_template.rsc" { extract_conditional_structural_template, ConditionalStructuralTemplate };
use "./extract_logical_and_template.rsc" { extract_logical_and_template, LogicalAndTemplate };

/**
 * Structural template result (union type)
 */
pub enum StructuralTemplate {
    Conditional(ConditionalStructuralTemplate),
    LogicalAnd(LogicalAndTemplate),
}

/**
 * Traverse JSX tree looking for conditional expressions that affect structure
 */
pub fn traverse_jsx(
    node: &JSXElementOrFragment,
    path: &Vec<i32>,
    structural_templates: &mut Vec<StructuralTemplate>,
    component: &Component
) {
    match node {
        JSXElementOrFragment::JSXElement(ref element) => {
            traverse_jsx_element(element, path, structural_templates, component);
        }

        JSXElementOrFragment::JSXFragment(ref fragment) => {
            traverse_jsx_fragment(fragment, path, structural_templates, component);
        }
    }
}

/**
 * Traverse JSX element
 */
fn traverse_jsx_element(
    node: &JSXElement,
    path: &Vec<i32>,
    structural_templates: &mut Vec<StructuralTemplate>,
    component: &Component
) {
    // Check children for conditional expressions
    for (i, child) in node.children.iter().enumerate() {
        match child {
            JSXChild::JSXExpressionContainer(ref expr_container) => {
                let mut child_path = path.clone();
                child_path.push(i as i32);
                let path_str = child_path.iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<Str>>()
                    .join(".");

                let expr = &expr_container.expression;

                // Ternary: {condition ? <A /> : <B />}
                if let Expression::ConditionalExpression(ref cond_expr) = expr {
                    if let Some(template) = extract_conditional_structural_template(
                        cond_expr,
                        component,
                        &path_str
                    ) {
                        structural_templates.push(StructuralTemplate::Conditional(template));
                    }
                }

                // Logical AND: {condition && <Component />}
                if let Expression::LogicalExpression(ref logical_expr) = expr {
                    if logical_expr.operator == "&&" {
                        if let Some(template) = extract_logical_and_template(
                            logical_expr,
                            component,
                            &path_str
                        ) {
                            structural_templates.push(StructuralTemplate::LogicalAnd(template));
                        }
                    }
                }
            }

            JSXChild::JSXElement(ref child_element) => {
                let mut child_path = path.clone();
                child_path.push(i as i32);
                traverse_jsx_element(child_element, &child_path, structural_templates, component);
            }

            _ => {}
        }
    }
}

/**
 * Traverse JSX fragment
 */
fn traverse_jsx_fragment(
    node: &JSXFragment,
    path: &Vec<i32>,
    structural_templates: &mut Vec<StructuralTemplate>,
    component: &Component
) {
    for (i, child) in node.children.iter().enumerate() {
        let mut child_path = path.clone();
        child_path.push(i as i32);
        let path_str = child_path.iter()
            .map(|n| n.to_string())
            .collect::<Vec<Str>>()
            .join(".");

        match child {
            JSXChild::JSXElement(ref child_element) => {
                traverse_jsx_element(child_element, &child_path, structural_templates, component);
            }

            JSXChild::JSXExpressionContainer(ref expr_container) => {
                let expr = &expr_container.expression;

                // Ternary
                if let Expression::ConditionalExpression(ref cond_expr) = expr {
                    if let Some(template) = extract_conditional_structural_template(
                        cond_expr,
                        component,
                        &path_str
                    ) {
                        structural_templates.push(StructuralTemplate::Conditional(template));
                    }
                }

                // Logical AND
                if let Expression::LogicalExpression(ref logical_expr) = expr {
                    if logical_expr.operator == "&&" {
                        if let Some(template) = extract_logical_and_template(
                            logical_expr,
                            component,
                            &path_str
                        ) {
                            structural_templates.push(StructuralTemplate::LogicalAnd(template));
                        }
                    }
                }
            }

            _ => {}
        }
    }
}
