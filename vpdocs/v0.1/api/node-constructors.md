# Node Constructors

API for creating new AST nodes in ReluxScript.

## Basic Constructors

### Identifier

```reluxscript
let id = Identifier::new("myVar");
```

### String Literal

```reluxscript
let str = StringLiteral::new("hello");
```

### Numeric Literal

```reluxscript
let num = NumericLiteral::new(42);
```

### Boolean Literal

```reluxscript
let bool_true = BooleanLiteral::new(true);
let bool_false = BooleanLiteral::new(false);
```

## Expression Constructors

### Call Expression

```reluxscript
let call = CallExpression {
    callee: Identifier::new("console.log"),
    arguments: vec![StringLiteral::new("message")],
};
```

### Member Expression

```reluxscript
let member = MemberExpression {
    object: Identifier::new("console"),
    property: Identifier::new("log"),
    computed: false,
};
```

### Binary Expression

```reluxscript
let binary = BinaryExpression {
    left: Identifier::new("a"),
    operator: "+",
    right: Identifier::new("b"),
};
```

### Arrow Function

```reluxscript
let arrow = ArrowFunctionExpression {
    params: vec![Pattern::Identifier(Identifier::new("x"))],
    body: Expression::BinaryExpression(BinaryExpression {
        left: Identifier::new("x"),
        operator: "*",
        right: NumericLiteral::new(2),
    }),
    async: false,
};
```

## Statement Constructors

### Empty Statement

```reluxscript
let empty = Statement::empty();
```

### Expression Statement

```reluxscript
let expr_stmt = Statement::expression(call_expression);
```

### Return Statement

```reluxscript
let ret = ReturnStatement {
    argument: Some(Identifier::new("value")),
};
```

### Variable Declaration

```reluxscript
let var_decl = VariableDeclaration {
    kind: "const",
    declarations: vec![
        VariableDeclarator {
            id: Pattern::Identifier(Identifier::new("x")),
            init: Some(NumericLiteral::new(42)),
        }
    ],
};
```

## Collection Constructors

### Array Expression

```reluxscript
let arr = ArrayExpression {
    elements: vec![
        Some(NumericLiteral::new(1)),
        Some(NumericLiteral::new(2)),
        Some(NumericLiteral::new(3)),
    ],
};
```

### Object Expression

```reluxscript
let obj = ObjectExpression {
    properties: vec![
        ObjectProperty {
            key: Identifier::new("name"),
            value: StringLiteral::new("John"),
        },
        ObjectProperty {
            key: Identifier::new("age"),
            value: NumericLiteral::new(30),
        },
    ],
};
```

## Helper Methods

### Clone Node

```reluxscript
let new_node = node.clone();
```

### Check Node Type

```reluxscript
if node.is_identifier() { }
if node.is_call_expression() { }
```

### Get Node Type

```reluxscript
let node_type = node.type_name();  // Returns "Identifier", "CallExpression", etc.
```

## Common Patterns

### Create Function Call

```reluxscript
fn create_call(fn_name: &str, args: Vec<Expression>) -> CallExpression {
    CallExpression {
        callee: Identifier::new(fn_name),
        arguments: args,
    }
}
```

### Create Constant Declaration

```reluxscript
fn create_const(name: &str, value: Expression) -> VariableDeclaration {
    VariableDeclaration {
        kind: "const",
        declarations: vec![
            VariableDeclarator {
                id: Pattern::Identifier(Identifier::new(name)),
                init: Some(value),
            }
        ],
    }
}
```

### Wrap in Block

```reluxscript
fn wrap_in_block(statements: Vec<Statement>) -> BlockStatement {
    BlockStatement {
        body: statements,
    }
}
```

See [Visitor Methods](/v0.1/api/visitor-methods) and [Node Types](/v0.1/language/node-types) for more.
