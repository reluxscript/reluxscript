//! AST definitions for ReluxScript

use crate::lexer::Span;

/// Root of the AST - a program containing a plugin or writer declaration
#[derive(Debug, Clone)]
pub struct Program {
    /// Use statements (imports)
    pub uses: Vec<UseStmt>,
    pub decl: TopLevelDecl,
    pub span: Span,
}

/// Use statement: `use fs;` or `use "./helpers.lux";` or `use "./helpers.lux" { foo, bar };`
#[derive(Debug, Clone)]
pub struct UseStmt {
    /// Module path (e.g., "fs", "./helpers.lux", "../utils/types.lux")
    pub path: String,

    /// Optional alias: `use "./helpers.lux" as h;`
    pub alias: Option<String>,

    /// Specific imports: `use "./helpers.lux" { foo, bar };`
    pub imports: Vec<String>,

    pub span: Span,
}

/// Top-level declaration
#[derive(Debug, Clone)]
pub enum TopLevelDecl {
    Plugin(PluginDecl),
    Writer(WriterDecl),
    Interface(InterfaceDecl),
    Module(ModuleDecl),
}

/// Module declaration (standalone module without plugin/writer)
#[derive(Debug, Clone)]
pub struct ModuleDecl {
    pub items: Vec<PluginItem>,
    pub span: Span,
}

/// Plugin declaration: `plugin Name { ... }`
#[derive(Debug, Clone)]
pub struct PluginDecl {
    pub name: String,
    pub body: Vec<PluginItem>,
    pub span: Span,
}

/// Writer declaration: `writer Name { ... }`
#[derive(Debug, Clone)]
pub struct WriterDecl {
    pub name: String,
    pub body: Vec<PluginItem>,
    pub span: Span,
}

/// Items that can appear inside a plugin/writer
#[derive(Debug, Clone)]
pub enum PluginItem {
    Struct(StructDecl),
    Enum(EnumDecl),
    Function(FnDecl),
    Impl(ImplBlock),
    PreHook(FnDecl),   // fn pre() hook - runs before visitors
    ExitHook(FnDecl),  // fn exit() hook - runs after all visitors
}

/// Struct declaration
#[derive(Debug, Clone)]
pub struct StructDecl {
    pub name: String,
    pub fields: Vec<StructField>,
    pub derives: Vec<String>,  // Traits to derive (e.g., "Clone", "Debug")
    pub span: Span,
}

/// Struct field
#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

/// Enum declaration
#[derive(Debug, Clone)]
pub struct EnumDecl {
    pub name: String,
    pub variants: Vec<EnumVariant>,
    pub span: Span,
}

/// Enum variant fields
#[derive(Debug, Clone)]
pub enum EnumVariantFields {
    /// Tuple variant: Some(T)
    Tuple(Vec<Type>),
    /// Struct variant: Error { message: String }
    Struct(Vec<(String, Type)>),
    /// Unit variant: None
    Unit,
}

/// Enum variant
#[derive(Debug, Clone)]
pub struct EnumVariant {
    pub name: String,
    pub fields: EnumVariantFields,
    pub span: Span,
}

/// Function declaration
#[derive(Debug, Clone)]
pub struct FnDecl {
    pub is_pub: bool,
    pub name: String,
    pub type_params: Vec<GenericParam>,  // <F, T>
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub where_clause: Vec<WherePredicate>,  // where F: Fn(...)
    pub body: Block,
    pub span: Span,
}

/// Generic type parameter (for Rust-style generics)
#[derive(Debug, Clone)]
pub struct GenericParam {
    pub name: String,
    pub span: Span,
}

/// Where clause predicate
#[derive(Debug, Clone)]
pub struct WherePredicate {
    pub target: String,  // The type parameter name (e.g., "F")
    pub bound: Type,     // The trait bound (e.g., Fn(...) -> Str)
    pub span: Span,
}

/// Function parameter
#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub span: Span,
}

/// Impl block
#[derive(Debug, Clone)]
pub struct ImplBlock {
    pub target: String,
    pub items: Vec<FnDecl>,
    pub span: Span,
}

/// Type representation
#[derive(Debug, Clone)]
pub enum Type {
    /// Primitive types: Str, i32, f64, bool, ()
    Primitive(String),
    /// Reference type: &T or &mut T
    Reference {
        mutable: bool,
        inner: Box<Type>,
    },
    /// Container types: Vec<T>, Option<T>, HashMap<K,V>, etc.
    Container {
        name: String,
        type_args: Vec<Type>,
    },
    /// User-defined type or AST node type
    Named(String),
    /// Array type: [T]
    Array { element: Box<Type> },
    /// Tuple type: (T1, T2)
    Tuple(Vec<Type>),
    /// Optional type: T?
    Optional(Box<Type>),
    /// Unit type: ()
    Unit,
    /// Function trait: Fn(T1, T2) -> R
    FnTrait {
        params: Vec<Type>,
        return_type: Box<Type>,
    },
}

