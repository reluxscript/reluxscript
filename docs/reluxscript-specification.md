# ReluxScript Language Specification

**Version:** 0.9.0
**Status:** Draft (Program Hooks & Verbatim Blocks Added)
**Target Platforms:** Babel (JavaScript) & SWC (Rust/WASM)

---

## 1. Overview

ReluxScript is a domain-specific language for writing AST transformation plugins that compile to both Babel (JavaScript) and SWC (Rust). It enforces a strict visitor pattern with explicit ownership semantics that map cleanly to both garbage-collected and borrow-checked runtimes.

### 1.1 Design Philosophy

- **Strict Visitor, Loose Context**: Enforces Rust-like `VisitMut` pattern
- **Immutable by Default**: All mutations must be explicit
- **No Path Magic**: No implicit parent traversal; state must be tracked explicitly
- **Unified AST**: Subset of nodes common to ESTree (Babel) and swc_ecma_ast
- **Clone-to-Own**: Explicit `.clone()` required for value extraction

### 1.2 Vector Alignment Principle

ReluxScript finds the intersection of JavaScript and Rust capabilities, not the union. Code that compiles must be semantically valid in both targets.

---

## 2. Lexical Structure

### 2.1 Keywords

```
plugin      fn          let         const       if          else
match       return      true        false       null        for
in          while       break       continue    struct      enum
impl        use         pub         mut         self        Self
traverse    using       writer      where
```

### 2.2 Reserved Keywords (Future Use)

```
async       await       trait       type        as          loop
mod         crate       super       dyn         static
```

### 2.3 Operators

```
// Arithmetic
+   -   *   /   %

// Comparison
==  !=  <   >   <=  >=

// Logical
&&  ||  !

// Assignment
=   +=  -=  *=  /=

// Reference/Dereference
&   *

// Member Access
.   ::  ?.

// Special
=>  ->  ..  ...  ?

// Delimiters
|   (for closures)
```

### 2.4 Comments

```reluxscript
// Single-line comment

/*
   Multi-line comment
*/

/// Documentation comment (preserved in output)
```

### 2.5 Literals

```reluxscript
// Strings (always use Str type internally)
"hello world"
"escaped \"quotes\""

// Numbers
42
3.14
0xFF
0b1010

// Booleans
true
false

// Null
null

// Unit (empty value)
()
```

---

## 3. Type System

### 3.1 Primitive Types

| ReluxScript | Babel (JS) | SWC (Rust) |
|------------|------------|------------|
| `Str` | `string` | `JsWord` / `Atom` |
| `i32` | `number` | `i32` |
| `f64` | `number` | `f64` |
| `bool` | `boolean` | `bool` |
| `()` | `undefined` | `()` |

### 3.2 Reference Types

```reluxscript
&T          // Immutable reference
&mut T      // Mutable reference
```

**Compilation:**
- **Babel**: References are ignored; values passed directly
- **SWC**: References preserved as-is

### 3.3 Container Types

```reluxscript
Vec<T>      // Dynamic array
Option<T>   // Optional value (Some/None)
HashMap<K, V>  // Key-value map
HashSet<T>  // Unique set
```

**Compilation:**

| ReluxScript | Babel (JS) | SWC (Rust) |
|------------|------------|------------|
| `Vec<T>` | `Array` | `Vec<T>` |
| `Option<T>` | `T \| null` | `Option<T>` |
| `HashMap<K,V>` | `Map` or `Object` | `HashMap<K,V>` |
| `HashSet<T>` | `Set` | `HashSet<T>` |

### 3.4 Tuple Types

```reluxscript
(T1, T2)       // Two-element tuple
(T1, T2, T3)   // Three-element tuple
()             // Unit type (zero-element tuple)
```

**Usage:**
```reluxscript
// Tuple types in function signatures
fn get_coords() -> (i32, i32) {
    (10, 20)
}

// Tuple destructuring
let (x, y) = get_coords();

// As Result type parameter
fn validate() -> Result<(), Str> {
    Ok(())
}
```

**Compilation:**
- **Babel**: Tuples become arrays `[T1, T2]`; unit `()` becomes `undefined`
- **SWC**: Tuples preserved as `(T1, T2)`; unit as `()`

### 3.5 The CodeBuilder Type

For generating code (transpiler use case):

```reluxscript
// A buffer that handles platform-specific string concatenation
type CodeBuilder;

impl CodeBuilder {
    fn new() -> CodeBuilder;
    fn append(s: Str);
    fn newline();
    fn indent();
    fn dedent();
    fn to_string() -> Str;
}
```

**Compilation:**
- **Babel**: Uses array-based string building with `.join()`
- **SWC**: Uses `String` with `push_str()`

### 3.5 AST Node Types

See Section 7 (Unified AST) for complete mapping.

```reluxscript
Identifier
CallExpression
MemberExpression
BinaryExpression
// ... etc
```

---

## 4. Declarations

### 4.1 Plugin Declaration

Every ReluxScript file must declare a plugin:

```reluxscript
plugin MyTransformer {
    // visitor methods and helpers
}
```

**Compiles to:**

```javascript
// Babel
module.exports = function({ types: t }) {
  return {
    visitor: { /* ... */ }
  };
};
```

```rust
// SWC
pub struct MyTransformer;
impl VisitMut for MyTransformer { /* ... */ }
```

### 4.2 Function Declaration

```reluxscript
fn function_name(param: Type, param2: &mut Type) -> ReturnType {
    // body
}
```

**Visibility:**

```reluxscript
pub fn public_function() { }    // Exported
fn private_function() { }       // Internal
```

#### 4.2.1 Generic Functions

Generic functions allow type parameters and trait bounds using Rust syntax:

```reluxscript
// Simple generic function
fn identity<T>(value: T) -> T {
    value
}

// Generic with trait bound (where clause)
pub fn map_expression<F>(expr: &Expression, mapper: F) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    mapper(expr, false)
}

// Multiple type parameters and constraints
fn transform<F, G>(input: Str, f: F, g: G) -> Str
where
    F: Fn(Str) -> Str,
    G: Fn(Str) -> Str
{
    g(f(input))
}
```

**Compilation Model:**

Generic functions use **syntactic passthrough** - the parser recognizes the syntax, but generics are treated as metadata for code generation rather than enforced by a constraint solver.

```javascript
// Babel: Generics are stripped entirely
export function mapExpression(expr, mapper) {
    return mapper(expr, false);
}
```

```rust
// SWC: Generics preserved exactly
pub fn map_expression<F>(expr: &Expr, mapper: F) -> String
where
    F: Fn(&Expr, bool) -> String
{
    mapper(expr, false)
}
```

**Supported Syntax:**

- Type parameters: `<F, T, U>`
- Where clauses: `where F: Fn(...) -> R`
- Function trait types: `Fn(T1, T2) -> R`
- Reference parameters in traits: `Fn(&Type, bool) -> Str`

**Limitations:**

- No generic constraint checking (Marathon philosophy: syntax without semantic solver)
- Type parameters are not tracked in the type system
- Generics only work on functions, not on structs or enums

### 4.3 Variable Declaration

```reluxscript
let name = value;           // Immutable binding
let mut name = value;       // Mutable binding
const NAME = value;         // Compile-time constant
```

**Clone Requirement:**

```reluxscript
// Extracting from a reference requires explicit clone
let name = node.name.clone();   // Required
let name = node.name;           // ERROR: implicit borrow
```

**Compilation:**

```javascript
// Babel: .clone() is stripped
const name = node.name;
```

```rust
// SWC: .clone() preserved
let name = node.name.clone();
```

### 4.4 Struct Declaration

```reluxscript
struct ComponentInfo {
    name: Str,
    props: Vec<Prop>,
    has_state: bool,
}
```

**Compiles to:**

```javascript
// Babel: Plain object shape (documentation only)
// Runtime: { name: string, props: Array, has_state: boolean }
```

```rust
// SWC
#[derive(Clone, Debug)]
pub struct ComponentInfo {
    pub name: String,
    pub props: Vec<Prop>,
    pub has_state: bool,
}
```

### 4.5 Enum Declaration

```reluxscript
enum HookType {
    State,
    Effect,
    Ref,
    Custom(Str),
}
```

**Compiles to:**

```javascript
// Babel: Tagged union pattern
// { type: "State" } | { type: "Effect" } | { type: "Custom", value: string }
```

```rust
// SWC
pub enum HookType {
    State,
    Effect,
    Ref,
    Custom(String),
}
```

### 4.6 Module Declaration

For multi-file projects, create standalone modules without `plugin` or `writer` declarations:

```reluxscript
// File: utils/helpers.lux

// Public function (exported)
pub fn get_component_name(node: &FunctionDeclaration) -> Str {
    node.id.name.clone()
}

// Private function (not exported)
fn internal_helper() -> Str {
    "internal"
}

// Public struct (exported)
pub struct ComponentInfo {
    pub name: Str,
    pub props: Vec<Str>,
}
```

