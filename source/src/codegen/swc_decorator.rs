//! SWC Decorator - Transforms parser AST into decorated AST with SWC semantics
//!
//! This is where the magic happens! The decorator:
//! 1. Walks the original AST
//! 2. Infers SWC types for each expression
//! 3. Transforms patterns based on CONTEXT (what type is being matched?)
//! 4. Annotates field access with correct unwrap strategies
//! 5. Returns a fully decorated AST ready for SWC codegen
//!
//! Example:
//! ```
//! // Input: if let Expression::Identifier(prop) = *member.property
//! // Context: member.property is MemberProp (not Expr!)
//! // Output: DecoratedPattern with swc_pattern = "MemberProp::Ident"
//! ```

use crate::parser::*;
use crate::type_system::{TypeContext, SwcTypeKind};
use crate::mapping::get_node_mapping;
use super::type_context::get_typed_field_mapping;
use super::swc_metadata::*;
use super::decorated_ast::*;
use std::collections::HashMap;

/// SwcDecorator transforms original AST into decorated AST with SWC semantics
pub struct SwcDecorator {
    /// Type environment for flow-sensitive typing
    /// Maps variable names to their SWC types
    type_env: HashMap<String, TypeContext>,

    /// Current function parameters (for type inference)
    current_params: HashMap<String, TypeContext>,
}

impl SwcDecorator {
    pub fn new() -> Self {
        Self {
            type_env: HashMap::new(),
            current_params: HashMap::new(),
        }
    }

    /// Decorate a full program (main entry point)
    pub fn decorate_program(&mut self, program: &Program) -> DecoratedProgram {
        DecoratedProgram {
            decl: self.decorate_top_level_decl(&program.decl),
        }
    }

    fn decorate_top_level_decl(&mut self, decl: &TopLevelDecl) -> DecoratedTopLevelDecl {
        match decl {
            TopLevelDecl::Plugin(plugin) => {
                DecoratedTopLevelDecl::Plugin(self.decorate_plugin_decl(plugin))
            }
            TopLevelDecl::Writer(writer) => {
                DecoratedTopLevelDecl::Writer(self.decorate_writer_decl(writer))
            }
            TopLevelDecl::Interface(_) | TopLevelDecl::Module(_) => {
                // For now, pass through undecorated
                // These don't need SWC-specific decoration
                DecoratedTopLevelDecl::Undecorated(decl.clone())
            }
        }
    }

    fn decorate_plugin_decl(&mut self, plugin: &PluginDecl) -> DecoratedPlugin {
        DecoratedPlugin {
            name: plugin.name.clone(),
            body: plugin.body.iter().map(|item| self.decorate_plugin_item(item)).collect(),
        }
    }

    fn decorate_writer_decl(&mut self, writer: &WriterDecl) -> DecoratedWriter {
        DecoratedWriter {
            name: writer.name.clone(),
            body: writer.body.iter().map(|item| self.decorate_plugin_item(item)).collect(),
        }
    }

    fn decorate_plugin_item(&mut self, item: &PluginItem) -> DecoratedPluginItem {
        match item {
            PluginItem::Function(func) => {
                DecoratedPluginItem::Function(self.decorate_fn_decl(func))
            }
            PluginItem::Struct(struct_decl) => {
                // Structs don't need decoration, pass through
                DecoratedPluginItem::Struct(struct_decl.clone())
            }
            PluginItem::Enum(enum_decl) => {
                // Enums don't need decoration, pass through
                DecoratedPluginItem::Enum(enum_decl.clone())
            }
            PluginItem::Impl(impl_block) => {
                DecoratedPluginItem::Impl(self.decorate_impl_block(impl_block))
            }
            PluginItem::PreHook(func) => {
                DecoratedPluginItem::PreHook(self.decorate_fn_decl(func))
            }
            PluginItem::ExitHook(func) => {
                DecoratedPluginItem::ExitHook(self.decorate_fn_decl(func))
            }
        }
    }

    fn decorate_fn_decl(&mut self, func: &FnDecl) -> DecoratedFnDecl {
        // Clear and register parameter types
        self.current_params.clear();
        for param in &func.params {
            let type_ctx = self.type_annotation_to_context(&param.ty);
            self.current_params.insert(param.name.clone(), type_ctx.clone());
            self.type_env.insert(param.name.clone(), type_ctx);
        }

        // Decorate the function body
        let decorated_body = self.decorate_block(&func.body);

        DecoratedFnDecl {
            name: func.name.clone(),
            params: func.params.clone(),
            return_type: func.return_type.clone(),
            body: decorated_body,
        }
    }

