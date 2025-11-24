/**
 * Extract Literal Value
 *
 * Extracts literal values from AST nodes (string, number, boolean)
 */

/**
 * Extract literal value from node (string, number, boolean)
 * Returns the value as a string, or None if not a literal
 */
pub fn extract_literal_value(node: &Expression) -> Option<Str> {
    match node {
        Expression::StringLiteral(ref str_lit) => {
            Some(str_lit.value.clone())
        }

        Expression::NumericLiteral(ref num_lit) => {
            Some(num_lit.value.to_string())
        }

        Expression::BooleanLiteral(ref bool_lit) => {
            Some(bool_lit.value.to_string())
        }

        _ => None
    }
}
