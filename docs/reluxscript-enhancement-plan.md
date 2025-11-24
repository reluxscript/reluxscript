# ReluxScript Enhancement Plan

**Version:** 0.4.0 Planning
**Status:** Proposed Enhancements
**Last Updated:** 2024

---

## Overview

This document outlines the next major enhancements for the ReluxScript compiler, building on the successful implementation of the MemberProp chasm fix (type-aware code generation with context-sensitive pattern matching).

---

## Milestone 1: Expand Pattern Coverage

### Goal
Extend the type context system to handle all common SWC wrapper enums, not just `Expr` and `MemberProp`.

### Priority: High
### Effort: 2-4 hours
### Dependencies: None (builds on existing infrastructure)

### Target Patterns

| Context | ReluxScript Type | SWC Pattern | SWC Struct |
|---------|----------------|-------------|------------|
| `Lit` | `StringLiteral` | `Lit::Str` | `Str` |
| `Lit` | `NumericLiteral` | `Lit::Num` | `Number` |
| `Lit` | `BooleanLiteral` | `Lit::Bool` | `Bool` |
| `Lit` | `NullLiteral` | `Lit::Null` | `Null` |
| `Lit` | `RegExpLiteral` | `Lit::Regex` | `Regex` |
| `Pat` | `Identifier` | `Pat::Ident` | `BindingIdent` |
| `Pat` | `ArrayPattern` | `Pat::Array` | `ArrayPat` |
| `Pat` | `ObjectPattern` | `Pat::Object` | `ObjectPat` |
| `Pat` | `RestElement` | `Pat::Rest` | `RestPat` |
| `PropName` | `Identifier` | `PropName::Ident` | `Ident` |
| `PropName` | `StringLiteral` | `PropName::Str` | `Str` |
| `PropName` | `NumericLiteral` | `PropName::Num` | `Number` |
| `PropName` | `ComputedPropName` | `PropName::Computed` | `ComputedPropName` |
| `JSXObject` | `Identifier` | `JSXObject::Ident` | `Ident` |
| `JSXObject` | `JSXMemberExpression` | `JSXObject::JSXMemberExpr` | `JSXMemberExpr` |
| `JSXElementName` | `Identifier` | `JSXElementName::Ident` | `Ident` |
| `JSXElementName` | `JSXMemberExpression` | `JSXElementName::JSXMemberExpr` | `JSXMemberExpr` |
| `JSXAttrValue` | `StringLiteral` | `JSXAttrValue::Lit` | `Lit` |
| `JSXAttrValue` | `JSXExpressionContainer` | `JSXAttrValue::JSXExprContainer` | `JSXExprContainer` |

### Implementation

#### 1. Update `get_swc_variant_in_context` in `type_context.rs`

```rust
pub fn get_swc_variant_in_context(rs_type: &str, context: &str) -> (String, String, String) {
    // Existing: MemberProp, Callee, Expr contexts...

    // Lit context
    if context == "Lit" {
        return match rs_type {
            "StringLiteral" => ("Lit".into(), "Str".into(), "Str".into()),
            "NumericLiteral" => ("Lit".into(), "Num".into(), "Number".into()),
            "BooleanLiteral" => ("Lit".into(), "Bool".into(), "Bool".into()),
            "NullLiteral" => ("Lit".into(), "Null".into(), "Null".into()),
            "RegExpLiteral" => ("Lit".into(), "Regex".into(), "Regex".into()),
            _ => ("Lit".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Pat context
    if context == "Pat" {
        return match rs_type {
            "Identifier" => ("Pat".into(), "Ident".into(), "BindingIdent".into()),
            "ArrayPattern" => ("Pat".into(), "Array".into(), "ArrayPat".into()),
            "ObjectPattern" => ("Pat".into(), "Object".into(), "ObjectPat".into()),
            "RestElement" => ("Pat".into(), "Rest".into(), "RestPat".into()),
            "AssignmentPattern" => ("Pat".into(), "Assign".into(), "AssignPat".into()),
            _ => ("Pat".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // PropName context
    if context == "PropName" {
        return match rs_type {
            "Identifier" => ("PropName".into(), "Ident".into(), "Ident".into()),
            "StringLiteral" => ("PropName".into(), "Str".into(), "Str".into()),
            "NumericLiteral" => ("PropName".into(), "Num".into(), "Number".into()),
            "ComputedPropName" => ("PropName".into(), "Computed".into(), "ComputedPropName".into()),
            _ => ("PropName".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // JSXObject context
    if context == "JSXObject" {
        return match rs_type {
            "Identifier" => ("JSXObject".into(), "Ident".into(), "Ident".into()),
            "JSXMemberExpression" => ("JSXObject".into(), "JSXMemberExpr".into(), "JSXMemberExpr".into()),
            _ => ("JSXObject".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // ... rest of existing code
}
```

