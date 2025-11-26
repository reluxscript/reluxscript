# SWC Codegen Modularization Refactor Plan

**Status:** Planning Phase
**Goal:** Split monolithic `swc.rs` (3381 lines, 55 methods) into focused, maintainable modules

---

## 📁 Proposed Directory Structure

```
src/codegen/
├── swc/
│   ├── mod.rs              # SwcGenerator struct, main entry points
│   ├── emit.rs             # Output emission utilities
│   ├── detection.rs        # Import/collection detection
│   ├── type_mapping.rs     # Type name conversions
│   ├── type_inference.rs   # Type inference (TO BE REMOVED)
│   ├── top_level.rs        # Top-level declaration generation
│   ├── structures.rs       # Struct/enum generation
│   ├── visitors.rs         # Visitor method generation
│   ├── statements.rs       # Statement generation
│   ├── expressions.rs      # Expression generation
│   └── patterns.rs         # Pattern generation
├── swc_backup.rs           # Backup of original swc.rs
├── swc_decorator.rs        # Decorator (transforms AST → DecoratedAST)
├── swc_metadata.rs         # Metadata types
├── decorated_ast.rs        # Decorated AST types
└── (babel.rs, etc.)
```

---

## 📊 Method Distribution by Module

### **mod.rs** (Main Entry Point)
**Lines:** ~150
**Responsibility:** SwcGenerator struct, main generate methods, module coordination

```rust
pub struct SwcGenerator {
    output: String,
    indent: usize,
    param_renames: HashMap<String, String>,
    plugin_name: String,
    hoisted_visitors: Vec<String>,
    captured_vars: HashSet<String>,
    uses_json: bool,
    uses_parser: bool,
    uses_codegen: bool,
    associated_functions: HashSet<String>,
    is_writer: bool,
}

impl SwcGenerator {
    pub fn new() -> Self
    pub fn generate(&mut self, program: &Program) -> String
    pub fn generate_decorated(&mut self, program: &DecoratedProgram) -> String
}
```

**Methods:**
- `new()` - Constructor
- `generate()` - Original AST entry point (calls old methods)
- `generate_decorated()` - Decorated AST entry point (calls decorated methods)

---

### **emit.rs** (Output Utilities)
**Lines:** ~50
**Responsibility:** Low-level string emission, indentation management

**Methods moved from swc.rs:**
- `emit(&mut self, s: &str)` - Line 377
- `emit_indent(&mut self)` - Line 381
- `emit_line(&mut self, s: &str)` - Line 387

**Additional utilities:**
- `with_indent<F>(&mut self, f: F)` - Helper for indented blocks
- `emit_separated<T, F>(&mut self, items: &[T], sep: &str, f: F)` - Emit list with separator

```rust
impl SwcGenerator {
    pub(super) fn emit(&mut self, s: &str);
    pub(super) fn emit_indent(&mut self);
    pub(super) fn emit_line(&mut self, s: &str);
    pub(super) fn with_indent<F>(&mut self, f: F) where F: FnOnce(&mut Self);
    pub(super) fn emit_separated<T, F>(&mut self, items: &[T], sep: &str, f: F);
}
```

---

### **detection.rs** (Import & Collection Detection)
**Lines:** ~200
**Responsibility:** Detect what imports/collections are used in program

**Methods moved from swc.rs:**
- `detect_std_collections(&self, program: &Program) -> (bool, bool)` - Line 190
- `detect_collections_in_item(&self, item: &PluginItem, ...)` - Line 217
- `detect_collections_in_type(&self, ty: &Type, ...)` - Line 232
- `detect_collections_in_block(&self, block: &Block, ...)` - Line 260
- `detect_collections_in_stmt(&self, stmt: &Stmt, ...)` - Line 266
- `detect_collections_in_expr(&self, expr: &Expr, ...)` - Line 308

**New methods:**
- `detect_imports(&self, program: &Program) -> ImportSet` - Detect all needed imports

```rust
pub struct ImportSet {
    pub uses_hashmap: bool,
    pub uses_hashset: bool,
    pub uses_json: bool,
    pub uses_parser: bool,
    pub uses_codegen: bool,
}

impl SwcGenerator {
    pub(super) fn detect_std_collections(&self, program: &Program) -> (bool, bool);
    pub(super) fn detect_imports(&self, program: &Program) -> ImportSet;
}
```

---

### **type_mapping.rs** (Type Name Conversions)
**Lines:** ~200
**Responsibility:** Convert ReluxScript type names to SWC Rust types

