# Bidirectional Type Inference Implementation

This document describes the implementation of bidirectional type inference in ReluxScript, which allows `vec![]` and other expressions to infer their types from context.

## Problem

When writing:
```reluxscript
struct Data {
    items: Vec<Str>,
}

let data = Data {
    items: vec![],  // Error: Vec<unknown> doesn't match Vec<Str>
};
```

The type checker couldn't infer that `vec![]` should be `Vec<Str>` because it only looked at the expression itself, not the context where it was being used.

## Solution: Expected Type Propagation

The key insight is that type information flows in two directions:
1. **Bottom-up**: Inferring types from expressions (e.g., `vec!["a", "b"]` → `Vec<Str>`)
2. **Top-down**: Propagating expected types from context (e.g., field expects `Vec<Str>` → `vec![]` is `Vec<Str>`)

### Implementation Steps

#### 1. Add Expected Type Parameter

Modified `infer_expr` to accept an optional expected type hint:

```rust
// src/semantic/type_checker.rs

/// Infer the type of an expression (backward compatible)
fn infer_expr(&mut self, expr: &Expr) -> TypeInfo {
    self.infer_expr_with_expected(expr, None)
}

/// Infer expression type with an expected type hint for bidirectional inference
fn infer_expr_with_expected(&mut self, expr: &Expr, expected: Option<&TypeInfo>) -> TypeInfo {
    match expr {
        // ... cases use `expected` where applicable
    }
}
```

#### 2. Handle VecInit with Expected Type

When `vec![]` is empty, use the expected type if available:

```rust
Expr::VecInit(vec_init) => {
    if vec_init.elements.is_empty() {
        // Use expected type if available (e.g., from struct field or variable annotation)
        if let Some(TypeInfo::Vec(inner)) = expected {
            TypeInfo::Vec(inner.clone())
        } else {
            TypeInfo::Vec(Box::new(TypeInfo::Unknown))
        }
    } else {
        // Infer from first element
        let elem_type = self.infer_expr(&vec_init.elements[0]);
        TypeInfo::Vec(Box::new(elem_type))
    }
}
```

#### 3. Pass Expected Type in Struct Initialization

When checking struct field values, pass the field's declared type as the expected type:

```rust
Expr::StructInit(init) => {
    if let Some(fields) = self.env.get_struct_fields(&init.name) {
        let fields = fields.clone();
        for (field_name, value) in &init.fields {
            // Pass expected field type for bidirectional inference
            let field_expected = fields.get(field_name);
            let value_type = self.infer_expr_with_expected(value, field_expected);

            if let Some(expected_type) = field_expected {
                if !value_type.is_assignable_to(expected_type) {
                    // Type error
                }
            }
        }
        // ...
    }
}
```

#### 4. Pass Expected Type in Variable Declarations

When a variable has a type annotation, use it as the expected type:

```rust
Stmt::Let(let_stmt) => {
    // If there's a type annotation, use it as the expected type
    let expected_type = let_stmt.ty.as_ref().map(ast_type_to_type_info);
    let init_type = self.infer_expr_with_expected(&let_stmt.init, expected_type.as_ref());

    if let Some(declared_type) = expected_type {
        if !init_type.is_assignable_to(&declared_type) {
            // Type error
        }
        self.env.define(let_stmt.name.clone(), declared_type);
    } else {
        self.env.define(let_stmt.name.clone(), init_type);
    }
}
```

## Additional Fix: If-Let Type Checking

When implementing if-let support, the type checker was incorrectly requiring the "condition" to be bool. For `if let Some(x) = expr`, the expression is being pattern-matched, not evaluated as a boolean.

```rust
Stmt::If(if_stmt) => {
    let cond_type = self.infer_expr(&if_stmt.condition);

    // For if-let, the condition is a pattern match expression, not a boolean
    // Only check for bool if there's no pattern
    if if_stmt.pattern.is_none() && !matches!(cond_type, TypeInfo::Bool | TypeInfo::Unknown) {
        self.errors.push(SemanticError::new(
            "RS003",
            format!("Condition must be bool, found {}", cond_type.display_name()),
            if_stmt.span,
        ));
    }
    // ...
}
```