**Compiles to:**

```javascript
// Babel (CommonJS)
function getComponentName(node) {
    return node.id.name;
}

function internalHelper() {
    return "internal";
}

class ComponentInfo {
    constructor(name, props) {
        this.name = name;
        this.props = props;
    }
}

module.exports = {
    getComponentName,
    ComponentInfo,
};
```

```rust
// SWC (Rust module)
//! Generated by ReluxScript compiler
//! Do not edit manually

#[derive(Debug, Clone)]
pub struct ComponentInfo {
    pub name: String,
    pub props: Vec<String>,
}

pub fn get_component_name(node: &FnDecl) -> String {
    node.ident.sym.to_string()
}

fn internal_helper() -> String {
    "internal".to_string()
}
```

### 4.7 Use Statements (Imports)

Import functionality from other modules:

```reluxscript
// Import from file module
use "./utils/helpers.lux";

// Import specific items
use "./utils/helpers.lux" { get_component_name, ComponentInfo };

// Import with alias
use "./utils/helpers.lux" as helpers;

// Import built-in modules
use fs;
use json;
use path;
```

**Module Resolution:**
- **Relative paths** (starting with `./` or `../`): File-based modules
- **Built-in names** (no path): Standard library modules (`fs`, `json`, `path`)
- **Extensions**: `.lux` extension is optional

**Compiles to:**

```javascript
// Babel
const helpers = require('./utils/helpers.js');
const { getComponentName, ComponentInfo } = require('./utils/helpers.js');
const fs = require('fs');
const path = require('path');
```

```rust
// SWC
mod utils {
    pub mod helpers;
}

use utils::helpers;
use utils::helpers::{get_component_name, ComponentInfo};
use std::fs;
use std::path;
```

### 4.8 Program Hooks

ReluxScript supports plugin lifecycle hooks that run before and after the visitor traversal:

```reluxscript
plugin MinimactPlugin {
    /// Pre-hook: runs before any visitors
    fn pre(file: &File) {
        // Save original code before React transforms JSX
        file.metadata.originalCode = file.code;
    }

    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        // Regular visitor method
    }

    /// Exit hook: runs after all visitors complete
    fn exit(program: &mut Program, state: &PluginState) {
        // Post-processing after all transformations
        generate_metadata_file(state);
    }
}
```

**Compiles to:**

```javascript
// Babel
module.exports = function({ types: t }) {
    // Pre-hook function
    function pre(file) {
        file.metadata.originalCode = file.code;
    }

    // Exit hook function
    function exit(program, state) {
        generate_metadata_file(state);
    }

    return {
        pre(file) {
            pre(file);
        },

        visitor: {
            Program: {
                exit(path, state) {
                    exit(path.node, state);
                }
            },

            JSXElement(path) {
                // ...
            }
        }
    };
};
```

```rust
// SWC: Hooks are not supported in SWC visitor pattern
// exit() becomes a regular method, pre() is omitted
impl VisitMut for MinimactPlugin {
    fn visit_mut_jsx_element(&mut self, n: &mut JSXElement) {
        // ...
    }
}
```

**Use Cases:**

- **pre()**: Save original source code for format-preserving transformations (Recast)
- **exit()**: Generate output files, collect metadata, perform final processing

**Limitations:**

- `pre()` only available in Babel (no SWC equivalent)
- `exit()` becomes a regular method in SWC, not automatically called
- Hooks receive different parameters than visitor methods

### 4.9 Verbatim Code Blocks

For platform-specific operations (like Recast in Babel or custom Rust code in SWC), ReluxScript provides verbatim blocks that emit raw code:

```reluxscript
plugin RecastPlugin {
    fn pre(file: &File) {
        babel! {
            // Raw JavaScript - preserved exactly as written
            file.metadata.originalCode = file.code;
        }
    }

    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        // Mix ReluxScript with verbatim blocks
        let tag_name = &node.opening_element.name;

        babel! {
            // Babel-specific: Use Recast for format-preserving edits
            const recast = require('recast');
            const keyAttr = t.jsxAttribute(
                t.jsxIdentifier('key'),
                t.stringLiteral('generated-key')
            );
            node.openingElement.attributes.push(keyAttr);
        }

        swc! {
            // SWC-specific: Rust AST manipulation
            node.opening.attrs.push(JSXAttr {
                span: DUMMY_SP,
                name: JSXAttrName::Ident(Ident::new("key".into(), DUMMY_SP)),
                value: Some(JSXAttrValue::Lit(Lit::Str("generated-key".into())))
            });
        }
    }

    fn exit(program: &mut Program, state: &PluginState) {
        babel! {
            // Use Recast to generate .keys file with preserved formatting
            const recast = require('recast');
            const originalAst = recast.parse(state.file.metadata.originalCode, {
                parser: require('recast/parsers/babel-ts')
            });

            // Traverse and modify...

            const output = recast.print(originalAst, {
                tabWidth: 2,
                quote: 'single'
            });

            fs.writeFileSync(keysFilePath, output.code);
        }
    }
}
```

**Syntax:**

- `babel! { ... }` or `js! { ... }` - JavaScript code (Babel only)
- `swc! { ... }` or `rust! { ... }` - Rust code (SWC only)

**Compiles to:**

```javascript
// Babel: babel!/js! blocks are emitted, swc!/rust! blocks are omitted
function visit_jsx_element(node, ctx) {
    const tag_name = node.opening_element.name;

    // Verbatim JavaScript
    const recast = require('recast');
    const keyAttr = t.jsxAttribute(
        t.jsxIdentifier('key'),
        t.stringLiteral('generated-key')
    );
    node.openingElement.attributes.push(keyAttr);

    /* SWC-only code omitted */
}
```

```rust
// SWC: swc!/rust! blocks are emitted, babel!/js! blocks are omitted
fn visit_mut_jsx_element(&mut self, n: &mut JSXElement) {
    let tag_name = &n.opening_element.name;

    // Babel-only code omitted

    // Verbatim Rust
    node.opening.attrs.push(JSXAttr {
        span: DUMMY_SP,
        name: JSXAttrName::Ident(Ident::new("key".into(), DUMMY_SP)),
        value: Some(JSXAttrValue::Lit(Lit::Str("generated-key".into())))
    });
}
```

**Characteristics:**

- **Format-preserving**: Whitespace and formatting inside verbatim blocks is preserved exactly
- **No type checking**: Contents are opaque to ReluxScript's semantic analysis
- **Platform-specific**: Each block targets one platform, other platform sees comment
- **Token-based extraction**: Uses token spans to extract raw source between braces

**Use Cases:**

- Recast integration for format-preserving Babel transformations
- Platform-specific npm package imports (`require('recast')`)
- Direct manipulation of Babel or SWC AST types not in the unified AST
- File I/O operations (`fs.writeFileSync`)
- Complex JavaScript patterns (object literal configs, etc.)

**Design Philosophy:**

Verbatim blocks embody the "surgical integration" principle: ReluxScript handles cross-platform AST transformations, verbatim blocks handle platform-specific edge cases. This provides maximum flexibility with minimum abstraction cost.

---

## 5. Module System

### 6.1 Multi-File Projects

ReluxScript supports organizing code across multiple files:

```
my-plugin/
  main.lux              # Entry point (plugin declaration)
  utils/
    helpers.lux         # Helper functions
    types.lux           # Type definitions
  extractors/
    props.lux           # Prop extraction
    hooks.lux           # Hook extraction
```

### 6.2 Module Types

**Plugin/Writer Modules (Entry Points):**
- Contain `plugin` or `writer` declaration
- Main file that orchestrates the transformation
- Can import from other modules

**Library Modules:**
- Pure function/struct/enum definitions
- Use `pub` to export items
- No `plugin` or `writer` declaration

### 6.3 Export Rules

Only items marked with `pub` are exported:

```reluxscript
// Exported (visible to importers)
pub fn public_function() { }
pub struct PublicStruct { }
pub enum PublicEnum { }

// Not exported (module-private)
fn private_function() { }
struct PrivateStruct { }
enum PrivateEnum { }
```

### 6.4 Import Patterns

```reluxscript
// Import all exports (use with module prefix)
use "./helpers.lux" as h;
let name = h::get_name(node);

// Import specific items (use directly)
use "./helpers.lux" { get_name };
let name = get_name(node);

// Import multiple items
use "./helpers.lux" { get_name, escape_string, ComponentInfo };
```

### 6.5 Built-in Modules

ReluxScript provides standard library modules:

- **fs** - File system operations (read, write, exists)
- **json** - JSON serialization and deserialization
- **path** - Path manipulation utilities
- **parser** - Runtime AST parsing (Code → AST)
- **codegen** - AST to code conversion (AST → Code)

#### fs Module (File System)

```reluxscript
use fs;

// Read file
let content = fs::read_file("input.txt")?;

// Write file
fs::write_file("output.txt", content)?;

// Check if file exists
if fs::exists("config.json") {
    // ...
}

// Read directory
let files = fs::read_dir("./src")?;
```

