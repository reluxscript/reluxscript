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
    StructInit(DecoratedStructInit),
    VecInit(Vec<DecoratedExpr>),
    If(Box<DecoratedIfExpr>),
    Match(Box<DecoratedMatchExpr>),
    Closure(DecoratedClosureExpr),
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
    CustomPropAccess(Box<DecoratedCustomPropAccess>),
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

/// Decorated struct initialization
#[derive(Debug, Clone)]
pub struct DecoratedStructInit {
    pub name: String,
    pub fields: Vec<(String, DecoratedExpr)>,
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

/// Decorated closure expression
#[derive(Debug, Clone)]
pub struct DecoratedClosureExpr {
    pub params: Vec<crate::parser::ClosureParam>,
    pub body: Box<DecoratedExpr>,
    pub span: crate::lexer::Span,
}

/// Decorated nested function declaration (for helper functions inside blocks)
/// This is different from the top-level DecoratedFnDecl in swc_decorator.rs which
/// is used for visitor methods.
#[derive(Debug, Clone)]
pub struct DecoratedNestedFnDecl {
    pub is_pub: bool,
    pub name: String,
    pub type_params: Vec<crate::parser::GenericParam>,
    pub params: Vec<crate::parser::Param>,
    pub return_type: Option<crate::parser::Type>,
    pub where_clause: Vec<crate::parser::WherePredicate>,
    pub body: DecoratedBlock,
    pub span: crate::lexer::Span,
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
    Traverse(Box<DecoratedTraverseStmt>),
    Function(DecoratedNestedFnDecl),
    Verbatim(crate::parser::VerbatimStmt), // Platform-specific code - no decoration needed
    CustomPropAssignment(Box<DecoratedCustomPropAssignment>),
    Unsafe(DecoratedUnsafeBlock),
}

/// Decorated unsafe block
#[derive(Debug, Clone)]
pub struct DecoratedUnsafeBlock {
    pub stmts: Vec<DecoratedStmt>,
}

/// Decorated let statement
#[derive(Debug, Clone)]
pub struct DecoratedLetStmt {
    pub mutable: bool,
    pub pattern: DecoratedPattern,
    pub ty: Option<crate::parser::Type>,
    pub init: Option<DecoratedExpr>,
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
    pub pattern: Option<DecoratedPattern>,
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

/// Decorated custom AST property assignment
#[derive(Debug, Clone)]
pub struct DecoratedCustomPropAssignment {
    /// The AST node being assigned to
    pub node: DecoratedExpr,

    /// The custom property name (e.g., "__hexPath")
    pub property: String,

    /// The value being assigned
    pub value: DecoratedExpr,

    /// Metadata about the assignment
    pub metadata: SwcCustomPropAssignmentMetadata,
}

/// Decorated custom AST property access
#[derive(Debug, Clone)]
pub struct DecoratedCustomPropAccess {
    /// The AST node being read from
    pub node: Box<DecoratedExpr>,

    /// The custom property name (e.g., "__hexPath")
    pub property: String,

    /// Metadata about the access
    pub metadata: SwcCustomPropAccessMetadata,
}

/// Decorated traverse statement
#[derive(Debug, Clone)]
pub struct DecoratedTraverseStmt {
    pub target: DecoratedExpr,
    pub captures: Vec<crate::parser::Capture>,
    pub kind: DecoratedTraverseKind,
    pub span: crate::lexer::Span,
}

/// Decorated traverse kind
#[derive(Debug, Clone)]
pub enum DecoratedTraverseKind {
    Inline(DecoratedInlineVisitor),
    Delegated(String),
}

/// Decorated inline visitor
#[derive(Debug, Clone)]
pub struct DecoratedInlineVisitor {
    pub state: Vec<crate::parser::LetStmt>, // TODO: Could decorate these too
    pub methods: Vec<DecoratedVisitorMethod>,
}

/// Decorated visitor method
#[derive(Debug, Clone)]
pub struct DecoratedVisitorMethod {
    pub name: String,
    pub params: Vec<crate::parser::Param>,
    pub body: DecoratedBlock,
}
