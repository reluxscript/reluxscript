/**
 * Object expression handlers
 *
 * Generates C# code for JavaScript/TypeScript object literals
 */

/**
 * Generate object expression
 *
 * @param node - ObjectExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @returns C# anonymous object or Dictionary literal code
 */
pub fn generate_object_expression<F>(
    node: &ObjectExpression,
    generate_csharp_expression: F
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    // Convert JS object literal to C# anonymous object or Dictionary
    // Check if any key has hyphens (invalid for C# anonymous types)
    let has_hyphenated_keys = node.properties.iter().any(|prop| {
        if let ObjectProperty::Property(ref obj_prop) = prop {
            let key = match &obj_prop.key {
                Expression::Identifier(ref id) => id.name.as_str(),
                Expression::StringLiteral(ref str_lit) => str_lit.value.as_str(),
                _ => "",
            };
            key.contains('-')
        } else {
            false
        }
    });

    let mut properties: Vec<Str> = vec![];

    for prop in &node.properties {
        if let ObjectProperty::Property(ref obj_prop) = prop {
            let key = match &obj_prop.key {
                Expression::Identifier(ref id) => id.name.clone(),
                Expression::StringLiteral(ref str_lit) => str_lit.value.clone(),
                _ => continue,
            };

            let value = generate_csharp_expression(&obj_prop.value, false);

            if has_hyphenated_keys {
                // Use Dictionary syntax with quoted keys
                properties.push(format!("[\"{}\"] = {}", key, value));
            } else {
                // Use anonymous object syntax
                properties.push(format!("{} = {}", key, value));
            }
        }
    }

    if properties.is_empty() {
        return "null".to_string();
    }

    let properties_str = properties.join(", ");

    if has_hyphenated_keys {
        format!("new Dictionary<string, object> {{ {} }}", properties_str)
    } else {
        format!("new {{ {} }}", properties_str)
    }
}
