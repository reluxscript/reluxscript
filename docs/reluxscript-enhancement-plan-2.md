# ReluxScript Compiler Enhancement Plan

**Version:** 0.3.0 → 0.4.0
**Status:** In Progress
**Goal:** Generate production-ready Babel and SWC plugin code from ReluxScript

---

## Current State

The ReluxScript compiler successfully:
- Parses `.lux` files
- Performs semantic analysis (name resolution, type checking, ownership validation)
- Generates both Babel (JavaScript) and SWC (Rust) output
- Has comprehensive AST mapping tables

However, the generated code has several issues that prevent it from being directly executable.

---

## Issues Identified

### 1. `matches!` Macro Code Generation

**Priority:** High
**Affects:** Both Babel and SWC

The `matches!` macro is not generating correct platform-specific code.

**Current Output (Babel):**
```javascript
if ((current === MemberExpression)) {
```

**Expected Output (Babel):**
```javascript
if (t.isMemberExpression(current)) {
```

**Current Output (SWC):**
```rust
if {
    let __matched = current == MemberExpression;
    __matched
} {
```

**Expected Output (SWC):**
```rust
if matches!(current, Expr::Member(_)) {
```

**Implementation Tasks:**
- [ ] Update `gen_matches_pattern` in `codegen/babel.rs` to generate `t.isXxx()` calls
- [ ] Update `gen_matches_pattern` in `codegen/swc.rs` to generate proper `matches!` macro
- [ ] Use `PatternMapping` from the mapping module
- [ ] Handle nested patterns (e.g., `MemberExpression { property: Identifier { name: "log" } }`)

---

### 2. JavaScript Method Translations

**Priority:** High
**Affects:** Babel only

ReluxScript uses Rust-like method names that need translation to JavaScript equivalents.

| ReluxScript | JavaScript | Notes |
|------------|------------|-------|
| `vec.insert(0, x)` | `vec.unshift(x)` | Insert at beginning |
| `vec.push(x)` | `vec.push(x)` | Same |
| `x.clone()` | `x` | No-op in JS |
| `str.to_string()` | `str` | No-op in JS |

**Implementation Tasks:**
- [ ] Create method translation table in `codegen/babel.rs`
- [ ] Intercept method calls and translate:
  - `.insert(0, x)` → `.unshift(x)`
  - `.clone()` → remove call entirely
  - `.to_string()` → remove call entirely
- [ ] Handle `.clone()` on member expressions (just return the expression)

---

### 3. SWC Field Access Patterns

**Priority:** High
**Affects:** SWC only

SWC uses different field names and requires Box/Option unwrapping.

| ReluxScript | SWC | Notes |
|------------|-----|-------|
| `member.property` | `member.prop` | Different name |
| `member.object` | `*member.obj` | Box unwrapping |
| `call.arguments` | `call.args` | Different name |
| `ident.name` | `ident.sym.to_string()` | JsWord conversion |

**Implementation Tasks:**
- [ ] Use `FieldMapping` from the mapping module during code generation
- [ ] Generate Box dereference (`*`) when `needs_box_unwrap` is true
- [ ] Apply read/write conversions from field mappings
- [ ] Handle nested field access patterns

---

### 4. Type-Specific Pattern Matching

**Priority:** Medium
**Affects:** SWC only

SWC requires knowing the enum variant for pattern matching.

**ReluxScript:**
```reluxscript
if matches!(node, MemberExpression) {
```

**SWC (Correct):**
```rust
if let Expr::Member(member) = node {
    // use member
}
```

**Implementation Tasks:**
- [ ] Track the type context when generating matches
- [ ] Use `NodeMapping.swc_enum_variant` to get the correct variant
- [ ] Generate `if let` patterns for simple type checks
- [ ] Generate proper destructuring when accessing fields

---

### 5. String/Atom Handling

**Priority:** Medium
**Affects:** SWC only

SWC uses `JsWord` (interned strings) instead of `String`.

**Implementation Tasks:**
- [ ] Generate `.into()` when assigning String to JsWord field
- [ ] Generate `.to_string()` when reading JsWord to String
- [ ] Use `Atom::from()` for string literals in node construction
- [ ] Handle the `Str` type correctly in both directions

---

### 6. Visitor Method Generation

**Priority:** Medium
**Affects:** Both

Currently generates empty visitor blocks. Need to:

**Implementation Tasks:**
- [ ] Generate visitor methods that delegate to helper functions
- [ ] Map ReluxScript visitor names to platform-specific names
- [ ] Handle the `ctx: &Context` parameter appropriately
- [ ] Generate `visit_mut_children_with(self)` calls in SWC

---

### 7. Module/Export Structure

**Priority:** Low
**Affects:** Both

**Babel:**
- [ ] Export helper functions properly
- [ ] Support CommonJS and ES modules

**SWC:**
- [ ] Generate proper `#[plugin_transform]` attribute
- [ ] Generate `program.visit_mut_with(&mut plugin)` boilerplate
- [ ] Add necessary `use` statements

---

## Implementation Order

### Phase 1: Core Expression Translation (Days 1-2)
1. Fix `matches!` macro for both targets
2. Implement JS method translations (clone, insert)
3. Implement SWC field mappings

### Phase 2: Pattern Matching (Days 3-4)
4. Implement proper `if let` generation for SWC
5. Handle nested pattern matching
6. Add field destructuring

### Phase 3: String Handling (Day 5)
7. Implement JsWord conversions
8. Handle Atom creation

### Phase 4: Structure & Polish (Days 6-7)
9. Fix module structure
10. Generate proper visitor methods
11. Add comprehensive tests

---

## Testing Strategy

### Unit Tests
- Test each transformation in isolation
- Compare generated output to expected output

### Integration Tests
1. Compile `build_member_path.lux`
2. Run Babel output against `@babel/core`
3. Compile SWC output with `cargo build`
4. Run both against sample AST and compare results

### Real-World Test
Convert a complete helper from minimact and validate both outputs produce identical transformations.

---

## Success Criteria

The enhancement is complete when:

1. `reluxscript build examples/build_member_path.lux` generates:
   - Babel output that runs without errors
   - SWC output that compiles without errors

2. Both outputs produce identical results when run on the same input AST

3. At least one complete minimact helper is successfully converted and working

---

## Files to Modify

- `src/codegen/babel.rs` - Babel code generator
- `src/codegen/swc.rs` - SWC code generator
- `src/mapping/mod.rs` - May need additional mapping helpers
- `src/semantic/type_checker.rs` - May need type context for codegen

---

## References

- [SWC AST Documentation](https://rustdoc.swc.rs/swc_ecma_ast/)
- [Babel Types API](https://babeljs.io/docs/babel-types)
- ReluxScript Specification: `docs/reluxscript-specification.md`
- AST Mapping Tables: `src/mapping/`
