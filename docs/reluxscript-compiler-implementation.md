# ReluxScript Compiler Implementation Plan

**Approach:** Hand-Written Recursive Descent (HWRD)
**Language:** Rust
**Timeline:** ~2 weeks
**Dependencies:** Minimal (ariadne for errors)

---

## 1. Project Structure

```
reluxscript/
├── Cargo.toml
├── src/
│   ├── main.rs              # CLI entry point
│   ├── lib.rs               # Library exports
│   │
│   ├── lexer/
│   │   ├── mod.rs
│   │   ├── token.rs         # Token definitions
│   │   ├── lexer.rs         # Lexer implementation
│   │   └── span.rs          # Source location tracking
│   │
│   ├── parser/
│   │   ├── mod.rs
│   │   ├── ast.rs           # AST node definitions
│   │   ├── parser.rs        # Recursive descent parser
│   │   └── precedence.rs    # Operator precedence
│   │
│   ├── semantic/
│   │   ├── mod.rs
│   │   ├── resolver.rs      # Name resolution
│   │   ├── typeck.rs        # Type checking
│   │   └── ownership.rs     # Clone/borrow validation
│   │
│   ├── codegen/
│   │   ├── mod.rs
│   │   ├── babel.rs         # JavaScript/Babel output
│   │   ├── swc.rs           # Rust/SWC output
│   │   └── shared.rs        # Common codegen utilities
│   │
│   ├── error/
│   │   ├── mod.rs
│   │   ├── diagnostic.rs    # Error types
│   │   └── report.rs        # Pretty printing with ariadne
│   │
│   └── prelude/
│       ├── mod.rs
│       ├── types.rs         # Built-in types (Str, Vec, etc.)
│       └── functions.rs     # Standard library functions
│
└── tests/
    ├── lexer_tests.rs
    ├── parser_tests.rs
    ├── codegen_tests.rs
    └── fixtures/            # Test .rs files
```

---

## 2. Phase 1: Lexer (Days 1-2)

### 2.1 Token Definitions

```rust
// src/lexer/token.rs

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Plugin,
    Writer,
    Fn,
    Let,
    Const,
    Mut,
    If,
    Else,
    Match,
    Return,
    For,
    In,
    While,
    Loop,
    Break,
    Continue,
    Struct,
    Enum,
    Impl,
    Pub,
    Self_,
    SelfType,
    True,
    False,
    None,
    Some,

    // Identifiers and Literals
    Ident(String),
    String(String),
    Int(i64),
    Float(f64),

    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Eq,             // =
    EqEq,           // ==
    NotEq,          // !=
    Lt,             // <
    Gt,             // >
    LtEq,           // <=
    GtEq,           // >=
    And,            // &&
    Or,             // ||
    Not,            // !
    Ampersand,      // &
    Pipe,           // |
    PlusEq,         // +=
    MinusEq,        // -=
    StarEq,         // *=
    SlashEq,        // /=

    // Delimiters
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Comma,          // ,
    Semicolon,      // ;
    Colon,          // :
    ColonColon,     // ::
    Arrow,          // ->
    FatArrow,       // =>
    Dot,            // .
    DotDot,         // ..
    DotDotDot,      // ...
    Question,       // ?

    // Special
    DocComment(String),
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}
```

### 2.2 Lexer Implementation

```rust
// src/lexer/lexer.rs

pub struct Lexer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::CharIndices<'a>>,
    pos: usize,
    line: usize,
    column: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self { ... }

    pub fn tokenize(&mut self) -> Result<Vec<Token>, LexError> {
        let mut tokens = vec![];
        loop {
            let token = self.next_token()?;
            let is_eof = token.kind == TokenKind::Eof;
            tokens.push(token);
            if is_eof { break; }
        }
        Ok(tokens)
    }

    fn next_token(&mut self) -> Result<Token, LexError> { ... }

    // Lexing methods
    fn lex_ident_or_keyword(&mut self) -> Result<Token, LexError> { ... }
    fn lex_number(&mut self) -> Result<Token, LexError> { ... }
    fn lex_string(&mut self) -> Result<Token, LexError> { ... }
    fn lex_operator(&mut self) -> Result<Token, LexError> { ... }

    // Utilities
    fn skip_whitespace(&mut self) { ... }
    fn skip_line_comment(&mut self) { ... }
    fn skip_block_comment(&mut self) -> Result<(), LexError> { ... }
    fn peek(&mut self) -> Option<char> { ... }
    fn peek_next(&mut self) -> Option<char> { ... }
    fn advance(&mut self) -> Option<char> { ... }
    fn make_span(&self, start: usize) -> Span { ... }
}
```

