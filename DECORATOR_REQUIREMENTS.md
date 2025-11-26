# SWC Decorator Requirements Analysis

## Goal
Make SWC codegen "dumb" - all semantic decisions made during decoration phase.

## Current swc.rs Analysis

### ✅ Already Handled by Decorator

1. **Pattern Type Mapping** (lines 2226-2247)
   - `Expression::Identifier` → `Expr::Ident` (basic mapping)
   - ✅ Decorator now provides: context-aware mapping (e.g., `MemberProp::Ident` when matching `member.prop`)

2. **Field Name Mapping** (lines 2570-2596)
   - `object` → `obj`, `property` → `prop`, `name` → `sym`
   - Uses `infer_type()` and `get_typed_field_mapping()`
   - ✅ Decorator now provides: `SwcFieldMetadata` with exact field name and accessor strategy

3. **Type Inference** (used throughout)
   - `self.type_env` tracks variable types
   - ✅ Decorator now provides: `SwcExprMetadata` with pre-computed type info

### ❌ NOT Yet Captured - Needs Decorator Enhancement

#### 1. **Identifier String Comparisons** (CRITICAL for console_remover)

**Problem:** `obj.name == "console"` needs to become `&*obj.sym == "console"`

**Current State:** No special handling in binary expression generation (line 2286-2291)

**Decorator Must Capture:**
```rust
// In DecoratedExprKind::Binary:
pub struct SwcBinaryMetadata {
    /// If left side is identifier.name, need to emit &*ident.sym
    pub left_needs_sym_deref: bool,
    /// If right side is identifier.name, need to emit &*ident.sym
    pub right_needs_sym_deref: bool,
}

// Detection logic in decorator:
fn decorate_binary(&mut self, bin: &BinaryExpr) -> DecoratedExpr {
    let left = self.decorate_expr(&bin.left);
    let right = self.decorate_expr(&bin.right);

    // Check if either side is member.name/member.sym access
    let left_needs_sym_deref = matches!(&left.kind,
        DecoratedExprKind::Member { property, field_metadata, .. }
        if property == "name" && field_metadata.swc_field_name == "sym"
    );

    let right_needs_sym_deref = matches!(&right.kind,
        DecoratedExprKind::Member { property, field_metadata, .. }
        if property == "name" && field_metadata.swc_field_name == "sym"
    );

    DecoratedExpr {
        kind: DecoratedExprKind::Binary {
            left: Box::new(left),
            op: bin.op.clone(),
            right: Box::new(right),
            binary_metadata: SwcBinaryMetadata {
                left_needs_sym_deref,
                right_needs_sym_deref,
            },
        },
        metadata: SwcExprMetadata { ... },
    }
}
```

**SWC Codegen becomes:**
```rust
DecoratedExprKind::Binary { left, op, right, binary_metadata } => {
    self.emit("(");
    if binary_metadata.left_needs_sym_deref {
        self.emit("&*");
    }
    self.gen_decorated_expr(left);
    self.emit(&format!(" {} ", op));
    if binary_metadata.right_needs_sym_deref {
        self.emit("&*");
    }
    self.gen_decorated_expr(right);
    self.emit(")");
}
```

#### 2. **Nested Member Access Unwrapping** (lines 2509-2531)

**Problem:** `node.callee.name` needs complex match expressions

**Current:** Ad-hoc code generation with hardcoded patterns

**Decorator Must Capture:**
```rust
// In SwcFieldMetadata:
pub enum FieldAccessStrategy {
    Direct,
    BoxAsRef,
    NestedUnwrap {
        /// Chain of unwraps needed
        /// e.g., ["Callee::Expr", "Expr::Ident"] for callee.name
        unwrap_chain: Vec<String>,
        /// Final accessor (e.g., "sym" for name)
        final_field: String,
    },
}
```

**Detection in Decorator:**
```rust
// When decorating member.property where property == "name"
if let DecoratedExprKind::Member { object, property: inner_prop, .. } = &object.kind {
    if inner_prop == "callee" {
        // This is obj.callee.name - needs nested unwrapping
        metadata.accessor = FieldAccessStrategy::NestedUnwrap {
            unwrap_chain: vec!["Callee::Expr".into(), "Expr::Ident".into()],
            final_field: "sym".into(),
        };
    }
}
```

#### 3. **Special Member Mappings** (lines 2533-2554)

**Problem:** `self.builder` → `self`, `self.state` → `self` in writers

**Current:** Hardcoded checks in member expression generation

