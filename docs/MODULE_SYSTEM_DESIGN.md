# ReluxScript Module System Design

## Current State

The parser already supports basic `use` statements:
```reluxscript
use fs;
use json;
```

This works for **built-in modules** but we need support for **file-based modules**.

## Goal

Support multi-file ReluxScript projects with explicit imports/exports.

## Design

### 1. Syntax

#### 1.1 Import Syntax

```reluxscript
// Import all public exports from a module
use "./utils/helpers.rsc";

// Import specific functions/types
use "./utils/helpers.rsc" { get_component_name, escape_string };

// Import with alias
use "./extractors/hooks.rsc" as hooks;

// Import specific items with module alias
use "./extractors/hooks.rsc" as hooks { extract_useState };

// Built-in modules (no path)
use fs;
use json;
```

#### 1.2 Export Syntax

Use `pub` keyword to export:

```reluxscript
// File: utils/helpers.rsc

// Exported function (available for import)
pub fn get_component_name(node: &FunctionDeclaration) -> Str {
    node.id.name.clone()
}

// Private function (NOT available for import)
fn internal_helper() -> Str {
    "internal"
}

// Exported struct
pub struct ComponentInfo {
    name: Str,
    props: Vec<Str>,
}

// Exported enum
pub enum HookType {
    State,
    Effect,
    Ref,
}
```

### 2. Module Types

#### 2.1 File Modules

Regular `.rsc` files containing functions, structs, enums:

```reluxscript
// File: utils/helpers.rsc
pub fn get_component_name(node: &FunctionDeclaration) -> Str {
    node.id.name.clone()
}

pub fn escape_string(s: &Str) -> Str {
    // ...
}
```

#### 2.2 Plugin Modules (Entry Points)

Files with `plugin` or `writer` declarations:

```reluxscript
// File: main.rsc
use "./utils/helpers.rsc" { get_component_name };

plugin MinimactPlugin {
    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        let name = get_component_name(node);
    }
}
```

#### 2.3 Built-in Modules

Special modules provided by ReluxScript:
- `fs` - File system operations
- `json` - JSON serialization/deserialization
- `path` - Path manipulation

### 3. Module Resolution

#### 3.1 Resolution Rules

1. **Relative paths** (`./ ` or `../`): Resolve relative to current file
2. **Built-in names** (no `.` or `/`): Use built-in module
3. **Extensions**: `.rsc` extension can be omitted

```reluxscript
use "./helpers.rsc";    // OK
use "./helpers";         // OK - adds .rsc automatically
use "fs";                // Built-in module
use "../utils/foo";      // OK - relative to parent
```

#### 3.2 Resolution Algorithm

```
resolve_module(current_file_path, import_path):
    if import_path starts with "." or "/":
        // File-based module
        abs_path = resolve_relative(current_file_path, import_path)
        if not ends_with(".rsc"):
            abs_path += ".rsc"
        return abs_path
    else:
        // Built-in module
        return builtin_module(import_path)
```

### 4. Compilation Model

#### 4.1 Multi-File Compilation

```
Input:
  main.rsc
  utils/helpers.rsc
  extractors/hooks.rsc

Compilation Steps:
  1. Parse all .rsc files → ASTs
  2. Build dependency graph
  3. Topological sort (detect cycles)
  4. Type-check in dependency order
  5. Generate code for each file
  6. Link/bundle for target platform
```

#### 4.2 Babel Output (CommonJS)

```javascript
// File: utils/helpers.js
function getComponentName(node) {
    return node.id.name;
}

function escapeString(s) {
    // ...
}

module.exports = {
    getComponentName,
    escapeString,
};
```

```javascript
// File: main.js
const { getComponentName } = require('./utils/helpers.js');

module.exports = function({ types: t }) {
    return {
        visitor: {
            FunctionDeclaration(path) {
                const name = getComponentName(path.node);
            }
        }
    };
};
```

#### 4.3 SWC Output (Rust modules)

```rust
// File: utils/helpers.rs
pub fn get_component_name(node: &FnDecl) -> String {
    node.ident.sym.to_string()
}

pub fn escape_string(s: &str) -> String {
    // ...
}
```

```rust
// File: main.rs
mod utils {
    pub mod helpers;
}

use utils::helpers::get_component_name;

pub struct MinimactPlugin;

impl VisitMut for MinimactPlugin {
    fn visit_mut_fn_decl(&mut self, node: &mut FnDecl) {
        let name = get_component_name(node);
    }
}
```

### 5. AST Changes

#### 5.1 Enhanced UseStmt

```rust
#[derive(Debug, Clone)]
pub struct UseStmt {
    /// Module path (e.g., "./helpers.rsc" or "fs")
    pub path: String,

    /// Optional alias (as name)
    pub alias: Option<String>,

    /// Specific imports { foo, bar }
    pub imports: Vec<String>,

    pub span: Span,
}
```

