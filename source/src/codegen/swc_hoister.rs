//! SWC Hoister - Extracts inline visitor structs from traverse statements
//!
//! This stage transforms the decorated AST by:
//! 1. Finding inline traverse statements
//! 2. Extracting inline visitor definitions into top-level struct declarations
//! 3. Replacing traverse statements with visitor instantiation + visit_mut_with calls
//!
//! Pipeline position: Decorator → Rewriter → **Hoister** → Emitter

use crate::codegen::swc_decorator::{
    DecoratedProgram, DecoratedTopLevelDecl, DecoratedPlugin, DecoratedWriter,
    DecoratedPluginItem, DecoratedFnDecl, DecoratedImplBlock,
};
use crate::codegen::decorated_ast::{
    DecoratedStmt, DecoratedExpr, DecoratedExprKind,
    DecoratedBlock, DecoratedIfStmt, DecoratedWhileStmt,
    DecoratedForStmt,
};
use crate::codegen::swc_metadata::SwcExprMetadata;
use crate::parser::{
    TraverseStmt, TraverseKind, Pattern, Type, Block, FnDecl,
    StructDecl, StructField, Param,
};
use crate::lexer::Span;
use crate::mapping::get_node_mapping;
use std::collections::HashSet;

pub struct SwcHoister {
    /// Counter for generating unique visitor struct names
    visitor_counter: usize,

    /// Hoisted visitor struct declarations to be added at module level
    hoisted_structs: Vec<StructDecl>,

    /// Hoisted impl blocks for visitor structs
    hoisted_impls: Vec<DecoratedImplBlock>,
}

impl SwcHoister {
    pub fn new() -> Self {
        Self {
            visitor_counter: 0,
            hoisted_structs: Vec::new(),
            hoisted_impls: Vec::new(),
        }
    }

    /// Convert ReluxScript/Babel type name to SWC type name
    fn reluxscript_type_to_swc(&self, type_name: &str) -> String {
        // Use mapping module to convert ReluxScript AST types to SWC types
        get_node_mapping(type_name)
            .map(|m| m.swc.to_string())
            .unwrap_or_else(|| type_name.to_string())
    }

    /// Convert visitor method name to SWC VisitMut method name
    fn visitor_method_to_swc(&self, method_name: &str) -> String {
        // Convert visit_xxx to visit_mut_xxx
        if let Some(stripped) = method_name.strip_prefix("visit_") {
            format!("visit_mut_{}", stripped)
        } else {
            method_name.to_string()
        }
    }

    /// Main entry point: hoist inline visitors from a program
    pub fn hoist_program(&mut self, program: DecoratedProgram) -> DecoratedProgram {
        let new_decl = match program.decl {
            DecoratedTopLevelDecl::Plugin(plugin) => {
                DecoratedTopLevelDecl::Plugin(self.hoist_plugin(plugin))
            }
            DecoratedTopLevelDecl::Writer(writer) => {
                DecoratedTopLevelDecl::Writer(self.hoist_writer(writer))
            }
            other => other,
        };

        DecoratedProgram {
            uses: program.uses,
            decl: new_decl,
        }
    }

    /// Hoist inline visitors from a plugin
    pub fn hoist_plugin(&mut self, plugin: DecoratedPlugin) -> DecoratedPlugin {
        let mut new_body = Vec::new();

        // First pass: process items and collect hoisted structs
        for item in plugin.body {
            match item {
                DecoratedPluginItem::Function(func) => {
                    let new_func = self.hoist_function(func);
                    new_body.push(DecoratedPluginItem::Function(new_func));
                }
                other => new_body.push(other),
            }
        }

        // Second pass: prepend hoisted structs and impls at the beginning
        let mut final_body = Vec::new();

        // Add hoisted structs first
        for hoisted_struct in std::mem::take(&mut self.hoisted_structs) {
            final_body.push(DecoratedPluginItem::Struct(hoisted_struct));
        }

        // Add hoisted impl blocks
        for hoisted_impl in std::mem::take(&mut self.hoisted_impls) {
            final_body.push(DecoratedPluginItem::Impl(hoisted_impl));
        }

        // Add original items
        final_body.extend(new_body);

        DecoratedPlugin {
            body: final_body,
            ..plugin
        }
    }

