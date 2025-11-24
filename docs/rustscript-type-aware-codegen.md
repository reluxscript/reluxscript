# ReluxScript Type-Aware Code Generation

**Version:** 0.4.0
**Status:** Implementation Plan
**Goal:** Generate type-correct SWC code that properly handles enum unwrapping, Box dereferencing, and field access patterns

---

## Problem Statement

ReluxScript's current code generator produces syntactically similar code for both targets, but SWC's AST has significant structural differences from Babel's that require type-aware transformations:

### Current Output (Broken)
```rust
let member = current.clone();
let property = member.prop.clone();  // ERROR: current is Expr, not MemberExpr
if matches!(property, Expr::Ident(_)) {
    let name = property.sym.clone();  // ERROR: property is MemberProp, not Ident
}
```

### Expected Output (Correct)
```rust
if let Expr::Member(member) = current.clone() {
    if let MemberProp::Ident(property) = &member.prop {
        let name = property.sym.to_string();
        parts.insert(0, name);
    }
    current = *member.obj;
}
```

---

## Core Challenge: Type Flow Analysis

The fundamental issue is that ReluxScript code assumes uniform field access, but SWC requires:

1. **Enum unwrapping** before accessing variant-specific fields
2. **Box dereferencing** for nested nodes
3. **Type-specific field names** that vary by node type
4. **Conversion methods** for JsWord ↔ String

---

## Architecture: Type Context Propagation

### Design Principle

Propagate type information through the code generator so each expression knows:
- What type it's operating on
- What type it produces
- What conversions are needed

### Type Context Structure

```rust
/// Type context for code generation
#[derive(Clone, Debug)]
pub struct TypeContext {
    /// The ReluxScript type name (e.g., "Expr", "MemberExpression")
    pub reluxscript_type: String,
    /// The SWC type (e.g., "Expr", "MemberExpr")
    pub swc_type: String,
    /// Whether this is wrapped in Box
    pub is_boxed: bool,
    /// Whether this is wrapped in Option
    pub is_optional: bool,
    /// The enum variant if this is an enum (e.g., "Member" for Expr::Member)
    pub enum_variant: Option<String>,
}

/// Variable binding with type information
pub struct TypedBinding {
    pub name: String,
    pub context: TypeContext,
}
```

---

## Implementation Phases

### Phase 1: Type Environment (Days 1-2)

Build a type environment that tracks variable types through the generated code.

#### 1.1 Variable Type Tracking

```rust
pub struct TypeEnvironment {
    /// Stack of scopes, each mapping variable names to types
    scopes: Vec<HashMap<String, TypeContext>>,
}

impl TypeEnvironment {
    pub fn define(&mut self, name: &str, ctx: TypeContext) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), ctx);
    }

    pub fn lookup(&self, name: &str) -> Option<&TypeContext> {
        for scope in self.scopes.iter().rev() {
            if let Some(ctx) = scope.get(name) {
                return Some(ctx);
            }
        }
        None
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }
}
```

#### 1.2 Type Inference for Expressions

```rust
impl SwcGenerator {
    /// Infer the type context for an expression
    fn infer_type(&self, expr: &Expr, env: &TypeEnvironment) -> TypeContext {
        match expr {
            Expr::Ident(ident) => {
                // Look up variable type
                env.lookup(&ident.name)
                    .cloned()
                    .unwrap_or(TypeContext::unknown())
            }
            Expr::Member(mem) => {
                // Get the object's type, then look up the field
                let obj_type = self.infer_type(&mem.object, env);
                self.get_field_type(&obj_type, &mem.property)
            }
            Expr::Call(call) => {
                // Handle .clone() specially
                if is_clone_call(call) {
                    self.infer_type(&get_clone_receiver(call), env)
                } else {
                    TypeContext::unknown()
                }
            }
            // ... other cases
        }
    }

    /// Get the type of a field access
    fn get_field_type(&self, obj_type: &TypeContext, field: &str) -> TypeContext {
        // Use field mapping to determine result type
        if let Some(mapping) = get_field_mapping(&obj_type.swc_type, field) {
            TypeContext {
                reluxscript_type: mapping.reluxscript_type.to_string(),
                swc_type: mapping.swc_type.to_string(),
                is_boxed: mapping.needs_box_unwrap,
                is_optional: mapping.is_optional,
                enum_variant: None,
            }
        } else {
            TypeContext::unknown()
        }
    }
}
```

---

### Phase 2: Pattern-Based Code Generation (Days 3-5)

Generate different code patterns based on the type context.

#### 2.1 Conditional Unwrapping

When accessing a field that requires enum unwrapping, generate `if let`:

**ReluxScript:**
```reluxscript
while matches!(current, MemberExpression) {
    let member = current;
    let property = member.property;
    // ...
}
```

**Generated SWC (Type-Aware):**
```rust
while let Expr::Member(member) = current {
    let property = &member.prop;
    // ...
}
```

#### 2.2 Implementation Strategy

