# Expressions

Expressions are constructs that evaluate to a value.

## Literals

```reluxscript
"hello"         // String literal
42              // Integer literal
3.14            // Float literal
true            // Boolean literal
null            // Null literal
()              // Unit literal
```

## Binary Expressions

```reluxscript
a + b           // Addition
x - y           // Subtraction
m * n           // Multiplication
p / q           // Division
a % b           // Modulo

x == y          // Equality
x != y          // Inequality
x < y           // Less than
x > y           // Greater than
x <= y          // Less than or equal
x >= y          // Greater than or equal

a && b          // Logical AND
a || b          // Logical OR
```

## Unary Expressions

```reluxscript
!flag           // Logical NOT
-num            // Negation
*ptr            // Dereference
&value          // Reference
&mut value      // Mutable reference
```

## Member Access

```reluxscript
obj.property    // Dot notation
obj?.property   // Optional chaining
arr[index]      // Bracket notation
```

## Function Calls

```reluxscript
foo()                    // No arguments
bar(arg1, arg2)         // With arguments
obj.method(arg)         // Method call
fn<T>(arg)              // Generic call
```

## Match Expression

```reluxscript
match value {
    Pattern1 => result1,
    Pattern2 => result2,
    _ => default
}
```

## If Expression

```reluxscript
let value = if condition {
    42
} else {
    0
};
```

## Block Expression

```reluxscript
let result = {
    let x = compute();
    x * 2
};
```

## String Formatting

```reluxscript
format!("Hello, {}!", name)
format!("Count: {}", count)
```

## Collection Construction

```reluxscript
vec![1, 2, 3]                    // Vector
HashMap::new()                    // HashMap
HashSet::new()                    // HashSet
```

## Struct Construction

```reluxscript
Point { x: 10, y: 20 }
Person { name, age: 30 }         // Shorthand
```

See [Syntax Overview](/v0.1/language/syntax) for more details.
