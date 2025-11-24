# ReluxScript Developer's Guide

**Version:** 0.3.0
**Last Updated:** 2024

A comprehensive guide for writing AST transformation plugins in ReluxScript that compile to both Babel (JavaScript) and SWC (Rust).

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Getting Started](#2-getting-started)
3. [Language Basics](#3-language-basics)
4. [The Unified AST](#4-the-unified-ast)
5. [Writing Plugins](#5-writing-plugins)
6. [Writing Writers (Transpilers)](#6-writing-writers-transpilers)
7. [Pattern Matching](#7-pattern-matching)
8. [Scoped Traversal](#8-scoped-traversal)
9. [Working with TypeScript AST](#9-working-with-typescript-ast)
10. [Type System](#10-type-system)
11. [Best Practices](#11-best-practices)
12. [Platform Differences](#12-platform-differences)
13. [Troubleshooting](#13-troubleshooting)
14. [API Reference](#14-api-reference)

---

## 1. Introduction

### What is ReluxScript?

ReluxScript is a domain-specific language designed for writing AST (Abstract Syntax Tree) transformation plugins. It compiles to both:

- **Babel plugins** (JavaScript) for the Node.js ecosystem
- **SWC plugins** (Rust/WASM) for high-performance compilation

### Why ReluxScript?

Writing the same transformation logic twice—once in JavaScript for Babel and once in Rust for SWC—is error-prone and time-consuming. ReluxScript solves this by:

1. **Write Once, Run Anywhere**: Single source compiles to both targets
2. **Unified AST**: Abstract away ESTree vs swc_ecma_ast differences
3. **Type Safety**: Rust-inspired ownership model catches errors early
4. **Performance**: Generated SWC plugins run at native speed

### The Vector Alignment Principle

ReluxScript operates on the principle of "vector alignment"—finding the intersection of JavaScript and Rust capabilities. Features that work on one platform but not the other are either:

- Abstracted into unified constructs
- Explicitly marked as platform-specific
- Prohibited to ensure correctness

---

## 2. Getting Started

### Installation

```bash
# Build the ReluxScript compiler
cd reluxscript
cargo build --release

# Add to PATH (optional)
export PATH="$PATH:/path/to/reluxscript/target/release"
```

### Your First Plugin

Create a file `hello.lux`:

```reluxscript
/// A simple plugin that logs function names
plugin HelloPlugin {
    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        let name = node.id.name.clone();
        // In a real plugin, you'd transform the AST here
        node.visit_children(self);
    }
}
```

### Building

```bash
# Build for both targets
reluxscript build hello.lux

# Build for specific target
reluxscript build hello.lux --target babel
reluxscript build hello.lux --target swc

# Specify output directory
reluxscript build hello.lux -o dist
```

### Output Structure

```
dist/
├── index.js    # Babel plugin
└── lib.rs      # SWC plugin
```

---

## 3. Language Basics

### Comments

```reluxscript
// Single-line comment

/// Documentation comment (included in output)

/* Multi-line
   comment */
```

### Variables

```reluxscript
// Immutable binding
let x = 5;
let name = "hello";

// Mutable binding
let mut count = 0;
count += 1;

// Constants (compile-time)
const MAX_DEPTH = 10;
```

### Types

#### Primitive Types

```reluxscript
let flag: bool = true;
let count: i32 = 42;
let index: u32 = 0;
let ratio: f64 = 3.14;
```

#### String Type

```reluxscript
// Str is the unified string type
// Compiles to: String (JS) / JsWord (SWC)
let name: Str = "hello";

// String operations
let upper = name.to_uppercase();
let len = name.len();
let starts = name.starts_with("he");
```

#### Container Types

```reluxscript
// Vector
let mut items: Vec<Str> = vec![];
items.push("first");
items.insert(0, "zeroth");

// HashMap
let mut map: HashMap<Str, i32> = HashMap::new();
map.insert("key", 42);

// Option
let maybe: Option<Str> = Some("value");
if let Some(val) = maybe {
    // use val
}
```

### Control Flow

```reluxscript
// If-else
if condition {
    // ...
} else if other {
    // ...
} else {
    // ...
}

// Match
match value {
    1 => handle_one(),
    2 | 3 => handle_two_or_three(),
    _ => handle_other(),
}

// Loops
for item in &items {
    // ...
}

while condition {
    // ...
}

loop {
    if done {
        break;
    }
}
```

### Functions

```reluxscript
// Basic function
fn add(a: i32, b: i32) -> i32 {
    return a + b;
}

// Public function (exported)
pub fn helper(name: &Str) -> bool {
    return name.len() > 0;
}

// Function with mutable parameter
fn transform(node: &mut Expr) {
    // modify node
}
```

---

## 4. The Unified AST

### Node Types

ReluxScript uses unified node type names that map to both platforms:

| ReluxScript | Babel (ESTree) | SWC |
|------------|----------------|-----|
| `Program` | `Program` | `Module` |
| `FunctionDeclaration` | `FunctionDeclaration` | `FnDecl` |
| `VariableDeclaration` | `VariableDeclaration` | `VarDecl` |
| `Identifier` | `Identifier` | `Ident` |
| `CallExpression` | `CallExpression` | `CallExpr` |
| `MemberExpression` | `MemberExpression` | `MemberExpr` |
| `BinaryExpression` | `BinaryExpression` | `BinExpr` |
| `StringLiteral` | `StringLiteral` | `Str` |
| `NumericLiteral` | `NumericLiteral` | `Number` |
| `TSInterfaceDeclaration` | `TSInterfaceDeclaration` | `TsInterfaceDecl` |
| `TSPropertySignature` | `TSPropertySignature` | `TsPropertySignature` |
| `TSTypeReference` | `TSTypeReference` | `TsTypeRef` |

### Field Access

Field names are also unified:

```reluxscript
// Identifier
let name = ident.name;  // .name (Babel) / .sym (SWC)

// MemberExpression
let obj = member.object;     // .object (Babel) / .obj (SWC)
let prop = member.property;  // .property (Babel) / .prop (SWC)

// CallExpression
let callee = call.callee;       // same
let args = call.arguments;      // .arguments (Babel) / .args (SWC)

// FunctionDeclaration
let id = func.id;
let params = func.params;
let body = func.body;
```

### Creating Nodes

```reluxscript
// Create an identifier
let id = Identifier {
    name: "myVar",
};

// Create a call expression
let call = CallExpression {
    callee: Identifier { name: "console" },
    arguments: vec![
        StringLiteral { value: "Hello" },
    ],
};

// Create a member expression
let member = MemberExpression {
    object: Identifier { name: "console" },
    property: Identifier { name: "log" },
};
```

---

## 5. Writing Plugins

### Plugin Structure

```reluxscript
plugin MyPlugin {
    // State (becomes struct fields)
    struct State {
        count: i32,
        found_items: Vec<Str>,
    }

    // Visitor methods
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Transform the node
        node.visit_children(self);
    }

    // Helper functions
    fn is_console_log(node: &CallExpression) -> bool {
        // ...
    }
}
```

### Visitor Methods

Visitor methods follow the naming convention `visit_<node_type>`:

```reluxscript
fn visit_program(node: &mut Program, ctx: &Context) { }
fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) { }
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) { }
fn visit_identifier(node: &mut Identifier, ctx: &Context) { }
// ... etc
```

### The Context Object

The `ctx` parameter provides traversal context:

```reluxscript
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    // Get the filename being processed
    let file = ctx.filename;

    // Get parent information (when available)
    // Note: Platform-specific behavior
}
```

### Traversal Control

```reluxscript
fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
    // Process children AFTER this node (post-order)
    // Do your transformation first
    transform_function(node);

    // Then visit children
    node.visit_children(self);
}

fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    // Process children BEFORE this node (pre-order)
    node.visit_children(self);

    // Then do your transformation
    transform_call(node);
}

fn visit_block_statement(node: &mut BlockStatement, ctx: &Context) {
    // Skip children entirely
    // Don't call visit_children
}
```

### Node Replacement (Statement Lowering)

Replace a node using pointer assignment:

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    if is_deprecated_api(node) {
        // Replace with new API call
        *node = CallExpression {
            callee: Identifier { name: "newApi" },
            arguments: node.arguments.clone(),
        };
    }
}
```

This compiles to:
- **Babel**: `path.replaceWith(t.callExpression(...))`
- **SWC**: Direct assignment `*node = CallExpr { ... }`

### Removing Nodes

```reluxscript
fn visit_expression_statement(node: &mut ExpressionStatement, ctx: &Context) {
    if should_remove(node) {
        // Mark for removal
        node.remove();
    }
}
```

---

## 6. Writing Writers (Transpilers)

Writers are read-only visitors for transpilation (generating different output):

```reluxscript
writer ReactToOrleans {
    builder: CodeBuilder,

    fn init() -> Self {
        Self { builder: CodeBuilder::new() }
    }

    fn visit_function_declaration(node: &FunctionDeclaration, ctx: &Context) {
        // Generate C# class
        self.builder.append("public class ");
        self.builder.append(node.id.name.clone());
        self.builder.append(" : Grain {\n");

        self.builder.indent();
        node.visit_children(self);
        self.builder.dedent();

        self.builder.append("}\n");
    }

    fn finish(self) -> Str {
        self.builder.to_string()
    }
}
```

### CodeBuilder API

```reluxscript
let mut builder = CodeBuilder::new();

// Append text
builder.append("text");
builder.append_line("line with newline");

// Indentation
builder.indent();   // Increase indent level
builder.dedent();   // Decrease indent level

// Get result
let output = builder.to_string();
```

---

## 7. Pattern Matching

### The `matches!` Macro

Type-check nodes without extracting values:

```reluxscript
if matches!(node, Identifier) {
    // node is an Identifier
}

if matches!(node.callee, MemberExpression) {
    // callee is a MemberExpression
}
```

### Nested Pattern Matching

```reluxscript
if matches!(node.callee, MemberExpression {
    object: Identifier { name: "console" },
    property: Identifier { name: "log" }
}) {
    // This is console.log
}
```

### Compiled Output

**Babel:**
```javascript
if (
    t.isMemberExpression(node.callee) &&
    t.isIdentifier(node.callee.object, { name: "console" }) &&
    t.isIdentifier(node.callee.property, { name: "log" })
) {
    // matched
}
```

**SWC:**
```rust
if let Expr::Member(member) = &node.callee {
    if let Expr::Ident(obj) = &*member.obj {
        if &*obj.sym == "console" {
            if let MemberProp::Ident(prop) = &member.prop {
                if &*prop.sym == "log" {
                    // matched
                }
            }
        }
    }
}
```

### Flow-Sensitive Typing

When you use `matches!` in a condition, the variable is automatically narrowed:

```reluxscript
fn visit_expression(node: &mut Expr, ctx: &Context) {
    // node is Expr (enum)

    if matches!(node, CallExpression) {
        // Inside this block, 'node' is narrowed to CallExpression
        // You can access .callee, .arguments directly
        let callee = node.callee;
    }

    // Outside the block, node is back to Expr
}
```

---

## 8. Scoped Traversal

### Inline Traversal

Define a one-off visitor for a subtree:

```reluxscript
fn visit_function_declaration(func: &mut FunctionDeclaration, ctx: &Context) {
    // Count returns in this function only
    traverse(func.body) {
        let count = 0;

        fn visit_return_statement(ret: &mut ReturnStatement, ctx: &Context) {
            self.count += 1;
        }
    }

    // 'count' is now available here if needed
}
```

### Delegated Traversal

Apply another plugin/visitor:

```reluxscript
plugin Cleanup {
    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        // cleanup logic
    }
}

plugin Main {
    fn visit_function(node: &mut Function, ctx: &Context) {
        if node.is_async {
            // Apply Cleanup to this subtree
            traverse(node) using Cleanup;
        }
    }
}
```

### Manual Iteration

Selectively visit children:

```reluxscript
fn visit_block_statement(node: &mut BlockStatement, ctx: &Context) {
    // Don't call node.visit_children(self)

    for stmt in &mut node.stmts {
        if needs_special_handling(stmt) {
            traverse(stmt) using SpecialVisitor;
        } else {
            stmt.visit_with(self);
        }
    }
}
```

---

## 9. Working with TypeScript AST

ReluxScript can parse and transform TypeScript/TSX files by visiting TypeScript-specific AST nodes.

### Visiting TypeScript Nodes

```reluxscript
plugin InterfaceExtractor {
    /// Visit TypeScript interface declarations
    pub fn visit_interface_declaration(node: &TSInterfaceDeclaration) -> Str {
        let mut parts = vec![];

        // Get interface name
        let interface_name = node.id.name.clone();
        parts.push(interface_name);

        // Iterate over interface body members
        for member in &node.body.body {
            // Check if this is a property signature
            if matches!(member, TSPropertySignature) {
                let prop_name = member.key.name.clone();
                parts.push(prop_name);
            }
        }

        return parts.join(",");
    }
}
```

### Accessing Type Arguments

CallExpressions can have type arguments (generics):

```reluxscript
fn visit_call_expression(node: &CallExpression) -> Str {
    // Check if this is a useState call with type args
    if matches!(node.callee, Identifier) {
        let callee_name = node.callee.name.clone();
        if callee_name == "useState" {
            // Access type arguments like useState<string>
            if node.type_args.len() > 0 {
                // First type argument is available
                return "has_type_arg";
            }
        }
    }
    return "";
}
```

### TypeScript Field Access Patterns

When accessing fields on TypeScript nodes, be aware of wrapper types:

```reluxscript
// TSPropertySignature.key is Box<Expr>, not directly an Identifier
// The codegen handles this automatically with pattern matching:
let prop_name = member.key.name.clone();

// Generates (in SWC):
// match member.key.as_ref() {
//     Expr::Ident(i) => i.sym.clone(),
//     _ => "".into()
// }
```

### Common TypeScript Visitor Methods

```reluxscript
fn visit_interface_declaration(node: &mut TSInterfaceDeclaration, ctx: &Context) { }
fn visit_type_alias_declaration(node: &mut TSTypeAliasDeclaration, ctx: &Context) { }
fn visit_property_signature(node: &mut TSPropertySignature, ctx: &Context) { }
```

---

## 10. Type System

### Ownership and Borrowing

ReluxScript enforces Rust-like ownership rules:

```reluxscript
// Immutable borrow
fn read_name(node: &Identifier) -> Str {
    return node.name.clone();  // Must clone to own the value
}

// Mutable borrow
fn update_name(node: &mut Identifier) {
    node.name = "newName";
}
```

### Clone-to-Own

When extracting values from borrowed references, you must explicitly clone:

```reluxscript
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    // ERROR: Cannot move out of borrowed content
    // let name = node.name;

    // CORRECT: Clone to own
    let name = node.name.clone();
}
```

### Type Inference

Types are inferred when possible:

```reluxscript
let x = 5;           // i32
let s = "hello";     // Str
let v = vec![];      // Vec<_> (needs context)
let v: Vec<Str> = vec![];  // Explicit when needed
```

---

## 11. Best Practices

### 1. Prefer Immutable Bindings

```reluxscript
// Good
let name = node.name.clone();

// Only use mut when necessary
let mut count = 0;
```

### 2. Clone Explicitly

```reluxscript
// Good - clear ownership
let args = node.arguments.clone();

// Avoid - unclear ownership
let args = node.arguments;  // May or may not work
```

### 3. Use Helper Functions

```reluxscript
plugin MyPlugin {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if Self::is_console_log(node) {
            Self::transform_console_log(node);
        }
    }

    fn is_console_log(node: &CallExpression) -> bool {
        matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" },
            property: Identifier { name: "log" }
        })
    }

    fn transform_console_log(node: &mut CallExpression) {
        // ...
    }
}
```

### 4. Handle All Cases

```reluxscript
fn get_name(expr: &Expr) -> Option<Str> {
    if matches!(expr, Identifier) {
        return Some(expr.name.clone());
    }
    return None;  // Don't forget the None case
}
```

### 5. Document Your Code

```reluxscript
/// Transform console.log calls to custom logger
///
/// Example:
///   console.log("hello") → logger.info("hello")
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    // ...
}
```

---

## 12. Platform Differences

### String Interning

**Babel:** Strings are regular JavaScript strings
**SWC:** Strings are interned as `JsWord`/`Atom`

ReluxScript abstracts this with the `Str` type, but be aware of performance implications when creating many strings in SWC.

### AST Structure

**Babel (ESTree):** Flat hierarchy, everything is a "Node"
**SWC:** Strict enum/struct hierarchy (`Expr`, `Stmt`, `Decl`, etc.)

ReluxScript's `matches!` macro handles this, but you may need to think about which enum variant you're working with.

### Box Wrapping

SWC uses `Box<T>` for recursive types. ReluxScript handles this automatically, but generated code may include dereferences.

### MemberProp vs Expression

In Babel, `member.property` is an `Expression`.
In SWC, `member.prop` is a `MemberProp` enum.

ReluxScript handles this in pattern matching, but be aware when debugging generated code.

---

## 13. Troubleshooting

### Common Errors

#### "Cannot move out of borrowed content"

```reluxscript
// Wrong
let name = node.name;

// Right
let name = node.name.clone();
```

#### "Type mismatch"

Check that you're using the correct type. The compiler will tell you what it expected vs what it found.

#### "Pattern doesn't match"

Make sure your `matches!` pattern uses the correct node type name (ReluxScript names, not platform-specific).

### Debugging Generated Code

1. Build with verbose output to see intermediate steps
2. Check the generated `index.js` and `lib.rs` files
3. For Babel, add `console.log` statements
4. For SWC, add `dbg!()` macros

### Performance Issues

1. Avoid cloning large node trees unnecessarily
2. Use `traverse` for targeted subtree processing
3. Short-circuit expensive checks early

---

## 14. API Reference

### Built-in Functions

```reluxscript
// String operations
str.len() -> usize
str.is_empty() -> bool
str.starts_with(prefix: &Str) -> bool
str.ends_with(suffix: &Str) -> bool
str.contains(substr: &Str) -> bool
str.to_uppercase() -> Str
str.to_lowercase() -> Str

// Vector operations
vec.len() -> usize
vec.is_empty() -> bool
vec.push(item: T)
vec.pop() -> Option<T>
vec.insert(index: usize, item: T)
vec.remove(index: usize) -> T
vec.get(index: usize) -> Option<&T>

// HashMap operations
map.insert(key: K, value: V)
map.get(key: &K) -> Option<&V>
map.contains_key(key: &K) -> bool
map.remove(key: &K) -> Option<V>
```

### Context API

```reluxscript
ctx.filename: Str           // Current file being processed
ctx.source: Option<Str>     // Source code (if available)
```

### Node Methods

```reluxscript
node.visit_children(self)   // Visit all children
node.visit_with(visitor)    // Visit with specific visitor
node.remove()               // Mark for removal
```

### CodeBuilder API

```reluxscript
CodeBuilder::new() -> CodeBuilder
builder.append(text: &Str)
builder.append_line(text: &Str)
builder.indent()
builder.dedent()
builder.to_string() -> Str
```

---

## Appendix: Full Example

```reluxscript
/// Transform console.log to a custom logger
///
/// Converts:
///   console.log("message")
/// To:
///   Logger.info("message")

plugin ConsoleToLogger {
    struct State {
        transformations: i32,
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if Self::is_console_log(node) {
            Self::transform_to_logger(node);
            self.state.transformations += 1;
        }

        node.visit_children(self);
    }

    fn is_console_log(node: &CallExpression) -> bool {
        matches!(node.callee, MemberExpression {
            object: Identifier { name: "console" },
            property: Identifier { name: "log" }
        })
    }

    fn transform_to_logger(node: &mut CallExpression) {
        // Replace console.log with Logger.info
        *node = CallExpression {
            callee: MemberExpression {
                object: Identifier { name: "Logger" },
                property: Identifier { name: "info" },
            },
            arguments: node.arguments.clone(),
        };
    }
}
```

Build and use:

```bash
# Build the plugin
reluxscript build console-to-logger.lux -o dist

# Use with Babel
npx babel src --plugins ./dist/index.js

# Use with SWC (after compiling lib.rs to WASM)
npx swc src --plugin ./dist/plugin.wasm
```

---

## Further Reading

- [ReluxScript Language Specification](reluxscript-specification.md)
- [Enhancement Plan](reluxscript-enhancement-plan.md)
- [Compiler Implementation](reluxscript-compiler-implementation.md)
- [Babel Plugin Handbook](https://github.com/jamiebuilds/babel-handbook)
- [SWC Plugin Documentation](https://swc.rs/docs/plugin/ecmascript/getting-started)
