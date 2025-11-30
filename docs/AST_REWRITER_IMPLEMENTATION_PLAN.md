# AST Rewriter Implementation Plan

**Status:** Planning Phase
**Goal:** Add a rewriting stage between decoration and codegen that transforms DecoratedAST → DecoratedAST, moving all semantic transformations out of codegen

---

## 🎯 MOTIVATION

### The Problem

Currently, the SWC codegen is "mostly dumb" but still contains transformation logic:

```rust
// In gen_if_let_stmt() - Lines 1723-1770 of swc_backup.rs
if pattern_str == "Callee::MemberExpression" {
    // Hardcoded desugaring: generate nested if-let chains
    self.emit("if let Callee::Expr(__callee_expr) = &node.callee {");
    self.emit("if let Expr::Member(member) = __callee_expr.as_ref() {");
    // ...
}
```

This violates our principle: **Codegen should be dumb - just emit what it sees.**

### The Solution

Add a **rewriting stage** that transforms the DecoratedAST before codegen:

```
Parser AST → Decorator (annotates) → Rewriter (transforms) → Codegen (emits)
              ↓                       ↓                          ↓
         Add metadata           Transform structure        Just emit text
```

---

## 📐 ARCHITECTURE

### Pipeline Overview

```rust
// OLD PIPELINE (2 stages)
let decorated = decorator.decorate_program(program);
let code = generator.generate_decorated(&decorated);

// NEW PIPELINE (3 stages)
let decorated = decorator.decorate_program(program);
let rewritten = rewriter.rewrite_program(decorated);  // ← NEW!
let code = generator.generate_decorated(&rewritten);
```

### Stage Responsibilities

| Stage | Input | Output | Responsibility |
|-------|-------|--------|----------------|
| **Decorator** | Parser AST | DecoratedAST with metadata | Type resolution, field mapping, pattern mapping |
| **Rewriter** | DecoratedAST | Transformed DecoratedAST | Desugaring, unwrapping, pattern expansion |
| **Codegen** | DecoratedAST | Rust source code | String emission only |

### Key Principle

**The Rewriter operates on DecoratedAST → DecoratedAST**

- Input: DecoratedAST with metadata
- Output: DecoratedAST with metadata (possibly more nodes)
- The output is still semantically annotated
- Codegen receives "ready-to-emit" AST

---

## 🔧 CORE TRANSFORMATIONS

### 1. Pattern Desugaring

**Problem:** ReluxScript allows simplified patterns that don't exist in SWC

**Example:**
```rust
// ReluxScript (simplified)
if let Callee::MemberExpression(member) = node.callee { ... }

// SWC (actual structure)
if let Callee::Expr(__expr) = &node.callee {
    if let Expr::Member(member) = __expr.as_ref() {
        ...
    }
}
```

**Rewriter Transformation:**

```rust
impl SwcRewriter {
    fn rewrite_pattern(&mut self, pattern: DecoratedPattern) -> DecoratedPattern {
        // Check if pattern needs desugaring
        if pattern.metadata.swc_pattern == "Callee::Member" {
            // This doesn't exist in SWC - desugar to nested patterns
            return self.desugar_callee_member_pattern(pattern);
        }
        pattern
    }

    fn desugar_callee_member_pattern(&mut self, pattern: DecoratedPattern) -> DecoratedPattern {
        // Transform:
        //   Callee::MemberExpression(inner)
        // Into:
        //   Callee::Expr(__expr) with nested Expr::Member(inner)

        DecoratedPattern {
            kind: DecoratedPatternKind::Variant {
                name: "Callee::Expr".into(),
                inner: Some(Box::new(DecoratedPattern {
                    kind: DecoratedPatternKind::Ident("__callee_expr".into()),
                    metadata: SwcPatternMetadata {
                        swc_pattern: "__callee_expr".into(),
                        unwrap_strategy: UnwrapStrategy::None,
                        // ...
                    }
                }))
            },
            metadata: SwcPatternMetadata {
                swc_pattern: "Callee::Expr(__callee_expr)".into(),
                // ...
            }
        }
    }
}
```

