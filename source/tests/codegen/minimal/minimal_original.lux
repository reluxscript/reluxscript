// Original version with if-let path-qualified patterns (before autofix)
writer MinimalTest {
    struct State {
        data: Vec<i32>,
    }

    fn init() -> State {
        State {
            data: vec![],
        }
    }

    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        // Test nested if-let with path qualifiers
        if let Some(id) = &node.id {
            let name = id.name.clone();
        }
    }

    fn process_statement(stmt: &Statement) {
        // This should be auto-fixed
        if let Statement::ExpressionStatement(expr_stmt) = stmt {
            if let Expression::CallExpression(call) = &expr_stmt.expression {
                if let Expression::MemberExpression(member) = &call.callee {
                    // Nested path-qualified if-let patterns
                    let test = 123;
                }
            }
        }

        // Another pattern
        if let Statement::VariableDeclaration(var_decl) = stmt {
            process_var(var_decl);
        }
    }

    fn process_expression(expr: &Expression) {
        // More patterns to fix
        if let Expression::Identifier(id) = expr {
            let name = id.name.clone();
        }

        if let Expression::ArrayExpression(arr) = expr {
            for elem in &arr.elements {
                if let Some(Expression::Identifier(id)) = elem {
                    test(id);
                }
            }
        }
    }

    fn process_var(decl: &VariableDeclaration) {}
    fn test(id: &Identifier) {}

    fn finish(&self) -> State {
        State {
            data: vec![],
        }
    }
}
