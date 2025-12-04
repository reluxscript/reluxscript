//! SWC Decorator - Transforms parser AST into decorated AST with SWC semantics
//!
//! This is where the magic happens! The decorator:
//! 1. Walks the ReluxScript AST
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
use crate::lexer::Span;
use crate::type_system::{TypeContext, SwcTypeKind};
use crate::mapping::{get_node_mapping, get_field_mapping};
use super::type_context::{get_typed_field_mapping, map_reluxscript_to_swc, get_swc_variant_in_context};
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

    /// Semantic type environment from semantic analysis pass
    /// Contains all type information already computed
    semantic_type_env: Option<crate::semantic::TypeEnv>,

    /// Tracks variables that were narrowed from a parent enum to a specific variant
    /// Maps variable name -> (parent_enum, variant)
    /// E.g., "arg" -> ("Expr", "JSXElement") when arg was bound in match Expr::JSXElement(arg)
    narrowed_enum_vars: HashMap<String, (String, String)>,

    /// Whether we're currently in a writer context
    /// (affects field replacements like self.builder → self)
    is_writer: bool,

    /// Whether we're inside a traverse block
    /// (affects field mapping - use SWC field names directly instead of translating)
    in_traverse: bool,

    /// Custom property registry
    /// Maps (node_type, property_name) -> inferred_type
    custom_props: HashMap<(String, String), crate::parser::Type>,
}

impl SwcDecorator {
    pub fn new() -> Self {
        let mut type_env = HashMap::new();

        // Register built-in types
        // CodeBuilder is a built-in type - we generate a CodeBuilder struct
        type_env.insert("CodeBuilder".to_string(), TypeContext {
            reluxscript_type: "CodeBuilder".to_string(),
            swc_type: "CodeBuilder".to_string(),
            kind: SwcTypeKind::Struct,
            known_variant: None,
            needs_deref: false,
        });

        Self {
            type_env,
            current_params: HashMap::new(),
            semantic_type_env: None,
            narrowed_enum_vars: HashMap::new(),
            is_writer: false,
            in_traverse: false,
            custom_props: HashMap::new(),
        }
    }

    /// Create a new decorator with semantic type information
    pub fn with_semantic_types(semantic_type_env: crate::semantic::TypeEnv) -> Self {
        let mut type_env = HashMap::new();

        // Register built-in types
        // CodeBuilder is a built-in type - we generate a CodeBuilder struct
        type_env.insert("CodeBuilder".to_string(), TypeContext {
            reluxscript_type: "CodeBuilder".to_string(),
            swc_type: "CodeBuilder".to_string(),
            kind: SwcTypeKind::Struct,
            known_variant: None,
            needs_deref: false,
        });

        Self {
            type_env,
            current_params: HashMap::new(),
            semantic_type_env: Some(semantic_type_env),
            narrowed_enum_vars: HashMap::new(),
            is_writer: false,
            in_traverse: false,
            custom_props: HashMap::new(),
        }
    }

    /// Create a decorator for traverse block context
    /// In traverse blocks, we skip Babel→SWC field mapping since traverse blocks use SWC types directly
    pub fn for_traverse() -> Self {
        let mut decorator = Self::new();
        decorator.in_traverse = true;
        decorator
    }

    /// Register a parameter type in the type environment
    /// Used by traverse blocks to enable field mapping
    pub fn register_param_type(&mut self, param_name: &str, swc_type: &str) {
        self.type_env.insert(param_name.to_string(), TypeContext {
            reluxscript_type: swc_type.to_string(),
            swc_type: swc_type.to_string(),
            kind: SwcTypeKind::Struct, // AST nodes are structs
            known_variant: None,
            needs_deref: false,
        });
    }

    /// Look up variable type from semantic TypeEnv and convert to SWC type string
    fn lookup_semantic_type(&self, var_name: &str) -> Option<String> {
        if let Some(ref type_env) = self.semantic_type_env {
            if let Some(type_info) = type_env.lookup(var_name) {
                let swc_type = Self::type_info_to_swc_type(type_info);
                eprintln!("[DEBUG SEMANTIC LOOKUP] '{}' found in semantic TypeEnv: {:?} -> {}", var_name, type_info, swc_type);
                return Some(swc_type);
            } else {
                eprintln!("[DEBUG SEMANTIC LOOKUP] '{}' NOT found in semantic TypeEnv", var_name);
            }
        } else {
            eprintln!("[DEBUG SEMANTIC LOOKUP] No semantic TypeEnv available");
        }
        None
    }

    /// Look up type by XPath - uses path-based resolution for member expressions
    fn lookup_type_by_path(&self, xpath: &str) -> Option<String> {
        if let Some(ref type_env) = self.semantic_type_env {
            // Try resolve_member_path which handles field access chains
            if let Some(type_info) = type_env.resolve_member_path(xpath) {
                let swc_type = Self::type_info_to_swc_type(&type_info);
                eprintln!("[DEBUG PATH LOOKUP] '{}' resolved to: {:?} -> {}", xpath, type_info, swc_type);
                return Some(swc_type);
            } else {
                eprintln!("[DEBUG PATH LOOKUP] '{}' NOT found via resolve_member_path", xpath);
            }
        }
        None
    }

    /// Convert semantic TypeInfo to SWC type string
    fn type_info_to_swc_type(type_info: &crate::semantic::TypeInfo) -> String {
        use crate::semantic::TypeInfo;
        use super::type_context::map_reluxscript_to_swc;

        match type_info {
            TypeInfo::Str => "String".to_string(),
            TypeInfo::I32 => "i32".to_string(),
            TypeInfo::U32 => "u32".to_string(),
            TypeInfo::F64 => "f64".to_string(),
            TypeInfo::Bool => "bool".to_string(),
            TypeInfo::Unit => "()".to_string(),
            TypeInfo::Null => "Option".to_string(),
            TypeInfo::Ref { inner, .. } => Self::type_info_to_swc_type(inner),
            TypeInfo::Vec(inner) => format!("Vec<{}>", Self::type_info_to_swc_type(inner)),
            TypeInfo::Option(inner) => format!("Option<{}>", Self::type_info_to_swc_type(inner)),
            TypeInfo::Result(ok, err) => format!("Result<{}, {}>",
                Self::type_info_to_swc_type(ok),
                Self::type_info_to_swc_type(err)),
            TypeInfo::AstNode(name) => {
                // Convert ReluxScript AST node name to SWC type
                // e.g., "MemberExpression" -> "MemberExpr"
                map_reluxscript_to_swc(name).0
            }
            TypeInfo::Struct { name, .. } => name.clone(),
            TypeInfo::Enum { name, .. } => name.clone(),
            _ => "Unknown".to_string(),
        }
    }

    /// Decorate a full program (main entry point)
    pub fn decorate_program(&mut self, program: &Program) -> DecoratedProgram {
        DecoratedProgram {
            uses: program.uses.clone(),
            decl: self.decorate_top_level_decl(&program.decl),
            uses_custom_props: false, // Will be set by rewriter if CustomPropAccess is found
        }
    }

