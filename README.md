# ReluxScript

<p align="center">
  <img src="./assets/lux-image-4.png" alt="ReluxScript Logo" width="250">
</p>

<p align="center">
  <strong>Write once, compile everywhere</strong><br>
  A unified language for building AST transformation plugins<br>
  <em>/ˈreɪ.lʌks.skrɪpt/ • ray-lucks-script</em>
</p>

<p align="center">
  <a href="#quick-start">Quick Start</a> •
  <a href="#features">Features</a> •
  <a href="#examples">Examples</a> •
  <a href="#documentation">Documentation</a> •
  <a href="#roadmap">Roadmap</a>
</p>

---

## What is ReluxScript?

ReluxScript is a **domain-specific language** for writing AST transformation plugins that compile to **both Babel (JavaScript) and SWC (Rust)**. Write your plugin logic once in ReluxScript, and generate production-ready plugins for both ecosystems.

```reluxscript
plugin RemoveConsole {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if let Callee::MemberExpression(ref member) = node.callee {
            if let Expression::Identifier(ref obj) = *member.object {
                if obj.name == "console" {
                    ctx.remove();
                }
            }
        }
    }
}
```

**Compiles to:**

<table>
<tr>
<td width="50%">

**Babel (JavaScript)**
```javascript
module.exports = function({ types: t }) {
  return {
    visitor: {
      CallExpression(path) {
        if (path.node.callee.type === 'MemberExpression') {
          if (path.node.callee.object.name === 'console') {
            path.remove();
          }
        }
      }
    }
  };
};
```

</td>
<td width="50%">

**SWC (Rust)**
```rust
pub struct RemoveConsole;

impl VisitMut for RemoveConsole {
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        if let Callee::Expr(box Expr::Member(member)) = &node.callee {
            if let Expr::Ident(obj) = &*member.obj {
                if obj.sym == "console" {
                    // Remove node
                }
            }
        }
    }
}
```

</td>
</tr>
</table>

## Why ReluxScript?

### 🎯 **Vector Intersection Philosophy**

ReluxScript follows the **"vector intersection" principle**: only features that work identically in both JavaScript and Rust are supported. This ensures your plugins behave consistently across both targets.

### 🚀 **Dual Compilation**

- **Babel target**: Generate JavaScript plugins for Node.js/browser ecosystems
- **SWC target**: Generate Rust plugins for maximum performance
- **One source**: Maintain a single codebase for both

### 🔒 **Type Safety**

- Strong static typing with bidirectional type inference
- Catches errors at compile-time, not runtime
- Full AST node type checking

### 📦 **Unified AST**

- Works with a subset common to ESTree (Babel) and swc_ecma_ast (SWC)
- Seamless mapping between JavaScript and Rust AST representations
- No impedance mismatch

### ⚡ **Rust-like Ownership**

- Explicit `&` and `&mut` references
- `.clone()` required for value extraction
- Borrow checker validation (for SWC target)

## Quick Start

### Installation

```bash
# Clone the repository
git clone https://github.com/yourusername/reluxscript.git
cd reluxscript

# Build the compiler
cargo build --release

# Add to PATH (optional)
export PATH=$PATH:$(pwd)/target/release
```

### Your First Plugin

Create a file `remove_debugger.lux`:

```reluxscript
/// Remove debugger statements from code
plugin RemoveDebugger {
    fn visit_debugger_statement(node: &mut DebuggerStatement, ctx: &Context) {
        ctx.remove();
    }
}
```

### Compile to Babel

```bash
reluxc build remove_debugger.lux --target babel
```

Generates `dist/index.js` - a ready-to-use Babel plugin!

### Compile to SWC

```bash
reluxc build remove_debugger.lux --target swc
```

Generates `dist/lib.rs` - a ready-to-use SWC plugin!

### Use Your Plugin

**With Babel:**
```javascript
// babel.config.js
module.exports = {
  plugins: [require('./dist/index.js')]
};
```

**With SWC:**
```toml
# .swcrc
[jsc]
experimental = { plugins = [["./dist/lib.so", {}]] }
```

## Features

### ✅ Currently Supported

- **Visitor Pattern**: Mutable AST traversal with `visit_*` methods
- **Type System**: `Str`, `i32`, `f64`, `bool`, `Vec<T>`, `HashMap<K,V>`, `Option<T>`, `Result<T,E>`
- **Pattern Matching**: `if let`, `match` expressions
- **Structs & Enums**: User-defined types
- **Functions**: Free functions and methods
- **String Methods**: `.starts_with()`, `.ends_with()`, `.contains()`, `.len()`, etc.
- **Format Strings**: `format!("Hello, {}!", name)`
- **Import/Export**: Multi-file projects with `use` declarations
- **Verbatim Blocks**: `babel! { }` and `swc! { }` for platform-specific code