    /// Hoist inline visitors from a writer
    fn hoist_writer(&mut self, writer: DecoratedWriter) -> DecoratedWriter {
        let mut new_body = Vec::new();

        // First pass: process items and collect hoisted structs
        for item in writer.body {
            match item {
                DecoratedPluginItem::Function(func) => {
                    let new_func = self.hoist_function(func);
                    new_body.push(DecoratedPluginItem::Function(new_func));
                }
                other => new_body.push(other),
            }
        }

        // Second pass: collect hoisted structs for module-level emission
        let hoisted_structs_vec = std::mem::take(&mut self.hoisted_structs);

        // Prepend impl blocks to body (they go in impl block)
        let mut final_body = Vec::new();
        for hoisted_impl in std::mem::take(&mut self.hoisted_impls) {
            final_body.push(DecoratedPluginItem::Impl(hoisted_impl));
        }
        final_body.extend(new_body);

        DecoratedWriter {
            body: final_body,
            hoisted_structs: [writer.hoisted_structs.clone(), hoisted_structs_vec].concat(),
            ..writer
        }
    }

    /// Hoist inline visitors from a function
    fn hoist_function(&mut self, func: DecoratedFnDecl) -> DecoratedFnDecl {
        let new_body = self.hoist_block(func.body);
        DecoratedFnDecl {
            body: new_body,
            ..func
        }
    }

    /// Hoist inline visitors from a block
    fn hoist_block(&mut self, block: DecoratedBlock) -> DecoratedBlock {
        let mut new_stmts = Vec::new();

        for stmt in block.stmts {
            let new_stmt = self.hoist_stmt(stmt);
            new_stmts.push(new_stmt);
        }

        DecoratedBlock {
            stmts: new_stmts,
        }
    }

    /// Hoist inline visitors from a statement
    fn hoist_stmt(&mut self, stmt: DecoratedStmt) -> DecoratedStmt {
        match stmt {
            DecoratedStmt::Traverse(traverse) => {
                self.hoist_traverse(traverse)
            }

            DecoratedStmt::If(if_stmt) => {
                DecoratedStmt::If(DecoratedIfStmt {
                    condition: if_stmt.condition,
                    then_branch: self.hoist_block(if_stmt.then_branch),
                    else_branch: if_stmt.else_branch.map(|b| self.hoist_block(b)),
                    pattern: if_stmt.pattern,
                    if_let_metadata: if_stmt.if_let_metadata,
                })
            }

            DecoratedStmt::While(while_stmt) => {
                DecoratedStmt::While(DecoratedWhileStmt {
                    condition: while_stmt.condition,
                    body: self.hoist_block(while_stmt.body),
                })
            }

            DecoratedStmt::For(for_stmt) => {
                DecoratedStmt::For(DecoratedForStmt {
                    pattern: for_stmt.pattern,
                    iter: for_stmt.iter,
                    body: self.hoist_block(for_stmt.body),
                })
            }

            DecoratedStmt::Loop(block) => {
                DecoratedStmt::Loop(self.hoist_block(block))
            }

            // Other statements don't contain traverse blocks
            other => other,
        }
    }