**Babel Compilation:**
```javascript
const fs = require('fs');

const content = fs.readFileSync("input.txt", "utf-8");
fs.writeFileSync("output.txt", content);
const exists = fs.existsSync("config.json");
const files = fs.readdirSync("./src");
```

**SWC Compilation:**
```rust
use std::fs;

let content = fs::read_to_string("input.txt")?;
fs::write("output.txt", content)?;
let exists = std::path::Path::new("config.json").exists();
let files = fs::read_dir("./src")?;
```

#### json Module (JSON Serialization)

```reluxscript
use json;

// Serialize to JSON
let json_str = json::stringify(data);

// Parse from JSON
let data = json::parse(json_str)?;
```

**Babel Compilation:**
```javascript
const jsonStr = JSON.stringify(data, null, 2);
const data = JSON.parse(jsonStr);
```

**SWC Compilation:**
```rust
let json_str = serde_json::to_string_pretty(&data)?;
let data = serde_json::from_str(&json_str)?;
```

#### path Module (Path Manipulation)

```reluxscript
use path;

// Join paths
let full_path = path::join(vec!["src", "utils", "helpers.lux"]);

// Get directory name
let dir = path::dirname("/src/utils/helpers.lux");

// Get filename
let name = path::basename("/src/utils/helpers.lux");

// Get extension
let ext = path::extname("/src/utils/helpers.lux");
```

**Babel Compilation:**
```javascript
const path = require('path');

const fullPath = path.join("src", "utils", "helpers.lux");
const dir = path.dirname("/src/utils/helpers.lux");
const name = path.basename("/src/utils/helpers.lux");
const ext = path.extname("/src/utils/helpers.lux");
```

**SWC Compilation:**
```rust
use std::path::{Path, PathBuf};

let full_path = PathBuf::from("src").join("utils").join("helpers.lux");
let dir = Path::new("/src/utils/helpers.lux").parent();
let name = Path::new("/src/utils/helpers.lux").file_name();
let ext = Path::new("/src/utils/helpers.lux").extension();
```

#### parser Module (Runtime AST Parsing)

The `parser` module provides dynamic parsing capabilities for analyzing imported files at runtime. This enables cross-file analysis, such as inspecting custom hooks or components from external files.

```reluxscript
use parser;

// Parse a TypeScript/JavaScript file
let ast = parser::parse_file("./useCounter.tsx")?;

// Parse code from a string
let code = "function foo() { return 42; }";
let ast = parser::parse(code)?;

// Parse with specific syntax
let ast = parser::parse_with_syntax(code, "TypeScript")?;

// Analyze the parsed AST
for stmt in &ast.body {
    if let Statement::FunctionDeclaration(ref func) = stmt {
        // Process function...
    }
}
```

**Babel Compilation:**
```javascript
const babel = require('@babel/core');
const fs = require('fs');
const parser = {
  parse_file: (path) => {
    try {
      const code = fs.readFileSync(path, 'utf-8');
      const ast = babel.parseSync(code, {
        filename: path,
        presets: ['@babel/preset-typescript'],
        plugins: ['@babel/plugin-syntax-jsx'],
      });
      return { ok: true, value: ast };
    } catch (error) {
      return { ok: false, error: error.message };
    }
  },
  parse: (code) => {
    try {
      const ast = babel.parseSync(code, {
        presets: ['@babel/preset-typescript'],
        plugins: ['@babel/plugin-syntax-jsx'],
      });
      return { ok: true, value: ast };
    } catch (error) {
      return { ok: false, error: error.message };
    }
  },
  parse_with_syntax: (code, syntax) => {
    try {
      let options = {};
      if (syntax === 'TypeScript') {
        options = {
          presets: ['@babel/preset-typescript'],
          plugins: ['@babel/plugin-syntax-jsx'],
        };
      } else if (syntax === 'JSX') {
        options = {
          plugins: ['@babel/plugin-syntax-jsx'],
        };
      }
      const ast = babel.parseSync(code, options);
      return { ok: true, value: ast };
    } catch (error) {
      return { ok: false, error: error.message };
    }
  },
};
```

**SWC Compilation:**
```rust
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig, EsConfig};
use std::sync::Arc;

mod parser {
    use super::*;

    pub fn parse_file(path: &str) -> Result<Program, String> {
        let source_map = Arc::new(SourceMap::default());
        let code = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read file: {}", e))?;
        let file = source_map.new_source_file(
            FileName::Real(path.into()),
            code,
        );
        let syntax = Syntax::Typescript(TsConfig {
            tsx: true,
            decorators: false,
            ..Default::default()
        });
        let mut parser = Parser::new(syntax, StringInput::from(&*file), None);
        parser.parse_program()
            .map_err(|e| format!("Parse error: {:?}", e))
    }

    pub fn parse(code: &str) -> Result<Program, String> {
        let source_map = Arc::new(SourceMap::default());
        let file = source_map.new_source_file(
            FileName::Anon,
            code.to_string(),
        );
        let syntax = Syntax::Typescript(TsConfig {
            tsx: true,
            decorators: false,
            ..Default::default()
        });
        let mut parser = Parser::new(syntax, StringInput::from(&*file), None);
        parser.parse_program()
            .map_err(|e| format!("Parse error: {:?}", e))
    }

    pub fn parse_with_syntax(code: &str, syntax_type: &str) -> Result<Program, String> {
        let source_map = Arc::new(SourceMap::default());
        let file = source_map.new_source_file(
            FileName::Anon,
            code.to_string(),
        );
        let syntax = match syntax_type {
            "TypeScript" => Syntax::Typescript(TsConfig {
                tsx: true,
                decorators: false,
                ..Default::default()
            }),
            "JSX" => Syntax::Es(EsConfig {
                jsx: true,
                ..Default::default()
            }),
            _ => Syntax::Es(EsConfig::default()),
        };
        let mut parser = Parser::new(syntax, StringInput::from(&*file), None);
        parser.parse_program()
            .map_err(|e| format!("Parse error: {:?}", e))
    }
}
```

**Notes:**
- All parser functions return `Result<Program, String>` for error handling
- Use the `?` operator for automatic error propagation
- The parser module is designed for read-only AST analysis (no modifications)
- See [reluxscript-parser-module.md](./reluxscript-parser-module.md) for detailed documentation and usage examples

---

#### codegen Module (AST to Code Conversion)

The `codegen` module provides the inverse of parsing: converting AST nodes back to source code strings. This is useful for code generation, templating, debugging, and extracting code snippets.

```reluxscript
use codegen;

// Convert any AST node to source code
let code = codegen::generate(expr);

// With formatting options
let options = CodegenOptions {
    minified: true,
    quotes: QuoteStyle::Single,
};
let code = codegen::generate_with_options(expr, options);
```

**Babel Compilation:**
```javascript
const generate = require('@babel/generator').default;

// Simple generation
const code = generate(expr).code;

// With options
const code = generate(expr, {
    minified: true,
    quotes: "single"
}).code;
```

**SWC Compilation:**
```rust
use swc_ecma_codegen::{text_writer::JsWriter, Emitter, Config as CodegenConfig};
use swc_common::SourceMap;
use std::sync::Arc;

// Helper functions are generated automatically
fn codegen_to_string<N: swc_ecma_visit::Node>(node: &N) -> String {
    let mut buf = vec![];
    {
        let cm = Arc::new(SourceMap::default());
        let mut emitter = Emitter {
            cfg: CodegenConfig::default(),
            cm: cm.clone(),
            comments: None,
            wr: Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None)),
        };
        node.emit_with(&mut emitter).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

fn codegen_to_string_with_config<N: swc_ecma_visit::Node>(node: &N, cfg: CodegenConfig) -> String {
    let mut buf = vec![];
    {
        let cm = Arc::new(SourceMap::default());
        let mut emitter = Emitter {
            cfg,
            cm: cm.clone(),
            comments: None,
            wr: Box::new(JsWriter::new(cm.clone(), "\n", &mut buf, None)),
        };
        node.emit_with(&mut emitter).unwrap();
    }
    String::from_utf8(buf).unwrap()
}

// Usage
let code = codegen_to_string(expr);
```

**CodegenOptions Struct:**
```reluxscript
struct CodegenOptions {
    minified: bool,      // Minify output (remove unnecessary whitespace)
    compact: bool,       // Compact output (some whitespace)
    quotes: QuoteStyle,  // Single or Double quotes
    semicolons: bool,    // Include semicolons
}

enum QuoteStyle {
    Single,
    Double,
}
```

**Use Cases:**

1. **Code Extraction** - Save generated code snippets:
```reluxscript
fn extract_function_code(func: &FunctionDeclaration) -> Str {
    codegen::generate(func)
}
```

2. **Template Generation** - Build code dynamically:
```reluxscript
fn generate_wrapper(inner_expr: &Expr) -> Str {
    let wrapper = build_wrapper_node(inner_expr);
    codegen::generate(wrapper)
}
```