// =============================================================================
// TypeScript AST Types
// =============================================================================

/// TypeScript type annotation AST node
/// Distinct from ReluxScript's Type enum which represents variable types
#[derive(Debug, Clone)]
pub enum TsType {
    // Keywords
    String,
    Number,
    Boolean,
    Any,
    Void,
    Null,
    Undefined,
    Never,
    Unknown,

    // Compound types
    Array(Box<TsType>),
    Tuple(Vec<TsType>),
    Union(Vec<TsType>),
    Intersection(Vec<TsType>),

    // Reference types
    TypeReference {
        name: String,
        type_args: Vec<TsType>,
    },

    // Function types
    FunctionType {
        params: Vec<TsType>,
        return_type: Box<TsType>,
    },

    // Literal types
    LiteralString(String),
    LiteralNumber(f64),
    LiteralBoolean(bool),
}

impl TsType {
    /// Convert TypeScript type to ReluxScript Type for codegen
    pub fn to_reluxscript_type(&self) -> Type {
        match self {
            TsType::String => Type::Primitive("Str".to_string()),
            TsType::Number => Type::Primitive("f64".to_string()),
            TsType::Boolean => Type::Primitive("bool".to_string()),
            TsType::Array(inner) => Type::Container {
                name: "Vec".to_string(),
                type_args: vec![inner.to_reluxscript_type()],
            },
            TsType::TypeReference { name, type_args } => {
                if type_args.is_empty() {
                    Type::Named(name.clone())
                } else {
                    Type::Container {
                        name: name.clone(),
                        type_args: type_args.iter().map(|t| t.to_reluxscript_type()).collect(),
                    }
                }
            }
            TsType::Void => Type::Unit,
            TsType::Null | TsType::Undefined => Type::Named("None".to_string()),
            _ => Type::Named("dynamic".to_string()),
        }
    }
}

// =============================================================================
// TypeScript Declaration Types
// =============================================================================

/// Interface declaration (per Refinement 3: flattened structure)
#[derive(Debug, Clone)]
pub struct InterfaceDecl {
    pub name: String,
    pub members: Vec<InterfaceMember>,
    pub extends: Vec<String>,
    pub type_params: Vec<TypeParam>,
    pub span: Span,
}

/// Interface member types
#[derive(Debug, Clone)]
pub enum InterfaceMember {
    Property(PropertySignature),
    Method(MethodSignature),
    Index(IndexSignature),
}

/// Property signature in an interface
#[derive(Debug, Clone)]
pub struct PropertySignature {
    pub key: String,
    pub type_annotation: Option<TsType>,
    pub optional: bool,
    pub readonly: bool,
    pub span: Span,
}

/// Method signature in an interface
#[derive(Debug, Clone)]
pub struct MethodSignature {
    pub name: String,
    pub params: Vec<TsType>,
    pub return_type: Option<TsType>,
    pub optional: bool,
    pub span: Span,
}

/// Index signature in an interface: [key: string]: T
#[derive(Debug, Clone)]
pub struct IndexSignature {
    pub key_name: String,
    pub key_type: TsType,
    pub value_type: TsType,
    pub span: Span,
}

/// Type parameter: T, T extends U, T = Default
#[derive(Debug, Clone)]
pub struct TypeParam {
    pub name: String,
    pub constraint: Option<TsType>,
    pub default: Option<TsType>,
    pub span: Span,
}

/// Template element for template literals
#[derive(Debug, Clone)]
pub struct TemplateElement {
    pub raw: String,
    pub cooked: Option<String>,
    pub tail: bool,
    pub span: Span,
}

/// Block of statements
#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

/// Statement
#[derive(Debug, Clone)]
pub enum Stmt {
    Let(LetStmt),
    Const(ConstStmt),
    Expr(ExprStmt),
    If(IfStmt),
    Match(MatchStmt),
    For(ForStmt),
    While(WhileStmt),
    Loop(LoopStmt),
    Return(ReturnStmt),
    Break(BreakStmt),
    Continue(ContinueStmt),
    Traverse(TraverseStmt),
    Function(FnDecl),  // Nested function declaration
    Verbatim(VerbatimStmt),  // Platform-specific code block
    CustomPropAssignment(CustomPropAssignment),  // Custom AST property assignment
}

