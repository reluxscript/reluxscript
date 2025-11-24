/**
 * Extract Ternary Element Template
 *
 * Extracts template from ternary expressions with JSX
 */

use codegen;
use "./extract_bindings_from_condition.rsc" { extract_bindings_from_condition };
use "./is_condition_evaluable_client_side.rsc" { is_condition_evaluable_client_side };
use "./extract_element_structure.rsc" { extract_element_structure, ElementStructure };
use "./extract_logical_and_element_template.rsc" { ConditionalElementTemplate, ConditionalBranches };

/**
 * Extract template from ternary expression
 * Example: {myState1 ? <div>Active</div> : <div>Inactive</div>}
 *
 * @param expr - Conditional expression (ternary)
 * @param container_node - Container JSX node
 * @param parent_path - Hex path of parent conditional template (for nesting)
 * @param state_key_map - Mapping from variable names to state keys
 */
pub fn extract_ternary_element_template(
    expr: &ConditionalExpression,
    container_node: &JSXElement,
    parent_path: Option<Str>,
    state_key_map: &HashMap<Str, Str>
) -> Option<ConditionalElementTemplate> {
    let test = &expr.test;
    let consequent = &expr.consequent;
    let alternate = &expr.alternate;

    // Check if branches are JSX elements
    let has_consequent = matches!(consequent,
        Expression::JSXElement(_) | Expression::JSXFragment(_)
    );

    let has_alternate = matches!(alternate,
        Expression::JSXElement(_) | Expression::JSXFragment(_) | Expression::NullLiteral
    );

    if !has_consequent && !has_alternate {
        return None; // Not a structural template
    }

    // Extract condition
    let condition_code = codegen::generate(test);
    let variable_names = extract_bindings_from_condition(test);

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

    let is_evaluable = is_condition_evaluable_client_side(test, &variable_names);

    // Extract both branches
    let true_branch = if has_consequent {
        extract_element_structure(consequent)
    } else {
        None
    };

    let false_branch = if has_alternate {
        if matches!(alternate, Expression::NullLiteral) {
            None
        } else {
            extract_element_structure(alternate)
        }
    } else {
        None
    };

    Some(ConditionalElementTemplate {
        template_type: String::from("conditional-element"),
        condition_expression: condition_code,
        condition_bindings: state_keys,
        condition_mapping,
        evaluable: is_evaluable,
        branches: ConditionalBranches {
            true_branch,
            false_branch,
        },
        operator: String::from("?"),
        parent_template: parent_path,
    })
}
