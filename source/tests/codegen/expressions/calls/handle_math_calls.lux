/**
 * Handle Math.* method calls
 *
 * Converts JavaScript Math API to C# Math API
 */

/**
 * Handle Math.* method calls
 *
 * @param node - CallExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @returns C# Math method call or None
 */
pub fn handle_math_calls<F>(
    node: &CallExpression,
    generate_csharp_expression: F
) -> Option<Str>
where
    F: Fn(&Expression, bool) -> Str
{
    // Check if callee is Math.*
    if let Expression::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref obj) = member.object {
            if obj.name != "Math" {
                return None;
            }

            // Get method name
            let method_name = if let Expression::Identifier(ref prop) = member.property {
                &prop.name
            } else {
                return None;
            };

            // Generate arguments
            let args: Vec<Str> = node.arguments
                .iter()
                .map(|arg| generate_csharp_expression(arg, false))
                .collect();
            let args_str = args.join(", ");

            // Handle Math.max() → Math.Max()
            if method_name == "max" {
                return Some(format!("Math.Max({})", args_str));
            }

            // Handle Math.min() → Math.Min()
            if method_name == "min" {
                return Some(format!("Math.Min({})", args_str));
            }

            // Handle other Math methods (floor, ceil, round, pow, log, etc.) → Pascal case
            let pascal_method_name = capitalize_first_letter(method_name);

            // Cast floor/ceil/round to int for array indexing compatibility
            if method_name == "floor" || method_name == "ceil" || method_name == "round" {
                return Some(format!("(int)Math.{}({})", pascal_method_name, args_str));
            }

            return Some(format!("Math.{}({})", pascal_method_name, args_str));
        }
    }

    None
}

/**
 * Capitalize first letter of a string
 */
fn capitalize_first_letter(s: &str) -> Str {
    if s.is_empty() {
        return s.to_string();
    }
    let mut chars = s.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    first + chars.as_str()
}
