# AST Node Types

Complete reference of AST node types available in ReluxScript.

## Expression Nodes

### Identifier

Variable or property name.

```reluxscript
struct Identifier {
    name: Str,
}
```

**Examples:** `foo`, `bar`, `myVariable`

---

### CallExpression

Function or method call.

```reluxscript
struct CallExpression {
    callee: Expression,
    arguments: Vec<Expression>,
    type_args: Option<TypeArgs>,
}
```

**Examples:** `fn()`, `obj.method(arg)`, `useState<string>()`

---

### MemberExpression

Property access.

```reluxscript
struct MemberExpression {
    object: Expression,
    property: Expression,
    computed: bool,
}
```

**Examples:** `obj.prop`, `arr[index]`, `console.log`

---

### BinaryExpression

Binary operations.

```reluxscript
struct BinaryExpression {
    left: Expression,
    operator: Str,
    right: Expression,
}
```

**Examples:** `a + b`, `x * y`, `foo == bar`

**Operators:** `+`, `-`, `*`, `/`, `%`, `==`, `!=`, `<`, `>`, `<=`, `>=`, `&&`, `||`

---

### UnaryExpression

Unary operations.

```reluxscript
struct UnaryExpression {
    operator: Str,
    argument: Expression,
    prefix: bool,
}
```

**Examples:** `!flag`, `-num`, `typeof x`

**Operators:** `!`, `-`, `+`, `~`, `typeof`, `void`, `delete`

---

### ArrowFunctionExpression

Arrow function.

```reluxscript
struct ArrowFunctionExpression {
    params: Vec<Pattern>,
    body: BlockStatement | Expression,
    async: bool,
}
```

**Examples:** `() => {}`, `(x) => x * 2`, `async () => await fetch()`

---

### ObjectExpression

Object literal.

```reluxscript
struct ObjectExpression {
    properties: Vec<ObjectProperty>,
}
```

**Examples:** `{}`, `{ a: 1, b: 2 }`, `{ name, age: 30 }`

---

### ArrayExpression

Array literal.

```reluxscript
struct ArrayExpression {
    elements: Vec<Option<Expression>>,
}
```

**Examples:** `[]`, `[1, 2, 3]`, `[...items, newItem]`

---

## Statement Nodes

### FunctionDeclaration

Function declaration.

```reluxscript
struct FunctionDeclaration {
    id: Identifier,
    params: Vec<Pattern>,
    body: BlockStatement,
    async: bool,
    generator: bool,
}
```

**Examples:** `function foo() {}`, `async function getData() {}`

---

### VariableDeclaration

Variable declaration.

```reluxscript
struct VariableDeclaration {
    kind: Str,  // "var", "let", or "const"
    declarations: Vec<VariableDeclarator>,
}
```

**Examples:** `let x = 1;`, `const name = "foo";`

---

### IfStatement

If statement.

```reluxscript
struct IfStatement {
    test: Expression,
    consequent: Statement,
    alternate: Option<Statement>,
}
```

**Examples:** `if (x) {}`, `if (y) {} else {}`

---

### ForStatement

For loop.

```reluxscript
struct ForStatement {
    init: Option<ForInit>,
    test: Option<Expression>,
    update: Option<Expression>,
    body: Statement,
}
```

**Examples:** `for (let i = 0; i < 10; i++) {}`

---

### WhileStatement

While loop.

```reluxscript
struct WhileStatement {
    test: Expression,
    body: Statement,
}
```

**Examples:** `while (condition) {}`

---

### ReturnStatement

Return statement.

```reluxscript
struct ReturnStatement {
    argument: Option<Expression>,
}
```

**Examples:** `return;`, `return value;`

---

### BlockStatement

Code block.

```reluxscript
struct BlockStatement {
    body: Vec<Statement>,
}
```

**Examples:** `{}`, `{ stmt1; stmt2; }`

---

## JSX Nodes

### JSXElement

JSX element.

```reluxscript
struct JSXElement {
    opening_element: JSXOpeningElement,
    closing_element: Option<JSXClosingElement>,
    children: Vec<JSXChild>,
}
```

**Examples:** `<div />`, `<Component prop="value" />`, `<div>content</div>`

---

### JSXAttribute

JSX attribute.

```reluxscript
struct JSXAttribute {
    name: JSXIdentifier,
    value: Option<JSXAttributeValue>,
}
```

**Examples:** `key="id"`, `onClick={handler}`, `disabled`

---

## TypeScript Nodes

### TSInterfaceDeclaration

TypeScript interface.

```reluxscript
struct TSInterfaceDeclaration {
    id: Identifier,
    body: TSInterfaceBody,
}
```

**Examples:** `interface User {}`, `interface Props extends Base {}`

---

### TSTypeAnnotation

Type annotation.

```reluxscript
struct TSTypeAnnotation {
    type_annotation: TSType,
}
```

**Examples:** `: string`, `: number[]`, `: { name: string }`

---

## Literal Nodes

### StringLiteral

```reluxscript
struct StringLiteral {
    value: Str,
}
```

**Example:** `"hello"`

---

### NumericLiteral

```reluxscript
struct NumericLiteral {
    value: f64,
}
```

**Examples:** `42`, `3.14`

---

### BooleanLiteral

```reluxscript
struct BooleanLiteral {
    value: bool,
}
```

**Examples:** `true`, `false`

---

### NullLiteral

```reluxscript
struct NullLiteral {}
```

**Example:** `null`

---

## Node Constructors

Create new nodes:

```reluxscript
// Identifier
Identifier::new("myVar")

// Call expression
CallExpression {
    callee: Identifier::new("foo"),
    arguments: vec![StringLiteral::new("arg")],
}

// String literal
StringLiteral::new("hello")

// Empty statement
Statement::empty()
```

## Node Type Checking

Check node types:

```reluxscript
if node.is_identifier() { }
if node.is_call_expression() { }
if node.is_member_expression() { }
```

## Complete Node Mapping

| ReluxScript | Babel | SWC |
|-------------|-------|-----|
| `Identifier` | `Identifier` | `Ident` |
| `CallExpression` | `CallExpression` | `CallExpr` |
| `MemberExpression` | `MemberExpression` | `MemberExpr` |
| `BinaryExpression` | `BinaryExpression` | `BinExpr` |
| `FunctionDeclaration` | `FunctionDeclaration` | `FnDecl` |
| `VariableDeclaration` | `VariableDeclaration` | `VarDecl` |
| `JSXElement` | `JSXElement` | `JSXElement` |

See the [full specification](/v0.1/language/specification) for complete mappings.