/// Let statement: `let [mut] name [: Type] = expr;`
#[derive(Debug, Clone)]
pub struct LetStmt {
    pub mutable: bool,
    pub pattern: Pattern,  // Changed from name: String to support destructuring
    pub ty: Option<Type>,
    pub init: Expr,
    pub span: Span,
}

/// Const statement: `const NAME = expr;`
#[derive(Debug, Clone)]
pub struct ConstStmt {
    pub name: String,
    pub ty: Option<Type>,
    pub init: Expr,
    pub span: Span,
}

/// Expression statement
#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub expr: Expr,
    pub span: Span,
}

/// If statement
#[derive(Debug, Clone)]
pub struct IfStmt {
    pub condition: Expr,
    /// Optional pattern for if-let: `if let Some(x) = expr`
    pub pattern: Option<Pattern>,
    pub then_branch: Block,
    pub else_if_branches: Vec<(Expr, Block)>,
    pub else_branch: Option<Block>,
    pub span: Span,
}

/// Match statement
#[derive(Debug, Clone)]
pub struct MatchStmt {
    pub scrutinee: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// Match arm
#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pattern: Pattern,
    pub body: Expr,
    pub span: Span,
}

/// Pattern for matching
#[derive(Debug, Clone)]
pub enum Pattern {
    Literal(Literal),
    Ident(String),
    Wildcard,
    Tuple(Vec<Pattern>),
    Struct {
        name: String,
        fields: Vec<(String, Pattern)>,
    },
    /// Enum variant pattern: Some(x), None, Ok(x), Err(e)
    Variant {
        name: String,
        inner: Option<Box<Pattern>>,
    },
    /// Array destructuring: [a, b, c]
    Array(Vec<Pattern>),
    /// Object destructuring: { x, y, z } or { x: a, y: b }
    Object(Vec<ObjectPatternProp>),
    /// Rest pattern: ...rest
    Rest(Box<Pattern>),
    /// Or pattern: pattern1 | pattern2
    Or(Vec<Pattern>),
    /// Ref pattern: ref x or ref mut x (creates a reference binding)
    Ref {
        is_mut: bool,
        pattern: Box<Pattern>,
    },
}

/// Object pattern property
#[derive(Debug, Clone)]
pub enum ObjectPatternProp {
    /// Shorthand: { x } binds x
    Shorthand(String),
    /// Key-value: { x: renamed } binds renamed
    KeyValue { key: String, value: Pattern },
    /// Rest: { ...rest }
    Rest(String),
    Or(Vec<Pattern>),
}

/// For loop
#[derive(Debug, Clone)]
pub struct ForStmt {
    pub pattern: Pattern,  // Changed from var: String to support tuple destructuring
    pub iter: Expr,
    pub body: Block,
    pub span: Span,
}

/// While loop
#[derive(Debug, Clone)]
pub struct WhileStmt {
    pub condition: Expr,
    pub body: Block,
    pub span: Span,
}

/// Loop (infinite)
#[derive(Debug, Clone)]
pub struct LoopStmt {
    pub body: Block,
    pub span: Span,
}

/// Return statement
#[derive(Debug, Clone)]
pub struct ReturnStmt {
    pub value: Option<Expr>,
    pub span: Span,
}

/// Break statement
#[derive(Debug, Clone)]
pub struct BreakStmt {
    pub span: Span,
}

/// Continue statement
#[derive(Debug, Clone)]
pub struct ContinueStmt {
    pub span: Span,
}

/// Verbatim code block: platform-specific raw code
/// Supports: babel!{}, js!{}, swc!{}, rust!{}
#[derive(Debug, Clone)]
pub struct VerbatimStmt {
    pub target: VerbatimTarget,
    pub code: String,
    pub span: Span,
}

/// Target platform for verbatim code
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VerbatimTarget {
    /// JavaScript (Babel) - babel!{} or js!{}
    JavaScript,
    /// Rust (SWC) - swc!{} or rust!{}
    Rust,
}

/// Custom AST property assignment: `node.__propName = value;`
/// Custom properties are identified by double underscore prefix and allow
/// attaching metadata to AST nodes in a cross-platform way.
#[derive(Debug, Clone)]
pub struct CustomPropAssignment {
    /// The AST node being assigned to (e.g., `node`)
    pub node: Box<Expr>,
    /// The custom property name (e.g., "__hexPath")
    pub property: String,
    /// The value being assigned
    pub value: Box<Expr>,
    /// Optional type annotation
    pub ty: Option<Type>,
    pub span: Span,
}

