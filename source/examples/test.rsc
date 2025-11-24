/// Test ReluxScript file
plugin TestPlugin {
    struct State {
        count: i32,
        name: Str,
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Check for console.log
        if matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" },
            property: Identifier { name: "log" }
        }) {
            let msg = format!("Found console.log at {}", ctx.filename);
            *node = CallExpression {
                callee: Identifier::new("debug"),
                arguments: vec![StringLiteral::new(msg)],
            };
        }

        node.visit_children(self);
    }

    fn is_hook_name(name: &Str) -> bool {
        name.starts_with("use") && name.len() > 3
    }
}

// Test various literals
const MAX_COUNT = 0xFF;
const MASK = 0b1010;
let value = 3.14;
let count = 1_000_000;
