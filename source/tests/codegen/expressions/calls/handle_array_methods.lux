/**
 * Handle array method calls
 *
 * Converts JavaScript array methods to C# LINQ methods
 */

// Note: This requires generate_jsx_element from jsx module
// We'll define the signature here and expect it to be provided at runtime

/**
 * Component metadata structure
 */
pub struct ComponentContext {
    pub use_state: Vec<UseStateInfo>,
    pub use_client_state: Vec<UseStateInfo>,
    pub current_map_context: Option<MapContext>,
}

pub struct UseStateInfo {
    pub name: Str,
    pub setter: Str,
}

pub struct MapContext {
    pub params: Vec<Str>,
}

/**
 * Handle .map() → .Select()
 */
pub fn handle_map<F, G, H>(
    node: &CallExpression,
    generate_csharp_expression: F,
    generate_csharp_statement: G,
    generate_jsx_element: H,
    current_component: Option<&mut ComponentContext>
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str,
    G: Fn(&Statement) -> Str,
    H: Fn(&JSXElement, Option<&mut ComponentContext>, i32) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name != "map" {
                return None;
            }

            let object = generate_csharp_expression(&member.object, false);

            if node.arguments.is_empty() {
                return None;
            }

            if let Expression::ArrowFunctionExpression(ref callback) = node.arguments[0] {
                // Extract parameter names
                let param_names: Vec<Str> = callback.params
                    .iter()
                    .filter_map(|p| {
                        if let Pattern::Identifier(ref id) = p {
                            Some(id.name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();

                // C# requires parentheses for 0 or 2+ parameters
                let params = if param_names.len() == 1 {
                    param_names[0].clone()
                } else {
                    format!("({})", param_names.join(", "))
                };

                // Handle JSX in arrow function body
                let body = if !callback.is_expression {
                    // Block statement
                    if let Some(ref block) = callback.body_block {
                        let statements: Vec<Str> = block.body
                            .iter()
                            .map(|stmt| generate_csharp_statement(stmt))
                            .collect();
                        format!("{{ {} }}", statements.join(" "))
                    } else {
                        "{}".to_string()
                    }
                } else if let Some(ref body_expr) = callback.body_expression {
                    // Check if it's JSX
                    match body_expr {
                        Expression::JSXElement(ref jsx) | Expression::JSXFragment(ref jsx) => {
                            // Store map context for event handler closure capture
                            // For nested maps, we need to ACCUMULATE params, not replace them
                            let previous_map_context = if let Some(ref comp) = current_component {
                                comp.current_map_context.clone()
                            } else {
                                None
                            };

                            let previous_params = previous_map_context
                                .as_ref()
                                .map(|ctx| ctx.params.clone())
                                .unwrap_or_else(Vec::new);

                            if let Some(ref mut comp) = current_component {
                                // Combine previous params with current params for nested map support
                                let mut combined_params = previous_params.clone();
                                combined_params.extend(param_names.clone());
                                comp.current_map_context = Some(MapContext {
                                    params: combined_params,
                                });
                            }

                            let jsx_code = generate_jsx_element(jsx, current_component, 0);

                            // Restore previous context
                            if let Some(ref mut comp) = current_component {
                                comp.current_map_context = previous_map_context;
                            }

                            jsx_code
                        }
                        _ => generate_csharp_expression(body_expr, false),
                    }
                } else {
                    "/* empty */".to_string()
                };

                // Cast to IEnumerable<dynamic> if we detect dynamic access
                // Check for optional chaining or property access (likely dynamic)
                let needs_cast = object.contains("?.") || object.contains('?') || object.contains('.');
                let casted_object = if needs_cast {
                    format!("((IEnumerable<dynamic>){})", object)
                } else {
                    object.clone()
                };

                // If the object needs casting (is dynamic), we also need to cast the lambda
                // to prevent CS1977: "Cannot use a lambda expression as an argument to a dynamically dispatched operation"
                let lambda_expr = format!("{} => {}", params, body);
                let casted_lambda = if needs_cast {
                    format!("(Func<dynamic, dynamic>)({})", lambda_expr)
                } else {
                    lambda_expr
                };

                return Some(format!("{}.Select({}).ToList()", casted_object, casted_lambda));
            }
        }
    }

    None
}

/**
 * Handle useState/useClientState setters → SetState calls
 */
pub fn handle_state_setters<F>(
    node: &CallExpression,
    generate_csharp_expression: F,
    current_component: Option<&ComponentContext>
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::Identifier(ref callee) = node.callee {
        if let Some(component) = current_component {
            let setter_name = &callee.name;

            // Check if this is a useState setter
            let use_state = component
                .use_state
                .iter()
                .chain(component.use_client_state.iter())
                .find(|state| &state.setter == setter_name);

            if let Some(state_info) = use_state {
                if !node.arguments.is_empty() {
                    let new_value = generate_csharp_expression(&node.arguments[0], false);
                    return Some(format!("SetState(nameof({}), {})", state_info.name, new_value));
                }
            }
        }
    }

    None
}
