//! SWC-specific metadata attached to AST nodes during decoration phase
//!
//! This module defines metadata types that are populated by the SwcDecorator
//! and consumed by the SwcCodegen to generate correct SWC Rust code.
//!
//! The decorator phase resolves semantic mismatches between ReluxScript's
//! Babel-like unified API and SWC's actual AST structure.

use crate::lexer::Span;

/// SWC-specific metadata for pattern matching
#[derive(Debug, Clone)]
pub struct SwcPatternMetadata {
    /// The SWC pattern to emit (e.g., "MemberProp::Ident" instead of "Expr::Ident")
    pub swc_pattern: String,

    /// How to unwrap the value being matched
    pub unwrap_strategy: UnwrapStrategy,

    /// Inner pattern metadata (for nested patterns)
    pub inner: Option<Box<SwcPatternMetadata>>,

    /// Source location for error reporting
    pub span: Option<Span>,

    /// Original ReluxScript pattern (for diagnostics)
    pub source_pattern: Option<String>,

    /// Desugaring strategy for patterns that don't exist in SWC
    pub desugar_strategy: Option<DesugarStrategy>,
}

/// SWC-specific metadata for field access (member expressions)
#[derive(Debug, Clone)]
pub struct SwcFieldMetadata {
    /// The actual SWC field name (e.g., "obj" instead of "object")
    pub swc_field_name: String,

    /// How to access this field
    pub accessor: FieldAccessor,

    /// Type of the field value in SWC (e.g., "Box<Expr>", "MemberProp")
    pub field_type: String,

    /// Original ReluxScript field name (for diagnostics/debug)
    pub source_field: Option<String>,

    /// Source location for error reporting
    pub span: Option<Span>,
}

/// SWC-specific metadata for identifier access
#[derive(Debug, Clone)]
pub struct SwcIdentifierMetadata {
    /// Whether this identifier access needs sym vs name
    /// In SWC, identifiers use `sym` (JsWord/Atom) instead of `name` (String)
    pub use_sym: bool,

    /// Deref pattern needed for string comparisons
    /// e.g., "&*ident.sym" vs just "ident.name"
    pub deref_pattern: Option<String>,

    /// Source location for error reporting
    pub span: Option<Span>,
}

/// SWC-specific metadata for expression evaluation
#[derive(Debug, Clone)]
pub struct SwcExprMetadata {
    /// Type of this expression in SWC
    pub swc_type: String,

    /// Whether this is Box<T>
    pub is_boxed: bool,

    /// Whether this is Option<T>
    pub is_optional: bool,

    /// SWC type kind (Enum, Struct, WrapperEnum, etc.)
    pub type_kind: crate::type_system::SwcTypeKind,

    /// Source location for error reporting
    pub span: Option<Span>,
}

/// Strategy for unwrapping values in pattern matching
#[derive(Debug, Clone, PartialEq)]
pub enum UnwrapStrategy {
    /// Use .as_ref() for Box<T> in read contexts
    AsRef,

    /// Use &* for Box<T> in pattern matching
    RefDeref,

    /// Use & reference only
    Ref,

    /// No unwrapping needed (direct access)
    None,
}

/// Strategy for desugaring patterns that don't exist in SWC
#[derive(Debug, Clone)]
pub enum DesugarStrategy {
    /// Pattern doesn't exist in SWC - needs nested if-let transformation
    /// Example: Callee::MemberExpression → Callee::Expr + Expr::Member
    NestedIfLet {
        /// Outer pattern to generate (e.g., "Callee::Expr")
        outer_pattern: String,
        /// Outer binding variable (e.g., "__callee_expr")
        outer_binding: String,
        /// Inner pattern to generate (e.g., "Expr::Member")
        inner_pattern: String,
        /// Inner binding variable (e.g., "member")
        inner_binding: String,
        /// Unwrap expression (e.g., ".as_ref()")
        unwrap_expr: String,
    },
}

/// Strategy for accessing struct fields
#[derive(Debug, Clone)]
pub enum FieldAccessor {
    /// Direct field access: member.field
    Direct,

    /// Boxed field, needs .as_ref() in read context
    /// Example: member.obj.as_ref()
    BoxedAsRef,

    /// Boxed field in pattern context, use &*
    /// Example: &*member.obj
    BoxedRefDeref,

