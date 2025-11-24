# ReluxScript Parser Module Specification

**Version:** 0.1.0
**Status:** Proposed Extension
**Addendum to:** reluxscript-specification.md v0.5.0

---

## Overview

The `parser` module provides runtime AST parsing capabilities for ReluxScript plugins that need to analyze imported files dynamically. This is essential for plugins that perform cross-file analysis, such as transpiling components that import custom hooks.

---

## Module: `parser`

### Import

```reluxscript
use parser;
```

---

## Core Functions

### `parser::parse_file(path: &Str) -> Result<Program, Str>`

Parse a TypeScript/JavaScript file from disk and return its AST.

**ReluxScript:**
```reluxscript
use parser;
use fs;

let ast = parser::parse_file("./useCounter.tsx")?;

// Traverse the parsed AST
for stmt in &ast.body {
    if let Statement::FunctionDeclaration(ref func) = stmt {
        // Analyze function...
    }
}
```

**Babel Compilation:**
```javascript
const babel = require('@babel/core');
const fs = require('fs');

const code = fs.readFileSync("./useCounter.tsx", "utf-8");
const ast = babel.parseSync(code, {
    filename: "./useCounter.tsx",
    presets: ['@babel/preset-typescript'],
    plugins: ['@babel/plugin-syntax-jsx'],
});
```

**SWC Compilation:**
```rust
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig};
use std::sync::Arc;

let source_map = Arc::new(SourceMap::default());
let code = std::fs::read_to_string("./useCounter.tsx")?;
let file = source_map.new_source_file(
    FileName::Real("./useCounter.tsx".into()),
    code,
);

let syntax = Syntax::Typescript(TsConfig {
    tsx: true,
    decorators: false,
    ..Default::default()
});

let mut parser = Parser::new(syntax, StringInput::from(&*file), None);
let ast = parser.parse_program().map_err(|e| format!("Parse error: {:?}", e))?;
```

---

### `parser::parse(code: &Str) -> Result<Program, Str>`

Parse source code from a string and return its AST.

**ReluxScript:**
```reluxscript
use parser;

let code = "function foo() { return 42; }";
let ast = parser::parse(code)?;
```

**Babel Compilation:**
```javascript
const babel = require('@babel/core');

const code = "function foo() { return 42; }";
const ast = babel.parseSync(code, {
    presets: ['@babel/preset-typescript'],
    plugins: ['@babel/plugin-syntax-jsx'],
});
```

**SWC Compilation:**
```rust
use swc_common::{FileName, SourceMap};
use swc_ecma_parser::{Parser, StringInput, Syntax, TsConfig};
use std::sync::Arc;

let source_map = Arc::new(SourceMap::default());
let code = "function foo() { return 42; }";
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
let ast = parser.parse_program().map_err(|e| format!("Parse error: {:?}", e))?;
```

---

### `parser::parse_with_syntax(code: &Str, syntax: Syntax) -> Result<Program, Str>`

Parse source code with a specific syntax configuration.

**ReluxScript:**
```reluxscript
use parser;

let code = "const x: number = 42;";
let ast = parser::parse_with_syntax(code, parser::Syntax::TypeScript)?;

let jsx_code = "<div>Hello</div>";
let ast2 = parser::parse_with_syntax(jsx_code, parser::Syntax::JSX)?;
```

**Babel Compilation:**
```javascript
const babel = require('@babel/core');

// TypeScript
const ast1 = babel.parseSync("const x: number = 42;", {
    presets: ['@babel/preset-typescript'],
});

// JSX
const ast2 = babel.parseSync("<div>Hello</div>", {
    plugins: ['@babel/plugin-syntax-jsx'],
});
```

**SWC Compilation:**
```rust
use swc_ecma_parser::{Syntax, TsConfig, EsConfig};

// TypeScript
let ts_syntax = Syntax::Typescript(TsConfig {
    tsx: false,
    ..Default::default()
});

// JSX
let jsx_syntax = Syntax::Es(EsConfig {
    jsx: true,
    ..Default::default()
});
```

---

## Syntax Types

### `parser::Syntax` enum

Specifies the parsing syntax mode.

**Variants:**

```reluxscript
pub enum Syntax {
    JavaScript,   // Plain JavaScript (ES2020+)
    TypeScript,   // TypeScript with JSX support
    JSX,          // JavaScript with JSX
}
```

