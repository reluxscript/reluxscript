# ReluxScript Guide for Babel Plugin Developers

A practical guide for JavaScript developers familiar with Babel plugin development who want to write cross-platform AST transformers using ReluxScript.

## What is ReluxScript?

ReluxScript is a domain-specific language that compiles to both:
- **Babel plugins** (JavaScript) - for use with Babel, webpack, etc.
- **SWC plugins** (Rust) - for use with SWC, Next.js, Turbopack, etc.

Write once, run on both platforms with optimal performance.

## Quick Comparison

| Babel (JavaScript) | ReluxScript | Notes |
|-------------------|------------|-------|
| `path.node` | `node` | Direct node reference |
| `t.isIdentifier(node)` | `matches!(node, Identifier)` | Type checking |
| `node.name` | `node.name.clone()` | Field access (explicit clone) |
| `path.traverse({...})` | `traverse(node) {...}` | Nested traversal |
| `const x = ...` | `let x = ...` | Variable declaration |
| `let x = ...` | `let mut x = ...` | Mutable variable |

## Your First ReluxScript Plugin

### Babel Version
```javascript
module.exports = function() {
  return {
    visitor: {
      Identifier(path) {
        if (path.node.name === 'foo') {
          path.node.name = 'bar';
        }
      }
    }
  };
};
```

### ReluxScript Version
```reluxscript
plugin RenameIdentifier {
    pub fn visit_identifier(node: &mut Identifier) {
        if node.name == "foo" {
            node.name = "bar".to_string();
        }
    }
}
```

## Core Concepts

### 1. Plugin Structure

Every ReluxScript plugin starts with a `plugin` declaration:

```reluxscript
plugin MyTransformer {
    // Struct definitions
    struct State {
        count: i32,
        names: Vec<Str>,
    }

    // Visitor functions
    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        // Transform logic
    }

    // Helper functions
    fn is_component_name(name: &Str) -> bool {
        // First character is uppercase
        let first = name.chars().next();
        if let Some(c) = first {
            return c.is_uppercase();
        }
        return false;
    }
}
```

### 2. Visitor Functions

Visitor functions are named after the AST node type they handle:

```reluxscript
// Babel: Identifier(path) { ... }
pub fn visit_identifier(node: &Identifier) { ... }

// Babel: FunctionDeclaration(path) { ... }
pub fn visit_function_declaration(node: &FunctionDeclaration) { ... }

// Babel: CallExpression(path) { ... }
pub fn visit_call_expression(node: &CallExpression) { ... }

// Babel: JSXElement(path) { ... }
pub fn visit_jsx_element(node: &JSXElement) { ... }
```

Use `&mut` for mutable access:
```reluxscript
pub fn visit_identifier(node: &mut Identifier) {
    node.name = "newName".to_string();
}
```

### 3. Type Checking with `matches!`

Instead of Babel's `t.isIdentifier()`, use `matches!`:

```reluxscript
// Babel: if (t.isIdentifier(node)) { ... }
if matches!(node, Identifier) {
    let name = node.name.clone();
}

// Babel: if (t.isCallExpression(node.init)) { ... }
if matches!(node.init, CallExpression) {
    let callee = node.init.callee.clone();
}

// Works with field access too
if matches!(decl.id, ArrayPattern) {
    let elements = decl.id.elements.clone();
}
```

### 4. Pattern Matching

ReluxScript uses Rust-style pattern matching:

```reluxscript
// Simple if-let for Option unwrapping
if let Some(init) = &decl.init {
    // init is now the unwrapped value
}

// Pattern matching on node types
if matches!(node, Identifier) {
    // node is an Identifier
} else if matches!(node, MemberExpression) {
    // node is a MemberExpression
}

// Full match statement
match node.kind {
    "const" => { /* ... */ }
    "let" => { /* ... */ }
    _ => { /* default */ }
}
```

### 5. Loops and Iteration

```reluxscript
// Iterate over array elements
for elem in &node.elements {
    if let Some(e) = elem {
        // Process element
    }
}

// Iterate over object properties
for prop in &node.properties {
    if matches!(prop, Property) {
        let key = prop.key.clone();
    }
}

// While loops
let mut i = 0;
while i < items.len() {
    let item = &items[i];
    i += 1;
}
```

### 6. Working with Strings

```reluxscript
// String literals
let name = "hello";

// Clone to own a string
let owned = node.name.clone();

// String comparison
if name == "useState" {
    // ...
}

// String methods
if name.starts_with("use") {
    // It's a hook
}

// Building strings with Vec
let mut parts = vec![];
parts.push("a");
parts.push("b");
let result = parts.join(".");  // "a.b"
```

### 7. Collections

```reluxscript
// Vec (array)
let mut items: Vec<Str> = vec![];
items.push("item");
let len = items.len();
let first = &items[0];

// Iteration
for item in &items {
    // process item
}
```

## Common Patterns

### Extracting Hook Calls

**Babel:**
```javascript
VariableDeclarator(path) {
  if (t.isCallExpression(path.node.init)) {
    const callee = path.node.init.callee;
    if (t.isIdentifier(callee) && callee.name === 'useState') {
      // Extract useState
    }
  }
}
```

