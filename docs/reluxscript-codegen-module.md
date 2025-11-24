# ReluxScript Codegen Module Specification

**Version:** 0.1.0
**Status:** Proposed Enhancement
**Addendum to:** reluxscript-specification.md v0.6.0

---

## Overview

The `codegen` module provides the inverse of the `parser` module - it converts AST nodes back into source code strings. This is essential for plugins that need to serialize AST fragments, generate code dynamically, or extract code snippets for analysis.

---

## Module: `codegen`

### Import

```reluxscript
use codegen;
```

---

## Core Functions

### `codegen::generate(node: &Node) -> Str`

Convert any AST node back to source code.

**ReluxScript:**
```reluxscript
use codegen;

fn extract_expression_code(expr: &Expression) -> Str {
    let code = codegen::generate(expr);
    code
}

// Example usage in visitor
fn visit_binary_expression(node: &mut BinaryExpression, ctx: &Context) {
    // Get the source code representation
    let code_str = codegen::generate(node);

    // Now you can store it, log it, analyze it, etc.
    console::log(&format!("Found expression: {}", code_str));
}
```

**Babel Compilation:**
```javascript
const generate = require('@babel/generator').default;

function extract_expression_code(expr) {
    const result = generate(expr);
    const code = result.code;
    return code;
}

function visit_binary_expression(node, ctx) {
    const result = generate(node);
    const code_str = result.code;

    console.log(`Found expression: ${code_str}`);
}
```

**SWC Compilation:**
```rust
use swc_ecma_codegen::{text_writer::JsWriter, Emitter, Config};
use swc_common::sync::Lrc;
use swc_common::{SourceMap, FilePathMapping};

fn extract_expression_code(expr: &Expr) -> String {
    let code = codegen_to_string(expr);
    code
}

fn visit_binary_expression(&mut self, node: &mut BinExpr) {
    let code_str = codegen_to_string(node);

    println!("Found expression: {}", code_str);
}

// Helper function (generated once per file)
fn codegen_to_string<N>(node: &N) -> String
where
    N: swc_ecma_codegen::Node,
{
    let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
    let mut buf = vec![];
    {
        let mut emitter = Emitter {
            cfg: Config::default(),
            cm: cm.clone(),
            comments: None,
            wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
        };
        node.emit_with(&mut emitter).unwrap();
    }
    String::from_utf8(buf).unwrap()
}
```

---

### `codegen::generate_with_options(node: &Node, options: CodegenOptions) -> Str`

Convert AST to code with custom formatting options.

**ReluxScript:**
```reluxscript
use codegen;

let options = codegen::CodegenOptions {
    compact: true,
    minified: false,
    semicolons: true,
};

let code = codegen::generate_with_options(expr, options);
```

**Babel Compilation:**
```javascript
const generate = require('@babel/generator').default;

const options = {
    compact: true,
    minified: false,
    semicolons: true,
};

const result = generate(expr, options);
const code = result.code;
```

**SWC Compilation:**
```rust
use swc_ecma_codegen::Config;

let config = Config {
    minify: false,
    ascii_only: false,
    omit_last_semi: false,
    target: EsVersion::Es2020,
};

let code = codegen_to_string_with_config(expr, config);
```

---

## CodegenOptions

Configuration for code generation output.

```reluxscript
pub struct CodegenOptions {
    /// Compact output (minimal whitespace)
    pub compact: bool,

    /// Minified output (removes all unnecessary characters)
    pub minified: bool,

    /// Include semicolons
    pub semicolons: bool,

    /// Indentation (spaces per level)
    pub indent: i32,
}
```

**Default Options:**
```reluxscript
CodegenOptions {
    compact: false,
    minified: false,
    semicolons: true,
    indent: 2,
}
```

---

## Use Cases

### 1. Code Extraction for Analysis

Extract code snippets to store in metadata or analysis results.

