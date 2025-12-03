# Unsupported Features from Minimact Lux Files

This document lists features used in `minimact/reluxscript-plugin-minimact/*.lux` files that the main branch currently does not support.

---

## 1. Module files (standalone `.lux` with `pub fn` functions)

All files in `minimact/reluxscript-plugin-minimact/` are **modules**, not writers/plugins:
- `type_conversion.lux` - standalone module with `pub fn ts_type_to_csharp_type(...)`
- `helpers.lux` - standalone module with `pub fn escape_csharp_string(...)`
- `classification.lux` - standalone module with `pub fn classify_node(...)`

Main outputs: `// Undecorated top-level declaration (not yet supported)`

---

## 2. TSType matching with keyword variants

`type_conversion.lux:17-66`:
```rust
match ts_type {
    TSType::TSStringKeyword => "string",
    TSType::TSNumberKeyword => "double",
    TSType::TSBooleanKeyword => "bool",
    TSType::TSAnyKeyword => "dynamic",
    TSType::TSArrayType(ref array_type) => {...}
    TSType::TSTypeLiteral(_) => "dynamic",
    TSType::TSTypeReference(ref type_ref) => {...}
}
```

`strip_pattern_binding` strips the nested struct patterns, so these become `TsType::TsKeywordType` without the discriminator.

---

## 3. TSEntityName matching

`type_conversion.lux:43`:
```rust
if let TSEntityName::Identifier(ref type_name) = type_ref.type_name {
```

---

## 4. `pub use` imports (re-exports)

`mod.lux` files throughout:
```rust
pub use "./build_member_path.lux" {
    build_member_path,
    build_optional_member_path
};
```

---

## 5. `use` imports from relative paths

`literals.lux:7`:
```rust
use "../../utils/helpers.lux" { escape_csharp_string };
```

---

## 6. Struct definitions in modules

`classification.lux:18-21`:
```rust
pub struct Dependency {
    pub name: Str,
    pub dep_type: Str,
}
```

---

## 7. HashSet/generic collection types

`classification.lux:29`:
```rust
pub fn classify_node(deps: &HashSet<Dependency>) -> Str {
```

---

## 8. Generic function parameters with trait bounds

`literals.lux:76-81`:
```rust
pub fn generate_template_literal<F>(
    node: &TemplateLiteral,
    generate_csharp_expression: F
) -> Str
where
    F: Fn(&Expression, bool) -> Str
```

---

## 9. Private `fn` (non-pub functions) in modules

`hook_analyzer.lux:119`, `type_conversion.lux:115`:
```rust
fn extract_use_state_calls(...) {...}
fn is_integer(value: f64) -> bool {...}
```

---

## 10. Complex nested pattern matching on AST nodes

`hook_analyzer.lux:121-128`:
```rust
if let Statement::VariableDeclaration(ref var_decl) = stmt {
    for declarator in &var_decl.declarations {
        if let Some(state_info) = extract_state_from_declarator(declarator) {
```

---

## 11. Multiple nested `match` expressions

`hook_analyzer.lux:223-253`:
```rust
match init {
    Expression::ArrowFunctionExpression(ref arrow) => {...}
    Expression::FunctionExpression(ref func_expr) => {...}
    _ => {}
}
```

---

## 12. Method calls on generic types

`classification.lux:43`:
```rust
types.contains("client")
```

---

## 13. String methods

`hook_analyzer.lux:421-428`:
```rust
name.starts_with("set")
name.ends_with("UI")
name.contains("handle")
```

---

## 14. User-defined enum types with data variants

`extract_binding.lux:16-21`:
```rust
pub enum Binding {
    Simple(Str),
    MethodCall(MethodCallBinding),
    Conditional(ConditionalBinding),
    OptionalChain(OptionalChainBinding),
}
```

---

## 15. Multi-pattern match arms with `|`

`operators.lux:112-117`:
```rust
let right_is_boolean_expr = matches!(
    node.right,
    Expression::BinaryExpression(_)
        | Expression::LogicalExpression(_)
        | Expression::UnaryExpression(_)
);
```

---

## 16. `matches!` macro

Used extensively:
```rust
matches!(init, Expression::ArrowFunctionExpression(_))
matches!(expr, Expression::JSXElement(_) | Expression::JSXFragment(_))
```

---

## 17. Type casts (`as`)

`hex_path.lux:109`, `hook_analyzer.lux:345`:
```rust
self.parse_path(path).len() as i32
index as i32
indent as usize
```

---

## 18. Iterator methods: `.iter()`, `.map()`, `.filter()`, `.any()`, `.collect()`

`typescript_to_rust.lux:50-66`:
```rust
let declarations: Vec<Str> = var_decl.declarations.iter()
    .map(|decl| { ... })
    .collect();
```

---

## 19. Closure syntax

`csharp_file.lux:20`:
```rust
components.iter().any(|c| uses_plugins(c))
```

---

## 20. `Result<T, E>` return types with `Ok()` and `Err()`

`style_converter.lux:89`, `typescript_to_c_sharp.lux:21`:
```rust
pub fn convert(...) -> Result<Str, Str> {
    Ok(css_properties.join("; "))
}
return Err("Expected arrow or function expression".into())
```

---

## 21. `vec![]` macro for vector literals

Used extensively:
```rust
props: vec![],
use_state: vec![],
let mut errors = vec![];
```

---

## 22. Accessing `.extra_data` map on AST nodes

`literals.lux:60`:
```rust
node.extra_data.get("__minimactPath")
```

---

## 23. String methods: `.split()`, `.chars()`, `.repeat()`

`hex_path.lux:96`:
```rust
path.split(".").map(|s| s.to_string()).collect()
"    ".repeat(indent as usize)
s.chars().next().unwrap().to_uppercase()
```

---

## 24. Collection methods: `.push()`, `.len()`, `.is_empty()`, `.first()`, `.get()`

Used extensively across all files.

---

## 25. `.min()` / `.max()` on numbers

`hex_path.lux:172`:
```rust
let min_len = segments1.len().min(segments2.len());
```

---

## 26. Range slice syntax

`rust_task.lux:162`:
```rust
let element_type = &ts_type[..ts_type.len() - 2];
let inner_type = &ts_type[5..ts_type.len() - 1];
segments1[0..common_length].join(".")
```

---

## 27. `usize::from_str_radix`

`path_assignment.lux:277`:
```rust
if let Ok(hex_value) = usize::from_str_radix(last_segment, 16) {
```

---

## 28. Nested `if let` with complex patterns

`loop_template.lux:50-55`:
```rust
match callback_arg {
    CallExpressionArgument::Expression(Expression::ArrowFunctionExpression(ref arrow)) => {
        Function::ArrowFunctionExpression(arrow.clone())
    }
```

---

## 29. Multi-variant `match` on function types

`loop_template.lux:98-102`:
```rust
let params = match callback {
    Function::ArrowFunctionExpression(ref arrow) => &arrow.params,
    Function::FunctionExpression(ref func) => &func.params,
    _ => return ("item".to_string(), None),
};
```

---

## 30. `unwrap_or()` / `unwrap_or_default()`

`hex_path.lux:213`:
```rust
path1.split(".").last().unwrap_or("0")
```

---

## 31. `.into()` for type conversion

`typescript_to_c_sharp.lux:21`:
```rust
return Err("Expected arrow or function expression".into())
```

---

## 32. `HashMap` / `HashSet` with `.keys()`, `.contains()`, `.insert()`

`classification.lux:36`, `hook_imports.lux:375`:
```rust
types.insert(dep.dep_type.clone());
imported_hooks.keys().cloned().collect()
```