3. **Debug Logging** - Readable AST output:
```reluxscript
fn debug_expr(expr: &Expr) {
    let code = codegen::generate(expr);
    println!("Expression: {}", code);
}
```

4. **Code Comparison** - Normalize and compare:
```reluxscript
fn expressions_equal(a: &Expr, b: &Expr) -> bool {
    let code_a = codegen::generate(a);
    let code_b = codegen::generate(b);
    code_a == code_b
}
```

**Notes:**
- The codegen module generates syntactically valid JavaScript/TypeScript code
- Formatting may differ between Babel and SWC targets
- Comments and source maps are not preserved in the generated code
- This is a one-way operation (Code → AST is parsing, AST → Code is codegen)
- See [reluxscript-codegen-module.md](./reluxscript-codegen-module.md) for detailed documentation and usage examples

---

### 6.6 Example: Multi-File Plugin

**main.lux** (Entry point):
```reluxscript
use "./utils/helpers.lux" { get_component_name };
use "./extractors/props.lux" { extract_props };
use fs;
use json;

plugin ReactAnalyzer {
    struct State {
        components: Vec<ComponentInfo>,
    }

    struct ComponentInfo {
        name: Str,
        props: Vec<PropInfo>,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        let name = get_component_name(node);
        let props = extract_props(node);

        self.state.components.push(ComponentInfo {
            name,
            props,
        });
    }

    fn visit_program_exit(node: &Program, ctx: &Context) {
        for component in &self.state.components {
            let json_data = json::stringify(component);
            let filename = format!("{}.meta.json", component.name);
            fs::write_file(filename, json_data)?;
        }
    }
}
```

**utils/helpers.lux**:
```reluxscript
pub fn get_component_name(node: &FunctionDeclaration) -> Str {
    node.id.name.clone()
}

pub fn is_component(name: &Str) -> bool {
    let first = name.chars().next();
    if let Some(c) = first {
        c.is_uppercase()
    } else {
        false
    }
}
```

**extractors/props.lux**:
```reluxscript
pub struct PropInfo {
    pub name: Str,
    pub type_name: Str,
}

pub fn extract_props(node: &FunctionDeclaration) -> Vec<PropInfo> {
    let mut props = vec![];

    if node.params.len() > 0 {
        // Extract from first parameter
        // ... extraction logic ...
    }

    props
}
```

---

## 19. Visitor Methods

### 6.1 Visitor Function Signature

```reluxscript
plugin MyPlugin {
    fn visit_<node_type>(node: &mut NodeType, ctx: &Context) {
        // transformation logic
    }
}
```

**Naming Convention:**
- `visit_identifier` → visits `Identifier` nodes
- `visit_call_expression` → visits `CallExpression` nodes
- Snake_case function name maps to PascalCase node type

### 6.2 Context Object

The `Context` provides limited scope analysis available in both engines:

```reluxscript
ctx.scope              // Current scope information
ctx.filename           // Source filename
ctx.generate_uid(hint) // Generate unique identifier
```

### 6.3 Node Replacement (Statement Lowering)

Direct assignment to the visitor argument triggers replacement:

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    // This is "statement lowering" - not regular assignment
    *node = CallExpression {
        callee: Identifier::new("newName"),
        arguments: vec![],
    };
}
```

**Compiles to:**

```javascript
// Babel: Lowered to path.replaceWith()
path.replaceWith(t.callExpression(
    t.identifier("newName"),
    []
));
```

```rust
// SWC: Direct assignment
*node = CallExpr {
    callee: Callee::Expr(Box::new(Expr::Ident(Ident::new("newName".into(), DUMMY_SP)))),
    args: vec![],
    ..Default::default()
};
```

### 6.4 Node Removal

In Babel, this is `path.remove()`. In SWC, this requires replacing with a NoOp or filtering. ReluxScript standardizes this:

```reluxscript
fn visit_expression_statement(node: &mut ExpressionStatement, ctx: &Context) {
    if should_remove(node) {
        *node = Statement::noop(); // Compiles to path.remove() or Stmt::Empty
    }
}
```

**Compiles to:**

```javascript
// Babel
if (shouldRemove(node)) {
    path.remove();
}
```

```rust
// SWC
if should_remove(node) {
    *node = Stmt::Empty(EmptyStmt { span: DUMMY_SP });
}
```

### 6.5 List/Sibling Manipulation

Inserting siblings is difficult in SWC's VisitMut. ReluxScript solves this by exposing a `flat_map_in_place` helper:

```reluxscript
fn visit_block_statement(node: &mut BlockStatement, ctx: &Context) {
    // "stmts" is a special view of the block's body
    node.stmts.flat_map_in_place(|stmt| {
        if stmt.is_return() {
            // Replace 1 statement with 2 (Injection)
            vec![
                Statement::expression(log_call()),
                stmt.clone()
            ]
        } else {
            // Keep as-is
            vec![stmt.clone()]
        }
    });
}
```

**Compiles to:**

```javascript
// Babel: Iterates path.get('body') and uses path.replaceWithMultiple()
const body = path.get('body');
const newBody = [];
for (const stmt of body) {
    if (t.isReturnStatement(stmt.node)) {
        newBody.push(t.expressionStatement(logCall()));
        newBody.push(stmt.node);
    } else {
        newBody.push(stmt.node);
    }
}
path.node.body = newBody;
```

```rust
// SWC: Uses flat_map inside visit_mut_block_stmt
let new_stmts: Vec<Stmt> = node.stmts.iter().flat_map(|stmt| {
    if matches!(stmt, Stmt::Return(_)) {
        vec![
            Stmt::Expr(ExprStmt { expr: Box::new(log_call()), span: DUMMY_SP }),
            stmt.clone()
        ]
    } else {
        vec![stmt.clone()]
    }
}).collect();
node.stmts = new_stmts;
```

### 6.6 Property Mutation (Forbidden)

Direct property mutation is NOT allowed:

```reluxscript
// ERROR: Cannot mutate properties directly
node.name = "newName";
```

**Rationale:** Babel's scope tracker may not be notified. Always replace the whole node or use clone-and-rebuild pattern.

### 6.7 Traversal Control

```reluxscript
fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    // Process children first (post-order)
    node.visit_children(self);

    // Or skip children entirely
    // (don't call visit_children)
}
```

### 6.8 Scoped Traversal (`traverse`)

ReluxScript allows interrupting the current visitor to perform a scoped traversal on a specific node using a different set of rules. This bridges the impedance mismatch between Babel's `path.traverse` (graph walk) and SWC's `visit_mut_with` (recursive function call).

#### 5.8.1 Inline Traversal

Use `traverse(node) { ... }` to define a one-off visitor for a subtree:

```reluxscript
fn visit_function_declaration(func: &mut FunctionDeclaration, ctx: &Context) {
    for stmt in &mut func.body.stmts {
        if stmt.is_if_statement() {
            // Spawn a nested visitor for just this statement
            traverse(stmt) {
                // Local state (becomes struct fields in Rust, object properties in JS)
                let found_returns = 0;

                fn visit_return_statement(ret: &mut ReturnStatement, ctx: &Context) {
                    ret.argument = None;
                    self.found_returns += 1;
                }
            }
        }
    }
}
```

**Compiles to:**

```javascript
// Babel
const __nestedVisitor = {
    state: { found_returns: 0 },
    ReturnStatement(path) {
        const ret = path.node;
        ret.argument = null;
        this.state.found_returns++;
    },
};
stmt.traverse(__nestedVisitor);
```

```rust
// SWC (with hoisted struct)
struct __InlineVisitor_0 {
    found_returns: i32,
}

impl VisitMut for __InlineVisitor_0 {
    fn visit_mut_return_stmt(&mut self, ret: &mut ReturnStmt) {
        ret.arg = None;
        self.found_returns += 1;
    }
}

// At usage site:
let mut __visitor = __InlineVisitor_0 { found_returns: 0 };
stmt.visit_mut_with(&mut __visitor);
```

**Capture Rules:**
- **Immutable**: Ambient variables from the parent scope can be read (cloned into the new visitor)
- **Mutable**: Ambient variables cannot be mutated from inside `traverse` (Rust borrow checker). Pass data via initial state instead.

#### 5.8.2 Delegated Traversal (`using`)

Use `traverse(node) using VisitorName` to apply a separately defined plugin/visitor:

```reluxscript
plugin CleanUp {
    fn visit_identifier(n: &mut Identifier, ctx: &Context) {
        // cleanup logic
    }
}

