# ReluxScript Codegen Issues - Kitchen Sink Tests

This document catalogs the codegen bugs discovered from the kitchen sink test files, along with minimal reproducible test cases.

---

## Issue #1: `.unwrap_or()` generates invalid Babel syntax

**Severity:** High (breaks Babel codegen)
**Affected Target:** Babel only
**Test File:** `test_unwrap_or.lux`

### Description

When `.unwrap_or()` is called on an `Option<T>`, the Babel generator produces `.??()` instead of valid JavaScript.

### Minimal Test Case

```rust
plugin TestUnwrapOr {
    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        let maybe_name: Option<Str> = Some("test".into());
        let name = maybe_name.unwrap_or("default".into());
    }
}
```

### Generated Output (INVALID)

```javascript
const name = maybe_name.??("default".into());
                        ^^  // Invalid JavaScript syntax
```

### Expected Output

Should generate either:
- Nullish coalescing: `const name = maybe_name ?? "default"`
- Or a helper function call

### Location

Likely in `source/src/codegen/babel.rs` where method calls are generated. The `.unwrap_or()` method call is being incorrectly translated to `.??()` instead of proper JavaScript.

---

## Issue #2: `Self::` function calls fail to parse

**Severity:** High (breaks writers with static methods)
**Affected Target:** Both (parse error)
**Test File:** `test_self_call.lux`

### Description

Calling static/associated functions using `Self::method()` syntax causes a parse error: "Expected LBrace".

### Minimal Test Case

```rust
writer TestSelfCall {
    fn is_valid(name: &Str) -> bool {
        !name.is_empty()
    }

    fn visit_identifier(node: &Identifier) {
        if Self::is_valid(&node.name) {
            self.output.write("valid");
        }
    }
}
```

### Error

```
Parse error at 12:16: Expected LBrace
```

The parser fails when it encounters `Self::is_valid` on line 12.

### Analysis

The parser doesn't recognize `Self::` as a valid call expression prefix. It likely expects:
- Direct function calls: `is_valid(...)`
- Method calls: `self.method(...)`

But doesn't handle:
- Associated function calls: `Self::method(...)`

### Location

`source/src/parser/parser.rs` in the expression parsing logic. Need to handle `TokenKind::SelfType` followed by `TokenKind::ColonColon` in call expression parsing.

---

## Issue #3: State field not added to SWC plugin struct

**Severity:** High (breaks SWC codegen with state)
**Affected Target:** SWC only
**Test File:** `test_state_field.lux`

### Description

When a plugin defines a `State` struct and accesses `self.state.field` in visitor methods, the SWC generator:
1. ✓ Generates the `State` struct correctly
2. ✗ Does NOT add a `state` field to the plugin struct
3. ✗ Generates code that accesses `self.state.field` anyway

This causes Rust compilation errors: "no field `state` on type `&mut PluginName`"

### Minimal Test Case

```rust
plugin TestStateField {
    struct State {
        count: i32,
    }

    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        self.state.count += 1;
    }
}
```

### Generated Output (INVALID)

```rust
#[derive(Debug, Clone)]
pub struct State {
    pub count: i32,
}

pub struct TestStateField {
    // Plugin state
    // ❌ No state field here!
}

impl VisitMut for TestStateField {
    fn visit_mut_ident(&mut self, n: &mut Ident) {
        self.state.count += 1  // ❌ Accessing non-existent field
    }
}
```

### Expected Output

```rust
pub struct TestStateField {
    state: State,  // ✓ Should be added
}

impl TestStateField {
    pub fn new() -> Self {
        Self {
            state: State { count: 0 }  // ✓ Initialize state
        }
    }
}
```

### Babel Comparison

Babel handles this correctly because it uses JavaScript's plugin state system:
```javascript
let state = {};  // Global plugin state

return {
  visitor: {
    Identifier(path) {
      this.state.count += 1;  // Uses Babel's built-in state
    }
  }
};
```

### Location

`source/src/codegen/swc.rs` in the plugin struct generation:
1. Need to detect if State struct is defined
2. Add `state: State` field to plugin struct
3. Initialize state field in `new()` method with default values

---

## Summary

| Issue | Severity | Target | Status |
|-------|----------|--------|--------|
| `.unwrap_or()` → `.??()` | High | Babel | Needs fix |
| `Self::method()` parse error | High | Both | Needs fix |
| Missing state field in SWC | High | SWC | Needs fix |

All three issues have minimal reproducible test cases in `source/examples/`:
- `test_unwrap_or.lux`
- `test_self_call.lux`
- `test_state_field.lux`

These should be fixed before the kitchen sink tests can pass.