**Methods moved from swc.rs:**
- `visitor_name_to_swc(&self, name: &str) -> String` - Line 1191
- `visitor_name_to_swc_type(&self, name: &str) -> String` - Line 1205
- `to_swc_node_name(&self, name: &str) -> String` - Line 1214
- `reluxscript_to_swc_type(&self, name: &str) -> String` - Line 1241
- `type_to_rust(&self, ty: &Type) -> String` - Line 1276
- `get_default_value_for_type(&self, ty: &Type) -> String` - Line 1330
- `visitor_method_to_swc(&self, method_name: &str) -> String` - Line 2029
- `reluxscript_type_to_swc(&self, type_name: &str) -> String` - Line 2038
- `binary_op_to_rust(&self, op: &BinaryOp) -> &'static str` - Line 3128
- `compound_op_to_rust(&self, op: &CompoundAssignOp) -> &'static str` - Line 3146

```rust
impl SwcGenerator {
    pub(super) fn visitor_name_to_swc(&self, name: &str) -> String;
    pub(super) fn reluxscript_to_swc_type(&self, name: &str) -> String;
    pub(super) fn type_to_rust(&self, ty: &Type) -> String;
    pub(super) fn binary_op_to_rust(&self, op: &BinaryOp) -> &'static str;
}
```

---

### **type_inference.rs** (Type Inference - TO BE REMOVED)
**Lines:** ~200
**Responsibility:** Flow-sensitive type inference (will be replaced by decorator metadata)

**⚠️ DEPRECATED - Will be removed once decorator is complete**

**Methods moved from swc.rs:**
- `infer_type(&self, expr: &Expr) -> TypeContext` - Line 2097
- `type_from_ast(&self, ty: &Type) -> TypeContext` - Line 2192
- `get_element_type(&self, container_type: &TypeContext) -> TypeContext` - Line 2229
- `extract_matches_pattern(&self, expr: &Expr) -> Option<...>` - Line 2051

```rust
// ⚠️ DEPRECATED - These will be removed
impl SwcGenerator {
    fn infer_type(&self, expr: &Expr) -> TypeContext;
    fn type_from_ast(&self, ty: &Type) -> TypeContext;
    fn get_element_type(&self, container_type: &TypeContext) -> TypeContext;
}
```

---

### **top_level.rs** (Top-Level Declaration Generation)
**Lines:** ~400
**Responsibility:** Generate plugins, writers, modules

**Methods moved from swc.rs:**
- `gen_plugin(&mut self, plugin: &PluginDecl)` - Line 419
- `gen_writer(&mut self, writer: &WriterDecl)` - Line 512
- `gen_module(&mut self, module: &ModuleDecl)` - Line 729
- `gen_decorated_plugin(&mut self, plugin: &DecoratedPlugin)` - Line 685
- `gen_decorated_writer(&mut self, writer: &DecoratedWriter)` - Line 717
- `gen_decorated_visitor_method(&mut self, func: &DecoratedFnDecl)` - Line 723

```rust
impl SwcGenerator {
    pub(super) fn gen_plugin(&mut self, plugin: &PluginDecl);
    pub(super) fn gen_writer(&mut self, writer: &WriterDecl);
    pub(super) fn gen_module(&mut self, module: &ModuleDecl);
    pub(super) fn gen_decorated_plugin(&mut self, plugin: &DecoratedPlugin);
    pub(super) fn gen_decorated_writer(&mut self, writer: &DecoratedWriter);
}
```

---

### **structures.rs** (Struct & Enum Generation)
**Lines:** ~250
**Responsibility:** Generate struct/enum definitions and helper functions

**Methods moved from swc.rs:**
- `gen_struct(&mut self, s: &StructDecl)` - Line 758
- `gen_enum(&mut self, e: &EnumDecl)` - Line 775
- `gen_helper_function(&mut self, f: &FnDecl)` - Line 811
- `gen_parser_module_helpers(&mut self)` - Line 953
- `gen_codegen_module_helpers(&mut self)` - Line 1056
- `gen_codegen_call(&mut self, function_name: &str, args: &[Expr])` - Line 867
- `gen_codegen_config(&mut self, options_expr: &Expr)` - Line 915

```rust
impl SwcGenerator {
    pub(super) fn gen_struct(&mut self, s: &StructDecl);
    pub(super) fn gen_enum(&mut self, e: &EnumDecl);
    pub(super) fn gen_helper_function(&mut self, f: &FnDecl);
    pub(super) fn gen_parser_module_helpers(&mut self);
    pub(super) fn gen_codegen_module_helpers(&mut self);
}
```

---

### **visitors.rs** (Visitor Method Generation)
**Lines:** ~200
**Responsibility:** Generate visitor methods for SWC's VisitMut/Visit traits

