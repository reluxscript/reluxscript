//! Shared type system utilities
//!
//! This module contains type representations and utilities that are used
//! by both semantic analysis and code generation layers.

mod type_context;
mod type_environment;

pub use type_context::{TypeContext, SwcTypeKind, classify_swc_type, map_reluxscript_to_swc};
pub use type_environment::TypeEnvironment;

// Re-export field mapping functions that work with types
pub use crate::mapping::{get_field_mapping, FieldMapping};

/// Get typed field mapping with SWC type information
pub fn get_typed_field_mapping(
    node_type: &str,
    field_name: &str,
) -> Option<&'static FieldMapping> {
    get_field_mapping(node_type, field_name)
}

/// Infer the expected variant for a field access
/// Returns None if the field doesn't require unwrapping
pub fn infer_expected_variant(
    base_type: &str,
    field_name: &str,
) -> Option<String> {
    let mapping = get_field_mapping(base_type, field_name)?;

    // If the field is a wrapper enum, we might be able to infer the variant
    // This is a simplified version - real implementation would use more heuristics
    if mapping.swc_type.contains("Expr") && !mapping.swc_type.contains("Box<") {
        // Common pattern: if accessing a field that should be an identifier
        if field_name == "name" || field_name == "id" {
            return Some("Ident".to_string());
        }
    }

    None
}
