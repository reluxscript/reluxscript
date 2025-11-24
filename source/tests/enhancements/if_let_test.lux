/// Test: If-Let Pattern Matching
///
/// Tests the if-let syntax for Option/Result pattern matching

plugin IfLetTest {
    /// Test if let in visitor context - extract function body
    pub fn visit_function_declaration(node: &FunctionDeclaration) -> Str {
        if let Some(body) = &node.body {
            let stmt_count = body.stmts.len();
            return stmt_count.to_string();
        }
        return "no body";
    }

    /// Test if let with else branch
    pub fn extract_name(expr: &Expr) -> Str {
        if let Some(ident) = &expr.name {
            return ident.clone();
        } else {
            return "anonymous";
        }
    }

    /// Test multiple if-let in sequence
    pub fn process_call(call: &CallExpression) -> Str {
        if let Some(callee) = &call.callee {
            if matches!(callee, Identifier) {
                return callee.name.clone();
            }
        }
        return "unknown";
    }
}
