# SWC Decorator Implementation Plan

**Status:** Design Phase
**Goal:** Add a dedicated SWC decoration phase between parsing and codegen to resolve semantic mismatches between ReluxScript's Babel-like API and SWC's actual AST structure.

---

## 🎯 Problem Statement

ReluxScript provides a **Babel-like unified AST API** (e.g., `Expression::Identifier`, `member.object`, `member.property`, `identifier.name`) but must compile to **SWC's actual Rust AST** which has:

- Different enum types: `MemberProp` instead of `Expr` for properties
- Different field names: `obj`/`prop`/`sym` instead of `object`/`property`/`name`
- Different wrapper types: `Box<Expr>` requiring `.as_ref()` in patterns
- Different string types: `JsWord` requiring `&*` dereference

**Current Issue:** SWC codegen does ad-hoc semantic analysis during emission, leading to:
- Complex, hard-to-follow codegen logic
- Bugs from missing edge cases
- Duplication of type inference logic
- No single source of truth for semantic mappings

**Solution:** Insert a **SwcDecorator** phase that annotates the ReluxScript AST with SWC-specific semantic metadata **before** codegen.

---

## 🏗️ Architecture

### Pipeline Flow

```
┌─────────┐     ┌──────────────┐     ┌──────────────┐     ┌─────────────┐
│ Parser  │ --> │   Semantic   │ --> │     SWC      │ --> │     SWC     │
│  (.lux) │     │   Analysis   │     │  Decorator   │     │   Codegen   │
└─────────┘     └──────────────┘     └──────────────┘     └─────────────┘
                                              │
                                              v
                                      Annotated AST
                                      with SWC metadata
```

### Decorator Responsibilities

The SwcDecorator phase:

1. **Walks the ReluxScript AST** (visitor pattern)
2. **Infers SWC semantics** for each node based on:
   - Field type mappings (object → obj: Box<Expr>)
   - Type context (what type is being matched?)
   - Access patterns (read vs write vs pattern match)
3. **Annotates nodes** with SWC-specific metadata
4. **Returns enriched AST** to codegen

### Codegen Simplification

After decoration, SWC codegen becomes **trivial**:
- Read metadata from nodes
- Emit based on metadata
- No type inference
- No complex lookups
- Just translation

---

## 📊 Metadata Design

### Core Metadata Types

```rust
/// SWC-specific metadata attached to AST nodes
pub mod swc_metadata {
    use crate::lexer::Span;

    /// Type-safe SWC node kinds (prevents typos in string-based variants)
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum SwcNodeKind {
        // Expressions
        ExprIdent,
        ExprMember,
        ExprCall,
        ExprBin,
        ExprUnary,
        ExprLit,

        // Member property variants
        MemberPropIdent,
        MemberPropComputed,

        // Callee variants
        CalleeExpr,
        CalleeSuper,
        CalleeImport,

        // Patterns
        PatIdent,
        PatArray,
        PatObject,

        // Other common types
        IdentName,
        JsWord,
    }

    impl std::fmt::Display for SwcNodeKind {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            match self {
                SwcNodeKind::ExprIdent => write!(f, "Expr::Ident"),
                SwcNodeKind::ExprMember => write!(f, "Expr::Member"),
                SwcNodeKind::MemberPropIdent => write!(f, "MemberProp::Ident"),
                SwcNodeKind::CalleeExpr => write!(f, "Callee::Expr"),
                // ... etc
                _ => write!(f, "{:?}", self),
            }
        }
    }

    /// Metadata for pattern matching
    #[derive(Debug, Clone)]
    pub struct SwcPatternMetadata {
        /// The SWC type/variant to match against (type-safe)
        pub target_kind: SwcNodeKind,

        /// Whether the value needs unwrapping (Box, Option, etc.)
        pub unwrap_strategy: UnwrapStrategy,

        /// Inner pattern metadata (for nested patterns)
        pub inner: Option<Box<SwcPatternMetadata>>,

        /// Source location for error reporting
        pub span: Option<Span>,

        /// Original ReluxScript pattern (for diagnostics)
        pub source_pattern: Option<String>,
    }

    /// Metadata for field access
    #[derive(Debug, Clone)]
    pub struct SwcFieldMetadata {
        /// The actual SWC field name
        /// e.g., "obj" instead of "object"
        pub swc_field_name: String,

        /// How to access this field
        pub accessor: FieldAccessor,

        /// Type of the field value in SWC
        pub field_type: String,

        /// Original ReluxScript type (for diagnostics/debug)
        pub source_type: Option<String>,

        /// Source location for error reporting
        pub span: Option<Span>,
    }

    /// Metadata for identifier access
    #[derive(Debug, Clone)]
    pub struct SwcIdentifierMetadata {
        /// Whether this needs sym vs name
        pub use_sym: bool,

        /// Deref pattern needed
        /// e.g., "&*ident.sym" vs just "ident.name"
        pub deref_pattern: Option<String>,
    }

    /// Metadata for expression evaluation context
    #[derive(Debug, Clone)]
    pub struct SwcExprMetadata {
        /// Type of this expression in SWC
        pub swc_type: String,

        /// Whether this is a Box<T>
        pub is_boxed: bool,

        /// Whether this is an Option<T>
        pub is_optional: bool,
    }

    #[derive(Debug, Clone)]
    pub enum UnwrapStrategy {
        /// Use .as_ref() for Box<T>
        AsRef,
        /// Use * dereference
        Deref,
        /// Use & reference
        Ref,
        /// No unwrapping needed
        None,
    }

    #[derive(Debug, Clone)]
    pub enum FieldAccessor {
        /// Direct field access: member.field
        Direct,
        /// Boxed field, read context: member.field.as_ref()
        BoxedAsRef,
        /// Boxed field, pattern context: &*member.field or member.field.as_ref()
        BoxedPattern,
        /// Enum field, needs different type: member.prop (MemberProp, not Expr)
        EnumField { enum_name: String },
    }
}
```