    /// Transform a traverse statement into visitor instantiation + call
    fn hoist_traverse(&mut self, traverse: TraverseStmt) -> DecoratedStmt {
        match &traverse.kind {
            TraverseKind::Inline(inline) => {
                // Generate unique struct name
                let struct_name = format!("__InlineVisitor_{}", self.visitor_counter);
                self.visitor_counter += 1;

                // Determine if we need lifetime parameters
                let has_captures = !traverse.captures.is_empty();

                // Build the hoisted struct
                let mut struct_fields = Vec::new();

                // Add captured variables as fields
                for capture in &traverse.captures {
                    let field_type = if capture.mutable {
                        // &mut T
                        Type::Reference {
                            inner: Box::new(Type::Primitive("i32".to_string())), // TODO: proper type inference
                            mutable: true,
                        }
                    } else {
                        // &T
                        Type::Reference {
                            inner: Box::new(Type::Primitive("i32".to_string())),
                            mutable: false,
                        }
                    };

                    struct_fields.push(StructField {
                        name: capture.name.clone(),
                        ty: field_type,
                        span: capture.span,
                    });
                }

                // Add local state fields
                for let_stmt in &inline.state {
                    if let Pattern::Ident(name) = &let_stmt.pattern {
                        let field_type = if let Some(ref ty) = let_stmt.ty {
                            ty.clone()
                        } else {
                            Type::Primitive("i32".to_string())
                        };

                        struct_fields.push(StructField {
                            name: name.clone(),
                            ty: field_type,
                            span: let_stmt.span,
                        });
                    }
                }

                // Create the struct
                let hoisted_struct = StructDecl {
                    name: struct_name.clone(),
                    fields: struct_fields,
                    derives: vec![], // TODO: Add derives if needed
                    span: traverse.span,
                };

                self.hoisted_structs.push(hoisted_struct);

                // Create the impl VisitMut block
                let mut impl_methods = Vec::new();

                for method in &inline.methods {
                    // Get parameter type and name from the first parameter
                    let (param_name, param_type, swc_type_name) = if !method.params.is_empty() {
                        let raw_type = &method.params[0].ty;
                        // Extract the inner type name and translate to SWC
                        let swc_type_name = match raw_type {
                            Type::Reference { inner, .. } => {
                                match inner.as_ref() {
                                    Type::Named(name) => self.reluxscript_type_to_swc(name),
                                    _ => "Expr".to_string(),
                                }
                            }
                            Type::Named(name) => self.reluxscript_type_to_swc(name),
                            _ => "Expr".to_string(),
                        };
                        (
                            method.params[0].name.clone(),
                            Type::Named(swc_type_name.clone()),
                            swc_type_name
                        )
                    } else {
                        ("n".to_string(), Type::Named("Expr".to_string()), "Expr".to_string())
                    };

                    // Generate method name from the SWC type: VarDeclarator → visit_mut_var_declarator
                    let swc_method_name = format!("visit_mut_{}",
                        swc_type_name
                            .chars()
                            .enumerate()
                            .flat_map(|(i, c)| {
                                if i > 0 && c.is_uppercase() {
                                    vec!['_', c.to_lowercase().next().unwrap()]
                                } else {
                                    vec![c.to_lowercase().next().unwrap()]
                                }
                            })
                            .collect::<String>()
                    );

                    // Convert method body, marking captured variables
                    let method_body = self.transform_method_body_with_captures(
                        &method.body,
                        &traverse.captures,
                        &inline.state,
                    );

                    let impl_method = DecoratedFnDecl {
                        name: swc_method_name,
                        params: vec![
                            Param {
                                name: "self".to_string(),
                                ty: Type::Reference {
                                    inner: Box::new(Type::Named("Self".to_string())),
                                    mutable: true,
                                },
                                span: Span { start: 0, end: 0, line: 0, column: 0 },
                            },
                            Param {
                                name: param_name,
                                ty: Type::Reference {
                                    inner: Box::new(param_type),
                                    mutable: true,
                                },
                                span: Span { start: 0, end: 0, line: 0, column: 0 },
                            },
                        ],
                        return_type: None,
                        body: method_body,
                    };

                    impl_methods.push(impl_method);
                }

                let impl_block = DecoratedImplBlock {
                    target: format!("VisitMut for {}", struct_name),
                    items: impl_methods,
                };

                self.hoisted_impls.push(impl_block);

                // Now generate the instantiation code at the traverse site
                self.generate_visitor_instantiation(&struct_name, &traverse, &inline.state)
            }

            TraverseKind::Delegated(visitor_name) => {
                // For delegated visitors, just generate the instantiation + call
                // TODO: Generate proper delegation code
                DecoratedStmt::Expr(DecoratedExpr {
                    kind: DecoratedExprKind::Literal(crate::parser::Literal::String(
                        format!("/* TODO: Delegate to {} */", visitor_name)
                    )),
                    metadata: SwcExprMetadata {
                        swc_type: "()".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                        span: Some(traverse.span),
                    },
                })
            }
        }
    }

