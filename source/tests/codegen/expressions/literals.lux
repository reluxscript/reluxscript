/**
 * Literal expression handlers
 *
 * Generates C# code for JavaScript/TypeScript literal values
 */

use "../../utils/helpers.rsc" { escape_csharp_string };

/**
 * Generate string literal
 *
 * @param node - StringLiteral node
 * @param in_interpolation - Whether this is inside string interpolation
 * @returns C# string literal code
 */
pub fn generate_string_literal(node: &StringLiteral, in_interpolation: bool) -> Str {
    let escaped = escape_csharp_string(&node.value);

    // In string interpolation context, escape the quotes: \"text\"
    // Otherwise use normal quotes: "text"
    if in_interpolation {
        format!("\\\"{}\\\"", escaped)
    } else {
        format!("\"{}\"", escaped)
    }
}

/**
 * Generate numeric literal
 *
 * @param node - NumericLiteral node
 * @returns C# numeric literal code
 */
pub fn generate_numeric_literal(node: &NumericLiteral) -> Str {
    node.value.to_string()
}

/**
 * Generate boolean literal
 *
 * @param node - BooleanLiteral node
 * @returns C# boolean literal code ("true" or "false")
 */
pub fn generate_boolean_literal(node: &BooleanLiteral) -> Str {
    if node.value {
        "true"
    } else {
        "false"
    }
}

/**
 * Generate null literal
 *
 * @param node - NullLiteral node
 * @returns C# VNull expression with path tracking
 */
pub fn generate_null_literal(node: &NullLiteral) -> Str {
    // Get the minimact path for debugging/tracking
    let node_path = if let Some(ref path) = node.extra_data.get("__minimactPath") {
        path.clone()
    } else {
        String::new()
    };

    format!("new VNull(\"{}\")", node_path)
}

/**
 * Generate template literal (template string)
 *
 * @param node - TemplateLiteral node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @returns C# interpolated string or verbatim string
 */
pub fn generate_template_literal<F>(
    node: &TemplateLiteral,
    generate_csharp_expression: F
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    // If no expressions, use verbatim string literal (@"...") to avoid escaping issues
    if node.expressions.is_empty() {
        if let Some(first_quasi) = node.quasis.first() {
            let text = &first_quasi.value.raw;
            // Use verbatim string literal (@"...") for multiline or strings with special chars
            // Escape " as "" in verbatim strings
            let escaped = text.replace("\"", "\"\"");
            return format!("@\"{}\"", escaped);
        }
    }

    // Has expressions - use C# string interpolation
    let mut result = String::from("$\"");

    for (i, quasi) in node.quasis.iter().enumerate() {
        // Escape special chars in C# interpolated strings
        let mut text = quasi.value.raw.clone();

        // Escape { and } by doubling them
        text = text.replace("{", "{{").replace("}", "}}");

        // Escape " as \"
        text = text.replace("\"", "\\\"");

        result.push_str(&text);

        // Add expression if not the last quasi
        if i < node.expressions.len() {
            if let Some(ref expr) = node.expressions.get(i) {
                // Wrap conditional (ternary) expressions in parentheses to avoid ':' conflict in C# interpolation
                let expr_code = generate_csharp_expression(expr, false);
                let needs_parens = matches!(expr, Expression::ConditionalExpression(_));

                result.push('{');
                if needs_parens {
                    result.push('(');
                    result.push_str(&expr_code);
                    result.push(')');
                } else {
                    result.push_str(&expr_code);
                }
                result.push('}');
            }
        }
    }

    result.push('"');
    result
}