### 2.3 Keyword Map

```rust
fn keyword_or_ident(s: &str) -> TokenKind {
    match s {
        "plugin" => TokenKind::Plugin,
        "writer" => TokenKind::Writer,
        "fn" => TokenKind::Fn,
        "let" => TokenKind::Let,
        "const" => TokenKind::Const,
        "mut" => TokenKind::Mut,
        "if" => TokenKind::If,
        "else" => TokenKind::Else,
        "match" => TokenKind::Match,
        "return" => TokenKind::Return,
        "for" => TokenKind::For,
        "in" => TokenKind::In,
        "while" => TokenKind::While,
        "loop" => TokenKind::Loop,
        "break" => TokenKind::Break,
        "continue" => TokenKind::Continue,
        "struct" => TokenKind::Struct,
        "enum" => TokenKind::Enum,
        "impl" => TokenKind::Impl,
        "pub" => TokenKind::Pub,
        "self" => TokenKind::Self_,
        "Self" => TokenKind::SelfType,
        "true" => TokenKind::True,
        "false" => TokenKind::False,
        "None" => TokenKind::None,
        "Some" => TokenKind::Some,
        "vec" => TokenKind::Ident("vec".into()), // vec! handled specially
        _ => TokenKind::Ident(s.to_string()),
    }
}
```

### 2.4 Deliverables

- [ ] Token enum with all ReluxScript tokens
- [ ] Span tracking for error reporting
- [ ] Lexer with all token recognition
- [ ] Line/column tracking
- [ ] Comment handling (line, block, doc)
- [ ] String escape sequences
- [ ] Number literals (int, float, hex, binary)
- [ ] Unit tests for each token type

---

## 3. Phase 2: Parser (Days 3-6)

### 3.1 AST Definitions

```rust
// src/parser/ast.rs

// Top-level
#[derive(Debug)]
pub enum Item {
    Plugin(Plugin),
    Writer(Writer),
    Struct(StructDef),
    Enum(EnumDef),
    Function(Function),
}

#[derive(Debug)]
pub struct Plugin {
    pub name: Ident,
    pub state: Option<StructDef>,
    pub methods: Vec<VisitorMethod>,
    pub span: Span,
}

#[derive(Debug)]
pub struct Writer {
    pub name: Ident,
    pub state: StructDef,
    pub init: Function,
    pub methods: Vec<VisitorMethod>,
    pub finish: Function,
    pub span: Span,
}

#[derive(Debug)]
pub struct VisitorMethod {
    pub name: Ident,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: Block,
    pub span: Span,
}

// Types
#[derive(Debug)]
pub enum Type {
    Named(Ident),
    Reference { mutable: bool, inner: Box<Type> },
    Generic { name: Ident, args: Vec<Type> },
    Tuple(Vec<Type>),
    Unit,
}

// Statements
#[derive(Debug)]
pub enum Stmt {
    Let {
        name: Pattern,
        ty: Option<Type>,
        value: Expr,
        mutable: bool,
        span: Span,
    },
    Expr {
        expr: Expr,
        semicolon: bool,
        span: Span,
    },
    Return {
        value: Option<Expr>,
        span: Span,
    },
    If {
        condition: Expr,
        then_block: Block,
        else_block: Option<Block>,
        span: Span,
    },
    Match {
        expr: Expr,
        arms: Vec<MatchArm>,
        span: Span,
    },
    For {
        pattern: Pattern,
        iter: Expr,
        body: Block,
        span: Span,
    },
    While {
        condition: Expr,
        body: Block,
        span: Span,
    },
    Break { span: Span },
    Continue { span: Span },
}

// Expressions
#[derive(Debug)]
pub enum Expr {
    // Literals
    Ident(Ident),
    Int(i64, Span),
    Float(f64, Span),
    String(String, Span),
    Bool(bool, Span),

    // Compound
    Array(Vec<Expr>, Span),
    Tuple(Vec<Expr>, Span),
    Struct {
        name: Ident,
        fields: Vec<(Ident, Expr)>,
        span: Span,
    },

    // Operations
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
        span: Span,
    },
    Unary {
        op: UnaryOp,
        expr: Box<Expr>,
        span: Span,
    },

    // Access
    Field {
        expr: Box<Expr>,
        field: Ident,
        span: Span,
    },
    Index {
        expr: Box<Expr>,
        index: Box<Expr>,
        span: Span,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
        span: Span,
    },
    MethodCall {
        receiver: Box<Expr>,
        method: Ident,
        args: Vec<Expr>,
        span: Span,
    },

    // Special
    If {
        condition: Box<Expr>,
        then_expr: Box<Expr>,
        else_expr: Box<Expr>,
        span: Span,
    },
    Match {
        expr: Box<Expr>,
        arms: Vec<MatchArm>,
        span: Span,
    },
    Block(Block),

    // Reference/Dereference
    Ref {
        mutable: bool,
        expr: Box<Expr>,
        span: Span,
    },
    Deref {
        expr: Box<Expr>,
        span: Span,
    },

    // Macros
    Matches {
        expr: Box<Expr>,
        pattern: Pattern,
        span: Span,
    },
    Vec {
        elements: Vec<Expr>,
        span: Span,
    },
    Format {
        template: String,
        args: Vec<Expr>,
        span: Span,
    },
}

#[derive(Debug)]
pub enum BinOp {
    Add, Sub, Mul, Div, Mod,
    Eq, NotEq, Lt, Gt, LtEq, GtEq,
    And, Or,
    Assign, AddAssign, SubAssign, MulAssign, DivAssign,
}

#[derive(Debug)]
pub enum UnaryOp {
    Neg, Not, Deref, Ref, RefMut,
}

#[derive(Debug)]
pub struct Ident {
    pub name: String,
    pub span: Span,
}
```

