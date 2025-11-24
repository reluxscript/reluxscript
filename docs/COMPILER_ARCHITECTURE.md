# ReluxScript Compiler - Developer Documentation

## Table of Contents
1. [Overall Architecture](#overall-architecture)
2. [Lexer](#lexer)
3. [Parser](#parser)
4. [Semantic Analysis](#semantic-analysis)
5. [Code Generation](#code-generation)
6. [Key Design Patterns](#key-design-patterns)
7. [Common Development Tasks](#common-development-tasks)
8. [Important Files Reference](#important-files-reference)
9. [Example Compilation Flow](#example-compilation-flow)

---

## Overall Architecture

### High-Level Pipeline

ReluxScript follows a classic multi-pass compiler architecture:

```
Source Code (.lux)
    ↓
┌─────────────────────┐
│  Lexer              │  → Tokens
│  (lexer/lexer.rs)   │
└─────────────────────┘
    ↓
┌─────────────────────┐
│  Parser             │  → AST (Abstract Syntax Tree)
│  (parser/parser.rs) │
└─────────────────────┘
    ↓
┌─────────────────────┐
│  Semantic Analysis  │  → Type-checked AST
│  - Resolver         │
│  - Type Checker     │
│  - Ownership Check  │
└─────────────────────┘
    ↓
┌─────────────────────┐
│  AST Lowering       │  → Normalized AST
│  (hoist_unwraps.rs) │
└─────────────────────┘
    ↓
┌─────────────────────┐
│  Code Generation    │  → Babel JS / SWC Rust
│  - babel.rs         │
│  - swc.rs           │
└─────────────────────┘
```

### Entry Point (main.rs)

**Location:** `source/src/main.rs`

The CLI provides several commands:

- **`lex`** - Tokenize and display tokens (debugging)
- **`parse`** - Parse and display AST (debugging)
- **`check`** - Run semantic analysis without codegen
- **`build`** - Full compilation to target platform(s)
- **`fix`** - Autofix common syntax issues

**Build Command Flow (lines 189-367):**

```rust
// 1. Read source file
let source = fs::read_to_string(&file)?;

// 2. Tokenize
let mut lexer = Lexer::new(&source);
let tokens = lexer.tokenize();

// 3. Optional autofix (rewrite problematic tokens)
if autofix {
    let rewriter = TokenRewriter::new(tokens);
    let (fixed_tokens, _) = rewriter.rewrite();
    tokens = fixed_tokens;
}

// 4. Parse to AST
let mut parser = Parser::new_with_source(tokens, source);
let mut program = parser.parse()?;

// 5. Semantic analysis
let result = analyze_with_base_dir(&program, base_dir);
if !result.errors.is_empty() {
    // Report errors and exit
}

// 6. AST lowering (normalize constructs)
lower(&mut program);

// 7. Code generation
let generated = generate(&program, target_enum);

// 8. Write output files and validate
// - Babel: validate with `node --check`
// - SWC: validate with `cargo check`
```

---

## Lexer

**Location:** `source/src/lexer/`

### Token Definition (token.rs)

**Key Data Structures:**

```rust
// Span tracks source location for error reporting
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

pub enum TokenKind {
    // Keywords
    Fn, Let, Const, Mut, If, Else, For, In, While, Loop,
    Return, Break, Continue, True, False, Null,
    Plugin, Writer, Struct, Enum, Impl, Use, Pub, As,
    Self_, SelfType, Match, Matches, Traverse, Using, Capturing,

    // Types
    Str, Bool, I32, U32, F64, Vec, Option, Result, HashMap, HashSet,

    // AST Node Types (built-in for pattern matching)
    Program, FunctionDeclaration, Identifier, CallExpression,
    BinaryExpression, MemberExpression, JSXElement, /* ... */

    // Identifiers and Literals
    Ident(String),
    StringLit(String),
    IntLit(i64),
    FloatLit(f64),

    // Operators
    Plus, Minus, Star, Slash, Percent,
    Eq, EqEq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or, Not,
    /* ... */

    // Delimiters
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Comma, Dot, Colon, Semicolon, Arrow, DDArrow, ColonColon,
    Question, QuestionDot,

    // Special
    Comment(String),
    DocComment(String),
    Newline,
    Eof,
    Error(String),
}
```

### Lexing Process (lexer.rs)

**Core Algorithm:**

The lexer maintains state for context-aware tokenization:

```rust
pub struct Lexer<'a> {
    source: &'a str,
    chars: Peekable<CharIndices<'a>>,
    current_pos: usize,
    line: usize,
    column: usize,
    line_start: usize,

    // Depth tracking for newline handling
    paren_depth: usize,
    bracket_depth: usize,
    brace_depth: usize,
}
```

**Key Features:**

1. **Context-Aware Newlines (lines 238-249):**
   ```rust
   '\n' => {
       self.line += 1;
       self.line_start = self.current_pos;
       self.column = 1;

       // Skip newlines inside delimiters
       if self.paren_depth > 0 || self.bracket_depth > 0 || self.brace_depth > 0 {
           return self.next_token(); // Skip and get next
       }

       TokenKind::Newline
   }
   ```

2. **Keyword vs Identifier Distinction (lines 515-607):**
   ```rust
   fn read_identifier(&mut self, first: char) -> TokenKind {
       let mut ident = String::new();
       ident.push(first);

       while let Some(c) = self.peek_char() {
           if c.is_alphanumeric() || c == '_' {
               ident.push(c);
               self.advance();
           } else {
               break;
           }
       }

       // Special handling for matches! macro
       if ident == "matches" && self.peek_char() == Some('!') {
           self.advance();
           return TokenKind::Matches;
       }

       // Keyword lookup
       match ident.as_str() {
           "fn" => TokenKind::Fn,
           "let" => TokenKind::Let,
           // ... all keywords ...

           // AST type keywords
           "Program" => TokenKind::Program,
           "Identifier" => TokenKind::Identifier,
           // ...

           // Regular identifier
           _ => TokenKind::Ident(ident),
       }
   }
   ```

3. **String Literals with Escape Sequences (lines 327-351):**
   - Handles: `\n`, `\t`, `\r`, `\\`, `\"`
   - Reports errors for invalid escapes

4. **Number Parsing (lines 416-513):**
   - Supports hex literals: `0xDEADBEEF`
   - Binary literals: `0b1010`
   - Float detection: `3.14`
   - Underscores in numbers: `1_000_000`
   - Method call disambiguation: `42.to_string()` vs `3.14`

---

## Parser

**Location:** `source/src/parser/`

### AST Structure (ast.rs)

**Top-Level Declarations:**

```rust
pub struct Program {
    pub uses: Vec<UseStmt>,     // use statements
    pub decl: TopLevelDecl,     // plugin/writer/module/interface
    pub span: Span,
}

pub enum TopLevelDecl {
    Plugin(PluginDecl),         // plugin Name { ... }
    Writer(WriterDecl),         // writer Name { ... }
    Interface(InterfaceDecl),   // TypeScript interface
    Module(ModuleDecl),         // Standalone module (no plugin/writer)
}

pub struct PluginDecl {
    pub name: String,
    pub body: Vec<PluginItem>,
    pub span: Span,
}

pub enum PluginItem {
    Struct(StructDecl),
    Enum(EnumDecl),
    Function(FnDecl),
    Impl(ImplBlock),
    PreHook(FnDecl),   // fn pre() - runs before visitors
    ExitHook(FnDecl),  // fn exit() - runs after all visitors
}
```

**Type System:**

```rust
pub enum Type {
    Primitive(String),           // Str, i32, f64, bool, Number
    Reference { mutable: bool, inner: Box<Type> },  // &T, &mut T
    Container { name: String, type_args: Vec<Type> }, // Vec<T>, Option<T>
    Named(String),               // User-defined or AST node type
    Array { element: Box<Type> }, // [T]
    Tuple(Vec<Type>),            // (T1, T2)
    Optional(Box<Type>),         // T?
    Unit,                        // ()
    FnTrait { params: Vec<Type>, return_type: Box<Type> }, // Fn(T) -> R
}
```

### Recursive Descent Parsing (parser.rs)

**Core Parser Structure:**

```rust
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    source: String,  // For extracting verbatim blocks
}
```

**Main Entry Point (lines 41-66):**

```rust
pub fn parse(&mut self) -> ParseResult<Program> {
    self.skip_newlines();
    let start_span = self.current_span();

    // Parse use statements
    let mut uses = Vec::new();
    while self.check(TokenKind::Use) {
        uses.push(self.parse_use_stmt()?);
        self.skip_newlines();
    }

    // Parse top-level declaration
    let decl = if self.check(TokenKind::Plugin) {
        TopLevelDecl::Plugin(self.parse_plugin()?)
    } else if self.check(TokenKind::Writer) {
        TopLevelDecl::Writer(self.parse_writer()?)
    } else {
        // No plugin/writer keyword - standalone module
        TopLevelDecl::Module(self.parse_module()?)
    };

    Ok(Program { uses, decl, span: start_span })
}
```

**Type Parsing (lines 569-661):**

Key features:
- Reference types: `&T`, `&mut T`
- Generic containers: `Vec<T>`, `Option<T>`, `HashMap<K, V>`
- Tuple types: `(T1, T2, T3)`
- Function trait types: `Fn(T1, T2) -> R`
- Distinguishes between primitives and named types

```rust
fn parse_type(&mut self) -> ParseResult<Type> {
    // ... (see lines 620-628 for primitive detection)

    let name = self.expect_type_name()?;

    // Check if it's a primitive
    match name.as_str() {
        "Str" | "i32" | "u32" | "f64" | "bool" | "Number" => {
            Ok(Type::Primitive(name))
        }
        _ => Ok(Type::Named(name)),
    }
}
```

---

## Semantic Analysis

**Location:** `source/src/semantic/`

### Overview (mod.rs)

Three passes run sequentially:

```rust
pub fn analyze_with_base_dir(program: &Program, base_dir: PathBuf) -> SemanticResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Pass 1: Name resolution
    let mut resolver = Resolver::with_base_dir(base_dir);
    if let Err(e) = resolver.resolve(program) {
        errors.extend(e);
    }

    // Pass 2: Type checking
    let mut type_checker = TypeChecker::new(resolver.get_env());
    if let Err(e) = type_checker.check(program) {
        errors.extend(e);
    }

    // Pass 3: Ownership checking
    let mut ownership_checker = OwnershipChecker::new();
    let (ownership_errors, ownership_warnings) = ownership_checker.check(program);
    errors.extend(ownership_errors);
    warnings.extend(ownership_warnings);

    SemanticResult { errors, warnings, type_env: type_checker.into_env() }
}
```

### Type System (types.rs)

**TypeInfo Enum (lines 6-67):**

```rust
pub enum TypeInfo {
    // Primitives
    Str, I32, U32, F64, Bool, Unit, Null,

    // Reference
    Ref { mutable: bool, inner: Box<TypeInfo> },

    // Containers
    Vec(Box<TypeInfo>),
    Option(Box<TypeInfo>),
    Result(Box<TypeInfo>, Box<TypeInfo>),
    HashMap(Box<TypeInfo>, Box<TypeInfo>),
    HashSet(Box<TypeInfo>),

    // Compound
    Tuple(Vec<TypeInfo>),
    Function { params: Vec<TypeInfo>, ret: Box<TypeInfo> },

    // User-defined
    Struct { name: String, fields: HashMap<String, TypeInfo> },
    Enum { name: String, variants: HashMap<String, Option<Vec<TypeInfo>>> },

    // AST node type (from Unified AST)
    AstNode(String),

    // Module type (for imports)
    Module { name: String },

    // Type variable (for inference)
    Var(usize),

    // Special
    Unknown,  // Error recovery
    Never,    // Non-returning functions
}
```

**Type Compatibility (lines 97-179):**

The `is_assignable_to` method implements **nominal typing** for structs and enums:

```rust
impl TypeInfo {
    pub fn is_assignable_to(&self, target: &TypeInfo) -> bool {
        match (self, target) {
            // Nominal typing for structs (compare names, not fields)
            (TypeInfo::Struct { name: n1, .. }, TypeInfo::Struct { name: n2, .. }) => n1 == n2,

            // Same types always assignable
            (a, b) if a == b => true,

            // Null assignable to Option
            (TypeInfo::Null, TypeInfo::Option(_)) => true,

            // Numeric coercion (i32 → f64)
            (TypeInfo::I32, TypeInfo::F64) => true,

            // Reference compatibility (mut ref → immut ref)
            (TypeInfo::Ref { mutable: m1, inner: i1 },
             TypeInfo::Ref { mutable: m2, inner: i2 }) => {
                (*m1 || !*m2) && i1.is_assignable_to(i2)
            }

            // Unknown matches anything (error recovery)
            (TypeInfo::Unknown, _) | (_, TypeInfo::Unknown) => true,

            _ => false,
        }
    }
}
```

**AST Type → TypeInfo Conversion (lines 382-456):**

```rust
pub fn ast_type_to_type_info(ty: &crate::parser::Type) -> TypeInfo {
    match ty {
        Type::Primitive(name) => match name.as_str() {
            "Str" => TypeInfo::Str,
            "i32" => TypeInfo::I32,
            "f64" | "Number" => TypeInfo::F64,  // Number is alias for f64
            "bool" | "Bool" => TypeInfo::Bool,
            "()" => TypeInfo::Unit,
            _ => TypeInfo::Unknown,
        },
        Type::Named(name) => match name.as_str() {
            "f64" | "Number" => TypeInfo::F64,  // Also handle Named("Number")
            "bool" | "Bool" => TypeInfo::Bool,
            "Str" | "String" => TypeInfo::Str,
            "i32" => TypeInfo::I32,
            "u32" => TypeInfo::U32,
            _ => TypeInfo::AstNode(name.clone()),
        },
        // ... other cases
    }
}
```

### Type Checker (type_checker.rs)

**Bidirectional Type Inference:**

The type checker uses **bidirectional type inference** where type information flows both:
- **Bottom-up:** Infer types from expressions
- **Top-down:** Expected types from context guide inference

**Expression Type Inference with Expected Type Hint:**

```rust
fn infer_expr_with_expected(&mut self, expr: &Expr, expected: Option<&TypeInfo>) -> TypeInfo {
    match expr {
        Expr::VecInit(vec_init) => {
            // Use expected type if available (TOP-DOWN)
            let elem_type = if let Some(TypeInfo::Vec(expected_elem)) = expected {
                (**expected_elem).clone()
            } else if let Some(first_elem) = vec_init.elements.first() {
                // Infer from first element (BOTTOM-UP)
                self.infer_expr(first_elem)
            } else {
                TypeInfo::Unknown
            };

            TypeInfo::Vec(Box::new(elem_type))
        }
        // ... more cases
    }
}
```

### Ownership Checker (ownership.rs)

**Clone-to-Own Principle:**

ReluxScript enforces **explicit ownership semantics**:

> **Extracting a value from a structure requires `.clone()`**

**Key Checks:**

1. **Value Extraction:** Member/index access that extracts values must use `.clone()`
2. **Property Mutation:** Direct mutation of properties not allowed (use `*node = new_value`)
3. **Reference Context:** Some contexts are allowed without clone (e.g., method calls)

---

## Code Generation

**Location:** `source/src/codegen/`

### Babel Generator (babel.rs)

**Overview:**

Generates JavaScript code that implements Babel plugin visitor pattern.

**Key Distinction: Path vs Node**

Babel has two types of values:
- **Path:** Wraps a node with metadata (e.g., `path.node`, `path.parent`)
- **Node:** Raw AST node (e.g., `Identifier`, `CallExpression`)

**Plugin Structure:**

```javascript
module.exports = function({ types: t }) {
  // Helper structs as classes
  class State {
    constructor() {
      this.count = 0;
    }
  }

  // Plugin state
  let state = {};

  return {
    visitor: {
      Identifier(path, state) {
        let node = path.node;

        if (node.name === "foo") {
          path.replaceWith(t.identifier("bar"));
        }
      }
    }
  };
};
```

### SWC Generator (swc.rs)

**Overview:**

Generates Rust code that implements SWC's `VisitMut` trait.

**Plugin Structure:**

```rust
use swc_common::{Span, DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub struct SimplePlugin {
    // Plugin state
}

impl SimplePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl VisitMut for SimplePlugin {
    fn visit_mut_ident(&mut self, n: &mut Ident) {
        if n.sym.as_ref() == "foo" {
            *n = Ident::new("bar".into(), DUMMY_SP, SyntaxContext::empty());
        }

        n.visit_mut_children_with(self);
    }
}
```

---

## Key Design Patterns

### 1. Bidirectional Type Inference

Type information flows both directions:

**Top-Down (Expected Type):**
```rust
// Type annotation provides expected type
let names: Vec<Str> = vec![];
//                    ^^^^^^ Expected: Vec<Str>
```

**Bottom-Up (Inferred Type):**
```rust
// Infer from literal
let x = 42;  // x: i32
```

### 2. Nominal vs Structural Typing

**Nominal Typing (Structs and Enums):**

Types are compared by name, not structure:

```rust
struct Point { x: i32, y: i32 }
struct Coords { x: i32, y: i32 }

let p: Point = Point { x: 0, y: 0 };
let c: Coords = p;  // ERROR: Point != Coords (different names)
```

### 3. Plugin vs Writer Distinction

**Plugin:** Modifies AST (visitor pattern)

```rust
plugin MyPlugin {
    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        // Mutate node in-place
        *node = Identifier { name: "new_name" };
    }
}
```

**Writer:** Generates code (accumulator pattern)

```rust
writer MyWriter {
    fn visit_identifier(node: &Identifier, ctx: &Context) {
        // Accumulate output (read-only)
        self.output.write(node.name.clone());
    }
}
```

---

## Common Development Tasks

### Task 1: Adding a New Type

See full documentation for step-by-step instructions on:
1. Adding to TokenKind
2. Keyword recognition in lexer
3. Adding to AST Type enum
4. Parsing support
5. Adding to TypeInfo
6. Type conversion
7. Compatibility rules
8. Code generation

### Task 2: Adding Type Coercion Rules

**Location:** `semantic/types.rs`, in `is_assignable_to` method (lines 97-179)

```rust
impl TypeInfo {
    pub fn is_assignable_to(&self, target: &TypeInfo) -> bool {
        match (self, target) {
            // ... existing rules ...

            // Add new coercion rule
            (TypeInfo::I32, TypeInfo::F64) => true,

            // ... rest of rules ...
        }
    }
}
```

**Best Practices:**

1. **Asymmetric coercions:** Most coercions are one-way (e.g., i32 → f64 but not f64 → i32)
2. **Transitive closure:** Ensure coercions compose correctly
3. **Test edge cases:** Add test cases for new coercions

---

## Important Files Reference

### Core Compiler Pipeline

| File | Purpose |
|------|---------|
| `source/src/main.rs` | CLI entry point, orchestrates compilation |
| `source/src/lib.rs` | Public API exports |

### Lexer

| File | Purpose |
|------|---------|
| `source/src/lexer/token.rs` | Token definitions (TokenKind, Span) |
| `source/src/lexer/lexer.rs` | Lexer implementation, tokenization logic |

### Parser

| File | Purpose |
|------|---------|
| `source/src/parser/ast.rs` | Complete AST definition |
| `source/src/parser/parser.rs` | Recursive descent parser |

### Semantic Analysis

| File | Purpose |
|------|---------|
| `source/src/semantic/mod.rs` | Semantic analysis orchestration |
| `source/src/semantic/types.rs` | Type system (TypeInfo, TypeEnv) |
| `source/src/semantic/type_checker.rs` | Type checking pass |
| `source/src/semantic/resolver.rs` | Name resolution pass |
| `source/src/semantic/ownership.rs` | Ownership checking pass |

### Code Generation

| File | Purpose |
|------|---------|
| `source/src/codegen/mod.rs` | Codegen orchestration |
| `source/src/codegen/babel.rs` | Babel (JavaScript) generator |
| `source/src/codegen/swc.rs` | SWC (Rust) generator |

### Mapping Tables

| File | Purpose |
|------|---------|
| `source/src/mapping/nodes.rs` | AST node type mappings |
| `source/src/mapping/fields.rs` | Field name mappings |
| `source/src/mapping/helpers.rs` | Helper method mappings |

---

## Example Compilation Flow

**Input (simple.lux):**
```rust
plugin SimplePlugin {
    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        if node.name == "foo" {
            *node = Identifier { name: "bar" };
        }
    }
}
```

**Generated Babel Output:**
```javascript
module.exports = function({ types: t }) {
  return {
    visitor: {
      Identifier(path, state) {
        let node = path.node;

        if (node.name === "foo") {
          path.replaceWith(t.identifier("bar"));
        }
      }
    }
  };
};
```

**Generated SWC Output:**
```rust
use swc_common::{Span, DUMMY_SP, SyntaxContext};
use swc_ecma_ast::*;
use swc_ecma_visit::{VisitMut, VisitMutWith};

pub struct SimplePlugin {}

impl SimplePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

impl VisitMut for SimplePlugin {
    fn visit_mut_ident(&mut self, n: &mut Ident) {
        if n.sym.as_ref() == "foo" {
            *n = Ident::new("bar".into(), DUMMY_SP, SyntaxContext::empty());
        }

        n.visit_mut_children_with(self);
    }
}
```

---

## Summary

The ReluxScript compiler is a sophisticated multi-pass compiler that:

1. **Lexes** source code into tokens with context-aware handling
2. **Parses** tokens into a rich AST with full type annotations
3. **Analyzes** semantics through name resolution, type checking, and ownership checking
4. **Lowers** the AST to normalize complex constructs
5. **Generates** both JavaScript (Babel) and Rust (SWC) code from a single source

Key architectural decisions:

- **Unified AST:** Single intermediate representation for both targets
- **Bidirectional typing:** Type information flows both directions
- **Nominal typing:** Structs/enums compared by name
- **Explicit ownership:** .clone() required for value extraction
- **Mapping tables:** Centralized AST type/field mappings

This architecture enables ReluxScript to compile to both garbage-collected (JavaScript) and borrow-checked (Rust) runtimes while maintaining type safety and ownership semantics.