**Usage:**
```reluxscript
let ast = parser::parse_with_syntax(code, parser::Syntax::TypeScript)?;
```

---

## Parse Options (Advanced)

### `parser::ParseOptions` struct

Fine-grained control over parser behavior.

**ReluxScript:**
```reluxscript
use parser;

let options = parser::ParseOptions {
    syntax: parser::Syntax::TypeScript,
    jsx: true,
    decorators: false,
    dynamic_import: true,
};

let ast = parser::parse_with_options(code, options)?;
```

**Fields:**
- `syntax: Syntax` - Base syntax mode (JavaScript, TypeScript, JSX)
- `jsx: bool` - Enable JSX syntax (default: true for TypeScript)
- `decorators: bool` - Enable decorator syntax (default: false)
- `dynamic_import: bool` - Enable dynamic import() (default: true)

**Babel Compilation:**
```javascript
const babel = require('@babel/core');

const options = {
    presets: [
        ['@babel/preset-typescript', {
            isTSX: true,
            allExtensions: true
        }]
    ],
    plugins: [
        '@babel/plugin-syntax-jsx',
        '@babel/plugin-syntax-dynamic-import',
    ],
};

const ast = babel.parseSync(code, options);
```

**SWC Compilation:**
```rust
use swc_ecma_parser::{Syntax, TsConfig};

let syntax = Syntax::Typescript(TsConfig {
    tsx: true,
    decorators: false,
    dynamic_import: true,
    ..Default::default()
});
```

---

## Complete Example: Cross-File Hook Analysis

```reluxscript
use parser;
use fs;
use path;

/**
 * Analyze imported hooks from other files
 */
pub fn analyze_imported_hooks(
    program: &Program,
    current_file_path: &Str
) -> HashMap<Str, HookMetadata> {
    let mut hooks = HashMap::new();
    let current_dir = path::dirname(current_file_path);

    // Find import declarations
    for stmt in &program.body {
        if let Statement::ImportDeclaration(ref import) = stmt {
            let source = &import.source.value;

            // Only process relative imports
            if source.starts_with("./") || source.starts_with("../") {
                // Resolve import path
                let resolved_path = resolve_import_path(source, &current_dir)?;

                // Check if file exists
                if fs::exists(&resolved_path) {
                    // Parse the imported file
                    let imported_ast = parser::parse_file(&resolved_path)?;

                    // Extract hooks from the imported file
                    let file_hooks = extract_hooks_from_program(&imported_ast);

                    // Map imported names to hook metadata
                    for spec in &import.specifiers {
                        match spec {
                            ImportSpecifier::ImportDefaultSpecifier(ref default) => {
                                let local_name = &default.local.name;
                                if let Some(hook) = find_default_export_hook(&file_hooks) {
                                    hooks.insert(local_name.clone(), hook);
                                }
                            }

                            ImportSpecifier::ImportSpecifier(ref named) => {
                                let imported_name = &named.imported.name;
                                let local_name = &named.local.name;

                                if let Some(hook) = file_hooks.get(imported_name) {
                                    hooks.insert(local_name.clone(), hook.clone());
                                }
                            }

                            _ => {}
                        }
                    }
                }
            }
        }
    }

    hooks
}

/**
 * Extract hook definitions from a parsed AST
 */
fn extract_hooks_from_program(program: &Program) -> HashMap<Str, HookMetadata> {
    let mut hooks = HashMap::new();

    for stmt in &program.body {
        match stmt {
            // export function useCounter(...) { ... }
            Statement::ExportNamedDeclaration(ref export) => {
                if let Some(ref decl) = export.declaration {
                    if let Declaration::FunctionDeclaration(ref func) = decl {
                        if is_hook_function(func) {
                            let metadata = analyze_hook_function(func);
                            hooks.insert(metadata.name.clone(), metadata);
                        }
                    }
                }
            }

            // export default function useCounter(...) { ... }
            Statement::ExportDefaultDeclaration(ref export) => {
                if let Declaration::FunctionDeclaration(ref func) = export.declaration {
                    if is_hook_function(func) {
                        let metadata = analyze_hook_function(func);
                        hooks.insert("default".to_string(), metadata);
                    }
                }
            }

            // function useCounter(...) { ... }
            Statement::FunctionDeclaration(ref func) => {
                if is_hook_function(func) {
                    // Check if it's exported later
                    let metadata = analyze_hook_function(func);
                    hooks.insert(metadata.name.clone(), metadata);
                }
            }

            _ => {}
        }
    }

    hooks
}

/**
 * Check if function is a hook (starts with "use")
 */
fn is_hook_function(func: &FunctionDeclaration) -> bool {
    if let Some(ref id) = func.id {
        id.name.starts_with("use")
    } else {
        false
    }
}

/**
 * Resolve import path relative to current directory
 */
fn resolve_import_path(import_source: &Str, current_dir: &Str) -> Result<Str, Str> {
    let extensions = vec![".tsx", ".ts", ".jsx", ".js"];

    for ext in extensions {
        let with_ext = if import_source.ends_with(ext) {
            import_source.clone()
        } else {
            format!("{}{}", import_source, ext)
        };

        let resolved = path::join(vec![current_dir.clone(), with_ext]);

        if fs::exists(&resolved) {
            return Ok(resolved);
        }
    }

    Err(format!("Could not resolve import: {}", import_source))
}
```