### 3.2 Parser Implementation

```rust
// src/parser/parser.rs

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
    errors: Vec<ParseError>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { ... }

    pub fn parse(&mut self) -> Result<Vec<Item>, Vec<ParseError>> {
        let mut items = vec![];

        while !self.is_eof() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(e) => {
                    self.errors.push(e);
                    self.synchronize(); // Error recovery
                }
            }
        }

        if self.errors.is_empty() {
            Ok(items)
        } else {
            Err(self.errors.clone())
        }
    }

    // Top-level parsing
    fn parse_item(&mut self) -> Result<Item, ParseError> { ... }
    fn parse_plugin(&mut self) -> Result<Plugin, ParseError> { ... }
    fn parse_writer(&mut self) -> Result<Writer, ParseError> { ... }
    fn parse_struct(&mut self) -> Result<StructDef, ParseError> { ... }
    fn parse_enum(&mut self) -> Result<EnumDef, ParseError> { ... }
    fn parse_function(&mut self) -> Result<Function, ParseError> { ... }

    // Statement parsing
    fn parse_block(&mut self) -> Result<Block, ParseError> { ... }
    fn parse_stmt(&mut self) -> Result<Stmt, ParseError> { ... }
    fn parse_let_stmt(&mut self) -> Result<Stmt, ParseError> { ... }
    fn parse_return_stmt(&mut self) -> Result<Stmt, ParseError> { ... }
    fn parse_if_stmt(&mut self) -> Result<Stmt, ParseError> { ... }
    fn parse_match_stmt(&mut self) -> Result<Stmt, ParseError> { ... }
    fn parse_for_stmt(&mut self) -> Result<Stmt, ParseError> { ... }
    fn parse_while_stmt(&mut self) -> Result<Stmt, ParseError> { ... }

    // Expression parsing (Pratt parser for precedence)
    fn parse_expr(&mut self) -> Result<Expr, ParseError> { ... }
    fn parse_expr_with_precedence(&mut self, min_prec: u8) -> Result<Expr, ParseError> { ... }
    fn parse_prefix(&mut self) -> Result<Expr, ParseError> { ... }
    fn parse_infix(&mut self, left: Expr) -> Result<Expr, ParseError> { ... }
    fn parse_postfix(&mut self, expr: Expr) -> Result<Expr, ParseError> { ... }

    // Type parsing
    fn parse_type(&mut self) -> Result<Type, ParseError> { ... }
    fn parse_generic_args(&mut self) -> Result<Vec<Type>, ParseError> { ... }

    // Pattern parsing
    fn parse_pattern(&mut self) -> Result<Pattern, ParseError> { ... }
    fn parse_struct_pattern(&mut self) -> Result<Pattern, ParseError> { ... }

    // Utilities
    fn expect(&mut self, kind: TokenKind) -> Result<Token, ParseError> { ... }
    fn expect_ident(&mut self) -> Result<Ident, ParseError> { ... }
    fn check(&self, kind: &TokenKind) -> bool { ... }
    fn advance(&mut self) -> Token { ... }
    fn peek(&self) -> &Token { ... }
    fn peek_kind(&self) -> &TokenKind { ... }
    fn is_eof(&self) -> bool { ... }
    fn synchronize(&mut self) { ... } // Error recovery
}
```

