/**
 * Style Converter
 *
 * Converts JavaScript style objects to CSS strings
 * Handles camelCase to kebab-case conversion for CSS properties
 */

/**
 * Convert camelCase to kebab-case
 * Example: marginTop -> margin-top
 */
pub fn camel_to_kebab(s: &Str) -> Str {
    let mut result = String::new();

    for ch in s.chars() {
        if ch.is_uppercase() {
            result.push('-');
            result.push(ch.to_lowercase().next().unwrap());
        } else {
            result.push(ch);
        }
    }

    result
}

/**
 * Convert a style value to CSS string
 *
 * Handles:
 * - String literals: use the string value directly
 * - Numeric literals: append 'px' for pixel values
 * - Identifiers: use the identifier name
 */
pub fn convert_style_value(value: &Expression) -> Str {
    match value {
        Expression::StringLiteral(ref str_lit) => {
            str_lit.value.clone()
        }

        Expression::NumericLiteral(ref num_lit) => {
            // Add 'px' for numeric values (except certain properties like opacity, zIndex)
            format!("{}px", num_lit.value)
        }

        Expression::Identifier(ref id) => {
            id.name.clone()
        }

        _ => {
            // For other expression types, return a placeholder
            "dynamic"
        }
    }
}

/**
 * Convert a JavaScript style object expression to CSS string
 * Example: { marginTop: '12px', color: 'red' } -> "margin-top: 12px; color: red;"
 *
 * Returns: CSS string with semicolon-separated properties
 */
pub fn convert_style_object_to_css(object_expr: &ObjectExpression) -> Result<Str, Str> {
    let mut css_properties = vec![];

    for prop in &object_expr.properties {
        // Only handle non-computed object properties
        if let ObjectProperty::Property(ref obj_prop) = prop {
            if !obj_prop.computed {
                // Get the key name
                let key = match &obj_prop.key {
                    Expression::Identifier(ref id) => id.name.clone(),
                    Expression::StringLiteral(ref str_lit) => str_lit.value.clone(),
                    _ => continue, // Skip computed or other key types
                };

                // Convert key to kebab-case
                let css_key = camel_to_kebab(&key);

                // Convert value to CSS string
                let css_value = convert_style_value(&obj_prop.value);

                // Add to properties list
                css_properties.push(format!("{}: {}", css_key, css_value));
            }
        }
    }

    Ok(css_properties.join("; "))
}

/**
 * Check if a property should not have 'px' appended
 * Properties like opacity, zIndex, etc. should remain unitless
 */
fn is_unitless_property(property: &Str) -> bool {
    match property.as_str() {
        "opacity" | "z-index" | "font-weight" | "line-height" | "flex" | "flex-grow" | "flex-shrink" | "order" => true,
        _ => false,
    }
}
