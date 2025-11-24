/// A simple plugin that removes console.log statements
plugin ConsoleStripper {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Check if this is a console.log call
        if matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" },
            property: Identifier { name: "log" }
        }) {
            // Remove the call by replacing with undefined
            *node = CallExpression {
                callee: Identifier { name: "void" },
                arguments: vec![Literal { value: 0 }],
            };
        }

        node.visit_children(self);
    }
}