    fn decorate_impl_block(&mut self, impl_block: &ImplBlock) -> DecoratedImplBlock {
        DecoratedImplBlock {
            target: impl_block.target.clone(),
            items: impl_block.items.iter().map(|m| self.decorate_fn_decl(m)).collect(),
        }
    }

    fn decorate_block(&mut self, block: &Block) -> DecoratedBlock {
        DecoratedBlock {
            stmts: block.stmts.iter().map(|s| self.decorate_stmt(s)).collect(),
        }
    }

    fn decorate_stmt(&mut self, stmt: &Stmt) -> DecoratedStmt {
        match stmt {
            Stmt::Let(let_stmt) => {
                // First decorate the initializer to get its type
                let init = self.decorate_expr(&let_stmt.init);
                let init_type = &init.metadata.swc_type;

                // Use that type for pattern decoration
                let pattern = self.decorate_pattern_with_context(&let_stmt.pattern, init_type);

                DecoratedStmt::Let(DecoratedLetStmt {
                    mutable: let_stmt.mutable,
                    pattern,
                    ty: let_stmt.ty.clone(),
                    init,
                })
            }

            Stmt::Const(const_stmt) => {
                let init = self.decorate_expr(&const_stmt.init);

                DecoratedStmt::Const(DecoratedConstStmt {
                    name: const_stmt.name.clone(),
                    ty: const_stmt.ty.clone(),
                    init,
                })
            }

            Stmt::Expr(expr_stmt) => {
                DecoratedStmt::Expr(self.decorate_expr(&expr_stmt.expr))
            }

            Stmt::If(if_stmt) => {
                DecoratedStmt::If(self.decorate_if_stmt(if_stmt))
            }

            Stmt::Match(match_stmt) => {
                // Decorate scrutinee first to get its type
                let expr = self.decorate_expr(&match_stmt.scrutinee);
                let scrutinee_type = expr.metadata.swc_type.clone();

                // Use scrutinee type for all arm patterns
                let arms = match_stmt.arms.iter().map(|arm| {
                    let decorated_body_expr = self.decorate_expr(&arm.body);
                    // Convert body expr to a block with single expression
                    let body_block = DecoratedBlock {
                        stmts: vec![DecoratedStmt::Expr(decorated_body_expr)],
                    };

                    DecoratedMatchArm {
                        pattern: self.decorate_pattern_with_context(&arm.pattern, &scrutinee_type),
                        guard: None, // MatchArm doesn't have guard in this AST
                        body: body_block,
                    }
                }).collect();

                DecoratedStmt::Match(DecoratedMatchStmt { expr, arms })
            }

            Stmt::For(for_stmt) => {
                // Decorate iterator first
                let iter = self.decorate_expr(&for_stmt.iter);

                // Infer element type from iterator
                // For Vec<T>, element type is T
                // For now, use a simplified heuristic
                let iter_type = &iter.metadata.swc_type;
                let element_type = if iter_type.starts_with("Vec<") {
                    // Extract T from Vec<T>
                    iter_type.trim_start_matches("Vec<")
                        .trim_end_matches('>')
                        .to_string()
                } else {
                    // Unknown iterator type, use Unknown
                    "Unknown".to_string()
                };

                let pattern = self.decorate_pattern_with_context(&for_stmt.pattern, &element_type);
                let body = self.decorate_block(&for_stmt.body);

                DecoratedStmt::For(DecoratedForStmt {
                    pattern,
                    iter,
                    body,
                })
            }

            Stmt::While(while_stmt) => {
                let condition = self.decorate_expr(&while_stmt.condition);
                let body = self.decorate_block(&while_stmt.body);

                DecoratedStmt::While(DecoratedWhileStmt {
                    condition,
                    body,
                })
            }

            Stmt::Loop(loop_stmt) => {
                DecoratedStmt::Loop(self.decorate_block(&loop_stmt.body))
            }

            Stmt::Return(ret_stmt) => {
                let value = ret_stmt.value.as_ref().map(|v| self.decorate_expr(v));
                DecoratedStmt::Return(value)
            }

            Stmt::Break(_) => DecoratedStmt::Break,

            Stmt::Continue(_) => DecoratedStmt::Continue,

            Stmt::Traverse(traverse) => DecoratedStmt::Traverse(traverse.clone()),

            Stmt::Function(func_decl) => DecoratedStmt::Function(func_decl.clone()),

            Stmt::Verbatim(verbatim) => DecoratedStmt::Verbatim(verbatim.clone()),
        }
    }

