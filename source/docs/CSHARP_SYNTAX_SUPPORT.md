# C# Syntax Support Implementation Plan

ReluxScript's target audience is C# Orleans developers writing `.lux` files to convert React to C# Grains. They are more comfortable with C# than Rust syntax, so we should support C#-style syntax in addition to the current Rust-style syntax.

This document outlines a comprehensive plan to add C#-like syntax features to ReluxScript.

---

## Priority 1: Parser-Only Changes (No Semantic Changes)

These features can be implemented entirely in the parser by desugaring to existing AST nodes.

### 1.1 `is` Pattern Matching Syntax

**Current (Rust-style):**
```lux
if let Expression::Identifier(id) = &node.callee {
    // use id
}
```

**Proposed (C#-style):**
```lux
if &node.callee is Expression::Identifier id {
    // use id
}
```

**Implementation:**

1. **Lexer Changes** (`src/lexer/mod.rs`):
   - Add `Is` keyword token

2. **Parser Changes** (`src/parser/parser.rs`):
   - In `parse_if_stmt()`, after parsing the condition expression, check for `is` keyword
   - If found, parse: `<expr> is <pattern> <binding>`
   - Desugar to `IfLet` with:
     - `scrutinee`: the expression before `is`
     - `pattern`: the pattern after `is`
     - `binding`: optional identifier after the pattern

3. **Grammar:**
   ```
   is_pattern_expr := expr "is" pattern_name binding_name?
   pattern_name := identifier ("::" identifier)*
   binding_name := identifier
   ```

4. **Desugaring Examples:**
   ```lux
   // Input
   if expr is Pattern binding { body }

   // Desugars to
   if let Pattern(binding) = expr { body }
   ```

   ```lux
   // With nested patterns
   if node is Expression::MemberExpression { object } {
       // use object
   }

   // Desugars to
   if let Expression::MemberExpression(mem) = node {
       let object = &mem.object;
       // use object
   }
   ```

**Files to Modify:**
- `src/lexer/mod.rs` - Add `Is` token
- `src/parser/parser.rs` - Add `is` pattern parsing in conditionals
- `src/parser/ast.rs` - No changes needed (reuses existing `IfLet`)

**Estimated Effort:** 2-3 hours

---

### 1.2 String Interpolation `$"..."`

**Current:**
```lux
format!("Component {} has {} hooks", name, count)
```

**Proposed (C#-style):**
```lux
$"Component {name} has {count} hooks"
```

**Implementation:**

1. **Lexer Changes** (`src/lexer/mod.rs`):
   - Add `InterpolatedString` token type
   - When encountering `$"`, enter interpolated string mode
   - Parse segments: literal parts and `{expr}` interpolations
   - Return structured token with segments

2. **Parser Changes** (`src/parser/parser.rs`):
   - Handle `InterpolatedString` token
   - Parse each `{...}` segment as an expression
   - Desugar to `format!()` macro call

3. **AST Representation:**
   ```rust
   enum Expr {
       // ... existing variants
       InterpolatedString {
           segments: Vec<InterpolatedSegment>,
       },
   }

   enum InterpolatedSegment {
       Literal(String),
       Expr(Box<Expr>),
   }
   ```

4. **Desugaring:**
   ```lux
   // Input
   $"Hello {name}, you have {count} items"

   // Desugars to
   format!("Hello {}, you have {} items", name, count)
   ```

5. **Edge Cases:**
   - Escaped braces: `$"Use {{braces}}"` → `"Use {braces}"`
   - Nested expressions: `$"Value: {obj.get_value()}"` → `format!("Value: {}", obj.get_value())`
   - Format specifiers (future): `$"Price: {price:0.2}"` → `format!("Price: {:.2}", price)`

**Files to Modify:**
- `src/lexer/mod.rs` - Add interpolated string lexing
- `src/parser/parser.rs` - Parse interpolated strings
- `src/parser/ast.rs` - Add `InterpolatedString` variant (or desugar immediately)

**Estimated Effort:** 3-4 hours

---

### 1.3 Property Initialization Shorthand

**Current:**
```lux
let name = node.id.name.clone();
let stats = ComponentStats {
    name: name,
    hooks: vec![],
};
```

**Proposed:**
```lux
let name = node.id.name.clone();
let stats = ComponentStats {
    name,  // shorthand for name: name
    hooks: vec![],
};
```

**Implementation:**

1. **Parser Changes** (`src/parser/parser.rs`):
   - In `parse_struct_literal()`, when parsing fields:
     - If field is just an identifier (no `:` follows), treat as shorthand
     - Expand `name` to `name: name`

2. **No AST Changes Needed:**
   - Desugar during parsing to existing `StructField { name, value }` representation

**Files to Modify:**
- `src/parser/parser.rs` - Modify struct literal field parsing

**Estimated Effort:** 30 minutes

---

### 1.4 Arrow Function Syntax `=>`

**Current:**
```lux
components.map(|c| c.name.clone())
```

**Proposed:**
```lux
components.map(c => c.name.clone())
```

**Implementation:**

1. **Lexer Changes** (`src/lexer/mod.rs`):
   - Ensure `=>` is lexed as `FatArrow` token (may already exist)

2. **Parser Changes** (`src/parser/parser.rs`):
   - In closure/lambda parsing, accept both syntaxes:
     - `|params| body` (Rust-style)
     - `params => body` (C#-style)
   - For single parameter: `x => expr`
   - For multiple parameters: `(x, y) => expr`

3. **Grammar:**
   ```
   closure := rust_closure | csharp_closure
   rust_closure := "|" param_list "|" expr
   csharp_closure := param_or_parens "=>" expr
   param_or_parens := identifier | "(" param_list ")"
   ```

4. **Desugaring:**
   ```lux
   // Input
   x => x + 1

   // Desugars to existing AST
   Closure { params: [x], body: x + 1 }
   ```

**Files to Modify:**
- `src/lexer/mod.rs` - Ensure `=>` token exists
- `src/parser/parser.rs` - Add C#-style closure parsing

**Estimated Effort:** 1-2 hours

---

## Priority 2: Semantic-Aware Changes

These features require type information to implement correctly.

### 2.1 Null-Conditional Operator `?.`

**Current:**
```lux
if let Some(init) = &node.init {
    if let Some(id) = init.as_identifier() {
        let name = id.name.clone();
    }
}
```

**Proposed:**
```lux
let name = node.init?.as_identifier()?.name.clone();
```

**Implementation:**

1. **Lexer Changes:**
   - Add `QuestionDot` token for `?.`

2. **Parser Changes:**
   - Parse `?.` as a special member access operator
   - Create new AST node: `NullConditionalAccess`

3. **AST Addition:**
   ```rust
   enum Expr {
       NullConditionalAccess {
           object: Box<Expr>,
           property: String,
       },
       NullConditionalCall {
           object: Box<Expr>,
           method: String,
           args: Vec<Expr>,
       },
   }
   ```

4. **Semantic Analysis:**
   - Type checker must verify `object` is `Option<T>`
   - Propagate `Option` type through the chain

5. **Codegen Desugaring:**
   ```lux
   // Input
   a?.b?.c

   // Desugars to Rust
   a.as_ref().and_then(|x| x.b.as_ref()).map(|x| x.c)

   // Or with method calls
   a?.method()?.field

   // Desugars to
   a.as_ref().and_then(|x| x.method()).map(|x| x.field)
   ```

6. **Chaining Semantics:**
   - `a?.b` returns `Option<T>` where `T` is type of `b`
   - `a?.b?.c` chains the Options
   - `a?.b.c` - if `b` is not Option, access `c` directly within the map

**Files to Modify:**
- `src/lexer/mod.rs` - Add `?.` token
- `src/parser/parser.rs` - Parse null-conditional access
- `src/parser/ast.rs` - Add new AST variants
- `src/semantic/type_checker.rs` - Handle Option propagation
- `src/codegen/swc_decorator.rs` - Desugar to and_then/map chains
- `src/codegen/swc_emit.rs` - Emit the desugared code

**Estimated Effort:** 6-8 hours

---

### 2.2 Null-Coalescing Operator `??`

**Current:**
```lux
let name = node.name.clone().unwrap_or("default".into());
```

**Proposed:**
```lux
let name = node.name ?? "default";
```

**Implementation:**

1. **Lexer Changes:**
   - Add `DoubleQuestion` token for `??`

2. **Parser Changes:**
   - Parse `??` as binary operator with appropriate precedence
   - Lower precedence than most operators, higher than assignment

3. **AST Addition:**
   ```rust
   enum BinaryOp {
       // ... existing ops
       NullCoalesce,  // ??
   }
   ```

4. **Semantic Analysis:**
   - Left operand must be `Option<T>`
   - Right operand must be `T` (or convertible to `T`)
   - Result type is `T`

5. **Codegen:**
   ```lux
   // Input
   a ?? b

   // Desugars to
   a.unwrap_or(b)

   // Or for lazy evaluation
   a.unwrap_or_else(|| b)
   ```

6. **Chaining:**
   ```lux
   // Input
   a ?? b ?? c

   // Desugars to
   a.or(b).unwrap_or(c)
   ```

**Files to Modify:**
- `src/lexer/mod.rs` - Add `??` token
- `src/parser/parser.rs` - Add operator precedence
- `src/parser/ast.rs` - Add `NullCoalesce` binary op
- `src/semantic/type_checker.rs` - Type check Option<T> ?? T
- `src/codegen/swc_emit.rs` - Emit unwrap_or

**Estimated Effort:** 3-4 hours

---

### 2.3 Null-Coalescing Assignment `??=`

**Current:**
```lux
if node.name.is_none() {
    node.name = Some("default".into());
}
```

**Proposed:**
```lux
node.name ??= "default";
```

**Implementation:**

1. **Lexer:** Add `DoubleQuestionEquals` token
2. **Parser:** Parse as assignment operator
3. **Semantic:** Left must be `Option<T>`, right must be `T`
4. **Codegen:**
   ```lux
   // Input
   a ??= b

   // Desugars to
   if a.is_none() { a = Some(b); }

   // Or
   a = a.or(Some(b));
   ```

**Estimated Effort:** 2 hours (after `??` is implemented)

---

## Priority 3: Method Aliases (LINQ-style)

These are simple method name mappings in the codegen layer.

### 3.1 Collection Method Aliases

| C# (LINQ) | Rust | Notes |
|-----------|------|-------|
| `.Select()` | `.map()` | Transform elements |
| `.Where()` | `.filter()` | Filter elements |
| `.Any()` | `.any()` | Check if any match |
| `.All()` | `.all()` | Check if all match |
| `.First()` | `.next()` | Get first element |
| `.FirstOrDefault()` | `.next().unwrap_or_default()` | First or default |
| `.ToList()` | `.collect::<Vec<_>>()` | Collect to Vec |
| `.ToArray()` | `.collect::<Vec<_>>()` | Same as ToList for our purposes |
| `.Count()` | `.count()` | Count elements |
| `.OrderBy()` | `.sorted_by_key()` | Sort by key |
| `.Distinct()` | `.dedup()` | Remove duplicates |
| `.Take()` | `.take()` | Take N elements |
| `.Skip()` | `.skip()` | Skip N elements |
| `.Aggregate()` | `.fold()` | Reduce/fold |
| `.Contains()` | `.contains()` | Check membership |

**Implementation:**

1. **Rewriter Layer** (`src/codegen/swc_rewriter.rs`):
   - When encountering method call, check if it's a LINQ alias
   - Transform method name and adjust arguments if needed

2. **Special Cases:**
   - `ToList()` needs to add type annotation: `.collect::<Vec<_>>()`
   - `FirstOrDefault()` chains two methods
   - `OrderBy(x => x.field)` needs different Rust syntax

**Files to Modify:**
- `src/codegen/swc_rewriter.rs` - Add method aliasing

**Estimated Effort:** 4-5 hours

---

## Priority 4: Advanced Pattern Matching

### 4.1 Property Patterns (C# 8.0+)

**Proposed:**
```lux
if node is Expression::MemberExpression { object: Expression::Identifier { name: "console" } } {
    // Found console.*
}
```

**Implementation:**

This is significantly more complex as it requires:
1. Parsing nested property patterns
2. Generating nested if-let chains
3. Handling optional properties

**Desugaring:**
```lux
// Input
if node is Expression::MemberExpression { object: Expression::Identifier { name } } {
    // use name
}

// Desugars to
if let Expression::MemberExpression(mem) = node {
    if let Expression::Identifier(id) = &mem.object {
        let name = &id.name;
        // use name
    }
}
```

**Estimated Effort:** 8-10 hours

---

### 4.2 Switch Expression (C# 8.0+)

**Proposed:**
```lux
let result = node switch {
    Expression::Identifier { name } => $"Found identifier: {name}",
    Expression::Literal(lit) => "Found literal",
    _ => "Unknown"
};
```

**Current Equivalent:**
```lux
let result = match node {
    Expression::Identifier(id) => format!("Found identifier: {}", id.name),
    Expression::Literal(lit) => "Found literal".into(),
    _ => "Unknown".into(),
};
```

**Implementation:**
- Parse `switch` as alternate syntax for `match`
- Same semantics, just different keyword

**Estimated Effort:** 1-2 hours

---

## Priority 5: Type Syntax Aliases

### 5.1 C# Type Names

| C# | ReluxScript/Rust |
|----|------------------|
| `string` | `Str` or `String` |
| `int` | `i32` |
| `long` | `i64` |
| `float` | `f32` |
| `double` | `f64` |
| `bool` | `bool` |
| `object` | `Any` or `dyn Any` |
| `List<T>` | `Vec<T>` |
| `Dictionary<K,V>` | `HashMap<K,V>` |
| `HashSet<T>` | `HashSet<T>` |

**Implementation:**
- Add type aliases in the parser or semantic layer
- Map C# type names to Rust equivalents

**Estimated Effort:** 2 hours

---

## Implementation Roadmap

### Phase 1: Quick Wins (Week 1)
1. Property initialization shorthand (30 min)
2. Arrow function `=>` syntax (1-2 hours)
3. `is` pattern matching (2-3 hours)
4. String interpolation `$""` (3-4 hours)

**Total: ~8 hours**

### Phase 2: Null Safety (Week 2)
1. Null-coalescing `??` (3-4 hours)
2. Null-conditional `?.` (6-8 hours)
3. Null-coalescing assignment `??=` (2 hours)

**Total: ~12 hours**

### Phase 3: LINQ-style Methods (Week 3)
1. Basic method aliases (Select, Where, Any, All) (2 hours)
2. Collection methods (ToList, First, Count) (2 hours)
3. Advanced methods (OrderBy, Aggregate) (2 hours)

**Total: ~6 hours**

### Phase 4: Advanced Features (Week 4+)
1. Property patterns (8-10 hours)
2. Switch expression (1-2 hours)
3. Type aliases (2 hours)

**Total: ~12 hours**

---

## Testing Strategy

For each feature:

1. **Parser Tests:**
   - Valid syntax parses correctly
   - Invalid syntax produces helpful error messages
   - Edge cases handled

2. **Semantic Tests:**
   - Type checking works correctly
   - Error messages are clear

3. **Codegen Tests:**
   - Generated Rust code compiles
   - Generated code behaves correctly

4. **Integration Tests:**
   - Full `.lux` files using new syntax
   - Verify output matches expected Rust/SWC code

---

## Backward Compatibility

All new syntax is **additive** - existing Rust-style syntax continues to work:

- `if let` still works alongside `is`
- `|x| expr` still works alongside `x => expr`
- `format!()` still works alongside `$""`
- `.map()` still works alongside `.Select()`

Users can mix and match based on preference.

---

## Documentation Updates

After implementation:

1. Update language reference with new syntax
2. Add migration guide for C# developers
3. Provide side-by-side examples (C# → ReluxScript)
4. Update error messages to suggest C# syntax where appropriate
