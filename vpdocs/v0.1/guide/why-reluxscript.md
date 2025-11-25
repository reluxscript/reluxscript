# Why ReluxScript?

## The Problem

Building AST transformation plugins is hard, and maintaining them across multiple platforms is even harder.

### The Current Landscape

**Babel Plugins (JavaScript)**
- ✅ Flexible and easy to write
- ✅ Rich ecosystem
- ❌ Slow (JavaScript runtime)
- ❌ No type safety
- ❌ Large runtime overhead

**SWC Plugins (Rust)**
- ✅ Fast (native Rust)
- ✅ Type-safe
- ✅ WASM support
- ❌ Complex to write
- ❌ Steep learning curve
- ❌ Smaller ecosystem

**The Duplication Problem**
- Maintaining both = 2x the code, 2x the bugs, 2x the work
- Features added to one but not the other
- Subtle behavioral differences between implementations

## The Solution

ReluxScript lets you **write once, compile to both**.

```reluxscript
// Write this once
plugin RemoveConsole {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if matches!(node.callee, "console.log") {
            *node = Statement::empty();
        }
    }
}
```

**Compiles to both:**
- `dist/index.js` - Babel plugin (JavaScript)
- `dist/lib.rs` - SWC plugin (Rust)

## Key Benefits

### 1. Write Once, Run Everywhere

Stop maintaining duplicate codebases. Write your plugin logic once in ReluxScript, and it automatically compiles to both Babel and SWC.

### 2. Type Safety

Catch errors at compile time with static type checking:

```reluxscript
// This won't compile - type mismatch
let name: Str = 42;  // ERROR: expected Str, found i32
```

### 3. Vector Intersection Principle

We only support features that work **identically** in both Babel and SWC:

- ✅ What compiles will work correctly on both platforms
- ✅ No surprises or platform-specific bugs
- ✅ Predictable behavior

Better to support 80% of use cases **perfectly** than 100% with edge case bugs.

### 4. Performance

Get the best of both worlds:

- **Development**: Fast iteration with Babel (Node.js)
- **Production**: Native performance with SWC (Rust/WASM)

### 5. Familiar Syntax

Rust-inspired syntax that feels natural:

```reluxscript
// Clean, expressive code
for arg in &node.arguments {
    if let Some(value) = get_string_value(arg) {
        println!("Argument: {}", value);
    }
}
```

### 6. Extensible Beyond Babel & SWC

ReluxScript isn't limited to Babel and SWC. The compiler architecture supports **custom code generation targets**.

Want to compile TypeScript/JSX to C#? Go? Python? Add a new codegen backend!

## Who It's For

### Plugin Authors

Stop maintaining duplicate codebases. Write once, target both ecosystems.

**Before ReluxScript:**
- 500 lines of Babel plugin code
- 500 lines of SWC plugin code
- 2x maintenance burden

**With ReluxScript:**
- 300 lines of ReluxScript
- Auto-generates both targets
- Single source of truth

### Tool Builders

Use ReluxScript as your plugin format. Let users write once, deploy everywhere.

**Example:** Build a bundler that accepts ReluxScript plugins and automatically provides both Babel and SWC compatibility.

### Framework Teams

Build custom transpilers from TypeScript/JSX to your target language.

**Example:** Minimact uses ReluxScript to transpile React/TSX to C# for ASP.NET Core.

## Comparison

| Feature | Babel Only | SWC Only | ReluxScript |
|---------|-----------|----------|-------------|
| **Language** | JavaScript | Rust | ReluxScript |
| **Targets** | Babel | SWC | Both |
| **Type Safety** | ❌ | ✅ | ✅ |
| **Easy to Write** | ✅ | ❌ | ✅ |
| **Performance** | Slow | Fast | Fast in production |
| **Maintenance** | 1 codebase | 1 codebase | 1 codebase |
| **Cross-platform** | ❌ | ❌ | ✅ |

## Real-World Use Case

### Minimact Plugin

The Minimact plugin (123 files, ~15K lines) was originally written in JavaScript as a Babel plugin. Converting to ReluxScript provides:

1. **Dual compilation** - Works with both Babel and SWC toolchains
2. **Type safety** - Catches errors before runtime
3. **Better performance** - SWC version runs natively
4. **Single codebase** - One implementation, two outputs

## Design Principles

### 1. Correctness Over Coverage

We choose to support a smaller feature set **perfectly** rather than a larger feature set with edge cases.

If a feature can't work identically on both platforms, we don't include it.

### 2. Explicit Over Implicit

All mutations must be explicit:

```reluxscript
*node = Statement::empty();  // Clear: replacing node
node.name.clone()             // Clear: copying value
```

No magic, no surprises.

### 3. Performance Through Options

Choose your performance profile:

- **Development**: Use Babel output for fast iteration
- **Production**: Use SWC output for maximum performance
- **Both**: Test in Babel, ship with SWC

## Getting Started

Ready to try ReluxScript?

1. [Install ReluxScript](/v0.1/guide/getting-started)
2. [Create your first plugin](/v0.1/guide/getting-started#your-first-plugin)
3. [Learn the language](/v0.1/language/syntax)
4. [Explore examples](/v0.1/examples/)

## Philosophy

> "The best code is code you only have to write once."

ReluxScript embodies this philosophy. Write your AST transformation logic once, in a clean, type-safe language, and let the compiler handle the complexity of generating platform-specific code.

## Next Steps

- [Getting Started](/v0.1/guide/getting-started)
- [Core Concepts](/v0.1/guide/concepts)
- [Language Syntax](/v0.1/language/syntax)