    /// 🔥 THE CRITICAL FUNCTION: Decorate if-let statements with context-aware patterns
    fn decorate_if_stmt(&mut self, if_stmt: &IfStmt) -> DecoratedIfStmt {
        // First, decorate the condition expression to get its type
        let decorated_condition = self.decorate_expr(&if_stmt.condition);

        // Get the SWC type of the condition
        let condition_type = &decorated_condition.metadata.swc_type;

        // If this is an if-let, decorate the pattern with CONTEXT
        let decorated_pattern = if let Some(ref pattern) = if_stmt.pattern {
            // THIS IS THE KEY: We know what type is being matched!
            Some(self.decorate_pattern_with_context(pattern, condition_type))
        } else {
            None
        };

        // Decorate branches
        let then_branch = self.decorate_block(&if_stmt.then_branch);
        let else_branch = if_stmt.else_branch.as_ref().map(|b| self.decorate_block(b));

        DecoratedIfStmt {
            condition: decorated_condition,
            pattern: decorated_pattern,
            then_branch,
            else_branch,
            if_let_metadata: None, // TODO: Add if-let metadata
        }
    }

    /// 🎯 CONTEXT-AWARE PATTERN DECORATION
    /// This is where we solve the MemberProp vs Expr problem!
    fn decorate_pattern_with_context(&mut self, pattern: &Pattern, expected_type: &str) -> DecoratedPattern {
        match pattern {
            Pattern::Variant { name, inner } => {
                // Parse the variant name: "Expression::Identifier" or "Callee::MemberExpression"
                let swc_pattern = if name.contains("::") {
                    let parts: Vec<&str> = name.split("::").collect();
                    if parts.len() == 2 {
                        let relux_enum = parts[0];  // "Expression"
                        let relux_variant = parts[1]; // "Identifier"

                        // 🔥 CONTEXT-AWARE MAPPING
                        // If we're matching against MemberProp, translate differently!
                        if expected_type == "MemberProp" {
                            // Expression::Identifier on MemberProp → MemberProp::Ident
                            if relux_enum == "Expression" && relux_variant == "Identifier" {
                                "MemberProp::Ident".to_string()
                            } else {
                                // Fallback to standard mapping
                                self.map_pattern_to_swc(name)
                            }
                        } else {
                            // Standard mapping for Expr, Stmt, etc.
                            self.map_pattern_to_swc(name)
                        }
                    } else {
                        name.clone()
                    }
                } else {
                    // Simple variants like Some, None, Ok, Err
                    name.clone()
                };

                // Determine unwrap strategy based on expected_type
                let unwrap_strategy = self.determine_unwrap_strategy(expected_type);

                // Recursively decorate inner pattern if present
                let decorated_inner = inner.as_ref().map(|inner_pat| {
                    // The inner type depends on the variant
                    // For MemberProp::Ident, inner is IdentName
                    let inner_type = if swc_pattern == "MemberProp::Ident" {
                        "IdentName"
                    } else if swc_pattern.starts_with("Expr::") {
                        // For Expr::Ident, inner is Ident
                        "Ident"
                    } else {
                        "Unknown"
                    };
                    Box::new(self.decorate_pattern_with_context(inner_pat, inner_type))
                });

                DecoratedPattern {
                    kind: DecoratedPatternKind::Variant {
                        name: name.clone(),
                        inner: decorated_inner,
                    },
                    metadata: SwcPatternMetadata {
                        swc_pattern,
                        unwrap_strategy,
                        inner: None,
                        span: None,
                        source_pattern: Some(name.clone()),
                    },
                }
            }

            Pattern::Ident(name) => {
                // Register this binding in type environment
                // Type is the expected_type we're matching against
                let type_ctx = TypeContext::from_reluxscript(expected_type);
                self.type_env.insert(name.clone(), type_ctx);

                DecoratedPattern {
                    kind: DecoratedPatternKind::Ident(name.clone()),
                    metadata: SwcPatternMetadata::direct(name.clone()),
                }
            }

            Pattern::Wildcard => {
                DecoratedPattern {
                    kind: DecoratedPatternKind::Wildcard,
                    metadata: SwcPatternMetadata::direct("_".to_string()),
                }
            }

            Pattern::Literal(lit) => {
                DecoratedPattern {
                    kind: DecoratedPatternKind::Literal(lit.clone()),
                    metadata: SwcPatternMetadata::direct(format!("{:?}", lit)),
                }
            }

            Pattern::Tuple(patterns) => {
                let decorated_patterns = patterns.iter()
                    .map(|p| self.decorate_pattern_with_context(p, "Unknown"))
                    .collect();

                DecoratedPattern {
                    kind: DecoratedPatternKind::Tuple(decorated_patterns),
                    metadata: SwcPatternMetadata::direct("Tuple".to_string()),
                }
            }

            Pattern::Struct { name, fields } => {
                let decorated_fields = fields.iter()
                    .map(|(fname, fpat)| (fname.clone(), self.decorate_pattern_with_context(fpat, "Unknown")))
                    .collect();

                DecoratedPattern {
                    kind: DecoratedPatternKind::Struct {
                        name: name.clone(),
                        fields: decorated_fields,
                    },
                    metadata: SwcPatternMetadata::direct(name.clone()),
                }
            }

            Pattern::Array(patterns) => {
                let decorated_patterns = patterns.iter()
                    .map(|p| self.decorate_pattern_with_context(p, "Unknown"))
                    .collect();

                DecoratedPattern {
                    kind: DecoratedPatternKind::Array(decorated_patterns),
                    metadata: SwcPatternMetadata::direct("Array".to_string()),
                }
            }

            Pattern::Object(props) => {
                DecoratedPattern {
                    kind: DecoratedPatternKind::Object(props.clone()),
                    metadata: SwcPatternMetadata::direct("Object".to_string()),
                }
            }

            Pattern::Rest(inner) => {
                let decorated_inner = Box::new(self.decorate_pattern_with_context(inner, "Unknown"));

                DecoratedPattern {
                    kind: DecoratedPatternKind::Rest(decorated_inner),
                    metadata: SwcPatternMetadata::direct("Rest".to_string()),
                }
            }

            Pattern::Or(patterns) => {
                let decorated_patterns = patterns.iter()
                    .map(|p| self.decorate_pattern_with_context(p, expected_type))
                    .collect();

                DecoratedPattern {
                    kind: DecoratedPatternKind::Or(decorated_patterns),
                    metadata: SwcPatternMetadata::direct("Or".to_string()),
                }
            }

            Pattern::Ref { is_mut, pattern: inner } => {
                let decorated_inner = Box::new(self.decorate_pattern_with_context(inner, expected_type));

                DecoratedPattern {
                    kind: DecoratedPatternKind::Ref {
                        is_mut: *is_mut,
                        pattern: decorated_inner,
                    },
                    metadata: SwcPatternMetadata::direct("Ref".to_string()),
                }
            }
        }
    }

