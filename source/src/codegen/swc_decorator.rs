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
use crate::type_system::{TypeContext, SwcTypeKind};
use crate::mapping::{get_node_mapping, get_field_mapping};
use super::type_context::{get_typed_field_mapping, map_reluxscript_to_swc};
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

    /// Whether we're currently in a writer context
    /// (affects field replacements like self.builder → self)
    is_writer: bool,

    /// Custom property registry
    /// Maps (node_type, property_name) -> inferred_type
    custom_props: HashMap<(String, String), crate::parser::Type>,
}

impl SwcDecorator {
    pub fn new() -> Self {
        Self {
            type_env: HashMap::new(),
            current_params: HashMap::new(),
            semantic_type_env: None,
            is_writer: false,
            custom_props: HashMap::new(),
        }
    }

    /// Create a new decorator with semantic type information
    pub fn with_semantic_types(semantic_type_env: crate::semantic::TypeEnv) -> Self {
        Self {
            type_env: HashMap::new(),
            current_params: HashMap::new(),
            semantic_type_env: Some(semantic_type_env),
            is_writer: false,
            custom_props: HashMap::new(),
        }
    }

    /// Look up variable type from semantic TypeEnv and convert to SWC type string
    fn lookup_semantic_type(&self, var_name: &str) -> Option<String> {
        if let Some(ref type_env) = self.semantic_type_env {
            if let Some(type_info) = type_env.lookup(var_name) {
                return Some(Self::type_info_to_swc_type(type_info));
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
            _ => "Unknown".to_string(),
        }
    }

    /// Decorate a full program (main entry point)
    pub fn decorate_program(&mut self, program: &Program) -> DecoratedProgram {
        DecoratedProgram {
            uses: program.uses.clone(),
            decl: self.decorate_top_level_decl(&program.decl),
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
        }
    }

    fn decorate_fn_decl(&mut self, func: &FnDecl) -> DecoratedFnDecl {
        // Clear and register parameter types
        self.current_params.clear();
        for param in &func.params {
            // First try semantic type env, fallback to annotation parsing
            let type_ctx = if let Some(swc_type) = self.lookup_semantic_type(&param.name) {
                TypeContext {
                    reluxscript_type: param.name.clone(),
                    swc_type,
                    kind: SwcTypeKind::Unknown, // Will be refined
                    known_variant: None,
                    needs_deref: false,
                }
            } else {
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

        DecoratedFnDecl {
            name: swc_name,
            params: swc_params,
            return_type: func.return_type.as_ref().map(|ty| self.map_type_to_swc(ty)),
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

            Stmt::Traverse(traverse) => DecoratedStmt::Traverse(traverse.clone()),

            Stmt::Function(func_decl) => DecoratedStmt::Function(func_decl.clone()),

            Stmt::Verbatim(verbatim) => DecoratedStmt::Verbatim(verbatim.clone()),

            Stmt::CustomPropAssignment(assign) => {
                self.decorate_custom_prop_assignment(assign)
            }
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
            let decorated_pat = self.decorate_pattern_with_context(pattern, condition_type);

            // Register pattern bindings in type environment for the then branch
            self.register_pattern_bindings(pattern, condition_type);

            Some(decorated_pat)
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
                eprintln!("[DEBUG] Registering pattern binding: {} -> {}", name, bound_type);
                self.type_env.insert(name.clone(), TypeContext {
                    reluxscript_type: bound_type.to_string(),
                    swc_type: bound_type.to_string(),
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
                    // Use node mapping for accurate type conversion
                    if let Some(mapping) = get_node_mapping(name) {
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

    /// Decorate expression with .as_ref() suppressed for Option<Box<T>> fields
    /// Used when decorating operands of & reference operator
    fn decorate_expr_suppress_asref(&mut self, expr: &Expr) -> DecoratedExpr {
        match expr {
            Expr::Member(mem) => {
                // Decorate member expression but suppress .as_ref()
                let object = Box::new(self.decorate_expr(&mem.object));
                let object_type = &object.metadata.swc_type;

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
                    let is_boxed = mapping.result_type_swc.starts_with("Box<");
                    SwcFieldMetadata {
                        swc_field_name: mapping.swc_field.to_string(),
                        accessor: if is_boxed {
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
                    // Fallback - also use Direct
                    SwcFieldMetadata {
                        swc_field_name: mem.property.clone(),
                        accessor: FieldAccessor::Direct,
                        field_type: "Unknown".to_string(),
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                        read_conversion: String::new(),
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
                    metadata: SwcExprMetadata {
                        swc_type: member_type,
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(mem.span),
                    },
                }
            }
            // For all other expression types, just use normal decoration
            _ => self.decorate_expr(expr),
        }
    }

    /// Decorate an expression and infer its SWC type
    fn decorate_expr(&mut self, expr: &Expr) -> DecoratedExpr {
        match expr {
            Expr::Ident(ident_expr) => {
                let name = &ident_expr.name;

                // Look up type in environment (local first, then semantic)
                let type_ctx = self.type_env.get(name)
                    .cloned()
                    .map(|ctx| {
                        eprintln!("[DEBUG] Ident '{}' found in type_env: {}", name, ctx.swc_type);
                        ctx
                    })
                    .unwrap_or_else(|| {
                        eprintln!("[DEBUG] Ident '{}' not found in type_env", name);
                        // Try semantic type environment
                        if let Some(swc_type_str) = self.lookup_semantic_type(name) {
                            eprintln!("[DEBUG] Found '{}' in semantic: {}", name, swc_type_str);
                            TypeContext {
                                reluxscript_type: name.clone(),
                                swc_type: swc_type_str.clone(),
                                kind: SwcTypeKind::Unknown, // Will be refined later
                                known_variant: None,
                                needs_deref: false,
                            }
                        } else {
                            eprintln!("[DEBUG] '{}' -> UserDefined (not found)", name);
                            TypeContext::unknown()
                        }
                    });

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
                eprintln!("[DEBUG] Member access: {}.{} (object type: {})",
                    format!("{:?}", mem.object).chars().take(30).collect::<String>(),
                    mem.property, object_type);

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
                } else if let Some(mapping) = get_typed_field_mapping(object_type, &mem.property) {
                    // We have precise mapping!
                    eprintln!("[DEBUG] Field mapping found: {}.{} → {}.{} (needs_deref: {})",
                        object_type, mem.property, object_type, mapping.swc_field, mapping.needs_deref);
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
                        read_conversion: mapping.read_conversion.to_string(),
                    }
                } else {
                    eprintln!("[DEBUG] NO field mapping for: {}.{} - using fallback",
                        object_type, mem.property);
                    // Fallback field mapping - only apply for SWC types, not user-defined types
                    let swc_field = if object_type == "UserDefined" {
                        // Don't apply field mappings to user-defined structs
                        &mem.property
                    } else {
                        // Apply field mappings for SWC types
                        match mem.property.as_str() {
                            "object" => "obj",
                            "property" => "prop",
                            "callee" => "callee",
                            "arguments" => "args",
                            _ => &mem.property,
                        }
                    };

                    SwcFieldMetadata {
                        swc_field_name: swc_field.to_string(),
                        accessor: FieldAccessor::Direct,
                        field_type: "UserDefined".to_string(),
                        source_field: Some(mem.property.clone()),
                        span: Some(mem.span),
                        read_conversion: String::new(),
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
                        swc_type: result_type,
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

                // Check if we need to deref for string comparisons
                // e.g., obj.sym (JsWord) compared to "string" needs &*obj.sym
                let left_needs_deref = self.needs_sym_deref(&left, &right, bin.op);
                let right_needs_deref = self.needs_sym_deref(&right, &left, bin.op);

                DecoratedExpr {
                    kind: DecoratedExprKind::Binary {
                        left,
                        op: bin.op,
                        right,
                        binary_metadata: SwcBinaryMetadata {
                            left_needs_deref,
                            right_needs_deref,
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
                    metadata: SwcExprMetadata {
                        swc_type: "UserDefined".to_string(),
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
                        swc_type: "UserDefined".to_string(),
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
                        swc_type: "UserDefined".to_string(),
                        is_boxed: false,
                        is_optional: false,
                        type_kind: SwcTypeKind::Unknown,
                        span: Some(index.span),
                    },
                }
            }

            Expr::StructInit(struct_init) => {
                // Map ReluxScript type to SWC type
                let swc_type = self.reluxscript_to_swc_type(&struct_init.name);

                // Recursively decorate field expressions
                let decorated_fields = struct_init.fields.iter()
                    .map(|(field_name, field_expr)| {
                        (field_name.clone(), self.decorate_expr(field_expr))
                    })
                    .collect();

                DecoratedExpr {
                    kind: DecoratedExprKind::StructInit(DecoratedStructInit {
                        name: struct_init.name.clone(),
                        fields: decorated_fields,
                        span: struct_init.span,
                    }),
                    metadata: SwcExprMetadata {
                        swc_type,
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
                        swc_type: "UserDefined".to_string(),
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
                        swc_type: "UserDefined".to_string(),
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

                // When dereferencing, the type is the inner type (unwrap Box/Ref)
                let swc_type = expr.metadata.swc_type.clone();
                let type_kind = expr.metadata.type_kind.clone();

                DecoratedExpr {
                    kind: DecoratedExprKind::Deref(expr),
                    metadata: SwcExprMetadata {
                        swc_type,
                        is_boxed: false,
                        is_optional: false,
                        type_kind,
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
                        swc_type: "UserDefined".to_string(),
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

            Expr::RegexCall(regex_call) => {
                self.decorate_regex_call(regex_call)
            }

            Expr::CustomPropAccess(access) => {
                self.decorate_custom_prop_access(access)
            }
        }
    }

    /// Strip the binding name from a pattern, keeping only the enum variant
    /// Example: "Expr::Array(array_lit)" → "Expr::Array"
    /// Example: "Pat::Ident(ident)" → "Pat::Ident"
    fn strip_pattern_binding(&self, pattern: &str) -> String {
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
                // Try to map the type name
                if let Some(mapping) = get_node_mapping(name) {
                    Type::Named(mapping.swc.to_string())
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
            metadata: SwcExprMetadata {
                swc_type,
                is_boxed: false,
                is_optional: true,
                type_kind: SwcTypeKind::Unknown,
                span: Some(access.span),
            },
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
            metadata: SwcExprMetadata {
                swc_type,
                is_boxed: false,
                is_optional,
                type_kind: SwcTypeKind::Primitive,
                span: Some(regex_call.span),
            },
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