    /// Generate visitor instantiation and visit_mut_with call
    fn generate_visitor_instantiation(
        &self,
        struct_name: &str,
        traverse: &TraverseStmt,
        state: &[crate::parser::LetStmt],
    ) -> DecoratedStmt {
        use crate::codegen::decorated_ast::{DecoratedCallExpr, DecoratedStructInit};
        use crate::parser::Literal;

        // Build struct initialization: __InlineVisitor_0 { capture1: &mut var1, state1: init1, ... }
        let mut fields = Vec::new();

        // Add captured variables as fields
        for capture in &traverse.captures {
            let field_expr = if capture.mutable {
                // &mut capture
                DecoratedExpr {
                    kind: DecoratedExprKind::Unary {
                        op: crate::parser::UnaryOp::RefMut,
                        operand: Box::new(DecoratedExpr {
                            kind: DecoratedExprKind::Ident {
                                name: capture.name.clone(),
                                ident_metadata: crate::codegen::swc_metadata::SwcIdentifierMetadata {
                                    use_sym: false,
                                    deref_pattern: None,
                                    span: Some(capture.span),
                                },
                            },
                            metadata: SwcExprMetadata {
                                swc_type: "i32".to_string(), // TODO: proper type
                                is_boxed: false,
                                is_optional: false,
                                type_kind: crate::type_system::SwcTypeKind::Unknown,
                                span: Some(capture.span),
                            },
                        }),
                        unary_metadata: crate::codegen::swc_metadata::SwcUnaryMetadata {
                            override_op: None,
                            span: Some(capture.span),
                        },
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "&mut i32".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                        span: Some(capture.span),
                    },
                }
            } else {
                // &capture
                DecoratedExpr {
                    kind: DecoratedExprKind::Unary {
                        op: crate::parser::UnaryOp::Ref,
                        operand: Box::new(DecoratedExpr {
                            kind: DecoratedExprKind::Ident {
                                name: capture.name.clone(),
                                ident_metadata: crate::codegen::swc_metadata::SwcIdentifierMetadata {
                                    use_sym: false,
                                    deref_pattern: None,
                                    span: Some(capture.span),
                                },
                            },
                            metadata: SwcExprMetadata {
                                swc_type: "i32".to_string(),
                                is_boxed: false,
                                is_optional: false,
                                type_kind: crate::type_system::SwcTypeKind::Unknown,
                                span: Some(capture.span),
                            },
                        }),
                        unary_metadata: crate::codegen::swc_metadata::SwcUnaryMetadata {
                            override_op: None,
                            span: Some(capture.span),
                        },
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "&i32".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                        span: Some(capture.span),
                    },
                }
            };

            fields.push((capture.name.clone(), field_expr));
        }

        // Add state initialization fields
        for let_stmt in state {
            if let Pattern::Ident(name) = &let_stmt.pattern {
                // Decorate the init expression
                let mut decorator = crate::codegen::swc_decorator::SwcDecorator::new();
                let init_expr = decorator.decorate_expr(&let_stmt.init);

                fields.push((name.clone(), init_expr));
            }
        }

        // Create the struct initialization
        let struct_init = DecoratedExpr {
            kind: DecoratedExprKind::StructInit(DecoratedStructInit {
                name: struct_name.to_string(),
                fields,
                span: traverse.span,
            }),
            metadata: SwcExprMetadata {
                swc_type: struct_name.to_string(),
                is_boxed: false,
                is_optional: false,
                type_kind: crate::type_system::SwcTypeKind::Unknown,
                span: Some(traverse.span),
            },
        };

        // Generate: target.visit_mut_with(&mut visitor)
        // Need to decorate the target expression first
        let mut decorator = crate::codegen::swc_decorator::SwcDecorator::new();
        let decorated_target = decorator.decorate_expr(&traverse.target);

        let visit_call = DecoratedExpr {
            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                callee: DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: Box::new(decorated_target),
                        property: "visit_mut_with".to_string(),
                        optional: false,
                        computed: false,
                        is_path: false,
                        field_metadata: crate::codegen::swc_metadata::SwcFieldMetadata {
                            swc_field_name: "visit_mut_with".to_string(),
                            accessor: crate::codegen::swc_metadata::FieldAccessor::Direct,
                            field_type: "fn(&mut Self)".to_string(),
                            source_field: Some("visit_mut_with".to_string()),
                            span: Some(traverse.span),
                            read_conversion: String::new(),
                        },
                    },
                    metadata: SwcExprMetadata {
                        swc_type: "fn(&mut Self)".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                        span: Some(traverse.span),
                    },
                },
                args: vec![
                    DecoratedExpr {
                        kind: DecoratedExprKind::Unary {
                            op: crate::parser::UnaryOp::RefMut,
                            operand: Box::new(struct_init),
                            unary_metadata: crate::codegen::swc_metadata::SwcUnaryMetadata {
                                override_op: None,
                                span: Some(traverse.span),
                            },
                        },
                        metadata: SwcExprMetadata {
                            swc_type: format!("&mut {}", struct_name),
                            is_boxed: false,
                            is_optional: false,
                            type_kind: crate::type_system::SwcTypeKind::Unknown,
                            span: Some(traverse.span),
                        },
                    }
                ],
                type_args: vec![],
                optional: false,
                is_macro: false,
                span: traverse.span,
            })),
            metadata: SwcExprMetadata {
                swc_type: "()".to_string(),
                is_boxed: false,
                is_optional: false,
                type_kind: crate::type_system::SwcTypeKind::Unknown,
                span: Some(traverse.span),
            },
        };

        DecoratedStmt::Expr(visit_call)
    }

    /// Transform method body to use self.var for captured variables
    fn transform_method_body_with_captures(
        &self,
        body: &Block,
        captures: &[crate::parser::Capture],
        state: &[crate::parser::LetStmt],
    ) -> DecoratedBlock {
        // Collect all captured variable names
        let mut captured_vars = HashSet::new();
        for capture in captures {
            captured_vars.insert(capture.name.clone());
        }
        for let_stmt in state {
            if let Pattern::Ident(name) = &let_stmt.pattern {
                captured_vars.insert(name.clone());
            }
        }

        // Create a transformer to prefix captured variables
        let mut transformer = CaptureTransformer {
            captured_vars,
        };

        // Transform each statement in the body
        let transformed_stmts = body.stmts.iter()
            .map(|stmt| transformer.transform_stmt(stmt))
            .collect();

        DecoratedBlock {
            stmts: transformed_stmts,
        }
    }

}

