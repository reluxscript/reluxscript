/// Test: flat_map_in_place for list manipulation
///
/// ReluxScript should support flat_map_in_place on statement lists
/// to insert siblings or remove statements.

plugin FlatMapTest {
    fn visit_block_statement(node: &mut BlockStatement, ctx: &Context) {
        // Use flat_map_in_place to transform the statement list
        node.stmts.flat_map_in_place(|stmt| {
            if stmt.is_return() {
                // Replace 1 statement with 2 (inject logging before return)
                vec![
                    Statement::expression(log_call()),
                    stmt.clone()
                ]
            } else if should_remove(&stmt) {
                // Remove statement by returning empty vec
                vec![]
            } else if should_duplicate(&stmt) {
                // Duplicate statement
                vec![stmt.clone(), stmt.clone()]
            } else {
                // Keep as-is
                vec![stmt.clone()]
            }
        });
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        // Insert prologue at start of function
        node.body.stmts.flat_map_in_place(|stmt| {
            // First statement gets prologue inserted before it
            if is_first_statement(&stmt) {
                vec![
                    Statement::expression(prologue_call()),
                    stmt.clone()
                ]
            } else {
                vec![stmt.clone()]
            }
        });
    }
}

fn log_call() -> CallExpression {
    CallExpression {
        callee: Identifier::new("log"),
        arguments: vec![StringLiteral::new("returning")],
    }
}

fn prologue_call() -> CallExpression {
    CallExpression {
        callee: Identifier::new("__prologue"),
        arguments: vec![],
    }
}

fn should_remove(stmt: &Statement) -> bool {
    false
}

fn should_duplicate(stmt: &Statement) -> bool {
    false
}

fn is_first_statement(stmt: &Statement) -> bool {
    true
}
