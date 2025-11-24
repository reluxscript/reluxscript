/**
 * Array expression handlers
 *
 * Generates C# code for JavaScript/TypeScript array literals
 */

/**
 * Generate array expression
 *
 * @param node - ArrayExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @returns C# array/list literal code
 */
pub fn generate_array_expression<F>(
    node: &ArrayExpression,
    generate_csharp_expression: F
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    // Check if array contains spread elements
    let has_spread = node.elements.iter().any(|e| matches!(e, ArrayElement::SpreadElement(_)));

    if has_spread {
        // Handle spread operator: [...array, item] → array.Concat(new[] { item }).ToList()
        let mut parts: Vec<Str> = vec![];
        let mut current_literal: Vec<&Expression> = vec![];

        for element in &node.elements {
            match element {
                ArrayElement::SpreadElement(ref spread) => {
                    // Flush current literal elements
                    if !current_literal.is_empty() {
                        let literal_elements: Vec<Str> = current_literal
                            .iter()
                            .map(|e| generate_csharp_expression(e, false))
                            .collect();
                        let literal_code = literal_elements.join(", ");
                        parts.push(format!("new[] {{ {} }}", literal_code));
                        current_literal.clear();
                    }
                    // Add spread array
                    parts.push(generate_csharp_expression(&spread.argument, false));
                }
                ArrayElement::Expression(ref expr) => {
                    current_literal.push(expr);
                }
                ArrayElement::Hole => {
                    // Handle array holes (sparse arrays)
                    // For now, treat as null
                    current_literal.push(&Expression::NullLiteral);
                }
            }
        }

        // Flush remaining literals
        if !current_literal.is_empty() {
            let literal_elements: Vec<Str> = current_literal
                .iter()
                .map(|e| generate_csharp_expression(e, false))
                .collect();
            let literal_code = literal_elements.join(", ");
            parts.push(format!("new[] {{ {} }}", literal_code));
        }

        // Combine with Concat
        if parts.len() == 1 {
            format!("{}.ToList()", parts[0])
        } else {
            let first = &parts[0];
            let concats: Vec<Str> = parts[1..]
                .iter()
                .map(|p| format!(".Concat({})", p))
                .collect();
            format!("{}{}.ToList()", first, concats.join(""))
        }
    } else {
        // No spread - simple array literal
        let elements: Vec<Str> = node.elements
            .iter()
            .filter_map(|e| match e {
                ArrayElement::Expression(ref expr) => Some(generate_csharp_expression(expr, false)),
                ArrayElement::Hole => Some("null".to_string()),
                _ => None,
            })
            .collect();

        let elements_str = elements.join(", ");

        // Infer type from first element if all are string literals
        let all_strings = node.elements.iter().all(|e| {
            matches!(e, ArrayElement::Expression(Expression::StringLiteral(_)))
        });

        if !node.elements.is_empty() && all_strings {
            format!("new List<string> {{ {} }}", elements_str)
        } else {
            // Use List<dynamic> for empty arrays to be compatible with dynamic LINQ results
            let list_type = if elements.is_empty() { "dynamic" } else { "object" };
            format!("new List<{}> {{ {} }}", list_type, elements_str)
        }
    }
}