/// Traverse statement: `traverse(node) { ... }` or `traverse(node) using Visitor;`
/// This is the scoped traversal construct that bridges Babel's path.traverse and SWC's visit_mut_with
#[derive(Debug, Clone)]
pub struct TraverseStmt {
    /// The node to traverse
    pub target: Expr,
    /// Captured variables from outer scope
    /// `traverse(node) capturing [&mut x, &y] { ... }`
    pub captures: Vec<Capture>,
    /// The kind of traversal (inline visitor or delegated to another visitor)
    pub kind: TraverseKind,
    pub span: Span,
}

/// A captured variable reference in a traverse block
/// Used in `capturing [&mut x, &y]` syntax
#[derive(Debug, Clone)]
pub struct Capture {
    /// Name of the captured variable
    pub name: String,
    /// Whether the capture is mutable (&mut vs &)
    pub mutable: bool,
    pub span: Span,
}

/// Kind of traverse operation
#[derive(Debug, Clone)]
pub enum TraverseKind {
    /// Inline visitor block with local state and visitor methods
    /// `traverse(node) { let count = 0; fn visit_identifier(...) { ... } }`
    Inline(InlineVisitor),
    /// Delegate to another visitor
    /// `traverse(node) using OtherVisitor;`
    Delegated(String),
}

/// Inline visitor defined within a traverse block
#[derive(Debug, Clone)]
pub struct InlineVisitor {
    /// Local state declarations (let statements)
    pub state: Vec<LetStmt>,
    /// Visitor methods
    pub methods: Vec<FnDecl>,
    pub span: Span,
}

impl TraverseStmt {
    pub fn span(&self) -> Span {
        self.span
    }
}

/// Expression
#[derive(Debug, Clone)]
pub enum Expr {
    /// Literal value
    Literal(Literal),
    /// Identifier
    Ident(IdentExpr),
    /// Binary operation
    Binary(BinaryExpr),
    /// Unary operation
    Unary(UnaryExpr),
    /// Function/method call
    Call(CallExpr),
    /// Member access: expr.field
    Member(MemberExpr),
    /// Index access: expr[index]
    Index(IndexExpr),
    /// Struct initialization
    StructInit(StructInitExpr),
    /// Vec initialization: vec![...]
    VecInit(VecInitExpr),
    /// If expression (when used as expression)
    If(Box<IfExpr>),
    /// Match expression
    Match(Box<MatchExpr>),
    /// Closure/lambda
    Closure(ClosureExpr),
    /// Reference: &expr or &mut expr
    Ref(RefExpr),
    /// Dereference: *expr
    Deref(DerefExpr),
    /// Assignment: lhs = rhs
    Assign(AssignExpr),
    /// Compound assignment: lhs += rhs, etc.
    CompoundAssign(CompoundAssignExpr),
    /// Range: start..end
    Range(RangeExpr),
    /// Parenthesized expression
    Paren(Box<Expr>),
    /// Block expression (used in closures, if/match arms, etc.)
    Block(Block),
    /// Try expression: expr?
    Try(Box<Expr>),
    /// Tuple expression: (expr1, expr2, ...)
    Tuple(Vec<Expr>),
    /// Matches macro: matches!(expr, pattern)
    Matches(MatchesExpr),
    /// Regex call: Regex::matches(), Regex::find(), etc.
    RegexCall(RegexCall),
    /// Custom AST property access: node.__propName
    CustomPropAccess(CustomPropAccess),
    /// Return expression: return expr
    Return(Option<Box<Expr>>),
    /// Break expression: break
    Break,
    /// Continue expression: continue
    Continue,
}

/// Literal values
#[derive(Debug, Clone)]
pub enum Literal {
    String(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Null,
    Unit,
}

/// Identifier expression
#[derive(Debug, Clone)]
pub struct IdentExpr {
    pub name: String,
    pub span: Span,
}

/// Binary expression
#[derive(Debug, Clone)]
pub struct BinaryExpr {
    pub op: BinaryOp,
    pub left: Box<Expr>,
    pub right: Box<Expr>,
    pub span: Span,
}

/// Binary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOp {
    Add,
    Sub,
    Mul,
    Div,
    Mod,
    Eq,
    NotEq,
    Lt,
    Gt,
    LtEq,
    GtEq,
    And,
    Or,
}

/// Unary expression
#[derive(Debug, Clone)]
pub struct UnaryExpr {
    pub op: UnaryOp,
    pub operand: Box<Expr>,
    pub span: Span,
}