#### 2. Add Field Mappings for New Types

Update `get_typed_field_mapping` to include result types for fields that return these wrapper enums:

```rust
// FnDecl fields
("FnDecl", "params") => Some(TypedFieldMapping {
    reluxscript_field: "params",
    swc_field: "function.params",
    needs_deref: false,
    result_type_rs: "Vec<Pat>",
    result_type_swc: "Vec<Param>",
    read_conversion: "",
    write_conversion: "",
}),

// ObjectLit fields (for object properties)
("ObjectLit", "properties") => Some(TypedFieldMapping {
    reluxscript_field: "properties",
    swc_field: "props",
    needs_deref: false,
    result_type_rs: "Vec<PropOrSpread>",
    result_type_swc: "Vec<PropOrSpread>",
    read_conversion: "",
    write_conversion: "",
}),
```

### Testing

Create test ReluxScript files that exercise each pattern:

```reluxscript
// test_patterns.lux
plugin PatternTests {
    fn test_literal_matching(expr: &Expr) {
        if matches!(expr, Literal) {
            let lit = expr;
            if matches!(lit, StringLiteral) {
                // Should generate: if let Lit::Str(lit) = lit
            }
        }
    }

    fn test_pattern_matching(param: &Pat) {
        if matches!(param, Identifier) {
            // Should generate: if let Pat::Ident(param) = param
            let name = param.name.clone();
        }
    }
}
```

---

## Milestone 2: Auto-Unwrap Chains

### Goal
Allow ReluxScript to access nested fields through wrapper enums without explicit pattern matching, with the compiler auto-generating the necessary unwrap code.

### Priority: High
### Effort: 8-12 hours
### Dependencies: Milestone 1 (pattern coverage)

### The Problem

In Babel, you can write:
```javascript
const name = node.property.name;
```

In SWC, you must write:
```rust
let name = if let MemberProp::Ident(prop) = &node.prop {
    prop.sym.clone()
} else {
    panic!("Expected Ident");
};
```

### The Solution

ReluxScript should allow the Babel-style syntax and auto-generate the unwrap:

```reluxscript
let name = node.property.name;
```

Compiles to:
```rust
let name = match &node.prop {
    MemberProp::Ident(__tmp) => __tmp.sym.clone(),
    _ => panic!("Expected Ident for property.name access"),
};
```

### Design Decisions

#### 1. Error Handling Strategy

**Option A: Panic on mismatch (strict mode)**
```rust
let name = match &node.prop {
    MemberProp::Ident(prop) => prop.sym.clone(),
    _ => panic!("Expected Ident"),
};
```
- Pro: Catches bugs early
- Con: Runtime panics

**Option B: Return Option (safe mode)**
```rust
let name: Option<JsWord> = match &node.prop {
    MemberProp::Ident(prop) => Some(prop.sym.clone()),
    _ => None,
};
```
- Pro: Safe, explicit
- Con: Requires `.unwrap()` or `?` propagation

**Option C: Configurable via attribute**
```reluxscript
#[unwrap_mode(panic)]  // or "option" or "result"
let name = node.property.name;
```

**Recommendation:** Start with Option A (panic) for simplicity, add Option C later.

#### 2. Chain Detection Algorithm

When visiting `Expr::Member` chains like `a.b.c.d`:

