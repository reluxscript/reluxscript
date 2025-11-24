# ReluxScript

A language that compiles to both Babel (JavaScript) and SWC (Rust) plugins from a single source.

## Overview

ReluxScript is a domain-specific language for writing AST transformation plugins. Write your plugin once in ReluxScript, and compile it to:

- **Babel plugin** (JavaScript) - for Node.js/npm ecosystem
- **SWC plugin** (Rust) - for high-performance native builds

## Installation

```bash
cd reluxscript
cargo build --release
```

## Usage

### Check a file for errors

```bash
cargo run -- check examples/my_plugin.rsc
```

### Build a plugin

```bash
# Build for both targets (default)
cargo run -- build examples/my_plugin.rsc -o dist

# Build for Babel only
cargo run -- build examples/my_plugin.rsc -o dist -t babel

# Build for SWC only
cargo run -- build examples/my_plugin.rsc -o dist -t swc
```

### Debug commands

```bash
# Tokenize (view lexer output)
cargo run -- lex examples/my_plugin.rsc

# Parse (view AST)
cargo run -- parse examples/my_plugin.rsc
```

## Language Syntax

ReluxScript uses a Rust-like syntax optimized for AST transformations.

### Basic Plugin Structure

```reluxscript
/// My transformation plugin
plugin MyPlugin {
    /// Visit function declarations
    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        let name = node.id.name.clone();

        if name.starts_with("use") {
            // Transform hooks...
        }
    }
}
```

### Structs

```reluxscript
struct Component {
    name: Str,
    props: Vec<Prop>,
    hooks: Vec<Str>,
}

struct Prop {
    name: Str,
    prop_type: Str,
}
```

### Pattern Matching with `matches!`

```reluxscript
pub fn visit_call_expression(node: &CallExpression) {
    if matches!(node.callee, Identifier) {
        let name = node.callee.name.clone();
        // ...
    }
}
```

### Traverse Blocks

Traverse into child nodes with nested visitors:

```reluxscript
pub fn visit_function_declaration(node: &FunctionDeclaration) {
    let mut hooks: Vec<Str> = vec![];

    traverse(node.body) {
        fn visit_call_expression(call: &CallExpression) {
            if matches!(call.callee, Identifier) {
                if call.callee.name.starts_with("use") {
                    hooks.push(call.callee.name.clone());
                }
            }
        }
    }
}
```

### Collections

```reluxscript
// Vec
let mut items: Vec<Str> = vec![];
items.push("hello");
let joined = items.join(", ");

// HashMap
let mut map: HashMap<Str, Str> = HashMap::new();
map.insert("key", "value");
if map.contains_key(&"key") {
    let value = map.get(&"key");
}

// HashSet
let mut set: HashSet<Str> = HashSet::new();
set.insert("item");
if set.contains(&"item") {
    // ...
}
```

### String Formatting

```reluxscript
let name = "World";
let greeting = format!("Hello, {}!", name);

// Multi-line
let code = format!(
    "public class {} {{\n    // body\n}}",
    class_name
);

// Concatenation
let path = base + "/" + &filename;

// String builder
let mut s = String::new();
s.push_str("line 1\n");
s.push_str("line 2\n");
```

### File I/O

```reluxscript
use fs;

fn save_output(content: &Str, path: &Str) {
    fs::create_dir_all(&"output").unwrap();
    fs::write(path, content).unwrap();
}

fn load_config(path: &Str) -> Str {
    return fs::read_to_string(path).unwrap();
}
```

### JSON Serialization

```reluxscript
use json;

#[derive(Serialize)]
struct Config {
    name: Str,
    version: Str,
}

fn save_config(config: &Config) {
    let json_str = json::to_string_pretty(config).unwrap();
    fs::write(&"config.json", &json_str).unwrap();
}
```

## AST Node Types

ReluxScript includes mappings for all Babel/SWC AST nodes. Common ones:

### Expressions
- `Identifier`
- `CallExpression`
- `MemberExpression`
- `ArrayExpression`
- `ObjectExpression`
- `ArrowFunctionExpression`
- `JSXElement`

### Statements
- `FunctionDeclaration`
- `VariableDeclaration`
- `VariableDeclarator`
- `BlockStatement`
- `ReturnStatement`
- `IfStatement`

### TypeScript
- `TSInterfaceDeclaration`
- `TSTypeAnnotation`
- `TSPropertySignature`

### Patterns
- `ArrayPattern`
- `ObjectPattern`

## Generated Output

### Babel Output

```javascript
module.exports = function({ types: t }) {
  return {
    visitor: {
      FunctionDeclaration(path) {
        const node = path.node;
        // Your transformed code...
      }
    }
  };
};
```

### SWC Output

```rust
use swc_core::ecma::{
    ast::*,
    visit::{VisitMut, VisitMutWith},
};

pub struct MyPlugin;

impl VisitMut for MyPlugin {
    fn visit_mut_fn_decl(&mut self, node: &mut FnDecl) {
        // Your transformed code...
        node.visit_mut_children_with(self);
    }
}
```

## Project Structure

```
reluxscript/
├── src/
│   ├── lib.rs           # Library exports
│   ├── main.rs          # CLI entry point
│   ├── lexer/           # Tokenizer
│   ├── parser/          # Parser and AST definitions
│   ├── semantic/        # Type checking, resolution, lowering
│   ├── codegen/         # Babel and SWC code generators
│   └── mapping/         # AST node/field mappings
├── examples/            # Example ReluxScript plugins
├── tests/
│   └── enhancements/    # Feature test cases
└── Cargo.toml
```

## Examples

See the `examples/` directory for sample plugins:

- `build_member_path.rsc` - Helper function for building member expressions
- `tsx_parser_test/` - Extract interfaces and hooks from TSX

## Documentation

- [Language Specification](../docs/reluxscript-language-spec.md)
- [Enhancement Plan](../docs/reluxscript-plugin-enhancements.md)

## License

MIT