```rust
impl SwcGenerator {
    fn gen_let_stmt_typed(&mut self, stmt: &LetStmt, env: &mut TypeEnvironment) {
        let init_type = self.infer_type(&stmt.init, env);

        // Check if we need to unwrap an enum
        if self.needs_enum_unwrap(&stmt.init, &init_type) {
            // Generate: if let Variant(binding) = expr { ... }
            self.gen_enum_unwrap_let(stmt, &init_type, env);
        } else if init_type.is_boxed {
            // Generate: let binding = *expr;
            self.gen_box_unwrap_let(stmt, &init_type, env);
        } else {
            // Generate: let binding = expr;
            self.gen_simple_let(stmt, env);
        }

        // Record the variable's type
        env.define(&stmt.name, init_type);
    }

    fn needs_enum_unwrap(&self, expr: &Expr, type_ctx: &TypeContext) -> bool {
        // Check if we're assigning from an enum type to a specific variant
        match expr {
            Expr::Ident(ident) => {
                if let Some(var_type) = self.env.lookup(&ident.name) {
                    // If the source is an enum and we're accessing it directly
                    var_type.enum_variant.is_none() &&
                    is_enum_type(&var_type.swc_type)
                } else {
                    false
                }
            }
            _ => false
        }
    }
}
```

#### 2.3 Field Access with Type Context

```rust
fn gen_member_expr_typed(&mut self, mem: &MemberExpr, env: &TypeEnvironment) {
    let obj_type = self.infer_type(&mem.object, env);
    let field_mapping = get_field_mapping(&obj_type.swc_type, &mem.property);

    // Generate object expression
    self.gen_expr_typed(&mem.object, env);

    // Apply necessary transformations
    if let Some(mapping) = field_mapping {
        // Box unwrapping
        if mapping.needs_box_unwrap {
            self.output = format!("(*{})", self.output);
        }

        // Field name mapping
        self.emit(".");
        self.emit(mapping.swc_field);

        // Read conversion (e.g., .to_string() for JsWord)
        if !mapping.swc_read_conversion.is_empty() {
            self.emit(mapping.swc_read_conversion);
        }
    } else {
        self.emit(".");
        self.emit(&mem.property);
    }
}
```

---

### Phase 3: matches! Macro Rewriting (Days 6-7)

Transform `matches!` + field access into proper `if let` patterns.

#### 3.1 Pattern Detection

Detect this pattern:
```reluxscript
while matches!(current, MemberExpression) {
    let member = current;
    let property = member.property;
    // ...
}
```

And transform to:
```rust
while let Expr::Member(member) = current {
    // member is now MemberExpr, not Expr
    let property = &member.prop;
    // ...
}
```

#### 3.2 Block Rewriting

```rust
fn gen_while_stmt_typed(&mut self, stmt: &WhileStmt, env: &mut TypeEnvironment) {
    // Check if condition is matches! macro
    if let Some((scrutinee, pattern)) = self.extract_matches_pattern(&stmt.condition) {
        // Check if first statement binds the same variable
        if let Some(binding) = self.get_binding_from_matches(&stmt.body, &scrutinee, &pattern) {
            // Generate: while let Pattern(binding) = scrutinee { ... }
            self.emit("while let ");
            self.emit_swc_pattern(&pattern, &binding);
            self.emit(" = ");
            self.gen_expr(&scrutinee);
            self.emit(" {\n");

            // Generate body with binding in scope as the unwrapped type
            env.push_scope();
            let unwrapped_type = self.get_unwrapped_type(&pattern);
            env.define(&binding, unwrapped_type);

            // Skip the binding statement since it's now in the pattern
            self.gen_block_skip_first(&stmt.body, env);

            env.pop_scope();
            self.emit("}\n");
            return;
        }
    }

    // Fallback to standard while generation
    self.gen_while_stmt_standard(stmt, env);
}
```

---

### Phase 4: MemberProp Special Handling (Day 8)

Handle the `MemberProp` enum which is different from `Expr`.

#### 4.1 MemberProp Type Mapping

```rust
// MemberProp is not Expr - it's its own enum
enum MemberProp {
    Ident(IdentName),      // .foo
    PrivateName(PrivateName), // .#foo
    Computed(ComputedPropName), // [expr]
}
```

#### 4.2 Special Generation for property Access

```rust
fn gen_property_access(&mut self, parent_type: &str, env: &TypeEnvironment) {
    if parent_type == "MemberExpr" {
        // member.prop returns MemberProp, not Expr
        // Need to generate: if let MemberProp::Ident(prop_ident) = &member.prop
        self.needs_memberprop_unwrap = true;
    }
}
```

#### 4.3 Generated Code Pattern

**ReluxScript:**
```reluxscript
if matches!(property, Identifier) {
    let name = property.name.clone();
}
```

**Generated SWC:**
```rust
if let MemberProp::Ident(prop_ident) = &member.prop {
    let name = prop_ident.sym.to_string();
}
```

---

### Phase 5: Complete Example Transform (Day 9-10)