```reluxscript
use codegen;
use json;
use fs;

plugin CodeExtractor {
    struct FunctionInfo {
        name: Str,
        source: Str,
        complexity: i32,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        let name = node.id.name.clone();

        // Convert entire function to source code
        let source = codegen::generate(node);

        let info = FunctionInfo {
            name: name.clone(),
            source: source,
            complexity: calculate_complexity(node),
        };

        // Save to JSON
        let json_str = json::stringify(&info);
        let filename = format!("{}.meta.json", name);
        fs::write_file(&filename, &json_str)?;
    }
}
```

### 2. Template String Generation

Generate code for templates or dynamic transformations.

```reluxscript
use codegen;

fn create_hook_wrapper(original_call: &CallExpression) -> Str {
    // Get the original call as a string
    let original = codegen::generate(original_call);

    // Build a template around it
    let wrapped = format!(
        "useTrackedState(() => {})",
        original
    );

    wrapped
}
```

### 3. Expression Comparison

Compare expressions by converting to normalized strings.

```reluxscript
use codegen;

fn are_expressions_equivalent(expr1: &Expression, expr2: &Expression) -> bool {
    let code1 = codegen::generate(expr1);
    let code2 = codegen::generate(expr2);

    // Normalize whitespace and compare
    normalize(&code1) == normalize(&code2)
}
```

### 4. Debug Logging

Log AST nodes in readable form during development.

```reluxscript
use codegen;

fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
    let callee_code = codegen::generate(&node.callee);

    if callee_code.starts_with("useState") {
        console::log(&format!("Found state hook: {}",
            codegen::generate(node)));
    }
}
```

### 5. Code Snippet Storage

Store frequently used patterns as strings for later injection.

```reluxscript
use codegen;

plugin PatternLibrary {
    struct State {
        patterns: HashMap<Str, Str>,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        // Check if function matches a pattern we want to save
        if is_reusable_pattern(node) {
            let name = node.id.name.clone();
            let code = codegen::generate(node);

            self.state.patterns.insert(name, code);
        }
    }

    fn visit_program_exit(node: &Program, ctx: &Context) {
        // Save all collected patterns
        let json = json::stringify(&self.state.patterns);
        fs::write_file("patterns.json", &json)?;
    }
}
```

---

## Complete Example: Expression Complexity Analyzer

This plugin analyzes expressions and stores complex ones as strings.

```reluxscript
use codegen;
use json;
use fs;

plugin ComplexityAnalyzer {
    struct State {
        complex_expressions: Vec<ExprInfo>,
    }

    struct ExprInfo {
        code: Str,
        complexity: i32,
        location: Location,
    }

    struct Location {
        file: Str,
        line: i32,
        column: i32,
    }

    fn visit_binary_expression(node: &mut BinaryExpression, ctx: &Context) {
        let complexity = calculate_complexity(node);

        // If expression is complex, save it
        if complexity > 5 {
            let code = codegen::generate(node);

            let info = ExprInfo {
                code: code,
                complexity: complexity,
                location: Location {
                    file: ctx.filename.clone(),
                    line: node.span.start.line,
                    column: node.span.start.column,
                },
            };

            self.state.complex_expressions.push(info);
        }
    }

    fn visit_program_exit(node: &Program, ctx: &Context) {
        // Generate report
        let report = json::stringify(&self.state.complex_expressions);
        fs::write_file("complexity-report.json", &report)?;
    }
}

fn calculate_complexity(expr: &BinaryExpression) -> i32 {
    let mut complexity = 1;

    // Recursively count operations
    if let Expression::BinaryExpression(ref left) = *expr.left {
        complexity += calculate_complexity(left);
    }
    if let Expression::BinaryExpression(ref right) = *expr.right {
        complexity += calculate_complexity(right);
    }

    complexity
}
```

**Output (complexity-report.json):**
```json
[
  {
    "code": "count * 2 + discount - shipping + tax",
    "complexity": 4,
    "location": {
      "file": "checkout.js",
      "line": 42,
      "column": 12
    }
  },
  {
    "code": "price * quantity * (1 - discount / 100) + (hasFreeShipping ? 0 : shippingCost)",
    "complexity": 6,
    "location": {
      "file": "cart.js",
      "line": 89,
      "column": 20
    }
  }
]
```