    fn decorate_top_level_decl(&mut self, decl: &TopLevelDecl) -> DecoratedTopLevelDecl {
        match decl {
            TopLevelDecl::Plugin(plugin) => {
                self.is_writer = false;
                DecoratedTopLevelDecl::Plugin(self.decorate_plugin_decl(plugin))
            }
            TopLevelDecl::Writer(writer) => {
                self.is_writer = true;
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
        // Separate structs from other items
        let mut hoisted_structs = Vec::new();
        let mut state_struct = None;
        let mut body_items = Vec::new();

        for item in &writer.body {
            match item {
                PluginItem::Struct(s) => {
                    if s.name == "State" {
                        // Filter out CodeBuilder fields from State
                        let mut filtered_state = s.clone();
                        filtered_state.fields.retain(|field| {
                            // Remove builder: CodeBuilder field
                            if let Type::Named(name) = &field.ty {
                                name != "CodeBuilder"
                            } else {
                                true
                            }
                        });
                        state_struct = Some(filtered_state);
                    } else {
                        hoisted_structs.push(s.clone());
                    }
                }
                PluginItem::Function(f) => {
                    // Skip init() function - it's replaced by the generated new()
                    if f.name != "init" && f.name != "finish" {
                        body_items.push(self.decorate_plugin_item(item));
                    }
                }
                _ => {
                    body_items.push(self.decorate_plugin_item(item));
                }
            }
        }

        DecoratedWriter {
            name: writer.name.clone(),
            body: body_items,
            hoisted_structs,
            state_struct,
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
            PluginItem::Static(static_decl) => {
                // Static declarations pass through with decorated initializer
                DecoratedPluginItem::Static(DecoratedStaticDecl {
                    name: static_decl.name.clone(),
                    ty: static_decl.ty.clone(),
                    init: self.decorate_expr(&static_decl.init),
                    is_mut: static_decl.is_mut,
                    span: static_decl.span,
                })
            }
            PluginItem::PubUse(use_stmt) => {
                // Re-exports pass through
                DecoratedPluginItem::PubUse(use_stmt.clone())
            }
        }
    }

    fn decorate_fn_decl(&mut self, func: &FnDecl) -> DecoratedFnDecl {
        // Clear and register parameter types
        self.current_params.clear();

        // For visitor methods, get the correct parameter type from the mapping
        let visitor_param_type = if func.name.starts_with("visit_") && !func.params.is_empty() {
            use crate::mapping::get_node_mapping_by_visitor;
            get_node_mapping_by_visitor(&func.name).map(|m| m.swc.to_string())
        } else {
            None
        };

        for (i, param) in func.params.iter().enumerate() {
            // For first parameter of visitor methods, use the mapped type
            let type_ctx = if i == 0 && visitor_param_type.is_some() {
                let swc_type = visitor_param_type.as_ref().unwrap().clone();
                TypeContext {
                    reluxscript_type: param.name.clone(),
                    swc_type,
                    kind: SwcTypeKind::Struct, // Visitor params are usually structs
                    known_variant: None,
                    needs_deref: false,
                }
            } else if let Some(swc_type) = self.lookup_semantic_type(&param.name) {
                // Try semantic type env
                TypeContext {
                    reluxscript_type: param.name.clone(),
                    swc_type,
                    kind: SwcTypeKind::Unknown, // Will be refined
                    known_variant: None,
                    needs_deref: false,
                }
            } else {
                // Fallback to annotation parsing
                self.type_annotation_to_context(&param.ty)
            };
            self.current_params.insert(param.name.clone(), type_ctx.clone());
            self.type_env.insert(param.name.clone(), type_ctx);
        }

        // Decorate the function body
        let decorated_body = self.decorate_block(&func.body);

        // Filter out the 'ctx' parameter - SWC doesn't have context
        let filtered_params: Vec<Param> = func.params.iter()
            .filter(|p| p.name != "ctx")
            .cloned()
            .collect();

        // Map visitor method names to SWC equivalents
        let swc_name = if func.name.starts_with("visit_") {
            self.map_visitor_method_name(&func.name)
        } else {
            func.name.clone()
        };

        // Map parameter types to SWC types
        let swc_params = filtered_params.into_iter().map(|mut param| {
            param.ty = self.map_type_to_swc(&param.ty);
            param
        }).collect();

        // Map where clause bounds to SWC types
        let swc_where_clause: Vec<WherePredicate> = func.where_clause.iter().map(|pred| {
            WherePredicate {
                target: pred.target.clone(),
                bound: self.map_type_to_swc(&pred.bound),
                span: pred.span,
            }
        }).collect();

        DecoratedFnDecl {
            name: swc_name,
            type_params: func.type_params.clone(),
            params: swc_params,
            return_type: func.return_type.as_ref().map(|ty| self.map_type_to_swc(ty)),
            where_clause: swc_where_clause,
            body: decorated_body,
        }
    }

    fn decorate_impl_block(&mut self, impl_block: &ImplBlock) -> DecoratedImplBlock {
        DecoratedImplBlock {
            target: impl_block.target.clone(),
            lifetimes: vec![],
            items: impl_block.items.iter().map(|m| self.decorate_fn_decl(m)).collect(),
        }
    }

    fn decorate_block(&mut self, block: &Block) -> DecoratedBlock {
        DecoratedBlock {
            stmts: block.stmts.iter().map(|s| self.decorate_stmt(s)).collect(),
        }
    }

    pub fn decorate_stmt(&mut self, stmt: &Stmt) -> DecoratedStmt {
        match stmt {
            Stmt::Let(let_stmt) => {
                // First decorate the initializer to get its type (if present)
                let (init, init_type) = if let Some(ref init_expr) = let_stmt.init {
                    let decorated = self.decorate_expr(init_expr);
                    let ty = decorated.metadata.swc_type.clone();
                    (Some(decorated), ty)
                } else {
                    (None, "Unknown".to_string())
                };

                // Use that type for pattern decoration
                let pattern = self.decorate_pattern_with_context(&let_stmt.pattern, &init_type);

                // Register the pattern bindings with the inferred type
                self.register_pattern_bindings(&let_stmt.pattern, &init_type);

                DecoratedStmt::Let(DecoratedLetStmt {
                    mutable: let_stmt.mutable,
                    pattern,
                    ty: let_stmt.ty.as_ref().map(|t| self.map_type_to_swc(t)),
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
                eprintln!("[DECORATOR MATCH] Decorating match statement with {} arms", match_stmt.arms.len());
                // Decorate scrutinee first to get its type
                let expr = self.decorate_expr(&match_stmt.scrutinee);
                let scrutinee_type = expr.metadata.swc_type.clone();

                // Use scrutinee type for all arm patterns
                let arms = match_stmt.arms.iter().map(|arm| {
                    // Decorate pattern first to get its metadata
                    let decorated_pattern = self.decorate_pattern_with_context(&arm.pattern, &scrutinee_type);

                    // Extract pattern bindings and add them to type_env before decorating body
                    // Pass scrutinee_type so Option<T> patterns can extract the inner T
                    self.add_pattern_bindings_to_env_with_type(&arm.pattern, &decorated_pattern, Some(scrutinee_type.clone()));

                    let decorated_body_expr = self.decorate_expr(&arm.body);

                    // Remove pattern bindings after decorating body
                    self.remove_pattern_bindings_from_env(&arm.pattern);

                    // Convert body expr to a block with single expression
                    let body_block = DecoratedBlock {
                        stmts: vec![DecoratedStmt::Expr(decorated_body_expr)],
                    };

                    DecoratedMatchArm {
                        pattern: decorated_pattern,
                        guard: None, // MatchArm doesn't have guard in this AST
                        body: body_block,
                    }
                }).collect();

                DecoratedStmt::Match(DecoratedMatchStmt { expr, arms })
            }

            Stmt::For(for_stmt) => {
                // Decorate iterator first
                eprintln!("[DEBUG FOR] Decorating for loop iterator: {:?}",
                    format!("{:?}", for_stmt.iter).chars().take(50).collect::<String>());
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

                // Register pattern bindings for the loop body
                self.register_pattern_bindings(&for_stmt.pattern, &element_type);

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

            Stmt::Traverse(traverse) => {
                let decorated_target = self.decorate_expr(&traverse.target);
                let decorated_kind = match &traverse.kind {
                    crate::parser::TraverseKind::Inline(inline_visitor) => {
                        let decorated_methods = inline_visitor.methods.iter().map(|method| {
                            // Register parameter types for field mapping
                            for param in &method.params {
                                if let crate::parser::Type::Reference { inner, .. } = &param.ty {
                                    if let crate::parser::Type::Named(type_name) = inner.as_ref() {
                                        let swc_type = crate::mapping::get_node_mapping(type_name)
                                            .map(|m| m.swc.to_string())
                                            .unwrap_or_else(|| type_name.clone());
                                        self.register_param_type(&param.name, &swc_type);
                                    }
                                }
                            }

                            let decorated_body = self.decorate_block(&method.body);

                            // Clean up param types after decorating this method
                            for param in &method.params {
                                self.type_env.remove(&param.name);
                            }

                            DecoratedVisitorMethod {
                                name: method.name.clone(),
                                params: method.params.clone(),
                                body: decorated_body,
                            }
                        }).collect();

                        DecoratedTraverseKind::Inline(DecoratedInlineVisitor {
                            state: inline_visitor.state.clone(),
                            methods: decorated_methods,
                        })
                    }
                    crate::parser::TraverseKind::Delegated(name) => {
                        DecoratedTraverseKind::Delegated(name.clone())
                    }
                };

                DecoratedStmt::Traverse(Box::new(DecoratedTraverseStmt {
                    target: decorated_target,
                    captures: traverse.captures.clone(),
                    kind: decorated_kind,
                    span: traverse.span,
                }))
            }

            Stmt::Function(func_decl) => {
                // Recursively decorate the function body
                let decorated_body = self.decorate_block(&func_decl.body);
                DecoratedStmt::Function(DecoratedNestedFnDecl {
                    is_pub: func_decl.is_pub,
                    name: func_decl.name.clone(),
                    type_params: func_decl.type_params.clone(),
                    params: func_decl.params.clone(),
                    return_type: func_decl.return_type.clone(),
                    where_clause: func_decl.where_clause.clone(),
                    body: decorated_body,
                    span: func_decl.span,
                })
            }

            Stmt::Verbatim(verbatim) => DecoratedStmt::Verbatim(verbatim.clone()),

            Stmt::CustomPropAssignment(assign) => {
                self.decorate_custom_prop_assignment(assign)
            }

            Stmt::Unsafe(unsafe_block) => {
                // Decorate statements inside the unsafe block
                let stmts = unsafe_block.body.stmts.iter()
                    .map(|s| self.decorate_stmt(s))
                    .collect();
                DecoratedStmt::Unsafe(DecoratedUnsafeBlock { stmts })
            }

        }
    }

    /// 🔥 THE CRITICAL FUNCTION: Decorate if-let statements with context-aware patterns
    fn decorate_if_stmt(&mut self, if_stmt: &IfStmt) -> DecoratedIfStmt {
        eprintln!("[DECORATOR IF-LET] if_stmt.condition = {:?}", if_stmt.condition);
        eprintln!("[DECORATOR IF-LET] if_stmt.pattern = {:?}", if_stmt.pattern);

        // First, decorate the condition expression to get its type
        let decorated_condition = self.decorate_expr(&if_stmt.condition);

        // Get the SWC type of the condition
        // Special case: if condition is matches!(expr, Pattern), extract the scrutinee type
        // Special case: if condition is &expr (Ref), add & prefix to the type
        let condition_type_base = if let DecoratedExprKind::Matches { ref expr, .. } = decorated_condition.kind {
            eprintln!("[DECORATOR IF-LET] Condition is Matches, using scrutinee type: '{}'", expr.metadata.swc_type);
            expr.metadata.swc_type.clone()
        } else {
            eprintln!("[DECORATOR IF-LET] Condition is not Matches, using condition type: '{}'", decorated_condition.metadata.swc_type);
            decorated_condition.metadata.swc_type.clone()
        };

        // If the condition is a Ref/RefMut expression, add the & prefix
        let condition_type_string = if let DecoratedExprKind::Unary { op, ref operand, .. } = &decorated_condition.kind {
            match op {
                crate::parser::UnaryOp::Ref => {
                    let with_ref = format!("&{}", condition_type_base);
                    eprintln!("[DECORATOR IF-LET] Condition is &expr, adjusting type: '{}' -> '{}'", condition_type_base, with_ref);
                    with_ref
                }
                crate::parser::UnaryOp::RefMut => {
                    format!("&mut {}", condition_type_base)
                }
                _ => condition_type_base
            }
        } else {
            condition_type_base
        };
        let condition_type = &condition_type_string;

        // If this is an if-let, decorate the pattern with CONTEXT
        let decorated_pattern = if let Some(ref pattern) = if_stmt.pattern {
            // THIS IS THE KEY: We know what type is being matched!
            eprintln!("[DECORATOR IF-LET] Decorating pattern {:?} with condition_type: '{}'", pattern, condition_type);
            let decorated_pat = self.decorate_pattern_with_context(pattern, condition_type);

            // Register pattern bindings in type environment for the then branch
            // Use the decorated pattern's metadata to get the correct narrowed types
            // Pass the condition type so Option<T> patterns can extract the T
            self.add_pattern_bindings_to_env_with_type(pattern, &decorated_pat, Some(condition_type.clone()));

            // CRITICAL: Also narrow the scrutinee variable in decorator's type_env
            // This mirrors what semantic type checker does, but decorator needs its own
            // narrowing because semantic's is scoped and gets popped
            if let Pattern::Variant { name, .. } = pattern {
                let narrowed_type = if name.contains("::") {
                    let parts: Vec<&str> = name.split("::").collect();
                    if parts.len() == 2 {
                        let (swc_type, _) = map_reluxscript_to_swc(parts[1]);
                        swc_type
                    } else {
                        condition_type.to_string()
                    }
                } else {
                    let (swc_type, _) = map_reluxscript_to_swc(name);
                    swc_type
                };

                let scrutinee_key = match &if_stmt.condition {
                    Expr::Ident(ident) => Some(ident.name.clone()),
                    Expr::Member(mem) => {
                        if let Expr::Ident(ident) = mem.object.as_ref() {
                            Some(format!("{}.{}", ident.name, mem.property))
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(key) = scrutinee_key.clone() {
                    // Check if field is boxed from original mapping
                    let is_boxed = if let Expr::Member(mem) = &if_stmt.condition {
                        if let Expr::Ident(ident) = mem.object.as_ref() {
                            // Try local type_env first (returns TypeContext)
                            let obj_type_str = if let Some(obj_ctx) = self.type_env.get(&ident.name) {
                                Some(obj_ctx.swc_type.clone())
                            } else if let Some(semantic_type) = self.semantic_type_env.as_ref()
                                .and_then(|env| env.lookup(&ident.name)) {
                                // semantic_type_env returns TypeInfo, extract the type name
                                Some(semantic_type.display_name())
                            } else {
                                None
                            };

                            if let Some(obj_type) = obj_type_str {
                                get_typed_field_mapping(&obj_type, &mem.property)
                                    .map(|m| m.needs_deref)
                                    .unwrap_or(false)
                            } else {
                                false
                            }
                        } else {
                            false
                        }
                    } else {
                        false
                    };

                    eprintln!("[DECORATOR NARROWING] Shadowing '{}' to {} (is_boxed: {}) in then-branch",
                        key, narrowed_type, is_boxed);

                    // Save old value to restore later
                    let old_value = self.type_env.get(&key).cloned();

                    self.type_env.insert(key.clone(), TypeContext {
                        reluxscript_type: narrowed_type.clone(),
                        swc_type: narrowed_type.clone(),
                        kind: SwcTypeKind::Struct,
                        known_variant: None,
                        needs_deref: is_boxed,
                    });

                    // Decorate then branch with narrowing active
                    let then_branch = self.decorate_block(&if_stmt.then_branch);

                    // Restore old value (pop scope)
                    if let Some(old) = old_value {
                        self.type_env.insert(key, old);
                    } else {
                        self.type_env.remove(&key);
                    }

                    let else_branch = if_stmt.else_branch.as_ref().map(|b| self.decorate_block(b));

                    return DecoratedIfStmt {
                        condition: decorated_condition,
                        pattern: Some(decorated_pat),
                        then_branch,
                        else_branch,
                        if_let_metadata: None,
                    };
                }
            }

            Some(decorated_pat)
        } else {
            None
        };

        // Decorate branches (no narrowing case)
        let then_branch = self.decorate_block(&if_stmt.then_branch);
        let else_branch = if_stmt.else_branch.as_ref().map(|b| self.decorate_block(b));

        // TODO: Restore the original type after the if-block (for proper scoping)
        // For now, the narrowed type persists which is acceptable

        DecoratedIfStmt {
            condition: decorated_condition,
            pattern: decorated_pattern,
            then_branch,
            else_branch,
            if_let_metadata: None, // TODO: Add if-let metadata
        }
    }

    /// Register variables bound by a pattern into the type environment
    fn register_pattern_bindings(&mut self, pattern: &Pattern, bound_type: &str) {
        match pattern {
            Pattern::Ident(name) => {
                // Simple identifier binding
                // Don't overwrite existing bindings with less specific types
                if let Some(existing) = self.type_env.get(name) {
                    // Only overwrite if the new type is more specific
                    let is_new_less_specific = bound_type == "UserDefined" || bound_type == "Unknown";
                    let is_existing_specific = existing.swc_type != "UserDefined" && existing.swc_type != "Unknown";

                    if is_existing_specific && is_new_less_specific {
                        eprintln!("[DEBUG] Skipping pattern binding: {} (already bound to {})",  name, existing.swc_type);
                        return;
                    }
                }
                // Prefer semantic type over decorator type for more accuracy
                let actual_type = if let Some(semantic_type) = self.lookup_semantic_type(name) {
                    eprintln!("[DEBUG] Using semantic type for '{}': {} (instead of {})", name, semantic_type, bound_type);
                    semantic_type
                } else {
                    bound_type.to_string()
                };
                eprintln!("[DEBUG] Registering pattern binding: {} -> {}", name, actual_type);
                self.type_env.insert(name.clone(), TypeContext {
                    reluxscript_type: actual_type.clone(),
                    swc_type: actual_type,
                    kind: SwcTypeKind::Unknown,
                    known_variant: None,
                    needs_deref: false,
                });
            }
            Pattern::Variant { name, inner } => {
                // For variant patterns, the inner binding gets the variant's type
                if let Some(inner_pattern) = inner {
                    // Extract the variant type name
                    // e.g., "Callee::MemberExpression" → inner type is "MemberExpression"
                    let inner_type = if name.contains("::") {
                        let parts: Vec<&str> = name.split("::").collect();
                        if parts.len() == 2 {
                            // Convert "MemberExpression" to "MemberExpr" using mapping
                            let (_swc_type, _kind) = map_reluxscript_to_swc(parts[1]);
                            eprintln!("[DEBUG] Variant {} -> inner type: {}", name, _swc_type);
                            _swc_type
                        } else {
                            eprintln!("[DEBUG] Variant {} (parts != 2) -> {}", name, bound_type);
                            bound_type.to_string()
                        }
                    } else if name == "Some" {
                        // Special case: Some(x) unwraps Option<T> → T
                        // Extract T from Option<T>
                        if bound_type.starts_with("Option<") && bound_type.ends_with('>') {
                            let inner = &bound_type[7..bound_type.len()-1];
                            eprintln!("[DEBUG] Variant Some unwraps {} -> {}", bound_type, inner);
                            inner.to_string()
                        } else {
                            eprintln!("[DEBUG] Variant Some but bound_type not Option: {}", bound_type);
                            bound_type.to_string()
                        }
                    } else {
                        eprintln!("[DEBUG] Variant {} (no ::) -> {}", name, bound_type);
                        bound_type.to_string()
                    };
                    self.register_pattern_bindings(inner_pattern, &inner_type);
                }
            }
            Pattern::Ref { pattern: inner, .. } => {
                // Ref pattern: register the inner pattern with the same type
                self.register_pattern_bindings(inner, bound_type);
            }
            _ => {
                // Other patterns don't bind simple names we can track
            }
        }
    }

    /// 🎯 CONTEXT-AWARE PATTERN DECORATION
    /// This is where we solve the MemberProp vs Expr problem!
    fn decorate_pattern_with_context(&mut self, pattern: &Pattern, expected_type: &str) -> DecoratedPattern {
        // Strip Box<T> wrapper from expected type if present
        // Box<Expr> → Expr, Box<Pat> → Pat
        let expected_type = if expected_type.starts_with("Box<") && expected_type.ends_with(">") {
            let stripped = expected_type.trim_start_matches("Box<").trim_end_matches(">");
            eprintln!("[DEBUG PATTERN] Stripped Box: {} -> {}", expected_type, stripped);
            stripped
        } else {
            expected_type
        };

        match pattern {
            Pattern::Variant { name, inner } => {
                // Parse the variant name: "Expression::Identifier" or "Callee::MemberExpression"
                let swc_pattern = if name.contains("::") {
                    let parts: Vec<&str> = name.split("::").collect();
                    if parts.len() == 2 {
                        let relux_enum = parts[0];  // "Expression"
                        let relux_variant = parts[1]; // "Identifier"

                        // 🔥 CONTEXT-AWARE MAPPING
                        // If we're matching against MemberProp, Pat, or Callee, translate differently!
                        if expected_type == "MemberProp" {
                            // Expression::Identifier on MemberProp → MemberProp::Ident
                            if relux_enum == "Expression" && relux_variant == "Identifier" {
                                "MemberProp::Ident".to_string()
                            } else {
                                // Fallback to standard mapping
                                self.map_pattern_to_swc(name)
                            }
                        } else if expected_type == "Callee" && relux_enum == "Expression" {
                            // Expression::* on Callee → Callee::Expr
                            // The nested pattern desugaring is handled by desugar_strategy below
                            "Callee::Expr".to_string()
                        } else if expected_type == "Pat" || relux_enum == "Pattern" {
                            // Pattern::Identifier → Pat::Ident
                            // Pattern::Array → Pat::Array (already handled by map_pattern_to_swc)
                            if relux_variant == "Identifier" {
                                "Pat::Ident".to_string()
                            } else {
                                // Use standard mapping which handles Pattern::ArrayPattern etc
                                self.map_pattern_to_swc(name)
                            }
                        } else {
                            // Standard mapping for Expr, Stmt, etc.
                            let full_pattern = self.map_pattern_to_swc(name);
                            // Strip the binding name if present, keep only the enum variant
                            // "Expr::Array(array_lit)" → "Expr::Array"
                            self.strip_pattern_binding(&full_pattern)
                        }
                    } else {
                        name.clone()
                    }
                } else {
                    // Simple variants like Some, None, Ok, Err
                    // Also handles struct-like patterns like CallExpression(_)
                    // Also handles literals like StringLiteral, NumericLiteral

                    // Strip reference prefix from expected_type to get base type
                    let base_expected_type = if expected_type.starts_with("&mut ") {
                        &expected_type[5..]
                    } else if expected_type.starts_with("&") {
                        &expected_type[1..]
                    } else {
                        expected_type
                    };
                    eprintln!("[DEBUG PATTERN] Stripped ref prefix: '{}' -> '{}'", expected_type, base_expected_type);

                    // First try context-aware mapping for literals
                    // If expected_type is a struct (like ObjectPat), find its parent enum first
                    let context = if let Some((parent_enum, _)) = crate::mapping::get_parent_enum_for_swc_type(base_expected_type) {
                        eprintln!("[DEBUG PATTERN] expected_type '{}' belongs to enum '{}'", base_expected_type, parent_enum);
                        parent_enum
                    } else {
                        base_expected_type.to_string()
                    };

                    let (enum_name, variant, _struct_name) = get_swc_variant_in_context(name, &context);
                    eprintln!("[DEBUG PATTERN] get_swc_variant_in_context('{}', '{}') = ('{}', '{}', ...)", name, context, enum_name, variant);

                    // Check if this is a nested pattern (variant contains '(' indicating nested enum)
                    // Example: variant = "Lit(Lit::Str" means we need Expr::Lit(Lit::Str(binding))
                    if variant.contains('(') {
                        // Nested pattern detected - build complete pattern with inner binding
                        let inner_binding = if let Some(Pattern::Ident(ref binding_name)) = inner.as_ref().map(|p| &**p) {
                            binding_name.clone()
                        } else {
                            "_".to_string()  // Default to wildcard if no binding
                        };
                        eprintln!("[DEBUG PATTERN] Using nested pattern: {}::{}({})))", enum_name, variant, inner_binding);
                        // Complete pattern: "Expr::Lit(Lit::Str(arg))"
                        // Need to close both the inner Lit::Str() and outer Expr::Lit()
                        format!("{}::{}({}))", enum_name, variant, inner_binding)
                    } else if enum_name != expected_type {
                        // Context-aware mapping found a different enum (e.g., Lit vs Expr)

                        // Special case: Option<ExprOrSpread> with Expr variant
                        // Don't use box pattern syntax (experimental), use guard pattern instead
                        if expected_type.contains("ExprOrSpread") && enum_name == "Expr" {
                            eprintln!("[DEBUG PATTERN] Option<ExprOrSpread> + Expr variant -> using guard pattern");
                            // Generate: Some(ref s) if s.spread.is_none()
                            // The inner Expr match will be handled by narrowing/rewriter
                            format!("Some(ref s) if s.spread.is_none()")
                        } else {
                            eprintln!("[DEBUG PATTERN] Using simple pattern: {}::{}", enum_name, variant);
                            format!("{}::{}", enum_name, variant)
                        }
                    } else if let Some(mapping) = get_node_mapping(name) {
                        // Use node mapping for struct types like CallExpression
                        let full_pattern = mapping.swc_pattern.to_string();
                        // Strip the binding name if present, keep only the enum variant
                        self.strip_pattern_binding(&full_pattern)
                    } else {
                        // Fallback to reluxscript_to_swc_type for built-in types
                        self.reluxscript_to_swc_type(name)
                    }
                };

                // Determine unwrap strategy based on expected_type
                let unwrap_strategy = self.determine_unwrap_strategy(expected_type);

                // 🔥 DETECT PATTERNS THAT NEED DESUGARING
                // Callee::MemberExpression doesn't exist in SWC - needs desugaring!
                let desugar_strategy = if name.contains("::") {
                    let parts: Vec<&str> = name.split("::").collect();
                    if parts.len() == 2 {
                        let relux_enum = parts[0];
                        let relux_variant = parts[1];

                        // Detect Callee::MemberExpression (legacy support)
                        if relux_enum == "Callee" && relux_variant == "MemberExpression" {
                            // Get the inner binding name from the pattern
                            let inner_binding = if let Some(Pattern::Ident(inner_name)) = inner.as_ref().map(|p| &**p) {
                                inner_name.clone()
                            } else {
                                "member".to_string()  // Default binding name
                            };

                            Some(DesugarStrategy::NestedIfLet {
                                outer_pattern: "Callee::Expr".to_string(),
                                outer_binding: "__callee_expr".to_string(),
                                inner_pattern: "Expr::Member".to_string(),
                                inner_binding,
                                unwrap_expr: ".as_ref()".to_string(),
                            })
                        // Detect Expression::* in Callee context (e.g., Expression::MemberExpression on CallExpression.callee)
                        } else if expected_type == "Callee" && relux_enum == "Expression" {
                            // Get the inner binding name from the pattern
                            let inner_binding = if let Some(Pattern::Ident(inner_name)) = inner.as_ref().map(|p| &**p) {
                                inner_name.clone()
                            } else {
                                "__expr_inner".to_string()  // Default binding name
                            };

                            // Map the expression variant to SWC pattern
                            let inner_swc_pattern = self.map_pattern_to_swc(&format!("Expression::{}", relux_variant));
                            let inner_pattern = self.strip_pattern_binding(&inner_swc_pattern);

                            Some(DesugarStrategy::NestedIfLet {
                                outer_pattern: "Callee::Expr".to_string(),
                                outer_binding: "__callee_expr".to_string(),
                                inner_pattern,
                                inner_binding,
                                unwrap_expr: ".as_ref()".to_string(),
                            })
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // 🔥 SPECIAL CASE: If this is a struct type with wildcard inner, generate struct pattern
                // Example: CallExpression(_) → CallExpr { .. } (not CallExpr(_))
                if !name.contains("::") && inner.as_ref().map_or(false, |p| matches!(**p, Pattern::Wildcard)) {
                    // This is a plain type name (no ::) with wildcard inner → it's a struct!
                    // Generate struct pattern with wildcard fields
                    return DecoratedPattern {
                        kind: DecoratedPatternKind::Struct {
                            name: swc_pattern.clone(),
                            fields: vec![],  // Empty fields means match any
                        },
                        metadata: SwcPatternMetadata::direct(format!("{} {{ .. }}", swc_pattern)),
                    };
                }

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
                        desugar_strategy,
                    },
                }
            }

            Pattern::Ident(name) => {
                // Check if this is a variant name (Some, None, Ok, Err) rather than a binding
                let is_variant = matches!(name.as_str(), "Some" | "None" | "Ok" | "Err");

                if is_variant {
                    // Treat as a variant pattern, not a binding
                    DecoratedPattern {
                        kind: DecoratedPatternKind::Variant {
                            name: name.clone(),
                            inner: None,
                        },
                        metadata: SwcPatternMetadata::direct(name.clone()),
                    }
                } else {
                    // Register this binding in type environment
                    // Type is the expected_type we're matching against
                    let type_ctx = TypeContext::from_reluxscript(expected_type);
                    self.type_env.insert(name.clone(), type_ctx);

                    DecoratedPattern {
                        kind: DecoratedPatternKind::Ident(name.clone()),
                        metadata: SwcPatternMetadata::direct(name.clone()),
                    }
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
                // Map the struct name to SWC type (e.g., CallExpression → CallExpr)
                let swc_name = self.reluxscript_to_swc_type(name);

                // Decorate fields with proper type information from field mappings
                let decorated_fields = fields.iter()
                    .map(|(fname, fpat)| {
                        // Look up field mapping to get the SWC type
                        let field_type = if let Some(mapping) = get_field_mapping(name, fname) {
                            mapping.swc_type.to_string()
                        } else {
                            "Unknown".to_string()
                        };

                        (fname.clone(), self.decorate_pattern_with_context(fpat, &field_type))
                    })
                    .collect();

                DecoratedPattern {
                    kind: DecoratedPatternKind::Struct {
                        name: swc_name.clone(),
                        fields: decorated_fields,
                    },
                    metadata: SwcPatternMetadata::direct(swc_name),
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

            Pattern::Ref { is_mut: _, pattern: inner } => {
                // In SWC patterns, we don't use 'ref' - just unwrap to the inner pattern
                // Example: `ref obj` in ReluxScript becomes just `obj` in SWC
                self.decorate_pattern_with_context(inner, expected_type)
            }
        }
    }

    /// Add pattern bindings to type environment based on decorated pattern metadata
    fn add_pattern_bindings_to_env(&mut self, pattern: &Pattern, decorated: &DecoratedPattern) {
        self.add_pattern_bindings_to_env_with_type(pattern, decorated, None);
    }

    /// Internal helper that tracks the expected type from parent patterns
    fn add_pattern_bindings_to_env_with_type(&mut self, pattern: &Pattern, decorated: &DecoratedPattern, expected_type: Option<String>) {
        eprintln!("[ADD PATTERN BINDING] pattern={:?}, decorated.kind={:?}, swc_pattern={}", pattern, decorated.kind, decorated.metadata.swc_pattern);
        match (pattern, &decorated.kind) {
            (Pattern::Ident(name), _) => {
                // Simple identifier - use the expected type from parent, or fall back to swc_pattern
                let swc_type = expected_type.unwrap_or_else(|| decorated.metadata.swc_pattern.clone());
                // Determine needs_deref: true if type is &Box<...> (reference to Box)
                // This occurs when binding from Option<&Box<T>>.as_ref() which gives &Box<T>
                let needs_deref = swc_type.starts_with("&Box<") || swc_type.starts_with("& Box<");
                eprintln!("[MATCH BINDING] Adding {} -> {} (needs_deref: {})", name, swc_type, needs_deref);
                self.type_env.insert(name.clone(), TypeContext {
                    reluxscript_type: swc_type.clone(),
                    swc_type: swc_type.clone(),
                    kind: SwcTypeKind::Struct,
                    known_variant: None,
                    needs_deref,
                });
            }
            (Pattern::Variant { inner: Some(inner_pat), ..  }, DecoratedPatternKind::Variant { inner: Some(inner_dec), .. }) => {
                // Variant with inner binding - extract the inner type from the variant
                // E.g., for Pat::Object(param), the inner type should be ObjectPat
                // Use the swc_pattern from metadata, not the name from kind
                let swc_pattern = &decorated.metadata.swc_pattern;
                let (inner_type, parent_enum_info) = if swc_pattern.contains("::") {
                    // Extract parts: "Pat::Object" -> enum="Pat", variant="Object"
                    let parts: Vec<&str> = swc_pattern.split("::").collect();
                    if parts.len() == 2 {
                        let enum_name = parts[0];  // "Pat"
                        let variant_name = parts[1]; // "Object"
                        // For SWC patterns, the inner type is the struct wrapped by the enum
                        // Pat::Object wraps ObjectPat, Pat::Array wraps ArrayPat, etc.
                        let inner_struct = if enum_name == "Pat" {
                            // Pat::Object -> ObjectPat, Pat::Array -> ArrayPat
                            format!("{}Pat", variant_name)
                        } else if enum_name == "Expr" {
                            // Expr::Call -> CallExpr, Expr::Member -> MemberExpr
                            format!("{}Expr", variant_name)
                        } else if enum_name == "ObjectPatProp" {
                            // ObjectPatProp::KeyValue -> KeyValuePatProp
                            format!("{}PatProp", variant_name)
                        } else if enum_name == "Option" {
                            // Option::Some wraps the inner type from Option<T> or &Option<T>
                            // Need to extract T from the expected_type and preserve & prefix
                            //
                            // IMPORTANT: When the inner type is Box<T>, the rewriter will add .as_ref()
                            // to the scrutinee, which changes Option<Box<T>> to Option<&Box<T>>.
                            // So the bound variable will be &Box<T>, not Box<T>.
                            if let Some(ref outer_type) = expected_type {
                                eprintln!("[STEP 3] Option::Some - outer_type: '{}'", outer_type);
                                // Check if outer_type is a reference
                                let (ref_prefix, base_type) = if outer_type.starts_with("&mut ") {
                                    ("&mut ", &outer_type[5..])
                                } else if outer_type.starts_with("&") {
                                    ("&", &outer_type[1..])
                                } else {
                                    ("", outer_type.as_str())
                                };
                                let extracted = Self::extract_generic_param(base_type).unwrap_or_else(|| variant_name.to_string());

                                // If the extracted type is Box<T>, the rewriter will add .as_ref()
                                // so the actual bound type will be &Box<T>
                                let full_type = if extracted.starts_with("Box<") && ref_prefix.is_empty() {
                                    let with_ref = format!("&{}", extracted);
                                    eprintln!("[STEP 3] Inner is Box<T>, anticipating .as_ref(): {} -> {}", extracted, with_ref);
                                    with_ref
                                } else {
                                    format!("{}{}", ref_prefix, extracted)
                                };
                                eprintln!("[STEP 3] Extracted: '{}' (with ref_prefix: '{}')", full_type, ref_prefix);
                                full_type
                            } else {
                                eprintln!("[STEP 3] Option::Some - NO expected_type");
                                variant_name.to_string()
                            }
                        } else {
                            variant_name.to_string()
                        };
                        eprintln!("[MATCH BINDING] Extracted inner type from {}: {}", swc_pattern, inner_struct);
                        // Return both the inner type and the parent enum info
                        (Some(inner_struct), Some((enum_name.to_string(), variant_name.to_string())))
                    } else {
                        (None, None)
                    }
                } else {
                    (None, None)
                };

                // Recursively add inner bindings with the extracted type
                self.add_pattern_bindings_to_env_with_type(inner_pat, inner_dec, inner_type);

                // If the inner pattern is an identifier, track that it was narrowed from the parent enum
                if let (Pattern::Ident(binding_name), Some((parent_enum, variant))) = (inner_pat.as_ref(), parent_enum_info) {
                    eprintln!("[MATCH BINDING] Tracking '{}' as narrowed from {}::{}", binding_name, parent_enum, variant);
                    self.narrowed_enum_vars.insert(binding_name.clone(), (parent_enum, variant));
                }
            }
            (Pattern::Ref { pattern: inner_pat, .. }, dec) => {
                // Ref patterns: unwrap and pass the type through
                // The decorated pattern may also be wrapped, but we want the actual inner
                let inner_dec = match dec {
                    DecoratedPatternKind::Ref { pattern: inner, .. } => inner.as_ref(),
                    // If the decorated isn't Ref, still unwrap the source pattern
                    _ => decorated,
                };
                eprintln!("[MATCH BINDING] Ref pattern - passing through type {:?} to inner", expected_type);
                self.add_pattern_bindings_to_env_with_type(inner_pat, inner_dec, expected_type);
            }
            _ => {
                // Other patterns don't create bindings we need to track
                eprintln!("[MATCH BINDING] Unhandled pattern case: {:?}", pattern);
            }
        }
    }

    /// Extract the type parameter from a generic type like "Option<Box<Expr>>" -> "Box<Expr>"
    fn extract_generic_param(type_str: &str) -> Option<String> {
        if let Some(start) = type_str.find('<') {
            if let Some(end) = type_str.rfind('>') {
                if end > start {
                    return Some(type_str[start + 1..end].to_string());
                }
            }
        }
        None
    }

    /// Remove pattern bindings from type environment
    fn remove_pattern_bindings_from_env(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Ident(name) => {
                eprintln!("[MATCH BINDING] Removing {}", name);
                self.type_env.remove(name);
            }
            Pattern::Variant { inner: Some(inner_pat), .. } => {
                // Recursively remove inner bindings
                self.remove_pattern_bindings_from_env(inner_pat);
            }
            _ => {}
        }
    }

    /// Decorate expression with .as_ref() suppressed for Option<Box<T>> fields
    /// Used when decorating operands of & reference operator
    fn decorate_expr_suppress_asref(&mut self, expr: &Expr) -> DecoratedExpr {
        match expr {
            Expr::Member(mem) => {
                // Decorate member expression but suppress .as_ref()
                eprintln!("[DEBUG SUPPRESS MEMBER] Decorating {}.{}",
                    format!("{:?}", mem.object).chars().take(30).collect::<String>(),
                    mem.property);
                let object = Box::new(self.decorate_expr(&mem.object));
                let object_type = &object.metadata.swc_type;
                eprintln!("[DEBUG SUPPRESS MEMBER] Object type: {}", object_type);

                let field_metadata = if mem.property == "builder" && self.is_writer {
                    SwcFieldMetadata {
                        swc_field_name: mem.property.clone(),
                        accessor: FieldAccessor::Replace {
                            with: "self".to_string(),
                        },
                        field_type: "Self".to_string(),
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                        read_conversion: String::new(),
                    }
                } else if let Some(mapping) = get_typed_field_mapping(object_type, &mem.property) {
                    // We have precise mapping - use BoxedRefDeref for Box fields
                    eprintln!("[DEBUG SUPPRESS] Field mapping found: {}.{} → {}.{}", object_type, mem.property, object_type, mapping.swc_field);
                    let is_boxed = mapping.result_type_swc.starts_with("Box<");
                    SwcFieldMetadata {
                        swc_field_name: mapping.swc_field.to_string(),
                        accessor: if mapping.read_conversion == ".as_bytes()" {
                            eprintln!("[DEBUG SUPPRESS ATOM] Using Utf8Lossy accessor for Atom field");
                            FieldAccessor::Utf8Lossy
                        } else if is_boxed {
                            FieldAccessor::BoxedRefDeref  // For Box<T> fields like member.obj
                        } else {
                            FieldAccessor::Direct
                        },
                        field_type: mapping.result_type_swc.to_string(),
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                        read_conversion: mapping.read_conversion.to_string(),
                    }
                } else {
                    // Fallback - apply common AST field name mappings
                    eprintln!("[DEBUG SUPPRESS] NO field mapping for: {}.{} - using fallback", object_type, mem.property);

                    // Check if this is likely a known SWC AST type
                    // Don't apply to "Unknown" - if we don't know the type, don't map fields
                    let is_likely_ast_type = object_type.ends_with("Expr")
                        || object_type.ends_with("Stmt")
                        || object_type.ends_with("Decl")
                        || object_type.ends_with("Pat")
                        || object_type.ends_with("Lit")
                        || object_type == "Ident"
                        || object_type == "Callee"
                        || object_type == "Param";

                    // Apply common SWC field name mappings only for AST types
                    let swc_field = if !is_likely_ast_type {
                        &mem.property
                    } else {
                        match mem.property.as_str() {
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
                            _ => &mem.property,
                        }
                    };

                    // Add .to_string() conversion for sym field (Atom -> String)
                    let read_conversion = if swc_field == "sym" && is_likely_ast_type {
                        ".to_string()"
                    } else {
                        ""
                    };

                    SwcFieldMetadata {
                        swc_field_name: swc_field.to_string(),
                        accessor: FieldAccessor::Direct,
                        field_type: "Unknown".to_string(),
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                        read_conversion: read_conversion.to_string(),
                    }
                };

                let member_type = field_metadata.field_type.clone();

                DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object,
                        property: mem.property.clone(),
                        optional: mem.optional,
                        computed: mem.computed,
                        is_path: mem.is_path,
                        field_metadata,
                    },
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: member_type,
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(mem.span),
                    
                    needs_to_string: false,},
                }
            }
            // For all other expression types, just use normal decoration
            _ => self.decorate_expr(expr),
        }
    }

    /// Decorate an expression and infer its SWC type
    pub fn decorate_expr(&mut self, expr: &Expr) -> DecoratedExpr {
        match expr {
            Expr::Ident(ident_expr) => {
                let name = &ident_expr.name;

                // Look up type in environment (local FIRST for pattern-narrowed types, then semantic)
                let (type_ctx, needs_enum_unwrap) = if let Some(ctx) = self.type_env.get(name).cloned() {
                    if name == "init" {
                        eprintln!("[DECORATOR IDENT 'init'] found in type_env: {} (needs_deref={})", ctx.swc_type, ctx.needs_deref);
                    }
                    eprintln!("[DECORATOR IDENT] '{}' found in type_env: {} (needs_deref={})", name, ctx.swc_type, ctx.needs_deref);
                    // Local type_env has pattern-narrowed types from match/if-let
                    eprintln!("[DEBUG] Ident '{}' found in local type_env: {} (needs_deref: {})", name, ctx.swc_type, ctx.needs_deref);

                    // Check if this was narrowed from Option<T> to T
                    let needs_option_unwrap = if let Some(semantic_type) = self.semantic_type_env.as_ref().and_then(|env| env.lookup(name)) {
                        eprintln!("[DEBUG NARROWING CHECK] '{}': semantic_type = {:?}, local_type = {}", name, semantic_type, ctx.swc_type);
                        // Check if semantic type is Option but local type is not
                        let is_option_semantic = matches!(semantic_type, crate::semantic::TypeInfo::Ref { inner, .. } if matches!(**inner, crate::semantic::TypeInfo::Option(_)));
                        let is_not_option_local = !ctx.swc_type.starts_with("Option<");
                        eprintln!("[DEBUG NARROWING CHECK] is_option_semantic={}, is_not_option_local={}", is_option_semantic, is_not_option_local);
                        is_option_semantic && is_not_option_local
                    } else {
                        eprintln!("[DEBUG NARROWING CHECK] '{}': no semantic type found", name);
                        false
                    };

                    if needs_option_unwrap {
                        eprintln!("[OPTION NARROWING] '{}' narrowed from Option to {}, needs unwrap", name, ctx.swc_type);
                    }

                    // Check if this was narrowed from a parent enum (e.g., Expr -> JSXElement)
                    // First check the local narrowed_enum_vars map (populated from match patterns)
                    let needs_enum_wrap = if let Some((parent_enum, variant)) = self.narrowed_enum_vars.get(name) {
                        eprintln!("[ENUM NARROWING LOCAL] '{}' tracked as narrowed from {}::{}", name, parent_enum, variant);
                        Some((parent_enum.clone(), variant.clone()))
                    } else if let Some(semantic_type) = self.semantic_type_env.as_ref().and_then(|env| env.lookup(name)) {
                        eprintln!("[DECORATOR CHECK NARROWING] '{}' semantic_type = {:?}", name, semantic_type);
                        // Check if semantic type is NarrowedAstNode
                        if let crate::semantic::TypeInfo::NarrowedAstNode { current_type, parent_enum, variant } = semantic_type {
                            eprintln!("[ENUM NARROWING] '{}' is NarrowedAstNode {{ current: {}, parent: {}, variant: {} }}",
                                      name, current_type, parent_enum, variant);
                            Some((parent_enum.clone(), variant.clone()))
                        } else if let crate::semantic::TypeInfo::Ref { inner, .. } = semantic_type {
                            // Handle &NarrowedAstNode
                            if let crate::semantic::TypeInfo::NarrowedAstNode { current_type, parent_enum, variant } = inner.as_ref() {
                                eprintln!("[ENUM NARROWING] '{}' is &NarrowedAstNode {{ current: {}, parent: {}, variant: {} }}",
                                          name, current_type, parent_enum, variant);
                                Some((parent_enum.clone(), variant.clone()))
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    } else {
                        None
                    };

                    // If local has Unknown/UserDefined, check semantic TypeEnv for better type
                    if (ctx.swc_type == "Unknown" || ctx.swc_type == "UserDefined") {
                        if let Some(swc_type_str) = self.lookup_semantic_type(name) {
                            eprintln!("[DEBUG] Local type is generic, using semantic TypeEnv: {}", swc_type_str);
                            (TypeContext {
                                reluxscript_type: name.clone(),
                                swc_type: swc_type_str.clone(),
                                kind: SwcTypeKind::Unknown,
                                known_variant: None,
                                needs_deref: false,
                            }, None)
                        } else {
                            (ctx, None)
                        }
                    } else if needs_option_unwrap {
                        // Return special unwrap marker
                        (ctx, Some(("Option".to_string(), "unwrap".to_string())))
                    } else if needs_enum_wrap.is_some() {
                        // Return enum wrap marker
                        (ctx, needs_enum_wrap)
                    } else {
                        (ctx, None)
                    }
                } else if let Some(swc_type_str) = self.lookup_semantic_type(name) {
                    // Semantic TypeEnv has original types from semantic analysis
                    eprintln!("[DEBUG] Ident '{}' found in semantic TypeEnv: {}", name, swc_type_str);

                    // Check if this is a narrowed type by looking up parent enum in mappings
                    let unwrap_info = if let Some((parent_enum, variant_name)) = crate::mapping::get_parent_enum_for_swc_type(&swc_type_str) {
                        eprintln!("[NARROWING DATA-DRIVEN] '{}' has type {} → parent enum {}::{}",
                            name, swc_type_str, parent_enum, variant_name);
                        Some((parent_enum, variant_name))
                    } else {
                        None
                    };

                    (TypeContext {
                        reluxscript_type: name.clone(),
                        swc_type: swc_type_str.clone(),
                        kind: SwcTypeKind::Unknown, // Will be refined later
                        known_variant: None,
                        needs_deref: false,
                    }, unwrap_info)
                } else {
                    eprintln!("[DEBUG] Ident '{}' not found, using UserDefined", name);
                    (TypeContext::unknown(), None)
                };

                eprintln!("[DECORATOR IDENT FINAL] '{}' → swc_type: '{}'", name, type_ctx.swc_type);

                DecoratedExpr {
                    kind: DecoratedExprKind::Ident {
                        name: name.clone(),
                        ident_metadata: SwcIdentifierMetadata::name(),
                    },
                    metadata: SwcExprMetadata {
                        needs_enum_unwrap,
                        swc_type: type_ctx.swc_type.clone(),
                        is_boxed: type_ctx.is_boxed(),
                        is_optional: false,
                        type_kind: type_ctx.kind.clone(),
                        span: Some(ident_expr.span),
                    
                    needs_to_string: false,},
                }
            }

            Expr::Member(mem) => {
                // Check if this member path is narrowed (e.g., node.expr narrowed to CallExpr)
                if let Expr::Ident(ident) = mem.object.as_ref() {
                    let member_path = format!("{}.{}", ident.name, mem.property);

                    // Check local type_env first, then semantic type_env
                    let narrowed_type = self.type_env.get(&member_path).map(|ctx| {
                        eprintln!("[DECORATOR MEMBER LOOKUP] Found '{}' in local type_env: {}", member_path, ctx.swc_type);
                        ctx.swc_type.clone()
                    })
                        .or_else(|| {
                            let result = self.semantic_type_env.as_ref()
                                .and_then(|env| env.lookup(&member_path))
                                .map(|t| {
                                    let name = t.display_name();
                                    eprintln!("[DECORATOR MEMBER LOOKUP] Found '{}' in semantic type_env: {}", member_path, name);
                                    // Extract the inner type from AstNode("CallExpr") -> "CallExpr"
                                    if name.starts_with("AstNode(\"") && name.ends_with("\")") {
                                        name[9..name.len()-2].to_string()
                                    } else {
                                        name
                                    }
                                });
                            if result.is_none() {
                                eprintln!("[DECORATOR MEMBER LOOKUP] '{}' NOT found in either type_env", member_path);
                            }
                            result
                        });

                    if let Some(_swc_type) = narrowed_type {
                        eprintln!("[DECORATOR MEMBER] Narrowed type for '{}': {} - will apply narrowing after field access", member_path, _swc_type);
                        // Don't return early! Let the normal member access logic handle it.
                        // The narrowing will be applied when this member is accessed.
                        // Fall through to normal member processing...
                    }
                }

                // First, decorate the object to get its type
                let decorated_object = Box::new(self.decorate_expr(&mem.object));
                let object_type = &decorated_object.metadata.swc_type;
                eprintln!("[DEBUG MEMBER] {}.{} (object type: {})",
                    format!("{:?}", mem.object).chars().take(30).collect::<String>(),
                    mem.property, object_type);

                // Debug: check if object is in type_env
                if let Expr::Ident(ident) = mem.object.as_ref() {
                    if let Some(ctx) = self.type_env.get(&ident.name) {
                        eprintln!("[DEBUG MEMBER] '{}' in type_env: {}", ident.name, ctx.swc_type);
                    } else {
                        eprintln!("[DEBUG MEMBER] '{}' NOT in type_env", ident.name);
                    }
                }

                // Check for writer-specific field replacements (self.builder → self)
                // This handles both direct access: self.builder, self.state
                // And method calls on them: self.builder.append() where object is self.builder
                let is_self_builder = self.is_writer &&
                    matches!(&decorated_object.kind, DecoratedExprKind::Ident { name, .. } if name == "self") &&
                    (mem.property == "builder" || mem.property == "state");

                // Check for self.builder.X pattern (methods on builder need replacement)
                let is_method_on_builder = self.is_writer &&
                    matches!(&decorated_object.kind,
                        DecoratedExprKind::Member { object, property, field_metadata, .. }
                        if matches!(&object.kind, DecoratedExprKind::Ident { name, .. } if name == "self")
                           && property == "builder"  // Only builder, NOT state
                           && matches!(&field_metadata.accessor, FieldAccessor::Replace { .. })
                    );

                // Look up the field in SWC schema
                eprintln!("[DEBUG MEMBER LOOKUP] is_self_builder={}, is_method_on_builder={}", is_self_builder, is_method_on_builder);
                let field_metadata = if is_self_builder || is_method_on_builder {
                    // In writers, self.builder and self.state should be replaced with just "self"
                    SwcFieldMetadata {
                        swc_field_name: mem.property.clone(),
                        accessor: FieldAccessor::Replace {
                            with: "self".to_string(),
                        },
                        field_type: "WriterContext".to_string(),
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                        read_conversion: String::new(),
                    }
                } else {
                    // Strip reference and Box<> wrappers for field mapping lookup
                    // e.g., "&Box<TsTypeAnn>" -> "TsTypeAnn", "Box<Expr>" -> "Expr"
                    let stripped_ref = object_type.strip_prefix("&mut ").or_else(|| object_type.strip_prefix("&")).unwrap_or(&object_type);
                    let base_object_type = if stripped_ref.starts_with("Box<") && stripped_ref.ends_with(">") {
                        &stripped_ref[4..stripped_ref.len()-1]
                    } else {
                        stripped_ref
                    };

                    if let Some(mapping) = get_typed_field_mapping(base_object_type, &mem.property) {
                    // We have precise mapping!
                    eprintln!("[DEBUG] Field mapping found: {}.{} → {}.{} (needs_deref: {})",
                        base_object_type, mem.property, base_object_type, mapping.swc_field, mapping.needs_deref);
                    SwcFieldMetadata {
                        swc_field_name: mapping.swc_field.to_string(),
                        accessor: {
                            eprintln!("[DEBUG ACCESSOR] read_conversion='{}', checking for .as_bytes()", mapping.read_conversion);
                            if mapping.read_conversion == ".as_bytes()" {
                                // Atom/JsWord needs String::from_utf8_lossy wrapping
                                eprintln!("[DEBUG ATOM] Using Utf8Lossy accessor for Atom field");
                                FieldAccessor::Utf8Lossy
                            } else if mapping.needs_deref {
                                FieldAccessor::BoxedAsRef
                            } else {
                                FieldAccessor::Direct
                            }
                        },
                        field_type: mapping.result_type_swc.to_string(),
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                        read_conversion: mapping.read_conversion.to_string(),
                    }
                } else {
                    eprintln!("[DEBUG] NO field mapping for: {}.{} - using fallback",
                        object_type, mem.property);

                    // Check if this is likely a known SWC AST type
                    // SWC types typically end with specific suffixes or are common AST node names
                    // Don't apply to "Unknown" - if we don't know the type, don't map fields
                    // Special case: Expr/Stmt/Pat are top-level enums, but accessing .name assumes Ident variant
                    let is_likely_ast_type = object_type.ends_with("Expr")
                        || object_type.ends_with("Stmt")
                        || object_type.ends_with("Decl")
                        || object_type.ends_with("Pat")
                        || object_type.ends_with("Lit")
                        || object_type == "Ident"
                        || object_type == "Callee"
                        || object_type == "PropName"  // Wrapper enum
                        || object_type == "MemberProp"  // Wrapper enum
                        || object_type == "Param"
                        || object_type == "Expr"  // Top-level enum, but .name assumes Ident
                        || object_type == "Pat";   // Top-level enum, but .name assumes Ident

                    // Fallback field mapping - only apply for SWC types, not user-defined types
                    let swc_field = if !is_likely_ast_type {
                        // Don't apply field mappings to user-defined structs
                        &mem.property
                    } else {
                        // Apply common SWC field name mappings
                        match mem.property.as_str() {
                            // Identifier.name -> Ident.sym
                            "name" => "sym",
                            // MemberExpression.object -> MemberExpr.obj
                            "object" => "obj",
                            // MemberExpression.property -> MemberExpr.prop
                            "property" => "prop",
                            // CallExpression.callee -> CallExpr.callee (no change)
                            "callee" => "callee",
                            // CallExpression.arguments -> CallExpr.args
                            "arguments" => "args",
                            // ArrayPattern.elements / ArrayExpression.elements -> elems
                            "elements" => "elems",
                            _ => &mem.property,
                        }
                    };

                    // Add .to_string() conversion for sym field (Atom -> String)
                    let read_conversion = if swc_field == "sym" && is_likely_ast_type {
                        ".to_string()"
                    } else {
                        ""
                    };

                    // Try to look up field type using XPath first (most accurate)
                    // Then fall back to struct field lookup
                    let field_type = if !is_likely_ast_type {
                        // First try XPath-based lookup using the member expression's path
                        self.lookup_type_by_path(&mem.path)
                            .or_else(|| {
                                // Fall back to struct field lookup
                                self.semantic_type_env.as_ref()
                                    .and_then(|env| env.get_struct_fields(object_type))
                                    .and_then(|fields| fields.get(&mem.property))
                                    .map(|type_info| {
                                        let type_name = type_info.display_name();
                                        eprintln!("[DEBUG] Found struct field type: {}.{} = {}",
                                            object_type, mem.property, type_name);
                                        type_name
                                    })
                            })
                            .unwrap_or_else(|| "UserDefined".to_string())
                    } else {
                        "UserDefined".to_string()
                    };

                    SwcFieldMetadata {
                        swc_field_name: swc_field.to_string(),
                        accessor: FieldAccessor::Direct,
                        field_type,
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                        read_conversion: read_conversion.to_string(),
                    }
                    }  // Close inner if-else (field mapping found vs fallback)
                };  // Close outer else (is_self_builder check)

                // Infer the type of this member expression
                // If there's a read_conversion that changes the type, use the converted type
                let base_member_type = if field_metadata.read_conversion == ".as_expr().unwrap()" {
                    // Callee -> &Expr after unwrapping
                    "Expr".to_string()
                } else if field_metadata.read_conversion == ".as_callee().unwrap()" {
                    "Callee".to_string()
                } else if field_metadata.read_conversion == ".as_prop().unwrap()" {
                    "MemberProp".to_string()
                } else {
                    field_metadata.field_type.clone()
                };

                // Check if this member expression has a narrowed type in the local type_env
                // e.g., after `if matches!(node.expr, CallExpression)`, `node.expr` is narrowed to `CallExpr`
                let member_type = if let Expr::Ident(ident) = mem.object.as_ref() {
                    let member_path = format!("{}.{}", ident.name, mem.property);
                    if let Some(ctx) = self.type_env.get(&member_path) {
                        eprintln!("[DECORATOR MEMBER] Member '{}' has narrowed type in local env: {}", member_path, ctx.swc_type);
                        ctx.swc_type.clone()
                    } else {
                        base_member_type
                    }
                } else {
                    base_member_type
                };

                DecoratedExpr {
                    kind: DecoratedExprKind::Member {
                        object: decorated_object,
                        property: mem.property.clone(),
                        optional: mem.optional,
                        computed: mem.computed,
                        is_path: mem.is_path,
                        field_metadata: field_metadata.clone(),
                    },
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,
                        swc_type: member_type,
                        is_boxed: matches!(field_metadata.accessor, FieldAccessor::BoxedAsRef | FieldAccessor::BoxedRefDeref),
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown, // TODO: Infer properly
                        span: Some(mem.span),
                    
                    needs_to_string: false,},
                }
            }

            Expr::Unary(unary) => {
                // For Ref/RefMut, suppress .as_ref() on Option<Box<T>> fields
                // because &node.field doesn't need .as_ref()
                let decorated_operand = if matches!(unary.op, UnaryOp::Ref | UnaryOp::RefMut) {
                    Box::new(self.decorate_expr_suppress_asref(&unary.operand))
                } else {
                    Box::new(self.decorate_expr(&unary.operand))
                };

                // Infer result type based on operation
                let result_type = match unary.op {
                    UnaryOp::Deref => {
                        // *expr unwraps Box<T> -> T
                        let operand_type = &decorated_operand.metadata.swc_type;
                        if operand_type.starts_with("Box<") && operand_type.ends_with(">") {
                            // Extract T from Box<T>
                            operand_type[4..operand_type.len()-1].to_string()
                        } else {
                            // Dereference of non-Box, type stays the same
                            operand_type.clone()
                        }
                    }
                    _ => {
                        // For other unary ops, type stays the same
                        decorated_operand.metadata.swc_type.clone()
                    }
                };

                // For reference operator (&), preserve is_boxed from operand
                // so that &bin.left where bin.left is Box<Expr> becomes &*bin.left
                let is_boxed = if matches!(unary.op, UnaryOp::Ref | UnaryOp::RefMut) {
                    decorated_operand.metadata.is_boxed
                } else {
                    false
                };

                DecoratedExpr {
                    kind: DecoratedExprKind::Unary {
                        op: unary.op,
                        operand: decorated_operand,
                        unary_metadata: SwcUnaryMetadata {
                            override_op: None,
                            span: Some(unary.span),
                        },
                    },
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,
                        swc_type: result_type,
                        is_boxed,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(unary.span),

                    needs_to_string: false,},
                }
            }

            Expr::Literal(lit) => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Literal(lit.clone()),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "Literal".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Primitive,
                        span: None,
                    
                    needs_to_string: false,},
                }
            }

            Expr::Binary(bin) => {
                let left = Box::new(self.decorate_expr(&bin.left));
                let right = Box::new(self.decorate_expr(&bin.right));

                // Check if we need to deref for string comparisons
                // e.g., obj.sym (JsWord) compared to "string" needs &*obj.sym
                let left_needs_deref = self.needs_sym_deref(&left, &right, bin.op);
                let right_needs_deref = self.needs_sym_deref(&right, &left, bin.op);

                // For null-coalescing (??), check if right side is also an Option
                // This determines whether to use .or() or .unwrap_or()
                let right_is_option = if matches!(bin.op, BinaryOp::NullCoalesce) {
                    right.metadata.is_optional ||
                    right.metadata.swc_type.starts_with("Option<") ||
                    // Check for another ?? on the right (chained)
                    matches!(&right.kind, DecoratedExprKind::Binary { op: BinaryOp::NullCoalesce, .. })
                } else {
                    false
                };

                // Determine result type for null-coalescing
                let (result_type, result_is_optional) = if matches!(bin.op, BinaryOp::NullCoalesce) {
                    // If right is also an Option, result is Option<T>
                    // Otherwise, result is the unwrapped type T
                    if right_is_option {
                        (left.metadata.swc_type.clone(), true)
                    } else {
                        // Extract inner type from Option<T>
                        let inner_type = if left.metadata.swc_type.starts_with("Option<") {
                            left.metadata.swc_type
                                .strip_prefix("Option<")
                                .and_then(|s| s.strip_suffix(">"))
                                .unwrap_or(&left.metadata.swc_type)
                                .to_string()
                        } else {
                            right.metadata.swc_type.clone()
                        };
                        (inner_type, false)
                    }
                } else {
                    ("bool".to_string(), false)
                };

                DecoratedExpr {
                    kind: DecoratedExprKind::Binary {
                        left,
                        op: bin.op,
                        right,
                        binary_metadata: SwcBinaryMetadata {
                            left_needs_deref,
                            right_needs_deref,
                            right_is_option,
                            span: Some(bin.span),
                        },
                    },
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,
                        swc_type: result_type,
                        is_boxed: false,
                        is_optional: result_is_optional,
                        type_kind: SwcTypeKind::Primitive,
                        span: Some(bin.span),

                    needs_to_string: false,},
                }
            }

            Expr::Call(call) => {
                let callee = self.decorate_expr(&call.callee);

                // Infer return type for known methods
                let return_type = match &callee.kind {
                    DecoratedExprKind::Member { object, property, .. } => {
                        // Check for .clone() -> returns the same type as the object
                        if property == "clone" {
                            let obj_type = &object.metadata.swc_type;
                            eprintln!("[CLONE INFERENCE] {}.clone() -> {}", obj_type, obj_type);
                            Some(obj_type.clone())
                        }
                        // Check for CodeBuilder::new() -> returns String type
                        else if property == "new" {
                            if let DecoratedExprKind::Ident { name, .. } = &object.kind {
                                if name == "CodeBuilder" {
                                    Some("String".to_string())
                                } else {
                                    None
                                }
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                // Check if this is a Result/Option constructor (Err, Ok, Some, None)
                // If so, string literal arguments need to be converted to String
                let callee_name = if let Expr::Ident(ref ident) = *call.callee {
                    Some(ident.name.as_str())
                } else {
                    None
                };

                let needs_string_conversion = matches!(callee_name, Some("Err") | Some("Ok") | Some("Some"));

                // Decorate arguments, potentially marking string literals as String type
                let args = call.args.iter().map(|a| {
                    let mut decorated = self.decorate_expr(a);

                    // If this is a string literal argument to Err/Ok/Some, mark it as String
                    if needs_string_conversion {
                        if let DecoratedExprKind::Literal(Literal::String(_)) = decorated.kind {
                            decorated.metadata.swc_type = "String".to_string();
                        }
                    }

                    decorated
                }).collect();

                DecoratedExpr {
                    kind: DecoratedExprKind::Call(Box::new(DecoratedCallExpr {
                        callee,
                        args,
                        type_args: call.type_args.clone(),
                        optional: call.optional,
                        is_macro: call.is_macro,
                        span: call.span,
                    })),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,
                        swc_type: return_type.unwrap_or_else(|| "UserDefined".to_string()),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(call.span),
                    
                    needs_to_string: false,},
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
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "UserDefined".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    
                    needs_to_string: false,},
                }
            }

            Expr::Index(index) => {
                let object = Box::new(self.decorate_expr(&index.object));
                let index_expr = Box::new(self.decorate_expr(&index.index));

                // Infer element type from array/vector type
                // For Vec<T>, element type is T
                let object_type = &object.metadata.swc_type;
                let element_type = if object_type.starts_with("Vec<") && object_type.ends_with('>') {
                    // Extract T from Vec<T> - need to handle nested generics like Vec<Option<Pat>>
                    // Remove "Vec<" from start and the matching ">" from end
                    let inner = &object_type[4..object_type.len()-1];
                    inner.to_string()
                } else {
                    // Unknown array type, use Unknown
                    "UserDefined".to_string()
                };

                // Check if element_type is a wrapper type that needs automatic unwrapping
                // Wrapper types: ExprOrSpread -> expr, Param -> pat, etc.
                let (needs_unwrap, unwrap_field, unwrapped_type) = match element_type.as_str() {
                    "ExprOrSpread" => (true, "expr", "Expr"),
                    "Param" => (true, "pat", "Pat"),
                    "Option<ExprOrSpread>" => (false, "", ""), // Handle separately if needed
                    _ => (false, "", ""),
                };

                if needs_unwrap {
                    // Transform arr[i] into arr[i].field automatically
                    // Create the index expression first
                    let index_decorated = DecoratedExpr {
                        kind: DecoratedExprKind::Index {
                            object: object.clone(),
                            index: index_expr.clone(),
                        },
                        metadata: SwcExprMetadata {
                            needs_enum_unwrap: None,
                            swc_type: element_type.clone(),
                            is_boxed: false,
                            is_optional: false,
                            type_kind: SwcTypeKind::Unknown,
                            span: Some(index.span),
                        
                        needs_to_string: false,},
                    };

                    // Look up the field mapping to get proper metadata
                    let field_metadata = if let Some(mapping) = get_typed_field_mapping(&element_type, unwrap_field) {
                        SwcFieldMetadata {
                            swc_field_name: mapping.swc_field.to_string(),
                            accessor: if mapping.needs_deref {
                                FieldAccessor::BoxedAsRef
                            } else {
                                FieldAccessor::Direct
                            },
                            field_type: mapping.result_type_swc.to_string(),
                            source_field: Some(unwrap_field.to_string()),
                            span: Some(index.span),
                            read_conversion: mapping.read_conversion.to_string(),
                        }
                    } else {
                        // Fallback if no mapping found
                        SwcFieldMetadata {
                            swc_field_name: unwrap_field.to_string(),
                            accessor: FieldAccessor::Direct,
                            field_type: unwrapped_type.to_string(),
                            source_field: Some(unwrap_field.to_string()),
                            span: Some(index.span),
                            read_conversion: String::new(),
                        }
                    };

                    // Wrap in a member access: (arr[i]).field
                    DecoratedExpr {
                        kind: DecoratedExprKind::Member {
                            object: Box::new(index_decorated),
                            property: unwrap_field.to_string(),
                            optional: false,
                            computed: false,
                            is_path: false,
                            field_metadata,
                        },
                        metadata: SwcExprMetadata {
                            needs_enum_unwrap: None,
                            swc_type: unwrapped_type.to_string(),
                            is_boxed: false, // Will be set by field_metadata.accessor
                            is_optional: false,
                            type_kind: SwcTypeKind::Unknown,
                            span: Some(index.span),
                        
                        needs_to_string: false,},
                    }
                } else {
                    // Normal index access, no unwrapping needed
                    DecoratedExpr {
                        kind: DecoratedExprKind::Index {
                            object,
                            index: index_expr,
                        },
                        metadata: SwcExprMetadata { needs_enum_unwrap: None,
                            swc_type: element_type,
                            is_boxed: false,
                            is_optional: false,
                            type_kind: SwcTypeKind::Unknown,
                            span: Some(index.span),
                        
                        needs_to_string: false,},
                    }
                }
            }

            Expr::StructInit(struct_init) => {
                // Map ReluxScript type to SWC type
                let swc_type = self.reluxscript_to_swc_type(&struct_init.name);

                // Get struct field types from semantic type env
                let struct_fields = self.semantic_type_env.as_ref()
                    .and_then(|env| env.get_struct_fields(&struct_init.name))
                    .cloned();

                // Recursively decorate field expressions, checking for String fields
                let decorated_fields: Vec<(String, DecoratedExpr)> = struct_init.fields.iter()
                    .map(|(field_name, field_expr)| {
                        let mut decorated = self.decorate_expr(field_expr);

                        // Check if field is String type and value is string literal
                        if let Some(ref fields) = struct_fields {
                            if let Some(field_type) = fields.get(field_name) {
                                let field_type_str = Self::type_info_to_swc_type(field_type);
                                let is_string_literal = matches!(&decorated.kind, DecoratedExprKind::Literal(crate::parser::Literal::String(_)));
                                if field_type_str == "String" && is_string_literal {
                                    decorated.metadata.needs_to_string = true;
                                }
                            }
                        }

                        (field_name.clone(), decorated)
                    })
                    .collect();

                DecoratedExpr {
                    kind: DecoratedExprKind::StructInit(DecoratedStructInit {
                        name: struct_init.name.clone(),
                        fields: decorated_fields,
                        span: struct_init.span,
                    }),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,
                        swc_type,
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Struct,
                        span: Some(struct_init.span),

                    needs_to_string: false,},
                }
            }

            Expr::VecInit(vec_init) => {
                let elements = vec_init.elements.iter().map(|e| self.decorate_expr(e)).collect();

                DecoratedExpr {
                    kind: DecoratedExprKind::VecInit(elements),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "Vec".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(vec_init.span),
                    
                    needs_to_string: false,},
                }
            }

            Expr::If(if_expr) => {
                let condition = self.decorate_expr(&if_expr.condition);
                let then_branch = self.decorate_block(&if_expr.then_branch);
                let else_branch = if_expr.else_branch.as_ref().map(|b| self.decorate_block(b));

                // Decorate the pattern if present (for if-let expressions)
                let pattern = if_expr.pattern.as_ref().map(|p| {
                    let condition_type = condition.metadata.swc_type.clone();
                    self.decorate_pattern_with_context(p, &condition_type)
                });

                DecoratedExpr {
                    kind: DecoratedExprKind::If(Box::new(DecoratedIfExpr {
                        condition,
                        pattern,
                        then_branch,
                        else_branch,
                    })),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,
                        swc_type: "UserDefined".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,

                    needs_to_string: false,},
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
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "UserDefined".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    
                    needs_to_string: false,},
                }
            }

            Expr::Closure(closure) => {
                // Recursively decorate the closure body
                let decorated_body = Box::new(self.decorate_expr(&closure.body));
                DecoratedExpr {
                    kind: DecoratedExprKind::Closure(DecoratedClosureExpr {
                        params: closure.params.clone(),
                        body: decorated_body,
                        span: closure.span,
                    }),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None,
                        swc_type: "Closure".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(closure.span),
                    
                    needs_to_string: false,},
                }
            }

            Expr::Ref(ref_expr) => {
                let mut expr = Box::new(self.decorate_expr(&ref_expr.expr));

                // IMPORTANT: If taking a reference to a field with read_conversion,
                // we need to apply the conversion BEFORE taking the reference.
                // Example: &node.callee where callee: Callee with read_conversion ".as_expr().unwrap()"
                // Should become: &node.callee.as_expr().unwrap(), not &(node.callee).as_expr().unwrap()
                if let DecoratedExprKind::Member { ref field_metadata, .. } = expr.kind {
                    if !field_metadata.read_conversion.is_empty() {
                        // Mark that this member access should have its conversion applied
                        // The emit phase will handle emitting &obj.field.conversion() correctly
                    }
                }

                DecoratedExpr {
                    kind: DecoratedExprKind::Ref {
                        mutable: ref_expr.mutable,
                        expr,
                    },
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "Reference".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(ref_expr.span),
                    
                    needs_to_string: false,},
                }
            }

            Expr::Deref(deref_expr) => {
                let expr = Box::new(self.decorate_expr(&deref_expr.expr));

                // When dereferencing, the type is the inner type (unwrap Box/Ref)
                let swc_type = expr.metadata.swc_type.clone();
                let type_kind = expr.metadata.type_kind.clone();

                DecoratedExpr {
                    kind: DecoratedExprKind::Deref(expr),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type,
                        is_boxed: false,
                        is_optional: false,
                        type_kind,
                        span: Some(deref_expr.span),
                    
                    needs_to_string: false,},
                }
            }

            Expr::Assign(assign) => {
                let left = Box::new(self.decorate_expr(&assign.target));
                let mut right = Box::new(self.decorate_expr(&assign.value));

                // Check if we need .to_string() conversion:
                // If left is String type and right is a string literal (&str)
                let left_type = &left.metadata.swc_type;
                let is_string_literal = matches!(&right.kind, DecoratedExprKind::Literal(crate::parser::Literal::String(_)));

                eprintln!("[ASSIGN DEBUG] left_type={}, is_string_literal={}", left_type, is_string_literal);

                // TypeInfo::Str converts to "String" via type_info_to_swc_type
                if left_type == "String" && is_string_literal {
                    eprintln!("[ASSIGN DEBUG] Setting needs_to_string=true");
                    right.metadata.needs_to_string = true;
                }

                DecoratedExpr {
                    kind: DecoratedExprKind::Assign { left, right },
                    metadata: SwcExprMetadata {
                        needs_enum_unwrap: None,
                        swc_type: "()".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Primitive,
                        span: Some(assign.span),
                        needs_to_string: false,
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
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "()".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Primitive,
                        span: Some(compound.span),
                    
                    needs_to_string: false,},
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
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "Range".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(range.span),
                    
                    needs_to_string: false,},
                }
            }

