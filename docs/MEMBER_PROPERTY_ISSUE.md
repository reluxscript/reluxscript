# Member Property Access Issue

**Status:** 4 remaining errors in kitchen_sink_plugin.lux
**Date:** 2025-01-29
**Context:** Down from 28 errors to 4 (86% fixed)

## Problem Summary

When accessing `member.property` in ReluxScript code, the generated SWC code emits `member.prop.clone()` which returns a `MemberProp` enum, but the code expects a `String`. This causes type mismatch errors.

## Root Cause Analysis

### The Type Mismatch

**ReluxScript (Unified AST):**
```reluxscript
// In ReluxScript, member.property is just a String
fn extract_member_call(call: &CallExpression) -> Option<MemberInfo> {
    if let Expression::MemberExpression(member) = &call.callee {
        if let Expression::Identifier(obj) = &member.object {
            return Some(MemberInfo {
                object: obj.name.clone(),      // String
                property: member.property.clone(),  // String
            });
        }
    }
    None
}
```

**SWC Generated Code (Current - INCORRECT):**
```rust
// In SWC, member.prop is a MemberProp enum, not a String
fn extract_member_call(call: &CallExpr) -> Option<MemberInfo> {
    if let Callee::Expr(__callee_expr) = &&call.callee.as_expr().unwrap() {
        if let Expr::Member(member) = __callee_expr.as_ref() {
            if let Expr::Ident(obj) = &member.obj {
                return Some(MemberInfo {
                    object: obj.sym.to_string(),      // ✅ String
                    property: member.prop.clone()     // ❌ MemberProp (not String)
                });
            }
        }
    }
    None
}
```

**Error:**
```
error[E0308]: mismatched types
   --> lib.rs:182:85
    |
182 |     return Some(MemberInfo { object: obj.sym.to_string(), property: member.prop.clone() });
    |                                                                      ^^^^^^^^^^^^^^^^^^^
    |                                                                      expected `String`, found `MemberProp`
```

### SWC's MemberProp Enum