### AST Node Extensions

Extend existing AST node structs with optional metadata:

```rust
// In src/parser/mod.rs

pub struct Pattern {
    pub kind: PatternKind,
    // ... existing fields ...

    /// SWC-specific metadata (populated by SwcDecorator)
    pub swc_metadata: Option<SwcPatternMetadata>,
}

pub struct Expr {
    pub kind: ExprKind,
    // ... existing fields ...

    /// SWC-specific metadata
    pub swc_metadata: Option<SwcExprMetadata>,
}

pub struct IfStmt {
    pub condition: Box<Expr>,
    pub pattern: Option<Pattern>,
    pub then_branch: Block,
    pub else_branch: Option<Block>,

    /// SWC-specific metadata for if-let patterns
    pub swc_metadata: Option<SwcIfLetMetadata>,
}

#[derive(Debug, Clone)]
pub struct SwcIfLetMetadata {
    /// Type of the condition expression in SWC
    pub condition_swc_type: String,

    /// Translated pattern to emit
    pub pattern_translation: String,
}
```

---

## 🔨 Implementation Phases

### Phase 1: Foundation (Minimal Viable Decorator)

**Goal:** Get basic pattern decoration working for the console.log test case

#### 1.1 Create Metadata Module

**File:** `src/codegen/swc_metadata.rs`

```rust
//! SWC-specific metadata types
//! Populated by SwcDecorator, consumed by SwcCodegen

pub struct SwcPatternMetadata { /* ... */ }
pub struct SwcFieldMetadata { /* ... */ }
pub struct SwcExprMetadata { /* ... */ }
pub enum UnwrapStrategy { /* ... */ }
pub enum FieldAccessor { /* ... */ }
```

#### 1.2 Add Metadata Fields to AST

**Files:** `src/parser/mod.rs`

- Add `swc_metadata: Option<SwcPatternMetadata>` to `Pattern`
- Add `swc_metadata: Option<SwcExprMetadata>` to `Expr`
- Add `swc_metadata: Option<SwcFieldMetadata>` to `MemberExpr`

#### 1.3 Create SwcDecorator Struct

**File:** `src/codegen/swc_decorator.rs`