            Expr::Try(try_expr) => {
                let expr = Box::new(self.decorate_expr(try_expr));

                DecoratedExpr {
                    kind: DecoratedExprKind::Try(expr),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "UserDefined".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    
                    needs_to_string: false,},
                }
            }

            Expr::Tuple(tuple) => {
                let elements = tuple.iter().map(|e| self.decorate_expr(e)).collect();

                DecoratedExpr {
                    kind: DecoratedExprKind::Tuple(elements),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "Tuple".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    
                    needs_to_string: false,},
                }
            }

            Expr::Matches(matches) => {
                eprintln!("[DECORATOR] Processing Matches expression");
                // Decorate scrutinee first to get its type
                let expr = Box::new(self.decorate_expr(&matches.scrutinee));
                let scrutinee_type = expr.metadata.swc_type.clone();

                let scrutinee_desc = match matches.scrutinee.as_ref() {
                    Expr::Ident(id) => format!("Ident({})", id.name),
                    Expr::Member(m) => {
                        let obj = if let Expr::Ident(id) = m.object.as_ref() { id.name.as_str() } else { "?" };
                        format!("Member({}.{})", obj, m.property)
                    },
                    Expr::Deref(_) => "Deref".to_string(),
                    _ => "Other".to_string(),
                };
                eprintln!("[DECORATOR MATCHES] Scrutinee {} decorated, type: '{}'", scrutinee_desc, scrutinee_type);

                // Use scrutinee type for pattern
                let pattern = self.decorate_pattern_with_context(&matches.pattern, &scrutinee_type);

                DecoratedExpr {
                    kind: DecoratedExprKind::Matches {
                        expr,
                        pattern,
                    },
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "bool".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Primitive,
                        span: Some(matches.span),
                    
                    needs_to_string: false,},
                }
            }

            Expr::Return(ret) => {
                let value = ret.as_ref().map(|v| Box::new(self.decorate_expr(v)));

                DecoratedExpr {
                    kind: DecoratedExprKind::Return(value),
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "!".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    
                    needs_to_string: false,},
                }
            }

            Expr::Break => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Break,
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "!".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    
                    needs_to_string: false,},
                }
            }

            Expr::Continue => {
                DecoratedExpr {
                    kind: DecoratedExprKind::Continue,
                    metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                        swc_type: "!".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: None,
                    
                    needs_to_string: false,},
                }
            }

            Expr::RegexCall(regex_call) => {
                self.decorate_regex_call(regex_call)
            }

            Expr::CustomPropAccess(access) => {
                self.decorate_custom_prop_access(access)
            }

            Expr::Path(path) => {
                // Path expressions like std::ptr::null() are Rust stdlib paths
                // Emit them as-is since they're valid Rust code
                DecoratedExpr::new(
                    DecoratedExprKind::Ident {
                        name: path.segments.join("::"),
                        ident_metadata: SwcIdentifierMetadata {
                            use_sym: false,
                            deref_pattern: None,
                            span: None,
                        },
                    },
                    SwcExprMetadata {
                        swc_type: "Unknown".to_string(),
                        is_optional: false,
                        is_boxed: false,
                        type_kind: crate::type_system::SwcTypeKind::Unknown,
                        needs_enum_unwrap: None,
                        span: None,
                    
                    needs_to_string: false,},
                )
            }
        }
    }

    /// Strip the binding name from a pattern, keeping only the enum variant
    /// Example: "Expr::Array(array_lit)" → "Expr::Array"
    /// Example: "Pat::Ident(ident)" → "Pat::Ident"
    fn strip_pattern_binding(&self, pattern: &str) -> String {
        // Don't strip struct patterns that contain field matching (e.g., `{ kind: ... }`)
        // These patterns need the full content for proper type discrimination
        if pattern.contains("{ ") {
            return pattern.to_string();
        }
        if let Some(paren_pos) = pattern.find('(') {
            pattern[..paren_pos].to_string()
        } else {
            pattern.to_string()
        }
    }

    /// Map ReluxScript pattern to SWC pattern
    fn map_pattern_to_swc(&self, relux_pattern: &str) -> String {
        // For patterns like "Pattern::ArrayPattern", try looking up just the variant part first
        if relux_pattern.contains("::") {
            let parts: Vec<&str> = relux_pattern.split("::").collect();
            if parts.len() == 2 {
                let variant_name = parts[1];  // "ArrayPattern" from "Pattern::ArrayPattern"

                // Try to find mapping for the variant
                if let Some(mapping) = get_node_mapping(variant_name) {
                    // Use the swc_pattern from the mapping (e.g., "Pat::Array")
                    return mapping.swc_pattern.to_string();
                }

                // Fallback: manual conversion
                let swc_enum = self.reluxscript_to_swc_type(parts[0]);
                let swc_variant = self.reluxscript_to_swc_type(variant_name);
                format!("{}::{}", swc_enum, swc_variant)
            } else {
                relux_pattern.to_string()
            }
        } else if let Some(mapping) = get_node_mapping(relux_pattern) {
            // Direct mapping for simple patterns
            mapping.swc_pattern.to_string()
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
            "Literal" => "Lit",
            "MemberExpression" => "Member",
            "CallExpression" => "Call",
            "BinaryExpression" => "Bin",
            "UnaryExpression" => "Unary",
            _ => relux_type,
        }.to_string()
    }

    /// Determine unwrap strategy based on type being matched
    fn determine_unwrap_strategy(&self, expected_type: &str) -> UnwrapStrategy {
        eprintln!("[UNWRAP STRATEGY] expected_type: '{}'", expected_type);
        // If matching against a Box<T> or &Box<T>, need unwrapping
        if expected_type.starts_with("Box<") || expected_type.starts_with("&Box<") {
            eprintln!("[UNWRAP STRATEGY] Detected Box, using AsRef");
            UnwrapStrategy::AsRef
        } else {
            eprintln!("[UNWRAP STRATEGY] Not a Box, using None");
            UnwrapStrategy::None
        }
    }

    /// Check if an expression needs &* deref for string comparison
    /// e.g., obj.sym (JsWord) == "console" needs &*obj.sym == "console"
    fn needs_sym_deref(&self, expr: &DecoratedExpr, other: &DecoratedExpr, op: BinaryOp) -> bool {
        use crate::parser::BinaryOp;

        // Only for equality/inequality comparisons
        if !matches!(op, BinaryOp::Eq | BinaryOp::NotEq) {
            return false;
        }

        // Check if expr is JsWord/Atom type (from identifier.sym access)
        let is_jsword = expr.metadata.swc_type == "JsWord"
            || expr.metadata.swc_type == "Atom"
            || expr.metadata.swc_type == "IdentName";

        // Check if other side is a string literal
        let is_string_literal = matches!(&other.kind, DecoratedExprKind::Literal(Literal::String(_)));

        // If comparing JsWord to string, need &* deref
        is_jsword && is_string_literal
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
            Type::Named(name) => {
                // Named type like CallExpression, MemberExpression, etc.
                TypeContext::from_reluxscript(name)
            }
            _ => TypeContext::unknown(),
        }
    }

    /// Map ReluxScript visitor method names to SWC visitor method names
    fn map_visitor_method_name(&self, relux_name: &str) -> String {
        use crate::mapping::get_node_mapping_by_visitor;

        // Try to find the mapping for this visitor method
        if let Some(mapping) = get_node_mapping_by_visitor(relux_name) {
            mapping.swc_visitor.to_string()
        } else {
            // If no mapping found, return as-is
            relux_name.to_string()
        }
    }

    /// Map ReluxScript type to SWC type
    fn map_type_to_swc(&self, ty: &Type) -> Type {
        use crate::mapping::{get_node_mapping, get_field_mapping};

        match ty {
            Type::Named(name) => {
                // Try to map the type name to an SWC AST node type
                if let Some(mapping) = get_node_mapping(name) {
                    // Use AstNode for SWC AST types - these should be emitted as-is
                    Type::AstNode(mapping.swc.to_string())
                } else {
                    ty.clone()
                }
            }
            Type::Reference { mutable, inner } => {
                Type::Reference {
                    mutable: *mutable,
                    inner: Box::new(self.map_type_to_swc(inner)),
                }
            }
            Type::Container { name, type_args } => {
                Type::Container {
                    name: name.clone(),
                    type_args: type_args.iter().map(|t| self.map_type_to_swc(t)).collect(),
                }
            }
            Type::FnTrait { params, return_type } => {
                // Map parameter and return types in function trait bounds
                Type::FnTrait {
                    params: params.iter().map(|t| self.map_type_to_swc(t)).collect(),
                    return_type: Box::new(self.map_type_to_swc(return_type)),
                }
            }
            _ => ty.clone(),
        }
    }

    /// Convert ReluxScript Type to CustomPropValue enum variant name
    fn type_to_custom_prop_variant(&self, ty: &crate::parser::Type) -> String {
        use crate::parser::Type;

        match ty {
            Type::Primitive(prim) => match prim.as_str() {
                "bool" => "Bool".to_string(),
                "i32" => "I32".to_string(),
                "i64" => "I64".to_string(),
                "f64" => "F64".to_string(),
                "Str" | "String" => "Str".to_string(),
                _ => "Unknown".to_string(),
            },
            Type::Container { name, type_args } if name == "Vec" => "Vec".to_string(),
            Type::Container { name, type_args } if name == "HashMap" => "Map".to_string(),
            Type::Named(name) => {
                // User-defined type - use the name as variant
                name.clone()
            }
            _ => "Unknown".to_string(),
        }
    }

    /// Generate unwrapper pattern for converting CustomPropValue back to concrete type
    fn gen_unwrapper_pattern(&self, ty: &crate::parser::Type) -> String {
        let variant = self.type_to_custom_prop_variant(ty);

        match variant.as_str() {
            "Bool" => "if let CustomPropValue::Bool(v) = v { Some(v.clone()) } else { None }".to_string(),
            "I32" => "if let CustomPropValue::I32(v) = v { Some(v.clone()) } else { None }".to_string(),
            "I64" => "if let CustomPropValue::I64(v) = v { Some(v.clone()) } else { None }".to_string(),
            "F64" => "if let CustomPropValue::F64(v) = v { Some(v.clone()) } else { None }".to_string(),
            "Str" => "if let CustomPropValue::Str(v) = v { Some(v.clone()) } else { None }".to_string(),
            "Vec" => "if let CustomPropValue::Vec(v) = v { Some(v.clone()) } else { None }".to_string(),
            "Map" => "if let CustomPropValue::Map(v) = v { Some(v.clone()) } else { None }".to_string(),
            variant_name => format!("if let CustomPropValue::{}(v) = v {{ Some(v.clone()) }} else {{ None }}", variant_name),
        }
    }

    /// Check if an expression is a None literal
    fn is_none_literal(&self, expr: &crate::parser::Expr) -> bool {
        matches!(expr, crate::parser::Expr::Ident(id) if id.name == "None")
    }

    /// Decorate custom property assignment with type inference and metadata
    fn decorate_custom_prop_assignment(&mut self, assign: &crate::parser::CustomPropAssignment) -> DecoratedStmt {
        use crate::codegen::decorated_ast::{DecoratedCustomPropAssignment, DecoratedStmt};
        use crate::codegen::swc_metadata::SwcCustomPropAssignmentMetadata;

        // Decorate the node and value expressions
        let decorated_node = self.decorate_expr(&assign.node);
        let decorated_value = self.decorate_expr(&assign.value);

        // Get the node type from the decorated node
        let node_type = decorated_node.metadata.swc_type.clone();

        // Check if this property was already registered with a type
        let key = (node_type.clone(), assign.property.clone());
        let value_type = if let Some(existing_type) = self.custom_props.get(&key) {
            // Use the existing registered type
            existing_type.clone()
        } else {
            // Infer the value type from the decorated expression
            let inferred_type = self.infer_type_from_expr(&decorated_value);

            // Register the property type in our registry
            self.custom_props.insert(key, inferred_type.clone());

            inferred_type
        };

        // Determine the CustomPropValue variant
        let variant = self.type_to_custom_prop_variant(&value_type);

        // Check if this is a deletion (None assignment)
        let is_deletion = self.is_none_literal(&assign.value);

        // Create the decorated assignment
        DecoratedStmt::CustomPropAssignment(Box::new(DecoratedCustomPropAssignment {
            node: decorated_node,
            property: assign.property.clone(),
            value: decorated_value,
            metadata: SwcCustomPropAssignmentMetadata {
                value_type,
                variant,
                is_deletion,
                span: Some(assign.span),
            },
        }))
    }

    /// Decorate custom property access with type tracking and unwrapper generation
    fn decorate_custom_prop_access(&mut self, access: &crate::parser::CustomPropAccess) -> DecoratedExpr {
        use crate::codegen::decorated_ast::{DecoratedCustomPropAccess, DecoratedExprKind};
        use crate::codegen::swc_metadata::SwcCustomPropAccessMetadata;

        // Decorate the node expression
        let decorated_node = self.decorate_expr(&access.node);

        // Get the node type
        let node_type = decorated_node.metadata.swc_type.clone();

        // Look up registered type for this property
        let key = (node_type, access.property.clone());
        let property_type = self.custom_props.get(&key).cloned();

        // Generate unwrapper pattern if we know the type
        let unwrapper_pattern = property_type.as_ref().map(|t| self.gen_unwrapper_pattern(t));

        // The return type is always Option<T>
        let swc_type = if let Some(ref ty) = property_type {
            format!("Option<{}>", self.type_to_swc_string(ty))
        } else {
            "Option<Unknown>".to_string()
        };

        DecoratedExpr {
            kind: DecoratedExprKind::CustomPropAccess(Box::new(DecoratedCustomPropAccess {
                node: Box::new(decorated_node),
                property: access.property.clone(),
                metadata: SwcCustomPropAccessMetadata {
                    property_type,
                    unwrapper_pattern,
                    span: Some(access.span),
                },
            })),
            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                swc_type,
                is_boxed: false,
                is_optional: true,
                type_kind: SwcTypeKind::Unknown,
                span: Some(access.span),
            
            needs_to_string: false,},
        }
    }

    /// Infer type from decorated expression by inspecting its kind
    fn infer_type_from_expr(&self, expr: &DecoratedExpr) -> crate::parser::Type {
        use crate::parser::{Type, Literal};
        use crate::codegen::decorated_ast::DecoratedExprKind;

        // Check if it's a literal - we can get precise type
        if let DecoratedExprKind::Literal(lit) = &expr.kind {
            return match lit {
                Literal::String(_) => Type::Primitive("Str".to_string()),
                Literal::Int(_) => Type::Primitive("i32".to_string()),
                Literal::Float(_) => Type::Primitive("f64".to_string()),
                Literal::Bool(_) => Type::Primitive("bool".to_string()),
                Literal::Null => Type::Named("Null".to_string()),
                Literal::Unit => Type::Named("Unit".to_string()),
            };
        }

        // Otherwise fall back to swc_type
        self.infer_type_from_swc_type(&expr.metadata.swc_type)
    }

    /// Infer ReluxScript Type from SWC type string and optional expression
    fn infer_type_from_swc_type(&self, swc_type: &str) -> crate::parser::Type {
        use crate::parser::Type;

        // Handle common SWC types
        match swc_type {
            "bool" => Type::Primitive("bool".to_string()),
            "i32" | "usize" => Type::Primitive("i32".to_string()),
            "i64" => Type::Primitive("i64".to_string()),
            "f64" => Type::Primitive("f64".to_string()),
            "String" | "&str" => Type::Primitive("Str".to_string()),
            "Literal" => Type::Primitive("Str".to_string()),  // Default literals to Str for now
            s if s.starts_with("Option<") => {
                // Extract inner type
                let inner = s.trim_start_matches("Option<").trim_end_matches(">");
                Type::Container {
                    name: "Option".to_string(),
                    type_args: vec![self.infer_type_from_swc_type(inner)],
                }
            }
            s if s.starts_with("Vec<") => {
                let inner = s.trim_start_matches("Vec<").trim_end_matches(">");
                Type::Container {
                    name: "Vec".to_string(),
                    type_args: vec![self.infer_type_from_swc_type(inner)],
                }
            }
            _ => Type::Named(swc_type.to_string()),
        }
    }

    /// Convert Type to SWC type string
    fn type_to_swc_string(&self, ty: &crate::parser::Type) -> String {
        use crate::parser::Type;

        match ty {
            Type::Primitive(name) => match name.as_str() {
                "Str" => "String".to_string(),
                _ => name.clone(),
            },
            Type::Named(name) => name.clone(),
            Type::Container { name, type_args } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    let args = type_args.iter()
                        .map(|t| self.type_to_swc_string(t))
                        .collect::<Vec<_>>()
                        .join(", ");
                    format!("{}<{}>", name, args)
                }
            }
            _ => "Unknown".to_string(),
        }
    }

    fn decorate_regex_call(&mut self, regex_call: &crate::parser::RegexCall) -> DecoratedExpr {
        use crate::parser::RegexMethod;
        use crate::codegen::decorated_ast::{DecoratedRegexCall, DecoratedExprKind};
        use crate::codegen::swc_metadata::SwcRegexMetadata;

        // Decorate arguments
        let text_arg = self.decorate_expr(&regex_call.text_arg);
        let replacement_arg = regex_call.replacement_arg.as_ref().map(|e| self.decorate_expr(e));

        // Determine if this method needs a helper function
        let needs_helper = matches!(regex_call.method, RegexMethod::Captures);
        let helper_name = if needs_helper {
            Some("__regex_captures".to_string())
        } else {
            None
        };

        // TODO: Implement pattern caching heuristic
        // For now, don't cache patterns (inline them)
        let cache_pattern = false;
        let pattern_id = None;

        // Determine return type for metadata
        let (swc_type, is_optional) = match regex_call.method {
            RegexMethod::Matches => ("bool".to_string(), false),
            RegexMethod::Find => ("String".to_string(), true),
            RegexMethod::FindAll => ("Vec<String>".to_string(), false),
            RegexMethod::Captures => ("__Captures".to_string(), true),
            RegexMethod::Replace | RegexMethod::ReplaceAll => ("String".to_string(), false),
        };

        DecoratedExpr {
            kind: DecoratedExprKind::RegexCall(Box::new(DecoratedRegexCall {
                method: regex_call.method,
                text_arg,
                pattern: regex_call.pattern_arg.clone(),
                replacement_arg,
                metadata: SwcRegexMetadata {
                    cache_pattern,
                    pattern_id,
                    needs_helper,
                    helper_name,
                },
                span: regex_call.span,
            })),
            metadata: SwcExprMetadata { needs_enum_unwrap: None, 
                swc_type,
                is_boxed: false,
                is_optional,
                type_kind: SwcTypeKind::Primitive,
                span: Some(regex_call.span),
            
            needs_to_string: false,},
        }
    }
}