1. Build the full chain: `[a, b, c, d]`
2. For each step, check if the field returns a wrapper enum
3. If yes, mark that step as "needs unwrap"
4. Generate nested unwrap code

```rust
// a.b.c.d where b returns WrapperEnum and d requires unwrap

let __result = {
    let __tmp1 = &a.b;
    match __tmp1 {
        WrapperEnum::Expected(__tmp2) => {
            __tmp2.c.d.clone()
        }
        _ => panic!("..."),
    }
};
```

#### 3. Type Inference for Chains

The compiler needs to track types through the entire chain:

```
node: MemberExpr
  .property → MemberProp (wrapper enum!)
    .name → ??? (only valid if MemberProp::Ident)
```

When we see `.name` after a `MemberProp`, we know:
- User expects `MemberProp::Ident`
- We should auto-unwrap to `Ident`
- Then access `.sym` (mapped from `.name`)

### Implementation Plan

#### Phase 1: Chain Analysis (2-3 hours)

Create a new analysis pass that:
1. Collects member expression chains
2. Identifies which steps need unwrapping
3. Stores this information for code generation

```rust
// New struct in type_context.rs
pub struct ChainAnalysis {
    pub steps: Vec<ChainStep>,
}

pub struct ChainStep {
    pub field_name: String,
    pub result_type: TypeContext,
    pub needs_unwrap: bool,
    pub expected_variant: Option<String>,
}

impl SwcGenerator {
    fn analyze_member_chain(&self, expr: &MemberExpr) -> ChainAnalysis {
        // ... implementation
    }
}
```

#### Phase 2: Unwrap Code Generation (3-4 hours)

Modify `gen_expr` for `Expr::Member` to:
1. Check if chain needs unwrapping
2. Generate match/if-let blocks
3. Handle nested unwraps

```rust
fn gen_member_with_unwrap(&mut self, chain: &ChainAnalysis) {
    let unwrap_steps: Vec<_> = chain.steps.iter()
        .filter(|s| s.needs_unwrap)
        .collect();

    if unwrap_steps.is_empty() {
        // Simple case: no unwrapping needed
        self.gen_simple_member_chain(chain);
    } else {
        // Generate nested match blocks
        self.gen_unwrap_chain(chain, &unwrap_steps);
    }
}
```

#### Phase 3: Edge Cases (2-3 hours)

Handle:
- Multiple unwraps in one chain: `a.b.c.d` where both `b` and `d` need unwrap
- Assignment targets: `node.property.name = "foo"`
- Method calls on unwrapped values: `node.property.name.to_uppercase()`

### Example Transformations

#### Simple Unwrap

```reluxscript
let name = member.property.name;
```

```rust
let name = match &member.prop {
    MemberProp::Ident(__prop) => __prop.sym.clone(),
    _ => panic!("Expected MemberProp::Ident for .property.name access"),
};
```

#### Chained Unwrap

```reluxscript
let value = expr.callee.object.name;
```

```rust
let value = match &expr.callee {
    Callee::Expr(__callee) => {
        match __callee.as_ref() {
            Expr::Member(__member) => {
                match &__member.obj.as_ref() {
                    Expr::Ident(__obj) => __obj.sym.clone(),
                    _ => panic!("Expected Expr::Ident"),
                }
            }
            _ => panic!("Expected Expr::Member"),
        }
    }
    _ => panic!("Expected Callee::Expr"),
};
```

#### With Explicit Check

If the user writes explicit `matches!`, no auto-unwrap is generated:

```reluxscript
if matches!(member.property, Identifier) {
    let name = member.property.name;  // Auto-unwrap here
}
```

```rust
if let MemberProp::Ident(__property) = &member.prop {
    let name = __property.sym.clone();  // Already unwrapped by if-let
}
```

### Testing

