/// Test: format! macro for string formatting
///
/// ReluxScript should support format! macro that compiles to
/// template literals in Babel and format!() in Rust.

plugin FormatMacroTest {
    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        let name = node.id.name.clone();

        // Simple format with single placeholder
        let msg = format!("Processing function: {}", name);

        // Multiple placeholders
        let param_count = node.params.len();
        let detailed = format!("Function {} has {} parameters", name, param_count);

        // With literals
        let prefixed = format!("fn_{}", name);

        // Complex formatting
        let summary = format!(
            "Function: {}\nParams: {}\nAsync: {}",
            name,
            param_count,
            node.is_async
        );
    }
}