### 3.3 Operator Precedence

```rust
// src/parser/precedence.rs

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Precedence {
    None = 0,
    Assignment = 1,    // =, +=, -=
    Or = 2,            // ||
    And = 3,           // &&
    Equality = 4,      // ==, !=
    Comparison = 5,    // <, >, <=, >=
    Term = 6,          // +, -
    Factor = 7,        // *, /, %
    Unary = 8,         // !, -, &, *
    Call = 9,          // (), [], .
    Primary = 10,
}

impl TokenKind {
    pub fn precedence(&self) -> Precedence {
        match self {
            TokenKind::Eq | TokenKind::PlusEq | TokenKind::MinusEq => Precedence::Assignment,
            TokenKind::Or => Precedence::Or,
            TokenKind::And => Precedence::And,
            TokenKind::EqEq | TokenKind::NotEq => Precedence::Equality,
            TokenKind::Lt | TokenKind::Gt | TokenKind::LtEq | TokenKind::GtEq => Precedence::Comparison,
            TokenKind::Plus | TokenKind::Minus => Precedence::Term,
            TokenKind::Star | TokenKind::Slash | TokenKind::Percent => Precedence::Factor,
            TokenKind::LParen | TokenKind::LBracket | TokenKind::Dot => Precedence::Call,
            _ => Precedence::None,
        }
    }

    pub fn is_right_associative(&self) -> bool {
        matches!(self, TokenKind::Eq | TokenKind::PlusEq | TokenKind::MinusEq)
    }
}
```

### 3.4 Deliverables

- [ ] Complete AST definitions
- [ ] Plugin/Writer parsing
- [ ] Struct/Enum parsing
- [ ] Function/Method parsing
- [ ] Statement parsing (all types)
- [ ] Expression parsing with Pratt precedence
- [ ] Type parsing with generics
- [ ] Pattern parsing
- [ ] Error recovery (synchronization)
- [ ] Unit tests for each construct

---

## 4. Phase 3: Semantic Analysis (Days 7-9)

### 4.1 Name Resolution

```rust
// src/semantic/resolver.rs

pub struct Resolver {
    scopes: Vec<Scope>,
    errors: Vec<SemanticError>,
}

#[derive(Debug)]
pub struct Scope {
    bindings: HashMap<String, Binding>,
    kind: ScopeKind,
}

#[derive(Debug)]
pub enum ScopeKind {
    Plugin,
    Function,
    Block,
    Loop,
}

#[derive(Debug)]
pub struct Binding {
    pub name: String,
    pub ty: Type,
    pub mutable: bool,
    pub defined_at: Span,
    pub used: bool,
}

impl Resolver {
    pub fn resolve(&mut self, items: &mut Vec<Item>) -> Result<(), Vec<SemanticError>> {
        for item in items {
            self.resolve_item(item)?;
        }

        self.check_unused_bindings();

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn resolve_item(&mut self, item: &mut Item) -> Result<(), SemanticError> { ... }
    fn resolve_plugin(&mut self, plugin: &mut Plugin) -> Result<(), SemanticError> { ... }
    fn resolve_function(&mut self, func: &mut Function) -> Result<(), SemanticError> { ... }
    fn resolve_block(&mut self, block: &mut Block) -> Result<(), SemanticError> { ... }
    fn resolve_stmt(&mut self, stmt: &mut Stmt) -> Result<(), SemanticError> { ... }
    fn resolve_expr(&mut self, expr: &mut Expr) -> Result<(), SemanticError> { ... }

    fn push_scope(&mut self, kind: ScopeKind) { ... }
    fn pop_scope(&mut self) { ... }
    fn declare(&mut self, name: &str, ty: Type, mutable: bool, span: Span) -> Result<(), SemanticError> { ... }
    fn lookup(&self, name: &str) -> Option<&Binding> { ... }
    fn mark_used(&mut self, name: &str) { ... }
}
```

