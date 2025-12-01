//! SWC Rewriter - Transforms decorated AST to prepare it for dumb codegen
//!
//! This stage sits between decoration and codegen:
//! 1. Receives decorated AST with metadata
//! 2. Applies structural transformations (desugaring, unwrapping, replacements)
//! 3. Returns transformed decorated AST ready for emission
//!
//! Example transformations:
//! - Pattern desugaring: Callee::MemberExpression → nested if-let
//! - Member unwrapping: node.callee.name → match chain
//! - Field replacements: self.builder → self (in writers)
//! - Matches! expansion: matches!(expr, pat) → if-let
//!
//! The key principle: **Codegen receives ready-to-emit AST with no decisions to make**

use crate::parser::*;
use crate::lexer::Span;
use super::decorated_ast::*;
use super::swc_metadata::*;
use crate::type_system::SwcTypeKind;
use super::swc_decorator::{DecoratedProgram, DecoratedTopLevelDecl, DecoratedPlugin, DecoratedWriter, DecoratedPluginItem, DecoratedFnDecl, DecoratedImplBlock};

/// SwcRewriter transforms DecoratedAST → DecoratedAST
/// All semantic transformations happen here, not in codegen
pub struct SwcRewriter {
    /// Counter for generating unique temporary variable names
    temp_var_counter: usize,

    /// Whether we're in a writer context (affects self.builder → self)
    is_writer: bool,

    /// Helper function names (non-visitor functions) in current plugin/writer
    helper_functions: Vec<String>,
}

impl SwcRewriter {
    /// Create new rewriter
    pub fn new() -> Self {
        Self {
            temp_var_counter: 0,
            is_writer: false,
            helper_functions: Vec::new(),
        }
    }

    /// Create new rewriter for writer context
    pub fn new_writer() -> Self {
        Self {
            temp_var_counter: 0,
            is_writer: true,
            helper_functions: Vec::new(),
        }
    }

    /// Main entry point: rewrite entire program
    pub fn rewrite_program(&mut self, program: DecoratedProgram) -> DecoratedProgram {
        DecoratedProgram {
            uses: program.uses,
            decl: self.rewrite_top_level_decl(program.decl),
        }
    }

    // ========================================================================
    // TOP-LEVEL DECLARATIONS
    // ========================================================================

    fn rewrite_top_level_decl(&mut self, decl: DecoratedTopLevelDecl) -> DecoratedTopLevelDecl {
        match decl {
            DecoratedTopLevelDecl::Plugin(plugin) => {
                self.is_writer = false;
                DecoratedTopLevelDecl::Plugin(self.rewrite_plugin(plugin))
            }
            DecoratedTopLevelDecl::Writer(writer) => {
                self.is_writer = true;
                DecoratedTopLevelDecl::Writer(self.rewrite_writer(writer))
            }
            DecoratedTopLevelDecl::Undecorated(decl) => {
                // Pass through undecorated nodes unchanged
                DecoratedTopLevelDecl::Undecorated(decl)
            }
        }
    }

    fn rewrite_plugin(&mut self, plugin: DecoratedPlugin) -> DecoratedPlugin {
        // First pass: collect helper function names
        self.helper_functions.clear();
        for item in &plugin.body {
            if let DecoratedPluginItem::Function(func) = item {
                if !func.name.starts_with("visit_") && !func.name.starts_with("visit_mut_") {
                    self.helper_functions.push(func.name.clone());
                }
            }
        }

        // Second pass: rewrite with helper function knowledge
        DecoratedPlugin {
            name: plugin.name,
            body: plugin.body
                .into_iter()
                .map(|item| self.rewrite_plugin_item(item))
                .collect(),
        }
    }

    fn rewrite_writer(&mut self, writer: DecoratedWriter) -> DecoratedWriter {
        // First pass: collect helper function names
        self.helper_functions.clear();
        for item in &writer.body {
            if let DecoratedPluginItem::Function(func) = item {
                // All functions in writers are helpers (no visit methods)
                self.helper_functions.push(func.name.clone());
            }
        }

        // Second pass: rewrite with helper function knowledge
        DecoratedWriter {
            name: writer.name,
            body: writer.body
                .into_iter()
                .map(|item| self.rewrite_plugin_item(item))
                .collect(),
            hoisted_structs: writer.hoisted_structs,
            state_struct: writer.state_struct,
        }
    }

    fn rewrite_plugin_item(&mut self, item: DecoratedPluginItem) -> DecoratedPluginItem {
        match item {
            DecoratedPluginItem::Function(func) => {
                DecoratedPluginItem::Function(self.rewrite_fn_decl(func))
            }
            DecoratedPluginItem::Struct(struct_decl) => {
                // Structs don't need rewriting
                DecoratedPluginItem::Struct(struct_decl)
            }
            DecoratedPluginItem::Enum(enum_decl) => {
                // Enums don't need rewriting
                DecoratedPluginItem::Enum(enum_decl)
            }
            DecoratedPluginItem::Impl(impl_block) => {
                DecoratedPluginItem::Impl(self.rewrite_impl_block(impl_block))
            }
            DecoratedPluginItem::PreHook(func) => {
                DecoratedPluginItem::PreHook(self.rewrite_fn_decl(func))
            }
            DecoratedPluginItem::ExitHook(func) => {
                DecoratedPluginItem::ExitHook(self.rewrite_fn_decl(func))
            }
        }
    }

    fn rewrite_fn_decl(&mut self, func: DecoratedFnDecl) -> DecoratedFnDecl {
        DecoratedFnDecl {
            name: func.name,
            params: func.params,
            return_type: func.return_type,
            body: self.rewrite_block(func.body),
        }
    }

    fn rewrite_impl_block(&mut self, impl_block: DecoratedImplBlock) -> DecoratedImplBlock {
        DecoratedImplBlock {
            target: impl_block.target,
            lifetimes: impl_block.lifetimes,
            items: impl_block.items
                .into_iter()
                .map(|m| self.rewrite_fn_decl(m))
                .collect(),
        }
    }

    // ========================================================================
    // BLOCKS AND STATEMENTS
    // ========================================================================

    fn rewrite_block(&mut self, block: DecoratedBlock) -> DecoratedBlock {
        let mut result_stmts = Vec::new();

        for stmt in block.stmts {
            let rewritten = self.rewrite_stmt(stmt.clone());
            result_stmts.push(rewritten);

            // Detect early-return guard: if !matches!(x, Some) { return; }
            // Insert unwrap: let x = x.as_ref().unwrap();
            // Or for Pat types, insert destructuring: let Pat::Array(x) = x else { return; };
            if let DecoratedStmt::If(ref if_stmt) = stmt {
                if let Some((var_name, var_type)) = Self::extract_option_guard_variable(if_stmt) {
                    // Only insert unwrap rebinding for simple identifiers, not member expressions
                    // Member expressions like "call.callee" can't be used as let patterns
                    let is_pat_type = var_type.contains("Pat") || var_type == "Ident";

                    if !var_name.contains('.') {
                        if is_pat_type {
                            // For Pat types, extract the variant from the matches! pattern
                            if let Some(pat_variant) = Self::extract_pat_variant_from_guard(if_stmt) {
                                eprintln!("[REWRITER] Detected Pat guard for '{}' (variant: {}), inserting destructuring", var_name, pat_variant);
                                // Create: let Pat::Array(var_name) = var_name else { return; };
                                let destructure_stmt = Self::create_pat_destructuring(&var_name, &pat_variant);
                                result_stmts.push(destructure_stmt);
                            } else {
                                eprintln!("[REWRITER] Skipping Pat destructuring for '{}' (couldn't extract variant)", var_name);
                            }
                        } else {
                            eprintln!("[REWRITER] Detected Option guard for '{}' (type: {}), inserting unwrap", var_name, var_type);
                            // Create: let var_name = var_name.as_ref().unwrap();
                            let unwrap_stmt = Self::create_unwrap_rebinding(&var_name);
                            result_stmts.push(unwrap_stmt);
                        }
                    } else {
                        eprintln!("[REWRITER] Skipping rebinding for '{}' (member expression)", var_name);
                    }
                }
            }
        }

        DecoratedBlock {
            stmts: result_stmts,
        }
    }

    fn rewrite_stmt(&mut self, stmt: DecoratedStmt) -> DecoratedStmt {
        match stmt {
            DecoratedStmt::Let(let_stmt) => {
                let init = self.rewrite_expr(let_stmt.init);
                // Apply auto-unwrap to the init expression if it contains narrowed identifiers
                let init = self.apply_auto_unwrap(init);

                DecoratedStmt::Let(DecoratedLetStmt {
                    mutable: let_stmt.mutable,
                    pattern: self.rewrite_pattern(let_stmt.pattern),
                    ty: let_stmt.ty,
                    init,
                })
            }

            DecoratedStmt::Const(const_stmt) => {
                DecoratedStmt::Const(DecoratedConstStmt {
                    name: const_stmt.name,
                    ty: const_stmt.ty,
                    init: self.rewrite_expr(const_stmt.init),
                })
            }

            DecoratedStmt::Expr(expr) => {
                DecoratedStmt::Expr(self.rewrite_expr(expr))
            }

            DecoratedStmt::If(if_stmt) => {
                DecoratedStmt::If(self.rewrite_if_stmt(if_stmt))
            }

            DecoratedStmt::Match(match_stmt) => {
                DecoratedStmt::Match(DecoratedMatchStmt {
                    expr: self.rewrite_expr(match_stmt.expr),
                    arms: match_stmt.arms
                        .into_iter()
                        .map(|arm| self.rewrite_match_arm(arm))
                        .collect(),
                })
            }

            DecoratedStmt::For(for_stmt) => {
                DecoratedStmt::For(DecoratedForStmt {
                    pattern: self.rewrite_pattern(for_stmt.pattern),
                    iter: self.rewrite_expr(for_stmt.iter),
                    body: self.rewrite_block(for_stmt.body),
                })
            }

            DecoratedStmt::While(while_stmt) => {
                DecoratedStmt::While(DecoratedWhileStmt {
                    condition: self.rewrite_expr(while_stmt.condition),
                    body: self.rewrite_block(while_stmt.body),
                })
            }

            DecoratedStmt::Loop(loop_block) => {
                DecoratedStmt::Loop(self.rewrite_block(loop_block))
            }

            DecoratedStmt::Return(ret_expr) => {
                let rewritten = ret_expr.map(|e| {
                    let expr = self.rewrite_expr(e);
                    // Check if it's a string literal that needs conversion
                    if let DecoratedExprKind::Literal(Literal::String(_)) = expr.kind {
                        self.wrap_with_to_string(expr)
                    } else {
                        expr
                    }
                });
                DecoratedStmt::Return(rewritten)
            }

            DecoratedStmt::Break => DecoratedStmt::Break,

            DecoratedStmt::Continue => DecoratedStmt::Continue,

            DecoratedStmt::Traverse(traverse) => {
                // Rewrite traverse block methods to expand matches! etc.
                let rewritten_traverse = match traverse.kind {
                    crate::codegen::decorated_ast::DecoratedTraverseKind::Inline(inline) => {
                        let mut rewritten_methods = Vec::new();
                        for method in &inline.methods {
                            let rewritten_body = self.rewrite_block(method.body.clone());
                            rewritten_methods.push(crate::codegen::decorated_ast::DecoratedVisitorMethod {
                                name: method.name.clone(),
                                params: method.params.clone(),
                                body: rewritten_body,
                            });
                        }
                        crate::codegen::decorated_ast::DecoratedTraverseStmt {
                            kind: crate::codegen::decorated_ast::DecoratedTraverseKind::Inline(
                                crate::codegen::decorated_ast::DecoratedInlineVisitor {
                                    state: inline.state.clone(),
                                    methods: rewritten_methods,
                                }
                            ),
                            target: traverse.target.clone(),
                            captures: traverse.captures.clone(),
                            span: traverse.span,
                        }
                    }
                    other => crate::codegen::decorated_ast::DecoratedTraverseStmt {
                        kind: other,
                        target: traverse.target.clone(),
                        captures: traverse.captures.clone(),
                        span: traverse.span,
                    },
                };
                DecoratedStmt::Traverse(Box::new(rewritten_traverse))
            }

            DecoratedStmt::Function(func_decl) => {
                // Function declarations don't need rewriting at this level
                DecoratedStmt::Function(func_decl)
            }

            DecoratedStmt::Verbatim(verbatim) => {
                // Verbatim code passes through unchanged
                DecoratedStmt::Verbatim(verbatim)
            }

            DecoratedStmt::CustomPropAssignment(assign) => {
                self.rewrite_custom_prop_assignment(*assign)
            }
        }
    }

    /// 🔥 CRITICAL: Rewrite if-statements (handles pattern desugaring)
    fn rewrite_if_stmt(&mut self, mut if_stmt: DecoratedIfStmt) -> DecoratedIfStmt {
        eprintln!("[DEBUG SHADOWING] rewrite_if_stmt called, pattern.is_none() = {}", if_stmt.pattern.is_none());

        // 🌟 PROBABILITY FIELD COLLAPSE: Convert `if matches!(expr, Pattern)` to `if let Pattern(expr) = expr`
        if if_stmt.pattern.is_none() {
            eprintln!("[DEBUG SHADOWING] Checking if condition for matches!");
            // Clone the condition to inspect it without moving
            if let DecoratedExprKind::Matches { expr: scrutinee, pattern } = if_stmt.condition.clone().kind {
                eprintln!("[DEBUG SHADOWING] Found matches! in condition");
                // Extract the variable name from the scrutinee
                if let DecoratedExprKind::Ident { name, .. } = &scrutinee.kind {
                    eprintln!("[DEBUG SHADOWING] Scrutinee is identifier: {} with type: {}", name, scrutinee.metadata.swc_type);
                    // Transform: if matches!(expr, StringLiteral)
                    // Into:      if let Expr::Lit(Lit::Str(expr)) = expr
                    //
                    // This shadows `expr` with the unwrapped variant!

                    // Create a binding pattern that shadows the original variable
                    let shadow_binding = DecoratedPattern {
                        kind: DecoratedPatternKind::Ident(name.clone()),
                        metadata: SwcPatternMetadata::direct(name.clone()),
                    };

                    // Wrap the variant pattern to include the shadow binding
                    let shadowing_pattern = self.wrap_pattern_with_binding(pattern, shadow_binding);

                    eprintln!("[DEBUG SHADOWING] Pattern after wrapping: {:?}", shadowing_pattern.metadata.swc_pattern);

                    // Convert to if-let
                    // Wrap scrutinee in & to match by reference
                    let scrutinee_type = scrutinee.metadata.swc_type.clone();
                    eprintln!("[DEBUG SHADOWING] Using scrutinee_type: {}", scrutinee_type);
                    let scrutinee_span = scrutinee.metadata.span;
                    let ref_condition = DecoratedExpr {
                        kind: DecoratedExprKind::Ref {
                            expr: scrutinee,
                            mutable: false,
                        },
                        metadata: SwcExprMetadata { needs_enum_unwrap: None,
                            swc_type: format!("&{}", scrutinee_type),
                            is_boxed: false,
                            is_optional: false,
                            type_kind: crate::type_system::SwcTypeKind::Unknown,
                            span: scrutinee_span,
                        },
                    };

                    if_stmt.pattern = Some(shadowing_pattern);
                    if_stmt.condition = ref_condition;

                    eprintln!("[DEBUG SHADOWING] Transformation applied! Pattern set.");
                }
            }
        }

        // Strip read_conversion from if-let conditions BEFORE any processing
        if if_stmt.pattern.is_some() {
            if_stmt.condition = self.strip_read_conversion_for_pattern_match(if_stmt.condition);
        }

        // Check if this is an if-let with a pattern that needs desugaring
        if let Some(ref pattern) = if_stmt.pattern {
            if pattern.metadata.needs_desugaring() {
                // 🔥 DESUGAR THE ENTIRE IF-STATEMENT!
                return self.desugar_if_let_stmt(if_stmt);
            }
        }

        // No desugaring needed - normal rewriting
        // Strip unnecessary deref from if-let conditions
        let mut condition = self.rewrite_expr(if_stmt.condition);
        if if_stmt.pattern.is_some() {
            condition = self.strip_unnecessary_deref(condition);
        }

        let pattern = if_stmt.pattern.as_ref().map(|p| self.rewrite_pattern(p.clone()));

        // Extract scrutinee name BEFORE transforming condition
        let scrutinee_name = match &condition.kind {
            DecoratedExprKind::Ident { ref name, .. } => Some(name.clone()),
            DecoratedExprKind::Member { ref object, ref property, .. } => {
                // Handle member expressions like node.expr
                if let DecoratedExprKind::Ident { ref name, .. } = object.kind {
                    Some(format!("{}.{}", name, property))
                } else {
                    None
                }
            }
            _ => None,
        };

        // Add .as_ref() to scrutinee if matching against Box<T>
        if let Some(ref pat) = pattern {
            condition = self.add_asref_for_box_match(condition, pat);
        }

        // Rewrite then branch, potentially replacing scrutinee with binding
        let then_branch = if let (Some(ref pat), Some(ref name)) = (&pattern, &scrutinee_name) {
            // Extract the binding name from the pattern (e.g., __inner from Expr::Lit(Lit::Str(__inner)))
            if let Some(binding_name) = self.extract_innermost_binding(pat) {
                // Extract the binding type from the pattern (e.g., Str from Expr::Lit(Lit::Str(__inner)))
                let binding_type = self.extract_binding_type_from_pattern(pat);
                eprintln!("[REWRITER] If-let: scrutinee '{}' -> binding '{}' (type: {})", name, binding_name, binding_type);
                // Rewrite the block, replacing scrutinee with binding
                self.rewrite_block_with_scrutinee_replacement_typed(if_stmt.then_branch, name, &binding_name, &binding_type)
            } else {
                eprintln!("[REWRITER] If-let: No binding found in pattern");
                self.rewrite_block(if_stmt.then_branch)
            }
        } else {
            // No scrutinee name or pattern, just rewrite normally
            self.rewrite_block(if_stmt.then_branch)
        };
        let then_branch = self.convert_block_tail_string_literal(then_branch);
        let else_branch = if_stmt.else_branch.map(|b| self.rewrite_block(b));
        let else_branch = else_branch.map(|b| self.convert_block_tail_string_literal(b));

        DecoratedIfStmt {
            condition,
            pattern,
            then_branch,
            else_branch,
            if_let_metadata: if_stmt.if_let_metadata,
        }
    }

