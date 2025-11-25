/**
 * Extract Logical And Element Template
 *
 * Extracts template from logical AND expressions with JSX
 */

use codegen;
use "./extract_left_side_of_and.rsc" { extract_left_side_of_and };
use "./extract_bindings_from_condition.rsc" { extract_bindings_from_condition };
use "./is_condition_evaluable_client_side.rsc" { is_condition_evaluable_client_side };
use "./extract_element_structure.rsc" { extract_element_structure, ElementStructure };

/**
 * Conditional element template result
 */
pub struct ConditionalElementTemplate {
    pub template_type: Str,
    pub condition_expression: Str,
    pub condition_bindings: Vec<Str>,
    pub condition_mapping: HashMap<Str, Str>,
    pub evaluable: bool,
    pub branches: ConditionalBranches,
    pub operator: Str,
    pub parent_template: Option<Str>,
}

/**
 * Conditional branches
 */
pub struct ConditionalBranches {
    pub true_branch: Option<ElementStructure>,
    pub false_branch: Option<ElementStructure>,
}

/**
 * Extract template from logical AND expression
 * Example: {myState1 && !myState2 && <div>{myState3}</div>}
 *
 * @param expr - Logical expression
 * @param container_node - Container JSX node
 * @param parent_path - Hex path of parent conditional template (for nesting)
 * @param state_key_map - Mapping from variable names to state keys
 */
pub fn extract_logical_and_element_template(
    expr: &LogicalExpression,
    container_node: &JSXElement,
    parent_path: Option<Str>,
    state_key_map: &HashMap<Str, Str>
) -> Option<ConditionalElementTemplate> {
    // Check if operator is AND
    if expr.operator != "&&" {
        return None;
    }

    let right = &expr.right;

    // Check if right side is JSX element (structural)
    let right_is_jsx = matches!(right,
        Expression::JSXElement(_) | Expression::JSXFragment(_)
    );

    if !right_is_jsx {
        return None;
    }

    // Extract full condition expression
    let condition = extract_left_side_of_and(&Expression::LogicalExpression(expr.clone()));
    let condition_code = codegen::generate(condition);

    // Extract bindings from condition (variable names)
    let variable_names = extract_bindings_from_condition(condition);

    // Build mapping from variable names to state keys
    let mut condition_mapping = HashMap::new();
    let mut state_keys = vec![];

    for var_name in &variable_names {
        let state_key = state_key_map.get(var_name)
            .cloned()
            .unwrap_or_else(|| var_name.clone());

        condition_mapping.insert(var_name.clone(), state_key.clone());
        state_keys.push(state_key);
    }

    // Can we evaluate this condition client-side?
    let is_evaluable = is_condition_evaluable_client_side(condition, &variable_names);

    // Extract element structure
    let element_structure = match right {
        Expression::JSXElement(ref elem) | Expression::JSXFragment(ref frag) => {
            extract_element_structure(right)?
        }
        _ => return None,
    };

    Some(ConditionalElementTemplate {
        template_type: String::from("conditional-element"),
        condition_expression: condition_code,
        condition_bindings: state_keys,
        condition_mapping,
        evaluable: is_evaluable,
        branches: ConditionalBranches {
            true_branch: Some(element_structure),
            false_branch: None,
        },
        operator: String::from("&&"),
        parent_template: parent_path,
    })
}