**What moves from codegen:**
- `gen_if_let_stmt()` lines 1723-1770: Pattern desugaring logic
- `gen_swc_pattern_check()` lines 2987-3040: Enum variant checking

**Result:** Codegen just emits `if let {pattern} = {expr} { ... }` - no special cases!

---

### 2. Nested Member Unwrapping

**Problem:** ReluxScript `node.callee.name` doesn't map directly to SWC

**Example:**
```rust
// ReluxScript
if obj.name == "console" { ... }

// SWC (when obj is Ident)
if &*obj.sym == "console" { ... }
```

**Current State:**
- Decorator provides `field_metadata.swc_field_name = "sym"`
- Codegen emits `&*obj.sym` based on metadata

**Issue:** For nested access like `node.callee.name`:
```rust
// ReluxScript
node.callee.name

// SWC (requires unwrapping)
match &node.callee {
    Callee::Expr(expr) => {
        match expr.as_ref() {
            Expr::Ident(ident) => &*ident.sym,
            _ => panic!()
        }
    }
    _ => panic!()
}
```

**Rewriter Transformation:**

```rust
impl SwcRewriter {
    fn rewrite_nested_member_access(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // Check if this is obj.field.subfield
        if let DecoratedExprKind::Member { object, field_metadata, .. } = &expr.kind {
            if let DecoratedExprKind::Member { .. } = &object.kind {
                // This is nested member access - needs unwrapping
                return self.generate_unwrap_chain(expr);
            }
        }
        expr
    }

    fn generate_unwrap_chain(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // Transform: node.callee.name
        // Into: match &node.callee { Callee::Expr(e) => match e.as_ref() { Expr::Ident(i) => &*i.sym } }

        // Extract chain: [node, callee, name]
        let chain = self.extract_member_chain(&expr);

        // Build match expression from inside-out
        let mut current = self.build_final_access(&chain);

        for (i, member) in chain.iter().enumerate().rev().skip(1) {
            current = self.wrap_in_match(member, current);
        }

        current
    }

    fn wrap_in_match(&mut self, member: &MemberAccess, inner: DecoratedExpr) -> DecoratedExpr {
        // Generate: match &obj.field { Pattern(binding) => inner }
        DecoratedExpr {
            kind: DecoratedExprKind::Match(Box::new(DecoratedMatchExpr {
                scrutinee: member.object.clone(),
                arms: vec![DecoratedMatchArm {
                    pattern: self.create_unwrap_pattern(&member.field_metadata),
                    guard: None,
                    body: inner,
                }],
            })),
            metadata: SwcExprMetadata {
                swc_type: member.field_metadata.field_type.clone(),
                // ...
            }
        }
    }
}
```

**What moves from codegen:**
- `gen_if_let_stmt()` lines 1771-1808: Nested member unwrapping
- All the "intermediate variable" generation logic

**Result:** Codegen receives a match expression - just emits it!

---

### 3. Field Replacements

**Problem:** Some field accesses need to be rewritten (e.g., `self.builder` → `self` in writers)

**Example:**
```rust
// ReluxScript (writer)
self.builder.write("foo")

// SWC (actual)
self.write("foo")
```

**Current State:**
- Lines 2533-2554 in swc.rs: Hardcoded checks in member expression generation

**Rewriter Transformation:**

```rust
impl SwcRewriter {
    fn rewrite_field_replacements(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // Check for special field replacements
        if let DecoratedExprKind::Member { object, property, field_metadata, .. } = expr.kind {
            // Check metadata for replacement strategy
            if let FieldAccessor::Replace { with } = &field_metadata.accessor {
                // Replace self.builder with self
                return DecoratedExpr {
                    kind: DecoratedExprKind::Ident {
                        name: with.clone(),
                        ident_metadata: SwcIdentifierMetadata::default(),
                    },
                    metadata: expr.metadata,
                };
            }
        }
        expr
    }
}
```

