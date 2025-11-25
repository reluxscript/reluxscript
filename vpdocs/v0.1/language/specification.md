# ReluxScript Language Specification

**Version:** 0.9.0
**Status:** Draft
**Target Platforms:** Babel (JavaScript) & SWC (Rust/WASM)

## Overview

ReluxScript is a domain-specific language for writing AST transformation plugins that compile to both Babel (JavaScript) and SWC (Rust). It enforces a strict visitor pattern with explicit ownership semantics that map cleanly to both garbage-collected and borrow-checked runtimes.

## Design Philosophy

- **Strict Visitor, Loose Context**: Enforces Rust-like `VisitMut` pattern
- **Immutable by Default**: All mutations must be explicit
- **No Path Magic**: No implicit parent traversal; state must be tracked explicitly
- **Unified AST**: Subset of nodes common to ESTree (Babel) and swc_ecma_ast
- **Clone-to-Own**: Explicit `.clone()` required for value extraction

## Vector Alignment Principle

ReluxScript finds the **intersection** of JavaScript and Rust capabilities, not the union. Code that compiles must be semantically valid in both targets.

This means:
- ✅ What compiles will work on both platforms
- ✅ No surprises, no "works in Babel but not SWC"
- ✅ Predictable, reliable transformations

## Complete Specification

For the full language specification including:
- Type system details
- Complete AST node mappings
- Module system
- Writer API for transpilers
- Error handling with `Result<T, E>`
- Advanced patterns (traverse, verbatim blocks, program hooks)

See the complete specification in the repository:
[reluxscript-specification.md](https://github.com/reluxscript/reluxscript/blob/main/docs/reluxscript-specification.md)

## Quick Reference

### Plugin Structure

```reluxscript
plugin MyPlugin {
    // Visitor methods
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // transformation logic
    }
}
```

### Common Patterns

**Node Replacement:**
```reluxscript
*node = Statement::empty();
```

**Pattern Matching:**
```reluxscript
if matches!(node.callee, "console.log") {
    // matched
}
```

**Error Handling:**
```reluxscript
fn process() -> Result<(), Str> {
    let ast = parser::parse_file("file.tsx")?;
    Ok(())
}
```

### Type System

| ReluxScript | Babel (JS) | SWC (Rust) |
|------------|------------|------------|
| `Str` | `string` | `String` |
| `i32` | `number` | `i32` |
| `bool` | `boolean` | `bool` |
| `Vec<T>` | `Array` | `Vec<T>` |
| `Option<T>` | `T \| null` | `Option<T>` |
| `Result<T,E>` | `{ok,value,error}` | `Result<T,E>` |

## Next Steps

- [Core Concepts](/v0.1/guide/concepts)
- [Types Deep Dive](/v0.1/language/types)
- [Pattern Matching](/v0.1/language/pattern-matching)
- [AST Node Reference](/v0.1/language/node-types)