plugin Main {
    fn visit_function(node: &mut Function, ctx: &Context) {
        if node.is_async {
            // Route this subtree through the CleanUp visitor
            traverse(node) using CleanUp;
        }
    }
}
```

**Compiles to:**

```javascript
// Babel
node.traverse(CleanUp);
```

```rust
// SWC
let mut __visitor = CleanUp::default();
node.visit_mut_with(&mut __visitor);
```

#### 5.8.3 Manual Iteration Pattern

This pattern is required when you want to selectively visit children:

```reluxscript
fn visit_block_statement(node: &mut BlockStatement, ctx: &Context) {
    // 1. Do NOT call node.visit_children(self);

    // 2. Manually iterate
    for stmt in &mut node.stmts {
        if needs_special_handling(stmt) {
            traverse(stmt) using SpecialVisitor;
        } else {
            // Continue with current visitor
            stmt.visit_with(self);
        }
    }
}
```

#### 5.8.4 The "Root Node" Guarantee

ReluxScript guarantees that the visitor runs on the node passed to `traverse`, not just its children:

- **SWC**: Maps to `node.visit_mut_with(&mut visitor)`
- **Babel**: Maps to `path.traverse(visitor)` + manual visit of `path.node`

This ensures consistent behavior across both targets.

---

## 19. Pattern Matching

### 6.1 The `matches!` Macro

Pattern matching that works on both targets:

```reluxscript
if matches!(node.callee, MemberExpression {
    object: Identifier { name: "console" },
    property: Identifier { name: "log" }
}) {
    // matched
}
```

**Compiles to:**

```javascript
// Babel
if (
    t.isMemberExpression(node.callee) &&
    t.isIdentifier(node.callee.object, { name: "console" }) &&
    t.isIdentifier(node.callee.property, { name: "log" })
) {
    // matched
}
```

```rust
// SWC
if let Callee::Expr(expr) = &node.callee {
    if let Expr::Member(member) = &**expr {
        if let Expr::Ident(obj) = &*member.obj {
            if &*obj.sym == "console" {
                if let MemberProp::Ident(prop) = &member.prop {
                    if &*prop.sym == "log" {
                        // matched
                    }
                }
            }
        }
    }
}
```

### 6.2 Match Expression

```reluxscript
match node.operator {
    "+" => handle_add(),
    "-" => handle_sub(),
    "*" | "/" => handle_mul_div(),
    _ => handle_default(),
}
```

### 6.3 If-Let Pattern

```reluxscript
if let Some(name) = get_identifier_name(node) {
    // use name
}
```

**Compiles to:**

```javascript
// Babel
const name = getIdentifierName(node);
if (name != null) {
    // use name
}
```

```rust
// SWC
if let Some(name) = get_identifier_name(node) {
    // use name
}
```

---

## 19. Unified AST (U-AST)

### 7.1 Node Mapping Table

| ReluxScript | Babel (ESTree) | SWC |
|------------|----------------|-----|
| `Identifier` | `t.Identifier` | `Ident` |
| `CallExpression` | `t.CallExpression` | `CallExpr` |
| `MemberExpression` | `t.MemberExpression` | `MemberExpr` |
| `BinaryExpression` | `t.BinaryExpression` | `BinExpr` |
| `UnaryExpression` | `t.UnaryExpression` | `UnaryExpr` |
| `StringLiteral` | `t.StringLiteral` | `Str` |
| `NumericLiteral` | `t.NumericLiteral` | `Number` |
| `BooleanLiteral` | `t.BooleanLiteral` | `Bool` |
| `NullLiteral` | `t.NullLiteral` | `Null` |
| `ArrayExpression` | `t.ArrayExpression` | `ArrayLit` |
| `ObjectExpression` | `t.ObjectExpression` | `ObjectLit` |
| `ArrowFunctionExpression` | `t.ArrowFunctionExpression` | `ArrowExpr` |
| `FunctionDeclaration` | `t.FunctionDeclaration` | `FnDecl` |
| `VariableDeclaration` | `t.VariableDeclaration` | `VarDecl` |
| `IfStatement` | `t.IfStatement` | `IfStmt` |
| `ReturnStatement` | `t.ReturnStatement` | `ReturnStmt` |
| `BlockStatement` | `t.BlockStatement` | `BlockStmt` |
| `ExpressionStatement` | `t.ExpressionStatement` | `ExprStmt` |
| `JSXElement` | `t.JSXElement` | `JSXElement` |
| `JSXAttribute` | `t.JSXAttribute` | `JSXAttr` |
| `JSXExpressionContainer` | `t.JSXExpressionContainer` | `JSXExprContainer` |
| `JSXText` | `t.JSXText` | `JSXText` |

### 7.2 TypeScript Node Mapping

| ReluxScript | Babel (ESTree) | SWC |
|------------|----------------|-----|
| `TSInterfaceDeclaration` | `TSInterfaceDeclaration` | `TsInterfaceDecl` |
| `TSPropertySignature` | `TSPropertySignature` | `TsPropertySignature` |
| `TSMethodSignature` | `TSMethodSignature` | `TsMethodSignature` |
| `TSTypeReference` | `TSTypeReference` | `TsTypeRef` |
| `TSTypeAnnotation` | `TSTypeAnnotation` | `TsTypeAnn` |
| `TSTypeAliasDeclaration` | `TSTypeAliasDeclaration` | `TsTypeAliasDecl` |

**Note:** TypeScript nodes use the `TS` prefix in ReluxScript to distinguish them from JavaScript nodes. When accessing fields on TypeScript nodes, be aware that:
- `key` on `TSPropertySignature` is `Box<Expr>` in SWC (requires pattern matching to extract identifier name)
- `type_args` on `CallExpression` provides access to generic type arguments like `useState<string>`

### 7.3 Node Construction

```reluxscript
// Creating nodes
let id = Identifier::new("myVar");
let call = CallExpression {
    callee: id,
    arguments: vec![StringLiteral::new("arg")],
};
```

**Compiles to:**

```javascript
// Babel
const id = t.identifier("myVar");
const call = t.callExpression(id, [t.stringLiteral("arg")]);
```

```rust
// SWC
let id = Ident::new("myVar".into(), DUMMY_SP);
let call = CallExpr {
    callee: Callee::Expr(Box::new(Expr::Ident(id))),
    args: vec![ExprOrSpread {
        expr: Box::new(Expr::Lit(Lit::Str(Str::from("arg")))),
        spread: None,
    }],
    ..Default::default()
};
```

### 7.4 Node Type Checking

```reluxscript
// Type checking
if node.is_identifier() { }
if node.is_call_expression() { }
```

**Compiles to:**

```javascript
// Babel
if (t.isIdentifier(node)) { }
if (t.isCallExpression(node)) { }
```

```rust
// SWC
if matches!(node, Expr::Ident(_)) { }
if matches!(node, Expr::Call(_)) { }
```

---

## 19. String Handling

### 8.1 The `Str` Type

All strings in ReluxScript use the `Str` type:

```reluxscript
let name: Str = "hello";
```

### 8.2 String Comparison

```reluxscript
if node.name == "console" {
    // works on both targets
}
```

**Compiles to:**

```javascript
// Babel
if (node.name === "console") { }
```

```rust
// SWC: JsWord implements PartialEq<&str>
if &*node.sym == "console" { }
```

### 8.3 String Methods

| ReluxScript | Babel (JS) | SWC (Rust) |
|------------|------------|------------|
| `s.starts_with("x")` | `s.startsWith("x")` | `s.starts_with("x")` |
| `s.ends_with("x")` | `s.endsWith("x")` | `s.ends_with("x")` |
| `s.contains("x")` | `s.includes("x")` | `s.contains("x")` |
| `s.len()` | `s.length` | `s.len()` |
| `s.is_empty()` | `s.length === 0` | `s.is_empty()` |
| `s.to_uppercase()` | `s.toUpperCase()` | `s.to_uppercase()` |
| `s.to_lowercase()` | `s.toLowerCase()` | `s.to_lowercase()` |

### 8.4 String Formatting

```reluxscript
let msg = format!("Hello, {}!", name);
```

**Compiles to:**

```javascript
// Babel
const msg = `Hello, ${name}!`;
```

```rust
// SWC
let msg = format!("Hello, {}!", name);
```

---

## 19. Option Handling

### 9.1 Creating Options

```reluxscript
let some_value = Some(42);
let no_value: Option<i32> = None;
```

### 9.2 Unwrapping

```reluxscript
// Safe unwrap with default
let value = opt.unwrap_or(default);
let value = opt.unwrap_or_else(|| compute_default());

// Conditional unwrap
if let Some(v) = opt {
    // use v
}

// Map transformation
let mapped = opt.map(|v| v + 1);

// Chain options
let result = opt.and_then(|v| other_option(v));
```

**Compiles to:**

```javascript
// Babel
const value = opt ?? default;
const value = opt ?? computeDefault();

if (opt != null) {
    const v = opt;
    // use v
}

const mapped = opt != null ? opt + 1 : null;
const result = opt != null ? otherOption(opt) : null;
```

```rust
// SWC: Direct Rust Option methods
let value = opt.unwrap_or(default);
let value = opt.unwrap_or_else(|| compute_default());

if let Some(v) = opt {
    // use v
}

