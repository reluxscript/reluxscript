# ReluxScript Codegen Root Cause Analysis

## Overview

This document analyzes the root causes of code generation issues and provides minimal test cases for each.

---

## Issue 1: `.as_str()` emitted to JavaScript

### Root Cause
**Location:** `src/codegen/babel.rs` line 1789-1810 (gen_expr for Expr::Ident)

When generating method calls like `name.as_str()`, the code falls through to the default case at line 2240-2248 which just emits the method call directly without checking if it's a Rust-specific method.

The problem: `.as_str()` is a Rust method to convert `&String` to `&str`. In JavaScript, strings are primitive and don't need conversion.

**Missing logic:** Need to add special handling for `.as_str()` around line 2130-2136 (where other method conversions live) to make it a no-op:

```rust
// s.as_str() -> s (no-op in JS, strings are already strings)
if prop == "as_str" {
    self.gen_expr(&mem.object);
    return;
}
```

### Minimal Test Case

**Input:** `tests/codegen/issues/test_as_str.rsc`
```rust
writer TestAsStr {
    fn check_name(name: &Str) -> bool {
        name.as_str() == "test"
    }
}
```

**Expected Babel Output:**
```javascript
function check_name(name) {
    return name === "test";  // ✅ No .as_str()
}
```

**Current Babel Output:**
```javascript
function check_name(name) {
    return name.as_str() === "test";  // ❌ Invalid JavaScript
}
```

---

## Issue 2: `Some()` and `None` emitted literally

### Root Cause
**Location:** `src/codegen/babel.rs` line 1792-1809 (gen_expr for Expr::Ident)

`Some` and `None` are just treated as regular identifiers and emitted as-is. There's no special handling to convert them to JavaScript null/undefined semantics.

**Missing logic:** Need to add special cases at line 1794-1797:

```rust
match ident.name.as_str() {
    "self" => self.emit("this"),
    "None" => self.emit("null"),  // ADD THIS
    "Some" => {
        // This is trickier - Some(x) should just be x
        // But Some as an identifier alone (rare) could be undefined
        self.emit("undefined")  // Or handle in Call expr
    },
    // ... rest
}
```

Better approach: Handle `Some(x)` at the Call expression level (line 1838-2249), checking if callee is "Some" and just emitting the argument.

### Minimal Test Case

**Input:** `tests/codegen/issues/test_option.rsc`
```rust
writer TestOption {
    struct State {
        name: Option<Str>,
    }

    fn init() -> State {
        State { name: None }
    }

    fn set_name(name: Str) {
        self.state.name = Some(name);
    }
}
```

**Expected Babel Output:**
```javascript
function init() {
    return { name: null };  // ✅
}

function set_name(name) {
    this.state.name = name;  // ✅ Just assign
}
```

**Current Babel Output:**
```javascript
function init() {
    return { name: None };  // ❌ None is undefined
}

function set_name(name) {
    return this.state.name = Some(name);  // ❌ Some is undefined
}
```

---

## Issue 3: String `.push()` method

### Root Cause
**Location:** `src/codegen/babel.rs` around line 2130

The codegen handles `push_str` (line 2131-2136) but not `push`. When Rust code uses `string.push(ch)`, it falls through to default method call emission.

In Rust: `String::push(char)` appends a character.
In JavaScript: Strings are immutable; should use `+=` for concatenation.

**Missing logic:** Need to add around line 2136:

```rust
// s.push(ch) -> s += ch
if prop == "push" && call.args.len() == 1 {
    self.gen_expr(&mem.object);
    self.emit(" += ");
    self.gen_expr(&call.args[0]);
    return;
}
```

### Minimal Test Case

**Input:** `tests/codegen/issues/test_string_push.rsc`
```rust
writer TestStringPush {
    fn add_char(s: &mut Str, ch: char) {
        s.push(ch);
        s.push_str(" done");
    }
}
```

**Expected Babel Output:**
```javascript
function add_char(s, ch) {
    s += ch;          // ✅
    s += " done";     // ✅ (already works)
}
```

**Current Babel Output:**
```javascript
function add_char(s, ch) {
    s.push(ch);       // ❌ strings don't have .push()
    s += " done";     // ✅ (already works)
}
```

---

## Issue 4: Early returns in loops

### Root Cause
**Location:** Multiple places - likely in `gen_stmt` for match expressions and loops

When generating match arms inside loops, every arm gets a `return` statement. This is correct for function-level matches but incorrect for matches inside loops where we want to continue/break the loop, not exit the function.

The issue is in how match expressions are compiled. Need to track context: are we inside a loop? If so, match arm returns should not emit `return`.

**Location to fix:** Around line 1100-1200 where match statements are generated.

### Minimal Test Case

