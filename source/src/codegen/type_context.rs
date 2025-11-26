//! Type context for type-aware SWC code generation
//!
//! This module implements flow-sensitive typing to handle the architectural
//! divergence between Babel's uniform Node hierarchy and SWC's strict
//! Enum/Struct hierarchy.

use std::collections::HashMap;

/// Classification of SWC types
#[derive(Clone, Debug, PartialEq)]
pub enum SwcTypeKind {
    /// Top-level enums that require pattern matching (Expr, Stmt, Decl, Pat)
    Enum,
    /// Direct structs with accessible fields (MemberExpr, Ident, CallExpr)
    Struct,
    /// Wrapper enums that wrap other types (MemberProp, PropName)
    WrapperEnum,
    /// Interned strings (JsWord, Atom)
    Atom,
    /// Primitive types (String, bool, i32)
    Primitive,
    /// Box<T> wrapper - treat as T but needs dereference
    Boxed(Box<SwcTypeKind>),
    /// Option<T> wrapper
    Optional(Box<SwcTypeKind>),
    /// Unknown type
    Unknown,
}

/// Type context for an expression or variable
#[derive(Clone, Debug)]
pub struct TypeContext {
    /// ReluxScript type name (e.g., "MemberExpression", "Identifier")
    pub reluxscript_type: String,
    /// SWC type name (e.g., "MemberExpr", "Ident")
    pub swc_type: String,
    /// Classification of the SWC type
    pub kind: SwcTypeKind,
    /// For Enums: which variant are we known to be?
    /// e.g., if swc_type is "Expr" but known_variant is "Member",
    /// we can safely access MemberExpr fields
    pub known_variant: Option<String>,
    /// Whether this value needs Box dereference to access
    pub needs_deref: bool,
}

impl TypeContext {
    /// Create an unknown type context
    pub fn unknown() -> Self {
        Self {
            reluxscript_type: "UserDefined".into(),
            swc_type: "UserDefined".into(),
            kind: SwcTypeKind::Unknown,
            known_variant: None,
            needs_deref: false,
        }
    }

    /// Create a type context for a known ReluxScript type
    pub fn from_reluxscript(rs_type: &str) -> Self {
        let (swc_type, kind) = map_reluxscript_to_swc(rs_type);
        Self {
            reluxscript_type: rs_type.to_string(),
            swc_type,
            kind,
            known_variant: None,
            needs_deref: false,
        }
    }

    /// Create a narrowed type context (after pattern matching)
    pub fn narrowed(rs_type: &str, swc_struct: &str) -> Self {
        Self {
            reluxscript_type: rs_type.to_string(),
            swc_type: swc_struct.to_string(),
            kind: SwcTypeKind::Struct,
            known_variant: None,
            needs_deref: false,
        }
    }

    /// Check if this type is boxed
    pub fn is_boxed(&self) -> bool {
        matches!(self.kind, SwcTypeKind::Boxed(_)) || self.needs_deref
    }

    /// Check if this type requires enum unwrapping
    pub fn needs_unwrap(&self) -> bool {
        matches!(self.kind, SwcTypeKind::Enum | SwcTypeKind::WrapperEnum)
            && self.known_variant.is_none()
    }

    /// Get the inner type if boxed
    pub fn unboxed(&self) -> Self {
        if let SwcTypeKind::Boxed(inner) = &self.kind {
            Self {
                reluxscript_type: self.reluxscript_type.clone(),
                swc_type: self.swc_type.trim_start_matches("Box<")
                    .trim_end_matches('>')
                    .to_string(),
                kind: (**inner).clone(),
                known_variant: self.known_variant.clone(),
                needs_deref: true,
            }
        } else {
            self.clone()
        }
    }

    /// Unwrap a generic type to get the element type
    /// e.g., "Vec<Option<Pat>>" -> "Option<Pat>"
    /// e.g., "Option<Pat>" -> "Pat"
    pub fn unwrap_generic(&self) -> Self {
        let swc = &self.swc_type;

        // Handle Vec<T>
        if swc.starts_with("Vec<") && swc.ends_with(">") {
            let inner = &swc[4..swc.len()-1];
            return Self::from_swc_type(inner);
        }

        // Handle Option<T>
        if swc.starts_with("Option<") && swc.ends_with(">") {
            let inner = &swc[7..swc.len()-1];
            return Self::from_swc_type(inner);
        }

        // Handle Box<T>
        if swc.starts_with("Box<") && swc.ends_with(">") {
            let inner = &swc[4..swc.len()-1];
            let mut ctx = Self::from_swc_type(inner);
            ctx.needs_deref = true;
            return ctx;
        }

        self.clone()
    }

    /// Create TypeContext from a SWC type string
    pub fn from_swc_type(swc_type: &str) -> Self {
        // Determine kind from type name
        let kind = match swc_type {
            "Expr" => SwcTypeKind::Enum,
            "Stmt" => SwcTypeKind::Enum,
            "Decl" => SwcTypeKind::Enum,
            "Pat" => SwcTypeKind::Enum,
            "Lit" => SwcTypeKind::Enum,
            "MemberProp" | "PropName" | "Callee" => SwcTypeKind::WrapperEnum,
            s if s.starts_with("Vec<") => SwcTypeKind::Unknown, // Collection
            s if s.starts_with("Option<") => SwcTypeKind::Unknown, // Optional
            s if s.starts_with("Box<") => SwcTypeKind::Unknown, // Boxed
            _ => SwcTypeKind::Struct,
        };

        Self {
            reluxscript_type: swc_type.to_string(),
            swc_type: swc_type.to_string(),
            kind,
            known_variant: None,
            needs_deref: false,
        }
    }

    /// Get the base enum type for pattern matching
    /// e.g., for "Pat" return "Pat", for "Ident" with context return "Pat" or "Expr"
    pub fn get_enum_context(&self) -> Option<String> {
        match self.swc_type.as_str() {
            "Expr" | "Stmt" | "Decl" | "Pat" | "Lit" => Some(self.swc_type.clone()),
            _ => None,
        }
    }
}

/// Type environment for tracking variable types through scopes
pub struct TypeEnvironment {
    /// Stack of scopes, each mapping variable names to types
    scopes: Vec<HashMap<String, TypeContext>>,
    /// Stack of field refinements per scope (e.g., "decl.id" -> ArrayPat)
    refinements: Vec<HashMap<String, TypeContext>>,
}

