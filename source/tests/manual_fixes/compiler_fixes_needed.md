# Compiler Fixes Needed for minimact_full.lux

Based on manual fixes that made the code compile, here are the changes the compiler needs to make:

## 1. Box Pattern Syntax (Line 316)
**Problem:** Emitting `box Expr::Ident(ref elem)` which is experimental syntax
**Current Output:**
```rust
Some(ExprOrSpread { expr: box Expr::Ident(ref elem), spread: None }) => {
```
**Should Generate:**
```rust
Some(ref s) if s.spread.is_none() => {
    if let Expr::Ident(elem) = s.expr.as_ref() {
```

## 2. PropName Field Access (Line 228)
**Problem:** Accessing `.sym` directly on PropName enum
**Current Output:**
```rust
let prop_name = prop.key.sym.to_string().clone();
```
**Should Generate:**
```rust
let prop_name = match &prop.key {
    PropName::Ident(ident) => ident.sym.to_string(),
    _ => String::new()
};
```

## 3. Pattern Binding .as_ref() (Lines 284, 340)
**Problem:** Calling `.as_ref().unwrap()` on a reference `&Pat`
**Current Output:**
```rust
let binding = binding.as_ref().unwrap();
```
**Should Generate:**
```rust
let binding = binding;
```
(Just use the binding directly since it's already a reference)

## 4. ExprOrSpread vs Expr (Lines 296, 301, 343)
**Problem:** Passing `&call.args[0]` which is `&ExprOrSpread`, but function expects `&Expr`
**Current Output:**
```rust
Self::expr_to_csharp(&call.args[0])
```
**Should Generate:**
```rust
Self::expr_to_csharp(&call.args[0].expr)
```

## 5. ExprOrSpread Array Element (Line 311-315)
**Problem:** `call.args[1]` is `ExprOrSpread`, need to access `.expr` field
**Current Output:**
```rust
let deps_arg = &call.args[1];
match deps_arg {
    Expr::Array(deps_arg) => {
```
**Should Generate:**
```rust
let deps_arg = &call.args[1].expr;
match deps_arg.as_ref() {
    Expr::Array(deps_arg) => {
```

## 6. Atom Display in format! (Line 358)
**Problem:** Using `&*expr.value` which dereferences to Wtf8, not &str
**Current Output:**
```rust
return format!("\"{}\"", &*expr.value);
```
**Should Generate:**
```rust
return format!("\"{}\"", expr.value.as_ref());
```

## Summary of Root Causes:

1. **Box patterns**: Need to avoid `box` syntax, use if-let inside match arm instead
2. **Enum field access**: PropName is an enum, needs pattern matching not direct field access
3. **Reference unwrapping**: Don't call `.as_ref()` on something that's already `&T`
4. **ExprOrSpread**: Need to access `.expr` field when indexing into args array
5. **Atom to &str**: Use `.as_ref()` not `&*` for Atom types in Display context

