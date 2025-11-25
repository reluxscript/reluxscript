# Getting Started

Welcome to ReluxScript! This guide will help you write your first AST transformation plugin.

## What is ReluxScript?

ReluxScript is a domain-specific language for writing AST transformation plugins that compile to both **Babel (JavaScript)** and **SWC (Rust)**. Write your transformation logic once, and ReluxScript generates plugins for both platforms.

## Installation

Install ReluxScript from crates.io:

```bash
cargo install reluxscript
```

This installs the `relux` CLI tool globally.

## Your First Plugin

Let's create a simple plugin that removes `console.log` statements:

### 1. Create a New Plugin

```bash
relux new remove-console
```

This creates `remove-console.lux` with a basic template.

### 2. Implement the Transformation

Edit `remove-console.lux`:

```reluxscript
// Remove all console.log calls from your code
plugin RemoveConsole {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Check if this is a console.log call
        if matches!(node.callee, "console.log") {
            // Replace with an empty statement
            *node = Statement::empty();
        }
    }
}
```

### 3. Compile to Babel

```bash
relux build remove-console.lux --target babel
```

This generates `dist/index.js` - a Babel plugin you can use with webpack, rollup, or babel-cli.

### 4. Compile to SWC

```bash
relux build remove-console.lux --target swc
```

This generates `dist/lib.rs` - a SWC plugin you can compile to WASM.

### 5. Use Your Plugin

**With Babel:**

```javascript
// babel.config.js
module.exports = {
  plugins: [
    './dist/index.js'
  ]
}
```

**With SWC:**

```json
// .swcrc
{
  "jsc": {
    "experimental": {
      "plugins": [
        ["./dist/lib.wasm", {}]
      ]
    }
  }
}
```

## What's Next?

- [Learn the Language Syntax](/v0.1/language/syntax)
- [Explore More Examples](/v0.1/examples/)
- [Read the API Reference](/v0.1/api/visitor-methods)
- [Understand Core Concepts](/v0.1/guide/concepts)

## Key Concepts

### The Visitor Pattern

ReluxScript uses the **visitor pattern** to traverse and transform the AST. You implement visitor methods like:

- `visit_call_expression` - Called for every function call
- `visit_function_declaration` - Called for every function declaration
- `visit_identifier` - Called for every identifier

### Mutations are Explicit

All changes to the AST must be explicit:

```reluxscript
*node = Statement::empty();  // Replace node
node.name = "newName";         // Modify property
```

### The `matches!` Macro

Use `matches!` to pattern match on AST nodes:

```reluxscript
if matches!(node.callee, "console.log") {
    // This is console.log
}

if matches!(node.operator, "+") {
    // This is addition
}
```

## Common Patterns

### Removing Nodes

```reluxscript
*node = Statement::empty();
```

### Modifying Properties

```reluxscript
node.name = "newName";
node.async = true;
```

### Creating New Nodes

```reluxscript
let new_call = CallExpression {
    callee: Identifier::new("console.log"),
    arguments: vec![StringLiteral::new("Hello")]
};
```

### Conditional Transformations

```reluxscript
if matches!(node.operator, "+") && node.left.is_number() {
    // Transform only numeric additions
}
```

## Getting Help

- **Documentation**: [docs.reluxscript.com](https://docs.reluxscript.com)
- **Examples**: Check the `/examples` folder in the repo
- **Issues**: [GitHub Issues](https://github.com/reluxscript/reluxscript/issues)

Ready to dive deeper? Continue to [Core Concepts](/v0.1/guide/concepts).
