# Remove Console Example

Remove all `console.log()` calls from your JavaScript code.

## Code

```reluxscript
plugin RemoveConsole {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Check if this is a console.log call
        if matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" },
            property: Identifier { name: "log" }
        }) {
            // Replace with an empty statement
            *node = Statement::empty();
        }
    }
}
```

## Input

```javascript
console.log("Starting...");
doWork();
console.log("Done!");
```

## Output

```javascript
doWork();
```

## How It Works

1. `visit_call_expression` is called for every function call
2. `matches!` checks if the callee is `console.log`
3. If it matches, replace the node with an empty statement
4. Empty statements are removed during code generation

## Variations

### Remove All Console Methods

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    if matches!(node.callee, MemberExpression {
        object: Identifier { name: "console" }
    }) {
        *node = Statement::empty();
    }
}
```

### Remove Specific Methods

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    match get_console_method(&node.callee) {
        Some("log") | Some("debug") => {
            *node = Statement::empty();
        }
        _ => {}
    }
}
```

[Back to Examples](/v0.1/examples/)
