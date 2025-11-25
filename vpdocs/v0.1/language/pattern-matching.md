# Pattern Matching

Pattern matching is a powerful feature in ReluxScript for working with AST nodes and data structures.

## The `matches!` Macro

Check if a value matches a pattern:

```reluxscript
// Simple name match
if matches!(node.callee, "console.log") {
    // Matched
}

// Operator match
if matches!(node.operator, "+") {
    // Addition operator
}

// Complex pattern
if matches!(node.callee, MemberExpression {
    object: Identifier { name: "console" },
    property: Identifier { name: "log" }
}) {
    // console.log()
}
```

## Match Expression

Pattern match with multiple branches:

```reluxscript
match node.operator {
    "+" => handle_addition(),
    "-" => handle_subtraction(),
    "*" | "/" => handle_mul_div(),
    _ => handle_other(),
}
```

### With Values

```reluxscript
let result = match node.type {
    "Identifier" => "id",
    "Literal" => "lit",
    _ => "other",
};
```

## If-Let Pattern

Conditionally extract values:

```reluxscript
if let Some(name) = get_identifier_name(node) {
    println!("Name: {}", name);
}

if let Ok(value) = parse_result {
    process(value);
}
```

## Destructuring

### Tuple Destructuring

```reluxscript
let (x, y) = get_coordinates();
let (name, age, city) = person_info;
```

### Struct Destructuring

```reluxscript
match &node.callee {
    MemberExpression { object, property } => {
        // Use object and property
    }
    _ => {}
}
```

## Pattern Types

### Literal Patterns

```reluxscript
match value {
    42 => "answer",
    0 => "zero",
    _ => "other",
}
```

### Identifier Patterns

```reluxscript
match node {
    Identifier { name } => println!("{}", name),
    _ => {},
}
```

### Wildcard Pattern

```reluxscript
match value {
    Some(x) => x,
    _ => default,  // Matches anything
}
```

### Multiple Patterns

```reluxscript
match op {
    "+" | "-" => "arithmetic",
    "*" | "/" | "%" => "multiplicative",
    _ => "other",
}
```

## Advanced Patterns

### Nested Patterns

```reluxscript
match expr {
    CallExpression {
        callee: MemberExpression {
            object: Identifier { name: "console" }
        }
    } => {
        // console.method()
    }
    _ => {}
}
```

### Guards (Coming Soon)

```reluxscript
// Future feature
match value {
    Some(x) if x > 0 => "positive",
    Some(x) if x < 0 => "negative",
    Some(0) => "zero",
    None => "none",
}
```

## Common Patterns

### Check Node Type

```reluxscript
if matches!(node, Expression::Identifier(_)) {
    // Is an identifier
}
```

### Extract Optional Value

```reluxscript
if let Some(arg) = node.argument {
    process(arg);
}
```

### Handle Result

```reluxscript
match parse_file(path) {
    Ok(ast) => process(ast),
    Err(msg) => eprintln!("Error: {}", msg),
}
```

### Check Multiple Conditions

```reluxscript
if matches!(node.callee, "useState") && node.type_args.is_some() {
    // useState<T>()
}
```

See [Syntax Overview](/v0.1/language/syntax) for more examples.