### 4.2 Type Checking

```rust
// src/semantic/typeck.rs

pub struct TypeChecker {
    errors: Vec<TypeError>,
}

impl TypeChecker {
    pub fn check(&mut self, items: &Vec<Item>) -> Result<(), Vec<TypeError>> {
        for item in items {
            self.check_item(item)?;
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    fn check_function(&mut self, func: &Function) -> Result<Type, TypeError> { ... }
    fn check_block(&mut self, block: &Block) -> Result<Type, TypeError> { ... }
    fn check_stmt(&mut self, stmt: &Stmt) -> Result<(), TypeError> { ... }
    fn check_expr(&mut self, expr: &Expr) -> Result<Type, TypeError> { ... }

    fn unify(&self, expected: &Type, found: &Type, span: Span) -> Result<(), TypeError> { ... }
    fn infer_binary_op(&self, op: BinOp, left: &Type, right: &Type) -> Result<Type, TypeError> { ... }
}
```

### 4.3 Ownership Validation

```rust
// src/semantic/ownership.rs

pub struct OwnershipChecker {
    errors: Vec<OwnershipError>,
}

impl OwnershipChecker {
    pub fn check(&mut self, items: &Vec<Item>) -> Result<(), Vec<OwnershipError>> {
        for item in items {
            self.check_item(item)?;
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(self.errors.clone())
        }
    }

    // Ensure .clone() is called when extracting from references
    fn check_clone_required(&mut self, expr: &Expr, context: &str) -> Result<(), OwnershipError> { ... }

    // Ensure no direct property mutation
    fn check_no_field_mutation(&mut self, stmt: &Stmt) -> Result<(), OwnershipError> { ... }

    // Check statement lowering (*node = ...) is valid
    fn check_statement_lowering(&mut self, stmt: &Stmt) -> Result<(), OwnershipError> { ... }
}
```

### 4.4 Deliverables

- [ ] Scope management
- [ ] Name resolution
- [ ] Duplicate binding detection
- [ ] Undefined variable detection
- [ ] Type inference
- [ ] Type checking
- [ ] Clone requirement validation
- [ ] Property mutation prevention
- [ ] Statement lowering validation
- [ ] Unused variable warnings

---

## 5. Phase 4: Code Generation (Days 10-13)

### 5.1 Babel/JavaScript Output

```rust
// src/codegen/babel.rs

pub struct BabelCodegen {
    output: String,
    indent: usize,
}

impl BabelCodegen {
    pub fn generate(&mut self, items: &Vec<Item>) -> String {
        for item in items {
            self.gen_item(item);
        }
        self.output.clone()
    }

    fn gen_plugin(&mut self, plugin: &Plugin) {
        self.emit("module.exports = function({ types: t }) {");
        self.newline();
        self.indent();

        self.emit("return {");
        self.newline();
        self.indent();

        self.emit("visitor: {");
        self.newline();
        self.indent();

        for method in &plugin.methods {
            self.gen_visitor_method(method);
        }

        self.dedent();
        self.emit("}");
        self.newline();

        self.dedent();
        self.emit("};");
        self.newline();

        self.dedent();
        self.emit("};");
    }

    fn gen_visitor_method(&mut self, method: &VisitorMethod) {
        // Convert visit_call_expression -> CallExpression
        let visitor_name = to_pascal_case(&method.name.name.replace("visit_", ""));

        self.emit(&format!("{}(path) {{", visitor_name));
        self.newline();
        self.indent();

        // Emit: const node = path.node;
        self.emit("const node = path.node;");
        self.newline();

        // Generate body
        self.gen_block(&method.body);

        self.dedent();
        self.emit("},");
        self.newline();
    }

    fn gen_stmt(&mut self, stmt: &Stmt) { ... }
    fn gen_expr(&mut self, expr: &Expr) -> String { ... }

    // Statement lowering: *node = ... -> path.replaceWith(...)
    fn gen_assignment_to_node(&mut self, value: &Expr) {
        let value_str = self.gen_expr(value);
        self.emit(&format!("path.replaceWith({});", value_str));
    }

    // Type mappings
    fn map_type_check(&self, method: &str) -> String {
        match method {
            "is_identifier" => "t.isIdentifier".into(),
            "is_call_expression" => "t.isCallExpression".into(),
            // ... etc
        }
    }

    fn map_node_constructor(&self, name: &str) -> String {
        match name {
            "Identifier" => "t.identifier".into(),
            "CallExpression" => "t.callExpression".into(),
            // ... etc
        }
    }

    // Utilities
    fn emit(&mut self, s: &str) { ... }
    fn newline(&mut self) { ... }
    fn indent(&mut self) { ... }
    fn dedent(&mut self) { ... }
}
```

