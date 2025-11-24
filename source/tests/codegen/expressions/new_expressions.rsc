/**
 * New expression handlers
 *
 * Generates C# code for JavaScript/TypeScript new expressions
 */

/**
 * Generate new expression
 *
 * @param node - NewExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @returns C# new expression or equivalent code
 */
pub fn generate_new_expression<F>(
    node: &NewExpression,
    generate_csharp_expression: F
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    // Handle new Promise(resolve => setTimeout(resolve, ms)) → Task.Delay(ms)
    if let Expression::Identifier(ref callee_id) = node.callee {
        if callee_id.name == "Promise" && !node.arguments.is_empty() {
            if let Some(ref first_arg) = node.arguments.first() {
                if let Expression::ArrowFunctionExpression(ref callback) = first_arg {
                    // Check if it's the setTimeout pattern
                    if callback.params.len() == 1 {
                        if let Some(Pattern::Identifier(ref resolve_param)) = callback.params.first() {
                            let resolve_name = &resolve_param.name;

                            // Check if body is: setTimeout(resolve, ms)
                            if callback.is_expression {
                                if let Some(ref body_expr) = callback.body_expression {
                                    if let Expression::CallExpression(ref call) = body_expr {
                                        if let Expression::Identifier(ref call_id) = call.callee {
                                            if call_id.name == "setTimeout" && call.arguments.len() == 2 {
                                                // Check first arg is resolve
                                                if let Some(Expression::Identifier(ref first)) = call.arguments.first() {
                                                    if first.name == *resolve_name {
                                                        // Get delay argument
                                                        if let Some(ref delay_arg) = call.arguments.get(1) {
                                                            let delay = generate_csharp_expression(delay_arg, false);
                                                            return format!("Task.Delay({})", delay);
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Generic Promise constructor - not directly supported in C#
            // Return Task.CompletedTask as a fallback
            return "Task.CompletedTask".to_string();
        }

        // Handle new Date() → DateTime.Now or DateTime.Parse()
        if callee_id.name == "Date" {
            if node.arguments.is_empty() {
                return "DateTime.Now".to_string();
            } else if node.arguments.len() == 1 {
                if let Some(ref arg) = node.arguments.first() {
                    let arg_code = generate_csharp_expression(arg, false);
                    return format!("DateTime.Parse({})", arg_code);
                }
            }
        }

        // Handle new Error() → new Exception()
        if callee_id.name == "Error" {
            let args: Vec<Str> = node.arguments
                .iter()
                .map(|arg| generate_csharp_expression(arg, false))
                .collect();
            let args_str = args.join(", ");
            return format!("new Exception({})", args_str);
        }
    }

    // Handle other new expressions: new Foo() → new Foo()
    let callee = generate_csharp_expression(&node.callee, false);
    let args: Vec<Str> = node.arguments
        .iter()
        .map(|arg| generate_csharp_expression(arg, false))
        .collect();
    let args_str = args.join(", ");

    format!("new {}({})", callee, args_str)
}
