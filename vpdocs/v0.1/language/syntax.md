# Language Syntax

ReluxScript is a Rust-inspired language for writing AST transformations. This page covers the core syntax and language constructs.

## Plugin Definition

Every ReluxScript file defines a plugin:

```reluxscript
plugin MyPlugin {
    // Visitor methods go here
}
```

The plugin name must be in PascalCase and should match the file name (without `.lux`).

## Visitor Methods

Visitor methods are called when traversing AST nodes:

```reluxscript
plugin MyPlugin {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Called for every function call
    }

    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        // Called for every identifier
    }
}
```

### Available Visitor Methods

- `visit_call_expression` - Function/method calls
- `visit_function_declaration` - Function declarations
- `visit_arrow_function` - Arrow functions
- `visit_identifier` - Variable names
- `visit_binary_expression` - Binary operations (+, -, *, etc.)
- `visit_member_expression` - Property access (obj.prop)
- `visit_if_statement` - If statements
- `visit_for_statement` - For loops
- `visit_variable_declaration` - Variable declarations

See [Visitor Methods API](/v0.1/api/visitor-methods) for the complete list.

## Variables

### Declaration

```reluxscript
let name = "value";          // Immutable
let mut counter = 0;         // Mutable
const MAX_SIZE = 100;        // Constant
```

### Types

ReluxScript infers types, but you can annotate them:

```reluxscript
let name: Str = "hello";
let count: i32 = 42;
let items: Vec<Str> = vec![];
let flag: bool = true;
```

Common types:
- `Str` - String
- `i32`, `u32`, `f64` - Numbers
- `bool` - Boolean
- `Vec<T>` - Vector/Array
- `Option<T>` - Optional value

## Pattern Matching

### The `matches!` Macro

Check if an AST node matches a pattern:

```reluxscript
if matches!(node.callee, "console.log") {
    // This is console.log
}

if matches!(node.operator, "+") {
    // This is addition
}

if matches!(node.type, "Identifier") {
    // This is an identifier
}
```

### Match Expressions

Full pattern matching on values:

```reluxscript
match node.operator {
    "+" => {
        // Addition
    }
    "-" => {
        // Subtraction
    }
    _ => {
        // Other operators
    }
}
```

### Destructuring

```reluxscript
match &node.callee {
    MemberExpression { object, property } => {
        // Access object and property
    }
    _ => {}
}
```

## Control Flow

### If Statements

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
    // Process each argument
}

// While loop
while condition {
    // Loop body
}
```

## Expressions

### Binary Operations

```reluxscript
let sum = a + b;
let product = x * y;
let is_equal = name == "test";
let is_valid = count > 0 && count < 100;
```

### Member Access

```reluxscript
node.callee          // Direct property access
node.callee.name     // Chained access
node?.optional       // Optional chaining
```

### Method Calls

```reluxscript
node.clone()         // Clone a node
vec.push(item)       // Add to vector
str.contains("test") // String methods
```

## Mutations

All AST modifications must be explicit:

### Replace Node

```reluxscript
*node = Statement::empty();
```

### Modify Properties

```reluxscript
node.name = "newName";
node.async = true;
node.params = vec![];
```

### Add to Collections

```reluxscript
node.arguments.push(new_arg);
node.body.statements.clear();
```

## Functions

### Helper Functions

Define helper functions within your plugin:

```reluxscript
plugin MyPlugin {
    fn is_console_method(callee: &Expression) -> bool {
        matches!(callee, "console.*")
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if self.is_console_method(&node.callee) {
            *node = Statement::empty();
        }
    }
}
```

## Comments

```reluxscript
// Single-line comment

/*
 * Multi-line comment
 */

/// Documentation comment
/// These are preserved in generated code
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    // ...
}
```

## Literals

```reluxscript
// Strings
"hello"
"escaped \"quotes\""

// Numbers
42
3.14
0xFF        // Hex
0b1010      // Binary

// Booleans
true
false

// Null
null
```

## Macros

### `matches!` - Pattern Matching

```reluxscript
matches!(node.callee, "console.log")
matches!(node.operator, "+")
```

### `vec!` - Create Vector

```reluxscript
vec![]                    // Empty vector
vec![a, b, c]            // Vector with items
```

### `println!` - Debug Print

```reluxscript
println!("Processing: {}", node.name);
```

## Keywords

### Reserved Keywords

```
plugin      fn          let         const       if          else
match       return      true        false       null        for
in          while       break       continue    struct      enum
impl        use         pub         mut         self        Self
```

### Operators

```
// Arithmetic
+   -   *   /   %

// Comparison
==  !=  <   >   <=  >=

// Logical
&&  ||  !

// Assignment
=   +=  -=  *=  /=

// Member Access
.   ::  ?.

// Special
=>  ->  ?
```

## Best Practices

### 1. Use Descriptive Names

```reluxscript
// Good
fn is_async_function(node: &FunctionDeclaration) -> bool

// Bad
fn check(n: &FunctionDeclaration) -> bool
```

### 2. Keep Visitor Methods Focused

```reluxscript
// Good - single responsibility
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    if self.should_remove(node) {
        *node = Statement::empty();
    }
}

// Bad - too much logic
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    // 50 lines of complex logic
}
```

### 3. Use Pattern Matching

```reluxscript
// Good
if matches!(node.operator, "+") {
    // Handle addition
}

// Less clear
if node.operator == "+" {
    // Handle addition
}
```

## Next Steps

- [Types and Type System](/v0.1/language/types)
- [Pattern Matching Deep Dive](/v0.1/language/pattern-matching)
- [AST Node Types](/v0.1/language/node-types)