### 🚧 In Development

- **Regex Support**: `Regex::matches()`, `Regex::find()`, `Regex::captures()` (see [REGEX_SUPPORT.md](docs/REGEX_SUPPORT.md))
- **Custom AST Properties**: Unified metadata tracking across both targets (see [CUSTOM_AST_PROPERTIES.md](docs/CUSTOM_AST_PROPERTIES.md))

### ❌ Not Supported

- Async/await (different semantics)
- External library imports (no cross-platform guarantee)
- Direct DOM/Node.js APIs
- Closures capturing mutable state

See [reluxscript-specification.md](docs/reluxscript-specification.md) for full language details.

## Examples

### Remove Console Logs

```reluxscript
plugin RemoveConsole {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if let Callee::MemberExpression(ref member) = node.callee {
            if let Expression::Identifier(ref obj) = *member.object {
                if obj.name == "console" {
                    ctx.remove();
                }
            }
        }
    }
}
```

### Transform Arrow Functions to Regular Functions

```reluxscript
plugin ArrowToFunction {
    fn visit_arrow_function_expression(node: &mut ArrowFunctionExpression, ctx: &Context) {
        let func = FunctionExpression {
            id: None,
            params: node.params.clone(),
            body: node.body.clone(),
            async_: node.async_,
            generator: false,
        };

        *node = func;
    }
}
```

### Add JSX Keys to Array Children

```reluxscript
plugin AddJSXKeys {
    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        // Check if element is inside map call
        if ctx.is_inside_map() {
            // Check if key attribute exists
            let has_key = node.opening_element.attributes
                .iter()
                .any(|attr| attr.name == "key");

            if !has_key {
                // Add key attribute
                let key_attr = JSXAttribute {
                    name: JSXIdentifier { name: "key".into() },
                    value: Some(JSXAttributeValue::StringLiteral(
                        StringLiteral { value: generate_key() }
                    )),
                };
                node.opening_element.attributes.push(key_attr);
            }
        }
    }
}
```

### Extract Hook Dependencies

```reluxscript
plugin HookAnalyzer {
    struct State {
        dependencies: Vec<Str>,
    }

    fn init() -> State {
        State { dependencies: vec![] }
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if let Callee::Identifier(ref ident) = node.callee {
            // Check for useEffect, useMemo, useCallback
            if ident.name == "useEffect" || ident.name == "useMemo" || ident.name == "useCallback" {
                // Extract second argument (dependency array)
                if let Some(Expression::ArrayExpression(ref arr)) = node.arguments.get(1) {
                    for elem in &arr.elements {
                        if let Expression::Identifier(ref id) = elem {
                            self.state.dependencies.push(id.name.clone());
                        }
                    }
                }
            }
        }
    }

    fn exit(program: &mut Program, ctx: &Context) {
        println!("Found dependencies: {:?}", self.state.dependencies);
    }
}
```

More examples in [source/examples/](source/examples/).

## Project Structure

```
reluxscript/
├── source/                      # ReluxScript compiler source
│   ├── src/
│   │   ├── lexer/              # Tokenization
│   │   ├── parser/             # AST parsing
│   │   ├── semantic/           # Type checking & analysis
│   │   ├── codegen/
│   │   │   ├── babel.rs        # Babel JavaScript generation
│   │   │   └── swc.rs          # SWC Rust generation
│   │   └── main.rs             # CLI entry point
│   ├── examples/               # Example plugins
│   │   └── minimal_tests/      # Codegen test cases
│   └── Cargo.toml
├── docs/                        # Documentation
│   ├── reluxscript-specification.md  # Language spec
│   ├── COMPILER_ARCHITECTURE.md      # Internals guide
│   ├── REGEX_SUPPORT.md              # Regex feature (WIP)
│   └── CUSTOM_AST_PROPERTIES.md      # AST props feature (WIP)
├── minimact/                    # Real-world example
│   ├── babel-plugin-minimact/  # Original Babel plugin
│   └── reluxscript-plugin-minimact/  # ReluxScript port
└── README.md
```

## Documentation

- **[Language Specification](docs/reluxscript-specification.md)** - Complete language reference
- **[Compiler Architecture](docs/COMPILER_ARCHITECTURE.md)** - Internals and development guide
- **[Regex Support](docs/REGEX_SUPPORT.md)** - Pattern matching feature (in development)
- **[Custom AST Properties](docs/CUSTOM_AST_PROPERTIES.md)** - Metadata tracking (in development)

## Real-World Example: Minimact

