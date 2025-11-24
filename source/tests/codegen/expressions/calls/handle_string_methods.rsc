/**
 * Handle string method calls
 *
 * Converts JavaScript string methods to C# string methods
 */

/**
 * Handle .toFixed(n) → .ToString("Fn")
 */
pub fn handle_to_fixed<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name == "toFixed" {
                let mut object = generate_csharp_expression(&member.object, false);

                // Preserve parentheses for complex expressions
                let needs_parens = matches!(
                    member.object,
                    Expression::BinaryExpression(_)
                        | Expression::LogicalExpression(_)
                        | Expression::ConditionalExpression(_)
                        | Expression::CallExpression(_)
                );

                if needs_parens {
                    object = format!("({})", object);
                }

                // Get decimal places (default 2)
                let decimals = if !node.arguments.is_empty() {
                    if let Expression::NumericLiteral(ref num) = node.arguments[0] {
                        num.value as i32
                    } else {
                        2
                    }
                } else {
                    2
                };

                return Some(format!("{}.ToString(\"F{}\")", object, decimals));
            }
        }
    }
    None
}

/**
 * Handle .toLocaleString() → .ToString("g")
 */
pub fn handle_to_locale_string<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name == "toLocaleString" {
                let object = generate_csharp_expression(&member.object, false);
                return Some(format!("{}.ToString(\"g\")", object));
            }
        }
    }
    None
}

/**
 * Handle .toLowerCase() → .ToLower()
 */
pub fn handle_to_lower_case<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name == "toLowerCase" {
                let object = generate_csharp_expression(&member.object, false);
                return Some(format!("{}.ToLower()", object));
            }
        }
    }
    None
}

/**
 * Handle .toUpperCase() → .ToUpper()
 */
pub fn handle_to_upper_case<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name == "toUpperCase" {
                let object = generate_csharp_expression(&member.object, false);
                return Some(format!("{}.ToUpper()", object));
            }
        }
    }
    None
}

/**
 * Handle .trim() → .Trim()
 */
pub fn handle_trim<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name == "trim" {
                let object = generate_csharp_expression(&member.object, false);
                return Some(format!("{}.Trim()", object));
            }
        }
    }
    None
}

/**
 * Handle .substring(start, end) → .Substring(start, end)
 */
pub fn handle_substring<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name == "substring" {
                let object = generate_csharp_expression(&member.object, false);
                let args: Vec<Str> = node.arguments
                    .iter()
                    .map(|arg| generate_csharp_expression(arg, false))
                    .collect();
                return Some(format!("{}.Substring({})", object, args.join(", ")));
            }
        }
    }
    None
}

/**
 * Handle .padStart(length, char) → .PadLeft(length, char)
 */
pub fn handle_pad_start<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name == "padStart" {
                let object = generate_csharp_expression(&member.object, false);

                let length = if !node.arguments.is_empty() {
                    generate_csharp_expression(&node.arguments[0], false)
                } else {
                    "0".to_string()
                };

                let mut pad_char = if node.arguments.len() > 1 {
                    generate_csharp_expression(&node.arguments[1], false)
                } else {
                    "\" \"".to_string()
                };

                // Convert string literal "0" to char literal '0'
                if node.arguments.len() > 1 {
                    if let Expression::StringLiteral(ref str_lit) = node.arguments[1] {
                        if str_lit.value.len() == 1 {
                            pad_char = format!("'{}'", str_lit.value);
                        }
                    }
                }

                return Some(format!("{}.PadLeft({}, {})", object, length, pad_char));
            }
        }
    }
    None
}

/**
 * Handle .padEnd(length, char) → .PadRight(length, char)
 */
pub fn handle_pad_end<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name == "padEnd" {
                let object = generate_csharp_expression(&member.object, false);

                let length = if !node.arguments.is_empty() {
                    generate_csharp_expression(&node.arguments[0], false)
                } else {
                    "0".to_string()
                };

                let mut pad_char = if node.arguments.len() > 1 {
                    generate_csharp_expression(&node.arguments[1], false)
                } else {
                    "\" \"".to_string()
                };

                // Convert string literal "0" to char literal '0'
                if node.arguments.len() > 1 {
                    if let Expression::StringLiteral(ref str_lit) = node.arguments[1] {
                        if str_lit.value.len() == 1 {
                            pad_char = format!("'{}'", str_lit.value);
                        }
                    }
                }

                return Some(format!("{}.PadRight({}, {})", object, length, pad_char));
            }
        }
    }
    None
}

/**
 * Handle response.json() → response.Content.ReadFromJsonAsync<dynamic>()
 */
pub fn handle_response_json<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref prop) = member.property {
            if prop.name == "json" {
                let object = generate_csharp_expression(&member.object, false);
                return Some(format!("{}.Content.ReadFromJsonAsync<dynamic>()", object));
            }
        }
    }
    None
}