/// Helper to transform captured variable references to self.var
struct CaptureTransformer {
    captured_vars: HashSet<String>,
}

impl CaptureTransformer {
    /// Transform a statement, prefixing captured variables with self.
    fn transform_stmt(&mut self, stmt: &crate::parser::Stmt) -> DecoratedStmt {
        use crate::parser::Stmt;
        use crate::codegen::swc_decorator::SwcDecorator;

        // For now, use the decorator to convert the statement, then we'll transform it
        // This is a simplified approach - we need to decorate first
        let mut decorator = SwcDecorator::new();
        let decorated = decorator.decorate_stmt(stmt);

        self.transform_decorated_stmt(decorated)
    }

    fn transform_decorated_stmt(&mut self, stmt: DecoratedStmt) -> DecoratedStmt {
        match stmt {
            DecoratedStmt::Expr(expr) => {
                DecoratedStmt::Expr(self.transform_expr(expr))
            }
            DecoratedStmt::Let(let_stmt) => {
                // Transform the RHS to use self. for captured vars
                DecoratedStmt::Let(crate::codegen::decorated_ast::DecoratedLetStmt {
                    pattern: let_stmt.pattern,
                    init: self.transform_expr(let_stmt.init),
                    ty: let_stmt.ty,
                    mutable: let_stmt.mutable,
                })
            }
            DecoratedStmt::If(if_stmt) => {
                DecoratedStmt::If(DecoratedIfStmt {
                    condition: self.transform_expr(if_stmt.condition),
                    then_branch: self.transform_block(if_stmt.then_branch),
                    else_branch: if_stmt.else_branch.map(|b| self.transform_block(b)),
                    pattern: if_stmt.pattern,
                    if_let_metadata: if_stmt.if_let_metadata,
                })
            }
            other => other,
        }
    }