**Methods moved from swc.rs:**
- `gen_visitor_method(&mut self, f: &FnDecl)` - Line 1103
- `gen_visit_method(&mut self, f: &FnDecl)` - Line 1143
- `is_visitor_method(&self, f: &FnDecl) -> bool` - Line 398

```rust
impl SwcGenerator {
    pub(super) fn gen_visitor_method(&mut self, f: &FnDecl);
    pub(super) fn gen_visit_method(&mut self, f: &FnDecl);
    pub(super) fn is_visitor_method(&self, f: &FnDecl) -> bool;
}
```

---

### **statements.rs** (Statement Generation)
**Lines:** ~700
**Responsibility:** Generate all statement types

**Methods moved from swc.rs:**
- `gen_block(&mut self, block: &Block)` - Line 1366
- `gen_stmt(&mut self, stmt: &Stmt)` - Line 1407
- `gen_stmt_with_context(&mut self, stmt: &Stmt, is_last_in_block: bool)` - Line 1374
- `gen_if_let_stmt(&mut self, if_stmt: &IfStmt, pattern: &Pattern)` - Line 1766
- `gen_traverse_stmt(&mut self, traverse_stmt: &TraverseStmt)` - Line 1878
- `gen_decorated_stmt(&mut self, stmt: &DecoratedStmt)` - Line 2384
- `gen_decorated_if_let_stmt(&mut self, if_stmt: &DecoratedIfStmt)` - Line 2349
- `gen_decorated_block(&mut self, block: &DecoratedBlock)` - Line 2377

```rust
impl SwcGenerator {
    pub(super) fn gen_block(&mut self, block: &Block);
    pub(super) fn gen_stmt(&mut self, stmt: &Stmt);
    pub(super) fn gen_if_let_stmt(&mut self, if_stmt: &IfStmt, pattern: &Pattern);
    pub(super) fn gen_decorated_stmt(&mut self, stmt: &DecoratedStmt);
    pub(super) fn gen_decorated_if_let_stmt(&mut self, if_stmt: &DecoratedIfStmt);
    pub(super) fn gen_decorated_block(&mut self, block: &DecoratedBlock);
}
```

---

### **expressions.rs** (Expression Generation)
**Lines:** ~700
**Responsibility:** Generate all expression types

**Methods moved from swc.rs:**
- `gen_expr(&mut self, expr: &Expr)` - Line 2484
- `gen_literal(&mut self, lit: &Literal)` - Line 3105
- `gen_matches_macro(&mut self, scrutinee: &Expr, pattern: &Expr)` - Line 3156
- `gen_decorated_expr(&mut self, expr: &DecoratedExpr)` - Line 2290

```rust
impl SwcGenerator {
    pub(super) fn gen_expr(&mut self, expr: &Expr);
    pub(super) fn gen_literal(&mut self, lit: &Literal);
    pub(super) fn gen_matches_macro(&mut self, scrutinee: &Expr, pattern: &Expr);
    pub(super) fn gen_decorated_expr(&mut self, expr: &DecoratedExpr);
}
```

---

### **patterns.rs** (Pattern Generation)
**Lines:** ~200
**Responsibility:** Generate pattern matching code

**Methods moved from swc.rs:**
- `gen_pattern(&mut self, pattern: &Pattern)` - Line 2400
- `gen_swc_pattern_check(&mut self, scrutinee: &Expr, pattern: &Expr, depth: usize)` - Line 3172
- `gen_decorated_pattern(&mut self, pattern: &DecoratedPattern)` - Line 2254

```rust
impl SwcGenerator {
    pub(super) fn gen_pattern(&mut self, pattern: &Pattern);
    pub(super) fn gen_swc_pattern_check(&mut self, scrutinee: &Expr, pattern: &Expr, depth: usize);
    pub(super) fn gen_decorated_pattern(&mut self, pattern: &DecoratedPattern);
}
```

---

## 🔄 Refactoring Steps

### Phase 1: Preparation (No Breaking Changes)
1. ✅ **Create backup**: `swc_backup.rs` already exists
2. ✅ **Create directory**: `mkdir src/codegen/swc/`
3. ✅ **Create module files**: Create all empty `.rs` files with proper headers
4. ✅ **Create mod.rs**: Move struct definition and imports

### Phase 2: Extract Utilities (Low Risk)
**Order: Leaf modules first (no dependencies)**

5. ✅ **Extract emit.rs**
   - Move `emit()`, `emit_indent()`, `emit_line()`
   - Add `pub(super)` visibility
   - Test: Ensure swc.rs compiles

6. ✅ **Extract type_mapping.rs**
   - Move all type conversion methods
   - No dependencies on other generator methods
   - Test: Ensure swc.rs compiles

7. ✅ **Extract detection.rs**
   - Move all `detect_*` methods
   - Test: Ensure swc.rs compiles

