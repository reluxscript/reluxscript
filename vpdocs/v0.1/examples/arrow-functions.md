# Arrow Functions Example

Convert arrow functions to regular function expressions.

## Code

```reluxscript
plugin ArrowToFunction {
    fn visit_arrow_function(node: &mut ArrowFunctionExpression, ctx: &Context) {
        *node = FunctionExpression {
            id: None,
            params: node.params.clone(),
            body: convert_body(&node.body),
            async: node.async,
            generator: false,
        };
    }
}

fn convert_body(body: &ArrowBody) -> BlockStatement {
    match body {
        ArrowBody::BlockStatement(block) => block.clone(),
        ArrowBody::Expression(expr) => BlockStatement {
            body: vec![ReturnStatement { argument: Some(expr.clone()) }],
        },
    }
}
```

## Input

```javascript
const add = (a, b) => a + b;
const log = (msg) => {
    console.log(msg);
};
```

## Output

```javascript
const add = function(a, b) { return a + b; };
const log = function(msg) {
    console.log(msg);
};
```

## How It Works

1. `visit_arrow_function` is called for every arrow function
2. Create a new `FunctionExpression` with the same parameters and body
3. If the arrow function body is an expression, wrap it in a return statement
4. Replace the arrow function with the function expression

[Back to Examples](/v0.1/examples/)
