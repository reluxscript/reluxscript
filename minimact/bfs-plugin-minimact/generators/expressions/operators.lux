/**
 * Operator expression handlers
 *
 * Generates C# code for JavaScript/TypeScript operators
 */

/**
 * Generate unary expression
 *
 * @param node - UnaryExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @param in_interpolation - Whether inside string interpolation
 * @returns C# unary expression code
 */
pub fn generate_unary_expression<F>(
    node: &UnaryExpression,
    generate_csharp_expression: F,
    in_interpolation: bool
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    let argument = generate_csharp_expression(&node.argument, in_interpolation);
    let operator = &node.operator;
    format!("{}{}", operator, argument)
}

/**
 * Get operator precedence (higher = tighter binding)
 */
fn get_precedence(op: &str) -> i32 {
    match op {
        "*" | "/" | "%" => 3,
        "+" | "-" => 2,
        "==" | "!=" | "===" | "!==" | "<" | ">" | "<=" | ">=" => 1,
        _ => 0,
    }
}

/**
 * Generate binary expression
 *
 * @param node - BinaryExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @returns C# binary expression code
 */
pub fn generate_binary_expression<F>(
    node: &BinaryExpression,
    generate_csharp_expression: F
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    let current_precedence = get_precedence(&node.operator);

    // Generate left side, wrap in parentheses if needed
    let mut left = generate_csharp_expression(&node.left, false);
    if let Expression::BinaryExpression(ref left_bin) = node.left {
        let left_precedence = get_precedence(&left_bin.operator);
        // Wrap in parentheses if left has lower precedence
        if left_precedence < current_precedence {
            left = format!("({})", left);
        }
    }

    // Generate right side, wrap in parentheses if needed
    let mut right = generate_csharp_expression(&node.right, false);
    if let Expression::BinaryExpression(ref right_bin) = node.right {
        let right_precedence = get_precedence(&right_bin.operator);
        // Wrap in parentheses if right has lower or equal precedence
        // Equal precedence on right needs parens for left-associative operators
        if right_precedence <= current_precedence {
            right = format!("({})", right);
        }
    }

    // Convert JavaScript operators to C# operators
    let operator = match node.operator.as_str() {
        "===" => "==",
        "!==" => "!=",
        _ => &node.operator,
    };

    format!("{} {} {}", left, operator, right)
}

/**
 * Generate logical expression
 *
 * @param node - LogicalExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @returns C# logical expression code
 */
pub fn generate_logical_expression<F>(
    node: &LogicalExpression,
    generate_csharp_expression: F
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    let left = generate_csharp_expression(&node.left, false);
    let right = generate_csharp_expression(&node.right, false);

    match node.operator.as_str() {
        "||" => {
            // JavaScript: a || b
            // C#: a ?? b (null coalescing)
            format!("({}) ?? ({})", left, right)
        }
        "&&" => {
            // Check if right side is a boolean expression (comparison, logical, etc.)
            let right_is_boolean_expr = matches!(
                node.right,
                Expression::BinaryExpression(_)
                    | Expression::LogicalExpression(_)
                    | Expression::UnaryExpression(_)
            );

            if right_is_boolean_expr {
                // JavaScript: a && (b > 0)
                // C#: (a) && (b > 0) - boolean AND
                format!("({}) && ({})", left, right)
            } else {
                // JavaScript: a && <jsx> or a && someValue
                // C#: a != null ? value : VNull (for objects)
                let node_path = if let Some(ref path) = node.extra_data.get("__minimactPath") {
                    path.clone()
                } else {
                    String::new()
                };
                format!("({}) != null ? ({}) : new VNull(\"{}\")", left, right, node_path)
            }
        }
        _ => {
            format!("{} {} {}", left, node.operator, right)
        }
    }
}

/**
 * Generate conditional (ternary) expression
 *
 * @param node - ConditionalExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @returns C# conditional expression code
 */
pub fn generate_conditional_expression<F>(
    node: &ConditionalExpression,
    generate_csharp_expression: F
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    // Handle ternary operator: test ? consequent : alternate
    // Children are always in normal C# expression context, not interpolation context
    let test = generate_csharp_expression(&node.test, false);
    let consequent = generate_csharp_expression(&node.consequent, false);
    let alternate = generate_csharp_expression(&node.alternate, false);

    format!("({}) ? {} : {}", test, consequent, alternate)
}

/**
 * Generate assignment expression
 *
 * @param node - AssignmentExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @param in_interpolation - Whether inside string interpolation
 * @returns C# assignment expression code
 */
pub fn generate_assignment_expression<F>(
    node: &AssignmentExpression,
    generate_csharp_expression: F,
    in_interpolation: bool
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    let left = generate_csharp_expression(&node.left, in_interpolation);
    let right = generate_csharp_expression(&node.right, in_interpolation);
    let operator = &node.operator; // =, +=, -=, etc.

    format!("{} {} {}", left, operator, right)
}