### Phase 3: Extract Generation Logic (Medium Risk)
**Order: Smallest to largest**

8. ✅ **Extract patterns.rs**
   - Move `gen_pattern()`, `gen_decorated_pattern()`, `gen_swc_pattern_check()`
   - Uses: emit, type_mapping
   - Test: Build sample plugin

9. ✅ **Extract expressions.rs**
   - Move `gen_expr()`, `gen_decorated_expr()`, `gen_literal()`, etc.
   - Uses: emit, type_mapping, patterns (for destructuring)
   - Test: Build sample plugin

10. ✅ **Extract statements.rs**
    - Move `gen_stmt()`, `gen_block()`, `gen_if_let_stmt()`, etc.
    - Uses: emit, expressions, patterns
    - Test: Build sample plugin

11. ✅ **Extract structures.rs**
    - Move `gen_struct()`, `gen_enum()`, helper functions
    - Uses: emit, type_mapping, statements
    - Test: Build sample plugin

12. ✅ **Extract visitors.rs**
    - Move `gen_visitor_method()`, `gen_visit_method()`
    - Uses: statements, type_mapping
    - Test: Build sample plugin

13. ✅ **Extract top_level.rs**
    - Move `gen_plugin()`, `gen_writer()`, `gen_module()`
    - Uses: ALL other modules
    - Test: Build sample plugin

### Phase 4: Temporary Isolation (Type Inference)
14. ✅ **Extract type_inference.rs**
    - Move `infer_type()`, `type_from_ast()`, etc.
    - Mark as `#[deprecated]` with comment
    - Add TODO comments for removal
    - Test: Ensure old pipeline still works

### Phase 5: Integration & Testing
15. ✅ **Update mod.rs**
    - Add `mod` declarations for all modules
    - Ensure all `pub(super)` methods are accessible
    - Test: Full compilation

16. ✅ **Run integration tests**
    - Test Babel codegen (should be unaffected)
    - Test SWC codegen with decorated AST
    - Test console_remover

17. ✅ **Update documentation**
    - Update COMPILER_ARCHITECTURE.md
    - Add module-level docs to each file
    - Update README if needed

### Phase 6: Cleanup (Post-Refactor)
18. ✅ **Remove swc.rs**
    - Delete original monolithic file
    - Keep swc_backup.rs for reference

19. ✅ **Remove type_inference.rs** (Future)
    - Once TypeEnvironment is fully removed
    - Once decorator handles all type inference

---

## 📝 Module Template

Each module should follow this structure:

```rust
//! [Module Name] - [Brief description]
//!
//! This module handles [specific responsibility].
//! Part of the SWC code generator.

use super::SwcGenerator;
use crate::parser::*;
use crate::codegen::decorated_ast::*;

impl SwcGenerator {
    /// [Method description]
    pub(super) fn method_name(&mut self, arg: &Type) {
        // Implementation
    }
}
```

---

## ⚠️ Important Notes

1. **Visibility**: All methods use `pub(super)` to be accessible within `swc/` module only
2. **No pub re-exports**: SwcGenerator methods stay private, only public API is `generate()`
3. **Circular dependencies**: Avoid! Structure modules in dependency order
4. **Testing**: Test after EACH module extraction, not at the end
5. **Decorated vs Original**: Keep both until full migration to decorated AST

---

## 🎯 Success Criteria

✅ **Modularity**: No single file > 700 lines
✅ **Maintainability**: Clear separation of concerns
✅ **Testability**: Each module can be unit tested
✅ **No regressions**: All existing tests pass
✅ **Performance**: No measurable slowdown
✅ **Documentation**: Each module has clear purpose

---

## 📊 Estimated Line Counts After Refactoring

| Module | Lines | Methods | Status |
|--------|-------|---------|--------|
| mod.rs | 150 | 3 | New |
| emit.rs | 50 | 5 | New |
| detection.rs | 200 | 6 | New |
| type_mapping.rs | 200 | 10 | New |
| type_inference.rs | 200 | 4 | **Deprecated** |
| top_level.rs | 400 | 6 | New |
| structures.rs | 250 | 7 | New |
| visitors.rs | 200 | 3 | New |
| statements.rs | 700 | 8 | New |
| expressions.rs | 700 | 4 | New |
| patterns.rs | 200 | 3 | New |
| **Total** | **3250** | **59** | - |

**Savings:** 131 lines removed (dead code, duplicates)

---

## 🚀 Next Steps

1. Review this plan for approval
2. Begin Phase 1: Create directory structure
3. Extract modules one by one in order
4. Test incrementally
5. Update documentation

---

**Status:** Ready for implementation
**Last Updated:** 2025-11-25
