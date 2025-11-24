# ReluxScript Priority 1 Implementation Plan

## Overview

This document outlines the implementation plan for Priority 1 patterns required to convert babel-plugin-minimact to ReluxScript. These patterns are **critical blockers** - without them, approximately 40% of minimact's functionality cannot be ported.

## Executive Summary

| Category | Patterns | Effort Estimate | Impact |
|----------|----------|-----------------|--------|
| TypeScript AST Support | 8 node types | 3-4 days | Critical |
| Optional Chaining | 2 node types | 1 day | High |
| Template Literals | Field enhancements | 0.5 days | High |
| Array Expressions | 1 node type | 0.5 days | Medium |

**Total Estimated Effort: 5-6 days**

---

## Critical Refinements

### Refinement 1: Type vs TsType Naming Collision

**Risk:** The plan introduces a `Type` enum for TypeScript AST nodes. However, ReluxScript already has a `Type` enum for its own variable definitions (`i32`, `Str`, `&MemberExpression`).

**Fix:** Rename the TypeScript AST type enum to `TsType`. This clearly distinguishes:
- `Type` = The type of a ReluxScript variable
- `TsType` = A node in the AST representing a TypeScript type annotation

All TypeScript type variants (`TSString`, `TSNumber`, `TSTypeReference`, etc.) will be variants of `TsType`, not `Type`.

### Refinement 2: Optional Chaining Structure

**Risk:** SWC's `OptChainExpr` is structurally different from Babel's `OptionalMemberExpression`:
- Babel: `OptionalMemberExpression` is a distinct node type
- SWC: `OptChainExpr` is a wrapper around the base expression

**Strategy:** Use Option B (adding `optional` flag) for the Unified AST. It's cleaner.

**Code Generation:**
- **Babel:** Trivial pass-through (`.?` is native JS)
- **SWC:** In `gen_member_expr`, check `if expr.optional`. If true, wrap output in `OptChainExpr` structure

### Refinement 3: Flatten Interface Body

**Risk:** `TSInterfaceBody` in Babel contains `body: [TSElement]`. In SWC, `TsInterfaceBody` contains `body: Vec<TsTypeElement>`. Modeling the intermediate `Body` node adds unnecessary complexity.

**Fix:** Map `InterfaceDeclaration.members` directly to the inner vector, skipping the intermediate Body node. This simplifies ReluxScript visitor logic significantly.

```rust
// GOOD: Flat structure
pub struct InterfaceDecl {
    pub name: String,
    pub members: Vec<InterfaceMember>,  // Direct access to members
    pub extends: Vec<String>,
    pub type_params: Vec<TypeParam>,
    pub span: Span,
}

// BAD: Nested structure (avoid)
pub struct InterfaceDecl {
    pub name: String,
    pub body: InterfaceBody,  // Extra indirection
    // ...
}
```

---

## 1. TypeScript AST Support

### 1.1 Problem Statement

Minimact heavily uses TypeScript for type safety and code generation. The plugin extracts types from:
- Interface declarations (ViewModel types)
- Generic type parameters (`useState<T>()`)
- Type annotations on parameters and return values
- Property signatures in interfaces

Without TypeScript AST support, ReluxScript cannot:
- Extract ViewModel property types
- Infer state types from generics
- Generate properly typed C# code

### 1.2 Required Node Types

#### 1.2.1 TSInterfaceDeclaration

**Babel AST Structure:**
```javascript
{
  type: "TSInterfaceDeclaration",
  id: Identifier,
  body: TSInterfaceBody,
  extends: [TSExpressionWithTypeArguments] | null,
  typeParameters: TSTypeParameterDeclaration | null
}
```

**Usage in Minimact:**
```javascript
// Finding ViewModel interface
for (const statement of programNode.body) {
  if (t.isTSInterfaceDeclaration(statement)) {
    if (statement.id.name.endsWith('ViewModel')) {
      viewModelInterface = statement;
    }
  }
}
```