**Examples**:
```reluxscript
use "./helpers.rsc";
// path = "./helpers.rsc", alias = None, imports = []

use "./helpers.rsc" as h;
// path = "./helpers.rsc", alias = Some("h"), imports = []

use "./helpers.rsc" { get_name, escape };
// path = "./helpers.rsc", alias = None, imports = ["get_name", "escape"]

use "./helpers.rsc" as h { get_name };
// path = "./helpers.rsc", alias = Some("h"), imports = ["get_name"]
```

#### 5.2 Module-Level Items

File modules can contain:
```rust
#[derive(Debug, Clone)]
pub struct Module {
    /// File path
    pub path: String,

    /// Use statements
    pub uses: Vec<UseStmt>,

    /// Module items (functions, structs, enums)
    pub items: Vec<ModuleItem>,

    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum ModuleItem {
    Function(FnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Impl(ImplBlock),
}
```

### 6. Parser Changes

#### 6.1 Enhanced parse_use_stmt()

```rust
fn parse_use_stmt(&mut self) -> ParseResult<UseStmt> {
    let start_span = self.current_span();
    self.expect(TokenKind::Use)?;

    // Parse module path (string literal or identifier)
    let path = if self.check(TokenKind::String) {
        self.expect_string()?
    } else {
        self.expect_ident()?
    };

    // Optional: as alias
    let alias = if self.check(TokenKind::As) {
        self.advance();
        Some(self.expect_ident()?)
    } else {
        None
    };

    // Optional: { imports }
    let imports = if self.check(TokenKind::LBrace) {
        self.parse_import_list()?
    } else {
        vec![]
    };

    self.expect(TokenKind::Semicolon)?;

    Ok(UseStmt {
        path,
        alias,
        imports,
        span: start_span,
    })
}

fn parse_import_list(&mut self) -> ParseResult<Vec<String>> {
    self.expect(TokenKind::LBrace)?;
    let mut imports = Vec::new();

    loop {
        imports.push(self.expect_ident()?);

        if !self.check(TokenKind::Comma) {
            break;
        }
        self.advance(); // consume comma
    }

    self.expect(TokenKind::RBrace)?;
    Ok(imports)
}
```

### 7. Compiler Changes

#### 7.1 Multi-File Compiler

```rust
pub struct Compiler {
    // Map of file paths to parsed modules
    modules: HashMap<String, Module>,

    // Dependency graph
    dependencies: HashMap<String, Vec<String>>,
}

impl Compiler {
    pub fn compile_project(&mut self, entry_point: &str) -> Result<(), Error> {
        // 1. Load and parse all modules
        self.load_module(entry_point)?;

        // 2. Build dependency graph
        self.build_dependency_graph()?;

        // 3. Topological sort (detect cycles)
        let sorted = self.topological_sort()?;

        // 4. Type-check in order
        for module_path in &sorted {
            self.type_check_module(module_path)?;
        }

        // 5. Generate code
        for module_path in &sorted {
            self.generate_code(module_path)?;
        }

        Ok(())
    }

    fn load_module(&mut self, path: &str) -> Result<(), Error> {
        if self.modules.contains_key(path) {
            return Ok(()); // Already loaded
        }

        // Read file
        let source = fs::read_to_string(path)?;

        // Parse
        let mut parser = Parser::new(&source);
        let ast = parser.parse()?;

        // Store module
        self.modules.insert(path.to_string(), ast);

        // Recursively load dependencies
        for use_stmt in &ast.uses {
            let dep_path = self.resolve_path(path, &use_stmt.path)?;
            self.load_module(&dep_path)?;
        }

        Ok(())
    }
}
```

### 8. Built-in Modules

#### 8.1 fs Module

```reluxscript
// Available after: use fs;

fs::read_file(path: Str) -> Result<Str, Str>
fs::write_file(path: Str, content: Str) -> Result<(), Str>
fs::exists(path: Str) -> bool
fs::read_dir(path: Str) -> Result<Vec<Str>, Str>
```

**Babel codegen**:
```javascript
const fs = require('fs');

fs.readFileSync(path, 'utf-8')
fs.writeFileSync(path, content)
fs.existsSync(path)
fs.readdirSync(path)
```

**SWC codegen**:
```rust
std::fs::read_to_string(path)
std::fs::write(path, content)
std::path::Path::new(path).exists()
std::fs::read_dir(path)
```

#### 8.2 json Module

```reluxscript
// Available after: use json;

json::stringify(value: dynamic) -> Str
json::parse(text: Str) -> Result<dynamic, Str>
```

**Babel codegen**:
```javascript
JSON.stringify(value, null, 2)
JSON.parse(text)
```

**SWC codegen**:
```rust
serde_json::to_string_pretty(&value)
serde_json::from_str(text)
```

#### 8.3 path Module