**What moves from codegen:**
- Lines 2533-2554: `self.builder` and `self.state` replacement logic

**Already in decorator:**
- Decorator sets `FieldAccessor::Replace { with: "self" }` metadata
- Rewriter just applies it!

---

### 4. Binary Expression Sym Dereferencing

**Problem:** Comparing identifier names requires `&*ident.sym`

**Example:**
```rust
// ReluxScript
if obj.name == "console" { ... }

// SWC
if &*obj.sym == "console" { ... }
```

**Current State:**
- Decorator sets `binary_metadata.left_needs_deref = true`
- Codegen emits `&*` prefix based on metadata

**Rewriter Option 1: Insert Explicit Deref Nodes**

```rust
impl SwcRewriter {
    fn rewrite_binary_comparison(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        if let DecoratedExprKind::Binary { left, op, right, binary_metadata } = expr.kind {
            let left = if binary_metadata.left_needs_deref {
                // Wrap in explicit deref node
                DecoratedExpr {
                    kind: DecoratedExprKind::Ref {
                        mutable: false,
                        expr: Box::new(DecoratedExpr {
                            kind: DecoratedExprKind::Deref(left),
                            metadata: left.metadata.clone(),
                        }),
                    },
                    metadata: left.metadata.clone(),
                }
            } else {
                *left
            };

            // Similar for right...

            DecoratedExpr {
                kind: DecoratedExprKind::Binary {
                    left: Box::new(left),
                    op,
                    right: Box::new(right),
                    binary_metadata: SwcBinaryMetadata {
                        left_needs_deref: false,  // Already handled!
                        right_needs_deref: false,
                    },
                },
                metadata: expr.metadata,
            }
        }
        expr
    }
}
```

**Rewriter Option 2: Keep Metadata**

```rust
// Alternative: Keep the metadata approach, don't expand
// Codegen still emits &* based on metadata
// This is simpler and metadata is clean
```

**Decision:** Keep metadata for this case - it's clean and doesn't add complexity.

**What stays in codegen:**
- Emitting `&*` based on `binary_metadata.left_needs_deref`
- This is a simple prefix, not a transformation

---

### 5. Matches! Macro Expansion

**Problem:** ReluxScript `matches!(expr, pattern)` needs expansion

**Example:**
```rust
// ReluxScript
if matches!(node.callee, Callee::MemberExpression(_)) { ... }

// SWC
if let Callee::Expr(__expr) = &node.callee {
    if let Expr::Member(_) = __expr.as_ref() {
        true
    } else { false }
} else { false }
```

**Rewriter Transformation:**

```rust
impl SwcRewriter {
    fn expand_matches_macro(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        if let DecoratedExprKind::Matches { expr: scrutinee, pattern } = expr.kind {
            // Transform matches!(expr, pattern) into:
            // if let pattern = expr { true } else { false }

            // But pattern might need desugaring first!
            let desugared_pattern = self.rewrite_pattern(*pattern);

            // Generate if-let expression
            DecoratedExpr {
                kind: DecoratedExprKind::If(Box::new(DecoratedIfExpr {
                    condition: DecoratedExpr {
                        kind: DecoratedExprKind::Literal(Literal::Bool(true)),
                        metadata: SwcExprMetadata::default(),
                    },
                    then_branch: DecoratedBlock {
                        stmts: vec![
                            DecoratedStmt::Expr(DecoratedExpr {
                                kind: DecoratedExprKind::Literal(Literal::Bool(true)),
                                metadata: SwcExprMetadata::default(),
                            })
                        ],
                    },
                    else_branch: Some(DecoratedBlock {
                        stmts: vec![
                            DecoratedStmt::Expr(DecoratedExpr {
                                kind: DecoratedExprKind::Literal(Literal::Bool(false)),
                                metadata: SwcExprMetadata::default(),
                            })
                        ],
                    }),
                })),
                metadata: SwcExprMetadata {
                    swc_type: "bool".into(),
                    // ...
                }
            }
        }
        expr
    }
}
```

