/// Test: Inline traverse blocks
///
/// ReluxScript should support inline traverse blocks that define
/// a scoped visitor for a subtree.

plugin TraverseInlineTest {
    fn visit_function_declaration(func: &mut FunctionDeclaration, ctx: &Context) {
        for stmt in &mut func.body.stmts {
            if stmt.is_if_statement() {
                // Spawn a nested visitor for just this statement
                traverse(stmt) {
                    // Local state (becomes struct fields in Rust, object properties in JS)
                    let found_returns = 0;

                    fn visit_return_statement(ret: &mut ReturnStatement, ctx: &Context) {
                        ret.argument = None;
                        self.found_returns += 1;
                    }
                }
            }
        }
    }
}