### 5.2 SWC/Rust Output

```rust
// src/codegen/swc.rs

pub struct SwcCodegen {
    output: String,
    indent: usize,
}

impl SwcCodegen {
    pub fn generate(&mut self, items: &Vec<Item>) -> String {
        // Emit imports
        self.emit_imports();

        for item in items {
            self.gen_item(item);
        }

        // Emit plugin boilerplate
        self.emit_plugin_transform();

        self.output.clone()
    }

    fn emit_imports(&mut self) {
        self.emit("use swc_core::ecma::{");
        self.newline();
        self.emit("    ast::*,");
        self.newline();
        self.emit("    visit::{VisitMut, VisitMutWith},");
        self.newline();
        self.emit("};");
        self.newline();
        self.emit("use swc_core::plugin::{plugin_transform, proxies::TransformPluginProgramMetadata};");
        self.newline();
        self.newline();
    }

    fn gen_plugin(&mut self, plugin: &Plugin) {
        // Struct definition
        self.emit(&format!("pub struct {};", plugin.name.name));
        self.newline();
        self.newline();

        // VisitMut impl
        self.emit(&format!("impl VisitMut for {} {{", plugin.name.name));
        self.newline();
        self.indent();

        for method in &plugin.methods {
            self.gen_visitor_method(method);
        }

        self.dedent();
        self.emit("}");
        self.newline();
    }

    fn gen_visitor_method(&mut self, method: &VisitorMethod) {
        // Convert visit_call_expression -> visit_mut_call_expr
        let swc_method = self.map_visitor_method(&method.name.name);
        let param_type = self.map_node_type(&method.name.name);

        self.emit(&format!("fn {}(&mut self, n: &mut {}) {{", swc_method, param_type));
        self.newline();
        self.indent();

        // Generate body
        self.gen_block(&method.body);

        self.dedent();
        self.emit("}");
        self.newline();
    }

    fn emit_plugin_transform(&mut self) {
        self.newline();
        self.emit("#[plugin_transform]");
        self.newline();
        self.emit("pub fn process_transform(program: Program, _metadata: TransformPluginProgramMetadata) -> Program {");
        self.newline();
        self.emit("    program.fold_with(&mut PluginName)");
        self.newline();
        self.emit("}");
    }

    // Type mappings
    fn map_visitor_method(&self, name: &str) -> String {
        match name {
            "visit_call_expression" => "visit_mut_call_expr".into(),
            "visit_identifier" => "visit_mut_ident".into(),
            // ... etc
        }
    }

    fn map_node_type(&self, visitor: &str) -> String {
        match visitor {
            "visit_call_expression" => "CallExpr".into(),
            "visit_identifier" => "Ident".into(),
            // ... etc
        }
    }

    fn map_type_check(&self, method: &str) -> String {
        match method {
            "is_identifier" => "matches!(node, Expr::Ident(_))".into(),
            "is_call_expression" => "matches!(node, Expr::Call(_))".into(),
            // ... etc
        }
    }
}
```

### 5.3 Shared Utilities

```rust
// src/codegen/shared.rs

/// Convert snake_case to PascalCase
pub fn to_pascal_case(s: &str) -> String { ... }

/// Convert PascalCase to snake_case
pub fn to_snake_case(s: &str) -> String { ... }

/// Escape string for output
pub fn escape_string(s: &str) -> String { ... }

/// U-AST node mapping
pub struct NodeMapping {
    pub reluxscript: &'static str,
    pub babel: &'static str,
    pub swc: &'static str,
}

pub const NODE_MAPPINGS: &[NodeMapping] = &[
    NodeMapping { reluxscript: "Identifier", babel: "t.identifier", swc: "Ident" },
    NodeMapping { reluxscript: "CallExpression", babel: "t.callExpression", swc: "CallExpr" },
    // ... complete mapping table
];
```

