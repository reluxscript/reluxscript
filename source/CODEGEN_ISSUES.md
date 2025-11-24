# ReluxScript Codegen Issues

Generated from: `tests/codegen/minimal/minimal_counter_transpiler.rsc`
Date: 2024-11-24

## Babel (JavaScript) Codegen Issues

### 1. Rust-specific method calls: `.as_str()`

**Location:** `dist/index.js` lines 83, 85, 88

**Issue:**
```javascript
if (callee.name.as_str() === "useState") {  // ❌ JavaScript strings don't have as_str()
```

**Fix:**
```javascript
if (callee.name === "useState") {  // ✅
```

**Root Cause:** The codegen is emitting Rust string methods instead of treating strings as JavaScript primitives.

---

### 2. Rust Option types: `Some()` and `None`

**Location:** `dist/index.js` line 105, line 71

**Issue:**
```javascript
return this.render_jsx = Some(jsx);  // ❌ Some is not defined in JavaScript
render_jsx: None  // ❌ None is not defined in JavaScript
```

**Fix:**
```javascript
this.render_jsx = jsx;  // ✅ Just assign the value
render_jsx: null  // ✅ Use null for empty
```

**Root Cause:** Option<T> types are being translated literally instead of using JavaScript null/undefined conventions.

---

### 3. Early returns in loops

**Location:** `dist/index.js` lines 84, 87, 89, 105, 145, 163

**Issue:**
```javascript
for (const declarator of var_decl.declarations) {
  if (callee.name === "useState") {
    return this.extract_use_state(declarator, call);  // ❌ Exits entire function, not just loop iteration!
  }
}
```

**Fix:**
```javascript
for (const declarator of var_decl.declarations) {
  if (callee.name === "useState") {
    this.extract_use_state(declarator, call);  // ✅ Just call, don't return
  }
}
```

**Root Cause:** Match arm bodies are being translated with `return` even when they shouldn't cause early exit from the enclosing function.

---

### 4. Functions with `self` parameter but using `this`

**Location:** `dist/index.js` line 124, 129, 139, etc.

**Issue:**
```javascript
function extract_use_state(self, declarator, call) {  // Has 'self' parameter
  const state_name = this.get_pattern_name(...);  // ❌ Uses 'this' instead of 'self'
}
```

**Fix:** Either:
- Option A: Remove `self` parameter and use `this` (JavaScript convention)
- Option B: Use `self` instead of `this` throughout the function body

**Root Cause:** Writer helper functions are generated with Rust-style `self` parameter but JavaScript code uses `this` for method calls.

---

### 5. String `.push()` method doesn't exist

**Location:** `dist/index.js` lines 298, 300, 303, 316, 319, 380, 396

**Issue:**
```javascript
result.push("{");  // ❌ Strings don't have push() in JavaScript
result.push(ch);   // ❌ Same issue
```

**Fix:**
```javascript
result += "{";  // ✅ Use string concatenation
result += ch;   // ✅ Or use .concat()
```

**Root Cause:** Rust's `String::push()` method is being emitted literally for JavaScript strings.

---

### 6. Missing return statements

**Location:** `dist/index.js` lines 196, 226-229

**Issue:**
```javascript
} else {
  "";  // ❌ No return statement, evaluates to undefined
}

if ((num.value === num.value.floor())) {
  "int".to_string();  // ❌ Not returned
}
```

**Fix:**
```javascript
} else {
  return "";  // ✅
}

if ((num.value === num.value.floor())) {
  return "int";  // ✅
}
```

**Root Cause:** Match/if-else arms that should return values are missing explicit `return` statements.

---

### 7. Unnecessary `.toString()` calls

**Location:** Throughout `dist/index.js`

**Issue:**
```javascript
"null".toString()  // ❌ Strings don't need toString()
"dynamic".toString()  // ❌ Same
```

**Fix:**
```javascript
"null"  // ✅ Already a string
"dynamic"  // ✅
```

