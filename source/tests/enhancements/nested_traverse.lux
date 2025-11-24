/// Test: Nested Traverse Support
///
/// ReluxScript should support multiple traverse blocks within a single function,
/// each with their own local state and visitor methods.

plugin NestedTraverseTest {
    struct ExtractedData {
        hooks: Vec<Str>,
        templates: Vec<Str>,
    }

    /// Test multiple sequential traverse blocks
    pub fn visit_function_declaration(node: &FunctionDeclaration) -> ExtractedData {
        let mut data = ExtractedData {
            hooks: vec![],
            templates: vec![],
        };

        if let Some(body) = &node.body {
            // First pass: extract hooks
            traverse(body) {
                let hook_count = 0;

                fn visit_call_expression(call: &CallExpression) {
                    if matches!(call.callee, Identifier) {
                        let name = call.callee.name.clone();
                        if name.starts_with("use") {
                            data.hooks.push(name);
                            hook_count += 1;
                        }
                    }
                }
            }

            // Second pass: extract templates
            traverse(body) {
                fn visit_jsx_element(jsx: &JSXElement) {
                    let tag = jsx.opening.name.clone();
                    data.templates.push(tag);
                }
            }
        }

        return data;
    }

    /// Test nested traverse with capturing syntax
    pub fn visit_program(node: &Program) {
        let mut components: Vec<Str> = vec![];
        let mut total_hooks = 0;

        for item in &node.body {
            if matches!(item, FunctionDeclaration) {
                let func = item.clone();

                // Traverse with explicit capture
                traverse(func.body) capturing [&mut components, &mut total_hooks] {
                    fn visit_call_expression(call: &CallExpression) {
                        if matches!(call.callee, Identifier) {
                            if call.callee.name.starts_with("use") {
                                total_hooks += 1;
                            }
                        }
                    }
                }

                components.push(func.id.name.clone());
            }
        }
    }
}
