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
    DecoratedForStmt, DecoratedTraverseStmt, DecoratedTraverseKind,
    DecoratedInlineVisitor, DecoratedVisitorMethod,
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

    /// Current writer type name for qualifying function calls
    current_writer: Option<String>,

    /// Semantic type environment for looking up captured variable types
    type_env: crate::semantic::TypeEnv,

    /// Pending statements to be inserted before the next statement
    pending_stmts: Vec<DecoratedStmt>,
}

impl SwcHoister {
    pub fn new(type_env: crate::semantic::TypeEnv) -> Self {
        Self {
            visitor_counter: 0,
            hoisted_structs: Vec::new(),
            hoisted_impls: Vec::new(),
            current_writer: None,
            type_env,
            pending_stmts: Vec::new(),
        }
    }

    /// Convert TypeInfo to SWC type string
    fn typeinfo_to_swc_string(&self, type_info: &crate::semantic::TypeInfo) -> String {
        use crate::semantic::TypeInfo;
        match type_info {
            TypeInfo::Str => "String".to_string(),
            TypeInfo::I32 => "i32".to_string(),
            TypeInfo::U32 => "u32".to_string(),
            TypeInfo::F64 => "f64".to_string(),
            TypeInfo::Bool => "bool".to_string(),
            TypeInfo::Unit => "()".to_string(),
            TypeInfo::Null => "()".to_string(),
            TypeInfo::Ref { mutable, inner } => {
                // Recursively convert inner type
                // But don't add & here, that's handled by the caller
                self.typeinfo_to_swc_string(inner)
            }
            TypeInfo::Vec(inner) => {
                format!("Vec<{}>", self.typeinfo_to_swc_string(inner))
            }
            TypeInfo::Option(inner) => {
                format!("Option<{}>", self.typeinfo_to_swc_string(inner))
            }
            TypeInfo::AstNode(name) => {
                // Convert AST node names to SWC types
                get_node_mapping(name).map(|m| m.swc.to_string()).unwrap_or_else(|| name.clone())
            }
            TypeInfo::Struct { name, .. } => name.clone(),
            _ => "UserDefined".to_string(),
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
        // Set the current writer name for qualifying function calls in hoisted visitors
        self.current_writer = Some(writer.name.clone());

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

            // Insert any pending statements before this one
            if !self.pending_stmts.is_empty() {
                new_stmts.append(&mut self.pending_stmts);
            }

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
    fn hoist_traverse(&mut self, traverse: Box<DecoratedTraverseStmt>) -> DecoratedStmt {
        match &traverse.kind {
            DecoratedTraverseKind::Inline(inline) => {
                // Generate unique struct name
                let struct_name = format!("__InlineVisitor_{}", self.visitor_counter);
                self.visitor_counter += 1;

                // Determine if we need lifetime parameters
                let has_captures = !traverse.captures.is_empty();

                // Build the hoisted struct
                let mut struct_fields = Vec::new();

                // Add captured variables as fields
                for capture in &traverse.captures {
                    // Look up the type of the captured variable
                    let inner_type_name = if let Some(var_type) = self.type_env.lookup(&capture.name) {
                        // Convert TypeInfo to SWC type string
                        self.typeinfo_to_swc_string(var_type)
                    } else {
                        eprintln!("[HOISTER] Warning: Could not find type for captured variable '{}', using UserDefined", capture.name);
                        "UserDefined".to_string()
                    };

                    let field_type = if capture.mutable {
                        // &mut T
                        Type::Reference {
                            inner: Box::new(Type::Primitive(inner_type_name)),
                            mutable: true,
                        }
                    } else {
                        // &T
                        Type::Reference {
                            inner: Box::new(Type::Primitive(inner_type_name)),
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
                    lifetimes: if has_captures { vec!["'a".to_string()] } else { vec![] },
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
                    // Pass parameter type information for field mapping
                    let method_body = self.transform_method_body_with_captures(
                        &method.body,
                        &traverse.captures,
                        &inline.state,
                        &method.params,
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

                let struct_name_with_lifetime = if has_captures {
                    format!("{}<'a>", struct_name)
                } else {
                    struct_name.clone()
                };

                let impl_block = DecoratedImplBlock {
                    target: format!("VisitMut for {}", struct_name_with_lifetime),
                    lifetimes: if has_captures { vec!["'a".to_string()] } else { vec![] },
                    items: impl_methods,
                };

                self.hoisted_impls.push(impl_block);

                // Now generate the instantiation code at the traverse site
                self.generate_visitor_instantiation(&struct_name, &traverse, &inline.state)
            }

            DecoratedTraverseKind::Delegated(visitor_name) => {
                // For delegated visitors, just generate the instantiation + call
                // TODO: Generate proper delegation code
                DecoratedStmt::Expr(DecoratedExpr {
                    kind: DecoratedExprKind::Literal(crate::parser::Literal::String(
                        format!("/* TODO: Delegate to {} */", visitor_name)
                    )),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
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
        &mut self,
        struct_name: &str,
        traverse: &DecoratedTraverseStmt,
        state: &[crate::parser::LetStmt],
    ) -> DecoratedStmt {
        use crate::codegen::decorated_ast::{DecoratedCallExpr, DecoratedStructInit, DecoratedLetStmt, DecoratedPattern};
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
                            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
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
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
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
                            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
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
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
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
            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                swc_type: struct_name.to_string(),
                is_boxed: false,
                is_optional: false,
                type_kind: crate::type_system::SwcTypeKind::Unknown,
                span: Some(traverse.span),
            },
        };

        // Generate: target.visit_mut_with(&mut visitor)
        // target is already decorated
        let decorated_target = traverse.target.clone();

        // STEP 1: Check if target is an immutable reference
        let target_type = &traverse.target.metadata.swc_type;
        eprintln!("[HOISTER STEP 1] Target type: '{}', starts with &: {}", target_type, target_type.starts_with("&"));
        let needs_clone = target_type.starts_with("&") && !target_type.starts_with("&mut");
        eprintln!("[HOISTER STEP 1] needs_clone: {}", needs_clone);

        // STEP 2: Get target variable name for clone
        let target_var_name = if let DecoratedExprKind::Ident { name, .. } = &decorated_target.kind {
            name.clone()
        } else {
            // For complex expressions, we can't easily clone - just use the expression directly
            // This shouldn't happen in our case since we're matching on a simple binding
            eprintln!("[HOISTER WARNING] Target is not a simple identifier, can't generate clone");
            "".to_string()
        };

        // STEP 3: Determine which target to use in visit_mut_with call
        let (visit_target, clone_stmt) = if needs_clone && !target_var_name.is_empty() {
            // Generate a clone variable
            let clone_var_name = format!("{}_clone", target_var_name);
            eprintln!("[HOISTER STEP 2] Generating clone: let mut {} = {}.clone()", clone_var_name, target_var_name);

            // Create ident expr for the clone variable
            let clone_ident = DecoratedExpr {
                kind: DecoratedExprKind::Ident {
                    name: clone_var_name.clone(),
                    ident_metadata: crate::codegen::swc_metadata::SwcIdentifierMetadata {
                        use_sym: false,
                        deref_pattern: None,
                        span: traverse.target.metadata.span,
                    },
                },
                metadata: SwcExprMetadata {
                    needs_enum_unwrap: None,
                    swc_type: target_type.trim_start_matches("&").to_string(), // Remove & from type
                    is_boxed: false,
                    is_optional: false,
                    type_kind: crate::type_system::SwcTypeKind::Unknown,
                    span: traverse.target.metadata.span,
                },
            };

            // Create clone() call
            let clone_call = DecoratedExpr {
                kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                    callee: DecoratedExpr {
                        kind: DecoratedExprKind::Member {
                            object: Box::new(decorated_target.clone()),
                            property: "clone".to_string(),
                            optional: false,
                            computed: false,
                            is_path: false,
                            field_metadata: crate::codegen::swc_metadata::SwcFieldMetadata {
                                swc_field_name: "clone".to_string(),
                                accessor: crate::codegen::swc_metadata::FieldAccessor::Direct,
                                field_type: target_type.trim_start_matches("&").to_string(),
                                source_field: Some("clone".to_string()),
                                span: traverse.target.metadata.span,
                                read_conversion: String::new(),
                            },
                        },
                        metadata: SwcExprMetadata {
                            needs_enum_unwrap: None,
                            swc_type: target_type.trim_start_matches("&").to_string(),
                            is_boxed: false,
                            is_optional: false,
                            type_kind: crate::type_system::SwcTypeKind::Unknown,
                            span: traverse.target.metadata.span,
                        },
                    },
                    args: vec![],
                    type_args: vec![],
                    optional: false,
                    is_macro: false,
                    span: traverse.span,
                })),
                metadata: SwcExprMetadata {
                    needs_enum_unwrap: None,
                    swc_type: target_type.trim_start_matches("&").to_string(),
                    is_boxed: false,
                    is_optional: false,
                    type_kind: crate::type_system::SwcTypeKind::Unknown,
                    span: traverse.target.metadata.span,
                },
            };

            // Create let mut clone_var = target.clone()
            let let_stmt = DecoratedStmt::Let(DecoratedLetStmt {
                mutable: true,
                pattern: DecoratedPattern {
                    kind: crate::codegen::decorated_ast::DecoratedPatternKind::Ident(clone_var_name.clone()),
                    metadata: crate::codegen::swc_metadata::SwcPatternMetadata {
                        swc_pattern: "".to_string(),
                        unwrap_strategy: crate::codegen::swc_metadata::UnwrapStrategy::None,
                        inner: None,
                        span: traverse.target.metadata.span,
                        source_pattern: None,
                        desugar_strategy: None,
                    },
                },
                ty: None,
                init: clone_call,
            });

            (clone_ident, Some(let_stmt))
        } else {
            (decorated_target, None)
        };

        let visit_call = DecoratedExpr {
            kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                callee: DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: Box::new(visit_target),
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
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
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
                        metadata: SwcExprMetadata { needs_enum_unwrap: None, 
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
            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                swc_type: "()".to_string(),
                is_boxed: false,
                is_optional: false,
                type_kind: crate::type_system::SwcTypeKind::Unknown,
                span: Some(traverse.span),
            },
        };

        // STEP 4: Add clone statement to pending_stmts if needed
        if let Some(clone_stmt) = clone_stmt {
            eprintln!("[HOISTER STEP 3] Adding clone statement to pending_stmts");
            self.pending_stmts.push(clone_stmt);
        }

        DecoratedStmt::Expr(visit_call)
    }

    /// Transform method body to use self.var for captured variables
    fn transform_method_body_with_captures(
        &self,
        body: &DecoratedBlock,
        captures: &[crate::parser::Capture],
        state: &[crate::parser::LetStmt],
        params: &[crate::parser::Param],
    ) -> DecoratedBlock {
        // Collect all captured variable names and their mutability
        let mut captured_vars = HashSet::new();
        let mut mutable_captures = HashSet::new();
        for capture in captures {
            eprintln!("DEBUG: Registering captured variable: {}", capture.name);
            captured_vars.insert(capture.name.clone());
            if capture.mutable {
                mutable_captures.insert(capture.name.clone());
            }
        }
        for let_stmt in state {
            if let Pattern::Ident(name) = &let_stmt.pattern {
                captured_vars.insert(name.clone());
            }
        }

        // Create a transformer with parameter type information
        let mut transformer = CaptureTransformer {
            captured_vars,
            mutable_captures,
            params: params.to_vec(),
            writer_type: self.current_writer.clone(),
        };

        // Transform the block to replace captured variables with self.var
        transformer.transform_block(body.clone())
    }

}

/// Helper to transform captured variable references to self.var
struct CaptureTransformer {
    captured_vars: HashSet<String>,
    mutable_captures: HashSet<String>,
    params: Vec<crate::parser::Param>,
    writer_type: Option<String>,
}

impl CaptureTransformer {
    /// Transform a statement, prefixing captured variables with self.
    fn transform_stmt(&mut self, stmt: &crate::parser::Stmt) -> DecoratedStmt {
        use crate::parser::Stmt;
        use crate::codegen::swc_decorator::SwcDecorator;
        use crate::type_system::TypeContext;

        // Create decorator for traverse blocks
        let mut decorator = SwcDecorator::for_traverse();

        // Register parameter types in the decorator's type environment
        // This enables field mapping (e.g., ident.name → ident.sym)
        for param in &self.params {
            if let crate::parser::Type::Reference { inner, .. } = &param.ty {
                if let crate::parser::Type::Named(type_name) = inner.as_ref() {
                    // Look up SWC type from mapping
                    let swc_type = crate::mapping::get_node_mapping(type_name)
                        .map(|m| m.swc.to_string())
                        .unwrap_or_else(|| type_name.clone());

                    decorator.register_param_type(&param.name, &swc_type);
                }
            }
        }

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
                    eprintln!("DEBUG: Transforming captured variable '{}' to 'self.{}'", name, name);
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
                                metadata: SwcExprMetadata { needs_enum_unwrap: None, 
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
            DecoratedExprKind::CompoundAssign { left, op, right } => {
                let transformed_left = self.transform_expr(*left);
                // If left side is a mutable capture (self.x where x is &mut), wrap in deref
                let final_left = if let DecoratedExprKind::Member { ref property, .. } = transformed_left.kind {
                    if self.mutable_captures.contains(property) {
                        // Wrap in *(...)
                        DecoratedExpr {
                            kind: DecoratedExprKind::Unary {
                                op: crate::parser::UnaryOp::Deref,
                                operand: Box::new(transformed_left.clone()),
                                unary_metadata: crate::codegen::swc_metadata::SwcUnaryMetadata {
                                    override_op: None,
                                    span: transformed_left.metadata.span,
                                },
                            },
                            metadata: transformed_left.metadata.clone(),
                        }
                    } else {
                        transformed_left
                    }
                } else {
                    transformed_left
                };

                DecoratedExpr {
                    kind: DecoratedExprKind::CompoundAssign {
                        left: Box::new(final_left),
                        op,
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
            DecoratedExprKind::Member { object, property, optional, computed, is_path, field_metadata } => {
                // Transform the object recursively
                let transformed_object = Box::new(self.transform_expr(*object));

                // Get the object's type to do field mapping
                // Strip reference/mut markers to get base type
                let obj_type_raw = &transformed_object.metadata.swc_type;
                let obj_type = obj_type_raw
                    .trim_start_matches("&mut ")
                    .trim_start_matches("&");

                // Try to map Babel field name to SWC field name
                let swc_field = if let Some(mapping) = crate::codegen::type_context::get_typed_field_mapping(obj_type, &property) {
                    mapping.swc_field.to_string()
                } else {
                    property
                };

                DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: transformed_object,
                        property: swc_field,
                        optional,
                        computed,
                        is_path,
                        field_metadata,
                    },
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Call(call) => {
                // Transform callee and arguments to handle captured variables
                eprintln!("DEBUG: Transforming Call expression with {} args", call.args.len());
                let transformed_args: Vec<_> = call.args.into_iter()
                    .enumerate()
                    .map(|(i, arg)| {
                        eprintln!("DEBUG: Transforming arg {}", i);
                        self.transform_expr(arg)
                    })
                    .collect();

                // Check if callee is a bare identifier - if so, qualify it with writer type
                let transformed_callee = if let DecoratedExprKind::Ident { name, .. } = &call.callee.kind {
                    // Don't qualify built-in types/functions
                    let is_builtin = matches!(name.as_str(),
                        "Some" | "None" | "Ok" | "Err" | "vec" | "Vec" | "Box" |
                        "String" | "println" | "format" | "print" | "panic" | "assert" |
                        "HashMap" | "HashSet" | "Option" | "Result"
                    );

                    if let Some(writer_type) = &self.writer_type {
                        if !is_builtin {
                            // Transform bare function call to WriterType::function_name
                            eprintln!("DEBUG: Qualifying bare function call '{}' to '{}::{}'", name, writer_type, name);
                        DecoratedExpr {
                            kind: DecoratedExprKind::Member {
                                object: Box::new(DecoratedExpr {
                                    kind: DecoratedExprKind::Ident {
                                        name: writer_type.clone(),
                                        ident_metadata: crate::codegen::swc_metadata::SwcIdentifierMetadata {
                                            use_sym: false,
                                            deref_pattern: None,
                                            span: call.callee.metadata.span,
                                        },
                                    },
                                    metadata: crate::codegen::swc_metadata::SwcExprMetadata { needs_enum_unwrap: None, 
                                        swc_type: writer_type.clone(),
                                        is_boxed: false,
                                        is_optional: false,
                                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                                        span: call.callee.metadata.span,
                                    },
                                }),
                                property: name.clone(),
                                optional: false,
                                computed: false,
                                is_path: true,  // This is a path like WriterType::function
                                field_metadata: crate::codegen::swc_metadata::SwcFieldMetadata {
                                    swc_field_name: name.clone(),
                                    accessor: crate::codegen::swc_metadata::FieldAccessor::Direct,
                                    field_type: "Function".to_string(),
                                    source_field: Some(name.clone()),
                                    span: call.callee.metadata.span,
                                    read_conversion: String::new(),
                                },
                            },
                            metadata: call.callee.metadata.clone(),
                        }
                        } else {
                            self.transform_expr(call.callee)
                        }
                    } else {
                        self.transform_expr(call.callee)
                    }
                } else {
                    self.transform_expr(call.callee)
                };

                let transformed_call = Box::new(crate::codegen::decorated_ast::DecoratedCallExpr {
                    callee: transformed_callee,
                    args: transformed_args,
                    type_args: call.type_args,
                    optional: call.optional,
                    is_macro: call.is_macro,
                    span: call.span,
                });
                DecoratedExpr {
                    kind: DecoratedExprKind::Call(transformed_call),
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Unary { op, operand, unary_metadata } => {
                // Transform the operand (e.g., &mut component -> &mut self.component)
                eprintln!("DEBUG: Transforming Unary expression with op {:?}", op);
                DecoratedExpr {
                    kind: DecoratedExprKind::Unary {
                        op,
                        operand: Box::new(self.transform_expr(*operand)),
                        unary_metadata,
                    },
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Ref { expr: inner, mutable } => {
                // Transform &component or &mut component -> &self.component or &mut self.component
                eprintln!("DEBUG: Transforming Ref expression (mutable: {})", mutable);
                DecoratedExpr {
                    kind: DecoratedExprKind::Ref {
                        expr: Box::new(self.transform_expr(*inner)),
                        mutable,
                    },
                    metadata: expr.metadata,
                }
            }
            DecoratedExprKind::Deref(inner) => {
                // Transform *expr
                DecoratedExpr {
                    kind: DecoratedExprKind::Deref(Box::new(self.transform_expr(*inner))),
                    metadata: expr.metadata,
                }
            }
            ref other @ _ => {
                use crate::codegen::decorated_ast::DecoratedExprKind as Kind;
                let name = match other {
                    Kind::Literal(_) => "Literal",
                    Kind::Ident { .. } => "Ident",
                    Kind::Binary { .. } => "Binary",
                    Kind::Unary { .. } => "Unary",
                    Kind::Call(_) => "Call",
                    Kind::Member { .. } => "Member",
                    Kind::Index { .. } => "Index",
                    Kind::StructInit(_) => "StructInit",
                    Kind::Assign { .. } => "Assign",
                    Kind::RegexCall(_) => "RegexCall",
                    Kind::CustomPropAccess(_) => "CustomPropAccess",
                    _ => "Other",
                };
                eprintln!("DEBUG: Unhandled expression kind: {}", name);
                DecoratedExpr {
                    kind: other.clone(),
                    metadata: expr.metadata,
                }
            },
        }
    }
}