**Root Cause:** String literals are being treated as if they need conversion to strings.

---

## SWC (Rust) Codegen Issues

### 1. Missing state fields in struct

**Location:** `dist/lib.rs` line 58-61

**Issue:**
```rust
pub struct MinimalCounterTranspiler {
    output: String,
    indent_level: usize,
    // ❌ Missing: component_name, use_state, use_effect, use_ref, event_handlers, render_jsx
}
```

But code at lines 98, 100, 121, 179, 207, 513 tries to access:
```rust
self.component_name  // ❌ Field doesn't exist
self.use_state       // ❌ Field doesn't exist
self.render_jsx      // ❌ Field doesn't exist
```

**Fix:**
The struct should include the State fields:
```rust
pub struct MinimalCounterTranspiler {
    output: String,
    indent_level: usize,
    component_name: String,           // ✅ Add state fields
    use_state: Vec<UseStateInfo>,     // ✅
    use_effect: Vec<UseEffectInfo>,   // ✅
    use_ref: Vec<UseRefInfo>,         // ✅
    event_handlers: Vec<EventHandler>,// ✅
    render_jsx: Option<JSXElement>,   // ✅
}
```

**Root Cause:** The writer's State struct is being generated separately (lines 11-18) but not incorporated into the main struct. The codegen should either:
- Inline the State fields into the main struct, OR
- Have the main struct contain a `state: State` field

---

### 2. `finish()` returns wrong type

**Location:** `dist/lib.rs` line 88

**Issue:**
```rust
pub fn finish(mut self) -> String {  // Returns String
    // ... generates output ...
    TranspilerOutput { csharp: lines.join("\n") }  // ❌ Returns TranspilerOutput
}
```

**Fix:**
```rust
pub fn finish(mut self) -> String {  // Returns String
    // ... generates output ...
    lines.join("\n")  // ✅ Return the actual string
}
```

OR change signature:
```rust
pub fn finish(mut self) -> TranspilerOutput {  // ✅ Match return type
    // ... generates output ...
    TranspilerOutput { csharp: lines.join("\n") }  // ✅
}
```

**Root Cause:** Function signature declares one return type but body returns a different type.

---

### 3. Missing semicolons after statements

**Location:** `dist/lib.rs` lines 103, 108, 117, 124, 132

**Issue:**
```rust
lines.push("".to_string())  // ❌ Missing semicolon
```

**Fix:**
```rust
lines.push("".to_string());  // ✅
```

**Root Cause:** Statement-level expressions are missing semicolons.

---

## Summary

### Critical Issues (Won't Compile):
- **Babel:** `Some()` and `None` undefined
- **Babel:** `.as_str()` method doesn't exist
- **Babel:** `.push()` on strings doesn't exist
- **SWC:** Missing struct fields (`component_name`, `use_state`, etc.)
- **SWC:** Type mismatch in `finish()` return
- **SWC:** Missing semicolons (syntax errors)

### Logical Issues (Compiles but Wrong Behavior):
- **Babel:** Early returns in loops terminate entire function
- **Babel:** `self` parameter but `this` usage mismatch
- **Babel:** Missing `return` statements in conditionals

### Minor Issues (Code Smell):
- **Babel:** Unnecessary `.toString()` on string literals

---

## Recommended Fix Priority

1. **SWC: Add state fields to struct** - Most critical, code won't compile
2. **Babel: Remove `.as_str()` calls** - JavaScript syntax error
3. **Babel: Replace `Some()`/`None` with JS equivalents** - JavaScript runtime error
4. **Babel: Fix string `.push()` to use `+=`** - JavaScript runtime error
5. **Babel: Remove early returns in loops** - Logic bug
6. **SWC: Fix `finish()` return type** - Type error
7. **Babel: Fix self/this parameter mismatch** - Incorrect binding
8. **Babel: Add missing return statements** - Logic bug
9. **SWC: Add missing semicolons** - Syntax error
10. **Babel: Remove unnecessary `.toString()`** - Code cleanup