#### Input: build_member_path.rsc
```reluxscript
pub fn build_member_path(expr: &Expr) -> Str {
    let mut parts = vec![];
    let mut current = expr.clone();

    while matches!(current, MemberExpression) {
        let member = current.clone();
        let property = member.property.clone();
        let object = member.object.clone();

        if matches!(property, Identifier) {
            let name = property.name.clone();
            parts.insert(0, name);
        }
        current = object;
    }

    if matches!(current, Identifier) {
        let name = current.name.clone();
        parts.insert(0, name);
    }

    return parts.join(".");
}
```

#### Output: Type-Aware Generated SWC
```rust
pub fn build_member_path(expr: &Expr) -> String {
    let mut parts: Vec<String> = vec![];
    let mut current = expr.clone();

    while let Expr::Member(member) = current {
        if let MemberProp::Ident(prop_ident) = &member.prop {
            let name = prop_ident.sym.to_string();
            parts.insert(0, name);
        }
        current = *member.obj.clone();
    }

    if let Expr::Ident(ident) = current {
        let name = ident.sym.to_string();
        parts.insert(0, name);
    }

    parts.join(".")
}
```

---

## Data Structures Required

### Extended Field Mapping

```rust
pub struct FieldMapping {
    pub reluxscript_node: &'static str,
    pub reluxscript_field: &'static str,
    pub babel_field: &'static str,
    pub swc_field: &'static str,
    pub needs_box_unwrap: bool,
    pub is_optional: bool,
    // NEW: Type information
    pub reluxscript_type: &'static str,  // Type of the field in ReluxScript
    pub swc_type: &'static str,         // Type of the field in SWC
    pub swc_read_conversion: &'static str,  // e.g., ".to_string()"
    pub swc_write_conversion: &'static str, // e.g., ".into()"
}
```

### Type Classification

```rust
pub enum SwcTypeKind {
    Struct,           // Direct struct (e.g., MemberExpr)
    Enum,             // Enum requiring pattern match (e.g., Expr)
    Boxed(Box<SwcTypeKind>),  // Box<T>
    Optional(Box<SwcTypeKind>), // Option<T>
    Atom,             // JsWord/Atom
    Primitive,        // String, bool, etc.
}

pub fn classify_swc_type(type_name: &str) -> SwcTypeKind {
    match type_name {
        "Expr" | "Stmt" | "Decl" | "Pat" | "Lit" | "MemberProp" => SwcTypeKind::Enum,
        "Ident" | "MemberExpr" | "CallExpr" | "FnDecl" => SwcTypeKind::Struct,
        "JsWord" | "Atom" => SwcTypeKind::Atom,
        _ if type_name.starts_with("Box<") => {
            let inner = &type_name[4..type_name.len()-1];
            SwcTypeKind::Boxed(Box::new(classify_swc_type(inner)))
        }
        _ if type_name.starts_with("Option<") => {
            let inner = &type_name[7..type_name.len()-1];
            SwcTypeKind::Optional(Box::new(classify_swc_type(inner)))
        }
        _ => SwcTypeKind::Primitive,
    }
}
```

---

## Files to Create/Modify

### New Files
- `src/codegen/type_context.rs` - Type context and environment
- `src/codegen/swc_patterns.rs` - Pattern-based code generation

### Modified Files
- `src/codegen/swc.rs` - Integrate type-aware generation
- `src/mapping/fields.rs` - Add type information to field mappings
- `src/mapping/nodes.rs` - Add type classification

---

## Testing Strategy

### Unit Tests
1. Type inference for simple expressions
2. Type inference through field access chains
3. Enum unwrap detection
4. Pattern rewriting for `while` and `if`

### Integration Tests
1. Compile `build_member_path.rsc` and verify SWC output compiles
2. Run generated code against sample AST
3. Compare results with hand-written implementation

### Validation
```bash
# Compile generated SWC code
cd reluxscript/examples/dist
rustc --edition 2021 lib.rs --extern swc_ecma_ast=... 2>&1
```

---

## Timeline

| Phase | Days | Description |
|-------|------|-------------|
| 1 | 1-2 | Type Environment & Inference |
| 2 | 3-5 | Pattern-Based Code Generation |
| 3 | 6-7 | matches! Macro Rewriting |
| 4 | 8 | MemberProp Special Handling |
| 5 | 9-10 | Complete Example & Testing |

**Total: 10 days**

---

## Success Criteria

1. `build_member_path.rsc` compiles to valid Rust code
2. Generated SWC plugin compiles without errors
3. Generated code correctly handles:
   - Enum unwrapping (Expr → MemberExpr)
   - Box dereferencing (*member.obj)
   - JsWord conversion (.sym.to_string())
   - MemberProp patterns

---

## Future Enhancements

1. **Bidirectional Type Flow** - Handle cases where the expected type influences generation
2. **Generic Type Support** - Handle Vec<T>, Option<T> properly
3. **Lifetime Annotations** - Generate correct lifetimes for references
4. **Error Recovery** - Generate placeholder code with TODO comments for unsupported patterns
