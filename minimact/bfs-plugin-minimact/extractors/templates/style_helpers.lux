/**
 * Style Helpers
 *
 * Helper functions for converting style values to CSS
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
 * Convert style value to CSS string
 */
pub fn convert_style_value(value: &Expression) -> Str {
    match value {
        Expression::StringLiteral(ref str_lit) => {
            str_lit.value.clone()
        }

        Expression::NumericLiteral(ref num_lit) => {
            format!("{}px", num_lit.value)
        }

        Expression::Identifier(ref id) => {
            id.name.clone()
        }

        _ => String::from("dynamic")
    }
}