    /// Strip unnecessary deref (*) from expressions that return references
    /// For example: *member.obj.as_ref() → member.obj.as_ref()
    fn strip_unnecessary_deref(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        match expr.kind {
            DecoratedExprKind::Unary { op: crate::parser::UnaryOp::Deref, operand, unary_metadata } => {
                // Check if the inner expression returns a reference
                // If it's a method call ending in .as_ref(), it returns &T, so we don't need *
                if self.returns_reference(&operand) {
                    *operand
                } else {
                    // Keep the deref
                    DecoratedExpr {
                        kind: DecoratedExprKind::Unary {
                            op: crate::parser::UnaryOp::Deref,
                            operand,
                            unary_metadata,
                        },
                        metadata: expr.metadata,
                    }
                }
            }
            _ => expr,
        }
    }

    /// Strip read_conversion from member expressions when used in pattern matching
    /// Example: call.callee.as_expr().unwrap() → call.callee
    fn strip_read_conversion_for_pattern_match(&self, expr: DecoratedExpr) -> DecoratedExpr {
        match expr.kind {
            // Check for member access with read_conversion
            DecoratedExprKind::Member { object, property, optional, computed, is_path, mut field_metadata } => {
                // Clear read_conversion when used as pattern match target
                field_metadata.read_conversion = String::new();
                DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object,
                        property,
                        optional,
                        computed,
                        is_path,
                        field_metadata,
                    },
                    metadata: expr.metadata,
                }
            }

