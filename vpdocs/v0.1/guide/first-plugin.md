# Your First Plugin

Let's create a simple but useful plugin that removes `console.log()` statements from your code.

## Step 1: Create the Plugin

Use the `relux new` command to scaffold a new plugin:

```bash
relux new remove-console
```

This creates a file `remove-console.lux` with a basic template.

## Step 2: Edit the Plugin

Open `remove-console.lux` and replace the contents with:

```reluxscript
// Remove all console.log calls from the code
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

### Understanding the Code

- `plugin RemoveConsole` - Declares a plugin named RemoveConsole
- `visit_call_expression` - Called for every function/method call
- `matches!` - Pattern matching macro to check the structure
- `*node = Statement::empty()` - Replaces the node with an empty statement

## Step 3: Compile to Babel

Compile your plugin to a Babel plugin:

```bash
relux build remove-console.lux --target babel
```

This generates `dist/index.js` - a JavaScript file you can use with Babel.

## Step 4: Compile to SWC

Compile your plugin to SWC:

```bash
relux build remove-console.lux --target swc
```

This generates `dist/lib.rs` - a Rust file for SWC.

## Step 5: Test It

Create a test JavaScript file `test.js`:

```javascript
console.log("debug info");
doWork();
console.log("more debug");
```

### Using with Babel

Create `babel.config.js`:

```javascript
module.exports = {
  plugins: [
    './dist/index.js'
  ]
};
```

Run Babel:

```bash
npx babel test.js
```

**Output:**
```javascript
doWork();
```

The `console.log()` calls are removed!

## What's Next?

Now that you've created your first plugin:

1. [Learn more patterns](/v0.1/examples/)
2. [Understand core concepts](/v0.1/guide/concepts)
3. [Explore the API](/v0.1/api/visitor-methods)

## Common Next Steps

### Make It Configurable

Add options to your plugin:

```reluxscript
plugin RemoveConsole {
    struct Options {
        remove_warn: bool,
        remove_error: bool,
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" },
            property: Identifier { name: "log" }
        }) {
            *node = Statement::empty();
        }

        if self.options.remove_warn {
            if matches!(node.callee, "console.warn") {
                *node = Statement::empty();
            }
        }
    }
}
```

### Add More Transformations

Extend to remove all console methods:

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    if matches!(node.callee, MemberExpression {
        object: Identifier { name: "console" }
    }) {
        // Removes console.log, console.warn, console.error, etc.
        *node = Statement::empty();
    }
}
```

### Track Statistics

Count how many console calls were removed:

```reluxscript
plugin RemoveConsole {
    struct State {
        removed_count: i32,
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" }
        }) {
            self.state.removed_count += 1;
            *node = Statement::empty();
        }
    }

    fn exit(program: &mut Program, state: &PluginState) {
        println!("Removed {} console calls", self.state.removed_count);
    }
}
```