// ============================================================================
// Decorated AST structures
// ============================================================================

#[derive(Debug, Clone)]
pub struct DecoratedProgram {
    pub uses: Vec<crate::parser::UseStmt>,
    pub decl: DecoratedTopLevelDecl,
    /// Whether custom properties are used (set by rewriter when CustomPropAccess is transformed)
    pub uses_custom_props: bool,
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
    /// Structs that should be emitted at module level (before the writer struct)
    pub hoisted_structs: Vec<StructDecl>,
    /// The State struct (if present), whose fields will be flattened into the writer
    pub state_struct: Option<StructDecl>,
}

#[derive(Debug, Clone)]
pub enum DecoratedPluginItem {
    Function(DecoratedFnDecl),
    Struct(StructDecl),
    Enum(EnumDecl),
    Impl(DecoratedImplBlock),
    PreHook(DecoratedFnDecl),
    ExitHook(DecoratedFnDecl),
    Static(DecoratedStaticDecl),
    PubUse(UseStmt),
}

#[derive(Debug, Clone)]
pub struct DecoratedStaticDecl {
    pub name: String,
    pub ty: Type,
    pub init: DecoratedExpr,
    pub is_mut: bool,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct DecoratedFnDecl {
    pub name: String,
    pub type_params: Vec<GenericParam>,
    pub params: Vec<Param>,
    pub return_type: Option<Type>,
    pub where_clause: Vec<WherePredicate>,
    pub body: DecoratedBlock,
}

#[derive(Debug, Clone)]
pub struct DecoratedImplBlock {
    pub target: String,
    pub lifetimes: Vec<String>,
    pub items: Vec<DecoratedFnDecl>,
}
