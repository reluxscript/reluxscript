//! Decorated AST for SWC code generation
//!
//! This module defines AST types that have been enriched with SWC-specific
//! semantic metadata. The decorator transforms the original parser AST into
//! this decorated form before SWC codegen.
//!
//! Benefits:
//! - Metadata is always present (no Option unwrapping)
//! - Type-safe: compiler enforces that decoration happened
//! - Clean separation: original AST unchanged for parser/Babel
//! - Fast: Rust's move semantics avoid unnecessary copies

use crate::lexer::Span;
use crate::parser::{Literal, ObjectPatternProp};
use super::swc_metadata::*;

/// Decorated pattern with SWC metadata
#[derive(Debug, Clone)]
pub struct DecoratedPattern {
    pub kind: DecoratedPatternKind,
    pub metadata: SwcPatternMetadata,
}

/// Decorated pattern kind
#[derive(Debug, Clone)]
pub enum DecoratedPatternKind {
    Literal(Literal),
    Ident(String),
    Wildcard,
    Tuple(Vec<DecoratedPattern>),
    Struct {
        name: String,
        fields: Vec<(String, DecoratedPattern)>,
    },
    Variant {
        name: String,
        inner: Option<Box<DecoratedPattern>>,
    },
    Array(Vec<DecoratedPattern>),
    Object(Vec<ObjectPatternProp>),
    Rest(Box<DecoratedPattern>),
    Or(Vec<DecoratedPattern>),
    Ref {
        is_mut: bool,
        pattern: Box<DecoratedPattern>,
    },
}

/// Decorated expression with SWC metadata
#[derive(Debug, Clone)]
pub struct DecoratedExpr {
    pub kind: DecoratedExprKind,
    pub metadata: SwcExprMetadata,
}

/// Decorated expression kind
#[derive(Debug, Clone)]
pub enum DecoratedExprKind {
    Literal(Literal),
    Ident {
        name: String,
        ident_metadata: SwcIdentifierMetadata,
    },
    Binary {
        left: Box<DecoratedExpr>,
        op: crate::parser::BinaryOp,
        right: Box<DecoratedExpr>,
        binary_metadata: SwcBinaryMetadata,
    },
    Unary {
        op: crate::parser::UnaryOp,
        operand: Box<DecoratedExpr>,
        unary_metadata: SwcUnaryMetadata,
    },
    Call(Box<DecoratedCallExpr>),
    Member {
        object: Box<DecoratedExpr>,
        property: String,
        optional: bool,
        computed: bool,
        is_path: bool,
        field_metadata: SwcFieldMetadata,
    },
    Index {
        object: Box<DecoratedExpr>,
        index: Box<DecoratedExpr>,
    },
    StructInit(crate::parser::StructInitExpr), // TODO: Decorate if needed
    VecInit(Vec<DecoratedExpr>),
    If(Box<DecoratedIfExpr>),
    Match(Box<DecoratedMatchExpr>),
    Closure(crate::parser::ClosureExpr), // TODO: Decorate if needed
    Ref {
        mutable: bool,
        expr: Box<DecoratedExpr>,
    },
    Deref(Box<DecoratedExpr>),
    Assign {
        left: Box<DecoratedExpr>,
        right: Box<DecoratedExpr>,
    },
    CompoundAssign {
        left: Box<DecoratedExpr>,
        op: crate::parser::CompoundAssignOp,
        right: Box<DecoratedExpr>,
    },
    Range {
        start: Option<Box<DecoratedExpr>>,
        end: Option<Box<DecoratedExpr>>,
        inclusive: bool,
    },
    Paren(Box<DecoratedExpr>),
    Block(DecoratedBlock),
    Try(Box<DecoratedExpr>),
    Tuple(Vec<DecoratedExpr>),
    Matches {
        expr: Box<DecoratedExpr>,
        pattern: DecoratedPattern,
    },
    Return(Option<Box<DecoratedExpr>>),
    Break,
    Continue,
    RegexCall(Box<DecoratedRegexCall>),
    CustomPropAccess(crate::parser::CustomPropAccess), // TODO: Add metadata for type info
}