impl TypeEnvironment {
    /// Create a new type environment with global scope
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
            refinements: vec![HashMap::new()],
        }
    }

    /// Push a new scope (for entering blocks, if/while let, etc.)
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
        self.refinements.push(HashMap::new());
    }

    /// Pop the current scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
            self.refinements.pop();
        }
    }

    /// Register a field refinement (e.g., "decl.id" is now ArrayPat)
    pub fn refine_field(&mut self, path: &str, ctx: TypeContext) {
        if let Some(scope) = self.refinements.last_mut() {
            scope.insert(path.to_string(), ctx);
        }
    }

    /// Look up a field refinement
    pub fn lookup_field_refinement(&self, obj: &str, field: &str) -> Option<&TypeContext> {
        let path = format!("{}.{}", obj, field);
        for scope in self.refinements.iter().rev() {
            if let Some(ctx) = scope.get(&path) {
                return Some(ctx);
            }
        }
        None
    }

    /// Define a variable in the current scope
    /// This implements variable shadowing - defining a variable with the same
    /// name in an inner scope shadows the outer definition
    pub fn define(&mut self, name: &str, ctx: TypeContext) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), ctx);
        }
    }

    /// Look up a variable's type, searching from innermost to outermost scope
    pub fn lookup(&self, name: &str) -> Option<&TypeContext> {
        for scope in self.scopes.iter().rev() {
            if let Some(ctx) = scope.get(name) {
                return Some(ctx);
            }
        }
        None
    }

    /// Check if a variable is defined in the current (innermost) scope
    pub fn is_defined_in_current_scope(&self, name: &str) -> bool {
        self.scopes.last()
            .map(|s| s.contains_key(name))
            .unwrap_or(false)
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}

/// Map ReluxScript type name to SWC type name and kind
pub fn map_reluxscript_to_swc(rs_type: &str) -> (String, SwcTypeKind) {
    match rs_type {
        // Expressions (Enum)
        "Expr" => ("Expr".into(), SwcTypeKind::Enum),
        "Expression" => ("Expr".into(), SwcTypeKind::Enum),

        // Statements (Enum)
        "Stmt" => ("Stmt".into(), SwcTypeKind::Enum),
        "Statement" => ("Stmt".into(), SwcTypeKind::Enum),

        // Declarations (Enum)
        "Decl" => ("Decl".into(), SwcTypeKind::Enum),
        "Declaration" => ("Decl".into(), SwcTypeKind::Enum),

        // Patterns (Enum)
        "Pat" => ("Pat".into(), SwcTypeKind::Enum),
        "Pattern" => ("Pat".into(), SwcTypeKind::Enum),

        // Literals (Enum)
        "Lit" => ("Lit".into(), SwcTypeKind::Enum),
        "Literal" => ("Lit".into(), SwcTypeKind::Enum),

        // Specific expression types (Struct after unwrapping)
        "MemberExpression" => ("MemberExpr".into(), SwcTypeKind::Struct),
        "CallExpression" => ("CallExpr".into(), SwcTypeKind::Struct),
        "Identifier" => ("Ident".into(), SwcTypeKind::Struct),
        "BinaryExpression" => ("BinExpr".into(), SwcTypeKind::Struct),
        "UnaryExpression" => ("UnaryExpr".into(), SwcTypeKind::Struct),
        "AssignmentExpression" => ("AssignExpr".into(), SwcTypeKind::Struct),
        "ArrayExpression" => ("ArrayLit".into(), SwcTypeKind::Struct),
        "ObjectExpression" => ("ObjectLit".into(), SwcTypeKind::Struct),
        "FunctionExpression" => ("FnExpr".into(), SwcTypeKind::Struct),
        "ArrowFunctionExpression" => ("ArrowExpr".into(), SwcTypeKind::Struct),

        // Specific statement types (Struct after unwrapping)
        "BlockStatement" => ("BlockStmt".into(), SwcTypeKind::Struct),
        "ReturnStatement" => ("ReturnStmt".into(), SwcTypeKind::Struct),
        "IfStatement" => ("IfStmt".into(), SwcTypeKind::Struct),
        "WhileStatement" => ("WhileStmt".into(), SwcTypeKind::Struct),
        "ForStatement" => ("ForStmt".into(), SwcTypeKind::Struct),
        "ExpressionStatement" => ("ExprStmt".into(), SwcTypeKind::Struct),

        // Specific declaration types (Struct after unwrapping)
        "FunctionDeclaration" => ("FnDecl".into(), SwcTypeKind::Struct),
        "VariableDeclaration" => ("VarDecl".into(), SwcTypeKind::Struct),
        "ClassDeclaration" => ("ClassDecl".into(), SwcTypeKind::Struct),

        // Wrapper enums (special handling needed)
        "MemberProp" => ("MemberProp".into(), SwcTypeKind::WrapperEnum),
        "PropName" => ("PropName".into(), SwcTypeKind::WrapperEnum),
        "Callee" => ("Callee".into(), SwcTypeKind::WrapperEnum),
        "JSXObject" => ("JSXObject".into(), SwcTypeKind::WrapperEnum),
        "JSXElementName" => ("JSXElementName".into(), SwcTypeKind::WrapperEnum),
        "JSXAttrValue" => ("JSXAttrValue".into(), SwcTypeKind::WrapperEnum),
        "JSXAttrOrSpread" => ("JSXAttrOrSpread".into(), SwcTypeKind::WrapperEnum),

        // Literal types (Struct after unwrapping from Lit)
        "StringLiteral" => ("Str".into(), SwcTypeKind::Struct),
        "NumericLiteral" => ("Number".into(), SwcTypeKind::Struct),
        "BooleanLiteral" => ("Bool".into(), SwcTypeKind::Struct),
        "NullLiteral" => ("Null".into(), SwcTypeKind::Struct),
        "RegExpLiteral" => ("Regex".into(), SwcTypeKind::Struct),
        "BigIntLiteral" => ("BigInt".into(), SwcTypeKind::Struct),

        // Pattern types (Struct after unwrapping from Pat)
        "ArrayPattern" => ("ArrayPat".into(), SwcTypeKind::Struct),
        "ObjectPattern" => ("ObjectPat".into(), SwcTypeKind::Struct),
        "RestElement" => ("RestPat".into(), SwcTypeKind::Struct),
        "AssignmentPattern" => ("AssignPat".into(), SwcTypeKind::Struct),
        "BindingIdentifier" => ("BindingIdent".into(), SwcTypeKind::Struct),

        // JSX types (Struct)
        "JSXElement" => ("JSXElement".into(), SwcTypeKind::Struct),
        "JSXFragment" => ("JSXFragment".into(), SwcTypeKind::Struct),
        "JSXAttribute" => ("JSXAttr".into(), SwcTypeKind::Struct),
        "JSXExpressionContainer" => ("JSXExprContainer".into(), SwcTypeKind::Struct),
        "JSXMemberExpression" => ("JSXMemberExpr".into(), SwcTypeKind::Struct),
        "JSXNamespacedName" => ("JSXNamespacedName".into(), SwcTypeKind::Struct),

        // Atom types
        "JsWord" => ("JsWord".into(), SwcTypeKind::Atom),
        "Atom" => ("Atom".into(), SwcTypeKind::Atom),
        "Str" => ("String".into(), SwcTypeKind::Primitive),

        // Primitives
        "bool" => ("bool".into(), SwcTypeKind::Primitive),
        "i32" => ("i32".into(), SwcTypeKind::Primitive),
        "u32" => ("u32".into(), SwcTypeKind::Primitive),
        "f64" => ("f64".into(), SwcTypeKind::Primitive),
        "String" => ("String".into(), SwcTypeKind::Primitive),

        // Unknown
        _ => (rs_type.to_string(), SwcTypeKind::Unknown),
    }
}