```rust
use crate::parser::*;
use crate::ast_schema::{ASTSchema, FieldAccessResolver};
use super::swc_metadata::*;

pub struct SwcDecorator {
    /// AST schema with SWC type information
    schema: ASTSchema,

    /// Field access resolver
    resolver: FieldAccessResolver,

    /// Type environment for tracking variable types
    type_env: TypeEnvironment,
}

impl SwcDecorator {
    pub fn new() -> Self {
        Self {
            schema: ASTSchema::swc_schema(),
            resolver: FieldAccessResolver::new(ASTSchema::swc_schema()),
            type_env: TypeEnvironment::new(),
        }
    }

    /// Decorate a program with SWC metadata
    pub fn decorate(&mut self, program: &mut Program) {
        for item in &mut program.items {
            match item {
                Item::Plugin(plugin) => self.visit_plugin(plugin),
                Item::Writer(writer) => self.visit_writer(writer),
                _ => {}
            }
        }
    }
}
```

#### 1.4 Implement Visitor Pattern

```rust
impl SwcDecorator {
    fn visit_plugin(&mut self, plugin: &mut Plugin) {
        for method in &mut plugin.methods {
            self.visit_function(method);
        }
    }

    fn visit_function(&mut self, func: &mut Function) {
        // Track parameter types
        for param in &func.params {
            self.register_param_type(param);
        }

        self.visit_block(&mut func.body);
    }

    fn visit_block(&mut self, block: &mut Block) {
        for stmt in &mut block.stmts {
            self.visit_stmt(stmt);
        }
    }

    fn visit_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::If(if_stmt) => self.visit_if_stmt(if_stmt),
            Stmt::Let(let_stmt) => self.visit_let_stmt(let_stmt),
            Stmt::Expr(expr) => self.visit_expr(expr),
            // ... other statements
            _ => {}
        }
    }
}
```

#### 1.5 Core Decoration Logic

**Pattern Decoration:**

```rust
impl SwcDecorator {
    fn visit_if_stmt(&mut self, if_stmt: &mut IfStmt) {
        // Decorate the condition expression first
        self.visit_expr(&mut if_stmt.condition);

        // If this is an if-let, decorate the pattern
        if let Some(ref mut pattern) = if_stmt.pattern {
            // Infer the type of the condition
            let cond_type = self.infer_expr_type(&if_stmt.condition);

            // Decorate pattern based on what type it's matching
            self.decorate_pattern(pattern, &cond_type);
        }

        // Visit branches
        self.visit_block(&mut if_stmt.then_branch);
        if let Some(ref mut else_branch) = if_stmt.else_branch {
            self.visit_block(else_branch);
        }
    }

    fn decorate_pattern(&mut self, pattern: &mut Pattern, expected_type: &str) {
        match &mut pattern.kind {
            PatternKind::Variant { name, inner } => {
                // Special case: Expression::Identifier matching against MemberProp
                if name == "Expression::Identifier" && expected_type == "MemberProp" {
                    pattern.swc_metadata = Some(SwcPatternMetadata {
                        target_pattern: "MemberProp::Ident".to_string(),
                        unwrap_strategy: UnwrapStrategy::None,
                        inner: inner.as_mut().map(|p| {
                            Box::new(SwcPatternMetadata {
                                target_pattern: "IdentName".to_string(),
                                unwrap_strategy: UnwrapStrategy::None,
                                inner: None,
                            })
                        }),
                    });
                    return;
                }

                // Standard variant pattern translation
                let swc_pattern = self.translate_pattern_variant(name);
                pattern.swc_metadata = Some(SwcPatternMetadata {
                    target_pattern: swc_pattern,
                    unwrap_strategy: self.determine_unwrap_strategy(expected_type),
                    inner: None,
                });
            }
            _ => {}
        }
    }

    fn translate_pattern_variant(&self, relux_name: &str) -> String {
        // Use existing mapping tables
        if let Some(mapping) = get_node_mapping(relux_name) {
            mapping.swc_pattern.to_string()
        } else {
            // Fallback for variants like Expression::Identifier
            if relux_name.contains("::") {
                let parts: Vec<&str> = relux_name.split("::").collect();
                let enum_name = reluxscript_to_swc_type(parts[0]);
                let variant = reluxscript_to_swc_type(parts[1]);
                format!("{}::{}", enum_name, variant)
            } else {
                relux_name.to_string()
            }
        }
    }
}
```