---

## Implementation Notes

### Babel Implementation

The Babel implementation will use `@babel/generator` with minimal wrapper:

```javascript
const generate = require('@babel/generator').default;

const codegen = {
  generate: (node) => {
    const result = generate(node);
    return result.code;
  },

  generate_with_options: (node, options) => {
    const babelOptions = {
      compact: options.compact || false,
      minified: options.minified || false,
      retainLines: false,
      comments: true,
    };
    const result = generate(node, babelOptions);
    return result.code;
  },
};
```

### SWC Implementation

The SWC implementation requires more boilerplate but will be wrapped in a helper module:

```rust
mod codegen {
    use swc_ecma_codegen::{text_writer::JsWriter, Emitter, Config};
    use swc_common::sync::Lrc;
    use swc_common::{SourceMap, FilePathMapping};

    pub fn generate<N>(node: &N) -> String
    where
        N: swc_ecma_codegen::Node,
    {
        generate_with_config(node, Config::default())
    }

    pub fn generate_with_options<N>(node: &N, options: &CodegenOptions) -> String
    where
        N: swc_ecma_codegen::Node,
    {
        let config = Config {
            minify: options.minified,
            ascii_only: false,
            omit_last_semi: !options.semicolons,
            target: EsVersion::Es2020,
        };
        generate_with_config(node, config)
    }

    fn generate_with_config<N>(node: &N, config: Config) -> String
    where
        N: swc_ecma_codegen::Node,
    {
        let cm = Lrc::new(SourceMap::new(FilePathMapping::empty()));
        let mut buf = vec![];
        {
            let mut emitter = Emitter {
                cfg: config,
                cm: cm.clone(),
                comments: None,
                wr: JsWriter::new(cm.clone(), "\n", &mut buf, None),
            };
            node.emit_with(&mut emitter).unwrap();
        }
        String::from_utf8(buf).unwrap()
    }
}
```

---

## Performance Considerations

### Caching

For expressions that are generated multiple times, consider caching:

```reluxscript
use codegen;

struct CachedCodegen {
    cache: HashMap<NodeId, Str>,
}

impl CachedCodegen {
    pub fn get_or_generate(&mut self, node: &Expression, id: NodeId) -> Str {
        if let Some(cached) = self.cache.get(&id) {
            return cached.clone();
        }

        let code = codegen::generate(node);
        self.cache.insert(id, code.clone());
        code
    }
}
```

### Lazy Generation

Only generate code when actually needed:

```reluxscript
// Bad: Generate immediately
fn visit_expression(node: &Expression) {
    let code = codegen::generate(node);  // May not be used
    if some_condition {
        use_code(code);
    }
}

// Good: Generate only when needed
fn visit_expression(node: &Expression) {
    if some_condition {
        let code = codegen::generate(node);  // Only generated if needed
        use_code(code);
    }
}
```

---

## Limitations

1. **No AST Modification from Strings**: The codegen module only converts AST → Code. To modify code, you must manipulate the AST directly.

2. **Formatting Differences**: Minor formatting differences may exist between Babel and SWC output (whitespace, parentheses placement).

3. **Comments Preservation**: Comment handling may differ between targets. By default, comments are preserved in Babel but may be lost in SWC without explicit configuration.

4. **Source Maps**: The basic `generate()` function does not produce source maps. For source map generation, use platform-specific APIs directly.

---

## See Also

- [ReluxScript Specification](./reluxscript-specification.md) - Main language spec
- [Parser Module](./reluxscript-parser-module.md) - The inverse operation (Code → AST)
- [@babel/generator Documentation](https://babeljs.io/docs/babel-generator)
- [swc_ecma_codegen Documentation](https://docs.rs/swc_ecma_codegen)

---

## Future Enhancements

### Planned Features:
- `codegen::generate_minified()` - Shorthand for minified output
- `codegen::generate_pretty()` - Shorthand for pretty-printed output
- `codegen::with_source_map()` - Generate with source maps
- `codegen::preserve_comments()` - Ensure comments are preserved
- Template literal support for code injection
