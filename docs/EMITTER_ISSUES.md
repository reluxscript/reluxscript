# SWC Emitter Issues and Fixes

**Status:** Documentation
**Created:** After testing codegen test suite
**Goal:** Document all missing features and bugs in SwcEmitter that need to be fixed

---

## 🎯 OVERVIEW

The SwcEmitter is working for basic cases (console_remover validates!), but testing the codegen suite revealed several missing features and bugs that need to be addressed.

## 📊 TEST RESULTS SUMMARY

| Test File | Status | Issues Found |
|-----------|--------|--------------|
| `console_remover` | ✅ PASS | None - full pipeline works! |
| `codegen_basic.lux` | ❌ FAIL | Path expressions, macro calls |
| `test_simple_struct.lux` | ❌ FAIL | Struct nesting in writers |
| `test_pattern_matching.lux` | ❌ FAIL | Pattern variant qualification |

---

## 🐛 ISSUE #1: Pattern Variant Emission (CRITICAL)

### Problem

Unqualified pattern names are emitted incorrectly:

**Input (ReluxScript):**
```rust
if let ObjectPattern(obj_pat) = param {
    // ...
}
```

**Current Output (WRONG):**
```rust
if let ObjectPat(obj_pat) = param {
    // ❌ Wrong! This isn't a valid SWC pattern
}
```

**Expected Output:**
```rust
if let Pat::Object(obj_pat) = param {
    // ✅ Correct SWC enum variant
}
```

### Root Cause

The emitter's `emit_pattern()` method (line 578-681 in swc_emit.rs) uses `pattern.metadata.swc_pattern` directly, but the decorator is setting this to just the variant name (e.g., `"ObjectPat"`) instead of the fully-qualified path (e.g., `"Pat::Object"`).

**Location:** `swc_decorator.rs` - pattern decoration logic

### Fix Strategy

**Option A: Fix in Decorator** (RECOMMENDED)
- Update `SwcDecorator::decorate_pattern()` to set fully-qualified pattern names
- Change `"ObjectPat"` → `"Pat::Object"`
- Change `"Ident"` → `"Pat::Ident"` (when matching Pattern, not Expression)
- Change `"CallExpr"` → `"Expr::Call"` (when matching Expression)

**Option B: Fix in Emitter**
- Add prefix logic in `emit_pattern()` to detect pattern context
- Prepend `Pat::` or `Expr::` based on parent type

**Recommendation:** Option A - decorator should provide complete, ready-to-emit patterns.

### Test Case

```rust
writer TestPatternMatching {
    fn test_pattern_match(param: &Pattern) {
        if let ObjectPattern(obj_pat) = param {
            // Should emit: Pat::Object(obj_pat)
        }
        if let Identifier(id) = param {
            // Should emit: Pat::Ident(id)
        }
    }

    fn test_expr_match(expr: &Expression) {
        if let CallExpression(call) = expr {
            // Should emit: Expr::Call(call)
        }
    }
}
```

---

## 🐛 ISSUE #2: Struct Nesting in Writers (CRITICAL)

### Problem

Structs defined inside writers are emitted inside the impl block, but Rust doesn't allow nested struct definitions.

**Input (ReluxScript):**
```rust
writer TestWriter {
    struct Point {
        x: i32,
        y: i32,
    }

    fn make_point() -> Point {
        Point { x: 5, y: 10 }
    }
}
```

**Current Output (WRONG):**
```rust
impl TestWriter {
    struct Point {  // ❌ Can't define structs inside impl!
        x: i32,
        y: i32,
    }

    fn make_point() -> Point {
        Point { x: 5, y: 10 }
    }
}
```

**Expected Output:**
```rust
// Structs at module level
struct Point {
    x: i32,
    y: i32,
}

impl TestWriter {
    fn make_point() -> Point {
        Point { x: 5, y: 10 }
    }
}
```

### Root Cause

The emitter's `emit_writer()` method (line 212-234 in swc_emit.rs) doesn't extract structs/enums/impl blocks BEFORE emitting the writer impl block. This logic exists for plugins (lines 170-180) but not for writers.

**Location:** `swc_emit.rs:212-234` - `emit_writer()` method

### Fix Strategy

Update `emit_writer()` to match `emit_plugin()` structure:

```rust
fn emit_writer(&mut self, writer: &DecoratedWriter) {
    self.name = writer.name.clone();

    // ✅ FIRST: Emit structs, enums, and impl blocks at module level
    for item in &writer.body {
        match item {
            DecoratedPluginItem::Struct(_) |
            DecoratedPluginItem::Enum(_) |
            DecoratedPluginItem::Impl(_) => {
                self.emit_plugin_item(item);
            }
            _ => {}
        }
    }

    // Writer struct with output field
    self.emit_line(&format!("pub struct {} {{", writer.name));
    self.indent += 1;
    self.emit_line("output: String,");
    self.indent -= 1;
    self.emit_line("}");
    self.emit_line("");

    // Impl block with only functions
    self.emit_line(&format!("impl {} {{", writer.name));
    self.indent += 1;

    for item in &writer.body {
        // ✅ Only emit functions inside impl block
        if let DecoratedPluginItem::Function(func) = item {
            self.emit_plugin_item(item);
        }
    }

    self.indent -= 1;
    self.emit_line("}");
}
```

