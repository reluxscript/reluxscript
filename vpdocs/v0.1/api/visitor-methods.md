# Visitor Methods API

This page documents all available visitor methods in ReluxScript.

## Overview

Visitor methods are called automatically when the AST traversal encounters matching node types. Each method receives:

- `node: &mut NodeType` - Mutable reference to the AST node
- `ctx: &Context` - Context object with scope and utilities

## Expression Visitors

### visit_identifier

Called for every identifier (variable name).

```reluxscript
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    // node.name: Str - The identifier name
    println!("Found identifier: {}", node.name);
}
```

**Example matches:**
- `foo`
- `bar`
- `myVariable`

---

### visit_call_expression

Called for function and method calls.

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    // node.callee: Expression - The function being called
    // node.arguments: Vec<Expression> - Call arguments
    // node.type_args: Option<TypeArgs> - Generic type arguments

    if matches!(node.callee, "console.log") {
        *node = Statement::empty();
    }
}
```

**Example matches:**
- `foo()`
- `obj.method(arg1, arg2)`
- `useState<string>()`

---

### visit_member_expression

Called for property access.

```reluxscript
fn visit_member_expression(node: &mut MemberExpression, ctx: &Context) {
    // node.object: Expression - The object being accessed
    // node.property: Expression - The property name
    // node.computed: bool - true for obj[prop], false for obj.prop

    if matches!(node.object, Identifier { name: "console" }) {
        // Accessing console.*
    }
}
```

**Example matches:**
- `obj.property`
- `arr[index]`
- `console.log`

---

### visit_binary_expression

Called for binary operations.

```reluxscript
fn visit_binary_expression(node: &mut BinaryExpression, ctx: &Context) {
    // node.left: Expression - Left operand
    // node.operator: Str - Operator (+, -, *, /, ==, etc.)
    // node.right: Expression - Right operand

    match node.operator.as_str() {
        "+" => handle_addition(),
        "-" => handle_subtraction(),
        _ => {}
    }
}
```

**Example matches:**
- `a + b`
- `x * y`
- `foo == bar`

---

### visit_unary_expression

Called for unary operations.

```reluxscript
fn visit_unary_expression(node: &mut UnaryExpression, ctx: &Context) {
    // node.operator: Str - Operator (!, -, +, ~, typeof, etc.)
    // node.argument: Expression - Operand
    // node.prefix: bool - true for prefix, false for postfix

    if node.operator == "!" {
        // Logical NOT
    }
}
```

**Example matches:**
- `!flag`
- `-num`
- `typeof x`

---

### visit_arrow_function

Called for arrow functions.

```reluxscript
fn visit_arrow_function(node: &mut ArrowFunctionExpression, ctx: &Context) {
    // node.params: Vec<Pattern> - Parameters
    // node.body: BlockStatement or Expression
    // node.async: bool - Is async function

    if node.async {
        // Handle async arrow function
    }
}
```

**Example matches:**
- `() => {}`
- `(x) => x * 2`
- `async (data) => await fetch(data)`

---

### visit_object_expression

Called for object literals.

```reluxscript
fn visit_object_expression(node: &mut ObjectExpression, ctx: &Context) {
    // node.properties: Vec<ObjectProperty> - Object properties

    for prop in &node.properties {
        // Process each property
    }
}
```

**Example matches:**
- `{}`
- `{ a: 1, b: 2 }`
- `{ name, age: 30 }`

---

### visit_array_expression

Called for array literals.

```reluxscript
fn visit_array_expression(node: &mut ArrayExpression, ctx: &Context) {
    // node.elements: Vec<Option<Expression>> - Array elements

    for elem in &node.elements {
        if let Some(expr) = elem {
            // Process element
        }
    }
}
```

**Example matches:**
- `[]`
- `[1, 2, 3]`
- `[...items, newItem]`

---

## Statement Visitors

### visit_function_declaration

Called for function declarations.

```reluxscript
fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
    // node.id: Identifier - Function name
    // node.params: Vec<Pattern> - Parameters
    // node.body: BlockStatement - Function body
    // node.async: bool - Is async
    // node.generator: bool - Is generator

    if node.async {
        // Handle async function
    }
}
```

**Example matches:**
- `function foo() {}`
- `async function getData() {}`
- `function* generator() {}`

---

### visit_variable_declaration

Called for variable declarations.

```reluxscript
fn visit_variable_declaration(node: &mut VariableDeclaration, ctx: &Context) {
    // node.kind: Str - "var", "let", or "const"
    // node.declarations: Vec<VariableDeclarator>

    for decl in &node.declarations {
        // Process each declarator
    }
}
```

**Example matches:**
- `let x = 1;`
- `const name = "foo";`
- `var a, b, c;`

---

### visit_if_statement

Called for if statements.

```reluxscript
fn visit_if_statement(node: &mut IfStatement, ctx: &Context) {
    // node.test: Expression - Condition
    // node.consequent: Statement - Then branch
    // node.alternate: Option<Statement> - Else branch

    if node.alternate.is_some() {
        // Has else clause
    }
}
```

**Example matches:**
- `if (x) {}`
- `if (y) {} else {}`
- `if (a) {} else if (b) {}`

---

### visit_for_statement

Called for for loops.

```reluxscript
fn visit_for_statement(node: &mut ForStatement, ctx: &Context) {
    // node.init: Option<ForInit> - Initializer
    // node.test: Option<Expression> - Condition
    // node.update: Option<Expression> - Update
    // node.body: Statement - Loop body
}
```

**Example matches:**
- `for (let i = 0; i < 10; i++) {}`
- `for (;;) {}`

---

### visit_while_statement

Called for while loops.

```reluxscript
fn visit_while_statement(node: &mut WhileStatement, ctx: &Context) {
    // node.test: Expression - Condition
    // node.body: Statement - Loop body
}
```

**Example matches:**
- `while (condition) {}`
- `while (true) {}`

---

### visit_return_statement

Called for return statements.

```reluxscript
fn visit_return_statement(node: &mut ReturnStatement, ctx: &Context) {
    // node.argument: Option<Expression> - Return value

    if let Some(expr) = &node.argument {
        // Has return value
    }
}
```

**Example matches:**
- `return;`
- `return value;`
- `return x + y;`

---

### visit_block_statement

Called for block statements (code blocks).

```reluxscript
fn visit_block_statement(node: &mut BlockStatement, ctx: &Context) {
    // node.body: Vec<Statement> - Statements in block

    for stmt in &mut node.body {
        // Process each statement
    }
}
```

**Example matches:**
- `{}`
- `{ stmt1; stmt2; }`
- Function/loop bodies

---

## JSX Visitors

### visit_jsx_element

Called for JSX elements.

```reluxscript
fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
    // node.opening_element: JSXOpeningElement
    // node.closing_element: Option<JSXClosingElement>
    // node.children: Vec<JSXChild>

    let tag_name = &node.opening_element.name;
    // Process JSX element
}
```

**Example matches:**
- `<div />`
- `<Component prop="value" />`
- `<div>content</div>`

---

### visit_jsx_attribute

Called for JSX attributes.

```reluxscript
fn visit_jsx_attribute(node: &mut JSXAttribute, ctx: &Context) {
    // node.name: JSXIdentifier - Attribute name
    // node.value: Option<JSXAttributeValue> - Attribute value

    if node.name.name == "key" {
        // Found key attribute
    }
}
```

**Example matches:**
- `key="id"`
- `onClick={handler}`
- `disabled`

---

## TypeScript Visitors

### visit_ts_interface_declaration

Called for TypeScript interfaces.

```reluxscript
fn visit_ts_interface_declaration(node: &mut TSInterfaceDeclaration, ctx: &Context) {
    // node.id: Identifier - Interface name
    // node.body: TSInterfaceBody - Interface members

    println!("Interface: {}", node.id.name);
}
```

**Example matches:**
- `interface User {}`
- `interface Props extends BaseProps {}`

---

### visit_ts_type_annotation

Called for TypeScript type annotations.

```reluxscript
fn visit_ts_type_annotation(node: &mut TSTypeAnnotation, ctx: &Context) {
    // node.type_annotation: TSType - The type

    // Process type annotation
}
```

**Example matches:**
- `: string`
- `: number[]`
- `: { name: string }`

---

## Program Hooks

### pre (or init)

Called before traversal begins. Used for initialization.

```reluxscript
plugin MyPlugin {
    fn pre(file: &File) {
        // Initialize state
        file.metadata.originalCode = file.code;
    }
}
```

**Alternative name:** `init()`

---

### exit (or finish)

Called after traversal completes. Used for finalization.

```reluxscript
plugin MyPlugin {
    fn exit(program: &mut Program, state: &PluginState) {
        // Generate output, write files, etc.
        generate_report(state);
    }
}
```

**Alternative name:** `finish()`

---

## Context API

The `Context` object provides utilities and information:

### ctx.scope

Scope information (expensive in SWC, requires pre-pass).

```reluxscript
// Check if name is bound in scope
if ctx.scope.has_binding("foo") {
    // 'foo' is declared in this scope
}