```reluxscript
// test_auto_unwrap.lux
plugin AutoUnwrapTests {
    fn test_simple_unwrap(member: &MemberExpr) {
        // Should auto-unwrap MemberProp::Ident
        let name = member.property.name;
    }

    fn test_chained_unwrap(call: &CallExpr) {
        // Should unwrap Callee::Expr, then Expr::Member, then Expr::Ident
        let name = call.callee.object.name;
    }

    fn test_conditional_access(member: &MemberExpr) {
        // Inside matches!, should use the narrowed type
        if matches!(member.property, Identifier) {
            let name = member.property.name;
        }
    }
}
```

---

## Milestone 3: Babel Output Improvements

### Goal
Improve the Babel code generator to match the quality of the SWC generator.

### Priority: Medium
### Effort: 4-6 hours
### Dependencies: None

### Current Issues

1. **`matches!` not translated**: Generates `(current === MemberExpression)` instead of `t.isMemberExpression(current)`

2. **`.clone()` not removed**: JavaScript doesn't need `.clone()`

3. **`.insert(0, x)` not translated**: Should be `unshift(x)` in JavaScript

4. **Method translations missing**: Various Rust idioms need JS equivalents

### Implementation

#### 1. Fix `matches!` Translation

In `babel.rs`, update `gen_expr` for `Expr::Call` when callee is `matches!`:

```javascript
// ReluxScript: matches!(node, Identifier)
// Babel: t.isIdentifier(node)
```

#### 2. Remove `.clone()` Calls

JavaScript passes by reference for objects, so `.clone()` is unnecessary:

```javascript
// ReluxScript: let copy = node.clone();
// Babel: const copy = node;  // Just reference
```

#### 3. Translate Collection Methods

| ReluxScript | JavaScript |
|------------|------------|
| `vec.insert(0, x)` | `vec.unshift(x)` |
| `vec.push(x)` | `vec.push(x)` |
| `vec.pop()` | `vec.pop()` |
| `vec.len()` | `vec.length` |
| `str.len()` | `str.length` |
| `parts.join(".")` | `parts.join(".")` |

---

## Milestone 4: Error Recovery and Diagnostics

### Goal
Improve error messages and add recovery for common mistakes.

### Priority: Medium
### Effort: 6-8 hours
### Dependencies: Milestones 1-2

### Features

1. **Type mismatch errors with suggestions**
   ```
   error[RS004]: Cannot access .name on MemberProp
     --> src/plugin.lux:15:20
      |
   15 |     let name = member.property.name;
      |                       ^^^^^^^^ MemberProp is a wrapper enum
      |
   help: MemberProp could be Ident or Computed
   help: use matches!(member.property, Identifier) to narrow the type
   ```

2. **Missing field suggestions**
   ```
   error[RS005]: Unknown field 'object' on MemberExpr
     --> src/plugin.lux:10:25
      |
   10 |     let obj = member.object;
      |                      ^^^^^^
      |
   help: did you mean 'obj'?
   ```

3. **Auto-fix suggestions**
   - Offer to wrap accesses in `matches!`
   - Suggest correct field names

---

## Implementation Timeline

| Week | Milestone | Deliverable |
|------|-----------|-------------|
| 1 | Milestone 1 | Extended pattern coverage for all wrapper enums |
| 1-2 | Milestone 2 | Auto-unwrap chains for single-level unwraps |
| 2 | Milestone 2 | Auto-unwrap chains for nested unwraps |
| 3 | Milestone 3 | Babel output improvements |
| 3-4 | Milestone 4 | Error recovery and diagnostics |

---

## Success Metrics

1. **Milestone 1 Complete**: All patterns in the table generate correct SWC code
2. **Milestone 2 Complete**: `build_member_path` compiles without explicit `matches!` checks
3. **Milestone 3 Complete**: Generated Babel code runs without modification
4. **Milestone 4 Complete**: Error messages include file/line and suggestions

---

## Future Considerations

### Type Inference Improvements
- Infer generic types for `Vec<T>`
- Track `Option<T>` through chains
- Support for `Result<T, E>` patterns

### Performance Optimizations
- Avoid unnecessary clones in generated code
- Use `&str` instead of `String` where possible
- Batch pattern matches for multiple field accesses

### Language Features
- `guard` expressions for pattern-with-condition
- `let else` patterns for early returns
- Destructuring in function parameters