**ReluxScript Mapping:**

| ReluxScript | Babel | SWC |
|------------|-------|-----|
| `InterfaceDeclaration` | `TSInterfaceDeclaration` | `TsInterfaceDecl` |

**Implementation:**

1. **Parser (ast.rs):** Add `InterfaceDecl` to `TopLevelDecl` enum
```rust
pub enum TopLevelDecl {
    // ... existing variants
    Interface(InterfaceDecl),
}

/// Interface declaration (per Refinement 3: flattened structure)
pub struct InterfaceDecl {
    pub name: String,
    pub members: Vec<InterfaceMember>,  // Direct access, no intermediate Body
    pub extends: Vec<String>,
    pub type_params: Vec<TypeParam>,
    pub span: Span,
}

pub enum InterfaceMember {
    Property(PropertySignature),
    Method(MethodSignature),
    Index(IndexSignature),
}

pub struct TypeParam {
    pub name: String,
    pub constraint: Option<TsType>,
    pub default: Option<TsType>,
}
```

2. **Mapping (nodes.rs):** Add node mapping
```rust
NodeMapping {
    reluxscript: "InterfaceDeclaration",
    babel: "TSInterfaceDeclaration",
    swc: "TsInterfaceDecl",
    category: "declaration",
}
```

3. **Field Mappings (fields.rs):** (per Refinement 3: flatten body access)
```rust
FieldMapping {
    node_type: "InterfaceDeclaration",
    field_name: "members",
    babel_access: "body.body",    // Babel: node.body.body
    swc_access: "body.body",      // SWC: node.body.body
    result_type_rs: "Vec<InterfaceMember>",
    result_type_babel: "Array<TSTypeElement>",
    result_type_swc: "Vec<TsTypeElement>",
    needs_deref: false,
}

FieldMapping {
    node_type: "InterfaceDeclaration",
    field_name: "extends",
    babel_access: "extends",
    swc_access: "extends",
    result_type_rs: "Vec<String>",
    result_type_babel: "Array<TSExpressionWithTypeArguments>",
    result_type_swc: "Vec<TsExprWithTypeArgs>",
    needs_deref: false,
}
```

---

#### 1.2.2 TSPropertySignature

**Babel AST Structure:**
```javascript
{
  type: "TSPropertySignature",
  key: Identifier | StringLiteral,
  typeAnnotation: TSTypeAnnotation | null,
  optional: boolean,
  readonly: boolean
}
```

**Usage in Minimact:**
```javascript
for (const member of viewModelInterface.body.body) {
  if (t.isTSPropertySignature(member)) {
    if (t.isIdentifier(member.key) && member.key.name === propertyName) {
      const typeAnnotation = member.typeAnnotation?.typeAnnotation;
      return tsTypeToCSharpType(typeAnnotation);
    }
  }
}
```

**ReluxScript Mapping:**

| ReluxScript | Babel | SWC |
|------------|-------|-----|
| `PropertySignature` | `TSPropertySignature` | `TsPropertySignature` |

**Implementation:**

```rust
pub struct PropertySignature {
    pub key: String,
    pub type_annotation: Option<Type>,
    pub optional: bool,
    pub readonly: bool,
    pub span: Span,
}
```

---

#### 1.2.3 TSTypeAnnotation

**Babel AST Structure:**
```javascript
{
  type: "TSTypeAnnotation",
  typeAnnotation: TSType  // The actual type
}
```

**Usage in Minimact:**
```javascript
// Extract type from annotation
const typeAnnotation = member.typeAnnotation?.typeAnnotation;
if (t.isTSStringKeyword(typeAnnotation)) {
  return 'string';
}
```

**ReluxScript Approach:**

Create a separate `TsType` enum for TypeScript type annotations (per Refinement 1). This keeps the existing `Type` enum clean for ReluxScript's own type system:

```rust
/// TypeScript type annotation AST node
/// Distinct from ReluxScript's Type enum which represents variable types
pub enum TsType {
    // Keywords
    String,
    Number,
    Boolean,
    Any,
    Void,
    Null,
    Undefined,
    Never,
    Unknown,

    // Compound types
    Array(Box<TsType>),
    Tuple(Vec<TsType>),
    Union(Vec<TsType>),
    Intersection(Vec<TsType>),

    // Reference types
    TypeReference {
        name: String,
        type_args: Vec<TsType>,
    },

    // Function types
    FunctionType {
        params: Vec<TsType>,
        return_type: Box<TsType>,
    },

    // Literal types
    LiteralString(String),
    LiteralNumber(f64),
    LiteralBoolean(bool),
}
```

**Conversion Helper:**

```rust
impl TsType {
    /// Convert TypeScript type to ReluxScript Type for codegen
    pub fn to_reluxscript_type(&self) -> Type {
        match self {
            TsType::String => Type::Primitive("Str".to_string()),
            TsType::Number => Type::Primitive("f64".to_string()),
            TsType::Boolean => Type::Primitive("bool".to_string()),
            TsType::Array(inner) => Type::Container {
                name: "Vec".to_string(),
                type_args: vec![inner.to_reluxscript_type()],
            },
            TsType::TypeReference { name, type_args } => {
                if type_args.is_empty() {
                    Type::Named(name.clone())
                } else {
                    Type::Container {
                        name: name.clone(),
                        type_args: type_args.iter().map(|t| t.to_reluxscript_type()).collect(),
                    }
                }
            }
            _ => Type::Named("dynamic".to_string()),
        }
    }
}
```

---

#### 1.2.4 TSTypeReference

**Babel AST Structure:**
```javascript
{
  type: "TSTypeReference",
  typeName: Identifier | TSQualifiedName,
  typeParameters: TSTypeParameterInstantiation | null
}
```

**Usage in Minimact:**
```javascript
// Promise<T> → T
if (t.isTSTypeReference(returnType) &&
    t.isIdentifier(returnType.typeName) &&
    returnType.typeName.name === 'Promise') {
  if (returnType.typeParameters?.params.length > 0) {
    return extractTypeAnnotation(returnType.typeParameters.params[0]);
  }
}
```

**ReluxScript Mapping:**

| ReluxScript | Babel | SWC |
|------------|-------|-----|
| `TypeReference` | `TSTypeReference` | `TsTypeRef` |

**Implementation:**

This maps to the `TSTypeReference` variant in the `Type` enum above.

---

#### 1.2.5 Type Parameter Extraction

**Babel AST Structure:**
```javascript
// On CallExpression
{
  type: "CallExpression",
  callee: Expression,
  arguments: [Expression],
  typeParameters: TSTypeParameterInstantiation | null  // <-- Key field
}

// TSTypeParameterInstantiation
{
  type: "TSTypeParameterInstantiation",
  params: [TSType]
}
```

**Usage in Minimact:**
```javascript
// useState<decimal>(0) - extract 'decimal' type
if (path.node.typeParameters && path.node.typeParameters.params.length > 0) {
  const typeParam = path.node.typeParameters.params[0];
  explicitType = tsTypeToCSharpType(typeParam);
}
```

**ReluxScript Implementation:**

Add `type_args` field to `CallExpr`:

```rust
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub type_args: Vec<Type>,  // NEW: Generic type arguments
    pub span: Span,
}
```

**Field Mapping:**
```rust
FieldMapping {
    node_type: "CallExpression",
    field_name: "typeArguments",  // ReluxScript name
    babel_access: "typeParameters.params",
    swc_access: "type_args.params",
    result_type_rs: "Vec<Type>",
    // ...
}
```

---

#### 1.2.6 Additional TypeScript Keywords

These are simple type keywords that need mapping:

| ReluxScript Check | Babel | SWC |
|------------------|-------|-----|
| `matches!(ty, TSString)` | `t.isTSStringKeyword()` | `TsKeywordType::TsStringKeyword` |
| `matches!(ty, TSNumber)` | `t.isTSNumberKeyword()` | `TsKeywordType::TsNumberKeyword` |
| `matches!(ty, TSBoolean)` | `t.isTSBooleanKeyword()` | `TsKeywordType::TsBooleanKeyword` |
| `matches!(ty, TSAny)` | `t.isTSAnyKeyword()` | `TsKeywordType::TsAnyKeyword` |
| `matches!(ty, TSVoid)` | `t.isTSVoidKeyword()` | `TsKeywordType::TsVoidKeyword` |
| `matches!(ty, TSArray)` | `t.isTSArrayType()` | `TsArrayType` |

---

### 1.3 TypeScript Implementation Strategy

#### Phase 1: Core Type System (Day 1)
1. Add TypeScript type variants to `Type` enum in `ast.rs`
2. Update parser to handle TypeScript type annotations
3. Add type keyword mappings to `nodes.rs`

#### Phase 2: Interface Support (Day 2)
1. Add `InterfaceDecl` and related structs to AST
2. Implement parser support for interface declarations
3. Add node and field mappings
4. Update Babel/SWC generators to handle interfaces

#### Phase 3: Generic Type Parameters (Day 3)
1. Add `type_args` to `CallExpr`
2. Update parser for `<T>` syntax after function calls
3. Add field mappings for type parameters
4. Test with `useState<string>()` patterns

#### Phase 4: Type Inference Helpers (Day 4)
1. Implement `tsTypeToCSharpType()` equivalent in ReluxScript
2. Add helper functions for common type conversions
3. Test with ViewModel type extraction scenarios

---

## 2. Optional Chaining Support

### 2.1 Problem Statement

Minimact uses optional chaining (`?.`) for safe property access:
```javascript
const email = viewModel?.userEmail;
const name = user?.profile?.name;
```

Without this, ReluxScript cannot handle defensive coding patterns common in React applications.

### 2.2 Required Node Types

#### 2.2.1 OptionalMemberExpression

**Babel AST Structure:**
```javascript
{
  type: "OptionalMemberExpression",
  object: Expression,
  property: Identifier | Expression,
  computed: boolean,
  optional: boolean  // true for the `?.` part
}
```

**Usage in Minimact:**
```javascript
} else if (t.isOptionalMemberExpression(expr)) {
  return extractOptionalChainBinding(expr);
}
```

**ReluxScript Mapping:**

| ReluxScript | Babel | SWC |
|------------|-------|-----|
| `OptionalMemberExpression` | `OptionalMemberExpression` | `OptChainExpr` |

**Implementation:**

Option A: Add as separate expression type
```rust
pub enum Expr {
    // ... existing
    OptionalMember(OptionalMemberExpr),
}

pub struct OptionalMemberExpr {
    pub object: Box<Expr>,
    pub property: String,
    pub computed: bool,
    pub span: Span,
}
```

Option B: Add `optional` flag to existing `MemberExpr`
```rust
pub struct MemberExpr {
    pub object: Box<Expr>,
    pub property: String,
    pub optional: bool,  // NEW: true for `?.`
    pub span: Span,
}
```

**Recommendation:** Option B is simpler and aligns with how SWC handles it.

---

#### 2.2.2 OptionalCallExpression

**Babel AST Structure:**
```javascript
{
  type: "OptionalCallExpression",
  callee: Expression,
  arguments: [Expression],
  optional: boolean
}
```

**Usage:**
```javascript
const result = callback?.();
```

**ReluxScript Implementation:**

Add `optional` flag to `CallExpr`:
```rust
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub type_args: Vec<Type>,
    pub optional: bool,  // NEW: true for `?.()`
    pub span: Span,
}
```

---

### 2.3 Code Generation (per Refinement 2)

**Babel Output:**
```javascript
// Trivial pass-through - native JS syntax
viewModel?.userEmail
callback?.()
```