**Field Access Decoration:**

```rust
impl SwcDecorator {
    fn visit_expr(&mut self, expr: &mut Expr) {
        match &mut expr.kind {
            ExprKind::Member(mem) => {
                // Decorate base expression
                self.visit_expr(&mut mem.object);

                // Get type of base object
                let base_type = self.infer_expr_type(&mem.object);

                // Resolve field access semantics
                if let Some(resolved) = self.resolver.resolve(
                    &base_type,
                    &mem.property,
                    AccessContext::Read,
                ) {
                    mem.swc_metadata = Some(SwcFieldMetadata {
                        swc_field_name: resolved.field_name.clone(),
                        accessor: match resolved.accessor {
                            FieldAccessor::Boxed { .. } =>
                                FieldAccessor::BoxedAsRef,
                            FieldAccessor::EnumVariant { enum_name, .. } =>
                                FieldAccessor::EnumField { enum_name },
                            _ => FieldAccessor::Direct,
                        },
                        field_type: self.get_field_type(&base_type, &mem.property),
                    });
                }
            }

            ExprKind::Unary(un) if un.op == UnaryOp::Deref => {
                // Special handling for dereference of Box fields
                self.visit_expr(&mut un.operand);

                // Check if operand is a member access of a Box field
                if let ExprKind::Member(mem) = &un.operand.kind {
                    if let Some(meta) = &mem.swc_metadata {
                        if matches!(meta.accessor, FieldAccessor::EnumField { .. }) {
                            // Dereferencing enum field: use & instead of *
                            un.swc_metadata = Some(SwcUnaryMetadata {
                                override_op: Some("&".to_string()),
                            });
                        }
                    }
                }
            }

            // Other expressions...
            _ => {
                // Default: visit children
                expr.visit_children_mut(self);
            }
        }
    }
}
```

#### 1.6 Type Inference

```rust
impl SwcDecorator {
    fn infer_expr_type(&self, expr: &Expr) -> String {
        match &expr.kind {
            ExprKind::Ident(ident) => {
                // Look up in type environment
                self.type_env.lookup(&ident.name)
                    .map(|t| t.swc_type.clone())
                    .unwrap_or_else(|| "Unknown".to_string())
            }

            ExprKind::Member(mem) => {
                // Get base type
                let base_type = self.infer_expr_type(&mem.object);

                // Look up field type
                if let Some(field_info) = self.schema.get_field(&base_type, &mem.property) {
                    match &field_info.field_type {
                        FieldType::Direct(t) => t.clone(),
                        FieldType::Boxed(t) => t.clone(),
                        FieldType::Enum(enum_info) => enum_info.name.clone(),
                        FieldType::Optional(inner) => {
                            // Unwrap Option<T> to get T
                            "Option<...>".to_string() // Simplified
                        }
                    }
                } else {
                    "Unknown".to_string()
                }
            }

            ExprKind::Unary(un) if un.op == UnaryOp::Deref => {
                // Dereferencing Box<T> gives T
                let operand_type = self.infer_expr_type(&un.operand);
                // Strip Box<...> wrapper
                operand_type // Simplified
            }

            _ => "Unknown".to_string()
        }
    }

    fn register_param_type(&mut self, param: &FunctionParam) {
        // Register visitor method parameters
        // e.g., visit_call_expression(node: &mut CallExpression)
        if let Some(type_ann) = &param.type_annotation {
            let swc_type = self.map_reluxscript_type_to_swc(type_ann);
            self.type_env.define(&param.name, TypeContext {
                swc_type,
                babel_type: "...".to_string(),
            });
        }
    }
}
```

---

### Phase 2: Codegen Integration

**Goal:** Simplify SWC codegen to use metadata instead of inline inference

#### 2.1 Update Codegen to Read Metadata

**File:** `src/codegen/swc.rs`