**What moves from codegen:**
- `gen_matches_macro()` lines 2936-2951: Macro expansion logic

**Result:** Codegen receives if-let expression - just emits it!

---

## 🏗️ IMPLEMENTATION STRUCTURE

### Module Organization

```
src/codegen/
├── swc_rewriter.rs         # NEW: Main rewriter implementation
├── swc_rewriter/
│   ├── mod.rs              # Re-exports
│   ├── pattern_desugar.rs  # Pattern desugaring transformations
│   ├── member_unwrap.rs    # Nested member access unwrapping
│   ├── field_replace.rs    # Field replacement transformations
│   └── macro_expand.rs     # Macro expansion (matches!, etc.)
├── swc_decorator.rs        # Existing decorator
├── decorated_ast.rs        # Existing decorated AST types
└── swc/                    # Modular codegen (already done)
```

### Core Types

```rust
/// AST rewriter that transforms DecoratedAST → DecoratedAST
pub struct SwcRewriter {
    /// Semantic type environment (from decorator)
    type_env: TypeEnv,

    /// Whether we're in a writer (affects self.builder → self)
    is_writer: bool,

    /// Counter for generating unique temp variable names
    temp_var_counter: usize,
}

impl SwcRewriter {
    /// Create new rewriter with semantic type environment
    pub fn new(type_env: TypeEnv, is_writer: bool) -> Self {
        Self {
            type_env,
            is_writer,
            temp_var_counter: 0,
        }
    }

    /// Rewrite entire program
    pub fn rewrite_program(&mut self, program: DecoratedProgram) -> DecoratedProgram {
        DecoratedProgram {
            decl: self.rewrite_top_level_decl(program.decl),
        }
    }

    /// Rewrite top-level declaration
    fn rewrite_top_level_decl(&mut self, decl: DecoratedTopLevelDecl) -> DecoratedTopLevelDecl {
        match decl {
            DecoratedTopLevelDecl::Plugin(plugin) => {
                DecoratedTopLevelDecl::Plugin(self.rewrite_plugin(plugin))
            }
            DecoratedTopLevelDecl::Writer(writer) => {
                DecoratedTopLevelDecl::Writer(self.rewrite_writer(writer))
            }
            DecoratedTopLevelDecl::Undecorated(_) => decl,
        }
    }

    /// Rewrite plugin
    fn rewrite_plugin(&mut self, plugin: DecoratedPlugin) -> DecoratedPlugin {
        DecoratedPlugin {
            name: plugin.name,
            body: plugin.body.into_iter()
                .map(|item| self.rewrite_plugin_item(item))
                .collect(),
        }
    }

    /// Rewrite plugin item
    fn rewrite_plugin_item(&mut self, item: DecoratedPluginItem) -> DecoratedPluginItem {
        match item {
            DecoratedPluginItem::Function(func) => {
                DecoratedPluginItem::Function(self.rewrite_function(func))
            }
            _ => item,
        }
    }

    /// Rewrite function
    fn rewrite_function(&mut self, func: DecoratedFnDecl) -> DecoratedFnDecl {
        DecoratedFnDecl {
            name: func.name,
            params: func.params,
            body: self.rewrite_block(func.body),
        }
    }

    /// Rewrite block
    fn rewrite_block(&mut self, block: DecoratedBlock) -> DecoratedBlock {
        DecoratedBlock {
            stmts: block.stmts.into_iter()
                .map(|stmt| self.rewrite_stmt(stmt))
                .collect(),
        }
    }

    /// Rewrite statement (main dispatch)
    fn rewrite_stmt(&mut self, stmt: DecoratedStmt) -> DecoratedStmt {
        match stmt {
            DecoratedStmt::If(if_stmt) => {
                DecoratedStmt::If(self.rewrite_if_stmt(if_stmt))
            }
            DecoratedStmt::Let(let_stmt) => {
                DecoratedStmt::Let(DecoratedLetStmt {
                    pattern: self.rewrite_pattern(let_stmt.pattern),
                    init: self.rewrite_expr(let_stmt.init),
                    mutable: let_stmt.mutable,
                })
            }
            DecoratedStmt::Expr(expr) => {
                DecoratedStmt::Expr(self.rewrite_expr(expr))
            }
            // ... other statement types
            _ => stmt,
        }
    }

    /// Rewrite if statement (handles pattern desugaring)
    fn rewrite_if_stmt(&mut self, if_stmt: DecoratedIfStmt) -> DecoratedIfStmt {
        // Check if pattern needs desugaring
        let pattern = if let Some(p) = if_stmt.pattern {
            Some(self.rewrite_pattern(p))
        } else {
            None
        };

        DecoratedIfStmt {
            pattern,
            condition: self.rewrite_expr(if_stmt.condition),
            then_branch: self.rewrite_block(if_stmt.then_branch),
            else_branch: if_stmt.else_branch.map(|b| self.rewrite_block(b)),
        }
    }

    /// Rewrite expression (main dispatch)
    fn rewrite_expr(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // First, recursively rewrite children
        let expr = self.rewrite_expr_children(expr);

        // Then apply transformations
        let expr = self.rewrite_nested_member_access(expr);
        let expr = self.rewrite_field_replacements(expr);
        let expr = self.expand_matches_macro(expr);

        expr
    }

    /// Recursively rewrite expression children
    fn rewrite_expr_children(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        let kind = match expr.kind {
            DecoratedExprKind::Binary { left, op, right, binary_metadata } => {
                DecoratedExprKind::Binary {
                    left: Box::new(self.rewrite_expr(*left)),
                    op,
                    right: Box::new(self.rewrite_expr(*right)),
                    binary_metadata,
                }
            }
            DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata } => {
                DecoratedExprKind::Member {
                    object: Box::new(self.rewrite_expr(*object)),
                    property,
                    optional,
                    computed,
                    is_path,
                    field_metadata,
                }
            }
            DecoratedExprKind::Call(call) => {
                DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                    callee: self.rewrite_expr(call.callee),
                    args: call.args.into_iter()
                        .map(|arg| self.rewrite_expr(arg))
                        .collect(),
                }))
            }
            // ... other expression types
            _ => expr.kind,
        };

        DecoratedExpr {
            kind,
            metadata: expr.metadata,
        }
    }

    /// Rewrite pattern (handles desugaring)
    fn rewrite_pattern(&mut self, pattern: DecoratedPattern) -> DecoratedPattern {
        // Check if pattern needs desugaring based on metadata
        if pattern.metadata.needs_desugaring() {
            return self.desugar_pattern(pattern);
        }

        // Recursively rewrite children
        let kind = match pattern.kind {
            DecoratedPatternKind::Variant { name, inner } => {
                DecoratedPatternKind::Variant {
                    name,
                    inner: inner.map(|p| Box::new(self.rewrite_pattern(*p))),
                }
            }
            DecoratedPatternKind::Tuple(patterns) => {
                DecoratedPatternKind::Tuple(
                    patterns.into_iter()
                        .map(|p| self.rewrite_pattern(p))
                        .collect()
                )
            }
            // ... other pattern types
            _ => pattern.kind,
        };

        DecoratedPattern {
            kind,
            metadata: pattern.metadata,
        }
    }

    /// Generate unique temporary variable name
    fn gen_temp_var(&mut self) -> String {
        let name = format!("__temp_{}", self.temp_var_counter);
        self.temp_var_counter += 1;
        name
    }
}
```