**Babel Generator:**
```rust
// In gen_member_expr
Expr::Member(mem) => {
    self.gen_expr(&mem.object);
    if mem.optional {
        self.emit("?.");
    } else {
        self.emit(".");
    }
    self.emit(&mem.property);
}
```

**SWC Output:**
```rust
// SWC uses OptChainExpr wrapper
OptChainExpr {
    span: DUMMY_SP,
    optional: true,
    base: Box::new(OptChainBase::Member(MemberExpr {
        span: DUMMY_SP,
        obj: Box::new(view_model),
        prop: MemberProp::Ident(Ident::new("user_email".into(), DUMMY_SP)),
    })),
}
```

**SWC Generator:**
```rust
// In gen_member_expr - check optional flag per Refinement 2
Expr::Member(mem) => {
    if mem.optional {
        // Wrap in OptChainExpr structure
        self.emit("OptChainExpr { span: DUMMY_SP, optional: true, base: Box::new(OptChainBase::Member(MemberExpr { span: DUMMY_SP, obj: Box::new(");
        self.gen_expr(&mem.object);
        self.emit("), prop: MemberProp::Ident(Ident::new(\"");
        self.emit(&mem.property);
        self.emit("\".into(), DUMMY_SP)) })) }");
    } else {
        // Regular member access
        self.gen_expr(&mem.object);
        self.emit(".");
        self.emit(&mem.property);
    }
}
```

**Alternative SWC Pattern (simpler):**
```rust
// Generate the runtime check pattern
if mem.optional {
    self.gen_expr(&mem.object);
    self.emit(".as_ref().map(|__v| __v.");
    self.emit(&mem.property);
    self.emit(".clone())");
} else {
    // Regular access
    self.gen_expr(&mem.object);
    self.emit(".");
    self.emit(&mem.property);
}
```

---

## 3. Template Literal Enhancements

### 3.1 Problem Statement

Minimact extracts template literal content for Razor markdown:
```javascript
const [content] = useRazorMarkdown(`
  # Hello {name}

  Your balance is {balance:C2}
`);
```

ReluxScript needs to access the `quasis` (static parts) and `expressions` (dynamic parts).

### 3.2 Current State

Template literals are partially supported but field access is limited.

### 3.3 Required Enhancements

#### 3.3.1 Field Mappings

```rust
// Quasis - the static string parts
FieldMapping {
    node_type: "TemplateLiteral",
    field_name: "quasis",
    babel_access: "quasis",
    swc_access: "quasis",
    result_type_rs: "Vec<TemplateElement>",
    result_type_babel: "Array<TemplateElement>",
    result_type_swc: "Vec<TplElement>",
    needs_deref: false,
}

// Expressions - the ${...} parts
FieldMapping {
    node_type: "TemplateLiteral",
    field_name: "expressions",
    babel_access: "expressions",
    swc_access: "exprs",
    result_type_rs: "Vec<Expression>",
    result_type_babel: "Array<Expression>",
    result_type_swc: "Vec<Box<Expr>>",
    needs_deref: false,
}
```

#### 3.3.2 TemplateElement Node

```rust
pub struct TemplateElement {
    pub value: TemplateValue,
    pub tail: bool,
    pub span: Span,
}

pub struct TemplateValue {
    pub raw: String,
    pub cooked: Option<String>,
}
```

**Node Mapping:**
```rust
NodeMapping {
    reluxscript: "TemplateElement",
    babel: "TemplateElement",
    swc: "TplElement",
    category: "literal",
}
```

#### 3.3.3 Example Usage in ReluxScript

```reluxscript
pub fn extract_template_content(literal: &TemplateLiteral) -> Str {
    let mut result = "";
    for quasi in literal.quasis {
        result = result + quasi.value.raw;
    }
    return result;
}
```

---

## 4. Array Expression Enhancement

### 4.1 Problem Statement

ReluxScript has `VecInit` for `vec![...]` syntax, but needs `ArrayExpression` for JavaScript array literals `[...]`.

### 4.2 Required Node Type