/// Unary operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
    Deref,
    Ref,
    RefMut,
}

/// Call expression
#[derive(Debug, Clone)]
pub struct CallExpr {
    pub callee: Box<Expr>,
    pub args: Vec<Expr>,
    pub type_args: Vec<TsType>,
    pub optional: bool,
    /// True if this is a macro call (e.g., format!(...), vec![...])
    pub is_macro: bool,
    pub span: Span,
}

/// Member access expression
#[derive(Debug, Clone)]
pub struct MemberExpr {
    pub object: Box<Expr>,
    pub property: String,
    pub optional: bool,
    pub computed: bool,
    /// True if this is a path expression (::) rather than member access (.)
    pub is_path: bool,
    pub span: Span,
}

/// Index expression
#[derive(Debug, Clone)]
pub struct IndexExpr {
    pub object: Box<Expr>,
    pub index: Box<Expr>,
    pub span: Span,
}

/// Struct initialization
#[derive(Debug, Clone)]
pub struct StructInitExpr {
    pub name: String,
    pub fields: Vec<(String, Expr)>,
    pub span: Span,
}

/// Vec initialization
#[derive(Debug, Clone)]
pub struct VecInitExpr {
    pub elements: Vec<Expr>,
    pub span: Span,
}

/// If expression
#[derive(Debug, Clone)]
pub struct IfExpr {
    pub condition: Expr,
    /// Optional pattern for if-let: `if let Some(x) = expr`
    pub pattern: Option<Pattern>,
    pub then_branch: Block,
    pub else_branch: Option<Block>,
    pub span: Span,
}

/// Match expression
#[derive(Debug, Clone)]
pub struct MatchExpr {
    pub scrutinee: Expr,
    pub arms: Vec<MatchArm>,
    pub span: Span,
}

/// Matches macro expression: matches!(expr, pattern)
#[derive(Debug, Clone)]
pub struct MatchesExpr {
    pub scrutinee: Box<Expr>,
    pub pattern: Pattern,
    pub span: Span,
}

/// Regex call expression: Regex::matches(), Regex::find(), etc.
#[derive(Debug, Clone)]
pub struct RegexCall {
    pub method: RegexMethod,
    pub text_arg: Box<Expr>,
    pub pattern_arg: String,  // Must be a string literal
    pub replacement_arg: Option<Box<Expr>>,  // For replace/replace_all
    pub span: Span,
}

/// Custom AST property access: `node.__propName`
/// Reads a custom property from an AST node. Always returns Option<T>.
#[derive(Debug, Clone)]
pub struct CustomPropAccess {
    /// The AST node being accessed (e.g., `node`)
    pub node: Box<Expr>,
    /// The custom property name (e.g., "__hexPath")
    pub property: String,
    pub span: Span,
}

/// Regex method variants
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegexMethod {
    /// Regex::matches(text, pattern) -> bool
    Matches,
    /// Regex::find(text, pattern) -> Option<String>
    Find,
    /// Regex::find_all(text, pattern) -> Vec<String>
    FindAll,
    /// Regex::captures(text, pattern) -> Option<Captures>
    Captures,
    /// Regex::replace(text, pattern, replacement) -> String
    Replace,
    /// Regex::replace_all(text, pattern, replacement) -> String
    ReplaceAll,
}

/// Closure expression
#[derive(Debug, Clone)]
pub struct ClosureExpr {
    pub params: Vec<String>,
    pub body: Box<Expr>,
    pub span: Span,
}

/// Reference expression
#[derive(Debug, Clone)]
pub struct RefExpr {
    pub mutable: bool,
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Dereference expression
#[derive(Debug, Clone)]
pub struct DerefExpr {
    pub expr: Box<Expr>,
    pub span: Span,
}

/// Assignment expression
#[derive(Debug, Clone)]
pub struct AssignExpr {
    pub target: Box<Expr>,
    pub value: Box<Expr>,
    pub span: Span,
}

/// Compound assignment expression
#[derive(Debug, Clone)]
pub struct CompoundAssignExpr {
    pub op: CompoundAssignOp,
    pub target: Box<Expr>,
    pub value: Box<Expr>,
    pub span: Span,
}

/// Compound assignment operators
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompoundAssignOp {
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

/// Range expression
#[derive(Debug, Clone)]
pub struct RangeExpr {
    pub start: Option<Box<Expr>>,
    pub end: Option<Box<Expr>>,
    pub inclusive: bool,
    pub span: Span,
}