```rust
fn gen_if_let_stmt(&mut self, if_stmt: &IfStmt, pattern: &Pattern) {
    self.emit_indent();
    self.emit("if let ");

    // Use decorated metadata if available
    if let Some(meta) = &pattern.swc_metadata {
        self.emit(&meta.target_pattern);
        // Emit inner pattern binding if present
        if let PatternKind::Variant { inner: Some(inner_pat), .. } = &pattern.kind {
            self.emit("(");
            self.gen_pattern(inner_pat);
            self.emit(")");
        }
    } else {
        // Fallback to old logic (during transition)
        self.gen_pattern(pattern);
    }

    self.emit(" = ");
    self.gen_expr(&if_stmt.condition);
    self.emit(" {\n");

    // ... rest of function
}
```

#### 2.2 Simplify Field Access Generation

```rust
fn gen_member_expr(&mut self, mem: &MemberExpr) {
    self.gen_expr(&mem.object);
    self.emit(".");

    // Use metadata if available
    if let Some(meta) = &mem.swc_metadata {
        self.emit(&meta.swc_field_name);

        match &meta.accessor {
            FieldAccessor::BoxedAsRef => {
                self.emit(".as_ref()");
            }
            FieldAccessor::EnumField { .. } => {
                // Enum field, no special accessor needed
            }
            FieldAccessor::Direct => {
                // Direct access, nothing extra
            }
            _ => {}
        }
    } else {
        // Fallback
        let swc_field = self.map_field_name(&mem.property);
        self.emit(&swc_field);
    }
}
```

#### 2.3 Simplify Identifier Access

```rust
fn gen_binary_expr(&mut self, bin: &BinaryExpr) {
    // For identifier.name == "string" comparisons
    if bin.op == BinaryOp::Eq {
        if let ExprKind::Member(mem) = &bin.left.kind {
            if mem.property == "name" {
                if let ExprKind::Ident(_) = &mem.object.kind {
                    // This is identifier.name - needs &*ident.sym in SWC
                    self.emit("&*");
                    self.gen_expr(&mem.object);
                    self.emit(".sym");
                    self.emit(" == ");
                    self.gen_expr(&bin.right);
                    return;
                }
            }
        }
    }

    // Default binary expression generation
    // ...
}
```

---

### Phase 3: Test & Validate

#### 3.1 Integration Point

**File:** `src/codegen/mod.rs`

```rust
pub fn generate(program: &Program, target: Target) -> Result<GeneratedCode, CodegenError> {
    match target {
        Target::Babel => {
            let mut gen = BabelGenerator::new();
            Ok(gen.generate(program))
        }
        Target::Swc => {
            // Clone program to avoid mutating original
            let mut program = program.clone();

            // DECORATION PHASE
            let mut decorator = SwcDecorator::new();
            decorator.decorate(&mut program);

            // CODEGEN PHASE
            let mut gen = SwcGenerator::new();
            Ok(gen.generate(&program))
        }
    }
}
```

#### 3.2 Test Console.log Integration Test

```bash
cd source
cargo run -- build --target swc tests/integration/console_remover/plugin.lux -o /tmp/swc_decorated
```

**Expected output (lib.rs):**

```rust
fn visit_mut_call_expr(&mut self, n: &mut CallExpr) {
    if let Callee::Expr(__callee_expr) = &n.callee {
        if let Expr::Member(ref member) = __callee_expr.as_ref() {
            if let Expr::Ident(ref obj) = member.obj.as_ref() {
                if let MemberProp::Ident(ref prop) = &member.prop {  // ✅ MemberProp::Ident
                    if &*obj.sym == "console" && &*prop.sym == "log" {  // ✅ sym with &*
                        // Remove or mark for deletion
                    }
                }
            }
        }
    }
}
```

#### 3.3 Add Debug Logging

```rust
impl SwcDecorator {
    pub fn decorate(&mut self, program: &mut Program) {
        #[cfg(debug_assertions)]
        eprintln!("[SwcDecorator] Starting decoration...");

        for item in &mut program.items {
            self.visit_item(item);
        }

        #[cfg(debug_assertions)]
        eprintln!("[SwcDecorator] Decoration complete");
    }

    fn decorate_pattern(&mut self, pattern: &mut Pattern, expected_type: &str) {
        #[cfg(debug_assertions)]
        eprintln!("[SwcDecorator] Decorating pattern: {:?} for type: {}",
                  pattern.kind, expected_type);

        // ... decoration logic ...

        #[cfg(debug_assertions)]
        eprintln!("[SwcDecorator] Pattern decorated with: {:?}", pattern.swc_metadata);
    }
}
```