**Minimact** is a production Babel plugin that transpiles React/TSX to C# for server-side rendering. We're converting it to ReluxScript to demonstrate real-world viability.

**Status:** 123/130 files converted (95%)

See [minimact/](minimact/) for the full conversion.

## Building from Source

### Prerequisites

- Rust 1.70+ (for the compiler)
- Cargo (comes with Rust)

### Build Steps

```bash
# Clone repository
git clone https://github.com/yourusername/reluxscript.git
cd reluxscript/source

# Run tests
cargo test

# Build release
cargo build --release

# The binary is at: target/release/reluxc
```

### Development

```bash
# Run compiler in dev mode
cargo run -- build examples/remove_console.lux --target babel

# Run specific tests
cargo test parser
cargo test codegen

# Check code
cargo clippy
cargo fmt
```

## CLI Usage

```bash
# Compile to Babel
reluxc build plugin.lux --target babel

# Compile to SWC
reluxc build plugin.lux --target swc

# Compile both targets
reluxc build plugin.lux --target all

# Watch mode (recompile on changes)
reluxc watch plugin.lux --target babel

# Type check only (no codegen)
reluxc check plugin.lux

# Show AST
reluxc ast plugin.lux

# Help
reluxc --help
```

## Roadmap

### ✅ Completed

- [x] Lexer and parser
- [x] Semantic analysis (type checking, ownership validation)
- [x] Babel code generation
- [x] SWC code generation
- [x] Multi-file projects (import/export)
- [x] Visitor pattern
- [x] String methods
- [x] Format strings
- [x] Pattern matching
- [x] Verbatim blocks

### 🚧 In Progress

- [ ] **Regex support** (v0.2.0) - `Regex::matches()`, `Regex::find()`, etc.
- [ ] **Custom AST properties** (v0.3.0) - Unified metadata tracking
- [ ] **Minimact conversion** (v0.4.0) - Complete real-world plugin

### 🔮 Future

- [ ] Language server protocol (LSP)
- [ ] VS Code extension
- [ ] Plugin registry
- [ ] Online playground
- [ ] More built-in AST node constructors
- [ ] Macro system
- [ ] Testing framework for plugins

## Contributing

Contributions are welcome! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

### Areas to Contribute

- **Language features**: Implement new syntax or built-in functions
- **Codegen improvements**: Optimize output for Babel or SWC
- **Documentation**: Improve guides, add examples
- **Tooling**: LSP, editor plugins, testing tools
- **Bug fixes**: See [issues](https://github.com/yourusername/reluxscript/issues)

## Philosophy

ReluxScript is guided by these principles:

1. **Vector Intersection, Not Union**: Support only what works identically in both targets
2. **Explicit Over Implicit**: Require explicit clones, mutations, and references
3. **Type Safety First**: Catch errors at compile-time
4. **Unified AST**: One AST representation for both ecosystems
5. **Zero Magic**: No implicit conversions or hidden behavior

Read more in [Language Philosophy](docs/reluxscript-specification.md#11-design-philosophy).

## Comparison

### vs Writing Babel Plugins Directly

| Aspect | Babel Plugin | ReluxScript |
|--------|-------------|-------------|
| Language | JavaScript | Rust-like syntax |
| Type safety | None (JSDoc at best) | Full static typing |
| SWC support | Manual rewrite | Automatic compilation |
| Maintenance | Two codebases | One codebase |
| Performance | Good (V8) | Excellent (native) with SWC |

### vs Writing SWC Plugins Directly

| Aspect | SWC Plugin | ReluxScript |
|--------|------------|-------------|
| Language | Rust | ReluxScript |
| Babel support | Manual rewrite | Automatic compilation |
| Learning curve | Steep (Rust + AST) | Moderate (DSL) |
| Flexibility | Full Rust power | Subset of features |
| Dev speed | Slower | Faster |

### vs Other AST Tools

- **Codemod**: ReluxScript generates reusable plugins, not one-off scripts
- **jscodeshift**: ReluxScript is typed and generates native code
- **ts-morph**: TypeScript only; ReluxScript supports any JS/TS code

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- **Babel** team for the JavaScript AST transformation ecosystem
- **SWC** team for blazing-fast Rust-based tooling
- **Rust** community for ownership semantics inspiration

## Community

- **GitHub Issues**: [Report bugs or request features](https://github.com/yourusername/reluxscript/issues)
- **Discussions**: [Ask questions and share ideas](https://github.com/yourusername/reluxscript/discussions)
- **Discord**: [Join our community](https://discord.gg/reluxscript) (coming soon)

---

<p align="center">
  Made with ❤️ by the ReluxScript team<br>
  <a href="#top">Back to top</a>
</p>
