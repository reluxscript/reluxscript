/**
 * Loop Templates Module
 *
 * Extracts loop template information from .map() calls in JSX.
 * This module provides functions to analyze and extract structured
 * templates from JavaScript array iteration patterns.
 */

// Core path and binding utilities
pub use "./build_member_expression_path.rsc" { build_member_expression_path };
pub use "./build_binding_path.rsc" { build_binding_path };
pub use "./extract_literal_value.rsc" { extract_literal_value };
pub use "./extract_array_binding.rsc" { extract_array_binding };

// Expression analysis
pub use "./extract_loop_identifiers.rsc" { extract_loop_identifiers };
pub use "./extract_loop_expressions.rsc" {
    extract_loop_binary_expression,
    extract_loop_logical_expression,
    extract_loop_unary_expression,
    extract_loop_call_expression
};

// Template extraction
pub use "./extract_conditional_template.rsc" { extract_conditional_template, ConditionalTemplate };
pub use "./extract_template_from_template_literal.rsc" {
    extract_template_from_template_literal,
    TemplateLiteralResult
};
pub use "./extract_text_template.rsc" { extract_text_template, TextTemplate };
pub use "./extract_prop_templates.rsc" { extract_prop_templates, PropTemplate };

// JSX structure extraction
pub use "./extract_children_templates.rsc" { extract_children_templates, ChildTemplate };
pub use "./extract_element_template.rsc" { extract_element_template, ElementTemplate };
pub use "./extract_key_binding.rsc" { extract_key_binding };
pub use "./extract_jsx_from_callback.rsc" { extract_jsx_from_callback };

// Loop template extraction (main)
pub use "./extract_loop_template.rsc" { extract_loop_template, LoopTemplate };

// Traversal and discovery
pub use "./find_map_expressions.rsc" { find_map_expressions };
pub use "./traverse_jsx.rsc" { traverse_jsx };