## Files Modified

1. **`src/semantic/type_checker.rs`**
   - Added `infer_expr_with_expected` method
   - Updated `Expr::VecInit` to use expected type
   - Updated `Expr::StructInit` to pass field types
   - Updated `Stmt::Let` to pass type annotations
   - Updated `Stmt::If` to handle if-let patterns

2. **`src/semantic/resolver.rs`**
   - Added pattern binding for if-let statements

## Result

Now the following code works correctly:

```reluxscript
struct ExtractedData {
    hooks: Vec<Str>,
    templates: Vec<Str>,
}

let data = ExtractedData {
    hooks: vec![],      // Inferred as Vec<Str>
    templates: vec![],  // Inferred as Vec<Str>
};

// Also works with explicit type annotations
let items: Vec<Str> = vec![];  // Inferred from annotation
```

## Strengthened Assignability

To handle cases where expected type propagation didn't reach (e.g., complex expressions), `is_assignable_to` was updated to treat `Unknown` element types as compatible:

```rust
// In src/semantic/types.rs

pub fn is_assignable_to(&self, target: &TypeInfo) -> bool {
    match (self, target) {
        // ... exact match, null to Option, etc. ...

        // Vec<Unknown> is assignable to any Vec<T> (inference fallback)
        (TypeInfo::Vec(elem), TypeInfo::Vec(expected_elem)) => {
            match elem.as_ref() {
                TypeInfo::Unknown => true,
                _ => elem.is_assignable_to(expected_elem),
            }
        }

        // HashMap<Unknown, Unknown> is assignable to any HashMap<K, V>
        (TypeInfo::HashMap(k, v), TypeInfo::HashMap(ek, ev)) => {
            let k_ok = matches!(k.as_ref(), TypeInfo::Unknown) || k.is_assignable_to(ek);
            let v_ok = matches!(v.as_ref(), TypeInfo::Unknown) || v.is_assignable_to(ev);
            k_ok && v_ok
        }

        // HashSet<Unknown> is assignable to any HashSet<T>
        (TypeInfo::HashSet(elem), TypeInfo::HashSet(expected_elem)) => {
            match elem.as_ref() {
                TypeInfo::Unknown => true,
                _ => elem.is_assignable_to(expected_elem),
            }
        }

        // Option<Unknown> is assignable to any Option<T>
        (TypeInfo::Option(inner), TypeInfo::Option(expected_inner)) => {
            match inner.as_ref() {
                TypeInfo::Unknown => true,
                _ => inner.is_assignable_to(expected_inner),
            }
        }

        // Result<Unknown, Unknown> is assignable to any Result<T, E>
        (TypeInfo::Result(ok, err), TypeInfo::Result(eok, eerr)) => {
            let ok_ok = matches!(ok.as_ref(), TypeInfo::Unknown) || ok.is_assignable_to(eok);
            let err_ok = matches!(err.as_ref(), TypeInfo::Unknown) || err.is_assignable_to(eerr);
            ok_ok && err_ok
        }

        // Unknown matches anything (for error recovery)
        (TypeInfo::Unknown, _) | (_, TypeInfo::Unknown) => true,

        _ => false,
    }
}
```

This ensures that even if expected type propagation doesn't reach a `vec![]` expression, it can still be assigned to a typed variable or field.

## Future Improvements

This implementation handles the most common cases. Future enhancements could include:

1. **Function call argument inference** - Pass expected parameter types to arguments
2. **Return statement inference** - Use function return type to infer expressions
3. **Match arm inference** - Ensure all arms return compatible types
4. **Better error messages** - Suggest adding type annotations when inference fails
5. **Generic type inference** - Infer type parameters from usage
