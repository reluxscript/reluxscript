# Core Concepts

This guide covers the fundamental concepts you need to understand to write effective ReluxScript plugins.

## The Visitor Pattern

ReluxScript uses the **visitor pattern** to traverse and transform Abstract Syntax Trees (ASTs).

### How It Works

1. The compiler parses JavaScript/TypeScript into an AST
2. Your visitor methods are called for each matching node type
3. You can inspect, modify, or replace nodes
4. The modified AST is converted back to code

```reluxscript
plugin MyPlugin {
    // This method is called for EVERY function call in the code
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Inspect the node
        if matches!(node.callee, "console.log") {
            // Transform it
            *node = Statement::empty();
        }
    }
}
```

### Visitor Methods

Each visitor method corresponds to an AST node type:

| Method | Node Type | Example |
|--------|-----------|---------|
| `visit_identifier` | Variable names | `foo`, `bar` |
| `visit_call_expression` | Function calls | `fn()`, `obj.method()` |
| `visit_binary_expression` | Operations | `a + b`, `x * y` |
| `visit_function_declaration` | Functions | `function foo() {}` |
| `visit_jsx_element` | JSX | `<Component />` |

See [Visitor Methods API](/v0.1/api/visitor-methods) for the complete list.

## Mutations Must Be Explicit

One of ReluxScript's core principles: **all changes must be explicit**.

### Node Replacement

```reluxscript
// ✅ Explicit: replacing the entire node
*node = Statement::empty();

// ✅ Explicit: creating a new node
*node = CallExpression {
    callee: Identifier::new("newFn"),
    arguments: vec![]
};
```

### Value Extraction

```reluxscript
// ✅ Explicit: cloning the value
let name = node.name.clone();

// ❌ Error: implicit borrow not allowed
let name = node.name;
```

**Why?** This ensures the generated code works correctly in both JavaScript (garbage collected) and Rust (borrow checked).

## The `matches!` Macro

Pattern matching on AST nodes:

```reluxscript
// Simple pattern
if matches!(node.callee, "console.log") {
    // This is console.log()
}

// Complex pattern
if matches!(node.callee, MemberExpression {
    object: Identifier { name: "console" },
    property: Identifier { name: "log" }
}) {
    // Also matches console.log()
}
```

## Type System

### Primitive Types

```reluxscript
let name: Str = "hello";      // String
let count: i32 = 42;           // Integer
let ratio: f64 = 3.14;         // Float
let flag: bool = true;         // Boolean
```

### Container Types

```reluxscript
let items: Vec<Str> = vec!["a", "b", "c"];     // Array/Vector
let maybe: Option<Str> = Some("value");         // Optional value
let result: Result<Str, Str> = Ok("success");  // Result type
```

### References

```reluxscript
fn process(node: &CallExpression) {     // Immutable reference
    // Can read, cannot modify
}

fn transform(node: &mut CallExpression) {  // Mutable reference
    // Can read and modify
}
```

## Pattern Matching

### Match Expression

```reluxscript
match node.operator {
    "+" => handle_addition(),
    "-" => handle_subtraction(),
    "*" | "/" => handle_multiplication_division(),
    _ => handle_other(),
}
```

### If-Let Pattern

```reluxscript
if let Some(name) = get_identifier_name(node) {
    // name is available here
    println!("Found identifier: {}", name);
}
```

### Destructuring

```reluxscript
match &node.callee {
    Expression::MemberExpression(member) => {
        // Access member.object and member.property
        let obj = &member.object;
        let prop = &member.property;
    }
    _ => {}
}
```

## Context Object

The `Context` provides information about the current traversal:

```reluxscript
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    // Check if this identifier is bound in the current scope
    if ctx.scope.has_binding(&node.name) {
        // It's a local variable
    }

    // Generate a unique identifier
    let unique_name = ctx.generate_uid("temp");

    // Get the filename
    println!("Processing: {}", ctx.filename);
}
```