/// Classify an SWC type name
pub fn classify_swc_type(type_name: &str) -> SwcTypeKind {
    match type_name {
        // Top-level enums
        "Expr" | "Stmt" | "Decl" | "Pat" | "Lit" | "ModuleItem" => SwcTypeKind::Enum,

        // Wrapper enums
        "MemberProp" | "PropName" | "Callee" |
        "JSXObject" | "JSXElementName" | "JSXAttrValue" | "JSXAttrOrSpread" => SwcTypeKind::WrapperEnum,

        // Structs - Expressions
        "Ident" | "MemberExpr" | "CallExpr" | "BinExpr" | "UnaryExpr" |
        "AssignExpr" | "ArrayLit" | "ObjectLit" | "FnExpr" | "ArrowExpr" |
        "CondExpr" | "NewExpr" | "SeqExpr" | "ThisExpr" | "Tpl" |
        // Structs - Statements
        "BlockStmt" | "ReturnStmt" | "IfStmt" | "WhileStmt" | "ForStmt" |
        "ForInStmt" | "ForOfStmt" | "SwitchStmt" | "ThrowStmt" | "TryStmt" |
        "DoWhileStmt" | "BreakStmt" | "ContinueStmt" | "ExprStmt" |
        // Structs - Declarations
        "FnDecl" | "VarDecl" | "ClassDecl" |
        // Structs - Function components
        "Param" | "Function" | "BlockStmtOrExpr" |
        // Structs - Variable components
        "VarDeclarator" |
        // Structs - Patterns
        "BindingIdent" | "ArrayPat" | "ObjectPat" | "RestPat" | "AssignPat" |
        // Structs - Literals
        "Str" | "Number" | "Bool" | "Null" | "Regex" | "BigInt" |
        // Structs - JSX
        "JSXElement" | "JSXFragment" | "JSXAttr" | "JSXExprContainer" |
        "JSXMemberExpr" | "JSXNamespacedName" | "JSXOpeningElement" |
        "JSXClosingElement" | "JSXText" | "JSXSpreadChild" => SwcTypeKind::Struct,

        // Atoms
        "JsWord" | "Atom" => SwcTypeKind::Atom,

        // Box<T>
        s if s.starts_with("Box<") => {
            let inner = &s[4..s.len() - 1];
            SwcTypeKind::Boxed(Box::new(classify_swc_type(inner)))
        }

        // Option<T>
        s if s.starts_with("Option<") => {
            let inner = &s[7..s.len() - 1];
            SwcTypeKind::Optional(Box::new(classify_swc_type(inner)))
        }

        // Primitives
        "i32" | "f64" | "bool" | "String" | "usize" => SwcTypeKind::Primitive,

        _ => SwcTypeKind::Unknown,
    }
}

/// Get the SWC enum and variant for a ReluxScript type
/// Returns (enum_name, variant_name, struct_name)
pub fn get_swc_variant(rs_type: &str) -> (String, String, String) {
    // Default context is Expr
    get_swc_variant_in_context(rs_type, "Expr")
}

