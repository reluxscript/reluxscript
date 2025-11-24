/// Test: Delegated traverse using another visitor
///
/// ReluxScript should support delegating traversal to another
/// visitor using the "traverse(node) using VisitorName" syntax.

plugin CleanUpVisitor {
    fn visit_identifier(n: &mut Identifier, ctx: &Context) {
        // cleanup logic - normalize names
        if n.name.starts_with("_temp_") {
            n.name = n.name.replace("_temp_", "");
        }
    }
}

plugin MainVisitor {
    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        if node.is_async {
            // Route this subtree through the CleanUp visitor
            traverse(node) using CleanUpVisitor;
        }
    }
}