---

### Phase 4: Expand Coverage

#### 4.1 Additional Pattern Cases

- `Callee::MemberExpression` → nested desugaring
- `Some(x)` / `None` → Option patterns
- `Ok(x)` / `Err(e)` → Result patterns
- Nested patterns with multiple levels

#### 4.2 Field Access Cases

- Box fields in different contexts (read, write, pattern)
- Optional fields (Option<T>)
- Nested member access (a.b.c.d)
- Method calls on members

#### 4.3 String Comparison Cases

- `identifier.name == "foo"` → `&*ident.sym == "foo"`
- `member.property.name` → needs double unwrap
- String methods (starts_with, contains, etc.)

---

## 📐 Design Decisions

### Why Clone the Program?

```rust
let mut program = program.clone();
decorator.decorate(&mut program);
```

**Options:**

1. **Clone before decorating** (current plan)
   - ✅ Keeps original AST immutable
   - ✅ Babel codegen unaffected
   - ❌ Memory overhead for large programs

2. **Mutate in-place, decoration is reversible**
   - ✅ No memory overhead
   - ❌ Complex cleanup logic
   - ❌ Risk of leaking SWC metadata to Babel

3. **Separate decorated AST type**
   - ✅ Type-safe separation
   - ❌ Massive refactor
   - ❌ Code duplication

**Decision:** Start with #1 (clone), optimize later if needed.

### Metadata Ownership

Should metadata be:

1. **`Option<Box<Metadata>>`** - Heap allocated, nullable
2. **`Option<Metadata>`** - Stack allocated, nullable
3. **Always present with a `None` variant in the enum**

**Decision:** Use `Option<Metadata>` for now. Most nodes won't have metadata, and stack allocation is fine for small structs.

### Type Environment Sharing

Should SwcDecorator reuse the existing `TypeEnvironment` from `type_context.rs` or create its own?

**Decision:** Reuse existing. It already has flow-sensitive typing and type unwrapping logic.

---

## 🧪 Testing Strategy

### Unit Tests

**File:** `src/codegen/swc_decorator_tests.rs`

```rust
#[test]
fn test_decorate_member_prop_pattern() {
    let mut program = parse("
        plugin Test {
            fn visit_call_expression(node: &mut CallExpression) {
                if let Expression::Identifier(ref prop) = *member.property {
                    // ...
                }
            }
        }
    ");

    let mut decorator = SwcDecorator::new();
    decorator.decorate(&mut program);

    // Find the if-let pattern
    let pattern = /* ... extract pattern ... */;

    assert!(pattern.swc_metadata.is_some());
    let meta = pattern.swc_metadata.unwrap();
    assert_eq!(meta.target_pattern, "MemberProp::Ident");
}
```

### Integration Tests

**File:** `tests/swc_decorator_integration.rs`

```rust
#[test]
fn test_console_log_removal() {
    let source = include_str!("integration/console_remover/plugin.lux");
    let output = compile(source, Target::Swc);

    // Check that output contains correct SWC patterns
    assert!(output.contains("MemberProp::Ident"));
    assert!(output.contains("&*obj.sym"));
    assert!(output.contains("member.obj.as_ref()"));
}
```

### Snapshot Tests

Use `insta` for snapshot testing:

```rust
#[test]
fn test_decorated_ast_snapshot() {
    let mut program = parse("...");
    decorator.decorate(&mut program);

    insta::assert_debug_snapshot!(program);
}
```

---

## 🚀 Rollout Plan

### Week 1: Foundation
- [ ] Create `swc_metadata.rs` module with `SwcNodeKind` enum
- [ ] Add metadata fields to AST nodes
- [ ] Create `SwcDecorator` skeleton
- [ ] Implement basic visitor pattern
- [ ] Add fallback mode config for gradual rollout

### Week 2: Core Logic
- [ ] Implement pattern decoration
- [ ] Implement field access decoration
- [ ] Implement type inference
- [ ] Add debug logging

### Week 3: Integration
- [ ] Integrate decorator into codegen pipeline
- [ ] Update SWC codegen to use metadata
- [ ] Test console.log integration test
- [ ] Fix any issues

