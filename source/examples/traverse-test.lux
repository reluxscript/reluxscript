/// Test plugin demonstrating the traverse construct
plugin TraverseTest {
    /// Find and strip return values only inside if statements
    fn visit_function_declaration(func: &mut FunctionDeclaration, ctx: &Context) {
        // Manually iterate over statements
        for stmt in &mut func.body.stmts {
            if stmt.is_if_statement() {
                // Use inline traverse to create a scoped visitor
                traverse(stmt) {
                    let return_count = 0;

                    fn visit_return_statement(ret: &mut ReturnStatement, ctx: &Context) {
                        // Strip return value by replacing the whole node
                        *ret = ReturnStatement {
                            argument: null,
                        };
                        self.return_count += 1;
                    }
                }
            }
        }
    }

    /// Example of delegated traverse
    fn visit_class_declaration(node: &mut ClassDeclaration, ctx: &Context) {
        if node.is_abstract {
            // Route through cleanup visitor
            traverse(node) using CleanupVisitor;
        }
    }
}

/// A reusable cleanup visitor
plugin CleanupVisitor {
    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        // Rename identifiers
        if node.name.starts_with("_") {
            let new_name = format!("private{}", node.name.clone());
            *node = Identifier {
                name: new_name,
            };
        }
    }
}