    fn transform_block(&mut self, block: DecoratedBlock) -> DecoratedBlock {
        DecoratedBlock {
            stmts: block.stmts.into_iter()
                .map(|stmt| self.transform_decorated_stmt(stmt))
                .collect(),
        }
    }

    fn transform_expr(&mut self, expr: DecoratedExpr) -> DecoratedExpr {
        match expr.kind {
            DecoratedExprKind::Ident { name, ident_metadata } => {
                // If this identifier is a captured variable, prefix with self.
                if self.captured_vars.contains(&name) {
                    let name_clone = name.clone();
                    DecoratedExpr {
                        kind: DecoratedExprKind::Member {
                            object: Box::new(DecoratedExpr {
                                kind: DecoratedExprKind::Ident {
                                    name: "self".to_string(),
                                    ident_metadata: crate::codegen::swc_metadata::SwcIdentifierMetadata {
                                        use_sym: false,
                                        deref_pattern: None,
                                        span: expr.metadata.span,
                                    },
                                },
                                metadata: SwcExprMetadata {
                                    swc_type: "Self".to_string(),
                                    is_boxed: false,
                                    is_optional: false,
                                    type_kind: crate::type_system::SwcTypeKind::Unknown,
                                    span: expr.metadata.span,
                                },
                            }),
                            property: name_clone.clone(),
                            optional: false,
                            computed: false,
                            is_path: false,
                            field_metadata: crate::codegen::swc_metadata::SwcFieldMetadata {
                                swc_field_name: name_clone.clone(),
                                accessor: crate::codegen::swc_metadata::FieldAccessor::Direct,
                                field_type: "i32".to_string(), // TODO: proper type
                                source_field: Some(name_clone),
                                span: expr.metadata.span,
                                read_conversion: String::new(),
                            },
                        },
                        metadata: expr.metadata,
                    }
                } else {
                    DecoratedExpr {
                        kind: DecoratedExprKind::Ident { name, ident_metadata },
                        metadata: expr.metadata,
                    }
                }
            }
            // Recursively transform other expression types
            DecoratedExprKind::Assign { left, right } => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Assign {
                        left: Box::new(self.transform_expr(*left)),
                        right: Box::new(self.transform_expr(*right)),
                    },
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Binary { left, op, right, binary_metadata } => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Binary {
                        left: Box::new(self.transform_expr(*left)),
                        op,
                        right: Box::new(self.transform_expr(*right)),
                        binary_metadata,
                    },
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Call(call) => {
                // TODO: Transform callee and args
                DecoratedExpr {
                    kind: DecoratedExprKind::Call(call),
                    metadata: expr.metadata,
                }
            }
            other => DecoratedExpr {
                kind: other,
                metadata: expr.metadata,
            },
        }
    }
}
