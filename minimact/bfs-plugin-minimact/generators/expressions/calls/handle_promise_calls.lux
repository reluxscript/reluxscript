/**
 * Handle Promise.* method calls
 *
 * Converts JavaScript Promise API to C# Task API
 */

/**
 * Handle Promise.resolve(value) → Task.FromResult(value)
 */
pub fn handle_promise_resolve<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref obj) = member.object {
            if obj.name == "Promise" {
                if let Expression::Identifier(ref prop) = member.property {
                    if prop.name == "resolve" {
                        if !node.arguments.is_empty() {
                            let value = generate_csharp_expression(&node.arguments[0], false);
                            return Some(format!("Task.FromResult({})", value));
                        }
                        return Some("Task.CompletedTask".to_string());
                    }
                }
            }
        }
    }
    None
}

/**
 * Handle Promise.reject(error) → Task.FromException(error)
 */
pub fn handle_promise_reject<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref obj) = member.object {
            if obj.name == "Promise" {
                if let Expression::Identifier(ref prop) = member.property {
                    if prop.name == "reject" {
                        if !node.arguments.is_empty() {
                            let error = generate_csharp_expression(&node.arguments[0], false);
                            return Some(format!("Task.FromException(new Exception({}))", error));
                        }
                    }
                }
            }
        }
    }
    None
}