---

## 🔍 METADATA EXTENSIONS

### SwcPatternMetadata Enhancement

Add field to indicate if desugaring is needed:

```rust
pub struct SwcPatternMetadata {
    pub swc_pattern: String,
    pub unwrap_strategy: UnwrapStrategy,
    pub inner: Option<Box<SwcPatternMetadata>>,
    pub span: Option<Span>,
    pub source_pattern: Option<String>,

    // NEW: Indicates pattern needs desugaring
    pub desugar_strategy: Option<DesugarStrategy>,
}

pub enum DesugarStrategy {
    /// Pattern doesn't exist in SWC - needs nested if-let
    NestedIfLet {
        /// Outer pattern to generate
        outer_pattern: String,
        /// Outer binding variable
        outer_binding: String,
        /// Inner pattern to generate
        inner_pattern: String,
        /// Inner binding variable
        inner_binding: String,
        /// Unwrap expression (e.g., ".as_ref()")
        unwrap_expr: String,
    },

    /// Pattern exists but needs special handling
    None,
}

impl SwcPatternMetadata {
    /// Check if this pattern needs desugaring
    pub fn needs_desugaring(&self) -> bool {
        self.desugar_strategy.is_some()
    }
}
```

### FieldAccessor Enhancement

Already has `Replace` variant - just ensure decorator uses it:

