/**
 * Type Conversion
 *
 * Converts TypeScript type annotations to C# types
 * and infers C# types from JavaScript/TypeScript values
 */

/**
 * Convert TypeScript type annotation to C# type
 *
 * Handles:
 * - Primitive types: string, number, boolean, any
 * - Array types: T[] -> List<T>
 * - Type references: custom types and @minimact/mvc type mappings
 * - Type literals: object types -> dynamic
 */
pub fn ts_type_to_csharp_type(ts_type: &TSType) -> Str {
    match ts_type {
        // TSStringKeyword -> string
        TSType::TSStringKeyword => "string",

        // TSNumberKeyword -> double
        TSType::TSNumberKeyword => "double",

        // TSBooleanKeyword -> bool
        TSType::TSBooleanKeyword => "bool",

        // TSAnyKeyword -> dynamic
        TSType::TSAnyKeyword => "dynamic",

        // TSArrayType -> List<T>
        TSType::TSArrayType(ref array_type) => {
            let element_type = ts_type_to_csharp_type(&array_type.element_type);
            format!("List<{}>", element_type)
        }

        // TSTypeLiteral (object type) -> dynamic
        TSType::TSTypeLiteral(_) => "dynamic",

        // TSTypeReference (custom types, interfaces)
        TSType::TSTypeReference(ref type_ref) => {
            // Handle @minimact/mvc type mappings
            if let TSEntityName::Identifier(ref type_name) = type_ref.type_name {
                // Map @minimact/mvc types to C# types
                match type_name.name.as_str() {
                    "decimal" => "decimal",
                    "int" | "int32" => "int",
                    "int64" | "long" => "long",
                    "float" | "float32" => "float",
                    "float64" | "double" => "double",
                    "short" | "int16" => "short",
                    "byte" => "byte",
                    "Guid" => "Guid",
                    "DateTime" => "DateTime",
                    "DateOnly" => "DateOnly",
                    "TimeOnly" => "TimeOnly",
                    _ => "dynamic",
                }
            } else {
                "dynamic"
            }
        }

        // Default to dynamic for full JSX semantics
        _ => "dynamic",
    }
}

/**
 * Infer C# type from JavaScript/TypeScript literal value
 *
 * Handles:
 * - String literals -> string
 * - Numeric literals -> int or double (based on whether it has decimals)
 * - Boolean literals -> bool
 * - Null literals -> dynamic
 * - Array expressions -> List<dynamic>
 * - Object expressions -> dynamic
 */
pub fn infer_type(node: &Expression) -> Str {
    match node {
        // String literal -> string
        Expression::StringLiteral(_) => "string",

        // Numeric literal -> int or double
        Expression::NumericLiteral(ref num_lit) => {
            // If the value is a whole number, use int; otherwise use double
            if num_lit.value.fract() == 0.0 {
                "int"
            } else {
                "double"
            }
        }

        // Boolean literal -> bool
        Expression::BooleanLiteral(_) => "bool",

        // Null literal -> dynamic
        Expression::NullLiteral => "dynamic",

        // Array expression -> List<dynamic>
        Expression::ArrayExpression(_) => "List<dynamic>",

        // Object expression -> dynamic
        Expression::ObjectExpression(_) => "dynamic",

        // Default to dynamic for other expressions
        _ => "dynamic",
    }
}

/**
 * Check if a numeric value is an integer (no fractional part)
 */
fn is_integer(value: f64) -> bool {
    value.fract() == 0.0
}