```reluxscript
// Available after: use path;

path::join(parts: Vec<Str>) -> Str
path::dirname(p: Str) -> Str
path::basename(p: Str) -> Str
path::extname(p: Str) -> Str
```

### 9. Example: Multi-File Project

```
minimact/
  main.rsc                  # Entry point (plugin declaration)
  utils/
    helpers.rsc             # Helper functions
    types.rsc               # Type conversion
  extractors/
    props.rsc               # Prop extraction
    hooks.rsc               # Hook extraction
```

#### main.rsc
```reluxscript
use "./utils/helpers.rsc" { get_component_name, escape_string };
use "./utils/types.rsc" { ts_to_csharp_type };
use "./extractors/props.rsc" { extract_props };
use "./extractors/hooks.rsc" { extract_hooks };
use fs;
use json;

plugin MinimactPlugin {
    struct State {
        components: Vec<ComponentInfo>,
    }

    struct ComponentInfo {
        name: Str,
        props: Vec<PropInfo>,
        hooks: Vec<HookInfo>,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        let name = get_component_name(node);
        let props = extract_props(node);
        let hooks = extract_hooks(node);

        let info = ComponentInfo {
            name: name.clone(),
            props,
            hooks,
        };

        self.state.components.push(info);
    }

    fn visit_program_exit(node: &Program, ctx: &Context) {
        for component in &self.state.components {
            let csharp = generate_csharp(component);
            let filename = format!("{}.cs", component.name);
            fs::write_file(filename, csharp)?;

            let json_data = json::stringify(component);
            let json_filename = format!("{}.meta.json", component.name);
            fs::write_file(json_filename, json_data)?;
        }
    }
}

fn generate_csharp(info: &ComponentInfo) -> Str {
    let mut code = format!("public class {} {{\n", info.name);

    for prop in &info.props {
        code += &format!("  public {} {};\n",
            ts_to_csharp_type(&prop.type_name),
            prop.name
        );
    }

    code += "}\n";
    code
}
```

#### utils/helpers.rsc
```reluxscript
pub fn get_component_name(node: &FunctionDeclaration) -> Str {
    node.id.name.clone()
}

pub fn escape_string(s: &Str) -> Str {
    s.replace("\\", "\\\\")
     .replace("\"", "\\\"")
     .replace("\n", "\\n")
}
```

#### extractors/props.rsc
```reluxscript
use "../utils/helpers.rsc" { escape_string };

pub struct PropInfo {
    pub name: Str,
    pub type_name: Str,
}

pub fn extract_props(node: &FunctionDeclaration) -> Vec<PropInfo> {
    let mut props = vec![];

    // Extract from function parameters
    if node.params.len() > 0 {
        let first_param = &node.params[0];
        // ... extraction logic ...
    }

    props
}
```

### 10. Implementation Plan

1. ✅ **Parser enhancement** (2-3 hours)
   - Update `UseStmt` AST
   - Implement enhanced `parse_use_stmt()`
   - Add `parse_import_list()`

2. ✅ **Module loader** (3-4 hours)
   - Create `Compiler` with multi-file support
   - Implement `load_module()` with recursive loading
   - Path resolution logic

3. ✅ **Dependency graph** (2-3 hours)
   - Build dependency graph
   - Topological sort
   - Cycle detection

4. ✅ **Babel codegen** (3-4 hours)
   - Generate `require()` statements
   - Generate `module.exports`
   - Handle import/export mapping

5. ✅ **SWC codegen** (3-4 hours)
   - Generate `mod` declarations
   - Generate `use` statements
   - Handle pub visibility

6. ✅ **Built-in modules** (2-3 hours)
   - Implement `fs` module
   - Implement `json` module
   - Implement `path` module

7. ✅ **Testing** (2-3 hours)
   - Create multi-file test projects
   - Test circular dependency detection
   - Test built-in modules

**Total estimated time**: 17-24 hours

### 11. Testing Strategy

#### Test 1: Simple two-file project
```
test1/
  main.rsc      - imports helpers
  helpers.rsc   - exports functions
```

#### Test 2: Multi-level imports
```
test2/
  main.rsc      - imports utils
  utils/
    index.rsc   - imports helpers
    helpers.rsc - exports functions
```

#### Test 3: Circular dependency detection
```
test3/
  a.rsc         - imports b
  b.rsc         - imports a (ERROR!)
```

#### Test 4: Built-in modules
```
test4/
  main.rsc      - uses fs, json, path
```

---

## Summary

This design provides:
- ✅ File-based modules with explicit imports
- ✅ Built-in modules (fs, json, path)
- ✅ Proper module resolution
- ✅ Babel (CommonJS) and SWC (Rust mod) output
- ✅ Cycle detection
- ✅ Clear export mechanism (`pub` keyword)

Next step: Start implementation with parser enhancements!