---

## Error Handling

Parse errors return `Result<Program, Str>` with error messages:

```reluxscript
use parser;

match parser::parse_file("./badfile.tsx") {
    Ok(ast) => {
        // Process AST
    }
    Err(error) => {
        // Handle parse error
        eprintln!("Parse error: {}", error);
    }
}
```

**Babel Error Format:**
```javascript
try {
    const ast = babel.parseSync(code, options);
} catch (error) {
    // error.message contains parse error details
    return Err(error.message);
}
```

**SWC Error Format:**
```rust
match parser.parse_program() {
    Ok(ast) => Ok(ast),
    Err(error) => {
        let msg = format!("Parse error at line {}: {}",
            error.span().lo,
            error
        );
        Err(msg)
    }
}
```

---

## Performance Considerations

### Caching

For plugins that parse the same file multiple times, consider caching:

```reluxscript
use parser;

struct ParserCache {
    cache: HashMap<Str, Program>,
}

impl ParserCache {
    pub fn new() -> ParserCache {
        ParserCache {
            cache: HashMap::new(),
        }
    }

    pub fn get_or_parse(&mut self, file_path: &Str) -> Result<Program, Str> {
        if let Some(cached) = self.cache.get(file_path) {
            return Ok(cached.clone());
        }

        let ast = parser::parse_file(file_path)?;
        self.cache.insert(file_path.clone(), ast.clone());
        Ok(ast)
    }
}
```

### Lazy Parsing

Only parse files when needed:

```reluxscript
// Bad: Parse all imports upfront
for import in &all_imports {
    let ast = parser::parse_file(&import.source)?; // Slow!
}

// Good: Parse only hook imports
for import in &all_imports {
    if looks_like_hook_import(import) {
        let ast = parser::parse_file(&import.source)?;
    }
}
```

---

## Limitations

1. **No AST Modification**: The parsed AST is read-only. You cannot modify and re-serialize it.
2. **Single Parse Pass**: Each `parse_file()` call re-parses the file. Use caching for repeated access.
3. **Platform Differences**: Minor AST structure differences between Babel and SWC may require conditional logic.

---

## Integration with Existing Modules

### Combined with `fs` module:

```reluxscript
use fs;
use parser;

// Read, parse, and analyze
let source = fs::read_file("./component.tsx")?;
let ast = parser::parse(&source)?;

// Modify and write back (requires code generation)
// Note: ReluxScript doesn't include codegen yet
```

### Combined with `path` module:

```reluxscript
use path;
use parser;

let import_path = "./hooks/useCounter";
let dir = path::dirname(current_file);
let resolved = path::join(vec![dir, import_path + ".tsx"]);

let ast = parser::parse_file(&resolved)?;
```

---

## Future Enhancements

### Planned Features:
- `parser::parse_module()` - Parse as ES module vs. script
- `parser::SourceLocation` - Preserve source locations for error reporting
- `parser::Comments` - Extract and preserve comments
- Code generation support (reverse operation)

---

## See Also

- [ReluxScript Specification](./reluxscript-specification.md) - Main language spec
- [Built-in Modules](./reluxscript-specification.md#65-built-in-modules) - fs, json, path modules
- [Babel Parser API](https://babeljs.io/docs/babel-parser)
- [SWC Parser Documentation](https://swc.rs/docs/usage/core#parse)