let mapped = opt.map(|v| v + 1);
let result = opt.and_then(|v| other_option(v));
```

---

## 19. Collections

### 10.1 Vec Operations

```reluxscript
let mut items: Vec<i32> = vec![];

items.push(1);
items.push(2);

let first = items.get(0);           // Option<&i32>
let len = items.len();
let is_empty = items.is_empty();

for item in &items {
    // iterate
}

let doubled: Vec<i32> = items.iter().map(|x| x * 2).collect();
```

### 10.2 HashMap Operations

```reluxscript
let mut map: HashMap<Str, i32> = HashMap::new();

map.insert("key", 42);
let value = map.get("key");         // Option<&i32>
let has_key = map.contains_key("key");

for (key, value) in &map {
    // iterate
}
```

### 10.3 HashSet Operations

```reluxscript
let mut set: HashSet<Str> = HashSet::new();

set.insert("item");
let has_item = set.contains("item");
```

---

## 19. Control Flow

### 11.1 If/Else

```reluxscript
if condition {
    // then
} else if other_condition {
    // else if
} else {
    // else
}
```

### 11.2 Match

```reluxscript
match value {
    Pattern1 => expression1,
    Pattern2 | Pattern3 => expression2,
    _ => default_expression,
}
```

### 11.3 Loops

```reluxscript
// For-in loop
for item in collection {
    // body
}

// While loop
while condition {
    // body
}

// Loop with break
loop {
    if done {
        break;
    }
}
```

### 11.4 Early Return

```reluxscript
fn process(node: &Node) -> Option<Str> {
    if !node.is_valid() {
        return None;
    }

    // continue processing
    Some(node.name.clone())
}
```

---

## 19. Standard Library (Prelude)

### 12.1 Automatically Imported

```reluxscript
// These are always available
Option, Some, None
Vec, vec!
HashMap, HashSet
Str
format!
matches!
```

### 12.2 AST Utilities

```reluxscript
/// Check if identifier matches a name
fn is_identifier_named(node: &Identifier, name: &Str) -> bool;

/// Get identifier name safely
fn get_identifier_name(expr: &Expression) -> Option<Str>;

/// Check if node is a specific literal value
fn is_literal_value<T>(node: &Expression, value: T) -> bool;
```

### 12.3 String Utilities

```reluxscript
/// Convert to PascalCase
fn to_pascal_case(s: &Str) -> Str;

/// Convert to camelCase
fn to_camel_case(s: &Str) -> Str;

/// Convert to snake_case
fn to_snake_case(s: &Str) -> Str;

/// Escape string for code generation
fn escape_string(s: &Str) -> Str;
```

---

## 20. Regex Support

ReluxScript provides unified regex pattern matching through the `Regex::` namespace. All methods are static and patterns must be compile-time string literals.

### 20.1 Supported Syntax

ReluxScript supports the **intersection** of JavaScript and Rust regex features:

**✅ Fully Supported:**
- Character classes: `[abc]`, `[^abc]`, `[a-z]`, `[0-9]`
- Predefined classes: `.`, `\d`, `\D`, `\w`, `\W`, `\s`, `\S`
- Anchors: `^`, `$`, `\b`, `\B`
- Quantifiers: `*`, `+`, `?`, `{n}`, `{n,}`, `{n,m}` (greedy and lazy)
- Groups: `(...)`, `(?:...)`, `|`
- Named captures: `(?P<name>...)` (Rust syntax, works in both targets)
- Lookahead: `(?=...)`, `(?!...)`

**❌ Not Supported:**
- Lookbehind: `(?<=...)`, `(?<!...)` (Rust `regex` crate limitation)
- Backreferences: `\1`, `\2`, `\k<name>` (Rust limitation)

### 20.2 API Reference

```reluxscript
/// Test if pattern matches anywhere in text
Regex::matches(text: &Str, pattern: &str) -> bool

/// Find first match
Regex::find(text: &Str, pattern: &str) -> Option<Str>

/// Find all matches
Regex::find_all(text: &Str, pattern: &str) -> Vec<Str>

/// Extract capture groups from first match
Regex::captures(text: &Str, pattern: &str) -> Option<Captures>

/// Replace first match
Regex::replace(text: &Str, pattern: &str, replacement: &Str) -> Str

/// Replace all matches
Regex::replace_all(text: &Str, pattern: &str, replacement: &Str) -> Str
```

### 20.3 Examples

**Test for React hooks:**
```reluxscript
fn is_hook(name: &Str) -> bool {
    Regex::matches(name, r"^use[A-Z]\w*$")
}
```

**Extract version numbers:**
```reluxscript
fn extract_version(text: &Str) -> Option<(i32, i32, i32)> {
    if let Some(caps) = Regex::captures(text, r"^v?(\d+)\.(\d+)\.(\d+)") {
        let major = caps.get(1).parse::<i32>().ok()?;
        let minor = caps.get(2).parse::<i32>().ok()?;
        let patch = caps.get(3).parse::<i32>().ok()?;
        Some((major, minor, patch))
    } else {
        None
    }
}
```

**Sanitize identifiers:**
```reluxscript
fn sanitize_identifier(name: &Str) -> Str {
    Regex::replace_all(name, r"[^a-zA-Z0-9_]", "_")
}
```

### 20.4 Compilation

**Babel Target:**
```javascript
// Regex::matches(name, r"^use[A-Z]")
/^use[A-Z]/.test(name)

// Regex::captures(text, r"(\d+)")
__regex_captures(text, /(\d+)/)
```

**SWC Target:**
```rust
// Regex::matches(name, r"^use[A-Z]")
regex::Regex::new(r"^use[A-Z]").unwrap().is_match(name)

// Cached patterns (in loops/visitor methods)
lazy_static::lazy_static! {
    static ref REGEX_0: Regex = Regex::new(r"^use[A-Z]").unwrap();
}
```

See [REGEX_SUPPORT.md](REGEX_SUPPORT.md) for complete implementation details.

---

## 21. Custom AST Properties

ReluxScript allows attaching custom metadata to AST nodes using the `__` prefix convention. Properties are stored separately and work identically across both Babel and SWC targets.

### 21.1 Property Assignment

Custom properties start with double underscore `__`:

```reluxscript
plugin MetadataTracker {
    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        // Assign string property
        node.__componentName = "Button";

        // Assign boolean flag
        node.__processed = true;

        // Assign integer
        node.__visitCount = 42;
    }
}
```

### 21.2 Property Access

Read properties using if-let patterns:

```reluxscript
fn check_component(node: &JSXElement) {
    if let Some(name) = node.__componentName {
        println!("Component: {}", name);
    }

    if let Some(count) = node.__visitCount {
        // Increment and reassign
        node.__visitCount = count + 1;
    }
}
```

### 21.3 Supported Types

Custom properties support these value types:
- `String` (Str)
- `bool`
- `i32`, `i64`
- `f64`

Type is inferred from first assignment and remains consistent.

### 21.4 Property Deletion

```reluxscript
// Delete by assigning None
node.__processed = None;
```

### 21.5 Compilation

**SWC Target:**
```rust
// Generated infrastructure
#[derive(Clone, Debug)]
enum CustomPropValue {
    Bool(bool),
    I32(i32),
    I64(i64),
    F64(f64),
    Str(String),
}

struct State {
    // Your fields...
    __custom_props: HashMap<usize, HashMap<String, CustomPropValue>>,
}

impl State {
    fn get_node_id<T>(&self, node: &T) -> usize {
        node as *const T as usize
    }

    fn set_custom_prop<T>(&mut self, node: &T, prop: &str, value: CustomPropValue) {
        let node_id = self.get_node_id(node);
        self.__custom_props
            .entry(node_id)
            .or_insert_with(HashMap::new)
            .insert(prop.to_string(), value);
    }

    fn get_custom_prop<T>(&self, node: &T, prop: &str) -> Option<&CustomPropValue> {
        let node_id = self.get_node_id(node);
        self.__custom_props.get(&node_id).and_then(|m| m.get(prop))
    }
}

// Usage
self.state.set_custom_prop(node, "__componentName", CustomPropValue::Str("Button".to_string()));

if let Some(name) = self.state.get_custom_prop(node, "__componentName")
    .and_then(|v| if let CustomPropValue::Str(v) = v { Some(v.clone()) } else { None })
{
    println!("Component: {}", name);
}
```

**Babel Target:**
```javascript
// Use WeakMap for storage
const __customProps = new WeakMap();

function setCustomProp(node, prop, value) {
    if (!__customProps.has(node)) {
        __customProps.set(node, {});
    }
    __customProps.get(node)[prop] = value;
}

function getCustomProp(node, prop) {
    return __customProps.get(node)?.[prop];
}

// Usage
setCustomProp(node, '__componentName', 'Button');

