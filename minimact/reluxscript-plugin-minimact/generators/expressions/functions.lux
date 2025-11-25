/**
 * Function expression handlers
 *
 * Generates C# code for JavaScript/TypeScript arrow functions and function expressions
 */

/**
 * Generate arrow function or function expression
 *
 * @param node - ArrowFunctionExpression or FunctionExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @param generate_csharp_statement - Function to generate C# statement code
 * @returns C# lambda expression code
 */
pub fn generate_function_expression<F, G>(
    node: &FunctionExpression,
    generate_csharp_expression: F,
    generate_csharp_statement: G
) -> Str
where
    F: Fn(&Expression, bool) -> Str,
    G: Fn(&Statement) -> Str
{
    // Arrow function: (x) => x * 2  →  x => x * 2
    // Function expression: function(x) { return x * 2; }  →  x => x * 2

    // Extract parameter names
    let params: Vec<Str> = node.params
        .iter()
        .map(|p| match p {
            Pattern::Identifier(ref id) => id.name.clone(),
            Pattern::ObjectPattern(_) => "{...}".to_string(), // Destructuring - simplified
            Pattern::ArrayPattern(_) => "[...]".to_string(),
            _ => "param".to_string(),
        })
        .collect();

    // Wrap params in parentheses if multiple or none
    let params_string = if node.params.len() == 1 {
        params.join(", ")
    } else {
        format!("({})", params.join(", "))
    };

    // Generate function body
    let body = match &node.body {
        FunctionBody::BlockStatement(ref block) => {
            // Block body: (x) => { return x * 2; }
            let statements: Vec<Str> = block.body
                .iter()
                .map(|stmt| generate_csharp_statement(stmt))
                .collect();
            format!("{{ {} }}", statements.join(" "))
        }
        FunctionBody::Expression(ref expr) => {
            // Expression body: (x) => x * 2
            generate_csharp_expression(expr, false)
        }
    };

    format!("{} => {}", params_string, body)
}

/**
 * Generate arrow function expression
 *
 * @param node - ArrowFunctionExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @param generate_csharp_statement - Function to generate C# statement code
 * @returns C# lambda expression code
 */
pub fn generate_arrow_function_expression<F, G>(
    node: &ArrowFunctionExpression,
    generate_csharp_expression: F,
    generate_csharp_statement: G
) -> Str
where
    F: Fn(&Expression, bool) -> Str,
    G: Fn(&Statement) -> Str
{
    // Extract parameter names
    let params: Vec<Str> = node.params
        .iter()
        .map(|p| match p {
            Pattern::Identifier(ref id) => id.name.clone(),
            Pattern::ObjectPattern(_) => "{...}".to_string(),
            Pattern::ArrayPattern(_) => "[...]".to_string(),
            _ => "param".to_string(),
        })
        .collect();

    // Wrap params in parentheses if multiple or none
    let params_string = if node.params.len() == 1 {
        params.join(", ")
    } else {
        format!("({})", params.join(", "))
    };

    // Generate function body
    let body = if node.is_expression {
        // Expression body: (x) => x * 2
        if let Some(ref expr) = node.body_expression {
            generate_csharp_expression(expr, false)
        } else {
            "/* empty */".to_string()
        }
    } else {
        // Block body: (x) => { return x * 2; }
        if let Some(ref block) = node.body_block {
            let statements: Vec<Str> = block.body
                .iter()
                .map(|stmt| generate_csharp_statement(stmt))
                .collect();
            format!("{{ {} }}", statements.join(" "))
        } else {
            "{}".to_string()
        }
    };

    format!("{} => {}", params_string, body)
}