**ReluxScript:**
```reluxscript
pub fn visit_variable_declarator(decl: &VariableDeclarator) {
    if let Some(init) = &decl.init {
        if matches!(init, CallExpression) {
            if matches!(init.callee, Identifier) {
                let callee_name = init.callee.name.clone();
                if callee_name == "useState" {
                    // Extract useState
                }
            }
        }
    }
}
```

### Array Destructuring Pattern

**Babel:**
```javascript
// const [value, setValue] = useState(initial)
if (t.isArrayPattern(decl.id)) {
  const [valueId, setterId] = decl.id.elements;
  const valueName = valueId.name;
  const setterName = setterId.name;
}
```

**ReluxScript:**
```reluxscript
// const [value, setValue] = useState(initial)
if matches!(decl.id, ArrayPattern) {
    let arr = decl.id.clone();

    if arr.elements.len() > 0 {
        if let Some(first) = &arr.elements[0] {
            if matches!(first, Identifier) {
                let value_name = first.name.clone();
            }
        }
    }

    if arr.elements.len() > 1 {
        if let Some(second) = &arr.elements[1] {
            if matches!(second, Identifier) {
                let setter_name = second.name.clone();
            }
        }
    }
}
```

### Building Member Expressions

**Babel:**
```javascript
function buildMemberPath(base, path) {
  const parts = path.split('.');
  let current = t.identifier(base);
  for (const part of parts) {
    current = t.memberExpression(current, t.identifier(part));
  }
  return current;
}
```

**ReluxScript:**
```reluxscript
fn build_member_path(base: &Str, path: &Str) -> Str {
    let mut parts = vec![];
    parts.push(base.clone());

    // Split path and add each part
    for part in path.split(".") {
        parts.push(part.to_string());
    }

    return parts.join(".");
}
```

### Walking the AST

**Babel:**
```javascript
FunctionDeclaration(path) {
  path.traverse({
    CallExpression(innerPath) {
      // Process nested call expressions
    }
  });
}
```

**ReluxScript:**
```reluxscript
pub fn visit_function_declaration(node: &FunctionDeclaration) {
    if let Some(body) = &node.body {
        traverse(body) {
            fn visit_call_expression(call: &CallExpression) {
                // Process nested call expressions
            }
        }
    }
}
```

### Checking for React Components

**Babel:**
```javascript
function isComponent(name) {
  return /^[A-Z]/.test(name);
}

FunctionDeclaration(path) {
  if (isComponent(path.node.id.name)) {
    // It's a React component
  }
}
```

**ReluxScript:**
```reluxscript
fn is_component(name: &Str) -> bool {
    let first = name.chars().next();
    if let Some(c) = first {
        return c.is_uppercase();
    }
    return false;
}

pub fn visit_function_declaration(node: &FunctionDeclaration) {
    let name = node.id.name.clone();
    if is_component(&name) {
        // It's a React component
    }
}
```

## AST Node Type Mappings

ReluxScript uses consistent names that map to both Babel and SWC:

| ReluxScript | Babel | SWC |
|------------|-------|-----|
| `Identifier` | `Identifier` | `Ident` |
| `CallExpression` | `CallExpression` | `CallExpr` |
| `MemberExpression` | `MemberExpression` | `MemberExpr` |
| `FunctionDeclaration` | `FunctionDeclaration` | `FnDecl` |
| `VariableDeclarator` | `VariableDeclarator` | `VarDeclarator` |
| `ArrayPattern` | `ArrayPattern` | `ArrayPat` |
| `ObjectPattern` | `ObjectPattern` | `ObjectPat` |
| `StringLiteral` | `StringLiteral` | `Str` |
| `NumericLiteral` | `NumericLiteral` | `Number` |
| `BooleanLiteral` | `BooleanLiteral` | `Bool` |
| `JSXElement` | `JSXElement` | `JSXElement` |
| `JSXAttribute` | `JSXAttribute` | `JSXAttr` |

## Field Name Mappings

Some fields have different names between platforms:

| ReluxScript | Babel | SWC | Notes |
|------------|-------|-----|-------|
| `node.name` | `node.name` | `node.sym` | Identifier name |
| `decl.id` | `decl.id` | `decl.name` | Variable declarator pattern |
| `node.property` | `node.property` | `node.prop` | Member expression property |
| `node.elements` | `node.elements` | `node.elems` | Array elements |
| `node.arguments` | `node.arguments` | `node.args` | Call arguments |
| `lit.value` | `lit.value` | `lit.value` | Literal value |

## TypeScript Node Support

ReluxScript can process TypeScript AST nodes:

```reluxscript
pub fn visit_ts_interface_declaration(node: &TSInterfaceDeclaration) {
    let interface_name = node.id.name.clone();

    for member in &node.body.body {
        if matches!(member, TSPropertySignature) {
            let prop_name = member.key.name.clone();
            // Process property
        }
    }
}

pub fn visit_call_expression(node: &CallExpression) {
    // Check for type arguments like useState<string>
    if node.type_args.len() > 0 {
        let first_type = &node.type_args[0];
        // Process type argument
    }
}
```