```rust
pub enum FieldAccessor {
    Direct,
    BoxedAsRef,
    BoxedRefDeref,
    EnumField { enum_name: String, is_boxed: bool },
    Optional { inner: Box<FieldAccessor> },

    Replace {
        /// Replacement expression (e.g., "self" for self.builder)
        with: String,
    },  // ← Already exists!
}
```

---

## 📝 IMPLEMENTATION PHASES

### Phase 1: Infrastructure Setup
**Goal:** Create rewriter skeleton and wire it into pipeline

**Tasks:**
1. Create `swc_rewriter.rs` with `SwcRewriter` struct
2. Implement basic traversal (rewrite_program, rewrite_stmt, rewrite_expr, rewrite_pattern)
3. Add rewriter to codegen pipeline in `mod.rs`
4. Test that it passes through AST unchanged

**Test:**
```rust
let decorated = decorator.decorate_program(&program);
let rewritten = rewriter.rewrite_program(decorated.clone());
assert_eq!(decorated, rewritten); // No transformations yet
```

**Files Modified:**
- `src/codegen/swc_rewriter.rs` (NEW)
- `src/codegen/mod.rs` (wire up rewriter)

---

### Phase 2: Pattern Desugaring
**Goal:** Implement pattern desugaring for `Callee::MemberExpression`

**Tasks:**
1. Add `DesugarStrategy` to `SwcPatternMetadata`
2. Update decorator to set `desugar_strategy` for non-existent patterns
3. Implement `SwcRewriter::desugar_pattern()`
4. Test with console_remover

**Test Cases:**
- `Callee::MemberExpression(_)` → nested if-let
- `Expr::Ident(_)` → no desugaring (exists in SWC)
- Nested desugaring (pattern inside pattern)

**Files Modified:**
- `src/codegen/swc_metadata.rs` (add DesugarStrategy)
- `src/codegen/swc_decorator.rs` (set desugar_strategy)
- `src/codegen/swc_rewriter.rs` (implement desugaring)

---

### Phase 3: Field Replacements
**Goal:** Implement `self.builder` → `self` transformation

**Tasks:**
1. Ensure decorator sets `FieldAccessor::Replace` for writer contexts
2. Implement `SwcRewriter::rewrite_field_replacements()`
3. Test with writer examples

**Test Cases:**
- `self.builder.write()` → `self.write()`
- `self.state.field` → `self.field`
- Regular field access unchanged

