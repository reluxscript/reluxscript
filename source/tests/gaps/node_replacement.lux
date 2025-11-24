/// Test: Node replacement with *node = ...
///
/// ReluxScript should support the *node = ... syntax for replacing
/// nodes, which compiles to path.replaceWith() in Babel and direct
/// assignment in SWC.

plugin NodeReplacementTest {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Check if it's console.log
        if let Some(name) = get_callee_name(&node.callee) {
            if name == "log" {
                // Replace the entire call expression
                *node = CallExpression {
                    callee: Identifier::new("customLog"),
                    arguments: node.arguments.clone(),
                };
            }
        }
    }

    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        // Replace identifier
        if node.name == "oldName" {
            *node = Identifier {
                name: "newName",
            };
        }
    }

    fn visit_expression_statement(node: &mut ExpressionStatement, ctx: &Context) {
        // Remove statement by replacing with noop
        if should_remove(node) {
            *node = Statement::noop();
        }
    }
}

fn get_callee_name(callee: &Expression) -> Option<Str> {
    if let Expression::Identifier(id) = callee {
        Some(id.name.clone())
    } else {
        None
    }
}

fn should_remove(node: &ExpressionStatement) -> bool {
    // Example logic
    false
}