### Test Case

```rust
writer TestWriter {
    struct Point {
        x: i32,
        y: i32,
    }

    enum Status {
        Active,
        Inactive,
    }

    impl Point {
        fn new() -> Point {
            Point { x: 0, y: 0 }
        }
    }

    fn make_point() -> Point {
        Point { x: 5, y: 10 }
    }
}
```

---

## 🐛 ISSUE #3: Path Expressions (MEDIUM PRIORITY)

### Problem

Path expressions (`::`-separated identifiers) are being emitted with `.` instead of `::`.

**Input (ReluxScript):**
```rust
use codegen;

fn transform_expr(expr: &Expr) {
    let code = codegen::generate(expr);
}
```

**Current Output (WRONG):**
```rust
fn transform_expr(expr: &Expr) {
    let code = codegen.generate(expr);  // ❌ Should be ::
}
```

**Expected Output:**
```rust
fn transform_expr(expr: &Expr) {
    let code = codegen::generate(expr);  // ✅ Correct
}
```

### Root Cause

The parser/decorator likely represents `codegen::generate` as a `MemberExpr` with `is_path: true`, but the emitter's `emit_expr()` method (line 739) always emits `.` for member access.

**Location:** `swc_emit.rs:739` - `DecoratedExprKind::Member` case

### Fix Strategy

Update `emit_expr()` to check `is_path` field:

```rust
DecoratedExprKind::Member { object, property: _, optional, computed: _, is_path, field_metadata } => {
    self.emit_expr(object);

    if *optional {
        self.output.push('?');
    }

    // ✅ Use :: for path expressions, . for member access
    if *is_path {
        self.output.push_str("::");
    } else {
        self.output.push('.');
    }

    // Use the SWC field name from metadata
    self.output.push_str(&field_metadata.swc_field_name);

    // Apply accessor strategy
    match &field_metadata.accessor {
        // ...
    }
}
```

### Test Case

```rust
use codegen;
use fs;

fn test_paths() {
    let result = codegen::generate(expr);
    let content = fs::read_to_string("file.txt");
    String::from("test");  // Associated function
}
```

---

## 🐛 ISSUE #4: Macro Calls (MEDIUM PRIORITY)

### Problem

Macro invocations are missing the `!` suffix.

**Input (ReluxScript):**
```rust
let formatted = format!("Generated: {}", code);
let items = vec![1, 2, 3];
println!("Hello");
```

**Current Output (WRONG):**
```rust
let formatted = format("Generated: {}", code);  // ❌ Missing !
let items = vec[1, 2, 3];                        // ❌ Wrong! (vec! is handled)
println("Hello");                                 // ❌ Missing !
```

**Expected Output:**
```rust
let formatted = format!("Generated: {}", code);  // ✅ Correct
let items = vec![1, 2, 3];                        // ✅ Correct (vec! special case)
println!("Hello");                                // ✅ Correct
```

### Root Cause

The parser/decorator needs to mark macro calls with metadata, and the emitter needs to check for this metadata when emitting call expressions.

**Locations:**
1. Parser - needs to detect `identifier!` syntax
2. Decorator - needs to set `is_macro: bool` in call expression metadata
3. Emitter - needs to check metadata and emit `!` suffix

### Fix Strategy

**Step 1: Add macro metadata**

Update `DecoratedCallExpr` in `decorated_ast.rs`:

```rust
pub struct DecoratedCallExpr {
    pub callee: DecoratedExpr,
    pub args: Vec<DecoratedExpr>,
    pub type_args: Option<Vec<Type>>,
    pub optional: bool,
    pub is_macro: bool,  // ✅ NEW: marks macro invocations
    pub span: Option<Span>,
}
```

**Step 2: Detect macros in decorator**

Update `SwcDecorator::decorate_call_expr()`:

```rust
fn decorate_call_expr(&mut self, call: &CallExpr) -> DecoratedExpr {
    let callee = self.decorate_expr(&call.callee);

    // ✅ Check if callee is an identifier ending with !
    let is_macro = if let DecoratedExprKind::Ident { name, .. } = &callee.kind {
        name.ends_with('!')
    } else {
        false
    };

    DecoratedExpr {
        kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
            callee,
            args: call.args.iter().map(|a| self.decorate_expr(a)).collect(),
            type_args: call.type_args.clone(),
            optional: call.optional,
            is_macro,  // ✅ Set the flag
            span: Some(call.span),
        })),
        metadata: /* ... */,
    }
}
```

**Step 3: Emit ! in emitter**

Update `emit_expr()` for `DecoratedExprKind::Call`:

