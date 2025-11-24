# ReluxScript Implementation Gaps

Tracking features that ReluxScript doesn't support yet, discovered while porting the minimact transpiler.

## Parser Gaps

### 1. `Self` type in return position
**Error**: `Parse error at 25:18: Expected type name`
**Code**: `fn init() -> Self { ... }`
**Workaround**: Use explicit type name `fn init() -> State { ... }`

### 2. Associated function calls with `::` on custom types
**Error**: `Parse error at 26:22: Expected expression`
**Code**: `let csharp = CodeBuilder::new();`
**Status**: `HashMap::new()` works (built-in), but custom types don't
**Workaround**: TBD - may need to implement `::` syntax for custom structs

### 3. Function calls as struct field initializers
**Error**: `Parse error at 27:21: Expected expression`
**Code**:
```reluxscript
State {
    csharp: CodeBuilder::new(),  // Error here
    templates: HashMap::new(),
}
```
**Workaround**: Assign to variables first, then use in struct literal

## Semantic Gaps

(None discovered yet)

## Codegen Gaps

### 1. `HashMap.len()` generates `.length` instead of `.size`
**Target**: Babel (JavaScript)
**Expected**: `map.size`
**Actual**: `map.length`
**File**: `reluxscript/src/codegen/babel.rs`

## Feature Gaps

### 1. `impl` blocks for custom structs
**Status**: Parser accepts `impl` blocks (parser.rs:128-129) BUT they fail at runtime
**Error**: `Parse error at 83:15: Expected identifier` on line `fn finish(self) -> TranspilerOutput {`
**Test Case**: `impl CodeBuilder { fn new() -> Self { ... } }` in writer body
**Theory**: Parser accepts `impl` keyword but function parser doesn't handle `self`/`&self`/`&mut self` parameters
**Workaround**: Use standalone functions like `fn code_builder_new() -> CodeBuilder`
**Impact**: HIGH - This is a significant limitation that makes code verbose and non-idiomatic. Every method call becomes `code_builder_append(&mut builder, text)` instead of `builder.append(text)`.

### 2. Associated function calls (`Type::method()`) on custom types
**Status**: Only works for built-ins (`HashMap::new()`, `String::new()`)
**Workaround**: Use standalone functions

### 3. Method chaining on custom types
**Status**: Unknown - need to test if custom methods can be called with `.`

### 4. Writer lifecycle (`init()` and `finish()`)
**Status**: Unclear how these are supposed to work
**Issue**: Writer needs state across visits, and a final output method
**Current approach**: `fn init() -> State` and `fn finish(&self) -> Output` both fail
**Theory**: Writer might need special lifecycle methods that aren't regular functions
**Needs investigation**: How does writer state management actually work?

---

*Updated as gaps are discovered during minimact transpiler port*