**Babel AST Structure:**
```javascript
{
  type: "ArrayExpression",
  elements: [Expression | SpreadElement | null]
}
```

**ReluxScript Mapping:**

| ReluxScript | Babel | SWC |
|------------|-------|-----|
| `ArrayExpression` | `ArrayExpression` | `ArrayLit` |

**Implementation:**

The existing `VecInit` can be reused, but we need to ensure proper mapping:

```rust
// In nodes.rs
NodeMapping {
    reluxscript: "ArrayExpression",
    babel: "ArrayExpression",
    swc: "ArrayLit",
    category: "expression",
}

// Field mapping
FieldMapping {
    node_type: "ArrayExpression",
    field_name: "elements",
    babel_access: "elements",
    swc_access: "elems",
    result_type_rs: "Vec<Option<Expression>>",  // null for holes
    result_type_babel: "Array<Expression | null>",
    result_type_swc: "Vec<Option<ExprOrSpread>>",
    needs_deref: false,
}
```

**Code Generation:**

```rust
// Babel
Expr::Array(arr) => {
    self.emit("[");
    for (i, elem) in arr.elements.iter().enumerate() {
        if i > 0 { self.emit(", "); }
        if let Some(e) = elem {
            self.gen_expr(e);
        }
        // null elements create holes: [1,,3]
    }
    self.emit("]");
}

// SWC
Expr::Array(arr) => {
    self.emit("vec![");
    for (i, elem) in arr.elements.iter().enumerate() {
        if i > 0 { self.emit(", "); }
        if let Some(e) = elem {
            self.gen_expr(e);
        } else {
            self.emit("None"); // Or handle differently
        }
    }
    self.emit("]");
}
```

---

## 5. Implementation Checklist

### Week 1: Core Implementation

#### Day 1-2: TypeScript Type System
- [ ] Add TypeScript type variants to `Type` enum
- [ ] Update `type_context.rs` with TS type classification
- [ ] Add type keyword mappings to `nodes.rs`
- [ ] Implement type-to-string conversion helpers
- [ ] Write tests for type recognition

#### Day 3: Interface Support
- [ ] Add `InterfaceDecl` to AST
- [ ] Add `PropertySignature` and related types
- [ ] Update parser for interface syntax
- [ ] Add node mappings for interfaces
- [ ] Add field mappings for interface members
- [ ] Update Babel generator
- [ ] Update SWC generator

#### Day 4: Generic Type Parameters
- [ ] Add `type_args` field to `CallExpr`
- [ ] Update parser for `fn<T>()` syntax
- [ ] Add field mappings for type parameters
- [ ] Test with hook generic patterns

#### Day 5: Optional Chaining
- [ ] Add `optional` field to `MemberExpr`
- [ ] Add `optional` field to `CallExpr`
- [ ] Update parser for `?.` syntax
- [ ] Add node mappings
- [ ] Update Babel generator (pass-through)
- [ ] Update SWC generator (Option handling)

#### Day 6: Template Literals & Arrays
- [ ] Add `TemplateElement` struct
- [ ] Add field mappings for `quasis` and `expressions`
- [ ] Verify `ArrayExpression` mapping
- [ ] Add hole handling for sparse arrays

### Week 2: Testing & Integration

#### Day 7-8: Test Cases
- [ ] Create test files for each new pattern
- [ ] Test TypeScript interface extraction
- [ ] Test generic type parameter extraction
- [ ] Test optional chaining code generation
- [ ] Test template literal field access

#### Day 9-10: Minimact Conversion Pilot
- [ ] Convert `buildMemberPath.cjs` to ReluxScript
- [ ] Convert `extractBinding.cjs` to ReluxScript
- [ ] Convert `findMapExpressions.cjs` to ReluxScript
- [ ] Validate generated Babel output matches original
- [ ] Validate generated SWC output compiles

---

## 6. Risk Assessment

### High Risk
- **TypeScript parser complexity**: May need significant parser changes
- **SWC type representation**: SWC's TypeScript AST differs from Babel's