const name = getCustomProp(node, '__componentName');
if (name !== undefined) {
    console.log(`Component: ${name}`);
}
```

### 21.6 Use Cases

**Track transformation state:**
```reluxscript
plugin MultiPassTransform {
    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        if let Some(count) = node.__transformCount {
            if count >= 3 {
                return; // Already processed
            }
            node.__transformCount = count + 1;
        } else {
            node.__transformCount = 1;
        }

        // Apply transformation...
    }
}
```

**Store computed paths:**
```reluxscript
plugin PathTracker {
    fn visit_jsx_element(node: &mut JSXElement, ctx: &Context) {
        let path = compute_jsx_path(node);
        node.__elementPath = path;
    }

    fn visit_jsx_closing_element(node: &mut JSXClosingElement, ctx: &Context) {
        // Retrieve path set on opening element
        if let Some(path) = node.parent.__elementPath {
            validate_closing_tag(path, node);
        }
    }
}
```

See [CUSTOM_AST_PROPERTIES.md](CUSTOM_AST_PROPERTIES.md) for complete implementation details.

---

## 19. Error Handling

### 13.1 Result Type

ReluxScript uses the `Result<T, E>` type for error handling, which compiles to different representations in Babel and SWC.

```reluxscript
fn parse_value(s: &Str) -> Result<i32, Str> {
    if s.is_empty() {
        return Err("Empty string");
    }
    Ok(42)
}
```

**Babel Compilation:**
```javascript
function parse_value(s) {
  if (s.length === 0) {
    return { ok: false, error: "Empty string" };
  }
  return { ok: true, value: 42 };
}
```

**SWC Compilation:**
```rust
fn parse_value(s: &String) -> Result<i32, String> {
    if s.is_empty() {
        return Err("Empty string".to_string());
    }
    Ok(42)
}
```

### 13.2 The `?` Operator

The `?` operator provides automatic error propagation, early-returning on errors.

```reluxscript
fn process_file(path: &Str) -> Result<(), Str> {
    let ast = parser::parse_file(path)?;  // Early return on error
    // Use ast...
    Ok(())
}
```

**Babel Compilation:**
```javascript
function process_file(path) {
  const __result = parser.parse_file(path);
  if (!__result.ok) {
    return { ok: false, error: __result.error };
  }
  const ast = __result.value;
  // Use ast...
  return { ok: true, value: undefined };
}
```

**SWC Compilation:**
```rust
fn process_file(path: &String) -> Result<(), String> {
    let ast = parser::parse_file(path)?;  // Native Rust ? operator
    // Use ast...
    Ok(())
}
```

**Result Representation:**
- **Babel (JavaScript)**: `{ ok: boolean, value?: T, error?: E }`
- **SWC (Rust)**: Native `Result<T, E>` enum

### 13.3 Ok() and Err() Constructors

```reluxscript
fn validate(input: &Str) -> Result<Str, Str> {
    if input.len() > 0 {
        Ok(input.clone())
    } else {
        Err("Input is empty")
    }
}
```

Both `Ok()` and `Err()` compile to appropriate Result representations in each target platform.

---

## 19. Compilation Model

### 14.1 File Structure

```
my-plugin/
  src/
    lib.rs          # Main ReluxScript source
    helpers.rs      # Helper functions
  dist/
    index.js        # Babel output
    plugin.wasm     # SWC WASM output
  Cargo.toml        # Generated for Rust build
  package.json      # Generated for npm
```

### 14.2 Build Command

```bash
reluxscript build

# Options
reluxscript build --target babel    # JS only
reluxscript build --target swc      # Rust/WASM only
reluxscript build --target both     # Default: both targets
```

### 14.3 Compilation Phases

1. **Parse**: ReluxScript source → AST
2. **Type Check**: Validate types and ownership
3. **Lower**: Resolve macros, statement lowering
4. **Emit JS**: Generate Babel plugin
5. **Emit Rust**: Generate SWC plugin
6. **Bundle**: Package for distribution

---

## 19. Example: Complete Plugin

```reluxscript
/// Plugin that transforms React hooks for analysis
plugin HookAnalyzer {

    // State tracked across the visitor
    struct State {
        hooks: Vec<HookInfo>,
        current_component: Option<Str>,
    }

    struct HookInfo {
        name: Str,
        hook_type: Str,
        component: Str,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        // Check if this is a component (PascalCase name)
        let name = node.id.name.clone();
        if is_component_name(&name) {
            self.state.current_component = Some(name);
        }

        // Visit children
        node.visit_children(self);

        // Clear component context
        self.state.current_component = None;
    }

    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
        // Check for hook calls
        if let Some(component) = &self.state.current_component {
            if let Some(name) = get_callee_name(&node.callee) {
                if name.starts_with("use") {
                    self.state.hooks.push(HookInfo {
                        name: name.clone(),
                        hook_type: categorize_hook(&name),
                        component: component.clone(),
                    });
                }
            }
        }

        // Visit children
        node.visit_children(self);
    }
}

// Helper functions
fn is_component_name(name: &Str) -> bool {
    if name.is_empty() {
        return false;
    }
    let first_char = name.chars().next().unwrap();
    first_char.is_uppercase()
}

fn get_callee_name(callee: &Expression) -> Option<Str> {
    if let Expression::Identifier(id) = callee {
        Some(id.name.clone())
    } else {
        None
    }
}

fn categorize_hook(name: &Str) -> Str {
    match name.as_str() {
        "useState" | "useReducer" => "state".into(),
        "useEffect" | "useLayoutEffect" => "effect".into(),
        "useRef" => "ref".into(),
        "useMemo" | "useCallback" => "memoization".into(),
        _ => "custom".into(),
    }
}
```

---

## 19. The Writer API (For Transpilers)

This section supports "Read-Only" visitors used for transpilation rather than transformation (e.g., TSX to C#).

### 16.1 Writer Plugin Declaration

Writers use lifecycle hooks to manage state during traversal:
- **`pub fn pre()` (or `fn init()`)** - Initialize state before traversal begins
- **`pub fn exit()` (or `fn finish()`)** - Generate output after traversal completes
- **`pub fn visit_*(...)`** - Read-only visitor methods

**Note:** `init()` and `finish()` are convenient aliases for `pre()` and `exit()` respectively.

```reluxscript
writer TsxToCSharp {
    // Internal state (define as struct fields)
    struct State {
        builder: CodeBuilder,
    }

    // Pre-hook: Initialize state before traversal
    pub fn pre() -> State {
        State { builder: CodeBuilder::new() }
    }

    // Read-Only Visitor (Note: `node` is immutable &T, not &mut T)
    pub fn visit_jsx_element(node: &JSXElement) {
        let tag = node.opening_element.name.get_name();

        self.builder.append("new ");
        self.builder.append(tag);
        self.builder.append("({");

        // Recurse manually or use helpers
        self.visit_jsx_attributes(&node.opening_element.attributes);

        self.builder.append("})");
    }

    // Exit-hook: Called after all visitors complete
    pub fn exit(&self) -> Str {
        self.builder.to_string()
    }
}
```

**Compiles to:**

```javascript
// Babel: Writer with pre/exit hooks
module.exports = function({ types: t }) {
  let state;

  return {
    pre(file) {
      // Call the pre hook to initialize state
      state = { builder: new CodeBuilder() };
    },

    visitor: {
      JSXElement(path) {
         const node = path.node;
         const tag = node.openingElement.name.name;

         state.builder.append("new ");
         state.builder.append(tag);
         state.builder.append("({");

         // ... attribute handling ...

         state.builder.append("})");
      }
    },

    post(file) {
       // Call the exit hook to get output
       const output = state.builder.toString();
       file.metadata.output = output;
    }
  }
}
```

```rust
// SWC: Uses Visit (read-only) instead of VisitMut
pub struct TsxToCSharp {
    builder: CodeBuilder,
}

impl TsxToCSharp {
    // Called by the pre hook
    pub fn new() -> Self {
        Self { builder: CodeBuilder::new() }
    }

    // Called by the exit hook
    pub fn finish(self) -> String {
        self.builder.to_string()
    }
}

