# Manual Fixes for Generated Code

## 1. Line 316: Box pattern syntax (experimental)
**Current:**
```rust
Some(ExprOrSpread { expr: box Expr::Ident(ref elem), spread: None }) => {
```

**Fixed:**
```rust
Some(ExprOrSpread { expr, spread: None }) if matches!(expr.as_ref(), Expr::Ident(_)) => {
    if let Expr::Ident(elem) = expr.as_ref() {
```

Or simpler:
```rust
Some(ref s) if s.spread.is_none() => {
    if let Expr::Ident(elem) = s.expr.as_ref() {
```

## 2. Line 107: arg.clone() type mismatch
Need to check what arg is and what render_body expects

## 3. Line 228: PropName.sym
PropName is an enum, not a struct with .sym field
Need to match on it:
```rust
match &prop.key {
    PropName::Ident(ident) => ident.sym.to_string(),
    _ => String::new()
}
```

## 4. Lines 284, 340: binding.as_ref().unwrap()
binding is &Pat, calling as_ref() on a reference doesn't work
Should be: binding (just use it directly, or &**binding if needed)

## 5. Lines 296, 301, 343: &call.args[0]
args[0] is ExprOrSpread, but functions expect &Expr
Need: &call.args[0].expr

## 6. Line 313: deps_arg type mismatch
Similar issue

## 7. Line 353: &*expr.value wtf8
Need different conversion, maybe:
```rust
format!("\"{}\"", expr.value.as_ref())
```
or
```rust
format!("\"{}\"", &**expr.value)
```