### 5.4 Deliverables

- [ ] Babel codegen with visitor wrapping
- [ ] SWC codegen with VisitMut impl
- [ ] Statement lowering for both targets
- [ ] Node constructor mapping
- [ ] Type check mapping
- [ ] String method mapping (.clone() stripping for JS)
- [ ] matches! macro expansion
- [ ] format! macro expansion
- [ ] vec! macro expansion
- [ ] Writer codegen (Visit instead of VisitMut)
- [ ] CodeBuilder implementation for both targets

---

## 6. Phase 5: Error Reporting (Days 14)

### 6.1 Error Types

```rust
// src/error/diagnostic.rs

#[derive(Debug)]
pub enum Diagnostic {
    Lex(LexError),
    Parse(ParseError),
    Semantic(SemanticError),
    Type(TypeError),
    Ownership(OwnershipError),
}

#[derive(Debug)]
pub enum LexError {
    UnexpectedChar { ch: char, span: Span },
    UnterminatedString { span: Span },
    UnterminatedComment { span: Span },
    InvalidNumber { span: Span },
}

#[derive(Debug)]
pub enum ParseError {
    Expected { expected: String, found: String, span: Span },
    UnexpectedToken { span: Span },
    UnexpectedEof { span: Span },
}

#[derive(Debug)]
pub enum SemanticError {
    UndefinedVariable { name: String, span: Span },
    DuplicateBinding { name: String, first: Span, second: Span },
    UnusedVariable { name: String, span: Span },
}

#[derive(Debug)]
pub enum TypeError {
    Mismatch { expected: String, found: String, span: Span },
    CannotInfer { span: Span },
}

#[derive(Debug)]
pub enum OwnershipError {
    MissingClone { span: Span, suggestion: String },
    PropertyMutation { span: Span },
    InvalidStatementLowering { span: Span },
}
```

### 6.2 Pretty Printing with Ariadne

```rust
// src/error/report.rs

use ariadne::{Color, Label, Report, ReportKind, Source};

impl Diagnostic {
    pub fn report(&self, filename: &str, source: &str) {
        match self {
            Diagnostic::Ownership(OwnershipError::MissingClone { span, suggestion }) => {
                Report::build(ReportKind::Error, filename, span.start)
                    .with_code("RS001")
                    .with_message("implicit borrow not allowed")
                    .with_label(
                        Label::new((filename, span.start..span.end))
                            .with_message(format!("help: use explicit clone: `{}`", suggestion))
                            .with_color(Color::Red)
                    )
                    .with_note("ReluxScript requires explicit .clone() to extract values from references")
                    .finish()
                    .eprint((filename, Source::from(source)))
                    .unwrap();
            }
            // ... other error types
        }
    }
}
```

### 6.3 Deliverables

- [ ] All error type definitions
- [ ] Ariadne integration
- [ ] Color-coded error messages
- [ ] Error codes (RS001, RS002, etc.)
- [ ] Suggestions and notes
- [ ] Multi-span errors (for duplicate bindings)

---

## 7. Phase 6: CLI & Integration (Day 14)

### 7.1 CLI Interface