    /// Decorate an expression and infer its SWC type
    fn decorate_expr(&mut self, expr: &Expr) -> DecoratedExpr {
        match expr {
            Expr::Ident(ident_expr) => {
                let name = &ident_expr.name;

                // Look up type in environment
                let type_ctx = self.type_env.get(name)
                    .cloned()
                    .unwrap_or_else(|| TypeContext::unknown());

                DecoratedExpr {
                    kind: DecoratedExprKind::Ident {
                        name: name.clone(),
                        ident_metadata: SwcIdentifierMetadata::name(),
                    },
                    metadata: SwcExprMetadata {
                        swc_type: type_ctx.swc_type.clone(),
                        is_boxed: type_ctx.is_boxed(),
                        is_optional: false,
                        type_kind: type_ctx.kind.clone(),
                        span: Some(ident_expr.span),
                    },
                }
            }

            Expr::Member(mem) => {
                // First, decorate the object to get its type
                let decorated_object = Box::new(self.decorate_expr(&mem.object));
                let object_type = &decorated_object.metadata.swc_type;

                // Look up the field in SWC schema
                let field_metadata = if let Some(mapping) = get_typed_field_mapping(object_type, &mem.property) {
                    // We have precise mapping!
                    SwcFieldMetadata {
                        swc_field_name: mapping.swc_field.to_string(),
                        accessor: if mapping.needs_deref {
                            FieldAccessor::BoxedAsRef
                        } else {
                            FieldAccessor::Direct
                        },
                        field_type: mapping.result_type_swc.to_string(),
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                    }
                } else {
                    // Fallback field mapping
                    let swc_field = match mem.property.as_str() {
                        "object" => "obj",
                        "property" => "prop",
                        "callee" => "callee",
                        "arguments" => "args",
                        _ => &mem.property,
                    };

                    SwcFieldMetadata {
                        swc_field_name: swc_field.to_string(),
                        accessor: FieldAccessor::Direct,
                        field_type: "Unknown".to_string(),
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                    }
                };

                // Infer the type of this member expression
                let member_type = field_metadata.field_type.clone();

                DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: decorated_object,
                        property: mem.property.clone(),
                        optional: mem.optional,
                        computed: mem.computed,
                        is_path: mem.is_path,
                        field_metadata: field_metadata.clone(),
                    },
                    metadata: SwcExprMetadata {
                        swc_type: member_type,
                        is_boxed: matches!(field_metadata.accessor, FieldAccessor::BoxedAsRef | FieldAccessor::BoxedRefDeref),
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown, // TODO: Infer properly
                        span: Some(mem.span),
                    },
                }
            }

