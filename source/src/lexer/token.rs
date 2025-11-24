//! Token definitions for ReluxScript

use std::fmt;

/// Source location span
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
    pub line: usize,
    pub column: usize,
}

impl Span {
    pub fn new(start: usize, end: usize, line: usize, column: usize) -> Self {
        Self { start, end, line, column }
    }
}

/// Token with kind and span
#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Span,
}

impl Token {
    pub fn new(kind: TokenKind, span: Span) -> Self {
        Self { kind, span }
    }
}

/// All token types in ReluxScript
#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Keywords
    Fn,
    Let,
    Const,
    Mut,
    If,
    Else,
    For,
    In,
    While,
    Loop,
    Return,
    Break,
    Continue,
    True,
    False,
    Null,
    Plugin,
    Writer,
    Struct,
    Enum,
    Impl,
    Use,
    Pub,
    As,
    Self_,
    SelfType,
    Match,
    Matches,
    Traverse,
    Using,
    Capturing,

    // Types
    Str,
    Bool,
    I32,
    U32,
    F64,
    Vec,
    Option,
    Result,
    HashMap,
    HashSet,
    CodeBuilder,

    // AST Node Types (Unified AST)
    Program,
    FunctionDeclaration,
    VariableDeclaration,
    ExpressionStatement,
    ReturnStatement,
    IfStatement,
    ForStatement,
    WhileStatement,
    BlockStatement,
    Identifier,
    Literal,
    BinaryExpression,
    UnaryExpression,
    CallExpression,
    MemberExpression,
    ArrayExpression,
    ObjectExpression,
    JSXElement,
    JSXFragment,
    JSXAttribute,
    JSXText,
    JSXExpressionContainer,

    // Identifiers and Literals
    Ident(String),
    StringLit(String),
    IntLit(i64),
    FloatLit(f64),

    // Operators
    Plus,           // +
    Minus,          // -
    Star,           // *
    Slash,          // /
    Percent,        // %
    Eq,             // =
    PlusEq,         // +=
    MinusEq,        // -=
    StarEq,         // *=
    SlashEq,        // /=
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
    Caret,          // ^
    DotDot,         // ..
    DotDotDot,      // ...

    // Delimiters
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Comma,          // ,
    Dot,            // .
    Colon,          // :
    Semicolon,      // ;
    Arrow,          // ->
    DDArrow,        // =>
    ColonColon,     // ::
    Question,       // ?
    QuestionDot,    // ?.
    Hash,           // #

    // Special
    Comment(String),
    DocComment(String),
    Newline,
    Eof,

    // Error token for recovery
    Error(String),
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Fn => write!(f, "fn"),
            TokenKind::Let => write!(f, "let"),
            TokenKind::Const => write!(f, "const"),
            TokenKind::Mut => write!(f, "mut"),
            TokenKind::If => write!(f, "if"),
            TokenKind::Else => write!(f, "else"),
            TokenKind::For => write!(f, "for"),
            TokenKind::In => write!(f, "in"),
            TokenKind::While => write!(f, "while"),
            TokenKind::Loop => write!(f, "loop"),
            TokenKind::Return => write!(f, "return"),
            TokenKind::Break => write!(f, "break"),
            TokenKind::Continue => write!(f, "continue"),
            TokenKind::True => write!(f, "true"),
            TokenKind::False => write!(f, "false"),
            TokenKind::Null => write!(f, "null"),
            TokenKind::Plugin => write!(f, "plugin"),
            TokenKind::Writer => write!(f, "writer"),
            TokenKind::Struct => write!(f, "struct"),
            TokenKind::Enum => write!(f, "enum"),
            TokenKind::Impl => write!(f, "impl"),
            TokenKind::Use => write!(f, "use"),
            TokenKind::Pub => write!(f, "pub"),
            TokenKind::As => write!(f, "as"),
            TokenKind::Self_ => write!(f, "self"),
            TokenKind::SelfType => write!(f, "Self"),
            TokenKind::Match => write!(f, "match"),
            TokenKind::Matches => write!(f, "matches!"),
            TokenKind::Traverse => write!(f, "traverse"),
            TokenKind::Using => write!(f, "using"),
            TokenKind::Capturing => write!(f, "capturing"),

            TokenKind::Str => write!(f, "Str"),
            TokenKind::Bool => write!(f, "bool"),
            TokenKind::I32 => write!(f, "i32"),
            TokenKind::U32 => write!(f, "u32"),
            TokenKind::F64 => write!(f, "f64"),
            TokenKind::Vec => write!(f, "Vec"),
            TokenKind::Option => write!(f, "Option"),
            TokenKind::Result => write!(f, "Result"),
            TokenKind::HashMap => write!(f, "HashMap"),
            TokenKind::HashSet => write!(f, "HashSet"),
            TokenKind::CodeBuilder => write!(f, "CodeBuilder"),

            TokenKind::Program => write!(f, "Program"),
            TokenKind::FunctionDeclaration => write!(f, "FunctionDeclaration"),
            TokenKind::VariableDeclaration => write!(f, "VariableDeclaration"),
            TokenKind::ExpressionStatement => write!(f, "ExpressionStatement"),
            TokenKind::ReturnStatement => write!(f, "ReturnStatement"),
            TokenKind::IfStatement => write!(f, "IfStatement"),
            TokenKind::ForStatement => write!(f, "ForStatement"),
            TokenKind::WhileStatement => write!(f, "WhileStatement"),
            TokenKind::BlockStatement => write!(f, "BlockStatement"),
            TokenKind::Identifier => write!(f, "Identifier"),
            TokenKind::Literal => write!(f, "Literal"),
            TokenKind::BinaryExpression => write!(f, "BinaryExpression"),
            TokenKind::UnaryExpression => write!(f, "UnaryExpression"),
            TokenKind::CallExpression => write!(f, "CallExpression"),
            TokenKind::MemberExpression => write!(f, "MemberExpression"),
            TokenKind::ArrayExpression => write!(f, "ArrayExpression"),
            TokenKind::ObjectExpression => write!(f, "ObjectExpression"),
            TokenKind::JSXElement => write!(f, "JSXElement"),
            TokenKind::JSXFragment => write!(f, "JSXFragment"),
            TokenKind::JSXAttribute => write!(f, "JSXAttribute"),
            TokenKind::JSXText => write!(f, "JSXText"),
            TokenKind::JSXExpressionContainer => write!(f, "JSXExpressionContainer"),

            TokenKind::Ident(s) => write!(f, "{}", s),
            TokenKind::StringLit(s) => write!(f, "\"{}\"", s),
            TokenKind::IntLit(n) => write!(f, "{}", n),
            TokenKind::FloatLit(n) => write!(f, "{}", n),

            TokenKind::Plus => write!(f, "+"),
            TokenKind::Minus => write!(f, "-"),
            TokenKind::Star => write!(f, "*"),
            TokenKind::Slash => write!(f, "/"),
            TokenKind::Percent => write!(f, "%"),
            TokenKind::Eq => write!(f, "="),
            TokenKind::PlusEq => write!(f, "+="),
            TokenKind::MinusEq => write!(f, "-="),
            TokenKind::StarEq => write!(f, "*="),
            TokenKind::SlashEq => write!(f, "/="),
            TokenKind::EqEq => write!(f, "=="),
            TokenKind::NotEq => write!(f, "!="),
            TokenKind::Lt => write!(f, "<"),
            TokenKind::Gt => write!(f, ">"),
            TokenKind::LtEq => write!(f, "<="),
            TokenKind::GtEq => write!(f, ">="),
            TokenKind::And => write!(f, "&&"),
            TokenKind::Or => write!(f, "||"),
            TokenKind::Not => write!(f, "!"),
            TokenKind::Ampersand => write!(f, "&"),
            TokenKind::Pipe => write!(f, "|"),
            TokenKind::Caret => write!(f, "^"),
            TokenKind::DotDot => write!(f, ".."),
            TokenKind::DotDotDot => write!(f, "..."),

            TokenKind::LParen => write!(f, "("),
            TokenKind::RParen => write!(f, ")"),
            TokenKind::LBrace => write!(f, "{{"),
            TokenKind::RBrace => write!(f, "}}"),
            TokenKind::LBracket => write!(f, "["),
            TokenKind::RBracket => write!(f, "]"),
            TokenKind::Comma => write!(f, ","),
            TokenKind::Dot => write!(f, "."),
            TokenKind::Colon => write!(f, ":"),
            TokenKind::Semicolon => write!(f, ";"),
            TokenKind::Arrow => write!(f, "->"),
            TokenKind::DDArrow => write!(f, "=>"),
            TokenKind::ColonColon => write!(f, "::"),
            TokenKind::Question => write!(f, "?"),
            TokenKind::QuestionDot => write!(f, "?."),
            TokenKind::Hash => write!(f, "#"),

            TokenKind::Comment(s) => write!(f, "// {}", s),
            TokenKind::DocComment(s) => write!(f, "/// {}", s),
            TokenKind::Newline => write!(f, "\\n"),
            TokenKind::Eof => write!(f, "EOF"),
            TokenKind::Error(s) => write!(f, "ERROR: {}", s),
        }
    }
}