    /// Enum field that has a different type than expected
    /// Example: member.prop is MemberProp, not Expr
    /// Need to pattern match on it differently
    EnumField {
        /// Name of the enum type (e.g., "MemberProp")
        enum_name: String,

        /// Whether the enum itself is wrapped in Box
        is_boxed: bool,
    },

    /// Optional field, needs unwrapping
    /// Example: node.id (Option<Ident>)
    Optional {
        /// Inner accessor for the unwrapped value
        inner: Box<FieldAccessor>,
    },

    /// Replace entire member expression with a different expression
    /// Example: self.builder → self (in writers)
    Replace {
        /// Replacement expression to emit
        with: String,
    },
}

/// Metadata for if-let statement patterns
#[derive(Debug, Clone)]
pub struct SwcIfLetMetadata {
    /// Type of the condition expression in SWC
    pub condition_swc_type: String,

    /// Translated pattern to emit (complete pattern string)
    pub pattern_translation: String,

    /// Bindings created by this pattern (variable name -> SWC type)
    pub bindings: Vec<(String, String)>,
}

/// Metadata for binary expressions (especially comparisons)
#[derive(Debug, Clone)]
pub struct SwcBinaryMetadata {
    /// Whether the left side needs special handling
    /// e.g., identifier.name == "foo" -> &*ident.sym == "foo"
    pub left_needs_deref: bool,

    /// Whether the right side needs special handling
    pub right_needs_deref: bool,

    /// Source location for error reporting
    pub span: Option<Span>,
}

/// Metadata for unary expressions (dereference, reference)
#[derive(Debug, Clone)]
pub struct SwcUnaryMetadata {
    /// Override the operator if needed
    /// e.g., * might need to become & for enum fields
    pub override_op: Option<String>,

    /// Source location for error reporting
    pub span: Option<Span>,
}

impl SwcPatternMetadata {
    /// Create metadata for a simple pattern with no unwrapping
    pub fn direct(swc_pattern: String) -> Self {
        Self {
            swc_pattern,
            unwrap_strategy: UnwrapStrategy::None,
            inner: None,
            span: None,
            source_pattern: None,
            desugar_strategy: None,
        }
    }

    /// Create metadata for a pattern that needs .as_ref()
    pub fn with_as_ref(swc_pattern: String) -> Self {
        Self {
            swc_pattern,
            unwrap_strategy: UnwrapStrategy::AsRef,
            inner: None,
            span: None,
            source_pattern: None,
            desugar_strategy: None,
        }
    }

    /// Create metadata for a pattern that needs &*
    pub fn with_ref_deref(swc_pattern: String) -> Self {
        Self {
            swc_pattern,
            unwrap_strategy: UnwrapStrategy::RefDeref,
            inner: None,
            span: None,
            source_pattern: None,
            desugar_strategy: None,
        }
    }

    /// Check if this pattern needs desugaring
    pub fn needs_desugaring(&self) -> bool {
        self.desugar_strategy.is_some()
    }
}

impl SwcFieldMetadata {
    /// Create metadata for a direct field access
    pub fn direct(swc_field_name: String, field_type: String) -> Self {
        Self {
            swc_field_name,
            accessor: FieldAccessor::Direct,
            field_type,
            source_field: None,
            span: None,
        }
    }

    /// Create metadata for a boxed field access
    pub fn boxed(swc_field_name: String, field_type: String, in_pattern: bool) -> Self {
        Self {
            swc_field_name,
            accessor: if in_pattern {
                FieldAccessor::BoxedRefDeref
            } else {
                FieldAccessor::BoxedAsRef
            },
            field_type,
            source_field: None,
            span: None,
        }
    }

    /// Create metadata for an enum field
    pub fn enum_field(swc_field_name: String, enum_name: String, field_type: String) -> Self {
        Self {
            swc_field_name,
            accessor: FieldAccessor::EnumField {
                enum_name,
                is_boxed: false
            },
            field_type,
            source_field: None,
            span: None,
        }
    }
}

impl SwcIdentifierMetadata {
    /// Create metadata for sym access with deref
    pub fn sym_with_deref() -> Self {
        Self {
            use_sym: true,
            deref_pattern: Some("&*".to_string()),
            span: None,
        }
    }

    /// Create metadata for direct name access
    pub fn name() -> Self {
        Self {
            use_sym: false,
            deref_pattern: None,
            span: None,
        }
    }
}