**Files Modified:**
- `src/codegen/swc_decorator.rs` (ensure Replace is set)
- `src/codegen/swc_rewriter.rs` (implement replacement)

---

### Phase 4: Nested Member Unwrapping
**Goal:** Transform `node.callee.name` into match chain

**Tasks:**
1. Implement member chain detection
2. Implement match chain generation
3. Handle edge cases (optional chaining, null safety)

**Test Cases:**
- `node.callee.name` → match chain
- `node.name` → no unwrapping
- `a.b.c.d` → nested match chain

**Files Modified:**
- `src/codegen/swc_rewriter.rs` (implement unwrapping)

---

### Phase 5: Matches! Macro Expansion
**Goal:** Expand `matches!(expr, pattern)` to if-let

**Tasks:**
1. Implement `SwcRewriter::expand_matches_macro()`
2. Combine with pattern desugaring
3. Test with complex patterns

**Test Cases:**
- `matches!(x, Pattern)` → if-let
- `matches!(x, Callee::MemberExpression(_))` → nested if-let
- Negated matches: `!matches!(...)` → swap true/false

**Files Modified:**
- `src/codegen/swc_rewriter.rs` (implement expansion)

---

### Phase 6: Codegen Simplification
**Goal:** Remove transformation logic from codegen

**Tasks:**
1. Remove `gen_swc_pattern_check()` - no longer needed
2. Remove desugaring logic from `gen_if_let_stmt()`
3. Remove field replacement logic from member expression gen
4. Remove matches! macro handling
5. Simplify all gen_* methods to just emit

**Files Modified:**
- `src/codegen/swc/patterns.rs` (remove gen_swc_pattern_check)
- `src/codegen/swc/statements.rs` (simplify gen_if_let_stmt)
- `src/codegen/swc/expressions.rs` (simplify member gen)

**Result:** Codegen becomes truly dumb!

---

### Phase 7: Testing & Validation
**Goal:** Ensure all integration tests pass

**Tasks:**
1. Test console_remover with rewriter
2. Test all 16 integration tests
3. Test all minimal tests
4. Compare generated code with/without rewriter
5. Benchmark performance impact

**Success Criteria:**
- All tests pass
- Generated SWC code is identical
- Performance impact < 5%

---

## 🎯 SUCCESS CRITERIA

### Functional Requirements

✅ **Pattern Desugaring:**
- `Callee::MemberExpression` correctly desugars to nested if-let
- All non-existent patterns are desugared
- Nested patterns work correctly

✅ **Field Replacements:**
- `self.builder` → `self` in writers
- `self.state` → `self` in writers
- Regular field access unchanged

✅ **Nested Member Unwrapping:**
- `node.callee.name` generates match chain
- Correct unwrapping (.as_ref(), &*, etc.)
- Multiple levels of nesting work

✅ **Matches! Expansion:**
- `matches!(expr, pattern)` → if-let
- Works with desugared patterns
- Negation works correctly

✅ **Codegen Simplification:**
- No transformation logic in codegen
- gen_* methods just emit strings
- No special case handling

### Quality Requirements

✅ **Correctness:**
- All integration tests pass
- Generated SWC code compiles
- Runtime behavior identical

✅ **Performance:**
- Rewriter adds < 100ms to compilation
- No significant memory overhead
- AST cloning is minimal

✅ **Maintainability:**
- Rewriter logic is isolated
- Each transformation is testable
- Clear separation of concerns

---

## 🚨 EDGE CASES & CHALLENGES

### 1. Pattern Binding Capture

**Problem:** When desugaring patterns, bindings must be preserved

```rust
// Input
if let Callee::MemberExpression(member) = node.callee { ... }

// Output must bind 'member'
if let Callee::Expr(__expr) = &node.callee {
    if let Expr::Member(member) = __expr.as_ref() {  // ← 'member' still bound
        ...
    }
}
```

**Solution:** Track bindings during desugaring, ensure inner pattern uses original binding name.

