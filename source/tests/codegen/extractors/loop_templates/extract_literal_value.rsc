/**
 * Extract literal value from expression
 *
 * Extracts the underlying value from literal expressions
 */

/**
 * Extract the value from a literal expression
 *
 * Supports:
 * - StringLiteral → String value
 * - NumericLiteral → Number value (as string)
 * - BooleanLiteral → "true" or "false"
 * - NullLiteral → "null"
 *
 * @param expr - The expression to extract from
 * @returns The literal value as a string, or None for complex expressions
 */
pub fn extract_literal_value(expr: &Expression) -> Option<Str> {
    match expr {
        Expression::StringLiteral(ref str_lit) => {
            Some(str_lit.value.clone())
        }

        Expression::NumericLiteral(ref num_lit) => {
            Some(num_lit.value.to_string())
        }

        Expression::BooleanLiteral(ref bool_lit) => {
            Some(if bool_lit.value { "true" } else { "false" }.to_string())
        }

        Expression::NullLiteral => {
            Some("null".to_string())
        }

        _ => {
            // Complex expression - not a literal
            None
        }
    }
}