/// Decorated call expression
#[derive(Debug, Clone)]
pub struct DecoratedCallExpr {
    pub callee: DecoratedExpr,
    pub args: Vec<DecoratedExpr>,
    pub type_args: Vec<crate::parser::TsType>,
    pub optional: bool,
    pub is_macro: bool,
    pub span: Span,
}

/// Decorated regex call expression
#[derive(Debug, Clone)]
pub struct DecoratedRegexCall {
    pub method: crate::parser::RegexMethod,
    pub text_arg: DecoratedExpr,
    pub pattern: String,
    pub replacement_arg: Option<DecoratedExpr>,
    pub metadata: SwcRegexMetadata,
    pub span: Span,
}

/// Decorated if statement
#[derive(Debug, Clone)]
pub struct DecoratedIfStmt {
    pub condition: DecoratedExpr,
    pub pattern: Option<DecoratedPattern>,
    pub then_branch: DecoratedBlock,
    pub else_branch: Option<DecoratedBlock>,
    pub if_let_metadata: Option<SwcIfLetMetadata>,
}

/// Decorated block
#[derive(Debug, Clone)]
pub struct DecoratedBlock {
    pub stmts: Vec<DecoratedStmt>,
}

/// Decorated statement
#[derive(Debug, Clone)]
pub enum DecoratedStmt {
    Let(DecoratedLetStmt),
    Const(DecoratedConstStmt),
    Expr(DecoratedExpr),
    If(DecoratedIfStmt),
    Match(DecoratedMatchStmt),
    For(DecoratedForStmt),
    While(DecoratedWhileStmt),
    Loop(DecoratedBlock),
    Return(Option<DecoratedExpr>),
    Break,
    Continue,
    Traverse(crate::parser::TraverseStmt), // TODO: Decorate if needed
    Function(crate::parser::FnDecl), // Nested functions - TODO: Decorate if needed
    Verbatim(crate::parser::VerbatimStmt), // Platform-specific code - no decoration needed
    CustomPropAssignment(crate::parser::CustomPropAssignment), // TODO: Decorate with metadata
}

/// Decorated let statement
#[derive(Debug, Clone)]
pub struct DecoratedLetStmt {
    pub mutable: bool,
    pub pattern: DecoratedPattern,
    pub ty: Option<crate::parser::Type>,
    pub init: DecoratedExpr,
}

/// Decorated const statement
#[derive(Debug, Clone)]
pub struct DecoratedConstStmt {
    pub name: String,
    pub ty: Option<crate::parser::Type>,
    pub init: DecoratedExpr,
}

/// Decorated match statement
#[derive(Debug, Clone)]
pub struct DecoratedMatchStmt {
    pub expr: DecoratedExpr,
    pub arms: Vec<DecoratedMatchArm>,
}

/// Decorated match arm
#[derive(Debug, Clone)]
pub struct DecoratedMatchArm {
    pub pattern: DecoratedPattern,
    pub guard: Option<DecoratedExpr>,
    pub body: DecoratedBlock,
}

/// Decorated for statement
#[derive(Debug, Clone)]
pub struct DecoratedForStmt {
    pub pattern: DecoratedPattern,
    pub iter: DecoratedExpr,
    pub body: DecoratedBlock,
}

/// Decorated while statement
#[derive(Debug, Clone)]
pub struct DecoratedWhileStmt {
    pub condition: DecoratedExpr,
    pub body: DecoratedBlock,
}

/// Decorated if expression (when used as expression)
#[derive(Debug, Clone)]
pub struct DecoratedIfExpr {
    pub condition: DecoratedExpr,
    pub then_branch: DecoratedBlock,
    pub else_branch: Option<DecoratedBlock>,
}

/// Decorated match expression
#[derive(Debug, Clone)]
pub struct DecoratedMatchExpr {
    pub expr: DecoratedExpr,
    pub arms: Vec<DecoratedMatchArm>,
}

impl DecoratedPattern {
    /// Create a simple decorated pattern
    pub fn new(kind: DecoratedPatternKind, metadata: SwcPatternMetadata) -> Self {
        Self { kind, metadata }
    }
}

impl DecoratedExpr {
    /// Create a decorated expression
    pub fn new(kind: DecoratedExprKind, metadata: SwcExprMetadata) -> Self {
        Self { kind, metadata }
    }
}
