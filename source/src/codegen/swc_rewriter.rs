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
}

impl SwcRewriter {
    /// Create new rewriter
    pub fn new() -> Self {
        Self {
            temp_var_counter: 0,
            is_writer: false,
        }
    }

    /// Create new rewriter for writer context
    pub fn new_writer() -> Self {
        Self {
            temp_var_counter: 0,
            is_writer: true,
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
        DecoratedPlugin {
            name: plugin.name,
            body: plugin.body
                .into_iter()
                .map(|item| self.rewrite_plugin_item(item))
                .collect(),
        }
    }

    fn rewrite_writer(&mut self, writer: DecoratedWriter) -> DecoratedWriter {
        DecoratedWriter {
            name: writer.name,
            body: writer.body
                .into_iter()
                .map(|item| self.rewrite_plugin_item(item))
                .collect(),
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
        DecoratedBlock {
            stmts: block.stmts
                .into_iter()
                .map(|s| self.rewrite_stmt(s))
                .collect(),
        }
    }

    fn rewrite_stmt(&mut self, stmt: DecoratedStmt) -> DecoratedStmt {
        match stmt {
            DecoratedStmt::Let(let_stmt) => {
                DecoratedStmt::Let(DecoratedLetStmt {
                    mutable: let_stmt.mutable,
                    pattern: self.rewrite_pattern(let_stmt.pattern),
                    ty: let_stmt.ty,
                    init: self.rewrite_expr(let_stmt.init),
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
                DecoratedStmt::Return(ret_expr.map(|e| self.rewrite_expr(e)))
            }

            DecoratedStmt::Break => DecoratedStmt::Break,

            DecoratedStmt::Continue => DecoratedStmt::Continue,

            DecoratedStmt::Traverse(traverse) => {
                // Traverse statements don't need rewriting
                DecoratedStmt::Traverse(traverse)
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
                // TODO: Transform to state.set_custom_prop() call
                // For now, pass through unchanged
                DecoratedStmt::CustomPropAssignment(assign)
            }
        }
    }

    /// 🔥 CRITICAL: Rewrite if-statements (handles pattern desugaring)
    fn rewrite_if_stmt(&mut self, if_stmt: DecoratedIfStmt) -> DecoratedIfStmt {
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

        let pattern = if_stmt.pattern.map(|p| self.rewrite_pattern(p));
        let then_branch = self.rewrite_block(if_stmt.then_branch);
        let else_branch = if_stmt.else_branch.map(|b| self.rewrite_block(b));

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
                                metadata: SwcExprMetadata {
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
                        metadata: SwcExprMetadata {
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
                metadata: SwcExprMetadata {
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
            // Wrap the condition in a reference
            let rewritten_condition = self.rewrite_expr(condition);
            let ref_condition = DecoratedExpr {
                kind: DecoratedExprKind::Ref {
                    mutable: false,
                    expr: Box::new(rewritten_condition.clone()),
                },
                metadata: rewritten_condition.metadata.clone(),
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
        let expr = self.apply_ast_struct_init(expr);
        let expr = self.apply_matches_expansion(expr);
        let expr = self.apply_iterator_methods(expr);
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
                // Don't rewrite struct init for now
                DecoratedExprKind::StructInit(struct_init)
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
            DecoratedExprKind::Literal(_) |
            DecoratedExprKind::Ident { .. } |
            DecoratedExprKind::Break |
            DecoratedExprKind::Continue |
            DecoratedExprKind::CustomPropAccess(_) |
            DecoratedExprKind::Closure(_) => {
                // TODO: Transform CustomPropAccess to state.get_custom_prop() call
                expr.kind
            }
        };

        DecoratedExpr {
            kind,
            metadata: expr.metadata,
        }
    }

    // ========================================================================
    // TRANSFORMATION: Field Replacements
    // ========================================================================

    /// 🔧 Apply field replacements (e.g., self.builder → self in writers)
    fn apply_field_replacements(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        match &expr.kind {
            DecoratedExprKind::Member { field_metadata, .. } => {
                // Check if this field access should be replaced
                if let FieldAccessor::Replace { with } = &field_metadata.accessor {
                    // Replace the entire member expression with the replacement
                    return DecoratedExpr {
                        kind: DecoratedExprKind::Ident {
                            name: with.clone(),
                            ident_metadata: SwcIdentifierMetadata::name(),
                        },
                        metadata: expr.metadata,
                    };
                }
            }
            _ => {}
        }

        // No replacement needed
        expr
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
    // TRANSFORMATION: AST Struct Initialization
    // ========================================================================

    /// 🔧 Transform AST node struct initialization to add required fields
    /// transforms: Identifier { name: "x" } → Ident { sym: "x".into(), span: DUMMY_SP }
    fn apply_ast_struct_init(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        use crate::parser::{Expr, StructInitExpr, IdentExpr, MemberExpr, CallExpr, Literal};
        use crate::lexer::Span;

        if let DecoratedExprKind::StructInit(ref struct_init) = expr.kind {
            // Check if this is an AST node type that needs transformation
            let swc_type = &expr.metadata.swc_type;

            // For Identifier → Ident, transform the fields
            if struct_init.name == "Identifier" && swc_type == "Ident" {
                let mut new_fields = Vec::new();

                // Map each field - working with undecorated Expr from StructInit
                for (field_name, field_expr) in &struct_init.fields {
                    if field_name == "name" {
                        // name → sym with .into()
                        // Create: field_expr.into()
                        let into_call = Expr::Call(CallExpr {
                            callee: Box::new(Expr::Member(MemberExpr {
                                object: Box::new(field_expr.clone()),
                                property: "into".to_string(),
                                optional: false,
                                computed: false,
                                is_path: false,
                                span: Span::new(0, 0, 0, 0),
                            })),
                            args: vec![],
                            type_args: vec![],
                            optional: false,
                            is_macro: false,
                            span: Span::new(0, 0, 0, 0),
                        });
                        new_fields.push(("sym".to_string(), into_call));
                    } else {
                        new_fields.push((field_name.clone(), field_expr.clone()));
                    }
                }

                // Add required fields that weren't specified
                if !new_fields.iter().any(|(name, _)| name == "span") {
                    new_fields.push((
                        "span".to_string(),
                        Expr::Ident(IdentExpr {
                            name: "DUMMY_SP".to_string(),
                            span: Span::new(0, 0, 0, 0),
                        })
                    ));
                }

                // Add optional: false
                if !new_fields.iter().any(|(name, _)| name == "optional") {
                    new_fields.push((
                        "optional".to_string(),
                        Expr::Literal(Literal::Bool(false))
                    ));
                }

                // Add ctxt: SyntaxContext::empty()
                // Use a simple identifier "SyntaxContext::empty()" as a workaround
                if !new_fields.iter().any(|(name, _)| name == "ctxt") {
                    new_fields.push((
                        "ctxt".to_string(),
                        Expr::Ident(IdentExpr {
                            name: "SyntaxContext::empty()".to_string(),
                            span: Span::new(0, 0, 0, 0),
                        })
                    ));
                }

                // Return transformed struct init with updated fields
                // Wrap in DecoratedExpr so it can be converted with .into() if needed
                let ident_expr = DecoratedExpr {
                    kind: DecoratedExprKind::StructInit(StructInitExpr {
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
                            metadata: SwcExprMetadata {
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
    fn apply_matches_expansion(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        match expr.kind {
            DecoratedExprKind::Matches { expr: scrutinee, pattern } => {
                // The pattern may have already been desugared (in rewrite_pattern)
                // Now we wrap it in a match expression

                // Create the match arms
                let match_arm = DecoratedMatchArm {
                    pattern,
                    guard: None,
                    body: DecoratedBlock {
                        stmts: vec![DecoratedStmt::Expr(DecoratedExpr {
                            kind: DecoratedExprKind::Literal(Literal::Bool(true)),
                            metadata: SwcExprMetadata {
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
                            metadata: SwcExprMetadata {
                                swc_type: "bool".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: crate::type_system::SwcTypeKind::Primitive,
                                span: None,
                            },
                        })],
                    },
                };

                // Create match expression
                DecoratedExpr {
                    kind: DecoratedExprKind::Match(Box::new(DecoratedMatchExpr {
                        expr: *scrutinee,
                        arms: vec![match_arm, wildcard_arm],
                    })),
                    metadata: SwcExprMetadata {
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
                                metadata: SwcExprMetadata {
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
                                metadata: SwcExprMetadata {
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
        SwcExprMetadata {
            swc_type: swc_type.to_string(),
            is_boxed: false,
            is_optional: false,
            type_kind: crate::type_system::SwcTypeKind::Unknown,
            span: None,
        }
    }
}
