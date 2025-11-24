/// Test: matches! macro for pattern matching
///
/// ReluxScript should support the matches! macro for inline
/// pattern matching that compiles to both Babel and SWC.

plugin MatchesMacroTest {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Test simple pattern matching
        if matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" },
            property: Identifier { name: "log" }
        }) {
            // This is console.log()
            *node = CallExpression {
                callee: Identifier::new("customLog"),
                arguments: node.arguments.clone(),
            };
        }

        // Test with enum patterns
        if matches!(node.callee, Identifier { name: "useState" }) {
            // This is a useState call
        }
    }

    fn visit_expression(expr: &mut Expression, ctx: &Context) {
        // Test with literal patterns
        if matches!(expr, NumericLiteral { value: 0 }) {
            *expr = BooleanLiteral::new(false);
        }
    }
}
