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

use super::decorated_ast::*;
use super::swc_metadata::*;
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
        let condition = self.rewrite_expr(if_stmt.condition);
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
            DecoratedIfStmt {
                condition: self.rewrite_expr(condition),
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
        let expr = self.apply_matches_expansion(expr);
        // TODO Phase 4: Apply nested member unwrapping
        // let expr = self.apply_member_unwrapping(expr);

        expr
    }

    /// Recursively rewrite expression children
    fn rewrite_expr_children(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
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

            // Return expressions
            DecoratedExprKind::Return(value) => {
                DecoratedExprKind::Return(value.map(|v| Box::new(self.rewrite_expr(*v))))
            }

            // Leaf expressions that don't need child rewriting
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
    // TRANSFORMATION: Field Replacements
    // ========================================================================

    /// 🔧 Apply field replacements (e.g., self.builder → self in writers)
    fn apply_field_replacements(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // TODO Phase 3: Check field_metadata.accessor for Replace variant
        // For now, pass through unchanged
        expr
    }

    // ========================================================================
    // TRANSFORMATION: Matches! Macro Expansion
    // ========================================================================

    /// 🔧 Expand matches! macro to if-let expression
    fn apply_matches_expansion(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        // TODO Phase 5: Expand matches!(expr, pattern) → if-let
        // For now, pass through unchanged
        expr
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
}
