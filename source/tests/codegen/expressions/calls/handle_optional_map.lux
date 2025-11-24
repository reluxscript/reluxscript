/**
 * Handle optional .map() call: array?.map(...)
 *
 * Converts JavaScript optional chaining map to C# optional LINQ
 */

use "./handle_array_methods.rsc" { ComponentContext, MapContext };

/**
 * Handle optional .map() call: array?.map(...)
 */
pub fn handle_optional_map<F, G, H>(
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
    if let Expression::OptionalMemberExpression(ref member) = node.callee {
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

                // Cast to IEnumerable<dynamic> for optional chaining (likely dynamic)
                let casted_object = format!("((IEnumerable<dynamic>){})", object);

                // Cast result to List<dynamic> for ?? operator compatibility
                // Anonymous types from Select need explicit Cast<dynamic>() before ToList()
                return Some(format!(
                    "{}?.Select({} => {})?.Cast<dynamic>().ToList()",
                    casted_object, params, body
                ));
            }
        }
    }

    None
}