/// Get the SWC enum and variant for a ReluxScript type within a specific context
/// The context determines which enum to match against (e.g., "Expr", "MemberProp", "Callee")
/// Returns (enum_name, variant_name, struct_name)
pub fn get_swc_variant_in_context(rs_type: &str, context: &str) -> (String, String, String) {
    // Handle MemberProp context - Identifier maps to MemberProp::Ident
    if context == "MemberProp" {
        return match rs_type {
            "Identifier" => ("MemberProp".into(), "Ident".into(), "Ident".into()),
            "ComputedPropName" => ("MemberProp".into(), "Computed".into(), "ComputedPropName".into()),
            _ => ("MemberProp".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Handle Callee context
    if context == "Callee" {
        return match rs_type {
            "Super" => ("Callee".into(), "Super".into(), "Super".into()),
            "Import" => ("Callee".into(), "Import".into(), "Import".into()),
            // Everything else is Callee::Expr(Box<Expr>)
            _ => ("Callee".into(), "Expr".into(), "Expr".into()),
        };
    }

    // Handle Lit context - literals
    if context == "Lit" {
        return match rs_type {
            "StringLiteral" => ("Lit".into(), "Str".into(), "Str".into()),
            "NumericLiteral" => ("Lit".into(), "Num".into(), "Number".into()),
            "BooleanLiteral" => ("Lit".into(), "Bool".into(), "Bool".into()),
            "NullLiteral" => ("Lit".into(), "Null".into(), "Null".into()),
            "RegExpLiteral" => ("Lit".into(), "Regex".into(), "Regex".into()),
            "BigIntLiteral" => ("Lit".into(), "BigInt".into(), "BigInt".into()),
            _ => ("Lit".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Handle Pat context - patterns
    if context == "Pat" {
        return match rs_type {
            "Identifier" => ("Pat".into(), "Ident".into(), "BindingIdent".into()),
            "ArrayPattern" => ("Pat".into(), "Array".into(), "ArrayPat".into()),
            "ObjectPattern" => ("Pat".into(), "Object".into(), "ObjectPat".into()),
            "RestElement" => ("Pat".into(), "Rest".into(), "RestPat".into()),
            "AssignmentPattern" => ("Pat".into(), "Assign".into(), "AssignPat".into()),
            _ => ("Pat".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Handle PropName context - property names in objects
    if context == "PropName" {
        return match rs_type {
            "Identifier" => ("PropName".into(), "Ident".into(), "Ident".into()),
            "StringLiteral" => ("PropName".into(), "Str".into(), "Str".into()),
            "NumericLiteral" => ("PropName".into(), "Num".into(), "Number".into()),
            "ComputedPropName" => ("PropName".into(), "Computed".into(), "ComputedPropName".into()),
            "BigIntLiteral" => ("PropName".into(), "BigInt".into(), "BigInt".into()),
            _ => ("PropName".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Handle JSXObject context - JSX member expression objects
    if context == "JSXObject" {
        return match rs_type {
            "Identifier" => ("JSXObject".into(), "Ident".into(), "Ident".into()),
            "JSXMemberExpression" => ("JSXObject".into(), "JSXMemberExpr".into(), "JSXMemberExpr".into()),
            _ => ("JSXObject".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Handle JSXElementName context - JSX element tag names
    if context == "JSXElementName" {
        return match rs_type {
            "Identifier" => ("JSXElementName".into(), "Ident".into(), "Ident".into()),
            "JSXMemberExpression" => ("JSXElementName".into(), "JSXMemberExpr".into(), "JSXMemberExpr".into()),
            "JSXNamespacedName" => ("JSXElementName".into(), "JSXNamespacedName".into(), "JSXNamespacedName".into()),
            _ => ("JSXElementName".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Handle JSXAttrValue context - JSX attribute values
    if context == "JSXAttrValue" {
        return match rs_type {
            "StringLiteral" => ("JSXAttrValue".into(), "Lit".into(), "Lit".into()),
            "JSXExpressionContainer" => ("JSXAttrValue".into(), "JSXExprContainer".into(), "JSXExprContainer".into()),
            "JSXElement" => ("JSXAttrValue".into(), "JSXElement".into(), "JSXElement".into()),
            "JSXFragment" => ("JSXAttrValue".into(), "JSXFragment".into(), "JSXFragment".into()),
            _ => ("JSXAttrValue".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Handle JSXAttrOrSpread context - JSX attributes
    if context == "JSXAttrOrSpread" {
        return match rs_type {
            "JSXAttribute" => ("JSXAttrOrSpread".into(), "JSXAttr".into(), "JSXAttr".into()),
            "SpreadElement" => ("JSXAttrOrSpread".into(), "SpreadElement".into(), "SpreadElement".into()),
            _ => ("JSXAttrOrSpread".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Handle TsTypeElement context - TypeScript interface members
    if context == "TsTypeElement" {
        return match rs_type {
            "TSPropertySignature" => ("TsTypeElement".into(), "TsPropertySignature".into(), "TsPropertySignature".into()),
            "TSMethodSignature" => ("TsTypeElement".into(), "TsMethodSignature".into(), "TsMethodSignature".into()),
            "TSIndexSignature" => ("TsTypeElement".into(), "TsIndexSignature".into(), "TsIndexSignature".into()),
            "TSCallSignatureDeclaration" => ("TsTypeElement".into(), "TsCallSignatureDecl".into(), "TsCallSignatureDecl".into()),
            "TSConstructSignatureDeclaration" => ("TsTypeElement".into(), "TsConstructSignatureDecl".into(), "TsConstructSignatureDecl".into()),
            "TSGetterSignature" => ("TsTypeElement".into(), "TsGetterSignature".into(), "TsGetterSignature".into()),
            "TSSetterSignature" => ("TsTypeElement".into(), "TsSetterSignature".into(), "TsSetterSignature".into()),
            _ => ("TsTypeElement".into(), rs_type.to_string(), rs_type.to_string()),
        };
    }

    // Default Expr context
    match rs_type {
        // Expressions
        "MemberExpression" => ("Expr".into(), "Member".into(), "MemberExpr".into()),
        "CallExpression" => ("Expr".into(), "Call".into(), "CallExpr".into()),
        "Identifier" => ("Expr".into(), "Ident".into(), "Ident".into()),
        "BinaryExpression" => ("Expr".into(), "Bin".into(), "BinExpr".into()),
        "UnaryExpression" => ("Expr".into(), "Unary".into(), "UnaryExpr".into()),
        "AssignmentExpression" => ("Expr".into(), "Assign".into(), "AssignExpr".into()),
        "ArrayExpression" => ("Expr".into(), "Array".into(), "ArrayLit".into()),
        "ObjectExpression" => ("Expr".into(), "Object".into(), "ObjectLit".into()),
        "FunctionExpression" => ("Expr".into(), "Fn".into(), "FnExpr".into()),
        "ArrowFunctionExpression" => ("Expr".into(), "Arrow".into(), "ArrowExpr".into()),
        // Literals - when context is Expr, we need Expr::Lit(Lit::X) pattern
        // But the current pattern generation doesn't support nested patterns
        // For now, map to Expr::Lit for literals in Expr context
        "StringLiteral" => ("Expr".into(), "Lit(Lit::Str".into(), "Str".into()),
        "NumericLiteral" => ("Expr".into(), "Lit(Lit::Num".into(), "Number".into()),
        "BooleanLiteral" => ("Expr".into(), "Lit(Lit::Bool".into(), "Bool".into()),
        "NullLiteral" => ("Expr".into(), "Lit(Lit::Null".into(), "Null".into()),

        // Statements
        "BlockStatement" => ("Stmt".into(), "Block".into(), "BlockStmt".into()),
        "ReturnStatement" => ("Stmt".into(), "Return".into(), "ReturnStmt".into()),
        "IfStatement" => ("Stmt".into(), "If".into(), "IfStmt".into()),
        "WhileStatement" => ("Stmt".into(), "While".into(), "WhileStmt".into()),
        "ForStatement" => ("Stmt".into(), "For".into(), "ForStmt".into()),
        "ExpressionStatement" => ("Stmt".into(), "Expr".into(), "ExprStmt".into()),

        // Declarations
        "FunctionDeclaration" => ("Decl".into(), "Fn".into(), "FnDecl".into()),
        "VariableDeclaration" => ("Decl".into(), "Var".into(), "VarDecl".into()),
        "ClassDeclaration" => ("Decl".into(), "Class".into(), "ClassDecl".into()),

        _ => ("UserDefined".into(), rs_type.to_string(), rs_type.to_string()),
    }
}

/// Field mapping with type information
#[derive(Clone, Debug)]
pub struct TypedFieldMapping {
    pub reluxscript_field: &'static str,
    pub swc_field: &'static str,
    pub needs_deref: bool,
    pub result_type_rs: &'static str,
    pub result_type_swc: &'static str,
    pub read_conversion: &'static str,  // e.g., ".to_string()"
    pub write_conversion: &'static str, // e.g., ".into()"
}

/// Get field mapping for a parent type and field name
pub fn get_typed_field_mapping(parent_swc_type: &str, field: &str) -> Option<TypedFieldMapping> {
    match (parent_swc_type, field) {
        // MemberExpr fields
        ("MemberExpr", "object") => Some(TypedFieldMapping {
            reluxscript_field: "object",
            swc_field: "obj",
            needs_deref: true, // Box<Expr>
            result_type_rs: "Expr",
            result_type_swc: "Expr",
            read_conversion: "",
            write_conversion: "",
        }),
        ("MemberExpr", "property") => Some(TypedFieldMapping {
            reluxscript_field: "property",
            swc_field: "prop",
            needs_deref: false,
            result_type_rs: "MemberProp",
            result_type_swc: "MemberProp", // WrapperEnum!
            read_conversion: "",
            write_conversion: "",
        }),

        // CallExpr fields
        ("CallExpr", "callee") => Some(TypedFieldMapping {
            reluxscript_field: "callee",
            swc_field: "callee",
            needs_deref: false,
            result_type_rs: "Callee",
            result_type_swc: "Callee",
            read_conversion: "",
            write_conversion: "",
        }),
        ("CallExpr", "arguments") => Some(TypedFieldMapping {
            reluxscript_field: "arguments",
            swc_field: "args",
            needs_deref: false,
            result_type_rs: "Vec<ExprOrSpread>",
            result_type_swc: "Vec<ExprOrSpread>",
            read_conversion: "",
            write_conversion: "",
        }),

        // Param fields
        ("Param", "pattern") => Some(TypedFieldMapping {
            reluxscript_field: "pattern",
            swc_field: "pat",
            needs_deref: false,
            result_type_rs: "Pat",
            result_type_swc: "Pat",
            read_conversion: "",
            write_conversion: "",
        }),

        // Ident fields
        ("Ident", "name") => Some(TypedFieldMapping {
            reluxscript_field: "name",
            swc_field: "sym",
            needs_deref: false,
            result_type_rs: "Str",
            result_type_swc: "JsWord",
            read_conversion: ".to_string()",
            write_conversion: ".into()",
        }),

        // BlockStmt fields
        ("BlockStmt", "body") | ("BlockStmt", "stmts") => Some(TypedFieldMapping {
            reluxscript_field: "body",
            swc_field: "stmts",
            needs_deref: false,
            result_type_rs: "Vec<Stmt>",
            result_type_swc: "Vec<Stmt>",
            read_conversion: "",
            write_conversion: "",
        }),

        // ReturnStmt fields
        ("ReturnStmt", "argument") => Some(TypedFieldMapping {
            reluxscript_field: "argument",
            swc_field: "arg",
            needs_deref: false,
            result_type_rs: "Option<Expr>",
            result_type_swc: "Option<Box<Expr>>",
            read_conversion: "",
            write_conversion: "",
        }),

        // FnDecl fields
        ("FnDecl", "id") => Some(TypedFieldMapping {
            reluxscript_field: "id",
            swc_field: "ident",
            needs_deref: false,
            result_type_rs: "Identifier",
            result_type_swc: "Ident",
            read_conversion: "",
            write_conversion: "",
        }),
        ("FnDecl", "params") => Some(TypedFieldMapping {
            reluxscript_field: "params",
            swc_field: "function.params",
            needs_deref: false,
            result_type_rs: "Vec<Pat>",
            result_type_swc: "Vec<Param>",
            read_conversion: "",
            write_conversion: "",
        }),

        // ArrayPat fields
        ("ArrayPat", "elements") => Some(TypedFieldMapping {
            reluxscript_field: "elements",
            swc_field: "elems",
            needs_deref: false,
            result_type_rs: "Vec<Option<Pat>>",
            result_type_swc: "Vec<Option<Pat>>",
            read_conversion: "",
            write_conversion: "",
        }),

        // ArrayLit fields (ArrayExpression)
        ("ArrayLit", "elements") => Some(TypedFieldMapping {
            reluxscript_field: "elements",
            swc_field: "elems",
            needs_deref: false,
            result_type_rs: "Vec<Option<ExprOrSpread>>",
            result_type_swc: "Vec<Option<ExprOrSpread>>",
            read_conversion: "",
            write_conversion: "",
        }),

        // VariableDeclarator fields
        ("VariableDeclarator", "id") | ("VarDeclarator", "id") => Some(TypedFieldMapping {
            reluxscript_field: "id",
            swc_field: "name",
            needs_deref: false,
            result_type_rs: "Pat",
            result_type_swc: "Pat",
            read_conversion: "",
            write_conversion: "",
        }),
        ("VariableDeclarator", "init") | ("VarDeclarator", "init") => Some(TypedFieldMapping {
            reluxscript_field: "init",
            swc_field: "init",
            needs_deref: true,
            result_type_rs: "Option<Expr>",
            result_type_swc: "Option<Box<Expr>>",
            read_conversion: "",
            write_conversion: "",
        }),

        // ExprOrSpread fields
        ("ExprOrSpread", "expr") => Some(TypedFieldMapping {
            reluxscript_field: "expr",
            swc_field: "expr",
            needs_deref: true,
            result_type_rs: "Expr",
            result_type_swc: "Box<Expr>",
            read_conversion: "",
            write_conversion: "",
        }),

        // Literal fields
        ("Str", "value") => Some(TypedFieldMapping {
            reluxscript_field: "value",
            swc_field: "value",
            needs_deref: false,
            result_type_rs: "Str",
            result_type_swc: "JsWord",
            read_conversion: ".to_string()",
            write_conversion: ".into()",
        }),
        ("Number", "value") => Some(TypedFieldMapping {
            reluxscript_field: "value",
            swc_field: "value",
            needs_deref: false,
            result_type_rs: "f64",
            result_type_swc: "f64",
            read_conversion: "",
            write_conversion: "",
        }),
        ("Bool", "value") => Some(TypedFieldMapping {
            reluxscript_field: "value",
            swc_field: "value",
            needs_deref: false,
            result_type_rs: "bool",
            result_type_swc: "bool",
            read_conversion: "",
            write_conversion: "",
        }),

        // Pattern fields
        ("BindingIdent", "name") | ("BindingIdent", "id") => Some(TypedFieldMapping {
            reluxscript_field: "name",
            swc_field: "id.sym",
            needs_deref: false,
            result_type_rs: "Str",
            result_type_swc: "JsWord",
            read_conversion: ".to_string()",
            write_conversion: ".into()",
        }),
        ("ArrayPat", "elements") => Some(TypedFieldMapping {
            reluxscript_field: "elements",
            swc_field: "elems",
            needs_deref: false,
            result_type_rs: "Vec<Option<Pat>>",
            result_type_swc: "Vec<Option<Pat>>",
            read_conversion: "",
            write_conversion: "",
        }),
        ("ObjectPat", "properties") => Some(TypedFieldMapping {
            reluxscript_field: "properties",
            swc_field: "props",
            needs_deref: false,
            result_type_rs: "Vec<ObjectPatProp>",
            result_type_swc: "Vec<ObjectPatProp>",
            read_conversion: "",
            write_conversion: "",
        }),
        ("RestPat", "argument") => Some(TypedFieldMapping {
            reluxscript_field: "argument",
            swc_field: "arg",
            needs_deref: true,
            result_type_rs: "Pat",
            result_type_swc: "Pat",
            read_conversion: "",
            write_conversion: "",
        }),

        // JSX fields
        ("JSXElement", "openingElement") => Some(TypedFieldMapping {
            reluxscript_field: "openingElement",
            swc_field: "opening",
            needs_deref: false,
            result_type_rs: "JSXOpeningElement",
            result_type_swc: "JSXOpeningElement",
            read_conversion: "",
            write_conversion: "",
        }),
        ("JSXElement", "closingElement") => Some(TypedFieldMapping {
            reluxscript_field: "closingElement",
            swc_field: "closing",
            needs_deref: false,
            result_type_rs: "Option<JSXClosingElement>",
            result_type_swc: "Option<JSXClosingElement>",
            read_conversion: "",
            write_conversion: "",
        }),
        ("JSXElement", "children") => Some(TypedFieldMapping {
            reluxscript_field: "children",
            swc_field: "children",
            needs_deref: false,
            result_type_rs: "Vec<JSXElementChild>",
            result_type_swc: "Vec<JSXElementChild>",
            read_conversion: "",
            write_conversion: "",
        }),
        ("JSXOpeningElement", "name") => Some(TypedFieldMapping {
            reluxscript_field: "name",
            swc_field: "name",
            needs_deref: false,
            result_type_rs: "JSXElementName",
            result_type_swc: "JSXElementName",
            read_conversion: "",
            write_conversion: "",
        }),
        ("JSXOpeningElement", "attributes") => Some(TypedFieldMapping {
            reluxscript_field: "attributes",
            swc_field: "attrs",
            needs_deref: false,
            result_type_rs: "Vec<JSXAttrOrSpread>",
            result_type_swc: "Vec<JSXAttrOrSpread>",
            read_conversion: "",
            write_conversion: "",
        }),
        ("JSXAttr", "name") => Some(TypedFieldMapping {
            reluxscript_field: "name",
            swc_field: "name",
            needs_deref: false,
            result_type_rs: "JSXAttrName",
            result_type_swc: "JSXAttrName",
            read_conversion: "",
            write_conversion: "",
        }),
        ("JSXAttr", "value") => Some(TypedFieldMapping {
            reluxscript_field: "value",
            swc_field: "value",
            needs_deref: false,
            result_type_rs: "Option<JSXAttrValue>",
            result_type_swc: "Option<JSXAttrValue>",
            read_conversion: "",
            write_conversion: "",
        }),
        ("JSXExprContainer", "expression") => Some(TypedFieldMapping {
            reluxscript_field: "expression",
            swc_field: "expr",
            needs_deref: false,
            result_type_rs: "JSXExpr",
            result_type_swc: "JSXExpr",
            read_conversion: "",
            write_conversion: "",
        }),
        ("JSXMemberExpr", "object") => Some(TypedFieldMapping {
            reluxscript_field: "object",
            swc_field: "obj",
            needs_deref: false,
            result_type_rs: "JSXObject",
            result_type_swc: "JSXObject",
            read_conversion: "",
            write_conversion: "",
        }),
        ("JSXMemberExpr", "property") => Some(TypedFieldMapping {
            reluxscript_field: "property",
            swc_field: "prop",
            needs_deref: false,
            result_type_rs: "Identifier",
            result_type_swc: "Ident",
            read_conversion: "",
            write_conversion: "",
        }),

        // TypeScript Interface fields
        ("TsInterfaceDecl", "id") => Some(TypedFieldMapping {
            reluxscript_field: "id",
            swc_field: "id",
            needs_deref: false,
            result_type_rs: "Identifier",
            result_type_swc: "Ident",
            read_conversion: "",
            write_conversion: "",
        }),
        ("TsInterfaceDecl", "body") => Some(TypedFieldMapping {
            reluxscript_field: "body",
            swc_field: "body",
            needs_deref: false,
            result_type_rs: "TsInterfaceBody",
            result_type_swc: "TsInterfaceBody",
            read_conversion: "",
            write_conversion: "",
        }),
        ("TsInterfaceBody", "body") => Some(TypedFieldMapping {
            reluxscript_field: "body",
            swc_field: "body",
            needs_deref: false,
            result_type_rs: "Vec<TsTypeElement>",
            result_type_swc: "Vec<TsTypeElement>",
            read_conversion: "",
            write_conversion: "",
        }),

        // TypeScript PropertySignature fields
        ("TsPropertySignature", "key") => Some(TypedFieldMapping {
            reluxscript_field: "key",
            swc_field: "key",
            needs_deref: true,
            result_type_rs: "Expr",
            result_type_swc: "Expr",
            read_conversion: "",
            write_conversion: "",
        }),

        _ => None,
    }
}

// =============================================================================
// Chain Analysis for Auto-Unwrap
// =============================================================================

/// Represents a single step in a member expression chain
#[derive(Clone, Debug)]
pub struct ChainStep {
    /// The field name in ReluxScript (e.g., "property", "object", "name")
    pub field_name: String,
    /// Type context after this step
    pub result_type: TypeContext,
    /// Whether this step needs enum unwrapping
    pub needs_unwrap: bool,
    /// If needs_unwrap is true, the expected variant (e.g., "Ident" for MemberProp::Ident)
    pub expected_variant: Option<String>,
    /// The SWC enum to match against (e.g., "MemberProp", "Expr")
    pub enum_type: Option<String>,
    /// The SWC struct after unwrapping (e.g., "Ident", "MemberExpr")
    pub unwrapped_struct: Option<String>,
}

/// Analysis of a complete member expression chain
#[derive(Clone, Debug)]
pub struct ChainAnalysis {
    /// The base expression type
    pub base_type: TypeContext,
    /// Steps in the chain
    pub steps: Vec<ChainStep>,
    /// Whether any step needs unwrapping
    pub has_unwraps: bool,
}

impl ChainAnalysis {
    /// Create a new chain analysis
    pub fn new(base_type: TypeContext) -> Self {
        Self {
            base_type,
            steps: Vec::new(),
            has_unwraps: false,
        }
    }

    /// Add a step to the chain
    pub fn add_step(&mut self, step: ChainStep) {
        if step.needs_unwrap {
            self.has_unwraps = true;
        }
        self.steps.push(step);
    }

    /// Get the final type after all steps
    pub fn final_type(&self) -> &TypeContext {
        self.steps.last()
            .map(|s| &s.result_type)
            .unwrap_or(&self.base_type)
    }
}

/// Analyze a member expression chain to determine unwrap requirements
///
/// Given a chain like `member.property.name`, this function:
/// 1. Looks up the base type (member: MemberExpr)
/// 2. For each field access, determines if unwrapping is needed
/// 3. Returns the complete analysis with unwrap points marked
pub fn analyze_member_chain(
    base_type: &TypeContext,
    fields: &[String],
) -> ChainAnalysis {
    let mut analysis = ChainAnalysis::new(base_type.clone());
    let mut current_type = base_type.clone();

    for field in fields {
        // Get field mapping for the current type
        let field_mapping = get_typed_field_mapping(&current_type.swc_type, field);

        if let Some(mapping) = field_mapping {
            // Determine the result type
            let result_swc_type = mapping.result_type_swc;
            let result_kind = classify_swc_type(result_swc_type);

            // Check if this result type needs unwrapping
            let needs_unwrap = matches!(result_kind, SwcTypeKind::WrapperEnum | SwcTypeKind::Enum);

            // If the next field access expects a specific type, we can infer the variant
            let (expected_variant, enum_type, unwrapped_struct) = if needs_unwrap {
                // Look at the next field to infer what variant we expect
                // For now, we'll need the caller to provide this
                // This will be filled in during code generation when we see what field comes next
                (None, Some(result_swc_type.to_string()), None)
            } else {
                (None, None, None)
            };

            let step = ChainStep {
                field_name: field.clone(),
                result_type: TypeContext {
                    reluxscript_type: mapping.result_type_rs.to_string(),
                    swc_type: result_swc_type.to_string(),
                    kind: result_kind.clone(),
                    known_variant: None,
                    needs_deref: mapping.needs_deref,
                },
                needs_unwrap,
                expected_variant,
                enum_type,
                unwrapped_struct,
            };

            current_type = step.result_type.clone();
            analysis.add_step(step);
        } else {
            // Field not found in mappings - create unknown step
            let step = ChainStep {
                field_name: field.clone(),
                result_type: TypeContext::unknown(),
                needs_unwrap: false,
                expected_variant: None,
                enum_type: None,
                unwrapped_struct: None,
            };
            current_type = step.result_type.clone();
            analysis.add_step(step);
        }
    }

    analysis
}

/// Infer the expected variant when accessing a field through a wrapper enum
///
/// For example, when accessing `.name` through `MemberProp`, we know:
/// - Only `MemberProp::Ident` has a `.name` field (actually `.sym`)
/// - So the expected variant is "Ident"
pub fn infer_expected_variant(wrapper_enum: &str, next_field: &str) -> Option<(String, String)> {
    match (wrapper_enum, next_field) {
        // MemberProp.name -> must be MemberProp::Ident
        ("MemberProp", "name") => Some(("Ident".to_string(), "Ident".to_string())),

        // Callee.object, Callee.property -> must be Callee::Expr(Expr::Member)
        ("Callee", "object") | ("Callee", "property") => Some(("Expr".to_string(), "Expr".to_string())),

        // Lit.value -> could be Str, Num, Bool, etc. - need more context
        ("Lit", "value") => None, // Ambiguous without more context

        // PropName.name -> must be PropName::Ident
        ("PropName", "name") => Some(("Ident".to_string(), "Ident".to_string())),

        // JSXElementName patterns
        ("JSXElementName", "name") => Some(("Ident".to_string(), "Ident".to_string())),

        _ => None,
    }
}

/// Generate the unwrap code for a chain step
///
/// Returns (pattern, binding_name) for use in match/if-let
pub fn generate_unwrap_pattern(step: &ChainStep) -> Option<(String, String)> {
    if !step.needs_unwrap {
        return None;
    }

    let enum_type = step.enum_type.as_ref()?;
    let variant = step.expected_variant.as_ref()?;
    let binding = format!("__{}", step.field_name);

    let pattern = format!("{}::{}({})", enum_type, variant, binding);
    Some((pattern, binding))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_type_environment_shadowing() {
        let mut env = TypeEnvironment::new();

        // Define outer variable
        env.define("current", TypeContext::from_reluxscript("Expr"));
        assert_eq!(env.lookup("current").unwrap().swc_type, "Expr");

        // Push scope and shadow with narrowed type
        env.push_scope();
        env.define("current", TypeContext::narrowed("MemberExpression", "MemberExpr"));
        assert_eq!(env.lookup("current").unwrap().swc_type, "MemberExpr");

        // Pop scope, should see outer type again
        env.pop_scope();
        assert_eq!(env.lookup("current").unwrap().swc_type, "Expr");
    }

    #[test]
    fn test_swc_variant_mapping() {
        let (enum_name, variant, struct_name) = get_swc_variant("MemberExpression");
        assert_eq!(enum_name, "Expr");
        assert_eq!(variant, "Member");
        assert_eq!(struct_name, "MemberExpr");
    }

    #[test]
    fn test_classify_swc_type() {
        assert!(matches!(classify_swc_type("Expr"), SwcTypeKind::Enum));
        assert!(matches!(classify_swc_type("MemberExpr"), SwcTypeKind::Struct));
        assert!(matches!(classify_swc_type("MemberProp"), SwcTypeKind::WrapperEnum));
        assert!(matches!(classify_swc_type("JsWord"), SwcTypeKind::Atom));
    }

    #[test]
    fn test_lit_context_patterns() {
        // Test Lit context mapping
        let (enum_name, variant, struct_name) = get_swc_variant_in_context("StringLiteral", "Lit");
        assert_eq!(enum_name, "Lit");
        assert_eq!(variant, "Str");
        assert_eq!(struct_name, "Str");

        let (enum_name, variant, struct_name) = get_swc_variant_in_context("NumericLiteral", "Lit");
        assert_eq!(enum_name, "Lit");
        assert_eq!(variant, "Num");
        assert_eq!(struct_name, "Number");

        let (enum_name, variant, struct_name) = get_swc_variant_in_context("BooleanLiteral", "Lit");
        assert_eq!(enum_name, "Lit");
        assert_eq!(variant, "Bool");
        assert_eq!(struct_name, "Bool");
    }

    #[test]
    fn test_pat_context_patterns() {
        // Test Pat context mapping
        let (enum_name, variant, struct_name) = get_swc_variant_in_context("Identifier", "Pat");
        assert_eq!(enum_name, "Pat");
        assert_eq!(variant, "Ident");
        assert_eq!(struct_name, "BindingIdent");

        let (enum_name, variant, struct_name) = get_swc_variant_in_context("ArrayPattern", "Pat");
        assert_eq!(enum_name, "Pat");
        assert_eq!(variant, "Array");
        assert_eq!(struct_name, "ArrayPat");

        let (enum_name, variant, struct_name) = get_swc_variant_in_context("ObjectPattern", "Pat");
        assert_eq!(enum_name, "Pat");
        assert_eq!(variant, "Object");
        assert_eq!(struct_name, "ObjectPat");
    }

    #[test]
    fn test_prop_name_context_patterns() {
        // Test PropName context mapping
        let (enum_name, variant, struct_name) = get_swc_variant_in_context("Identifier", "PropName");
        assert_eq!(enum_name, "PropName");
        assert_eq!(variant, "Ident");
        assert_eq!(struct_name, "Ident");

        let (enum_name, variant, struct_name) = get_swc_variant_in_context("StringLiteral", "PropName");
        assert_eq!(enum_name, "PropName");
        assert_eq!(variant, "Str");
        assert_eq!(struct_name, "Str");
    }

    #[test]
    fn test_jsx_context_patterns() {
        // Test JSXObject context mapping
        let (enum_name, variant, struct_name) = get_swc_variant_in_context("Identifier", "JSXObject");
        assert_eq!(enum_name, "JSXObject");
        assert_eq!(variant, "Ident");
        assert_eq!(struct_name, "Ident");

        let (enum_name, variant, struct_name) = get_swc_variant_in_context("JSXMemberExpression", "JSXObject");
        assert_eq!(enum_name, "JSXObject");
        assert_eq!(variant, "JSXMemberExpr");
        assert_eq!(struct_name, "JSXMemberExpr");

        // Test JSXAttrValue context
        let (enum_name, variant, struct_name) = get_swc_variant_in_context("StringLiteral", "JSXAttrValue");
        assert_eq!(enum_name, "JSXAttrValue");
        assert_eq!(variant, "Lit");
        assert_eq!(struct_name, "Lit");

        let (enum_name, variant, struct_name) = get_swc_variant_in_context("JSXExpressionContainer", "JSXAttrValue");
        assert_eq!(enum_name, "JSXAttrValue");
        assert_eq!(variant, "JSXExprContainer");
        assert_eq!(struct_name, "JSXExprContainer");
    }

    #[test]
    fn test_field_mappings_literals() {
        // Test literal field mappings
        let mapping = get_typed_field_mapping("Str", "value").unwrap();
        assert_eq!(mapping.swc_field, "value");
        assert_eq!(mapping.result_type_swc, "JsWord");

        let mapping = get_typed_field_mapping("Number", "value").unwrap();
        assert_eq!(mapping.swc_field, "value");
        assert_eq!(mapping.result_type_swc, "f64");
    }

    #[test]
    fn test_field_mappings_jsx() {
        // Test JSX field mappings
        let mapping = get_typed_field_mapping("JSXElement", "openingElement").unwrap();
        assert_eq!(mapping.swc_field, "opening");

        let mapping = get_typed_field_mapping("JSXOpeningElement", "attributes").unwrap();
        assert_eq!(mapping.swc_field, "attrs");
        assert_eq!(mapping.result_type_swc, "Vec<JSXAttrOrSpread>");
    }

    #[test]
    fn test_chain_analysis_simple() {
        // Test simple chain: member.property
        let base = TypeContext::narrowed("MemberExpression", "MemberExpr");
        let analysis = analyze_member_chain(&base, &["property".to_string()]);

        assert_eq!(analysis.steps.len(), 1);
        assert!(analysis.has_unwraps); // MemberProp is a wrapper enum
        assert_eq!(analysis.steps[0].result_type.swc_type, "MemberProp");
    }

    #[test]
    fn test_chain_analysis_nested() {
        // Test nested chain: member.object
        let base = TypeContext::narrowed("MemberExpression", "MemberExpr");
        let analysis = analyze_member_chain(&base, &["object".to_string()]);

        assert_eq!(analysis.steps.len(), 1);
        assert!(analysis.has_unwraps); // Expr is an enum
        assert_eq!(analysis.steps[0].result_type.swc_type, "Expr");
        assert!(analysis.steps[0].result_type.needs_deref); // Box<Expr>
    }

    #[test]
    fn test_infer_expected_variant() {
        // MemberProp.name -> Ident
        let result = infer_expected_variant("MemberProp", "name");
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "Ident");

        // PropName.name -> Ident
        let result = infer_expected_variant("PropName", "name");
        assert!(result.is_some());
        assert_eq!(result.unwrap().0, "Ident");

        // Unknown -> None
        let result = infer_expected_variant("Unknown", "foo");
        assert!(result.is_none());
    }
}