**Input:** `tests/codegen/issues/test_loop_return.rsc`
```rust
writer TestLoopReturn {
    fn process_items(items: Vec<Str>) {
        for item in items {
            match item.as_str() {
                "skip" => continue,
                "stop" => break,
                _ => println!("Processing: {}", item),
            }
        }
    }
}
```

**Expected Babel Output:**
```javascript
function process_items(items) {
    for (const item of items) {
        if (item === "skip") {
            continue;  // ✅
        } else if (item === "stop") {
            break;     // ✅
        } else {
            console.log(`Processing: ${item}`);
        }
    }
}
```

**Current Babel Output:**
```javascript
function process_items(items) {
    for (const item of items) {
        if (item === "skip") {
            return;  // ❌ Exits entire function!
        } else if (item === "stop") {
            return;  // ❌ Exits entire function!
        } else {
            console.log(`Processing: ${item}`);
        }
    }
}
```

---

## Issue 5: `self` parameter with `this` usage

### Root Cause
**Location:** `src/codegen/babel.rs` around line 700-800 (function parameter generation)

When generating function signatures, `self` parameters are being emitted literally as `self` in JavaScript. But then in the function body, `self` is correctly translated to `this` (line 1795).

This creates a mismatch: the parameter is named `self` but the code uses `this`.

**Two solutions:**
1. Don't emit `self` as a parameter (JavaScript methods don't have explicit `this` parameter)
2. Emit `self` and use `self` instead of `this` in the body

Solution 1 is more idiomatic JavaScript.

**Location to fix:** In function parameter generation (around line 700-750).

### Minimal Test Case

**Input:** `tests/codegen/issues/test_self_param.rsc`
```rust
writer TestSelfParam {
    struct State {
        count: i32,
    }

    fn increment(self: &mut Self) {
        self.state.count += 1;
    }
}
```

**Expected Babel Output (Option A):**
```javascript
function increment() {  // ✅ No self parameter
    this.state.count += 1;
}
```

**Expected Babel Output (Option B):**
```javascript
function increment(self) {  // Has self parameter
    self.state.count += 1;  // Use self not this
}
```

**Current Babel Output:**
```javascript
function increment(self) {  // ❌ Has self parameter
    this.state.count += 1;  // ❌ But uses this!
}
```

---

## Issue 6: Missing return statements

### Root Cause
**Location:** `src/codegen/babel.rs` in expression statement generation

In Rust, the last expression in a block is automatically returned. JavaScript requires explicit `return` statements.

The codegen likely has logic to add `return` for the last statement in a function, but it's not working for all cases (especially in if/else branches).

**Location to fix:** Around line 900-1000 (statement generation) - need to track if we're generating the last expression in a block and whether that block is in a return position.

### Minimal Test Case

**Input:** `tests/codegen/issues/test_missing_return.rsc`
```rust
writer TestMissingReturn {
    fn get_type(is_int: bool) -> Str {
        if is_int {
            "int"
        } else {
            "float"
        }
    }
}
```

**Expected Babel Output:**
```javascript
function get_type(is_int) {
    if (is_int) {
        return "int";    // ✅
    } else {
        return "float";  // ✅
    }
}
```

**Current Babel Output:**
```javascript
function get_type(is_int) {
    if (is_int) {
        "int";      // ❌ Not returned
    } else {
        "float";    // ❌ Not returned
    }
}
```

---

## Issue 7: Unnecessary `.toString()` on string literals

### Root Cause
**Location:** `src/codegen/babel.rs` line 2222-2226

The code has a special case for `.to_string()` that makes it a no-op (just emits the object). This is correct!

But the issue is that string LITERALS are calling `.toString()`. This suggests the AST has `Expr::Call` with `"string_literal".to_string()` structure, when it should just be `Expr::Literal("string_literal")`.

This is likely a **parser/AST construction issue**, not a codegen issue. Or it's in how struct fields are being generated with default values.

**Location to investigate:** Where struct literals with string field values are generated.

### Minimal Test Case

**Input:** `tests/codegen/issues/test_string_literal_tostring.rsc`
```rust
writer TestStringLiteral {
    fn get_default_type() -> Str {
        "null".to_string()
    }
}
```

**Expected Babel Output:**
```javascript
function get_default_type() {
    return "null";  // ✅ Just the string
}
```

**Current Babel Output:**
```javascript
function get_default_type() {
    return "null".toString();  // ❌ Unnecessary call
}
```

---

## SWC Issues

### Issue 8: Missing state fields in struct

### Root Cause
**Location:** `src/codegen/swc.rs` - writer struct generation

For writers, there's a State struct defined separately. The main writer struct should either:
1. Flatten the State fields into the main struct
2. Include a `state: State` field