// Get binding information
let binding = ctx.scope.get_binding("foo");
```

### ctx.generate_uid(hint)

Generate a unique identifier.

```reluxscript
let temp_name = ctx.generate_uid("temp");
// Returns: "_temp", "_temp2", etc.
```

### ctx.filename

Current source filename.

```reluxscript
println!("Processing: {}", ctx.filename);
```

---

## Node Replacement

Replace nodes using the dereference operator:

```reluxscript
// Replace with empty statement
*node = Statement::empty();

// Replace with new node
*node = CallExpression {
    callee: Identifier::new("newFunc"),
    arguments: vec![],
};

// Replace with cloned node
*node = other_node.clone();
```

---

## Traversal Control

### Skip Children

Don't call `visit_children()` to skip traversing child nodes:

```reluxscript
fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
    // Don't traverse function body
    // (omit node.visit_children(self))
}
```

### Explicit Traversal

Manually traverse children:

```reluxscript
fn visit_block_statement(node: &mut BlockStatement, ctx: &Context) {
    for stmt in &mut node.body {
        stmt.visit_with(self);
    }
}
```

### Nested Visitors

Use `traverse` for scoped traversal with different rules:

```reluxscript
fn visit_function_declaration(func: &mut FunctionDeclaration, ctx: &Context) {
    traverse(&mut func.body) {
        let mut count = 0;

        fn visit_return_statement(ret: &mut ReturnStatement, ctx: &Context) {
            self.count += 1;
        }
    }
}
```

---

## Complete Node Type List

| Visitor Method | Node Type | Example |
|----------------|-----------|---------|
| `visit_identifier` | `Identifier` | `foo` |
| `visit_call_expression` | `CallExpression` | `fn()` |
| `visit_member_expression` | `MemberExpression` | `obj.prop` |
| `visit_binary_expression` | `BinaryExpression` | `a + b` |
| `visit_unary_expression` | `UnaryExpression` | `!x` |
| `visit_arrow_function` | `ArrowFunctionExpression` | `() => {}` |
| `visit_function_expression` | `FunctionExpression` | `function() {}` |
| `visit_function_declaration` | `FunctionDeclaration` | `function foo() {}` |
| `visit_variable_declaration` | `VariableDeclaration` | `let x = 1` |
| `visit_variable_declarator` | `VariableDeclarator` | `x = 1` |
| `visit_if_statement` | `IfStatement` | `if (x) {}` |
| `visit_for_statement` | `ForStatement` | `for(;;) {}` |
| `visit_while_statement` | `WhileStatement` | `while(x) {}` |
| `visit_return_statement` | `ReturnStatement` | `return x` |
| `visit_block_statement` | `BlockStatement` | `{ }` |
| `visit_expression_statement` | `ExpressionStatement` | `expr;` |
| `visit_object_expression` | `ObjectExpression` | `{ a: 1 }` |
| `visit_array_expression` | `ArrayExpression` | `[1, 2]` |
| `visit_string_literal` | `StringLiteral` | `"text"` |
| `visit_numeric_literal` | `NumericLiteral` | `42` |
| `visit_boolean_literal` | `BooleanLiteral` | `true` |
| `visit_jsx_element` | `JSXElement` | `<div />` |
| `visit_jsx_attribute` | `JSXAttribute` | `key="x"` |
| `visit_ts_interface_declaration` | `TSInterfaceDeclaration` | `interface I {}` |
| `visit_ts_type_annotation` | `TSTypeAnnotation` | `: string` |

---

## Next Steps

- [See Examples](/v0.1/examples/)
- [Learn Core Concepts](/v0.1/guide/concepts)
- [Read the Specification](/v0.1/language/specification)