### Medium Risk
- **Optional chaining in SWC**: Rust Option handling may be verbose
- **Type inference edge cases**: Complex generic types may fail

### Low Risk
- **Template literals**: Straightforward field additions
- **Array expressions**: Already partially implemented

---

## 7. Success Criteria

### Functional Requirements
1. Can parse TypeScript interface declarations
2. Can extract property types from interfaces
3. Can extract generic type parameters from function calls
4. Can handle optional chaining expressions
5. Can access template literal parts

### Performance Requirements
1. No regression in compile times
2. Generated code is not significantly larger

### Compatibility Requirements
1. Generated Babel code matches original plugin behavior
2. Generated SWC code compiles without errors
3. All existing tests continue to pass

---

## 8. Future Considerations

### Beyond Priority 1

These patterns may be needed for full minimact conversion but are lower priority:

1. **Async Generators** (`function*`)
2. **Spread Elements** (`...args`)
3. **Decorators** (`@decorator`)
4. **JSX Namespace** (`<ns:Element>`)
5. **Private Fields** (`#privateField`)

### Long-term Architecture

Consider whether ReluxScript should:
1. **Full TypeScript support**: Become a TypeScript-aware compiler
2. **TypeScript stripping**: Require pre-processing with `@babel/preset-typescript`
3. **Hybrid approach**: Support common patterns, delegate complex types

**Recommendation**: Start with Option 3 (hybrid) - support the patterns actually used by minimact, and expand based on real needs.

---

## 9. Appendix

### A. Example Minimact Code to Convert

```javascript
// hooks.cjs - extractUseState function
function extractUseState(path, component, hookType) {
  const parent = path.parent;

  if (!t.isVariableDeclarator(parent)) return;
  if (!t.isArrayPattern(parent.id)) return;

  const [stateVar, setterVar] = parent.id.elements;
  const initialValue = path.node.arguments[0];

  // Check for generic type parameter
  let explicitType = null;
  if (path.node.typeParameters && path.node.typeParameters.params.length > 0) {
    const typeParam = path.node.typeParameters.params[0];
    explicitType = tsTypeToCSharpType(typeParam);
  }

  const stateInfo = {
    name: stateVar.name,
    setter: setterVar ? setterVar.name : null,
    initialValue: generateCSharpExpression(initialValue),
    type: explicitType || inferType(initialValue)
  };

  component.useState.push(stateInfo);
}
```

### B. Equivalent ReluxScript (Target)

```reluxscript
/// Extract useState hook
pub fn extract_use_state(path: &NodePath, component: &mut Component, hook_type: Str) {
    let parent = path.parent;

    if !matches!(parent, VariableDeclarator) {
        return;
    }

    let declarator = parent;
    if !matches!(declarator.id, ArrayPattern) {
        return;
    }

    let elements = declarator.id.elements;
    let state_var = elements[0];
    let setter_var = if elements.len() > 1 { elements[1] } else { None };
    let initial_value = path.node.arguments[0];

    // Check for generic type parameter
    let explicit_type = if path.node.typeArguments.len() > 0 {
        let type_param = path.node.typeArguments[0];
        ts_type_to_csharp(type_param)
    } else {
        None
    };

    let state_info = StateInfo {
        name: state_var.name.clone(),
        setter: setter_var.map(|s| s.name.clone()),
        initial_value: generate_csharp_expression(initial_value),
        state_type: explicit_type.unwrap_or_else(|| infer_type(initial_value)),
    };

    component.use_state.push(state_info);
}
```

---

## 10. References

- [Babel AST Spec](https://github.com/babel/babel/blob/main/packages/babel-parser/ast/spec.md)
- [SWC AST Types](https://rustdoc.swc.rs/swc_ecma_ast/)
- [TypeScript AST Viewer](https://ts-ast-viewer.com/)
- [ReluxScript Language Spec](./reluxscript-language-spec.md)