**Warning:** `ctx.scope` operations are cheap in Babel but expensive in SWC (requires pre-pass analysis).

## Error Handling

ReluxScript uses `Result<T, E>` for error handling:

```reluxscript
fn parse_config(path: &Str) -> Result<Config, Str> {
    // The ? operator propagates errors
    let content = fs::read_file(path)?;
    let config = json::parse(&content)?;

    Ok(config)
}
```

**Compiles to:**
- **Babel**: `{ ok: boolean, value?: T, error?: E }`
- **SWC**: Native `Result<T, E>` enum

## Collections

### Vec (Array/Vector)

```reluxscript
let mut items = vec!["a", "b"];
items.push("c");

for item in &items {
    println!("{}", item);
}

let doubled: Vec<Str> = items.iter()
    .map(|s| format!("{}_{}", s, s))
    .collect();
```

### HashMap (Map/Dictionary)

```reluxscript
let mut map = HashMap::new();
map.insert("key", "value");

if let Some(value) = map.get("key") {
    println!("Found: {}", value);
}
```

### HashSet (Unique Set)

```reluxscript
let mut seen = HashSet::new();
seen.insert("item1");

if seen.contains("item1") {
    // Already seen
}
```

## Control Flow

### If/Else

```reluxscript
if node.async {
    // Async function
} else if node.generator {
    // Generator function
} else {
    // Regular function
}
```

### Loops

```reluxscript
// For loop
for arg in &node.arguments {
    process_argument(arg);
}

// While loop
while has_more() {
    process_next();
}

// Loop with break
loop {
    if done() {
        break;
    }
}
```

## Vector Alignment

ReluxScript only supports features that work **identically** in both Babel and SWC.

### Supported

✅ Pattern matching on AST nodes
✅ Explicit mutations
✅ Type-safe collections
✅ Error handling with `Result`
✅ String operations
✅ Closures (limited)

### Not Supported

❌ Async/await (different semantics)
❌ Direct DOM/Node.js APIs
❌ External library imports
❌ Regex literals (use string matching)
❌ Closures capturing mutable state

### Escape Hatches

For platform-specific code, use verbatim blocks:

```reluxscript
babel! {
    // Raw JavaScript - only in Babel output
    const recast = require('recast');
    // ...
}

swc! {
    // Raw Rust - only in SWC output
    use swc_common::DUMMY_SP;
    // ...
}
```

**Use sparingly** - this breaks the "write once" guarantee.

## Best Practices

### 1. Keep Visitor Methods Focused

```reluxscript
// ✅ Good: Single responsibility
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    if self.should_transform(node) {
        self.transform(node);
    }
}

// ❌ Bad: Too much logic in visitor
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    // 50 lines of complex logic...
}
```

### 2. Use Helper Functions

```reluxscript
plugin MyPlugin {
    // Helper function
    fn is_console_method(&self, callee: &Expression) -> bool {
        matches!(callee, MemberExpression {
            object: Identifier { name: "console" }
        })
    }

    // Visitor uses helper
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if self.is_console_method(&node.callee) {
            *node = Statement::empty();
        }
    }
}
```

### 3. Track State Explicitly

```reluxscript
plugin MyPlugin {
    struct State {
        component_name: Option<Str>,
        hook_calls: Vec<HookInfo>,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        self.state.component_name = Some(node.id.name.clone());
        node.visit_children(self);
        self.state.component_name = None;
    }
}
```

### 4. Use Pattern Matching

```reluxscript
// ✅ Good: Clear pattern matching
if matches!(node.operator, "+") {
    handle_addition();
}

// ❌ Less clear: String comparison
if node.operator == "+" {
    handle_addition();
}
```

## Next Steps

Now that you understand the core concepts:

- [Learn the full syntax](/v0.1/language/syntax)
- [Explore complete examples](/v0.1/examples/)
- [Read the API reference](/v0.1/api/visitor-methods)
- [See the full specification](/v0.1/language/specification)
