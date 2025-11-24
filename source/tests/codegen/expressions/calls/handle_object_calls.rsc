/**
 * Handle Object.* and other object-related calls
 *
 * Converts JavaScript Object API and Date/console methods to C# equivalents
 */

/**
 * Handle Object.keys() → dictionary.Keys
 */
pub fn handle_object_keys<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref obj) = member.object {
            if obj.name == "Object" {
                if let Expression::Identifier(ref prop) = member.property {
                    if prop.name == "keys" {
                        if !node.arguments.is_empty() {
                            let obj_arg = generate_csharp_expression(&node.arguments[0], false);
                            return Some(format!("((IDictionary<string, object>){}).Keys", obj_arg));
                        }
                    }
                }
            }
        }
    }
    None
}

/**
 * Handle Date.now() → DateTimeOffset.Now.ToUnixTimeMilliseconds()
 */
pub fn handle_date_now(node: &CallExpression) -> Option<Str> {
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref obj) = member.object {
            if obj.name == "Date" {
                if let Expression::Identifier(ref prop) = member.property {
                    if prop.name == "now" {
                        return Some("DateTimeOffset.Now.ToUnixTimeMilliseconds()".to_string());
                    }
                }
            }
        }
    }
    None
}

/**
 * Handle console.log → Console.WriteLine
 */
pub fn handle_console_log<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref obj) = member.object {
            if obj.name == "console" {
                if let Expression::Identifier(ref prop) = member.property {
                    if prop.name == "log" {
                        let args: Vec<Str> = node.arguments
                            .iter()
                            .map(|arg| generate_csharp_expression(arg, false))
                            .collect();
                        let args_str = args.join(" + ");
                        return Some(format!("Console.WriteLine({})", args_str));
                    }
                }
            }
        }
    }
    None
}
