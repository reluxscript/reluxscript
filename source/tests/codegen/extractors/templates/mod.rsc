/**
 * Templates Module
 *
 * Re-exports all template extraction utilities
 */

// Member path building
pub use "./build_member_path.rsc" {
    build_member_path,
    build_optional_member_path
};

// Identifier extraction
pub use "./extract_identifiers.rsc" {
    extract_identifiers
};

// Literal value extraction
pub use "./extract_literal_value.rsc" {
    extract_literal_value
};

// Method call binding
pub use "./extract_method_call_binding.rsc" {
    extract_method_call_binding,
    MethodCallBinding
};

// Conditional binding
pub use "./extract_conditional_binding.rsc" {
    extract_conditional_binding,
    ConditionalBinding
};

// Optional chain binding
pub use "./extract_optional_chain_binding.rsc" {
    extract_optional_chain_binding,
    OptionalChainBinding
};

// Main binding extraction
pub use "./extract_binding.rsc" {
    extract_binding,
    Binding
};

// Shared binding utilities
pub use "./extract_binding_shared.rsc" {
    extract_binding_shared,
    BindingResult,
    TransformBinding
};

// Template literal extraction
pub use "./extract_template_literal.rsc" {
    extract_template_literal,
    TemplateLiteralResult,
    TemplateLiteralTransform
};

// Style helpers
pub use "./style_helpers.rsc" {
    camel_to_kebab,
    convert_style_value
};

// Style object template
pub use "./extract_style_object_template.rsc" {
    extract_style_object_template,
    StyleTemplate
};

// Text template extraction
pub use "./extract_text_template.rsc" {
    extract_text_template,
    TextTemplate,
    ConditionalTemplates,
    TransformMetadata
};

// Path helpers
pub use "./path_helpers.rsc" {
    build_path_key,
    build_attribute_path_key
};

// Map call expression detection
pub use "./is_map_call_expression.rsc" {
    is_map_call_expression
};