**Decorator Must Capture:**
```rust
pub enum FieldAccessStrategy {
    Direct,
    BoxAsRef,
    Replace {
        /// Replacement expression
        /// e.g., "self" for self.builder
        with: String,
    },
    // ...
}

// In decorator:
if object_is_self && property == "builder" && self.is_writer {
    return DecoratedExpr {
        kind: DecoratedExprKind::Ident {
            name: "self".into(),
            ident_metadata: SwcIdentifierMetadata::name(),
        },
        // ...
    };
}
```

#### 4. **Dereference Operator Context** (lines 2303-2306)

**Problem:** `*member.prop` might need to be `&member.prop` for enum fields

**Current:** Simple emission without context awareness

**Decorator Must Capture:**
```rust
// In SwcUnaryMetadata (already defined):
pub struct SwcUnaryMetadata {
    pub override_op: Option<String>,  // ✅ Already have this!
}

// In decorator:
fn decorate_unary(&mut self, un: &UnaryExpr) -> DecoratedExpr {
    let operand = self.decorate_expr(&un.operand);

    let override_op = if un.op == UnaryOp::Deref {
        // Check if operand is an enum field
        if let DecoratedExprKind::Member { field_metadata, .. } = &operand.kind {
            if matches!(field_metadata.accessor, FieldAccessor::EnumField { .. }) {
                Some("&".into())  // Use & instead of * for enum fields
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    DecoratedExpr {
        kind: DecoratedExprKind::Unary {
            op: un.op.clone(),
            operand: Box::new(operand),
            unary_metadata: SwcUnaryMetadata { override_op, span: None },
        },
        // ...
    }
}
```

#### 5. **Callee::MemberExpression Desugaring** (lines 1700-1753)

**Problem:** `Callee::MemberExpression` pattern needs to desugar into nested if-lets

**Current:** Hardcoded special case in `gen_if_let_stmt`

**Decorator Must Capture:**
```rust
// In SwcPatternMetadata:
pub enum PatternStrategy {
    Direct {
        swc_pattern: String,
    },
    Desugar {
        /// Sequence of nested patterns to generate
        nested_patterns: Vec<NestedPattern>,
    },
}

pub struct NestedPattern {
    pub pattern: String,
    pub binding: String,
    pub condition_expr: String,  // e.g., "&node.callee", "__callee_expr.as_ref()"
}

// In decorator:
if pattern_name == "Callee::MemberExpression" {
    metadata.strategy = PatternStrategy::Desug ar {
        nested_patterns: vec![
            NestedPattern {
                pattern: "Callee::Expr(__callee_expr)".into(),
                binding: "__callee_expr".into(),
                condition_expr: "&{condition}".into(),
            },
            NestedPattern {
                pattern: "Expr::Member({inner})".into(),
                binding: "member".into(),
                condition_expr: "__callee_expr.as_ref()".into(),
            },
        ],
    };
}
```

### ✅ Simple Cases (Already Fine)

1. **Basic Literals** (lines 2266, 2182) - No decoration needed
2. **Wildcards** (line 2184) - No decoration needed
3. **Tuples/Arrays** (lines 2185-2204) - Structural, no special handling
4. **Module Path Translation** (lines 2558-2561) - Simple mapping, can stay in codegen
5. **Associated Functions** (lines 2324-2335) - Decorator tracks this via `associated_functions` set

## Summary: What Decorator Must Add

### High Priority (Breaks console_remover):

1. ✅ **Context-aware pattern mapping** - DONE
2. ✅ **Field access metadata** - DONE
3. ❌ **Binary expression sym deref** - MISSING
4. ❌ **Identifier.name → &*ident.sym in comparisons** - MISSING

### Medium Priority (Nice to have):

5. ❌ **Nested member unwrapping** - MISSING (callee.name, key.name)
6. ❌ **Special member replacements** - MISSING (self.builder → self)
7. ❌ **Unary operator context** - PARTIALLY DONE (metadata exists, logic missing)

### Low Priority (Can stay in codegen):

8. ✅ **Module path translation** - CAN SKIP
9. ✅ **Associated function detection** - ALREADY TRACKED
10. ✅ **Callee::MemberExpression desugaring** - CAN BE HANDLED IN CODEGEN

## Immediate Next Steps

1. Add `left_needs_sym_deref` / `right_needs_sym_deref` to `SwcBinaryMetadata`
2. Implement binary expression decoration with sym deref detection
3. Update SWC codegen to emit `&*` prefix based on metadata
4. Test console_remover!

Then tackle nested unwrapping and special cases.
