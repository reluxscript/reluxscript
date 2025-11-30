---
layout: home

hero:
  name: ReluxScript
  text: Light, Light, Write!
  tagline: >
    Write AST transformations once. Compile to Babel, SWC, and beyond.
    One .lux file. Infinite possibilities.
  image:
    src: /logo.png
    alt: ReluxScript
  actions:
    - theme: brand
      text: Get Started
      link: /v0.1/guide/getting-started
    - theme: alt
      text: View on GitHub
      link: https://github.com/reluxscript/reluxscript

features:
  - icon: 🔺
    title: Write Once
    details: Stop maintaining duplicate Babel and SWC plugins. Write your AST transformation logic once in clean, Rust-inspired syntax.

  - icon: ⚡
    title: Dual Compilation
    details: Compile to both Babel (JavaScript) and SWC (Rust/WASM) from a single source. Perfect for modern toolchains.

  - icon: ✨
    title: Type Safety
    details: Catch errors at compile time with static type checking. Your transformations are validated before they run.

  - icon: 🎯
    title: Vector Alignment
    details: Uses the intersection of Babel and SWC features, not the union. What compiles will work correctly on both platforms.

  - icon: 🔧
    title: Familiar Syntax
    details: Rust-inspired syntax that feels natural to systems programmers while being accessible to JavaScript developers.

  - icon: 🚀
    title: Extensible
    details: Beyond Babel and SWC—compile to custom transpilers like TSX→C# for innovative architectures.
---

## Quick Start

```bash
# Install ReluxScript
cargo install reluxscript

# Create your first plugin
relux new my-plugin

# Build to Babel
relux build my-plugin.lux --target babel

# Build to SWC
relux build my-plugin.lux --target swc
```

## Example

Write once in ReluxScript:

```reluxscript
/// Remove console.log statements
plugin RemoveConsole {
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        if let Callee::MemberExpression(ref member) = node.callee {
            if let Expression::Identifier(ref obj) = *member.object {
                if obj.name == "console" {
                    if let Expression::Identifier(ref prop) = *member.property {
                        if prop.name == "log" {
                            ctx.remove();
                        }
                    }
                }
            }
        }
    }
}
```

Compiles to both:

**Babel (JavaScript)**
```javascript
module.exports = function({ types: t }) {
  return {
    visitor: {
      CallExpression(path) {
        const node = path.node;
        const __iflet_0 = node.callee;
        if (__iflet_0 !== null) {
          const member = __iflet_0;
          const __iflet_1 = member.object;
          if (__iflet_1 !== null) {
            const obj = __iflet_1;
            if (obj.name === "console") {
              const __iflet_2 = member.property;
              if (__iflet_2 !== null) {
                const prop = __iflet_2;
                if (prop.name === "log") {
                  path.remove();
                }
              }
            }
          }
        }
      }
    }
  };
};
```

**SWC (Rust)**
```rust
pub struct RemoveConsole {}

impl VisitMut for RemoveConsole {
    fn visit_mut_call_expr(&mut self, node: &mut CallExpr) {
        if let Callee::Expr(__callee_expr) = &node.callee {
            if let Expr::Member(member) = __callee_expr.as_ref() {
                if let Expr::Ident(obj) = &*member.obj.as_ref() {
                    if (&*obj.sym.to_string() == "console") {
                        if let MemberProp::Ident(prop) = &member.prop {
                            if (&*prop.sym.to_string() == "log") {
                                node.callee = Callee::Expr(Box::new(
                                    Expr::Ident(Ident::new(
                                        "undefined".into(),
                                        DUMMY_SP,
                                        SyntaxContext::empty()
                                    ))
                                ))
                            }
                        }
                    }
                }
            }
        }
    }
}
```

---

## Why ReluxScript?

### The Problem

Building AST transformation plugins is hard:
- **Babel plugins** are JavaScript-based, flexible, but slow
- **SWC plugins** are Rust-based, fast, but complex
- **Maintaining both** means duplicate code, double the bugs, twice the work

### The Solution

ReluxScript is a domain-specific language that compiles to both Babel and SWC:
- **Write once** in clean, type-safe syntax
- **Compile to both** Babel (JS) and SWC (Rust)
- **Vector intersection** principle ensures compatibility
- **Type safety** catches errors at compile time
- **Extensible** to custom transpiler targets

### Who It's For

**For Plugin Authors:** Stop maintaining duplicate codebases. Write once, target both ecosystems.

**For Tool Builders:** Use ReluxScript as your plugin format. Let users write once, deploy everywhere.

**For Framework Teams:** Build custom transpilers from TypeScript/JSX to your target language (C#, Go, etc).

---

## Philosophy

ReluxScript follows the **Vector Intersection** principle:

> We only support features that work identically in both Babel and SWC.

This means:
- ✅ What compiles will work on both platforms
- ✅ No surprises, no "works in Babel but not SWC"
- ✅ Predictable, reliable transformations

We choose **correctness over coverage**. Better to support 80% of use cases perfectly than 100% with edge case bugs.

---

## Pronunciation

**/ˈreɪ.lʌks.skrɪpt/** • ray-lucks-script

**ReluxScript** = **Re**ay + **Lux** + **Script**

- **Ray** (sunshine) - Illuminating the path forward
- **Lux** (light) - Bringing clarity to AST transformations
- **Script** (code) - The language itself

> *"Light, light, write!"* ☀️
