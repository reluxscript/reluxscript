/// Test matches! macro
plugin MatchesTest {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Test matches! macro
        if matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" },
            property: Identifier { name: "log" }
        }) {
            // Matched console.log
            let x = 1;
        }
    }
}
