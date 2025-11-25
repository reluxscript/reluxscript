# Types

ReluxScript has a static type system that compiles to both JavaScript and Rust types.

## Primitive Types

### Str

String type that maps to both platforms:

```reluxscript
let name: Str = "hello";
```

**Compiles to:**
- Babel: `string`
- SWC: `String`

### Numbers

```reluxscript
let count: i32 = 42;        // 32-bit integer
let ratio: f64 = 3.14;      // 64-bit float
```

**Compiles to:**
- Babel: `number` (both)
- SWC: `i32` and `f64`

### Boolean

```reluxscript
let flag: bool = true;
```

**Compiles to:**
- Babel: `boolean`
- SWC: `bool`

### Unit Type

```reluxscript
let empty: () = ();
```

**Compiles to:**
- Babel: `undefined`
- SWC: `()`

## Container Types

### Vec&lt;T&gt;

Dynamic array/vector:

```reluxscript
let items: Vec<Str> = vec!["a", "b", "c"];
items.push("d");
```

**Compiles to:**
- Babel: `Array`
- SWC: `Vec<T>`

### Option&lt;T&gt;

Optional value (may or may not exist):

```reluxscript
let maybe: Option<Str> = Some("value");
let nothing: Option<Str> = None;

if let Some(value) = maybe {
    println!("Got: {}", value);
}
```

**Compiles to:**
- Babel: `T | null`
- SWC: `Option<T>`

### Result&lt;T, E&gt;

Result type for error handling:

```reluxscript
fn parse_value(s: &Str) -> Result<i32, Str> {
    if s.is_empty() {
        return Err("Empty string");
    }
    Ok(42)
}
```

**Compiles to:**
- Babel: `{ ok: boolean, value?: T, error?: E }`
- SWC: `Result<T, E>`

### HashMap&lt;K, V&gt;

Key-value map:

```reluxscript
let mut map: HashMap<Str, i32> = HashMap::new();
map.insert("key", 42);
```

**Compiles to:**
- Babel: `Map` or `Object`
- SWC: `HashMap<K, V>`

### HashSet&lt;T&gt;

Set of unique values:

```reluxscript
let mut set: HashSet<Str> = HashSet::new();
set.insert("item");
```

**Compiles to:**
- Babel: `Set`
- SWC: `HashSet<T>`

## Reference Types

### Immutable Reference

```reluxscript
fn read_node(node: &CallExpression) {
    // Can read, cannot modify
    let name = node.callee.name.clone();
}
```

**Compiles to:**
- Babel: Regular value (no concept of references)
- SWC: `&T`

### Mutable Reference

```reluxscript
fn transform_node(node: &mut CallExpression) {
    // Can read and modify
    *node = Statement::empty();
}
```

**Compiles to:**
- Babel: Regular value
- SWC: `&mut T`

## Tuple Types

Fixed-size collections of heterogeneous types:

```reluxscript
let pair: (Str, i32) = ("answer", 42);
let triple: (bool, f64, Str) = (true, 3.14, "pi");

// Destructuring
let (name, value) = pair;
```

**Compiles to:**
- Babel: Arrays `[T1, T2]`
- SWC: Tuples `(T1, T2)`

## AST Node Types

### Expression Types

```reluxscript
Identifier
CallExpression
MemberExpression
BinaryExpression
UnaryExpression
ArrowFunctionExpression
ObjectExpression
ArrayExpression
```

### Statement Types

```reluxscript
FunctionDeclaration
VariableDeclaration
IfStatement
ForStatement
WhileStatement
ReturnStatement
BlockStatement
ExpressionStatement
```

### JSX Types

```reluxscript
JSXElement
JSXAttribute
JSXExpressionContainer
JSXText
```

### TypeScript Types

```reluxscript
TSInterfaceDeclaration
TSTypeAnnotation
TSTypeReference
```

See [Node Types](/v0.1/language/node-types) for complete reference.

## Type Inference

ReluxScript infers types when possible:

```reluxscript
let name = "hello";          // Inferred as Str
let count = 42;              // Inferred as i32
let items = vec![1, 2, 3];  // Inferred as Vec<i32>
```

## Type Conversion

### Explicit Conversion

```reluxscript
let num: i32 = 42;
let text: Str = format!("{}", num);  // Convert to string
```

### The .into() Method

```reluxscript
let s: Str = "hello".into();
```

**Note:** `.into()` is a no-op in Babel (type conversions are implicit in JavaScript).

## Generic Functions

Functions can have type parameters:

```reluxscript
fn identity<T>(value: T) -> T {
    value
}

fn map_items<F>(items: Vec<Str>, f: F) -> Vec<Str>
where
    F: Fn(Str) -> Str
{
    items.iter().map(|s| f(s.clone())).collect()
}
```

**Compiles to:**
- Babel: Generics stripped (no type parameters)
- SWC: Generics preserved

## Type Aliases

Create aliases for complex types:

```reluxscript
type NodeVisitor = fn(&mut Node, &Context);
type Result<T> = Result<T, Str>;
```

## Next Steps

- [Expressions](/v0.1/language/expressions)
- [Pattern Matching](/v0.1/language/pattern-matching)
- [Node Types](/v0.1/language/node-types)