---

### 2. Temporary Variable Naming

**Problem:** Need unique names for intermediate variables

```rust
// If user already uses __expr, we need __expr_1, __expr_2, etc.
```

**Solution:** Use counter-based naming with conflict detection.

---

### 3. Metadata Propagation

**Problem:** When creating new AST nodes, metadata must be correct

```rust
// New match expression needs correct SwcExprMetadata
DecoratedExpr {
    kind: DecoratedExprKind::Match(...),
    metadata: SwcExprMetadata {
        swc_type: ???,  // What type does this expression have?
        // ...
    }
}
```

**Solution:**
- For pattern desugaring: use same metadata as original if-let
- For matches! expansion: type is always `bool`
- For member unwrapping: use final field's type

---

### 4. Span Information

**Problem:** Generated AST nodes don't have source locations

**Solution:** Copy spans from original nodes, mark generated nodes with `Span::DUMMY`

---

### 5. Recursive Transformations

**Problem:** Transformations might trigger more transformations

```rust
// matches!(node.callee.name, "console")
// 1. Expand matches! → if-let with nested member access
// 2. Unwrap nested member → match chain
// 3. Desugar pattern if needed
```

**Solution:** Apply transformations in specific order:
1. Pattern desugaring (bottom-up)
2. Expression expansion (matches!, macros)
3. Member unwrapping (top-down)
4. Field replacements (final pass)

---

## 📊 COMPARISON: BEFORE VS AFTER

### Before Rewriter

```
Decorator (TypeEnv) → DecoratedAST (with metadata)
                           ↓
                    Codegen (gen_expr, gen_stmt)
                           ↓
                    - Check metadata
                    - Decide transformation
                    - Generate code
                           ↓
                      Rust source
```

**Codegen Complexity:** HIGH
- Checks metadata
- Makes decisions
- Generates transformations inline
- Special cases everywhere

---

### After Rewriter

```
Decorator (TypeEnv) → DecoratedAST (with metadata)
                           ↓
                    Rewriter (transformations)
                           ↓
                    - Desugar patterns
                    - Unwrap members
                    - Replace fields
                    - Expand macros
                           ↓
                  Transformed DecoratedAST
                           ↓
                    Codegen (gen_expr, gen_stmt)
                           ↓
                    - Read metadata
                    - Emit strings
                           ↓
                      Rust source
```

**Codegen Complexity:** LOW
- Just reads metadata
- Just emits strings
- No decisions
- No special cases

---

## 🎓 LEARNING POINTS

### Why This Works

1. **Separation of Concerns:**
   - Decorator: Semantic analysis (types, fields, patterns)
   - Rewriter: Structural transformation (desugaring, unwrapping)
   - Codegen: String emission (just output)

2. **AST → AST Transformation:**
   - Input and output are both DecoratedAST
   - Preserves type information
   - Can be tested independently
   - Can be composed (multiple rewrites)

3. **Metadata-Driven:**
   - Rewriter decisions based on metadata
   - No ad-hoc inference
   - Deterministic transformations

### Similar Patterns in Other Compilers

- **Rust:** Multiple MIR passes (borrowck, optimization, codegen)
- **LLVM:** Multiple IR passes (optimization, lowering)
- **GHC:** Core-to-Core transformations (desugaring, optimization)
- **Babel:** Plugin transformations on AST

**Our approach:** Same principle, but DecoratedAST → DecoratedAST

---

## 🚀 NEXT STEPS

1. Review this plan with team
2. Get approval on architecture
3. Create tracking issue for implementation
4. Begin Phase 1: Infrastructure setup
5. Iterate through phases 2-7
6. Ship rewriter, remove codegen complexity!

---

**Status:** Ready for implementation
**Estimated Effort:** 3-5 days (1 day per phase)
**Impact:** HIGH - Makes codegen truly dumb, easier to maintain, easier to extend