### Week 4: Expansion
- [ ] Add more pattern cases
- [ ] Add more field access cases
- [ ] Add string comparison decoration
- [ ] Write comprehensive tests

### Week 5: Cleanup
- [ ] Remove old inline inference from codegen
- [ ] Add documentation
- [ ] Performance testing
- [ ] Final polish

---

## 📚 Future Enhancements

### Phase 5+: Advanced Features

1. **Callee desugaring**
   - `Callee::MemberExpression` → nested if-let

2. **Generic type handling**
   - `Option<T>`, `Result<T, E>`, `Vec<T>`

3. **Method call decoration**
   - `.push()`, `.clone()`, `.as_str()`, etc.

4. **String method translation**
   - `.starts_with()`, `.contains()`, etc.

5. **Control flow decoration**
   - `ctx.remove()` → different strategies per target

6. **Optimization hints**
   - Mark hot paths, suggest inlining, etc.

### Babel Decorator (Maybe)

If Babel codegen also needs decoration:

```rust
pub struct BabelDecorator {
    // Simpler than SWC - Babel is closer to ReluxScript
}
```

Likely not needed since Babel AST is already similar to ReluxScript's unified AST.

---

## 🔍 Design Decisions & Solutions

### 1. Fallback Strategy for Partial Decoration

**Question:** What if decorator fails on some nodes?

**Solution:** Use a configurable fallback mode:

```rust
pub struct CodegenConfig {
    pub allow_fallback: bool,  // true during dev, false in production
}

impl SwcGenerator {
    fn gen_pattern(&mut self, pattern: &Pattern) {
        if let Some(meta) = &pattern.swc_metadata {
            // Use decorated metadata
            self.emit(&meta.target_kind.to_string());
        } else if self.config.allow_fallback {
            // Fallback to old logic (transitional)
            self.gen_pattern_fallback(pattern);
        } else {
            // Strict mode: fail fast
            panic!("Missing SWC decoration for pattern: {:?}", pattern);
        }
    }
}
```

During rollout: `allow_fallback = true`
After full coverage: `allow_fallback = false`

### 2. Feature Flags for Debug Output

Use conditional compilation for decorator debugging:

```rust
#[cfg(feature = "swc-decorator-debug")]
eprintln!("[SwcDecorator] Decorating {:?} → {:?}", node, metadata);
```

Enable with: `cargo build --features swc-decorator-debug`

### 3. Metadata Serialization

**Decision:** Not needed initially, but structure supports it:
- All metadata types derive `Clone` and `Debug`
- Can add `Serialize`/`Deserialize` later for IDE/LSP support

### 4. Performance Optimization

**Strategies:**
1. **Memoization:** Cache schema queries for repeated type lookups
2. **Lazy decoration:** Only decorate nodes that codegen will visit
3. **Avoid cloning:** Use `Arc` for shared metadata if profiling shows overhead

**Initial approach:** Simple cloning, optimize if benchmarks show >10% overhead

### 5. Error Handling

**Decision:** Decorator should not fail silently
- Return `Result<(), DecoratorError>` for reportable errors
- Panic on internal bugs (invalid AST structure)
- Use fallback mode during development

---

## 📖 References

- **Decorator Pattern:** https://refactoring.guru/design-patterns/decorator
- **Visitor Pattern:** https://rust-unofficial.github.io/patterns/patterns/behavioural/visitor.html
- **AST Transformations:** https://github.com/rust-lang/rust/tree/master/compiler/rustc_ast_passes
- **SWC AST:** https://rustdoc.swc.rs/swc_ecma_ast/

---

## 🎯 Success Criteria

The decorator is successful when:

1. ✅ Console.log test compiles to correct SWC code
2. ✅ All pattern matching generates correct SWC patterns
3. ✅ All field access uses correct SWC field names
4. ✅ String comparisons use `&*sym` correctly
5. ✅ SWC codegen is 50%+ simpler (measured by LoC)
6. ✅ No inline type inference in codegen
7. ✅ All integration tests pass
8. ✅ Performance overhead < 10%

---

**Status:** Ready for implementation
**Next Step:** Create `swc_metadata.rs` and add metadata fields to AST
