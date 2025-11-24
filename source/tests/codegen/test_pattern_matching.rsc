writer TestPatternMatching {
    fn test_pattern_match(param: &Pattern) {
        // Test if unqualified pattern matching works
        if let ObjectPattern(obj_pat) = param {
            // Do something with obj_pat
        }

        if let Identifier(id) = param {
            // Do something with id
        }
    }

    fn test_expr_match(expr: &Expression) {
        if let CallExpression(call) = expr {
            // Do something
        }
    }
}