            // Recursively strip from unary expressions (e.g., &call.callee.as_expr())
            DecoratedExprKind::Unary { op, operand, unary_metadata } => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Unary {
                        op,
                        operand: Box::new(self.strip_read_conversion_for_pattern_match(*operand)),
                        unary_metadata,
                    },
                    metadata: expr.metadata,
                }
            }

            // Recursively strip from ref expressions
            DecoratedExprKind::Ref { mutable, expr: inner } => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Ref {
                        mutable,
                        expr: Box::new(self.strip_read_conversion_for_pattern_match(*inner)),
                    },
                    metadata: expr.metadata,
                }
            }

            _ => expr,
        }
    }

    /// Check if an expression returns a reference or is a direct enum access
    fn returns_reference(&self, expr: &DecoratedExpr) -> bool {
        match &expr.kind {
            DecoratedExprKind::Call(call) => {
                // Check if it's a call to .as_ref()
                if let DecoratedExprKind::Member { property, .. } = &call.callee.kind {
                    property == "as_ref"
                } else {
                    false
                }
            }
            DecoratedExprKind::Member { field_metadata, .. } => {
                // Check if the accessor returns a reference OR if it's a direct enum field
                // For example: member.prop (MemberProp enum) doesn't need *
                matches!(field_metadata.accessor,
                    FieldAccessor::BoxedAsRef |
                    FieldAccessor::Direct |
                    FieldAccessor::EnumField { .. })
            }
            _ => false,
        }
    }

    /// Add .as_ref() to scrutinee when matching &Box<T> against T pattern
    /// Example: if let Expr::Array(arr) = init  →  if let Expr::Array(arr) = init.as_ref()
    fn add_asref_for_box_match(&self, scrutinee: DecoratedExpr, pattern: &DecoratedPattern) -> DecoratedExpr {
        // Check if scrutinee is an identifier with Box (check is_boxed flag, not string)
        // This handles both explicit Box<T> types and narrowed types that are still boxed
        let is_ident_with_box = matches!(&scrutinee.kind, DecoratedExprKind::Ident { .. })
            && scrutinee.metadata.is_boxed;

        // Check if pattern is a variant pattern (like Expr::Array)
        let is_variant_pattern = matches!(&pattern.kind, DecoratedPatternKind::Variant { .. });

        if is_ident_with_box && is_variant_pattern {
            // Wrap scrutinee with .as_ref() call
            DecoratedExpr {
                kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                    callee: DecoratedExpr {
                        kind: DecoratedExprKind::Member {
                            object: Box::new(scrutinee.clone()),
                            property: "as_ref".to_string(),
                            optional: false,
                            computed: false,
                            is_path: false,
                            field_metadata: SwcFieldMetadata::direct("as_ref".to_string(), "fn".to_string()),
                        },
                        metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                            swc_type: "fn".to_string(),
                            is_boxed: false,
                            is_optional: false,
                            type_kind: SwcTypeKind::Unknown,
                            span: None,
                        },
                    },
                    args: vec![],
                    type_args: vec![],
                    optional: false,
                    is_macro: false,
                    span: Span::new(0, 0, 0, 0),
                })),
                metadata: scrutinee.metadata,
            }
        } else {
            // No transformation needed
            scrutinee
        }
    }

    /// 🔧 DESUGAR IF-LET STATEMENT with nested pattern
    /// Transforms: if let Callee::MemberExpression(member) = node.callee { body }
    /// Into: if let Callee::Expr(__expr) = &node.callee { if let Expr::Member(member) = __expr.as_ref() { body } }
    fn desugar_if_let_stmt(&mut self, if_stmt: DecoratedIfStmt) -> DecoratedIfStmt {
        use super::swc_metadata::DesugarStrategy;

        // Destructure all fields at once to avoid partial move
        let DecoratedIfStmt {
            condition,
            pattern,
            then_branch,
            else_branch,
            if_let_metadata: _,
        } = if_stmt;

        let pattern = pattern.unwrap(); // Safe: we checked needs_desugaring()

        if let Some(DesugarStrategy::NestedIfLet {
            outer_pattern,
            outer_binding,
            inner_pattern,
            inner_binding,
            unwrap_expr,
        }) = &pattern.metadata.desugar_strategy {
            // Build the OUTER if-let: if let Callee::Expr(__callee_expr) = &node.callee
            let outer_pattern = DecoratedPattern {
                kind: DecoratedPatternKind::Variant {
                    name: outer_pattern.clone(),
                    inner: Some(Box::new(DecoratedPattern {
                        kind: DecoratedPatternKind::Ident(outer_binding.clone()),
                        metadata: SwcPatternMetadata::direct(outer_binding.clone()),
                    })),
                },
                metadata: SwcPatternMetadata::direct(format!("{}({})", outer_pattern, outer_binding)),
            };

            // Build the INNER if-let: if let Expr::Member(member) = __callee_expr.as_ref()
            let inner_condition = DecoratedExpr {
                kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                    callee: DecoratedExpr {
                        kind: DecoratedExprKind::Member {
                            object: Box::new(DecoratedExpr {
                                kind: DecoratedExprKind::Ident {
                                    name: outer_binding.clone(),
                                    ident_metadata: SwcIdentifierMetadata::name(),
                                },
                                metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                    swc_type: "Box<Expr>".to_string(),
                                    is_boxed: true,
                                    is_optional: false,
                                    type_kind: crate::type_system::SwcTypeKind::WrapperEnum,
                                    span: None,
                                },
                            }),
                            property: "as_ref".to_string(),
                            optional: false,
                            computed: false,
                            is_path: false,
                            field_metadata: SwcFieldMetadata::direct("as_ref".to_string(), "fn".to_string()),
                        },
                        metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                            swc_type: "fn".to_string(),
                            is_boxed: false,
                            is_optional: false,
                            type_kind: crate::type_system::SwcTypeKind::Unknown,
                            span: None,
                        },
                    },
                    args: vec![],
                    type_args: vec![],
                    optional: false,
                    is_macro: false,
                    span: crate::lexer::Span::new(0, 0, 0, 0),
                })),
                metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                    swc_type: "&Expr".to_string(),
                    is_boxed: false,
                    is_optional: false,
                    type_kind: crate::type_system::SwcTypeKind::Unknown,
                    span: None,
                },
            };

            let inner_pattern = DecoratedPattern {
                kind: DecoratedPatternKind::Variant {
                    name: inner_pattern.clone(),
                    inner: Some(Box::new(DecoratedPattern {
                        kind: DecoratedPatternKind::Ident(inner_binding.clone()),
                        metadata: SwcPatternMetadata::direct(inner_binding.clone()),
                    })),
                },
                metadata: SwcPatternMetadata::direct(format!("{}({})", inner_pattern, inner_binding)),
            };

            // Build the inner if-let statement
            let inner_if_stmt = DecoratedIfStmt {
                condition: inner_condition,
                pattern: Some(inner_pattern),
                then_branch: self.rewrite_block(then_branch),
                else_branch: else_branch.map(|b| self.rewrite_block(b)),
                if_let_metadata: None,
            };

            // Wrap inner if-let in outer if-let's then branch
            let outer_then_branch = DecoratedBlock {
                stmts: vec![DecoratedStmt::If(inner_if_stmt)],
            };

            // Build the outer if-let: if let Callee::Expr(__callee_expr) = &node.callee
            // Ensure the condition is wrapped in a Ref if it's not already one
            let rewritten_condition = self.rewrite_expr(condition);
            let ref_condition = if matches!(rewritten_condition.kind, DecoratedExprKind::Unary { op: crate::parser::UnaryOp::Ref, .. }) {
                // Already a reference, use as-is
                rewritten_condition
            } else {
                // Need to wrap in &
                DecoratedExpr {
                    kind: DecoratedExprKind::Unary {
                        op: crate::parser::UnaryOp::Ref,
                        operand: Box::new(rewritten_condition.clone()),
                        unary_metadata: crate::codegen::swc_metadata::SwcUnaryMetadata {
                            override_op: None,
                            span: None,
                        },
                    },
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: format!("&{}", rewritten_condition.metadata.swc_type),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                        span: None,
                    },
                }
            };

            DecoratedIfStmt {
                condition: ref_condition,
                pattern: Some(outer_pattern),
                then_branch: outer_then_branch,
                else_branch: None, // Else goes on inner if-let, not outer
                if_let_metadata: None,
            }
        } else {
            // No desugaring strategy, shouldn't reach here - return a dummy
            DecoratedIfStmt {
                condition: self.rewrite_expr(condition),
                pattern: Some(pattern),
                then_branch: self.rewrite_block(then_branch),
                else_branch: else_branch.map(|b| self.rewrite_block(b)),
                if_let_metadata: None,
            }
        }
    }

    fn rewrite_match_arm(&mut self, arm: DecoratedMatchArm) -> DecoratedMatchArm {
        DecoratedMatchArm {
            pattern: self.rewrite_pattern(arm.pattern),
            guard: arm.guard.map(|g| self.rewrite_expr(g)),
            body: self.rewrite_block(arm.body),
        }
    }

    // ========================================================================
    // PATTERNS (Desugaring happens here!)
    // ========================================================================

    /// 🎯 PATTERN REWRITING - Just recursively rewrite children
    /// NOTE: Pattern desugaring is handled at the if-statement level (desugar_if_let_stmt)
    fn rewrite_pattern(&mut self, pattern: DecoratedPattern) -> DecoratedPattern {
        let kind = match pattern.kind {
            DecoratedPatternKind::Variant { name, inner } => {
                DecoratedPatternKind::Variant {
                    name,
                    inner: inner.map(|p| Box::new(self.rewrite_pattern(*p))),
                }
            }

            DecoratedPatternKind::Tuple(patterns) => {
                DecoratedPatternKind::Tuple(
                    patterns.into_iter()
                        .map(|p| self.rewrite_pattern(p))
                        .collect()
                )
            }

            DecoratedPatternKind::Struct { name, fields } => {
                DecoratedPatternKind::Struct {
                    name,
                    fields: fields.into_iter()
                        .map(|(fname, fpat)| (fname, self.rewrite_pattern(fpat)))
                        .collect(),
                }
            }

            DecoratedPatternKind::Array(patterns) => {
                DecoratedPatternKind::Array(
                    patterns.into_iter()
                        .map(|p| self.rewrite_pattern(p))
                        .collect()
                )
            }

            DecoratedPatternKind::Rest(inner) => {
                DecoratedPatternKind::Rest(Box::new(self.rewrite_pattern(*inner)))
            }

            DecoratedPatternKind::Or(patterns) => {
                DecoratedPatternKind::Or(
                    patterns.into_iter()
                        .map(|p| self.rewrite_pattern(p))
                        .collect()
                )
            }

            DecoratedPatternKind::Ref { is_mut, pattern: inner } => {
                DecoratedPatternKind::Ref {
                    is_mut,
                    pattern: Box::new(self.rewrite_pattern(*inner)),
                }
            }

            // Leaf patterns that don't need rewriting
            DecoratedPatternKind::Literal(_) |
            DecoratedPatternKind::Ident(_) |
            DecoratedPatternKind::Wildcard |
            DecoratedPatternKind::Object(_) => {
                pattern.kind
            }
        };

        DecoratedPattern {
            kind,
            metadata: pattern.metadata,
        }
    }

    // ========================================================================
    // EXPRESSIONS (All transformations happen here!)
    // ========================================================================

    /// Main expression rewriter - applies ALL transformations
    fn rewrite_expr(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // First, recursively rewrite children (bottom-up)
        let expr = self.rewrite_expr_children(expr);

        // Then apply transformations (top-down)
        let expr = self.apply_field_replacements(expr);
        let expr = self.apply_context_remove(expr);
        let expr = self.apply_codegen_helpers(expr);
        let expr = self.apply_helper_function_calls(expr);
        let expr = self.apply_field_conversions(expr);
        let expr = self.apply_member_prop_to_string(expr);  // NEW: MemberProp → String
        let expr = self.apply_visit_children_rewrite(expr);
        let expr = self.apply_atom_to_string_conversion(expr);
        let expr = self.apply_ast_struct_init(expr);
        let expr = self.apply_matches_expansion(expr);  // Expand matches! first
        // NOTE: Auto-unwrap is applied selectively in Let statements, not here
        let expr = self.apply_iterator_methods(expr);
        let expr = self.apply_string_literal_conversion(expr);
        // TODO Phase 4: Apply nested member unwrapping
        // let expr = self.apply_member_unwrapping(expr);

        expr
    }

    /// Recursively rewrite expression children
    fn rewrite_expr_children(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // First check if this is a Deref that should be transformed
        let expr = match expr.kind {
            DecoratedExprKind::Unary { op: crate::parser::UnaryOp::Deref, operand, unary_metadata } => {
                // Check if the operand returns a reference
                if self.returns_reference(&operand) {
                    // Check if it's a .as_ref() call - if so, just strip the deref
                    if let DecoratedExprKind::Call(ref call) = operand.kind {
                        if let DecoratedExprKind::Member { property, .. } = &call.callee.kind {
                            if property == "as_ref" {
                                // Strip the deref - .as_ref() already returns &T
                                return self.rewrite_expr(*operand);
                            }
                        }
                    }

                    // Otherwise, it's a direct field access (like member.prop)
                    // Replace *member.prop with &member.prop
                    DecoratedExpr {
                        kind: DecoratedExprKind::Ref {
                            mutable: false,
                            expr: operand,
                        },
                        metadata: expr.metadata.clone(),
                    }
                } else {
                    // Keep the deref as-is
                    DecoratedExpr {
                        kind: DecoratedExprKind::Unary {
                            op: crate::parser::UnaryOp::Deref,
                            operand,
                            unary_metadata,
                        },
                        metadata: expr.metadata,
                    }
                }
            }
            _ => expr,
        };

        let kind = match expr.kind {
            // Binary expressions
            DecoratedExprKind::Binary { left, op, right, binary_metadata } => {
                DecoratedExprKind::Binary {
                    left: Box::new(self.rewrite_expr(*left)),
                    op,
                    right: Box::new(self.rewrite_expr(*right)),
                    binary_metadata,
                }
            }

            // Unary expressions
            DecoratedExprKind::Unary { op, operand, unary_metadata } => {
                DecoratedExprKind::Unary {
                    op,
                    operand: Box::new(self.rewrite_expr(*operand)),
                    unary_metadata,
                }
            }

            // Member expressions
            DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata } => {
                DecoratedExprKind::Member {
                    object: Box::new(self.rewrite_expr(*object)),
                    property,
                    optional,
                    computed,
                    is_path,
                    field_metadata,
                }
            }

            // Call expressions
            DecoratedExprKind::Call(call) => {
                DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                    callee: self.rewrite_expr(call.callee),
                    args: call.args
                        .into_iter()
                        .map(|arg| self.rewrite_expr(arg))
                        .collect(),
                    type_args: call.type_args,
                    optional: call.optional,
                    is_macro: call.is_macro,
                    span: call.span,
                }))
            }

            // Parenthesized expressions
            DecoratedExprKind::Paren(inner) => {
                DecoratedExprKind::Paren(Box::new(self.rewrite_expr(*inner)))
            }

            // Block expressions
            DecoratedExprKind::Block(block) => {
                DecoratedExprKind::Block(self.rewrite_block(block))
            }

            // Index expressions
            DecoratedExprKind::Index { object, index } => {
                DecoratedExprKind::Index {
                    object: Box::new(self.rewrite_expr(*object)),
                    index: Box::new(self.rewrite_expr(*index)),
                }
            }

            // Struct initialization
            DecoratedExprKind::StructInit(struct_init) => {
                // Recursively rewrite field values
                let rewritten_fields = struct_init.fields.into_iter()
                    .map(|(name, value)| (name, self.rewrite_expr(value)))
                    .collect();

                DecoratedExprKind::StructInit(DecoratedStructInit {
                    name: struct_init.name,
                    fields: rewritten_fields,
                    span: struct_init.span,
                })
            }

            // Vec initialization
            DecoratedExprKind::VecInit(elements) => {
                DecoratedExprKind::VecInit(
                    elements.into_iter()
                        .map(|e| self.rewrite_expr(e))
                        .collect()
                )
            }

            // If expressions
            DecoratedExprKind::If(if_expr) => {
                DecoratedExprKind::If(Box::new(DecoratedIfExpr {
                    condition: self.rewrite_expr(if_expr.condition),
                    then_branch: self.rewrite_block(if_expr.then_branch),
                    else_branch: if_expr.else_branch.map(|b| self.rewrite_block(b)),
                }))
            }

            // Match expressions
            DecoratedExprKind::Match(match_expr) => {
                DecoratedExprKind::Match(Box::new(DecoratedMatchExpr {
                    expr: self.rewrite_expr(match_expr.expr),
                    arms: match_expr.arms
                        .into_iter()
                        .map(|arm| self.rewrite_match_arm(arm))
                        .collect(),
                }))
            }

            // Reference expressions
            DecoratedExprKind::Ref { mutable, expr: inner } => {
                DecoratedExprKind::Ref {
                    mutable,
                    expr: Box::new(self.rewrite_expr(*inner)),
                }
            }

            // Dereference expressions
            DecoratedExprKind::Deref(inner) => {
                DecoratedExprKind::Deref(Box::new(self.rewrite_expr(*inner)))
            }

            // Assignment
            DecoratedExprKind::Assign { left, right } => {
                DecoratedExprKind::Assign {
                    left: Box::new(self.rewrite_expr(*left)),
                    right: Box::new(self.rewrite_expr(*right)),
                }
            }

            // Compound assignment
            DecoratedExprKind::CompoundAssign { left, op, right } => {
                DecoratedExprKind::CompoundAssign {
                    left: Box::new(self.rewrite_expr(*left)),
                    op,
                    right: Box::new(self.rewrite_expr(*right)),
                }
            }

            // Range expressions
            DecoratedExprKind::Range { start, end, inclusive } => {
                DecoratedExprKind::Range {
                    start: start.map(|s| Box::new(self.rewrite_expr(*s))),
                    end: end.map(|e| Box::new(self.rewrite_expr(*e))),
                    inclusive,
                }
            }

            // Try expressions
            DecoratedExprKind::Try(inner) => {
                DecoratedExprKind::Try(Box::new(self.rewrite_expr(*inner)))
            }

            // Tuple expressions
            DecoratedExprKind::Tuple(elements) => {
                DecoratedExprKind::Tuple(
                    elements.into_iter()
                        .map(|e| self.rewrite_expr(e))
                        .collect()
                )
            }

            // Matches macro - will be expanded in transformation phase
            DecoratedExprKind::Matches { expr: inner, pattern } => {
                DecoratedExprKind::Matches {
                    expr: Box::new(self.rewrite_expr(*inner)),
                    pattern: self.rewrite_pattern(pattern),
                }
            }

            // Regex calls - recursively rewrite child expressions
            DecoratedExprKind::RegexCall(regex_call) => {
                DecoratedExprKind::RegexCall(Box::new(crate::codegen::decorated_ast::DecoratedRegexCall {
                    method: regex_call.method,
                    text_arg: self.rewrite_expr(regex_call.text_arg),
                    pattern: regex_call.pattern,
                    replacement_arg: regex_call.replacement_arg.map(|e| self.rewrite_expr(e)),
                    metadata: regex_call.metadata,
                    span: regex_call.span,
                }))
            }

            // Return expressions
            DecoratedExprKind::Return(value) => {
                DecoratedExprKind::Return(value.map(|v| Box::new(self.rewrite_expr(*v))))
            }

            // Leaf expressions that don't need child rewriting
            DecoratedExprKind::CustomPropAccess(access) => {
                return self.rewrite_custom_prop_access(*access);
            }

            DecoratedExprKind::Literal(_) |
            DecoratedExprKind::Ident { .. } |
            DecoratedExprKind::Break |
            DecoratedExprKind::Continue |
            DecoratedExprKind::Closure(_) => {
                expr.kind
            }
        };

        DecoratedExpr {
            kind,
            metadata: expr.metadata,
        }
    }

    // ========================================================================
    // HELPER: Pattern Wrapping for Shadowing
    // ========================================================================

    /// Wrap a variant pattern with a binding to enable implicit shadowing
    /// Example: Expr::Lit(Lit::Str(_)) → Expr::Lit(Lit::Str(expr))
    fn wrap_pattern_with_binding(&self, mut pattern: DecoratedPattern, binding: DecoratedPattern) -> DecoratedPattern {
        // Update the metadata's swc_pattern to include the binding
        // The metadata contains the SWC pattern like "Expr::Lit(Lit::Str(_))"
        // We need to replace the _ with the binding name

        if let DecoratedPatternKind::Ident(binding_name) = &binding.kind {
            // Replace _ or __ with the binding name in the swc_pattern
            let swc_pattern = pattern.metadata.swc_pattern.clone();

            // Map ReluxScript names to proper SWC patterns if needed
            let proper_swc_pattern = match swc_pattern.as_str() {
                "StringLiteral" => "Expr::Lit(Lit::Str(_))".to_string(),
                "NumericLiteral" => "Expr::Lit(Lit::Num(_))".to_string(),
                "BooleanLiteral" => "Expr::Lit(Lit::Bool(_))".to_string(),
                "NullLiteral" => "Expr::Lit(Lit::Null(_))".to_string(),
                "Identifier" => "Expr::Ident(_)".to_string(),
                "CallExpression" => "Expr::Call(_)".to_string(),
                "MemberExpression" => "Expr::Member(_)".to_string(),
                "ArrayExpression" => "Expr::Array(_)".to_string(),
                "ObjectExpression" => "Expr::Object(_)".to_string(),
                "BinaryExpression" => "Expr::Bin(_)".to_string(),
                "UnaryExpression" => "Expr::Unary(_)".to_string(),
                _ => swc_pattern.clone(),
            };

            // Common patterns to replace:
            // "Expr::Lit(Lit::Str(_))" → "Expr::Lit(Lit::Str(binding))"
            let new_pattern = if proper_swc_pattern.contains("(_)") {
                proper_swc_pattern.replace("(_)", &format!("({})", binding_name))
            } else if proper_swc_pattern.contains('(') {
                // Already has parentheses, use as-is
                proper_swc_pattern
            } else {
                // No placeholder found, append the binding
                format!("{}({})", proper_swc_pattern, binding_name)
            };

            pattern.metadata.swc_pattern = new_pattern;
        }

        pattern
    }

    // ========================================================================
    // TRANSFORMATION: Field Replacements
    // ========================================================================

    /// 🔧 Apply field replacements for writers
    /// In writers, State struct is flattened, so self.state.X becomes self.X
    /// Also, self.builder.X() becomes self.X() since CodeBuilder methods are on the writer
    fn apply_field_replacements(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        if !self.is_writer {
            return expr;
        }

        match expr.kind {
            DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata } => {
                // Check if object is self.state or self.builder - if so, replace with just self
                if let DecoratedExprKind::Member {
                    object: inner_obj,
                    property: inner_prop,
                    ..
                } = &object.kind {
                    if let DecoratedExprKind::Ident { name, .. } = &inner_obj.kind {
                        if name == "self" && (inner_prop == "state" || inner_prop == "builder") {
                            // self.state.X → self.X  or  self.builder.X() → self.X()
                            return DecoratedExpr {
                                kind: DecoratedExprKind::Member {
                                    object: inner_obj.clone(),  // Just "self"
                                    property,
                                    optional,
                                    computed,
                                    is_path,
                                    field_metadata,
                                },
                                metadata: expr.metadata,
                            };
                        }
                    }
                }

                // Check if THIS is self.state or self.builder (not followed by another property)
                // This handles cases where self.state or self.builder is used directly
                if let DecoratedExprKind::Ident { name, .. } = &object.kind {
                    if name == "self" && (property == "state" || property == "builder") {
                        // self.state → self  or  self.builder → self
                        // But only if it's being replaced (has Replace accessor)
                        if let FieldAccessor::Replace { with } = &field_metadata.accessor {
                            return DecoratedExpr {
                                kind: DecoratedExprKind::Ident {
                                    name: with.clone(),
                                    ident_metadata: SwcIdentifierMetadata::name(),
                                },
                                metadata: expr.metadata,
                            };
                        }
                    }
                }

                DecoratedExpr {
                    kind: DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata },
                    metadata: expr.metadata,
                }
            }
            _ => expr
        }
    }

    // ========================================================================
    // TRANSFORMATION: Context Remove
    // ========================================================================

    /// 🔧 Transform ctx.remove() into actual SWC node replacement
    /// Returns a statement that replaces the node with undefined
    fn apply_context_remove(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // Check if this is a call to ctx.remove()
        if let DecoratedExprKind::Call(ref call) = expr.kind {
            if let DecoratedExprKind::Member { ref object, ref property, .. } = call.callee.kind {
                if let DecoratedExprKind::Ident { ref name, .. } = object.kind {
                    if name == "ctx" && property == "remove" {
                        // Replace with: node.callee = Callee::Expr(Box::new(Expr::Ident(Ident::new("undefined".into(), DUMMY_SP))))
                        return DecoratedExpr {
                            kind: DecoratedExprKind::Assign {
                                left: Box::new(DecoratedExpr {
                                    kind: DecoratedExprKind::Member {
                                        object: Box::new(DecoratedExpr {
                                            kind: DecoratedExprKind::Ident {
                                                name: "node".to_string(),
                                                ident_metadata: SwcIdentifierMetadata::name(),
                                            },
                                            metadata: Self::simple_metadata("&mut CallExpr"),
                                        }),
                                        property: "callee".to_string(),
                                        optional: false,
                                        computed: false,
                                        is_path: false,
                                        field_metadata: SwcFieldMetadata::direct("callee".to_string(), "Callee".to_string()),
                                    },
                                    metadata: Self::simple_metadata("Callee"),
                                }),
                                right: Box::new(DecoratedExpr {
                                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                        callee: DecoratedExpr {
                                            kind: DecoratedExprKind::Ident {
                                                name: "Callee::Expr".to_string(),
                                                ident_metadata: SwcIdentifierMetadata::name(),
                                            },
                                            metadata: Self::simple_metadata("fn"),
                                        },
                                        args: vec![DecoratedExpr {
                                            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                                callee: DecoratedExpr {
                                                    kind: DecoratedExprKind::Ident {
                                                        name: "Box::new".to_string(),
                                                        ident_metadata: SwcIdentifierMetadata::name(),
                                                    },
                                                    metadata: Self::simple_metadata("fn"),
                                                },
                                                args: vec![DecoratedExpr {
                                                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                                        callee: DecoratedExpr {
                                                            kind: DecoratedExprKind::Ident {
                                                                name: "Expr::Ident".to_string(),
                                                                ident_metadata: SwcIdentifierMetadata::name(),
                                                            },
                                                            metadata: Self::simple_metadata("fn"),
                                                        },
                                                        args: vec![DecoratedExpr {
                                                            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                                                callee: DecoratedExpr {
                                                                    kind: DecoratedExprKind::Ident {
                                                                        name: "Ident::new".to_string(),
                                                                        ident_metadata: SwcIdentifierMetadata::name(),
                                                                    },
                                                                    metadata: Self::simple_metadata("fn"),
                                                                },
                                                                args: vec![
                                                                    DecoratedExpr {
                                                                        kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                                                            callee: DecoratedExpr {
                                                                                kind: DecoratedExprKind::Member {
                                                                                    object: Box::new(DecoratedExpr {
                                                                                        kind: DecoratedExprKind::Literal(Literal::String("undefined".to_string())),
                                                                                        metadata: Self::simple_metadata("&str"),
                                                                                    }),
                                                                                    property: "into".to_string(),
                                                                                    optional: false,
                                                                                    computed: false,
                                                                                    is_path: false,
                                                                                    field_metadata: SwcFieldMetadata::direct("into".to_string(), "fn".to_string()),
                                                                                },
                                                                                metadata: Self::simple_metadata("fn"),
                                                                            },
                                                                            args: vec![],
                                                                            type_args: vec![],
                                                                            optional: false,
                                                                            is_macro: false,
                                                                            span: Span::new(0, 0, 0, 0),
                                                                        })),
                                                                        metadata: Self::simple_metadata("JsWord"),
                                                                    },
                                                                    DecoratedExpr {
                                                                        kind: DecoratedExprKind::Ident {
                                                                            name: "DUMMY_SP".to_string(),
                                                                            ident_metadata: SwcIdentifierMetadata::name(),
                                                                        },
                                                                        metadata: Self::simple_metadata("Span"),
                                                                    },
                                                                    DecoratedExpr {
                                                                        kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                                                            callee: DecoratedExpr {
                                                                                kind: DecoratedExprKind::Ident {
                                                                                    name: "SyntaxContext::empty".to_string(),
                                                                                    ident_metadata: SwcIdentifierMetadata::name(),
                                                                                },
                                                                                metadata: Self::simple_metadata("fn"),
                                                                            },
                                                                            args: vec![],
                                                                            type_args: vec![],
                                                                            optional: false,
                                                                            is_macro: false,
                                                                            span: Span::new(0, 0, 0, 0),
                                                                        })),
                                                                        metadata: Self::simple_metadata("SyntaxContext"),
                                                                    },
                                                                ],
                                                                type_args: vec![],
                                                                optional: false,
                                                                is_macro: false,
                                                                span: Span::new(0, 0, 0, 0),
                                                            })),
                                                            metadata: Self::simple_metadata("Ident"),
                                                        }],
                                                        type_args: vec![],
                                                        optional: false,
                                                        is_macro: false,
                                                        span: Span::new(0, 0, 0, 0),
                                                    })),
                                                    metadata: Self::simple_metadata("Expr"),
                                                }],
                                                type_args: vec![],
                                                optional: false,
                                                is_macro: false,
                                                span: Span::new(0, 0, 0, 0),
                                            })),
                                            metadata: Self::simple_metadata("Box<Expr>"),
                                        }],
                                        type_args: vec![],
                                        optional: false,
                                                        is_macro: false,
                                        span: Span::new(0, 0, 0, 0),
                                    })),
                                    metadata: Self::simple_metadata("Callee"),
                                }),
                            },
                            metadata: expr.metadata.clone(),
                        };
                    }
                }
            }
        }

        expr
    }

    // ========================================================================
    // TRANSFORMATION: Codegen Helper Functions
    // ========================================================================

    /// 🔧 Transform codegen::generate() calls to codegen_to_string() helper
    /// transforms: codegen::generate(expr) → codegen_to_string(expr)
    /// transforms: codegen::generate_with_options(expr, opts) → codegen_to_string_with_config(expr, config)
    fn apply_codegen_helpers(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // Check if this is a call expression
        if let DecoratedExprKind::Call(ref call) = expr.kind {
            // Check if the callee is a member expression (module::function)
            if let DecoratedExprKind::Member { ref object, ref property, is_path, .. } = call.callee.kind {
                // Check if it's codegen::generate or codegen::generate_with_options
                if is_path {
                    if let DecoratedExprKind::Ident { ref name, .. } = object.kind {
                        if name == "codegen" {
                            match property.as_str() {
                                "generate" => {
                                    // Transform: codegen::generate(node) → codegen_to_string(node)
                                    return DecoratedExpr {
                                        kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                            callee: DecoratedExpr {
                                                kind: DecoratedExprKind::Ident {
                                                    name: "codegen_to_string".to_string(),
                                                    ident_metadata: SwcIdentifierMetadata::name(),
                                                },
                                                metadata: Self::simple_metadata("fn"),
                                            },
                                            args: call.args.clone(),
                                            type_args: vec![],
                                            optional: false,
                                            is_macro: false,
                                            span: call.span,
                                        })),
                                        metadata: expr.metadata.clone(),
                                    };
                                }
                                "generate_with_options" => {
                                    // Transform: codegen::generate_with_options(node, opts) → codegen_to_string_with_config(node, config)
                                    return DecoratedExpr {
                                        kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                            callee: DecoratedExpr {
                                                kind: DecoratedExprKind::Ident {
                                                    name: "codegen_to_string_with_config".to_string(),
                                                    ident_metadata: SwcIdentifierMetadata::name(),
                                                },
                                                metadata: Self::simple_metadata("fn"),
                                            },
                                            args: call.args.clone(),
                                            type_args: vec![],
                                            optional: false,
                                            is_macro: false,
                                            span: call.span,
                                        })),
                                        metadata: expr.metadata.clone(),
                                    };
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // No transformation needed
        expr
    }

    // ========================================================================
    // TRANSFORMATION: Helper Function Calls
    // ========================================================================

    /// 🔧 Add Self:: prefix to helper function calls
    /// transforms: is_helper("test") → Self::is_helper("test")
    fn apply_helper_function_calls(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // Check if this is a call expression with a simple identifier callee
        if let DecoratedExprKind::Call(ref call) = expr.kind {
            if let DecoratedExprKind::Ident { ref name, .. } = call.callee.kind {
                // Check if this is a helper function call
                if self.helper_functions.contains(name) {
                    // Transform: helper_func(args) → Self::helper_func(args)
                    return DecoratedExpr {
                        kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                            callee: DecoratedExpr {
                                kind: DecoratedExprKind::Member {
                                    object: Box::new(DecoratedExpr {
                                        kind: DecoratedExprKind::Ident {
                                            name: "Self".to_string(),
                                            ident_metadata: SwcIdentifierMetadata::name(),
                                        },
                                        metadata: Self::simple_metadata("type"),
                                    }),
                                    property: name.clone(),
                                    optional: false,
                                    computed: false,
                                    is_path: true,  // Use :: separator
                                    field_metadata: SwcFieldMetadata::direct(name.clone(), "fn".to_string()),
                                },
                                metadata: Self::simple_metadata("fn"),
                            },
                            args: call.args.clone(),
                            type_args: call.type_args.clone(),
                            optional: call.optional,
                            is_macro: call.is_macro,
                            span: call.span,
                        })),
                        metadata: expr.metadata.clone(),
                    };
                }
            }
        }

        // No transformation needed
        expr
    }

    // ========================================================================
    // TRANSFORMATION: Field Conversions (e.g., .clone() with read_conversion)
    // ========================================================================

    /// 🔧 Transform field access with .clone() to apply read_conversion
    /// transforms: id.name.clone() → id.sym.to_string() (when read_conversion is set)
    /// transforms: member.property → needs special handling for MemberProp
    fn apply_field_conversions(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // Check if this is a call to .clone()
        if let DecoratedExprKind::Call(ref call) = expr.kind {
            if let DecoratedExprKind::Member { ref object, ref property, .. } = call.callee.kind {
                if property == "clone" && call.args.is_empty() {
                    // This is a .clone() call - check if the object is a member access with read_conversion
                    if let DecoratedExprKind::Member { object: ref inner_object, field_metadata: ref inner_field_metadata, .. } = object.kind {
                        if !inner_field_metadata.read_conversion.is_empty() {
                            // We have a read_conversion! Transform member.field.clone() → member.field.to_string()
                            // The read_conversion already includes the method (e.g., ".to_string()")
                            // So we just need to apply it to the inner member expression
                            return DecoratedExpr {
                                kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                    callee: DecoratedExpr {
                                        kind: DecoratedExprKind::Member {
                                            object: inner_object.clone(),
                                            property: inner_field_metadata.read_conversion.trim_start_matches('.').to_string(),
                                            optional: false,
                                            computed: false,
                                            is_path: false,
                                            field_metadata: inner_field_metadata.clone(),
                                        },
                                        metadata: object.metadata.clone(),
                                    },
                                    args: vec![],
                                    type_args: vec![],
                                    optional: false,
                                    is_macro: false,
                                    span: call.span,
                                })),
                                metadata: expr.metadata.clone(),
                            };
                        }
                    }
                }
            }
        }

        // No transformation needed
        expr
    }

    // ========================================================================
    // TRANSFORMATION: MemberProp → String Conversion
    // ========================================================================

    /// 🔧 Transform member.prop.clone() → match expression for MemberProp → String
    /// This handles the case where member.prop (MemberProp enum) needs to be converted to String
    fn apply_member_prop_to_string(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // ONLY transform member.prop.clone() - NOT direct member.prop access
        // (because direct access might be used in pattern matching!)
        let is_prop_clone = if let DecoratedExprKind::Call(ref call) = expr.kind {
            // Check for .clone() call
            if let DecoratedExprKind::Member { ref object, ref property, .. } = call.callee.kind {
                property == "clone" && call.args.is_empty() &&
                matches!(&object.kind, DecoratedExprKind::Member { field_metadata, .. }
                    if field_metadata.swc_field_name == "prop")
            } else {
                false
            }
        } else {
            false
        };

        if !is_prop_clone {
            return expr;
        }

        // Extract the member.prop expression
        let member_prop_expr = if let DecoratedExprKind::Call(ref call) = expr.kind {
            if let DecoratedExprKind::Member { ref object, .. } = call.callee.kind {
                object.clone()
            } else {
                return expr;
            }
        } else {
            return expr;
        };

        // Create the match expression:
        // match &member.prop {
        //     MemberProp::Ident(id) => id.sym.to_string(),
        //     MemberProp::Computed(_) => "[computed]".to_string(),
        //     MemberProp::PrivateName(name) => format!("#{}", name.name.to_string()),
        // }

        // Create match scrutinee: &member.prop
        let scrutinee = DecoratedExpr {
            kind: DecoratedExprKind::Unary {
                op: crate::parser::UnaryOp::Ref,
                operand: member_prop_expr,
                unary_metadata: SwcUnaryMetadata { override_op: None, span: None },
            },
            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                swc_type: "&MemberProp".to_string(),
                is_boxed: false,
                is_optional: false,
                type_kind: crate::type_system::SwcTypeKind::Unknown,
                span: None,
            },
        };

        // Arm 1: MemberProp::Ident(id) => id.sym.to_string()
        let arm1 = DecoratedMatchArm {
            pattern: DecoratedPattern {
                kind: DecoratedPatternKind::Variant {
                    name: "MemberProp::Ident".to_string(),
                    inner: Some(Box::new(DecoratedPattern {
                        kind: DecoratedPatternKind::Ident("id".to_string()),
                        metadata: SwcPatternMetadata::direct("id".to_string()),
                    })),
                },
                metadata: SwcPatternMetadata::direct("MemberProp::Ident(id)".to_string()),
            },
            guard: None,
            body: DecoratedBlock {
                stmts: vec![DecoratedStmt::Expr(DecoratedExpr {
                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                        callee: DecoratedExpr {
                            kind: DecoratedExprKind::Member {
                                object: Box::new(DecoratedExpr {
                                    kind: DecoratedExprKind::Member {
                                        object: Box::new(DecoratedExpr {
                                            kind: DecoratedExprKind::Ident {
                                                name: "id".to_string(),
                                                ident_metadata: SwcIdentifierMetadata::name(),
                                            },
                                            metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                                        }),
                                        property: "sym".to_string(),
                                        optional: false,
                                        computed: false,
                                        is_path: false,
                                        field_metadata: SwcFieldMetadata::direct("sym".to_string(), "JsWord".to_string()),
                                    },
                                    metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                                }),
                                property: "to_string".to_string(),
                                optional: false,
                                computed: false,
                                is_path: false,
                                field_metadata: SwcFieldMetadata::direct("to_string".to_string(), "fn".to_string()),
                            },
                            metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                        },
                        args: vec![],
                        type_args: vec![],
                        optional: false,
                        is_macro: false,
                        span: crate::lexer::Span::new(0, 0, 0, 0),
                    })),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                })],
            },
        };

        // Arm 2: MemberProp::Computed(_) => "[computed]".to_string()
        let arm2 = DecoratedMatchArm {
            pattern: DecoratedPattern {
                kind: DecoratedPatternKind::Variant {
                    name: "MemberProp::Computed".to_string(),
                    inner: Some(Box::new(DecoratedPattern {
                        kind: DecoratedPatternKind::Wildcard,
                        metadata: SwcPatternMetadata::direct("_".to_string()),
                    })),
                },
                metadata: SwcPatternMetadata::direct("MemberProp::Computed(_)".to_string()),
            },
            guard: None,
            body: DecoratedBlock {
                stmts: vec![DecoratedStmt::Expr(DecoratedExpr {
                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                        callee: DecoratedExpr {
                            kind: DecoratedExprKind::Member {
                                object: Box::new(DecoratedExpr {
                                    kind: DecoratedExprKind::Literal(crate::parser::Literal::String("[computed]".to_string())),
                                    metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                                }),
                                property: "to_string".to_string(),
                                optional: false,
                                computed: false,
                                is_path: false,
                                field_metadata: SwcFieldMetadata::direct("to_string".to_string(), "fn".to_string()),
                            },
                            metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                        },
                        args: vec![],
                        type_args: vec![],
                        optional: false,
                        is_macro: false,
                        span: crate::lexer::Span::new(0, 0, 0, 0),
                    })),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                })],
            },
        };

        // Arm 3: MemberProp::PrivateName(name) => format!("#{}", name.name.to_string())
        let arm3 = DecoratedMatchArm {
            pattern: DecoratedPattern {
                kind: DecoratedPatternKind::Variant {
                    name: "MemberProp::PrivateName".to_string(),
                    inner: Some(Box::new(DecoratedPattern {
                        kind: DecoratedPatternKind::Ident("name".to_string()),
                        metadata: SwcPatternMetadata::direct("name".to_string()),
                    })),
                },
                metadata: SwcPatternMetadata::direct("MemberProp::PrivateName(name)".to_string()),
            },
            guard: None,
            body: DecoratedBlock {
                stmts: vec![DecoratedStmt::Expr(DecoratedExpr {
                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                        callee: DecoratedExpr {
                            kind: DecoratedExprKind::Ident {
                                name: "format".to_string(),
                                ident_metadata: SwcIdentifierMetadata::name(),
                            },
                            metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                        },
                        args: vec![
                            DecoratedExpr {
                                kind: DecoratedExprKind::Literal(crate::parser::Literal::String("\"#{}\"".to_string())),
                                metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                            },
                            DecoratedExpr {
                                kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                    callee: DecoratedExpr {
                                        kind: DecoratedExprKind::Member {
                                            object: Box::new(DecoratedExpr {
                                                kind: DecoratedExprKind::Member {
                                                    object: Box::new(DecoratedExpr {
                                                        kind: DecoratedExprKind::Ident {
                                                            name: "name".to_string(),
                                                            ident_metadata: SwcIdentifierMetadata::name(),
                                                        },
                                                        metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                                                    }),
                                                    property: "name".to_string(),
                                                    optional: false,
                                                    computed: false,
                                                    is_path: false,
                                                    field_metadata: SwcFieldMetadata::direct("name".to_string(), "JsWord".to_string()),
                                                },
                                                metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                                            }),
                                            property: "to_string".to_string(),
                                            optional: false,
                                            computed: false,
                                            is_path: false,
                                            field_metadata: SwcFieldMetadata::direct("to_string".to_string(), "fn".to_string()),
                                        },
                                        metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                                    },
                                    args: vec![],
                                    type_args: vec![],
                                    optional: false,
                                    is_macro: false,
                                    span: crate::lexer::Span::new(0, 0, 0, 0),
                                })),
                                metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                            },
                        ],
                        type_args: vec![],
                        optional: false,
                        is_macro: true,  // format! is a macro
                        span: crate::lexer::Span::new(0, 0, 0, 0),
                    })),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,  swc_type: "Unknown".to_string(), is_boxed: false, is_optional: false, type_kind: crate::type_system::SwcTypeKind::Unknown, span: None },
                })],
            },
        };

        // Create the match expression
        DecoratedExpr {
            kind: DecoratedExprKind::Match(Box::new(DecoratedMatchExpr {
                expr: scrutinee,
                arms: vec![arm1, arm2, arm3],
            })),
            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                swc_type: "String".to_string(),
                is_boxed: false,
                is_optional: false,
                type_kind: crate::type_system::SwcTypeKind::Primitive,
                span: None,
            },
        }
    }

    // ========================================================================
    // TRANSFORMATION: Visit Children Method Rewrite
    // ========================================================================

    /// 🔧 Transform node.visit_children(self) to appropriate SWC method
    /// - Plugins (mutable): node.visit_mut_children_with(self)
    /// - Writers (immutable): node.visit_children_with(self)
    fn apply_visit_children_rewrite(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // Check if this is a call expression
        if let DecoratedExprKind::Call(ref call) = expr.kind {
            // Check if the callee is a member expression
            if let DecoratedExprKind::Member { ref object, ref property, .. } = call.callee.kind {
                // Check if it's .visit_children
                if property == "visit_children" {
                    // Choose the right method based on writer vs plugin context
                    let method_name = if self.is_writer {
                        "visit_children_with"
                    } else {
                        "visit_mut_children_with"
                    };

                    return DecoratedExpr {
                        kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                            callee: DecoratedExpr {
                                kind: DecoratedExprKind::Member {
                                    object: object.clone(),
                                    property: method_name.to_string(),
                                    optional: false,
                                    computed: false,
                                    is_path: false,
                                    field_metadata: SwcFieldMetadata::direct(
                                        method_name.to_string(),
                                        "()".to_string()
                                    ),
                                },
                                metadata: call.callee.metadata.clone(),
                            },
                            args: call.args.clone(),
                            type_args: vec![],
                            optional: false,
                            is_macro: false,
                            span: call.span,
                        })),
                        metadata: expr.metadata.clone(),
                    };
                }
            }
        }

        // No transformation needed
        expr
    }

    // ========================================================================
    // TRANSFORMATION: Atom to String Conversion
    // ========================================================================

    /// 🔧 Transform .sym.clone() to .sym.to_string() when String type is needed
    /// SWC uses Atom (interned string) for identifiers, but ReluxScript code expects String
    fn apply_atom_to_string_conversion(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // Check if this is a method call
        if let DecoratedExprKind::Call(ref call) = expr.kind {
            // Check if the callee is a member expression (something.clone())
            if let DecoratedExprKind::Member { ref object, ref property, .. } = call.callee.kind {
                // Check if it's .clone() and the object ends with .sym or .name
                if property == "clone" && self.ends_with_sym_access(object) {
                    // Transform: [anything].name.clone() or [anything].sym.clone() → [anything].to_string()
                        return DecoratedExpr {
                            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                callee: DecoratedExpr {
                                    kind: DecoratedExprKind::Member {
                                        object: object.clone(),
                                        property: "to_string".to_string(),
                                        optional: false,
                                        computed: false,
                                        is_path: false,
                                        field_metadata: SwcFieldMetadata::direct(
                                            "to_string".to_string(),
                                            "String".to_string()
                                        ),
                                    },
                                    metadata: call.callee.metadata.clone(),
                                },
                                args: vec![],  // to_string() takes no args
                                type_args: vec![],
                                optional: false,
                                is_macro: false,
                                span: call.span,
                            })),
                            metadata: expr.metadata.clone(),
                        };
                }
            }

            // Also check for Ident with use_sym (decorated form of id.name)
            if let DecoratedExprKind::Ident { ref ident_metadata, .. } = call.callee.kind {
                if ident_metadata.use_sym {
                    // Transform: id.sym() → id.sym.to_string()
                    // The callee is already `id` with use_sym, we need to make it id.sym.to_string()
                    return DecoratedExpr {
                        kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                            callee: DecoratedExpr {
                                kind: DecoratedExprKind::Member {
                                    object: Box::new(call.callee.clone()),
                                    property: "to_string".to_string(),
                                    optional: false,
                                    computed: false,
                                    is_path: false,
                                    field_metadata: SwcFieldMetadata::direct(
                                        "to_string".to_string(),
                                        "String".to_string()
                                    ),
                                },
                                metadata: call.callee.metadata.clone(),
                            },
                            args: vec![],
                            type_args: vec![],
                            optional: false,
                            is_macro: false,
                            span: call.span,
                        })),
                        metadata: expr.metadata.clone(),
                    };
                }
            }
        }

        // No transformation needed
        expr
    }

    /// Helper: Check if an expression ends with .sym or .name access (identifier string fields)
    fn ends_with_sym_access(&self, expr: &DecoratedExpr) -> bool {
        if let DecoratedExprKind::Member { property, .. } = &expr.kind {
            // Check for both .sym (SWC) and .name (ReluxScript) as they map to Atom/String
            property == "sym" || property == "name"
        } else {
            false
        }
    }

    // ========================================================================
    // TRANSFORMATION: AST Struct Initialization
    // ========================================================================

    /// 🔧 Transform AST node struct initialization to add required fields
    /// transforms: Identifier { name: "x" } → Ident { sym: "x".into(), span: DUMMY_SP }
    fn apply_ast_struct_init(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        use crate::codegen::decorated_ast::{DecoratedStructInit};
        use crate::codegen::swc_metadata::{FieldAccessor};
        use crate::lexer::Span;
        use crate::type_system::SwcTypeKind;

        if let DecoratedExprKind::StructInit(ref struct_init) = expr.kind {
            // Check if this is an AST node type that needs transformation
            let swc_type = &expr.metadata.swc_type;

            // For Identifier → Ident, transform the fields
            if struct_init.name == "Identifier" && swc_type == "Ident" {
                let mut new_fields = Vec::new();

                // Map each field - working with DecoratedExpr from DecoratedStructInit
                for (field_name, field_expr) in &struct_init.fields {
                    if field_name == "name" {
                        // name → sym with .into()
                        // Create: field_expr.into() as a DecoratedExpr
                        let into_call = DecoratedExpr {
                            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                callee: DecoratedExpr {
                                    kind: DecoratedExprKind::Member {
                                        object: Box::new(field_expr.clone()),
                                        property: "into".to_string(),
                                        optional: false,
                                        computed: false,
                                        is_path: false,
                                        field_metadata: SwcFieldMetadata {
                                            swc_field_name: "into".to_string(),
                                            accessor: FieldAccessor::Direct,
                                            field_type: "".to_string(),
                                            source_field: None,
                                            span: None,
                                            read_conversion: String::new(),
                                        },
                                    },
                                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                        swc_type: "".to_string(),
                                        is_boxed: false,
                                        is_optional: false,
                                        type_kind: SwcTypeKind::Unknown,
                                        span: None,
                                    },
                                },
                                args: vec![],
                                type_args: vec![],
                                optional: false,
                                is_macro: false,
                                span: Span::new(0, 0, 0, 0),
                            })),
                            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                swc_type: "Atom".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: SwcTypeKind::Unknown,
                                span: None,
                            },
                        };
                        new_fields.push(("sym".to_string(), into_call));
                    } else {
                        new_fields.push((field_name.clone(), field_expr.clone()));
                    }
                }

                // Add required fields that weren't specified
                if !new_fields.iter().any(|(name, _)| name == "span") {
                    new_fields.push((
                        "span".to_string(),
                        DecoratedExpr {
                            kind: DecoratedExprKind::Ident {
                                name: "DUMMY_SP".to_string(),
                                ident_metadata: SwcIdentifierMetadata {
                                    use_sym: false,
                                    deref_pattern: None,
                                    span: None,
                                },
                            },
                            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                swc_type: "Span".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: SwcTypeKind::Unknown,
                                span: None,
                            },
                        }
                    ));
                }

                // Add optional: false
                if !new_fields.iter().any(|(name, _)| name == "optional") {
                    new_fields.push((
                        "optional".to_string(),
                        DecoratedExpr {
                            kind: DecoratedExprKind::Literal(Literal::Bool(false)),
                            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                swc_type: "bool".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: SwcTypeKind::Unknown,
                                span: None,
                            },
                        }
                    ));
                }

                // Add ctxt: SyntaxContext::empty()
                // Use a simple identifier "SyntaxContext::empty()" as a workaround
                if !new_fields.iter().any(|(name, _)| name == "ctxt") {
                    new_fields.push((
                        "ctxt".to_string(),
                        DecoratedExpr {
                            kind: DecoratedExprKind::Ident {
                                name: "SyntaxContext::empty()".to_string(),
                                ident_metadata: SwcIdentifierMetadata {
                                    use_sym: false,
                                    deref_pattern: None,
                                    span: None,
                                },
                            },
                            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                swc_type: "SyntaxContext".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: SwcTypeKind::Unknown,
                                span: None,
                            },
                        }
                    ));
                }

                // Return transformed struct init with updated fields
                // Wrap in DecoratedExpr so it can be converted with .into() if needed
                let ident_expr = DecoratedExpr {
                    kind: DecoratedExprKind::StructInit(DecoratedStructInit {
                        name: swc_type.clone(),
                        fields: new_fields,
                        span: struct_init.span,
                    }),
                    metadata: expr.metadata.clone(),
                };

                // Wrap in .into() call for automatic conversion (Ident -> BindingIdent, etc.)
                return DecoratedExpr {
                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                        callee: DecoratedExpr {
                            kind: DecoratedExprKind::Member {
                                object: Box::new(ident_expr),
                                property: "into".to_string(),
                                optional: false,
                                computed: false,
                                is_path: false,
                                field_metadata: SwcFieldMetadata::direct("into".to_string(), "fn".to_string()),
                            },
                            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                swc_type: "fn".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: SwcTypeKind::Primitive,
                                span: Some(struct_init.span),
                            },
                        },
                        args: vec![],
                        type_args: vec![],
                        optional: false,
                        is_macro: false,
                        span: struct_init.span,
                    })),
                    metadata: expr.metadata.clone(),
                };
            }
        }

        // No transformation needed
        expr
    }

    // ========================================================================
    // TRANSFORMATION: Matches! Macro Expansion
    // ========================================================================

    /// 🔧 Expand matches! macro to match expression
    /// transforms: matches!(expr, pattern) → match &expr { pattern => true, _ => false }
    // ========================================================================
    // TRANSFORMATION: Auto-unwrap narrowed enum types
    // ========================================================================

    /// 🔧 Automatically unwrap identifiers that have narrowed enum types
    /// Example: if binding has type Pat but was narrowed to ArrayPat
    /// Transform: binding.clone() → match binding { Pat::Array(ref inner) => inner, _ => unreachable!() }.clone()
    fn apply_auto_unwrap(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        match expr.kind {
            // If it's a method call, unwrap the object if needed
            DecoratedExprKind::Call(call) => {
                if let DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata } = call.callee.kind.clone() {
                    // Check if the object is an identifier that needs unwrapping
                    if let DecoratedExprKind::Ident { ref name, .. } = object.kind {
                        if let Some((parent_enum, variant_name)) = &object.metadata.needs_enum_unwrap {
                            eprintln!("[AUTO-UNWRAP] Unwrapping '{}' from {}::{} to {} before calling .{}",
                                name, parent_enum, variant_name, object.metadata.swc_type, property);

                            // Create the unwrapping match expression for the object
                            // match binding { Pat::Array(ref inner) => inner, _ => unreachable!() }

                            let pattern_str = format!("{}::{}", parent_enum, variant_name);
                            let inner_type = object.metadata.swc_type.clone();

                // Create the match arm pattern: Pat::Array(ref inner)
                let match_pattern = DecoratedPattern {
                    kind: DecoratedPatternKind::Variant {
                        name: pattern_str.clone(),
                        inner: Some(Box::new(DecoratedPattern {
                            kind: DecoratedPatternKind::Ident("inner".to_string()),
                            metadata: SwcPatternMetadata::direct(inner_type.clone()),
                        })),
                    },
                    metadata: SwcPatternMetadata {
                        swc_pattern: pattern_str.clone(),
                        unwrap_strategy: UnwrapStrategy::Ref,  // Use ref in the pattern
                        inner: None,
                        span: None,
                        source_pattern: None,
                        desugar_strategy: None,
                    },
                };

                // Create the match arm body: inner
                let match_body = DecoratedExpr {
                    kind: DecoratedExprKind::Ident {
                        name: "inner".to_string(),
                        ident_metadata: SwcIdentifierMetadata::name(),
                    },
                    metadata: SwcExprMetadata {
                        needs_enum_unwrap: None,
                        swc_type: inner_type.clone(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: expr.metadata.type_kind.clone(),
                        span: None,
                    },
                };

                // Create match arm
                let match_arm = DecoratedMatchArm {
                    pattern: match_pattern,
                    guard: None,
                    body: DecoratedBlock {
                        stmts: vec![DecoratedStmt::Expr(match_body)],
                    },
                };

                // Create wildcard arm with unreachable!()
                let unreachable_call = DecoratedExpr {
                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                        callee: DecoratedExpr {
                            kind: DecoratedExprKind::Ident {
                                name: "unreachable".to_string(),  // Macro name without ! (emitter adds it)
                                ident_metadata: SwcIdentifierMetadata::name(),
                            },
                            metadata: SwcExprMetadata {
                                needs_enum_unwrap: None,
                                swc_type: "macro".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: crate::type_system::SwcTypeKind::Unknown,
                                span: None,
                            },
                        },
                        args: vec![],
                        type_args: vec![],
                        optional: false,
                        is_macro: true,
                        span: Span::new(0, 0, 0, 0),
                    })),
                    metadata: SwcExprMetadata {
                        needs_enum_unwrap: None,
                        swc_type: "!".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                        span: None,
                    },
                };

                let wildcard_arm = DecoratedMatchArm {
                    pattern: DecoratedPattern {
                        kind: DecoratedPatternKind::Wildcard,
                        metadata: SwcPatternMetadata::direct("_".to_string()),
                    },
                    guard: None,
                    body: DecoratedBlock {
                        stmts: vec![DecoratedStmt::Expr(unreachable_call)],
                    },
                };

                            // Create the match expression that unwraps the object
                            let unwrapped_object = DecoratedExpr {
                                kind: DecoratedExprKind::Match(Box::new(DecoratedMatchExpr {
                                    expr: *object.clone(),
                                    arms: vec![match_arm, wildcard_arm],
                                })),
                                metadata: SwcExprMetadata {
                                    needs_enum_unwrap: None,
                                    swc_type: inner_type.clone(),
                                    is_boxed: false,
                                    is_optional: false,
                                    type_kind: object.metadata.type_kind.clone(),
                                    span: object.metadata.span,
                                },
                            };

                            // Rebuild the member expression with the unwrapped object
                            let new_callee = DecoratedExpr {
                                kind: DecoratedExprKind::Member {
                                    object: Box::new(unwrapped_object),
                                    property,
                                    optional,
                                    computed,
                                    is_path,
                                    field_metadata,
                                },
                                metadata: call.callee.metadata.clone(),
                            };

                            // Return the call with the new callee
                            return DecoratedExpr {
                                kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                    callee: new_callee,
                                    args: call.args,
                                    type_args: call.type_args,
                                    optional: call.optional,
                                    is_macro: call.is_macro,
                                    span: call.span,
                                })),
                                metadata: expr.metadata,
                            };
                        }
                    }
                }
                // No unwrapping needed, return as-is
                DecoratedExpr {
                    kind: DecoratedExprKind::Call(call),
                    metadata: expr.metadata,
                }
            }
            // If it's a field access, unwrap the object if needed
            DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata } => {
                // Check if the object is an identifier that needs unwrapping
                if let DecoratedExprKind::Ident { ref name, .. } = object.kind {
                    if let Some((parent_enum, variant_name)) = &object.metadata.needs_enum_unwrap {
                        eprintln!("[AUTO-UNWRAP] Unwrapping '{}' from {}::{} to {} before accessing .{}",
                            name, parent_enum, variant_name, object.metadata.swc_type, property);

                        // Create the unwrapping match expression
                        let pattern_str = format!("{}::{}", parent_enum, variant_name);
                        let inner_type = object.metadata.swc_type.clone();

                        // Create the match arm pattern
                        let match_pattern = DecoratedPattern {
                            kind: DecoratedPatternKind::Variant {
                                name: pattern_str.clone(),
                                inner: Some(Box::new(DecoratedPattern {
                                    kind: DecoratedPatternKind::Ident("inner".to_string()),
                                    metadata: SwcPatternMetadata::direct(inner_type.clone()),
                                })),
                            },
                            metadata: SwcPatternMetadata {
                                swc_pattern: pattern_str.clone(),
                                unwrap_strategy: UnwrapStrategy::Ref,
                                inner: None,
                                span: None,
                                source_pattern: None,
                                desugar_strategy: None,
                            },
                        };

                        // Create the match arm body
                        let match_body = DecoratedExpr {
                            kind: DecoratedExprKind::Ident {
                                name: "inner".to_string(),
                                ident_metadata: SwcIdentifierMetadata::name(),
                            },
                            metadata: SwcExprMetadata {
                                needs_enum_unwrap: None,
                                swc_type: inner_type.clone(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: object.metadata.type_kind.clone(),
                                span: None,
                            },
                        };

                        // Create match arm
                        let match_arm = DecoratedMatchArm {
                            pattern: match_pattern,
                            guard: None,
                            body: DecoratedBlock {
                                stmts: vec![DecoratedStmt::Expr(match_body)],
                            },
                        };

                        // Create wildcard arm with unreachable!()
                        let unreachable_call = DecoratedExpr {
                            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                callee: DecoratedExpr {
                                    kind: DecoratedExprKind::Ident {
                                        name: "unreachable".to_string(),
                                        ident_metadata: SwcIdentifierMetadata::name(),
                                    },
                                    metadata: SwcExprMetadata {
                                        needs_enum_unwrap: None,
                                        swc_type: "macro".to_string(),
                                        is_boxed: false,
                                        is_optional: false,
                                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                                        span: None,
                                    },
                                },
                                args: vec![],
                                type_args: vec![],
                                optional: false,
                                is_macro: true,
                                span: Span::new(0, 0, 0, 0),
                            })),
                            metadata: SwcExprMetadata {
                                needs_enum_unwrap: None,
                                swc_type: "!".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: crate::type_system::SwcTypeKind::Unknown,
                                span: None,
                            },
                        };

                        let wildcard_arm = DecoratedMatchArm {
                            pattern: DecoratedPattern {
                                kind: DecoratedPatternKind::Wildcard,
                                metadata: SwcPatternMetadata::direct("_".to_string()),
                            },
                            guard: None,
                            body: DecoratedBlock {
                                stmts: vec![DecoratedStmt::Expr(unreachable_call)],
                            },
                        };

                        // Create the match expression
                        let unwrapped_object = DecoratedExpr {
                            kind: DecoratedExprKind::Match(Box::new(DecoratedMatchExpr {
                                expr: *object.clone(),
                                arms: vec![match_arm, wildcard_arm],
                            })),
                            metadata: SwcExprMetadata {
                                needs_enum_unwrap: None,
                                swc_type: inner_type.clone(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: object.metadata.type_kind.clone(),
                                span: object.metadata.span,
                            },
                        };

                        // Return the member expression with the unwrapped object
                        return DecoratedExpr {
                            kind: DecoratedExprKind::Member {
                                object: Box::new(unwrapped_object),
                                property,
                                optional,
                                computed,
                                is_path,
                                field_metadata,
                            },
                            metadata: expr.metadata,
                        };
                    }
                }
                // No unwrapping needed, return as-is
                DecoratedExpr {
                    kind: DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata },
                    metadata: expr.metadata,
                }
            }
            _ => expr,
        }
    }

    fn apply_matches_expansion(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        match expr.kind {
            DecoratedExprKind::Matches { expr: scrutinee, pattern } => {
                // The pattern may have already been desugared (in rewrite_pattern)
                // Now we wrap it in a match expression

                eprintln!("[REWRITER MATCHES START] scrutinee metadata: swc_type='{}', is_boxed={}, needs_enum_unwrap={:?}",
                          scrutinee.metadata.swc_type, scrutinee.metadata.is_boxed, scrutinee.metadata.needs_enum_unwrap);

                // Create the match arms
                let match_arm = DecoratedMatchArm {
                    pattern,
                    guard: None,
                    body: DecoratedBlock {
                        stmts: vec![DecoratedStmt::Expr(DecoratedExpr {
                            kind: DecoratedExprKind::Literal(Literal::Bool(true)),
                            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                swc_type: "bool".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: crate::type_system::SwcTypeKind::Primitive,
                                span: None,
                            },
                        })],
                    },
                };

                let wildcard_arm = DecoratedMatchArm {
                    pattern: DecoratedPattern {
                        kind: DecoratedPatternKind::Wildcard,
                        metadata: SwcPatternMetadata::direct("_".to_string()),
                    },
                    guard: None,
                    body: DecoratedBlock {
                        stmts: vec![DecoratedStmt::Expr(DecoratedExpr {
                            kind: DecoratedExprKind::Literal(Literal::Bool(false)),
                            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                swc_type: "bool".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: crate::type_system::SwcTypeKind::Primitive,
                                span: None,
                            },
                        })],
                    },
                };

                // Create match expression - wrap scrutinee in & to match by reference
                let scrutinee_type = scrutinee.metadata.swc_type.clone();
                let scrutinee_span = scrutinee.metadata.span;

                eprintln!("[REWRITER MATCHES] Scrutinee type: '{}'", scrutinee_type);

                // Check if scrutinee type is &Box<T> - if so, we need to unwrap with .as_ref()
                let unwrapped_scrutinee = if scrutinee_type.starts_with("&Box<") || scrutinee_type.starts_with("&mut Box<") {
                    // Extract inner type from &Box<T> or &mut Box<T>
                    let inner_type = if let Some(inner) = scrutinee_type.strip_prefix("&mut Box<") {
                        inner.strip_suffix(">").unwrap_or(inner)
                    } else if let Some(inner) = scrutinee_type.strip_prefix("&Box<") {
                        inner.strip_suffix(">").unwrap_or(inner)
                    } else {
                        &scrutinee_type
                    };

                    eprintln!("[REWRITER MATCHES] Scrutinee type is '{}', unwrapping Box with .as_ref()", scrutinee_type);

                    // Generate: scrutinee.as_ref()
                    let as_ref_call = DecoratedExpr {
                        kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                            callee: DecoratedExpr {
                                kind: DecoratedExprKind::Member {
                                    object: scrutinee,
                                    property: "as_ref".to_string(),
                                    optional: false,
                                    computed: false,
                                    is_path: false,
                                    field_metadata: crate::codegen::swc_metadata::SwcFieldMetadata {
                                        swc_field_name: "as_ref".to_string(),
                                        accessor: crate::codegen::swc_metadata::FieldAccessor::Direct,
                                        field_type: format!("&{}", inner_type),
                                        source_field: Some("as_ref".to_string()),
                                        span: scrutinee_span,
                                        read_conversion: String::new(),
                                    },
                                },
                                metadata: SwcExprMetadata {
                                    needs_enum_unwrap: None,
                                    swc_type: format!("&{}", inner_type),
                                    is_boxed: false,
                                    is_optional: false,
                                    type_kind: crate::type_system::SwcTypeKind::Unknown,
                                    span: scrutinee_span,
                                },
                            },
                            args: vec![],
                            type_args: vec![],
                            optional: false,
                            is_macro: false,
                            span: scrutinee_span.unwrap_or(crate::lexer::Span::new(0, 0, 0, 0)),
                        })),
                        metadata: SwcExprMetadata {
                            needs_enum_unwrap: None,
                            swc_type: format!("&{}", inner_type),
                            is_boxed: false,
                            is_optional: false,
                            type_kind: crate::type_system::SwcTypeKind::Unknown,
                            span: scrutinee_span,
                        },
                    };

                    // Now wrap in &* to get &T from &T
                    let deref_expr = DecoratedExpr {
                        kind: DecoratedExprKind::Unary {
                            op: crate::parser::UnaryOp::Deref,
                            operand: Box::new(as_ref_call),
                            unary_metadata: crate::codegen::swc_metadata::SwcUnaryMetadata {
                                override_op: None,
                                span: scrutinee_span,
                            },
                        },
                        metadata: SwcExprMetadata {
                            needs_enum_unwrap: None,
                            swc_type: inner_type.to_string(),
                            is_boxed: false,
                            is_optional: false,
                            type_kind: crate::type_system::SwcTypeKind::Unknown,
                            span: scrutinee_span,
                        },
                    };

                    deref_expr
                } else {
                    *scrutinee
                };

                let ref_scrutinee = DecoratedExpr {
                    kind: DecoratedExprKind::Ref {
                        expr: Box::new(unwrapped_scrutinee),
                        mutable: false,
                    },
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,
                        swc_type: format!("&{}", scrutinee_type),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                        span: scrutinee_span,
                    },
                };

                DecoratedExpr {
                    kind: DecoratedExprKind::Match(Box::new(DecoratedMatchExpr {
                        expr: ref_scrutinee,
                        arms: vec![match_arm, wildcard_arm],
                    })),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "bool".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: crate::type_system::SwcTypeKind::Primitive,
                        span: expr.metadata.span,
                    },
                }
            }
            _ => expr,
        }
    }

    // ========================================================================
    // TRANSFORMATION: Iterator Methods
    // ========================================================================

    /// 🔧 Apply iterator method transformations
    /// Transforms vec.map() → vec.iter().map() for iterator methods on Vec
    fn apply_iterator_methods(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        match &expr.kind {
            DecoratedExprKind::Call(call) => {
                // Check if this is a method call (callee is a member expression)
                if let DecoratedExprKind::Member { object, property, .. } = &call.callee.kind {
                    // Check if the method is an iterator method
                    let iterator_methods = ["map", "filter", "find", "any", "all", "fold", "for_each"];

                    if iterator_methods.contains(&property.as_str()) {
                        // Check if the object is a Vec (swc_type contains "Vec")
                        if object.metadata.swc_type.contains("Vec") ||
                           object.metadata.swc_type == "vec" {
                            // Insert .iter() call between object and method
                            // vec.map(f) → vec.iter().map(f)

                            let iter_call = DecoratedExpr {
                                kind: DecoratedExprKind::Member {
                                    object: object.clone(),
                                    property: "iter".to_string(),
                                    optional: false,
                                    computed: false,
                                    is_path: false,
                                    field_metadata: SwcFieldMetadata::direct("iter".to_string(), "fn".to_string()),
                                },
                                metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                    swc_type: "fn".to_string(),
                                    is_boxed: false,
                                    is_optional: false,
                                    type_kind: crate::type_system::SwcTypeKind::Unknown,
                                    span: object.metadata.span,
                                },
                            };

                            let iter_call_expr = DecoratedExpr {
                                kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                    callee: iter_call,
                                    args: vec![],
                                    type_args: vec![],
                                    optional: false,
                                    is_macro: false,
                    span: call.span,
                                })),
                                metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                                    swc_type: "Iterator".to_string(),
                                    is_boxed: false,
                                    is_optional: false,
                                    type_kind: crate::type_system::SwcTypeKind::Unknown,
                                    span: object.metadata.span,
                                },
                            };

                            // Now create the final method call with iter() as the object
                            let new_callee = DecoratedExpr {
                                kind: DecoratedExprKind::Member {
                                    object: Box::new(iter_call_expr),
                                    property: property.clone(),
                                    optional: false,
                                    computed: false,
                                    is_path: false,
                                    field_metadata: SwcFieldMetadata::direct(property.clone(), "fn".to_string()),
                                },
                                metadata: call.callee.metadata.clone(),
                            };

                            return DecoratedExpr {
                                kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                    callee: new_callee,
                                    args: call.args.clone(),
                                    type_args: call.type_args.clone(),
                                    optional: call.optional,
                                    is_macro: call.is_macro,
                                    span: call.span,
                                })),
                                metadata: expr.metadata.clone(),
                            };
                        }
                    }
                }

                expr
            }
            _ => expr,
        }
    }

    // ========================================================================
    // TRANSFORMATION: String Literal Conversion
    // ========================================================================

    /// 🔧 Convert string literals to String when needed
    /// transforms: "hello" → "hello".to_string() (in return position or if/else arms returning String)
    fn apply_string_literal_conversion(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        match &expr.kind {
            // Check for string literals that might need conversion
            DecoratedExprKind::Literal(Literal::String(_)) => {
                // For SWC, always add .to_string() to bare string literals
                // The emitter will check context (return statements, assignments, etc.)
                // and add the conversion when needed
                // Actually, we can't easily determine context here, so we'll handle
                // this in specific positions like return statements
                expr
            }

            // Handle if expressions - convert string literals in branches
            DecoratedExprKind::If(if_expr) => {
                DecoratedExpr {
                    kind: DecoratedExprKind::If(Box::new(DecoratedIfExpr {
                        condition: if_expr.condition.clone(),
                        then_branch: self.convert_block_tail_string_literal(if_expr.then_branch.clone()),
                        else_branch: if_expr.else_branch.as_ref().map(|b| self.convert_block_tail_string_literal(b.clone())),
                    })),
                    metadata: expr.metadata.clone(),
                }
            }

            // Other expressions pass through unchanged
            _ => expr,
        }
    }

    /// Helper: Convert string literal in block's tail position
    fn convert_block_tail_string_literal(&mut self, block: DecoratedBlock) -> DecoratedBlock {
        let mut stmts = block.stmts;
        if let Some(last_stmt) = stmts.last_mut() {
            if let DecoratedStmt::Expr(ref mut expr) = last_stmt {
                if let DecoratedExprKind::Literal(Literal::String(_)) = expr.kind {
                    // Wrap with .to_string() call
                    *expr = self.wrap_with_to_string(expr.clone());
                }
            } else if let DecoratedStmt::Return(Some(ref mut expr)) = last_stmt {
                if let DecoratedExprKind::Literal(Literal::String(_)) = expr.kind {
                    // Wrap with .to_string() call
                    *expr = self.wrap_with_to_string(expr.clone());
                }
            }
        }
        DecoratedBlock { stmts }
    }

    /// Helper: Wrap expression with .to_string() method call
    fn wrap_with_to_string(&self, expr: DecoratedExpr) -> DecoratedExpr {
        use crate::lexer::Span as LexerSpan;
        let span = expr.metadata.span.unwrap_or(LexerSpan::new(0, 0, 0, 0));

        DecoratedExpr {
            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                callee: DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: Box::new(expr.clone()),
                        property: "to_string".to_string(),
                        optional: false,
                        computed: false,
                        is_path: false,
                        field_metadata: SwcFieldMetadata::direct("to_string".to_string(), "fn".to_string()),
                    },
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "fn".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                        span: Some(span),
                    },
                },
                args: vec![],
                type_args: vec![],
                optional: false,
                is_macro: false,
                span,
            })),
            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                swc_type: "String".to_string(),
                is_boxed: false,
                is_optional: false,
                type_kind: crate::type_system::SwcTypeKind::Unknown,
                span: Some(span),
            },
        }
    }

    // ========================================================================
    // UTILITIES
    // ========================================================================

    /// Generate unique temporary variable name
    fn _gen_temp_var(&mut self) -> String {
        let name = format!("__temp_{}", self.temp_var_counter);
        self.temp_var_counter += 1;
        name
    }

    /// Helper to create simple metadata
    fn simple_metadata(swc_type: &str) -> SwcExprMetadata {
        SwcExprMetadata { needs_enum_unwrap: None, 
            swc_type: swc_type.to_string(),
            is_boxed: false,
            is_optional: false,
            type_kind: crate::type_system::SwcTypeKind::Unknown,
            span: None,
        }
    }

    /// Rewrite custom property assignment to state.set_custom_prop() call
    fn rewrite_custom_prop_assignment(&mut self, assign: DecoratedCustomPropAssignment) -> DecoratedStmt {
        use crate::codegen::decorated_ast::{DecoratedCallExpr, DecoratedExprKind};

        // Check if this is a deletion (assignment to None)
        if assign.metadata.is_deletion {
            // Transform: node.__prop = None → self.state.delete_custom_prop(node, "__prop")
            return self.build_delete_call(assign.node, assign.property);
        }

        // Transform: node.__prop = value → self.state.set_custom_prop(node, "__prop", CustomPropValue::Variant(value))
        let wrapped_value = self.wrap_in_custom_prop_value(
            assign.value,
            &assign.metadata.variant
        );

        // Build self.state.set_custom_prop(node, "__prop", wrapped_value)
        let set_call = DecoratedExpr {
            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                callee: self.build_state_method_path("set_custom_prop"),
                args: vec![
                    assign.node,
                    self.string_literal(&assign.property),
                    wrapped_value,
                ],
                is_macro: false,
                optional: false,
                type_args: vec![],
                span: crate::lexer::Span::new(0, 0, 0, 0),
            })),
            metadata: Self::simple_metadata("()"),
        };

        DecoratedStmt::Expr(set_call)
    }

    /// Rewrite custom property access to state.get_custom_prop() call
    fn rewrite_custom_prop_access(&mut self, access: DecoratedCustomPropAccess) -> DecoratedExpr {
        use crate::codegen::decorated_ast::{DecoratedCallExpr, DecoratedExprKind};

        // Build: self.state.get_custom_prop(node, "__prop")
        let get_call = DecoratedExpr {
            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                callee: self.build_state_method_path("get_custom_prop"),
                args: vec![
                    *access.node,
                    self.string_literal(&access.property),
                ],
                is_macro: false,
                optional: false,
                type_args: vec![],
                span: crate::lexer::Span::new(0, 0, 0, 0),
            })),
            metadata: Self::simple_metadata("Option<&CustomPropValue>"),
        };

        // If we have an unwrapper pattern, chain with .and_then(|v| unwrapper)
        if let Some(unwrapper) = access.metadata.unwrapper_pattern {
            self.chain_and_then(get_call, unwrapper)
        } else {
            get_call
        }
    }

    /// Build a delete_custom_prop call
    fn build_delete_call(&mut self, node: DecoratedExpr, property: String) -> DecoratedStmt {
        use crate::codegen::decorated_ast::{DecoratedCallExpr, DecoratedExprKind};

        let delete_call = DecoratedExpr {
            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                callee: self.build_state_method_path("delete_custom_prop"),
                args: vec![
                    node,
                    self.string_literal(&property),
                ],
                is_macro: false,
                optional: false,
                type_args: vec![],
                span: crate::lexer::Span::new(0, 0, 0, 0),
            })),
            metadata: Self::simple_metadata("()"),
        };

        DecoratedStmt::Expr(delete_call)
    }

    /// Wrap a value in CustomPropValue::Variant(value)
    fn wrap_in_custom_prop_value(&mut self, value: DecoratedExpr, variant: &str) -> DecoratedExpr {
        use crate::codegen::decorated_ast::{DecoratedCallExpr, DecoratedExprKind};
        use crate::codegen::swc_metadata::SwcFieldMetadata;

        // Rewrite the value expression first
        let mut rewritten_value = self.rewrite_expr(value);

        // For Str variant, wrap string literals with .to_string()
        if variant == "Str" {
            if matches!(rewritten_value.kind, DecoratedExprKind::Literal(_)) {
                rewritten_value = DecoratedExpr {
                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                        callee: DecoratedExpr {
                            kind: DecoratedExprKind::Member {
                                object: Box::new(rewritten_value),
                                property: "to_string".to_string(),
                                optional: false,
                                computed: false,
                                is_path: false,
                                field_metadata: SwcFieldMetadata::direct("to_string".to_string(), "fn".to_string()),
                            },
                            metadata: Self::simple_metadata("String"),
                        },
                        args: vec![],
                        is_macro: false,
                        optional: false,
                        type_args: vec![],
                        span: crate::lexer::Span::new(0, 0, 0, 0),
                    })),
                    metadata: Self::simple_metadata("String"),
                };
            }
        }

        // Build: CustomPropValue::Variant(value)
        DecoratedExpr {
            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                callee: self.path_expr(&format!("CustomPropValue::{}", variant)),
                args: vec![rewritten_value],
                is_macro: false,
                optional: false,
                type_args: vec![],
                span: crate::lexer::Span::new(0, 0, 0, 0),
            })),
            metadata: Self::simple_metadata("CustomPropValue"),
        }
    }

    /// Build self.state.method_name expression
    fn build_state_method_path(&self, method_name: &str) -> DecoratedExpr {
        use crate::codegen::decorated_ast::DecoratedExprKind;
        use crate::codegen::swc_metadata::SwcIdentifierMetadata;

        // Build: self.state.method_name
        DecoratedExpr {
            kind: DecoratedExprKind::Member {
                object: Box::new(DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: Box::new(DecoratedExpr {
                            kind: DecoratedExprKind::Ident {
                                name: "self".to_string(),
                                ident_metadata: SwcIdentifierMetadata::name(),
                            },
                            metadata: Self::simple_metadata("&mut Self"),
                        }),
                        property: "state".to_string(),
                        optional: false,
                        computed: false,
                        is_path: false,
                        field_metadata: crate::codegen::swc_metadata::SwcFieldMetadata::direct("state".to_string(), "State".to_string()),
                    },
                    metadata: Self::simple_metadata("&mut State"),
                }),
                property: method_name.to_string(),
                optional: false,
                computed: false,
                is_path: false,
                field_metadata: crate::codegen::swc_metadata::SwcFieldMetadata::direct(method_name.to_string(), "fn".to_string()),
            },
            metadata: Self::simple_metadata("fn"),
        }
    }

    /// Build a string literal expression
    fn string_literal(&self, s: &str) -> DecoratedExpr {
        use crate::codegen::decorated_ast::DecoratedExprKind;
        use crate::parser::Literal;

        DecoratedExpr {
            kind: DecoratedExprKind::Literal(Literal::String(s.to_string())),
            metadata: Self::simple_metadata("&str"),
        }
    }

    /// Build a path expression like "CustomPropValue::Str"
    fn path_expr(&self, path: &str) -> DecoratedExpr {
        use crate::codegen::decorated_ast::DecoratedExprKind;
        use crate::codegen::swc_metadata::SwcIdentifierMetadata;

        DecoratedExpr {
            kind: DecoratedExprKind::Ident {
                name: path.to_string(),
                ident_metadata: SwcIdentifierMetadata::name(),
            },
            metadata: Self::simple_metadata("Path"),
        }
    }

    /// Chain a .and_then(|v| unwrapper) call
    fn chain_and_then(&mut self, expr: DecoratedExpr, unwrapper_pattern: String) -> DecoratedExpr {
        use crate::codegen::decorated_ast::{DecoratedCallExpr, DecoratedExprKind};
        use crate::codegen::swc_metadata::SwcIdentifierMetadata;
        use crate::parser::Literal;

        // For now, we'll emit this as a verbatim closure call
        // TODO: Properly construct a closure DecoratedExpr

        // Build: expr.and_then(|v| unwrapper_pattern)
        DecoratedExpr {
            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                callee: DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: Box::new(expr),
                        property: "and_then".to_string(),
                        optional: false,
                        computed: false,
                        is_path: false,
                        field_metadata: crate::codegen::swc_metadata::SwcFieldMetadata::direct("and_then".to_string(), "fn".to_string()),
                    },
                    metadata: Self::simple_metadata("fn"),
                },
                // For the closure argument, we'll use a special marker that the emitter will handle
                args: vec![DecoratedExpr {
                    kind: DecoratedExprKind::Ident {
                        name: format!("CLOSURE_UNWRAPPER:{}", unwrapper_pattern),
                        ident_metadata: SwcIdentifierMetadata::name(),
                    },
                    metadata: Self::simple_metadata("Closure"),
                }],
                is_macro: false,
                optional: false,
                type_args: vec![],
                span: crate::lexer::Span::new(0, 0, 0, 0),
            })),
            metadata: Self::simple_metadata("Option<T>"),
        }
    }

    /// Extract the innermost binding name from a pattern
    /// Example: Expr::Lit(Lit::Str(__inner)) → Some("__inner")
    fn extract_innermost_binding(&self, pattern: &DecoratedPattern) -> Option<String> {
        match &pattern.kind {
            DecoratedPatternKind::Ident(name) => Some(name.clone()),
            DecoratedPatternKind::Variant { inner, .. } => {
                inner.as_ref().and_then(|p| self.extract_innermost_binding(p))
            }
            DecoratedPatternKind::Tuple(patterns) if patterns.len() == 1 => {
                self.extract_innermost_binding(&patterns[0])
            }
            _ => None,
        }
    }

    /// Rewrite a block, replacing member access on scrutinee with binding
    /// Example: expr.value → __inner.value
    fn rewrite_block_with_scrutinee_replacement(
        &mut self,
        block: DecoratedBlock,
        scrutinee_name: &str,
        binding_name: &str,
    ) -> DecoratedBlock {
        let stmts = block.stmts.into_iter().map(|stmt| {
            self.rewrite_stmt_replacing_scrutinee(stmt, scrutinee_name, binding_name)
        }).collect();
        DecoratedBlock { stmts }
    }

    fn rewrite_stmt_replacing_scrutinee(
        &mut self,
        stmt: DecoratedStmt,
        scrutinee_name: &str,
        binding_name: &str,
    ) -> DecoratedStmt {
        match stmt {
            DecoratedStmt::Expr(expr) => {
                DecoratedStmt::Expr(self.rewrite_expr_replacing_scrutinee(expr, scrutinee_name, binding_name))
            }
            DecoratedStmt::Return(ret) => {
                DecoratedStmt::Return(ret.map(|v| self.rewrite_expr_replacing_scrutinee(v, scrutinee_name, binding_name)))
            }
            DecoratedStmt::Let(let_stmt) => {
                DecoratedStmt::Let(DecoratedLetStmt {
                    init: self.rewrite_expr_replacing_scrutinee(let_stmt.init, scrutinee_name, binding_name),
                    ..let_stmt
                })
            }
            // For other statement types, recursively rewrite
            _ => self.rewrite_stmt(stmt),
        }
    }

    fn rewrite_expr_replacing_scrutinee(
        &mut self,
        expr: DecoratedExpr,
        scrutinee_name: &str,
        binding_name: &str,
    ) -> DecoratedExpr {
        // Check if this is member access on the scrutinee
        if let DecoratedExprKind::Member { object, property, .. } = &expr.kind {
            if let DecoratedExprKind::Ident { name, .. } = &object.kind {
                if name == scrutinee_name {
                    // Replace scrutinee with binding
                    // We need to recompute field_metadata for the new object type (binding)
                    // The binding type is stored in object.metadata.swc_type, but we need
                    // to map the field for the ACTUAL type the binding represents

                    // Keep the original field metadata - the typed version will be used
                    // when called from if-let statements
                    let field_metadata = if let DecoratedExprKind::Member { field_metadata, .. } = &expr.kind {
                        field_metadata.clone()
                    } else {
                        SwcFieldMetadata::direct(property.clone(), "Unknown".to_string())
                    };

                    return DecoratedExpr {
                        kind: DecoratedExprKind::Member {
                            object: Box::new(DecoratedExpr {
                                kind: DecoratedExprKind::Ident {
                                    name: binding_name.to_string(),
                                    ident_metadata: SwcIdentifierMetadata::name(),
                                },
                                metadata: object.metadata.clone(),
                            }),
                            property: property.clone(),
                            optional: false,
                            computed: false,
                            is_path: false,
                            field_metadata,
                        },
                        metadata: expr.metadata.clone(),
                    };
                }
            }
        }

        // For non-member access, recursively process the expression structure
        // but continue looking for scrutinee replacements in nested expressions
        match expr.kind {
            DecoratedExprKind::Call(mut call) => {
                // Recursively replace in callee (e.g., expr.value in expr.value.to_string())
                call.callee = self.rewrite_expr_replacing_scrutinee(call.callee, scrutinee_name, binding_name);
                // Also replace in arguments
                call.args = call.args.into_iter()
                    .map(|arg| self.rewrite_expr_replacing_scrutinee(arg, scrutinee_name, binding_name))
                    .collect();
                DecoratedExpr {
                    kind: DecoratedExprKind::Call(call),
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Binary { mut left, op, mut right, binary_metadata } => {
                left = Box::new(self.rewrite_expr_replacing_scrutinee(*left, scrutinee_name, binding_name));
                right = Box::new(self.rewrite_expr_replacing_scrutinee(*right, scrutinee_name, binding_name));
                DecoratedExpr {
                    kind: DecoratedExprKind::Binary { left, op, right, binary_metadata },
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::If(mut if_expr) => {
                if_expr.condition = self.rewrite_expr_replacing_scrutinee(if_expr.condition, scrutinee_name, binding_name);
                if_expr.then_branch = self.rewrite_block_with_scrutinee_replacement(if_expr.then_branch, scrutinee_name, binding_name);
                if_expr.else_branch = if_expr.else_branch.map(|e| self.rewrite_block_with_scrutinee_replacement(e, scrutinee_name, binding_name));
                DecoratedExpr {
                    kind: DecoratedExprKind::If(if_expr),
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Unary { op, mut operand, unary_metadata } => {
                operand = Box::new(self.rewrite_expr_replacing_scrutinee(*operand, scrutinee_name, binding_name));
                DecoratedExpr {
                    kind: DecoratedExprKind::Unary { op, operand, unary_metadata },
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Paren(mut inner) => {
                inner = Box::new(self.rewrite_expr_replacing_scrutinee(*inner, scrutinee_name, binding_name));
                DecoratedExpr {
                    kind: DecoratedExprKind::Paren(inner),
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Index { mut object, mut index } => {
                object = Box::new(self.rewrite_expr_replacing_scrutinee(*object, scrutinee_name, binding_name));
                index = Box::new(self.rewrite_expr_replacing_scrutinee(*index, scrutinee_name, binding_name));
                DecoratedExpr {
                    kind: DecoratedExprKind::Index { object, index },
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::VecInit(elements) => {
                let elements = elements.into_iter()
                    .map(|e| self.rewrite_expr_replacing_scrutinee(e, scrutinee_name, binding_name))
                    .collect();
                DecoratedExpr {
                    kind: DecoratedExprKind::VecInit(elements),
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Return(value) => {
                let value = value.map(|v| Box::new(self.rewrite_expr_replacing_scrutinee(*v, scrutinee_name, binding_name)));
                DecoratedExpr {
                    kind: DecoratedExprKind::Return(value),
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Assign { mut left, mut right } => {
                left = Box::new(self.rewrite_expr_replacing_scrutinee(*left, scrutinee_name, binding_name));
                right = Box::new(self.rewrite_expr_replacing_scrutinee(*right, scrutinee_name, binding_name));
                DecoratedExpr {
                    kind: DecoratedExprKind::Assign { left, right },
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Match(mut match_expr) => {
                match_expr.expr = self.rewrite_expr_replacing_scrutinee(match_expr.expr, scrutinee_name, binding_name);
                match_expr.arms = match_expr.arms.into_iter().map(|mut arm| {
                    arm.guard = arm.guard.map(|g| self.rewrite_expr_replacing_scrutinee(g, scrutinee_name, binding_name));
                    arm.body = self.rewrite_block_with_scrutinee_replacement(arm.body, scrutinee_name, binding_name);
                    arm
                }).collect();
                DecoratedExpr {
                    kind: DecoratedExprKind::Match(match_expr),
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Block(block) => {
                let block = self.rewrite_block_with_scrutinee_replacement(block, scrutinee_name, binding_name);
                DecoratedExpr {
                    kind: DecoratedExprKind::Block(block),
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Member { mut object, property, optional, computed, is_path, field_metadata } => {
                // This handles non-scrutinee member access (scrutinee member access already handled above)
                // Recursively process the object in case it contains scrutinee access
                // E.g., expr.value.to_string() - object is expr.value (needs replacement)
                object = Box::new(self.rewrite_expr_replacing_scrutinee(*object, scrutinee_name, binding_name));
                DecoratedExpr {
                    kind: DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata },
                    metadata: expr.metadata,
                }
            }
            // For leaf expressions (already handled member access matches above), return as-is
            _ => expr,
        }
    }

    /// Extract the binding type from a pattern
    /// E.g., Expr::Lit(Lit::Str(__inner)) -> "Str"
    ///       Pat::Ident -> "BindingIdent"
    ///       Expr::Ident -> "Ident"
    fn extract_binding_type_from_pattern(&self, pattern: &DecoratedPattern) -> String {
        // Get the swc_pattern from metadata
        let swc_pattern = &pattern.metadata.swc_pattern;

        eprintln!("[DEBUG] extract_binding_type from pattern: {}", swc_pattern);

        // Parse patterns like "Expr::Lit(Lit::Str(__inner))" or "Pat::Ident(__inner)"
        // We want the innermost type before the binding

        // Find the last occurrence of "::" before a "("
        if let Some(last_paren) = swc_pattern.rfind('(') {
            let before_paren = &swc_pattern[..last_paren];
            if let Some(last_colon) = before_paren.rfind("::") {
                let type_name = &before_paren[last_colon + 2..];
                eprintln!("[DEBUG] Extracted type (with binding): {}", type_name);

                // Map variant names to SWC types: Call -> CallExpr, Member -> MemberExpr, etc.
                let swc_type = if swc_pattern.starts_with("Expr::") {
                    format!("{}Expr", type_name)
                } else if swc_pattern.starts_with("Stmt::") {
                    format!("{}Stmt", type_name)
                } else if swc_pattern.starts_with("Pat::") {
                    format!("{}Pat", type_name)
                } else {
                    type_name.to_string()
                };

                return swc_type;
            }
        }

        // No binding in pattern - extract the last part after ::
        // E.g., "Expr::Call" -> "CallExpr", "Pat::Object" -> "ObjectPat"
        if let Some(last_colon) = swc_pattern.rfind("::") {
            let type_name = &swc_pattern[last_colon + 2..];
            eprintln!("[DEBUG] Extracted type (no binding): {}", type_name);

            // Map variant names to SWC types
            let swc_type = if swc_pattern.starts_with("Expr::") {
                format!("{}Expr", type_name)
            } else if swc_pattern.starts_with("Stmt::") {
                format!("{}Stmt", type_name)
            } else if swc_pattern.starts_with("Pat::") {
                if type_name == "Ident" {
                    "BindingIdent".to_string()
                } else {
                    format!("{}Pat", type_name)
                }
            } else {
                type_name.to_string()
            };

            return swc_type;
        }

        // Fallback: Unknown type
        eprintln!("[DEBUG] Could not extract type, using Unknown");
        "Unknown".to_string()
    }

    /// Rewrite block replacing scrutinee with binding, with type information
    fn rewrite_block_with_scrutinee_replacement_typed(
        &mut self,
        block: DecoratedBlock,
        scrutinee_name: &str,
        binding_name: &str,
        binding_type: &str,
    ) -> DecoratedBlock {
        DecoratedBlock {
            stmts: block.stmts.into_iter().map(|stmt| {
                self.rewrite_stmt_replacing_scrutinee_typed(stmt, scrutinee_name, binding_name, binding_type)
            }).collect(),
        }
    }

    fn rewrite_stmt_replacing_scrutinee_typed(
        &mut self,
        stmt: DecoratedStmt,
        scrutinee_name: &str,
        binding_name: &str,
        binding_type: &str,
    ) -> DecoratedStmt {
        match stmt {
            DecoratedStmt::Expr(expr) => {
                DecoratedStmt::Expr(self.rewrite_expr_replacing_scrutinee_typed(expr, scrutinee_name, binding_name, binding_type))
            }
            DecoratedStmt::Let(mut let_stmt) => {
                let_stmt.init = self.rewrite_expr_replacing_scrutinee_typed(let_stmt.init, scrutinee_name, binding_name, binding_type);
                DecoratedStmt::Let(let_stmt)
            }
            DecoratedStmt::Return(value) => {
                DecoratedStmt::Return(value.map(|v| self.rewrite_expr_replacing_scrutinee_typed(v, scrutinee_name, binding_name, binding_type)))
            }
            _ => self.rewrite_stmt(stmt),
        }
    }

    fn rewrite_expr_replacing_scrutinee_typed(
        &mut self,
        expr: DecoratedExpr,
        scrutinee_name: &str,
        binding_name: &str,
        binding_type: &str,
    ) -> DecoratedExpr {
        // Check if this expression IS the scrutinee (e.g., node.expr matches "node.expr")
        if let DecoratedExprKind::Member { ref object, ref property, .. } = expr.kind {
            if let DecoratedExprKind::Ident { ref name, .. } = object.kind {
                let full_path = format!("{}.{}", name, property);
                if full_path == scrutinee_name {
                    // This IS the scrutinee - replace with binding
                    eprintln!("[SCRUTINEE REPLACEMENT] Replacing {} with {} (type: {})", full_path, binding_name, binding_type);
                    return DecoratedExpr {
                        kind: DecoratedExprKind::Ident {
                            name: binding_name.to_string(),
                            ident_metadata: SwcIdentifierMetadata::name(),
                        },
                        metadata: SwcExprMetadata {
                            swc_type: binding_type.to_string(),
                            is_boxed: false,
                            is_optional: false,
                            type_kind: SwcTypeKind::Struct,
                            span: expr.metadata.span,
                            needs_enum_unwrap: None,
                        },
                    };
                }
            }
        }

        // Check if this is member access on the scrutinee
        if let DecoratedExprKind::Member { object, property, .. } = &expr.kind {
            if let DecoratedExprKind::Ident { name, .. } = &object.kind {
                if name == scrutinee_name {
                    // Replace scrutinee with binding
                    // Recompute field metadata based on the binding type
                    let field_metadata = self.get_field_metadata_for_type(binding_type, property);

                    return DecoratedExpr {
                        kind: DecoratedExprKind::Member {
                            object: Box::new(DecoratedExpr {
                                kind: DecoratedExprKind::Ident {
                                    name: binding_name.to_string(),
                                    ident_metadata: SwcIdentifierMetadata::name(),
                                },
                                metadata: object.metadata.clone(),
                            }),
                            property: property.clone(),
                            optional: false,
                            computed: false,
                            is_path: false,
                            field_metadata,
                        },
                        metadata: expr.metadata.clone(),
                    };
                }
            }
        }

        // Recursively process all expression types
        match expr.kind {
            DecoratedExprKind::Call(mut call) => {
                call.callee = self.rewrite_expr_replacing_scrutinee_typed(call.callee, scrutinee_name, binding_name, binding_type);
                call.args = call.args.into_iter()
                    .map(|arg| self.rewrite_expr_replacing_scrutinee_typed(arg, scrutinee_name, binding_name, binding_type))
                    .collect();
                DecoratedExpr {
                    kind: DecoratedExprKind::Call(call),
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Member { mut object, property, optional, computed, is_path, field_metadata } => {
                object = Box::new(self.rewrite_expr_replacing_scrutinee_typed(*object, scrutinee_name, binding_name, binding_type));
                DecoratedExpr {
                    kind: DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata },
                    metadata: expr.metadata,
                }
            }
            // For other expression types, use the non-typed version
            _ => self.rewrite_expr_replacing_scrutinee(expr, scrutinee_name, binding_name),
        }
    }

    /// Get field metadata for a specific type and field name
    fn get_field_metadata_for_type(&self, type_name: &str, field_name: &str) -> SwcFieldMetadata {
        use crate::codegen::type_context::get_typed_field_mapping;

        eprintln!("[REWRITER GET FIELD META] type={}, field={}", type_name, field_name);
        // Try to get typed field mapping
        if let Some(mapping) = get_typed_field_mapping(type_name, field_name) {
            eprintln!("[REWRITER] Found mapping with needs_deref={}", mapping.needs_deref);
            SwcFieldMetadata {
                swc_field_name: mapping.swc_field.to_string(),
                field_type: mapping.result_type_swc.to_string(),
                accessor: if mapping.read_conversion == ".as_bytes()" {
                    FieldAccessor::Utf8Lossy
                } else if mapping.needs_deref {
                    // Check if this is Atom type which needs &* for Display
                    if mapping.result_type_swc == "Atom" {
                        FieldAccessor::DerefDisplay
                    } else {
                        FieldAccessor::BoxedAsRef
                    }
                } else {
                    FieldAccessor::Direct
                },
                source_field: Some(field_name.to_string()),
                span: None,
                read_conversion: mapping.read_conversion.to_string(),
            }
        } else {
            // Fallback: Apply common AST field name mappings
            // These handle cases where type information is not available (e.g., in traverse blocks)
            let swc_field = match field_name {
                // Identifier.name -> Ident.sym
                "name" => "sym",
                // MemberExpression.property -> MemberExpr.prop
                "property" => "prop",
                // MemberExpression.object -> MemberExpr.obj
                "object" => "obj",
                // CallExpression.arguments -> CallExpr.args
                "arguments" => "args",
                // CallExpression.callee -> CallExpr.callee (no change)
                "callee" => "callee",
                // ArrayPattern.elements / ArrayExpression.elements -> elems
                "elements" => "elems",
                // No mapping found
                _ => field_name,
            };

            // Add .to_string() conversion for sym field (Atom -> String)
            let read_conversion = if swc_field == "sym" {
                ".to_string()"
            } else {
                ""
            };

            SwcFieldMetadata {
                swc_field_name: swc_field.to_string(),
                field_type: "Unknown".to_string(),
                accessor: FieldAccessor::Direct,
                source_field: Some(field_name.to_string()),
                span: None,
                read_conversion: read_conversion.to_string(),
            }
        }
    }

    /// Extract variable name from Option guard pattern: if !matches!(x, Some) { return; }
    fn extract_option_guard_variable(if_stmt: &DecoratedIfStmt) -> Option<(String, String)> {
        // Check if condition is: !matches!(var, Some)
        if let DecoratedExprKind::Unary { op, operand, .. } = &if_stmt.condition.kind {
            if *op == UnaryOp::Not {
                if let DecoratedExprKind::Matches { expr, .. } = &operand.kind {
                    // Check if expr is an identifier
                    if let DecoratedExprKind::Ident { name, .. } = &expr.kind {
                        // Check if then-branch is just `return;`
                        if if_stmt.then_branch.stmts.len() == 1 {
                            if matches!(if_stmt.then_branch.stmts[0], DecoratedStmt::Return(_)) {
                                return Some((name.clone(), expr.metadata.swc_type.clone()));
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Create unwrap rebinding statement: let x = x.as_ref().unwrap();
    fn create_unwrap_rebinding(var_name: &str) -> DecoratedStmt {
        use crate::lexer::Span;

        let meta = SwcExprMetadata {
            needs_enum_unwrap: None,
            swc_type: "Unknown".to_string(),
            is_boxed: false,
            is_optional: false,
            type_kind: SwcTypeKind::Unknown,
            span: None,
        };

        // Create: var_name.as_ref().unwrap()
        let unwrap_expr = DecoratedExpr {
            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                callee: DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: Box::new(DecoratedExpr {
                            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                                callee: DecoratedExpr {
                                    kind: DecoratedExprKind::Member {
                                        object: Box::new(DecoratedExpr {
                                            kind: DecoratedExprKind::Ident {
                                                name: var_name.to_string(),
                                                ident_metadata: SwcIdentifierMetadata::name(),
                                            },
                                            metadata: meta.clone(),
                                        }),
                                        property: "as_ref".to_string(),
                                        optional: false,
                                        computed: false,
                                        is_path: false,
                                        field_metadata: SwcFieldMetadata::direct("as_ref".to_string(), "Unknown".to_string()),
                                    },
                                    metadata: meta.clone(),
                                },
                                args: vec![],
                                type_args: vec![],
                                optional: false,
                                is_macro: false,
                                span: Span::new(0, 0, 0, 0),
                            })),
                            metadata: meta.clone(),
                        }),
                        property: "unwrap".to_string(),
                        optional: false,
                        computed: false,
                        is_path: false,
                        field_metadata: SwcFieldMetadata::direct("unwrap".to_string(), "Unknown".to_string()),
                    },
                    metadata: meta.clone(),
                },
                args: vec![],
                type_args: vec![],
                optional: false,
                is_macro: false,
                span: Span::new(0, 0, 0, 0),
            })),
            metadata: meta,
        };

        // Create: let var_name = ...
        DecoratedStmt::Let(DecoratedLetStmt {
            mutable: false,
            pattern: DecoratedPattern {
                kind: DecoratedPatternKind::Ident(var_name.to_string()),
                metadata: SwcPatternMetadata::direct(var_name.to_string()),
            },
            ty: None,
            init: unwrap_expr,
        })
    }

    /// Extract Pat variant from matches! guard: if !matches!(x, ArrayPattern) { return; }
    fn extract_pat_variant_from_guard(if_stmt: &DecoratedIfStmt) -> Option<String> {
        // Check if condition is: !matches!(var, PatVariant)
        if let DecoratedExprKind::Unary { op, operand, .. } = &if_stmt.condition.kind {
            if *op == UnaryOp::Not {
                if let DecoratedExprKind::Matches { pattern, .. } = &operand.kind {
                    eprintln!("[REWRITER] Extracting Pat variant, pattern.kind: {:?}", pattern.kind);
                    // Extract the pattern variant name
                    match &pattern.kind {
                        DecoratedPatternKind::Variant { name, .. } => {
                            // Convert ArrayPattern -> Array, ObjectPattern -> Object, etc.
                            let variant = if name.ends_with("Pattern") {
                                name.trim_end_matches("Pattern").to_string()
                            } else {
                                name.clone()
                            };
                            eprintln!("[REWRITER] Extracted variant from Variant: {}", variant);
                            return Some(variant);
                        }
                        DecoratedPatternKind::Ident(name) => {
                            // The lowering has converted it to just an identifier name
                            // Convert ArrayPattern -> Array, ObjectPattern -> Object, etc.
                            let variant = if name.ends_with("Pattern") {
                                name.trim_end_matches("Pattern").to_string()
                            } else {
                                name.clone()
                            };
                            eprintln!("[REWRITER] Extracted variant from Ident: {}", variant);
                            return Some(variant);
                        }
                        _ => {
                            eprintln!("[REWRITER] Pattern kind is neither Variant nor Ident");
                        }
                    }
                }
            }
        }
        None
    }

    /// Create Pat destructuring statement: let Pat::Array(var_name) = var_name else { return; };
    fn create_pat_destructuring(var_name: &str, pat_variant: &str) -> DecoratedStmt {
        use crate::lexer::Span;
        use crate::parser::VerbatimTarget;

        // Map ReluxScript pattern names to SWC Pat enum variants
        let swc_variant = if pat_variant == "Identifier" {
            "Ident"
        } else {
            pat_variant
        };

        // Use the same variable name for the destructured value to avoid needing to rewrite
        // all subsequent uses of the variable
        // For now, emit a verbatim statement since let-else is complex to construct
        // The emitter will need to handle this specially
        let code = format!("let Pat::{}({}) = {} else {{ return; }};", swc_variant, var_name, var_name);

        DecoratedStmt::Verbatim(crate::parser::VerbatimStmt {
            target: VerbatimTarget::Rust,
            code,
            span: Span::new(0, 0, 0, 0),
        })
    }
}
