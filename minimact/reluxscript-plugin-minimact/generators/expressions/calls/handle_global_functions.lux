/**
 * Handle global function calls
 *
 * Converts JavaScript global functions to C# equivalents
 */

/**
 * Handle encodeURIComponent() → Uri.EscapeDataString()
 */
pub fn handle_encode_uri_component<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::Identifier(ref callee) = node.callee {
        if callee.name == "encodeURIComponent" {
            let args: Vec<Str> = node.arguments
                .iter()
                .map(|arg| generate_csharp_expression(arg, false))
                .collect();
            return Some(format!("Uri.EscapeDataString({})", args.join(", ")));
        }
    }
    None
}

/**
 * Handle setState(key, value) → SetState(key, value)
 */
pub fn handle_set_state<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::Identifier(ref callee) = node.callee {
        if callee.name == "setState" {
            if node.arguments.len() >= 2 {
                let key = generate_csharp_expression(&node.arguments[0], false);
                let value = generate_csharp_expression(&node.arguments[1], false);
                return Some(format!("SetState({}, {})", key, value));
            } else {
                console_warn("[Babel Plugin] setState requires 2 arguments (key, value)");
                return Some("SetState(\"\", null)".to_string());
            }
        }
    }
    None
}

/**
 * Handle fetch() → HttpClient call
 */
pub fn handle_fetch<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::Identifier(ref callee) = node.callee {
        if callee.name == "fetch" {
            let url = if !node.arguments.is_empty() {
                generate_csharp_expression(&node.arguments[0], false)
            } else {
                "\"\"".to_string()
            };
            return Some(format!("new HttpClient().GetAsync({})", url));
        }
    }
    None
}

/**
 * Handle alert() → Console.WriteLine()
 */
pub fn handle_alert<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::Identifier(ref callee) = node.callee {
        if callee.name == "alert" {
            let args: Vec<Str> = node.arguments
                .iter()
                .map(|arg| generate_csharp_expression(arg, false))
                .collect();
            let args_str = args.join(" + ");
            return Some(format!("Console.WriteLine({})", args_str));
        }
    }
    None
}

/**
 * Handle String(value) → value.ToString()
 */
pub fn handle_string_constructor<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::Identifier(ref callee) = node.callee {
        if callee.name == "String" {
            if !node.arguments.is_empty() {
                let arg = generate_csharp_expression(&node.arguments[0], false);
                return Some(format!("({}).ToString()", arg));
            }
            return Some("\"\"".to_string());
        }
    }
    None
}

/**
 * Console warning helper
 */
fn console_warn(message: &str) {
    // Placeholder for warning/logging functionality
}