```rust
// src/main.rs

use clap::Parser;

#[derive(Parser)]
#[command(name = "reluxscript")]
#[command(about = "Compile ReluxScript to Babel and SWC plugins")]
struct Cli {
    /// Input file
    input: PathBuf,

    /// Output directory
    #[arg(short, long, default_value = "dist")]
    output: PathBuf,

    /// Target platform
    #[arg(short, long, default_value = "both")]
    target: Target,

    /// Show verbose output
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Clone, ValueEnum)]
enum Target {
    Babel,
    Swc,
    Both,
}

fn main() {
    let cli = Cli::parse();

    // Read input
    let source = std::fs::read_to_string(&cli.input)
        .expect("Failed to read input file");

    // Compile
    match compile(&source, &cli) {
        Ok(outputs) => {
            // Write outputs
            if cli.target == Target::Babel || cli.target == Target::Both {
                std::fs::write(cli.output.join("index.js"), &outputs.babel)
                    .expect("Failed to write Babel output");
            }
            if cli.target == Target::Swc || cli.target == Target::Both {
                std::fs::write(cli.output.join("lib.rs"), &outputs.swc)
                    .expect("Failed to write SWC output");
            }

            println!("✓ Compiled successfully");
        }
        Err(diagnostics) => {
            for diag in diagnostics {
                diag.report(&cli.input.to_string_lossy(), &source);
            }
            std::process::exit(1);
        }
    }
}

fn compile(source: &str, cli: &Cli) -> Result<Outputs, Vec<Diagnostic>> {
    // Lex
    let tokens = Lexer::new(source).tokenize()?;

    // Parse
    let mut parser = Parser::new(tokens);
    let mut ast = parser.parse()?;

    // Semantic analysis
    Resolver::new().resolve(&mut ast)?;
    TypeChecker::new().check(&ast)?;
    OwnershipChecker::new().check(&ast)?;

    // Code generation
    let babel = BabelCodegen::new().generate(&ast);
    let swc = SwcCodegen::new().generate(&ast);

    Ok(Outputs { babel, swc })
}
```

### 7.2 Cargo.toml

```toml
[package]
name = "reluxscript"
version = "0.1.0"
edition = "2021"

[dependencies]
ariadne = "0.4"
clap = { version = "4", features = ["derive"] }

[dev-dependencies]
insta = "1.34"  # Snapshot testing
```

### 7.3 Deliverables

- [ ] CLI argument parsing
- [ ] File I/O
- [ ] Error exit codes
- [ ] Verbose mode
- [ ] Output directory creation

---

## 8. Testing Strategy

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_simple_plugin() {
        let input = "plugin Foo { }";
        let tokens = Lexer::new(input).tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Plugin);
        assert_eq!(tokens[1].kind, TokenKind::Ident("Foo".into()));
    }

    #[test]
    fn parse_visitor_method() {
        let input = r#"
            plugin Test {
                fn visit_identifier(node: &mut Identifier, ctx: &Context) {
                    let name = node.name.clone();
                }
            }
        "#;
        let ast = parse(input).unwrap();
        // assertions...
    }
}
```

### 8.2 Snapshot Tests

Using `insta` for codegen output:

```rust
#[test]
fn codegen_simple_plugin() {
    let input = r#"
        plugin ConsoleStripper {
            fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {
                if matches!(node.callee, Identifier { name: "console" }) {
                    *node = Identifier::new("void 0");
                }
            }
        }
    "#;

    let ast = parse(input).unwrap();
    let babel = BabelCodegen::new().generate(&ast);
    let swc = SwcCodegen::new().generate(&ast);

    insta::assert_snapshot!("babel_output", babel);
    insta::assert_snapshot!("swc_output", swc);
}
```

### 8.3 Integration Tests

```rust
// tests/integration.rs

#[test]
fn full_pipeline() {
    let input = std::fs::read_to_string("tests/fixtures/hook_analyzer.rs").unwrap();
    let output = compile(&input).unwrap();

    // Verify Babel output is valid JS
    // Verify Rust output compiles
}
```

---

## 9. Timeline Summary

| Day | Phase | Deliverables |
|-----|-------|--------------|
| 1-2 | Lexer | Token definitions, lexer implementation, tests |
| 3-6 | Parser | AST definitions, recursive descent parser, precedence, tests |
| 7-9 | Semantic | Name resolution, type checking, ownership validation |
| 10-13 | Codegen | Babel output, SWC output, macro expansion |
| 14 | Polish | Error reporting, CLI, integration tests |

**Total: ~2 weeks**

---

## 10. Future Enhancements

### 10.1 Short Term
- LSP server for editor support
- Watch mode for development
- Source maps for debugging

### 10.2 Medium Term
- Module system (`use` imports)
- Generic functions
- Derive macros

### 10.3 Long Term
- Package registry
- REPL
- Playground website

---

## 11. Success Criteria

1. **Correctness**: Generated code produces identical behavior in both targets
2. **Performance**: Compilation under 100ms for typical plugins
3. **Errors**: Clear, actionable error messages with suggestions
4. **Completeness**: All ReluxScript 0.2.0 spec features implemented

---

*End of Implementation Plan*