Currently, the codegen generates the main struct with only builder-related fields (output, indent_level) but not the State fields, yet the methods try to access `self.component_name`, `self.use_state`, etc.

**Location to fix:** Around line 300-400 (writer struct generation).

### Minimal Test Case

**Input:** `tests/codegen/issues/test_writer_state.rsc`
```rust
writer TestWriterState {
    struct State {
        component_name: Str,
        count: i32,
    }

    fn init() -> State {
        State {
            component_name: String::new(),
            count: 0,
        }
    }

    fn process() {
        self.component_name = "Test".to_string();
        self.count += 1;
    }
}
```

**Expected SWC Output:**
```rust
pub struct TestWriterState {
    output: String,
    indent_level: usize,
    component_name: String,  // ✅ State fields included
    count: i32,              // ✅
}
```

**Current SWC Output:**
```rust
pub struct TestWriterState {
    output: String,
    indent_level: usize,
    // ❌ Missing: component_name, count
}

// State struct generated separately but not used
struct State {
    component_name: String,
    count: i32,
}
```

---

### Issue 9: `finish()` return type mismatch

### Root Cause
**Location:** `src/codegen/swc.rs` - finish() method generation

The function signature declares `-> String` but the body returns a struct `TranspilerOutput { csharp: ... }`.

This happens when the ReluxScript source declares finish() with a return type, but the body constructs a different type.

**Location to fix:** Either:
1. Fix the return type inference/generation
2. Fix the return expression to match the declared type

### Minimal Test Case

**Input:** `tests/codegen/issues/test_finish_return_type.rsc`
```rust
writer TestFinishReturnType {
    struct TranspilerOutput {
        csharp: Str,
    }

    fn finish(&self) -> Str {
        TranspilerOutput {
            csharp: "output".to_string()
        }
    }
}
```

**Expected SWC Output (Option A):**
```rust
pub fn finish(&self) -> String {
    "output".to_string()  // ✅ Match return type
}
```

**Expected SWC Output (Option B):**
```rust
pub fn finish(&self) -> TranspilerOutput {  // ✅ Match return value
    TranspilerOutput {
        csharp: "output".to_string()
    }
}
```

**Current SWC Output:**
```rust
pub fn finish(&self) -> String {  // Declares String
    TranspilerOutput {                // ❌ Returns TranspilerOutput!
        csharp: "output".to_string()
    }
}
```

---

### Issue 10: Missing semicolons

### Root Cause
**Location:** `src/codegen/swc.rs` - statement generation

In Rust, statements need semicolons. The codegen is sometimes emitting expressions without semicolons when they should be statements.

Need to ensure that when generating statements (not the final expression in a block), semicolons are added.

**Location to fix:** Statement generation logic (around line 600-900).

### Minimal Test Case

**Input:** `tests/codegen/issues/test_semicolons.rsc`
```rust
writer TestSemicolons {
    fn append_lines(lines: &mut Vec<Str>) {
        lines.push("line1".to_string());
        lines.push("line2".to_string());
        lines.push("line3".to_string())
    }
}
```

**Expected SWC Output:**
```rust
fn append_lines(lines: &mut Vec<String>) {
    lines.push("line1".to_string());  // ✅
    lines.push("line2".to_string());  // ✅
    lines.push("line3".to_string());  // ✅ Last one too (not return position)
}
```

**Current SWC Output:**
```rust
fn append_lines(lines: &mut Vec<String>) {
    lines.push("line1".to_string())  // ❌ Missing semicolon
    lines.push("line2".to_string())  // ❌ Missing semicolon
    lines.push("line3".to_string())  // ❌ Missing semicolon
}
```

---

## Summary Table

| Issue | Root Cause Location | Fix Type | Priority |
|-------|-------------------|----------|----------|
| `.as_str()` | `babel.rs:2130` | Add special case | High |
| `Some`/`None` | `babel.rs:1794` | Add special cases | Critical |
| String `.push()` | `babel.rs:2136` | Add special case | Critical |
| Early returns | `babel.rs:1100-1200` | Context tracking | High |
| `self`/`this` mismatch | `babel.rs:700-750` | Skip self param | Medium |
| Missing returns | `babel.rs:900-1000` | Return position tracking | High |
| `.toString()` on literals | Parser/AST issue? | Investigate parser | Low |
| Missing state fields | `swc.rs:300-400` | Flatten State struct | Critical |
| `finish()` type | `swc.rs` finish gen | Fix return type/value | High |
| Missing semicolons | `swc.rs:600-900` | Add semicolons | Critical |

---

## Next Steps

1. Create the minimal test case files
2. Verify they reproduce the issues
3. Fix issues one by one in priority order
4. Run tests after each fix to verify
5. Update CODEGEN_ISSUES.md with "FIXED" markers