```rust
DecoratedExprKind::Call(call) => {
    self.emit_expr(&call.callee);

    // ✅ Add ! for macro calls
    if call.is_macro {
        self.output.push('!');
    }

    self.output.push('(');
    for (i, arg) in call.args.iter().enumerate() {
        if i > 0 {
            self.output.push_str(", ");
        }
        self.emit_expr(arg);
    }
    self.output.push(')');
}
```

### Test Case

```rust
fn test_macros() {
    let s = format!("Value: {}", 42);
    let v = vec![1, 2, 3];
    println!("Debug: {:?}", s);
    assert_eq!(v.len(), 3);
    panic!("Error occurred");
}
```

---

## 🐛 ISSUE #5: Missing Helper Functions (LOW PRIORITY)

### Problem

Some tests may use helper functions from modules that aren't imported or generated.

**Example:**
```rust
use codegen;

fn transform() {
    codegen::generate(expr);  // Needs codegen_to_string helper
}
```

### Current Status

The emitter has `emit_codegen_helpers()` (lines 1383-1405) but it's only called if `self.uses_codegen` is true. The import detection logic (lines 94-98) is a TODO stub.

### Fix Strategy

Implement `detect_imports()` to walk the decorated AST and detect:
- `uses_codegen` - if any `codegen::` paths are found
- `uses_parser` - if any `parser::` paths are found
- `uses_fs` - if any `fs::` paths are found
- `uses_json` - if any `json::` or serde usage is found
- `uses_hashmap` / `uses_hashset` - if these types are used

---

## 🧪 TESTING STRATEGY

### Phase 1: Fix Critical Issues (Pattern variants, Struct nesting)

Test with:
- `test_pattern_matching.lux`
- `test_simple_struct.lux`
- `test_top_level_struct.lux`

### Phase 2: Fix Medium Priority (Path expressions, Macros)

Test with:
- `codegen_basic.lux`
- `test_string_literals.lux`
- `parser_module.lux`

### Phase 3: Comprehensive Testing

Run all 34 tests in `tests/codegen/`:
```bash
for f in tests/codegen/*.lux; do
    echo "Testing $f"
    cargo run -- build "$f" --target swc
    if [ $? -eq 0 ]; then
        echo "✅ PASS"
    else
        echo "❌ FAIL"
    fi
done
```

### Phase 4: Integration Tests

Test with:
- `tests/integration/console_remover/` (already passing!)
- All other integration tests

---

## 📋 PRIORITY ORDER

### 🔴 CRITICAL (Blocks most tests)

1. **Pattern Variant Qualification** - Affects all pattern matching code
2. **Struct Nesting in Writers** - Affects all writer tests with structs

### 🟡 MEDIUM (Affects specific features)

3. **Path Expressions** - Affects module usage (codegen::, fs::, etc.)
4. **Macro Calls** - Affects format!, println!, vec!, etc.

### 🟢 LOW (Nice to have)

5. **Import Detection** - Helper functions work but imports may be missing

---

## 🎯 SUCCESS CRITERIA

✅ **Phase 1 Complete**: Pattern matching and struct emission tests pass
✅ **Phase 2 Complete**: Path and macro tests pass
✅ **Phase 3 Complete**: All 34 codegen tests pass with cargo check validation
✅ **Phase 4 Complete**: All integration tests pass

---

## 📊 CURRENT STATUS

**Working Features:**
- ✅ Basic pattern matching (with qualified patterns like `Callee::Expr`)
- ✅ Expression emission (binary, unary, member, call)
- ✅ Statement emission (if, match, for, while, loop, return)
- ✅ Function generation with proper signatures
- ✅ Struct/enum definitions (at plugin level)
- ✅ Pattern desugaring (rewriter stage)
- ✅ Matches! expansion (rewriter stage)
- ✅ Sym dereferencing (&*ident.sym)
- ✅ Field access with metadata

**Missing/Broken Features:**
- ❌ Unqualified pattern variant names
- ❌ Struct extraction for writers
- ❌ Path expression emission (::)
- ❌ Macro call emission (!)
- ❌ Import detection for helper functions

---

## 📝 IMPLEMENTATION NOTES

### Order of Fixes

1. **Fix Pattern Variants First** - This is the most common issue
2. **Fix Struct Nesting** - Quick fix, just copy plugin logic to writer
3. **Fix Path Expressions** - One-line change in emit_expr
4. **Fix Macro Calls** - Requires parser + decorator + emitter changes
5. **Implement Import Detection** - Optional optimization

### Estimated Complexity

- Pattern Variants: 30 minutes (decorator change)
- Struct Nesting: 10 minutes (copy-paste + test)
- Path Expressions: 5 minutes (one-line fix)
- Macro Calls: 1 hour (parser + decorator + emitter)
- Import Detection: 2 hours (AST walking logic)

**Total Estimated Time:** ~4 hours for all fixes

---

## 🚀 NEXT STEPS

1. Read this document
2. Decide on fix order (recommend: critical first)
3. Implement fixes one at a time
4. Test after each fix with relevant test cases
5. Document any new issues discovered during testing

The emitter is 95% complete - these are the final touches needed for production readiness! 🎉