## Key Differences from Babel

### 1. Explicit Cloning

ReluxScript requires explicit `.clone()` when you need to own a value:

```reluxscript
// Need to clone to store or return
let name = node.name.clone();

// References are fine for immediate use
if &node.name == "foo" { ... }
```

### 2. Option Handling

Many fields that are nullable in Babel are `Option<T>` in ReluxScript:

```reluxscript
// Babel: if (node.init) { ... }
// ReluxScript:
if let Some(init) = &node.init {
    // init is now available
}
```

### 3. No Implicit Type Coercion

```reluxscript
// Babel: if (node.elements.length) { ... }
// ReluxScript:
if node.elements.len() > 0 {
    // ...
}
```

### 4. Return Statements

All functions need explicit returns:

```reluxscript
fn get_name(node: &Identifier) -> Str {
    return node.name.clone();  // explicit return
}
```

### 5. Mutable vs Immutable

Variables are immutable by default:

```reluxscript
let x = 5;        // immutable
let mut y = 5;    // mutable
y = 10;           // OK
// x = 10;        // Error!
```

## Building and Running

### Compile to Babel
```bash
reluxscript build my-plugin.lux --target babel -o dist/
# Output: dist/index.js
```

### Compile to SWC
```bash
reluxscript build my-plugin.lux --target swc -o dist/
# Output: dist/lib.rs
```

### Compile to Both
```bash
reluxscript build my-plugin.lux --target both -o dist/
# Output: dist/index.js and dist/lib.rs
```

### Check for Errors
```bash
reluxscript check my-plugin.lux
```

## Example: Complete Plugin

Here's a complete example that extracts React hooks from components:

```reluxscript
/// Hook Extractor Plugin
/// Extracts useState calls from React components

plugin HookExtractor {
    struct HookInfo {
        state_var: Str,
        setter_var: Str,
        initial_value: Str,
    }

    /// Check if function is a React component
    fn is_component(name: &Str) -> bool {
        let first = name.chars().next();
        if let Some(c) = first {
            return c.is_uppercase();
        }
        return false;
    }

    /// Main visitor for function declarations
    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        let name = node.id.name.clone();

        if !is_component(&name) {
            return;
        }

        let mut hooks: Vec<HookInfo> = vec![];

        // Traverse function body for hook calls
        if let Some(body) = &node.body {
            traverse(body) {
                fn visit_variable_declarator(decl: &VariableDeclarator) {
                    if let Some(init) = &decl.init {
                        if matches!(init, CallExpression) {
                            if matches!(init.callee, Identifier) {
                                if init.callee.name == "useState" {
                                    extract_use_state(decl, init, &mut hooks);
                                }
                            }
                        }
                    }
                }
            }
        }

        // Process extracted hooks
        for hook in &hooks {
            // Generate output or transform
        }
    }

    /// Extract useState hook information
    fn extract_use_state(
        decl: &VariableDeclarator,
        call: &CallExpression,
        hooks: &mut Vec<HookInfo>
    ) {
        if matches!(decl.id, ArrayPattern) {
            let arr = decl.id.clone();

            let mut state_var = "".to_string();
            let mut setter_var = "".to_string();

            // Get state variable name
            if arr.elements.len() > 0 {
                if let Some(first) = &arr.elements[0] {
                    if matches!(first, Identifier) {
                        state_var = first.name.clone();
                    }
                }
            }

            // Get setter function name
            if arr.elements.len() > 1 {
                if let Some(second) = &arr.elements[1] {
                    if matches!(second, Identifier) {
                        setter_var = second.name.clone();
                    }
                }
            }

            // Get initial value
            let mut initial_value = "null".to_string();
            if call.arguments.len() > 0 {
                let arg = &call.arguments[0];
                if matches!(arg.expression, StringLiteral) {
                    initial_value = arg.expression.value.clone();
                }
            }

            hooks.push(HookInfo {
                state_var: state_var,
                setter_var: setter_var,
                initial_value: initial_value,
            });
        }
    }
}
```

## Tips for Babel Developers

1. **Think in Types**: ReluxScript is statically typed. The compiler catches errors that would be runtime errors in Babel.

2. **Clone When Needed**: Use `.clone()` when storing values. References (`&`) are for temporary access.

3. **Handle Options**: Use `if let Some(x) = ...` for nullable values instead of truthy checks.

4. **Use matches!**: It's your replacement for `t.is*()` functions.

5. **Explicit Returns**: Always use `return` statement in functions that return values.

6. **Check the Mappings**: Field names differ between platforms. ReluxScript abstracts this, but understanding helps debugging.

7. **Start Simple**: Port small Babel plugins first to learn the patterns.

## Next Steps

- Read the [ReluxScript Language Specification](./reluxscript-spec.md)
- Check out the [AST Node Reference](./ast-reference.md)
- See [Enhancement Plan](./reluxscript-plugin-enhancements.md) for upcoming features
- Browse example plugins in `reluxscript/examples/`

## Getting Help

- Check error messages carefully - they include hints
- Use `reluxscript check` to validate before building
- The field and node mappings in `src/mapping/` are the source of truth