            Expr::Unary(unary) => {
                let decorated_operand = Box::new(self.decorate_expr(&unary.operand));

                DecoratedExpr {
                    kind: DecoratedExprKind::Unary {
                        op: unary.op,
                        operand: decorated_operand,
                        unary_metadata: SwcUnaryMetadata {
                            override_op: None,
                            span: Some(unary.span),
                        },
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "Unknown".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(unary.span),
                    },
                }
            }

            Expr::Literal(lit) => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Literal(lit.clone()),
                    metadata: SwcExprMetadata {
                        swc_type: "Literal".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Primitive,
                        span: None,
                    },
                }
            }

            Expr::Binary(bin) => {
                let left = Box::new(self.decorate_expr(&bin.left));
                let right = Box::new(self.decorate_expr(&bin.right));

                DecoratedExpr {
                    kind: DecoratedExprKind::Binary {
                        left,
                        op: bin.op,
                        right,
                        binary_metadata: SwcBinaryMetadata {
                            left_needs_deref: false,
                            right_needs_deref: false,
                            span: Some(bin.span),
                        },
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "bool".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Primitive,
                        span: Some(bin.span),
                    },
                }
            }

            Expr::Call(call) => {
                let callee = self.decorate_expr(&call.callee);
                let args = call.args.iter().map(|a| self.decorate_expr(a)).collect();

                DecoratedExpr {
                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                        callee,
                        args,
                        type_args: call.type_args.clone(),
                        optional: call.optional,
                        span: call.span,
                    })),
                    metadata: SwcExprMetadata {
                        swc_type: "Unknown".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(call.span),
                    },
                }
            }

            Expr::Paren(inner) => {
                let decorated_inner = Box::new(self.decorate_expr(inner));
                let metadata = decorated_inner.metadata.clone();

                DecoratedExpr {
                    kind: DecoratedExprKind::Paren(decorated_inner),
                    metadata,
                }
            }

            Expr::Block(block) => {
                let decorated_block = self.decorate_block(block);

                DecoratedExpr {
                    kind: DecoratedExprKind::Block(decorated_block),
                    metadata: SwcExprMetadata {
                        swc_type: "Unknown".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    },
                }
            }

            Expr::Index(index) => {
                let object = Box::new(self.decorate_expr(&index.object));
                let index_expr = Box::new(self.decorate_expr(&index.index));

                DecoratedExpr {
                    kind: DecoratedExprKind::Index {
                        object,
                        index: index_expr,
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "Unknown".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(index.span),
                    },
                }
            }

            Expr::StructInit(struct_init) => {
                DecoratedExpr {
                    kind: DecoratedExprKind::StructInit(struct_init.clone()),
                    metadata: SwcExprMetadata {
                        swc_type: struct_init.name.clone(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Struct,
                        span: Some(struct_init.span),
                    },
                }
            }

            Expr::VecInit(vec_init) => {
                let elements = vec_init.elements.iter().map(|e| self.decorate_expr(e)).collect();

                DecoratedExpr {
                    kind: DecoratedExprKind::VecInit(elements),
                    metadata: SwcExprMetadata {
                        swc_type: "Vec".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(vec_init.span),
                    },
                }
            }

            Expr::If(if_expr) => {
                let condition = self.decorate_expr(&if_expr.condition);
                let then_branch = self.decorate_block(&if_expr.then_branch);
                let else_branch = if_expr.else_branch.as_ref().map(|b| self.decorate_block(b));

                DecoratedExpr {
                    kind: DecoratedExprKind::If(Box::new(DecoratedIfExpr {
                        condition,
                        then_branch,
                        else_branch,
                    })),
                    metadata: SwcExprMetadata {
                        swc_type: "Unknown".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    },
                }
            }

            Expr::Match(match_expr) => {
                // Decorate scrutinee first to get its type
                let expr = self.decorate_expr(&match_expr.scrutinee);
                let scrutinee_type = expr.metadata.swc_type.clone();

                // Use scrutinee type for all arm patterns
                let arms = match_expr.arms.iter().map(|arm| {
                    let decorated_body_expr = self.decorate_expr(&arm.body);
                    let body_block = DecoratedBlock {
                        stmts: vec![DecoratedStmt::Expr(decorated_body_expr)],
                    };

                    DecoratedMatchArm {
                        pattern: self.decorate_pattern_with_context(&arm.pattern, &scrutinee_type),
                        guard: None,
                        body: body_block,
                    }
                }).collect();

                DecoratedExpr {
                    kind: DecoratedExprKind::Match(Box::new(DecoratedMatchExpr {
                        expr,
                        arms,
                    })),
                    metadata: SwcExprMetadata {
                        swc_type: "Unknown".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    },
                }
            }

            Expr::Closure(closure) => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Closure(closure.clone()),
                    metadata: SwcExprMetadata {
                        swc_type: "Closure".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(closure.span),
                    },
                }
            }

            Expr::Ref(ref_expr) => {
                let expr = Box::new(self.decorate_expr(&ref_expr.expr));

                DecoratedExpr {
                    kind: DecoratedExprKind::Ref {
                        mutable: ref_expr.mutable,
                        expr,
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "Reference".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(ref_expr.span),
                    },
                }
            }

            Expr::Deref(deref_expr) => {
                let expr = Box::new(self.decorate_expr(&deref_expr.expr));

                DecoratedExpr {
                    kind: DecoratedExprKind::Deref(expr),
                    metadata: SwcExprMetadata {
                        swc_type: "Unknown".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(deref_expr.span),
                    },
                }
            }

            Expr::Assign(assign) => {
                let left = Box::new(self.decorate_expr(&assign.target));
                let right = Box::new(self.decorate_expr(&assign.value));

                DecoratedExpr {
                    kind: DecoratedExprKind::Assign { left, right },
                    metadata: SwcExprMetadata {
                        swc_type: "()".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Primitive,
                        span: Some(assign.span),
                    },
                }
            }

            Expr::CompoundAssign(compound) => {
                let left = Box::new(self.decorate_expr(&compound.target));
                let right = Box::new(self.decorate_expr(&compound.value));

                DecoratedExpr {
                    kind: DecoratedExprKind::CompoundAssign {
                        left,
                        op: compound.op,
                        right,
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "()".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Primitive,
                        span: Some(compound.span),
                    },
                }
            }

            Expr::Range(range) => {
                let start = range.start.as_ref().map(|s| Box::new(self.decorate_expr(s)));
                let end = range.end.as_ref().map(|e| Box::new(self.decorate_expr(e)));

                DecoratedExpr {
                    kind: DecoratedExprKind::Range {
                        start,
                        end,
                        inclusive: range.inclusive,
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "Range".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(range.span),
                    },
                }
            }

            Expr::Try(try_expr) => {
                let expr = Box::new(self.decorate_expr(try_expr));

                DecoratedExpr {
                    kind: DecoratedExprKind::Try(expr),
                    metadata: SwcExprMetadata {
                        swc_type: "Unknown".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    },
                }
            }

            Expr::Tuple(tuple) => {
                let elements = tuple.iter().map(|e| self.decorate_expr(e)).collect();

                DecoratedExpr {
                    kind: DecoratedExprKind::Tuple(elements),
                    metadata: SwcExprMetadata {
                        swc_type: "Tuple".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    },
                }
            }

            Expr::Matches(matches) => {
                // Decorate scrutinee first to get its type
                let expr = Box::new(self.decorate_expr(&matches.scrutinee));
                let scrutinee_type = expr.metadata.swc_type.clone();

                // Use scrutinee type for pattern
                let pattern = self.decorate_pattern_with_context(&matches.pattern, &scrutinee_type);

                DecoratedExpr {
                    kind: DecoratedExprKind::Matches {
                        expr,
                        pattern,
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "bool".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Primitive,
                        span: Some(matches.span),
                    },
                }
            }

            Expr::Return(ret) => {
                let value = ret.as_ref().map(|v| Box::new(self.decorate_expr(v)));

                DecoratedExpr {
                    kind: DecoratedExprKind::Return(value),
                    metadata: SwcExprMetadata {
                        swc_type: "!".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    },
                }
            }

            Expr::Break => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Break,
                    metadata: SwcExprMetadata {
                        swc_type: "!".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    },
                }
            }

            Expr::Continue => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Continue,
                    metadata: SwcExprMetadata {
                        swc_type: "!".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    },
                }
            }
        }
    }

    /// Map ReluxScript pattern to SWC pattern
    fn map_pattern_to_swc(&self, relux_pattern: &str) -> String {
        if let Some(mapping) = get_node_mapping(relux_pattern) {
            mapping.swc_pattern.to_string()
        } else if relux_pattern.contains("::") {
            // Parse and convert: Expression::Identifier → Expr::Ident
            let parts: Vec<&str> = relux_pattern.split("::").collect();
            if parts.len() == 2 {
                let swc_enum = self.reluxscript_to_swc_type(parts[0]);
                let swc_variant = self.reluxscript_to_swc_type(parts[1]);
                format!("{}::{}", swc_enum, swc_variant)
            } else {
                relux_pattern.to_string()
            }
        } else {
            relux_pattern.to_string()
        }
    }

    /// Convert ReluxScript type name to SWC type name
    fn reluxscript_to_swc_type(&self, relux_type: &str) -> String {
        match relux_type {
            "Expression" => "Expr",
            "Statement" => "Stmt",
            "Declaration" => "Decl",
            "Identifier" => "Ident",
            "MemberExpression" => "Member",
            "CallExpression" => "Call",
            "BinaryExpression" => "Bin",
            "UnaryExpression" => "Unary",
            _ => relux_type,
        }.to_string()
    }

    /// Determine unwrap strategy based on type being matched
    fn determine_unwrap_strategy(&self, expected_type: &str) -> UnwrapStrategy {
        // If matching against a Box<T>, need unwrapping
        if expected_type.starts_with("Box<") {
            UnwrapStrategy::RefDeref
        } else {
            UnwrapStrategy::None
        }
    }

    /// Convert type annotation to type context
    fn type_annotation_to_context(&self, type_ann: &Type) -> TypeContext {
        match type_ann {
            Type::Reference { inner, .. } => {
                // For reference types like &mut CallExpression, get the inner type
                self.type_annotation_to_context(inner)
            }
            Type::Primitive(name) => {
                // Primitive type like CallExpression, Identifier, etc.
                TypeContext::from_reluxscript(name)
            }
            Type::Container { name, .. } => {
                // Container type like Vec<T>, Option<T>
                TypeContext::from_reluxscript(name)
            }
            _ => TypeContext::unknown(),
        }
    }
}

// ============================================================================
// Decorated AST structures
// ============================================================================

#[derive(Debug, Clone)]
pub struct DecoratedProgram {
    pub decl: DecoratedTopLevelDecl,
}

#[derive(Debug, Clone)]
pub enum DecoratedTopLevelDecl {
    Plugin(DecoratedPlugin),
    Writer(DecoratedWriter),
    Undecorated(TopLevelDecl),
}

#[derive(Debug, Clone)]
pub struct DecoratedPlugin {
    pub name: String,
    pub body: Vec<DecoratedPluginItem>,
}

#[derive(Debug, Clone)]
pub struct DecoratedWriter {
    pub name: String,
    pub body: Vec<DecoratedPluginItem>,
}

#[derive(Debug, Clone)]
pub enum DecoratedPluginItem {
    Function(DecoratedFnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Impl(DecoratedImplBlock),
    PreHook(DecoratedFnDecl),
    ExitHook(DecoratedFnDecl),
}

#[derive(Debug, Clone)]
pub struct DecoratedFnDecl {
    pub name: String,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub body: DecoratedBlock,
}

#[derive(Debug, Clone)]
pub struct DecoratedImplBlock {
    pub target: String,
    pub items: Vec<DecoratedFnDecl>,
}