From [swc_ecma_ast source](https://github.com/swc-project/swc/blob/main/crates/swc_ecma_ast/src/expr.rs):

```rust
#[ast_node]
#[derive(Eq, Hash, Is, EqIgnoreSpan)]
pub enum MemberProp {
    #[tag("Identifier")]
    Ident(IdentName),           // obj.property (static property access)

    #[tag("PrivateName")]
    PrivateName(PrivateName),   // obj.#private (private field access)

    #[tag("Computed")]
    Computed(ComputedPropName), // obj[expr] (dynamic property access)
}

impl MemberProp {
    pub fn is_ident_with(&self, sym: &str) -> bool {
        matches!(self, MemberProp::Ident(i) if i.sym == sym)
    }
}
```

The enum has **no built-in string conversion** - you must pattern match to extract the string.

### Correct SWC Pattern

**What the code SHOULD generate:**
```rust
fn extract_member_call(call: &CallExpr) -> Option<MemberInfo> {
    if let Callee::Expr(__callee_expr) = &call.callee.as_expr().unwrap() {  // Fix ref depth
        if let Expr::Member(member) = __callee_expr.as_ref() {
            if let Expr::Ident(obj) = &*member.obj {  // Fix box deref
                // Extract string from MemberProp enum
                let property_str = match &member.prop {
                    MemberProp::Ident(id) => id.sym.to_string(),
                    MemberProp::Computed(ComputedPropName { expr, .. }) => {
                        // For computed properties, we'd need to generate code or use a placeholder
                        "[computed]".to_string()
                    }
                    MemberProp::PrivateName(name) => {
                        format!("#{}", name.name.to_string())
                    }
                };

                return Some(MemberInfo {
                    object: obj.sym.to_string(),
                    property: property_str
                });
            }
        }
    }
    None
}
```

## Error Locations

All 4 errors in `kitchen_sink_plugin.lux` compiled to `dist/lib.rs`:

| Line | Source Function | Issue | ReluxScript Source |
|------|----------------|-------|-------------------|
| 171  | `get_callee_name` | `member.prop.clone()` → String | Line 76 |
| 179  | `extract_member_call` | Pattern `Callee::Expr` ref depth | Line 83 |
| 181  | `extract_member_call` | Pattern `&member.obj` needs `&*` | Line 84 |
| 182  | `extract_member_call` | `member.prop.clone()` → String | Line 87 |

## Solution Strategy

The fix needs to happen in the **decorator** and **rewriter**, keeping the **emitter dumb**.

### Option 1: Rewriter Transformation (Recommended)

The rewriter should detect when `member.property` (which becomes `member.prop`) is being used as a value, and transform it:

**Transform:**
```rust
// Input (decorated AST):
member.prop.clone()

// Output (rewritten AST):
match &member.prop {
    MemberProp::Ident(id) => id.sym.to_string(),
    MemberProp::Computed(_) => "[computed]".to_string(),
    MemberProp::PrivateName(name) => format!("#{}", name.name.to_string()),
}
```

This transformation should happen in `swc_rewriter.rs` as a new rewrite phase:
- Detect member field access on `prop` field of `MemberExpr`
- When the result is used as a `String`, inject the match expression
- Preserve the expression metadata

### Option 2: Decorator Metadata (Alternative)

The decorator could add metadata indicating that `member.prop` needs string conversion:

```rust
SwcFieldMetadata {
    swc_field_name: "prop",
    accessor: FieldAccessor::EnumToString {
        enum_type: "MemberProp",
        variants: vec![
            ("Ident", "id.sym.to_string()"),
            ("Computed", "\"[computed]\".to_string()"),
            ("PrivateName", "format!(\"#{}\", name.name.to_string())"),
        ]
    }
}
```

Then the emitter would emit the match expression based on this metadata.

**Trade-off:** This makes the emitter less dumb (it needs to know how to emit match expressions for enum conversion).

## Implementation Plan

### Phase 1: Add Rewriter Transformation

1. **Add detection in `swc_rewriter.rs`:**
   ```rust
   fn apply_member_prop_string_conversion(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
       // Detect: member.prop.clone() or member.prop access
       // When field_metadata.swc_field_name == "prop"
       // And parent is MemberExpr
       // And expected type is String
   }
   ```

2. **Create match expression AST:**
   ```rust
   DecoratedExpr {
       kind: DecoratedExprKind::Match(Box::new(DecoratedMatchExpr {
           expr: /* &member.prop */,
           arms: vec![
               // MemberProp::Ident(id) => id.sym.to_string()
               // MemberProp::Computed(_) => "[computed]".to_string()
               // MemberProp::PrivateName(name) => format!("#{}", name.name)
           ]
       }))
   }
   ```

3. **Add to rewriter pipeline:**
   ```rust
   fn rewrite_expr(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
       let expr = self.rewrite_expr_children(expr);
       let expr = self.apply_field_replacements(expr);
       let expr = self.apply_context_remove(expr);
       let expr = self.apply_codegen_helpers(expr);
       let expr = self.apply_member_prop_conversion(expr);  // NEW
       let expr = self.apply_matches_expansion(expr);
       let expr = self.apply_iterator_methods(expr);
       expr
   }
   ```

### Phase 2: Fix Pattern Matching Issues

The other errors (179, 181) are related to pattern matching on boxed/referenced types:

**Issue at line 179:**
```rust
// Current (WRONG):
if let Callee::Expr(__callee_expr) = &&call.callee.as_expr().unwrap() {

// Should be:
if let Callee::Expr(__callee_expr) = &call.callee.as_expr().unwrap() {
```

**Issue at line 181:**
```rust
// Current (WRONG):
if let Expr::Ident(obj) = &member.obj {

// Should be (member.obj is Box<Expr>):
if let Expr::Ident(obj) = &*member.obj {
```

These should be handled by the decorator's pattern metadata generation.

## Testing

After implementing, test with:
```bash
cd source/examples
../target/release/relux build kitchen_sink_plugin.lux
```

Expected: 0 compilation errors (down from 4)

## References

- [SWC MemberProp enum source](https://github.com/swc-project/swc/blob/main/crates/swc_ecma_ast/src/expr.rs)
- [MemberProp Rust docs](https://rustdoc.swc.rs/swc_ecma_ast/enum.MemberProp.html)
- ReluxScript field mapping: `source/src/mapping/fields.rs:141-147`
- Current rewriter: `source/src/codegen/swc_rewriter.rs`
- Current decorator: `source/src/codegen/swc_decorator.rs`

## Notes

- This is a known limitation of the "vector alignment" principle - SWC's AST is more complex than Babel's
- Babel's `MemberExpression.property` is just an `Identifier` node with a `name` string
- SWC's `MemberExpr.prop` is an enum to handle different property access patterns
- The unified ReluxScript AST follows Babel's simpler model (just a string)
- The compiler must bridge this gap during SWC codegen
