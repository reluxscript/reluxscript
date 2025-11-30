# Remove Console Example

Remove all `console.log()` calls from your JavaScript code.

## Code

```reluxscript
plugin RemoveConsole {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Check if this is a console.log call
        if let Callee::MemberExpression(ref member) = node.callee {
            if let Expression::Identifier(ref obj) = *member.object {
                if obj.name == "console" {
                    if let Expression::Identifier(ref prop) = *member.property {
                        if prop.name == "log" {
                            ctx.remove();
                        }
                    }
                }
            }
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
2. Use nested `if let` to check if callee is a member expression
3. Check if the object is an identifier named "console"
4. Check if the property is an identifier named "log"
5. If all conditions match, call `ctx.remove()` to remove the statement

## Variations

### Remove All Console Methods

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    if let Callee::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref obj) = *member.object {
            if obj.name == "console" {
                // Remove any console.* call
                ctx.remove();
            }
        }
    }
}
```

### Remove Specific Methods

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    if let Callee::MemberExpression(ref member) = node.callee {
        if let Expression::Identifier(ref obj) = *member.object {
            if obj.name == "console" {
                if let Expression::Identifier(ref prop) = *member.property {
                    if prop.name == "log" || prop.name == "debug" {
                        ctx.remove();
                    }
                }
            }
        }
    }
}
```

[Back to Examples](/v0.1/examples/)