impl Visit for TsxToCSharp {
    fn visit_jsx_element(&mut self, n: &JSXElement) {
        let tag = get_jsx_element_name(&n.opening.name);

        self.builder.append("new ");
        self.builder.append(&tag);
        self.builder.append("({");

        // ... attribute handling ...

        self.builder.append("})");
    }
}
```

### 16.2 Writer vs Plugin

| Aspect | `plugin` | `writer` |
|--------|----------|----------|
| Visitor Type | `VisitMut` (mutable) | `Visit` (read-only) |
| Purpose | Transform AST | Generate output |
| Node Access | `&mut T` | `&T` |
| Output | Modified AST | String/CodeBuilder |

### 16.3 Example: React to Orleans Grain

```reluxscript
writer ReactToOrleans {
    struct State {
        builder: CodeBuilder,
    }

    pub fn pre() -> State {
        State { builder: CodeBuilder::new() }
    }

    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        self.builder.append("public class ");
        self.builder.append(node.id.name.clone());
        self.builder.append(" : Grain, I");
        self.builder.append(node.id.name.clone());
        self.builder.append(" {\n");

        self.builder.indent();
        // Visit children (logic to convert hooks to state fields)
        node.visit_children(self);
        self.builder.dedent();

        self.builder.append("}\n");
    }

    pub fn visit_call_expression(node: &CallExpression) {
        // Check for useState hooks
        if let Some(name) = get_callee_name(&node.callee) {
            if name == "useState" {
                self.emit_state_field(node);
                return;
            }
        }

        node.visit_children(self);
    }

    fn emit_state_field(&mut self, node: &CallExpression) {
        // Extract state name from parent (const [x, setX] = useState(...))
        // This would use ctx or tracked state
        self.builder.append("private ");
        self.builder.append("dynamic"); // infer type
        self.builder.append(" _state;\n");
    }

    pub fn exit(&self) -> Str {
        self.builder.to_string()
    }
}
```

**Alternative with `init/finish` aliases:**

```reluxscript
writer ReactToOrleans {
    struct State {
        builder: CodeBuilder,
    }

    // init() is an alias for pre()
    fn init() -> State {
        State { builder: CodeBuilder::new() }
    }

    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        self.builder.append("public class ");
        self.builder.append(node.id.name.clone());
        self.builder.append(" {\n");
        self.builder.append("}\n");
    }

    // finish() is an alias for exit()
    fn finish(&self) -> Str {
        self.builder.to_string()
    }
}
```

Both styles compile to the same Babel `pre()` and `post()` hooks and SWC `Visit` trait with lifecycle methods.

---

## 19. Context & Scoping

### 17.1 The Scope Cost

Because SWC does not track scope by default, accessing `ctx.scope` triggers a pre-pass analysis in the SWC target.

```reluxscript
fn visit_identifier(node: &mut Identifier, ctx: &Context) {
    // WARNING: This call is cheap in Babel but expensive in SWC (requires O(n) pre-pass)
    if ctx.scope.has_binding(&node.name) {
        // ...
    }
}
```

### 17.2 Scope Methods

| Method | Description | Babel Cost | SWC Cost |
|--------|-------------|------------|----------|
| `has_binding(name)` | Check if name is bound in scope | O(1) | O(n) pre-pass |
| `generate_uid(hint)` | Generate unique identifier | O(1) | O(n) tracking |
| `get_binding(name)` | Get binding info | O(1) | O(n) pre-pass |

### 17.3 Avoiding Scope Lookups

For performance in SWC, track bindings manually when possible:

```reluxscript
plugin OptimizedVisitor {
    // Track bindings ourselves
    bindings: HashSet<Str>,

    fn visit_variable_declarator(node: &mut VariableDeclarator, ctx: &Context) {
        if let Pattern::Identifier(id) = &node.id {
            self.bindings.insert(id.name.clone());
        }
        node.visit_children(self);
    }

    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        // Use our tracked bindings instead of ctx.scope
        if self.bindings.contains(&node.name) {
            // ...
        }
    }
}
```

---

## 19. Limitations

### 18.1 Not Supported

- Async/await (different semantics between targets)
- External library imports (no cross-platform guarantee)
- Direct DOM/Node.js APIs
- Regex literals (use string-based matching instead)
- Closures that capture mutable state (borrow checker issues)

### 16.2 Platform-Specific Escape Hatches

For cases requiring platform-specific code:

```reluxscript
#[cfg(target = "babel")]
fn babel_specific() {
    // Only compiled to Babel
}

#[cfg(target = "swc")]
fn swc_specific() {
    // Only compiled to SWC
}
```

**Warning:** Use sparingly. This breaks the "write once" guarantee.

---

## 19. Future Considerations

### 17.1 Planned Features

- Derive macros for common patterns
- LSP support for editor integration
- Source maps for debugging
- Advanced module features (re-exports, wildcards)
- Generic constraints on structs and enums
- Full trait system with trait bounds

### 17.2 Ecosystem

- Package registry for shared ReluxScript libraries
- Testing framework with dual-target test runner
- Benchmark suite comparing Babel vs SWC performance

---

## Appendix A: Grammar (EBNF)

```ebnf
program         = plugin_decl ;
plugin_decl     = "plugin" IDENT "{" plugin_body "}" ;
plugin_body     = { struct_decl | fn_decl | impl_block } ;

struct_decl     = "struct" IDENT "{" struct_fields "}" ;
struct_fields   = { IDENT ":" type "," } ;

fn_decl         = ["pub"] "fn" IDENT "(" params ")" ["->" type] block ;
params          = [ param { "," param } ] ;
param           = IDENT ":" type ;

type            = primitive_type | reference_type | container_type | tuple_type | IDENT ;
primitive_type  = "Str" | "i32" | "f64" | "bool" | "()" ;
reference_type  = "&" ["mut"] type ;
container_type  = ("Vec" | "Option" | "HashMap" | "HashSet" | "Result") "<" type_args ">" ;
type_args       = type { "," type } ;
tuple_type      = "(" [ type { "," type } ] ")" ;

block           = "{" { statement } "}" ;
statement       = let_stmt | expr_stmt | if_stmt | match_stmt | for_stmt
                | while_stmt | return_stmt | break_stmt | continue_stmt ;

let_stmt        = "let" ["mut"] IDENT [":" type] "=" expr ";" ;
expr_stmt       = expr ";" ;
if_stmt         = "if" expr block { "else" "if" expr block } [ "else" block ] ;
match_stmt      = "match" expr "{" { match_arm } "}" ;
match_arm       = pattern "=>" expr "," ;
for_stmt        = "for" IDENT "in" expr block ;
while_stmt      = "while" expr block ;
return_stmt     = "return" [expr] ";" ;

expr            = assignment | logical_or ;
assignment      = (deref | member) "=" expr | logical_or ;
logical_or      = logical_and { "||" logical_and } ;
logical_and     = equality { "&&" equality } ;
equality        = comparison { ("==" | "!=") comparison } ;
comparison      = term { ("<" | ">" | "<=" | ">=") term } ;
term            = factor { ("+" | "-") factor } ;
factor          = unary { ("*" | "/" | "%") unary } ;
unary           = ("!" | "-" | "*" | "&" ["mut"]) unary | call ;
call            = primary { "(" args ")" | "." IDENT | "[" expr "]" | "?" } ;
primary         = IDENT | literal | "(" expr ")" | struct_init | vec_init | closure | block_expr ;

literal         = STRING | NUMBER | "true" | "false" | "null" | "()" ;
struct_init     = IDENT "{" { IDENT ":" expr "," } "}" ;
vec_init        = "vec!" "[" [ expr { "," expr } ] "]" ;
closure         = "|" [ IDENT { "," IDENT } ] "|" ( expr | block ) ;
block_expr      = block ;
args            = [ expr { "," expr } ] ;
pattern         = literal | IDENT | "_" | struct_pattern | tuple_pattern ;
tuple_pattern   = "(" [ pattern { "," pattern } ] ")" ;
deref           = "*" IDENT ;
member          = expr "." IDENT ;
```

---

## Appendix B: Compilation Examples

### B.1 Simple Function

**ReluxScript:**
```reluxscript
fn is_hook_name(name: &Str) -> bool {
    name.starts_with("use") && name.len() > 3
}
```

**Babel Output:**
```javascript
function isHookName(name) {
    return name.startsWith("use") && name.length > 3;
}
```

**SWC Output:**
```rust
fn is_hook_name(name: &str) -> bool {
    name.starts_with("use") && name.len() > 3
}
```

### B.2 Pattern Matching

**ReluxScript:**
```reluxscript
fn get_string_value(node: &Expression) -> Option<Str> {
    if let Expression::StringLiteral(lit) = node {
        Some(lit.value.clone())
    } else {
        None
    }
}
```

**Babel Output:**
```javascript
function getStringValue(node) {
    if (t.isStringLiteral(node)) {
        return node.value;
    } else {
        return null;
    }
}
```

**SWC Output:**
```rust
fn get_string_value(node: &Expr) -> Option<String> {
    if let Expr::Lit(Lit::Str(lit)) = node {
        Some(lit.value.to_string())
    } else {
        None
    }
}
```

---

## Appendix C: Error Messages

### C.1 Ownership Errors

```
error[RS001]: implicit borrow not allowed
  --> src/lib.rs:10:15
   |
10 |     let name = node.name;
   |               ^^^^^^^^^^ help: use explicit clone: `node.name.clone()`
   |
   = note: ReluxScript requires explicit .clone() to extract values from references
```

### C.2 Mutation Errors

```
error[RS002]: direct property mutation not allowed
  --> src/lib.rs:15:5
   |
15 |     node.name = "new";
   |     ^^^^^^^^^^^^^^^^^
   |
   = note: replace the entire node instead of mutating properties
   = help: use `*node = NodeType { ... }` pattern
```

### C.3 Type Errors

```
error[RS003]: type mismatch
  --> src/lib.rs:20:20
   |
20 |     let count: Str = 42;
   |                      ^^ expected `Str`, found `i32`
```

---

*End of Specification*
