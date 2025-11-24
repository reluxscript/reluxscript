//! Name resolution pass for ReluxScript

use std::collections::HashMap;
use std::path::PathBuf;
use std::fs;
use crate::parser::*;
use crate::{Lexer, Parser, Span};
use crate::semantic::{SemanticError, TypeEnv, TypeInfo, types::ast_type_to_type_info};
use crate::mapping::get_node_mapping;

/// Exported symbols from a module
#[derive(Debug, Clone)]
struct ModuleExports {
    functions: HashMap<String, TypeInfo>,
    structs: HashMap<String, TypeInfo>,
    enums: HashMap<String, TypeInfo>,
}

/// Name resolver - resolves all identifiers and builds the type environment
pub struct Resolver {
    env: TypeEnv,
    errors: Vec<SemanticError>,
    /// Cache of loaded modules (path -> exports)
    module_cache: HashMap<PathBuf, ModuleExports>,
    /// Track which modules are currently being resolved (for circular dependency detection)
    resolving_stack: Vec<PathBuf>,
    /// Base directory for resolving relative imports
    base_dir: PathBuf,
}

impl Resolver {
    pub fn new() -> Self {
        Self::with_base_dir(PathBuf::from("."))
    }

    pub fn with_base_dir(base_dir: PathBuf) -> Self {
        Self {
            env: TypeEnv::new(),
            errors: Vec::new(),
            module_cache: HashMap::new(),
            resolving_stack: Vec::new(),
            base_dir,
        }
    }

    /// Run name resolution
    pub fn resolve(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        // Process use statements - define imported modules
        for use_stmt in &program.uses {
            self.resolve_use(use_stmt);
        }

        match &program.decl {
            TopLevelDecl::Plugin(plugin) => self.resolve_plugin(plugin),
            TopLevelDecl::Writer(writer) => self.resolve_writer(writer),
            TopLevelDecl::Interface(_) => {} // Interfaces don't need resolution
            TopLevelDecl::Module(module) => self.resolve_module(module),
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Resolve a use statement
    fn resolve_use(&mut self, use_stmt: &UseStmt) {
        // Define the module in the environment
        // Valid modules: fs, json, parser, codegen, or file paths starting with "./" or "../"
        let valid_modules = ["fs", "json", "parser", "codegen"];

        // Check if it's a built-in module or a file path
        let is_file_module = use_stmt.path.starts_with("./") || use_stmt.path.starts_with("../");

        if is_file_module {
            // Load and resolve the module file
            match self.load_module(&use_stmt.path, use_stmt.span) {
                Ok(exports) => {
                    // If there are specific imports, resolve them
                    if !use_stmt.imports.is_empty() {
                        for import_name in &use_stmt.imports {
                            // Check if the symbol exists in the module
                            let type_info = exports.functions.get(import_name)
                                .or_else(|| exports.structs.get(import_name))
                                .or_else(|| exports.enums.get(import_name))
                                .cloned();

                            if let Some(ty) = type_info {
                                self.env.define(import_name.clone(), ty);
                            } else {
                                self.errors.push(SemanticError::new(
                                    "RS006",
                                    format!("Module '{}' does not export '{}'", use_stmt.path, import_name),
                                    use_stmt.span,
                                ));
                            }
                        }
                    } else {
                        // Import the whole module
                        let module_name = use_stmt.alias.clone().unwrap_or_else(|| use_stmt.path.clone());
                        self.env.define(
                            module_name.clone(),
                            TypeInfo::Module {
                                name: module_name,
                            },
                        );
                    }
                }
                Err(err) => {
                    self.errors.push(err);
                }
            }
        } else if valid_modules.contains(&use_stmt.path.as_str()) {
            // Built-in module - register it
            if use_stmt.alias.is_some() || use_stmt.imports.is_empty() {
                let module_name = use_stmt.alias.clone().unwrap_or_else(|| use_stmt.path.clone());
                self.env.define(
                    module_name.clone(),
                    TypeInfo::Module {
                        name: module_name,
                    },
                );
            }

            // For built-in modules, we don't know what specific symbols they export
            // So we mark specific imports as Unknown for now
            for import_name in &use_stmt.imports {
                self.env.define(
                    import_name.clone(),
                    TypeInfo::Unknown,
                );
            }
        } else {
            self.errors.push(SemanticError::new(
                "RS007",
                format!("Unknown module: {}", use_stmt.path),
                use_stmt.span,
            ));
        }
    }

    /// Load a module file and extract its exports
    fn load_module(&mut self, module_path: &str, import_span: Span) -> Result<ModuleExports, SemanticError> {
        // Resolve the file path relative to base_dir
        let resolved_path = self.base_dir.join(module_path);
        let canonical_path = resolved_path.canonicalize().unwrap_or(resolved_path.clone());

        // Check for circular dependencies
        if self.resolving_stack.contains(&canonical_path) {
            return Err(SemanticError::new(
                "RS008",
                format!("Circular dependency detected: {}", module_path),
                import_span,
            ));
        }

        // Check cache first
        if let Some(exports) = self.module_cache.get(&canonical_path) {
            return Ok(exports.clone());
        }

        // Load the file
        let source = fs::read_to_string(&resolved_path).map_err(|e| {
            SemanticError::new(
                "RS009",
                format!("Failed to load module '{}': {}", module_path, e),
                import_span,
            )
        })?;

        // Parse the module
        let mut lexer = Lexer::new(&source);
        let tokens = lexer.tokenize();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().map_err(|e| {
            SemanticError::new(
                "RS010",
                format!("Failed to parse module '{}': {}", module_path, e.message),
                import_span,
            )
        })?;

        // Mark as currently resolving
        self.resolving_stack.push(canonical_path.clone());

        // Extract exports from the module
        let exports = self.extract_exports(&program);

        // Remove from resolving stack
        self.resolving_stack.pop();

        // Cache the exports
        self.module_cache.insert(canonical_path, exports.clone());

        Ok(exports)
    }

    /// Extract all exported symbols from a module
    fn extract_exports(&mut self, program: &Program) -> ModuleExports {
        let mut exports = ModuleExports {
            functions: HashMap::new(),
            structs: HashMap::new(),
            enums: HashMap::new(),
        };

        // Determine what to export based on the top-level declaration
        match &program.decl {
            TopLevelDecl::Plugin(plugin) => {
                self.extract_exports_from_items(&plugin.body, &mut exports);
            }
            TopLevelDecl::Writer(writer) => {
                self.extract_exports_from_items(&writer.body, &mut exports);
            }
            TopLevelDecl::Module(module) => {
                self.extract_exports_from_items(&module.items, &mut exports);
            }
            TopLevelDecl::Interface(_) => {
                // Interfaces don't export symbols
            }
        }

        exports
    }

    /// Extract exports from a list of plugin items
    fn extract_exports_from_items(&mut self, items: &[PluginItem], exports: &mut ModuleExports) {
        for item in items {
            match item {
                PluginItem::Function(f) => {
                    let params: Vec<TypeInfo> = f.params.iter()
                        .map(|p| ast_type_to_type_info(&p.ty))
                        .collect();
                    let ret = f.return_type.as_ref()
                        .map(ast_type_to_type_info)
                        .unwrap_or(TypeInfo::Unit);

                    exports.functions.insert(
                        f.name.clone(),
                        TypeInfo::Function {
                            params,
                            ret: Box::new(ret),
                        },
                    );
                }
                PluginItem::Struct(s) => {
                    let mut fields = HashMap::new();
                    for field in &s.fields {
                        let ty = ast_type_to_type_info(&field.ty);
                        fields.insert(field.name.clone(), ty);
                    }
                    exports.structs.insert(
                        s.name.clone(),
                        TypeInfo::Struct {
                            name: s.name.clone(),
                            fields,
                        },
                    );
                }
                PluginItem::Enum(e) => {
                    let mut variants = HashMap::new();
                    for variant in &e.variants {
                        let fields = match &variant.fields {
                            EnumVariantFields::Tuple(types) => {
                                Some(types.iter().map(ast_type_to_type_info).collect())
                            }
                            EnumVariantFields::Struct(named_fields) => {
                                Some(named_fields.iter().map(|(_, ty)| ast_type_to_type_info(ty)).collect())
                            }
                            EnumVariantFields::Unit => None,
                        };
                        variants.insert(variant.name.clone(), fields);
                    }
                    exports.enums.insert(
                        e.name.clone(),
                        TypeInfo::Enum {
                            name: e.name.clone(),
                            variants,
                        },
                    );
                }
                _ => {
                    // Other items (visitor, impl, traverse) don't export symbols
                }
            }
        }
    }

    /// Get the type environment
    pub fn get_env(&self) -> &TypeEnv {
        &self.env
    }

    /// Take ownership of the type environment
    pub fn into_env(self) -> TypeEnv {
        self.env
    }

    fn resolve_plugin(&mut self, plugin: &PluginDecl) {
        // Define the plugin in scope
        self.env.define(
            plugin.name.clone(),
            TypeInfo::Struct {
                name: plugin.name.clone(),
                fields: HashMap::new(),
            },
        );

        // Enter plugin scope
        self.env.push_scope();

        // First pass: collect all struct and enum definitions
        for item in &plugin.body {
            match item {
                PluginItem::Struct(s) => self.declare_struct(s),
                PluginItem::Enum(e) => self.declare_enum(e),
                _ => {}
            }
        }

        // Second pass: collect function signatures
        for item in &plugin.body {
            if let PluginItem::Function(f) = item {
                self.declare_function(f);
            }
        }

        // Third pass: resolve function bodies
        for item in &plugin.body {
            if let PluginItem::Function(f) = item {
                self.resolve_function(f);
            }
        }

        self.env.pop_scope();
    }

    fn resolve_writer(&mut self, writer: &WriterDecl) {
        // Same as plugin for now
        self.env.define(
            writer.name.clone(),
            TypeInfo::Struct {
                name: writer.name.clone(),
                fields: HashMap::new(),
            },
        );

        self.env.push_scope();

        for item in &writer.body {
            match item {
                PluginItem::Struct(s) => self.declare_struct(s),
                PluginItem::Enum(e) => self.declare_enum(e),
                _ => {}
            }
        }

        for item in &writer.body {
            if let PluginItem::Function(f) = item {
                self.declare_function(f);
            }
        }

        for item in &writer.body {
            if let PluginItem::Function(f) = item {
                self.resolve_function(f);
            }
        }

        self.env.pop_scope();
    }

    fn resolve_module(&mut self, module: &ModuleDecl) {
        // Modules don't define themselves in scope (they're imported via use statements)
        // Just resolve their contents
        self.env.push_scope();

        // First pass: declare structs and enums
        for item in &module.items {
            match item {
                PluginItem::Struct(s) => self.declare_struct(s),
                PluginItem::Enum(e) => self.declare_enum(e),
                _ => {}
            }
        }

        // Second pass: declare functions
        for item in &module.items {
            if let PluginItem::Function(f) = item {
                self.declare_function(f);
            }
        }

        // Third pass: resolve function bodies
        for item in &module.items {
            if let PluginItem::Function(f) = item {
                self.resolve_function(f);
            }
        }

        self.env.pop_scope();
    }

    fn declare_struct(&mut self, s: &StructDecl) {
        let mut fields = HashMap::new();
        for field in &s.fields {
            let ty = ast_type_to_type_info(&field.ty);
            fields.insert(field.name.clone(), ty);
        }
        self.env.define_struct(s.name.clone(), fields);
    }

    fn declare_enum(&mut self, e: &EnumDecl) {
        let mut variants = HashMap::new();
        for variant in &e.variants {
            let fields = match &variant.fields {
                EnumVariantFields::Tuple(types) => {
                    Some(types.iter().map(ast_type_to_type_info).collect())
                }
                EnumVariantFields::Struct(named_fields) => {
                    Some(named_fields.iter().map(|(_, ty)| ast_type_to_type_info(ty)).collect())
                }
                EnumVariantFields::Unit => None,
            };
            variants.insert(variant.name.clone(), fields);
        }
        self.env.define_enum(e.name.clone(), variants);
    }

    fn declare_function(&mut self, f: &FnDecl) {
        let params: Vec<TypeInfo> = f.params.iter().map(|p| ast_type_to_type_info(&p.ty)).collect();
        let ret = f
            .return_type
            .as_ref()
            .map(ast_type_to_type_info)
            .unwrap_or(TypeInfo::Unit);
        self.env.define_function(f.name.clone(), params, ret);
    }

    fn resolve_function(&mut self, f: &FnDecl) {
        self.env.push_scope();

        // Define parameters
        for param in &f.params {
            let ty = ast_type_to_type_info(&param.ty);
            if self.env.is_defined_in_current_scope(&param.name) {
                self.errors.push(SemanticError::new(
                    "RS004",
                    format!("Duplicate parameter name: {}", param.name),
                    param.span,
                ));
            } else {
                self.env.define(param.name.clone(), ty);
            }
        }

        // Resolve body
        self.resolve_block(&f.body);

        self.env.pop_scope();
    }

    fn resolve_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.resolve_stmt(stmt);
        }
    }

    fn resolve_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_stmt) => {
                // Resolve initializer first
                self.resolve_expr(&let_stmt.init);

                // Check for redefinition in same scope (only for simple identifiers)
                if let Pattern::Ident(ref name) = let_stmt.pattern {
                    if self.env.is_defined_in_current_scope(name) {
                        self.errors.push(SemanticError::new(
                            "RS005",
                            format!("Variable '{}' already defined in this scope", name),
                            let_stmt.span,
                        ));
                    }
                }

                // Resolve the pattern and define variables
                self.resolve_pattern(&let_stmt.pattern);

                // Determine type
                let ty = if let Some(ref type_ann) = let_stmt.ty {
                    ast_type_to_type_info(type_ann)
                } else {
                    // Type will be inferred during type checking
                    self.env.fresh_var()
                };

                // Define variables from the pattern
                self.define_pattern(&let_stmt.pattern, ty);
            }

            Stmt::Const(const_stmt) => {
                self.resolve_expr(&const_stmt.init);

                if self.env.is_defined_in_current_scope(&const_stmt.name) {
                    self.errors.push(SemanticError::new(
                        "RS005",
                        format!("Constant '{}' already defined in this scope", const_stmt.name),
                        const_stmt.span,
                    ));
                }

                let ty = if let Some(ref type_ann) = const_stmt.ty {
                    ast_type_to_type_info(type_ann)
                } else {
                    self.env.fresh_var()
                };

                self.env.define(const_stmt.name.clone(), ty);
            }

            Stmt::Expr(expr_stmt) => {
                self.resolve_expr(&expr_stmt.expr);
            }

            Stmt::If(if_stmt) => {
                self.resolve_expr(&if_stmt.condition);
                self.env.push_scope();
                // If this is an if-let, resolve the pattern to bind variables
                if let Some(ref pattern) = if_stmt.pattern {
                    self.resolve_pattern(pattern);
                }
                self.resolve_block(&if_stmt.then_branch);
                self.env.pop_scope();

                for (cond, block) in &if_stmt.else_if_branches {
                    self.resolve_expr(cond);
                    self.env.push_scope();
                    self.resolve_block(block);
                    self.env.pop_scope();
                }

                if let Some(ref else_block) = if_stmt.else_branch {
                    self.env.push_scope();
                    self.resolve_block(else_block);
                    self.env.pop_scope();
                }
            }

            Stmt::Match(match_stmt) => {
                self.resolve_expr(&match_stmt.scrutinee);
                for arm in &match_stmt.arms {
                    self.env.push_scope();
                    self.resolve_pattern(&arm.pattern);
                    self.resolve_expr(&arm.body);
                    self.env.pop_scope();
                }
            }

            Stmt::For(for_stmt) => {
                self.resolve_expr(&for_stmt.iter);
                self.env.push_scope();
                // Define variables from pattern
                // Use Unknown type for loop variables (type checker will refine this)
                self.define_pattern(&for_stmt.pattern, TypeInfo::Unknown);
                self.resolve_block(&for_stmt.body);
                self.env.pop_scope();
            }

            Stmt::While(while_stmt) => {
                self.resolve_expr(&while_stmt.condition);
                self.env.push_scope();
                self.resolve_block(&while_stmt.body);
                self.env.pop_scope();
            }

            Stmt::Loop(loop_stmt) => {
                self.env.push_scope();
                self.resolve_block(&loop_stmt.body);
                self.env.pop_scope();
            }

            Stmt::Return(return_stmt) => {
                if let Some(ref value) = return_stmt.value {
                    self.resolve_expr(value);
                }
            }

            Stmt::Break(_) | Stmt::Continue(_) => {}

            Stmt::Traverse(traverse_stmt) => {
                // Resolve the target expression
                self.resolve_expr(&traverse_stmt.target);

                // Handle the traverse kind
                match &traverse_stmt.kind {
                    crate::parser::TraverseKind::Inline(inline) => {
                        // Create a new scope for the inline visitor
                        self.env.push_scope();

                        // Resolve state variables
                        for let_stmt in &inline.state {
                            self.resolve_expr(&let_stmt.init);
                            self.resolve_pattern(&let_stmt.pattern);
                            let ty = if let Some(ref type_ann) = let_stmt.ty {
                                ast_type_to_type_info(type_ann)
                            } else {
                                self.env.fresh_var()
                            };
                            self.define_pattern(&let_stmt.pattern, ty);
                        }

                        // Resolve methods
                        for method in &inline.methods {
                            self.resolve_function(method);
                        }

                        self.env.pop_scope();
                    }
                    crate::parser::TraverseKind::Delegated(visitor_name) => {
                        // Check if the visitor exists (would need to track plugin definitions)
                        // For now, just note it for later validation
                        let _ = visitor_name;
                    }
                }
            }

            Stmt::Function(fn_decl) => {
                // Nested function declaration
                // Define the function in the current scope
                let param_types: Vec<TypeInfo> = fn_decl.params.iter()
                    .map(|p| ast_type_to_type_info(&p.ty))
                    .collect();
                let ret_type = fn_decl.return_type.as_ref()
                    .map(ast_type_to_type_info)
                    .unwrap_or(TypeInfo::Unit);
                let fn_type = TypeInfo::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                };
                if self.env.is_defined_in_current_scope(&fn_decl.name) {
                    self.errors.push(SemanticError::new(
                        "RS005",
                        format!("Function '{}' already defined in this scope", fn_decl.name),
                        fn_decl.span,
                    ));
                }
                self.env.define(fn_decl.name.clone(), fn_type);

                // Resolve the function body in a new scope
                self.env.push_scope();
                // Define parameters in the function scope
                for param in &fn_decl.params {
                    let param_type = ast_type_to_type_info(&param.ty);
                    self.env.define(param.name.clone(), param_type);
                }
                // Resolve the body
                self.resolve_block(&fn_decl.body);
                self.env.pop_scope();
            }

            Stmt::Verbatim(_) => {
                // Verbatim blocks are opaque to semantic analysis
                // No type checking or variable tracking
            }
        }
    }

    fn resolve_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Ident(name) => {
                // Bind the pattern variable
                let var_type = self.env.fresh_var();
                self.env.define(name.clone(), var_type);
            }
            Pattern::Tuple(patterns) => {
                for pat in patterns {
                    self.resolve_pattern(pat);
                }
            }
            Pattern::Array(patterns) => {
                for pat in patterns {
                    self.resolve_pattern(pat);
                }
            }
            Pattern::Object(props) => {
                for prop in props {
                    match prop {
                        crate::parser::ObjectPatternProp::Shorthand(name) => {
                            let var_type = self.env.fresh_var();
                            self.env.define(name.clone(), var_type);
                        }
                        crate::parser::ObjectPatternProp::KeyValue { value, .. } => {
                            self.resolve_pattern(value);
                        }
                        crate::parser::ObjectPatternProp::Rest(name) => {
                            let var_type = self.env.fresh_var();
                            self.env.define(name.clone(), var_type);
                        }
                        crate::parser::ObjectPatternProp::Or(patterns) => {
                            // For OR patterns in object props, resolve all branches
                            for pat in patterns {
                                self.resolve_pattern(pat);
                            }
                        }
                    }
                }
            }
            Pattern::Rest(inner) => {
                self.resolve_pattern(inner);
            }
            Pattern::Struct { fields, .. } => {
                for (_, pat) in fields {
                    self.resolve_pattern(pat);
                }
            }
            Pattern::Or(patterns) => {
                for pat in patterns {
                    self.resolve_pattern(pat);
                }
            }
            Pattern::Variant { inner, .. } => {
                // Resolve inner pattern if present (e.g., Some(x) -> resolve x)
                if let Some(inner_pat) = inner {
                    self.resolve_pattern(inner_pat);
                }
            }
            Pattern::Ref { pattern: inner, .. } => {
                // ref pattern - resolve the inner pattern
                self.resolve_pattern(inner);
            }
            Pattern::Literal(_) | Pattern::Wildcard => {}
        }
    }

    /// Define variables from a pattern with a given type info
    fn define_pattern(&mut self, pattern: &Pattern, type_info: TypeInfo) {
        match pattern {
            Pattern::Ident(name) => {
                self.env.define(name.clone(), type_info);
            }
            Pattern::Tuple(patterns) => {
                // Extract tuple element types if available
                match &type_info {
                    TypeInfo::Tuple(elem_types) => {
                        for (i, pat) in patterns.iter().enumerate() {
                            let elem_type = elem_types.get(i)
                                .cloned()
                                .unwrap_or(TypeInfo::Unknown);
                            self.define_pattern(pat, elem_type);
                        }
                    }
                    _ => {
                        // If not a tuple type, give all elements Unknown type
                        for pat in patterns {
                            self.define_pattern(pat, TypeInfo::Unknown);
                        }
                    }
                }
            }
            Pattern::Array(_) => {
                // Array destructuring not yet implemented
            }
            Pattern::Object(_) => {
                // Object destructuring not yet implemented
            }
            Pattern::Rest(_) => {
                // Rest pattern not yet implemented
            }
            Pattern::Or(patterns) => {
                // For OR patterns, all branches must bind the same variables with same types
                // For now, just define variables from the first pattern
                if let Some(first) = patterns.first() {
                    self.define_pattern(first, type_info);
                }
            }
            Pattern::Ref { pattern: inner, .. } => {
                // ref pattern - define variables from the inner pattern
                // The type remains the same (ref doesn't change the type in our IR)
                self.define_pattern(inner, type_info);
            }
            Pattern::Struct { .. } | Pattern::Variant { .. } | Pattern::Literal(_) | Pattern::Wildcard => {
                // No variables to define
            }
        }
    }

    fn resolve_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(ident) => {
                if self.env.lookup(&ident.name).is_none() {
                    // Check for special names and built-in macros
                    let is_special = matches!(ident.name.as_str(),
                        "self" | "Self" | "matches!" | "format!" | "format" | "vec!" | "Some" | "None" | "Ok" | "Err" | "String" | "HashMap" | "HashSet" | "Vec" | "Option" | "Result" | "Box" | "CodeBuilder" | "_"
                    );
                    // Check if it's a known AST node type (used in matches!)
                    let is_ast_type = get_node_mapping(&ident.name).is_some();
                    // Check if it's a pattern placeholder from matches! macro
                    let is_pattern_placeholder = ident.name.starts_with("_pattern_");
                    if !is_special && !is_ast_type && !is_pattern_placeholder {
                        self.errors.push(SemanticError::new(
                            "RS006",
                            format!("Undefined variable: {}", ident.name),
                            ident.span,
                        ));
                    }
                }
            }

            Expr::Binary(binary) => {
                self.resolve_expr(&binary.left);
                self.resolve_expr(&binary.right);
            }

            Expr::Unary(unary) => {
                self.resolve_expr(&unary.operand);
            }

            Expr::Call(call) => {
                // Check if callee is an undefined function
                if let Expr::Ident(ident) = &*call.callee {
                    if self.env.lookup(&ident.name).is_none() {
                        let is_special = matches!(ident.name.as_str(),
                            "self" | "Self" | "matches!" | "format!" | "format" | "vec!" | "Some" | "None" | "Ok" | "Err" | "String" | "HashMap" | "HashSet" | "Vec" | "Option" | "Result" | "Box" | "CodeBuilder" | "_"
                        );
                        let is_ast_type = get_node_mapping(&ident.name).is_some();
                        if !is_special && !is_ast_type {
                            self.errors.push(SemanticError::new(
                                "RS006",
                                format!("Undefined function: {}", ident.name),
                                ident.span,
                            ));
                        }
                    }
                } else {
                    self.resolve_expr(&call.callee);
                }
                for arg in &call.args {
                    self.resolve_expr(arg);
                }
            }

            Expr::Member(member) => {
                self.resolve_expr(&member.object);
                // Property name doesn't need resolution
            }

            Expr::Index(index) => {
                self.resolve_expr(&index.object);
                self.resolve_expr(&index.index);
            }

            Expr::StructInit(init) => {
                // Check struct exists
                if self.env.get_struct_fields(&init.name).is_none()
                   && self.env.lookup(&init.name).is_none() {
                    self.errors.push(SemanticError::new(
                        "RS007",
                        format!("Unknown struct: {}", init.name),
                        init.span,
                    ));
                }
                for (_, value) in &init.fields {
                    self.resolve_expr(value);
                }
            }

            Expr::VecInit(vec_init) => {
                for elem in &vec_init.elements {
                    self.resolve_expr(elem);
                }
            }

            Expr::If(if_expr) => {
                self.resolve_expr(&if_expr.condition);
                self.env.push_scope();
                // If this is an if-let, define the pattern bindings in the then-branch scope
                if let Some(ref pattern) = if_expr.pattern {
                    self.resolve_pattern(pattern);
                }
                self.resolve_block(&if_expr.then_branch);
                self.env.pop_scope();
                if let Some(ref else_block) = if_expr.else_branch {
                    self.env.push_scope();
                    self.resolve_block(else_block);
                    self.env.pop_scope();
                }
            }

            Expr::Match(match_expr) => {
                self.resolve_expr(&match_expr.scrutinee);
                for arm in &match_expr.arms {
                    self.env.push_scope();
                    self.resolve_pattern(&arm.pattern);
                    self.resolve_expr(&arm.body);
                    self.env.pop_scope();
                }
            }

            Expr::Closure(closure) => {
                self.env.push_scope();
                for param in &closure.params {
                    let var_type = self.env.fresh_var();
                    self.env.define(param.clone(), var_type);
                }
                self.resolve_expr(&closure.body);
                self.env.pop_scope();
            }

            Expr::Ref(ref_expr) => {
                self.resolve_expr(&ref_expr.expr);
            }

            Expr::Deref(deref_expr) => {
                self.resolve_expr(&deref_expr.expr);
            }

            Expr::Assign(assign) => {
                self.resolve_expr(&assign.target);
                self.resolve_expr(&assign.value);
            }

            Expr::CompoundAssign(compound) => {
                self.resolve_expr(&compound.target);
                self.resolve_expr(&compound.value);
            }

            Expr::Range(range) => {
                if let Some(ref start) = range.start {
                    self.resolve_expr(start);
                }
                if let Some(ref end) = range.end {
                    self.resolve_expr(end);
                }
            }

            Expr::Paren(inner) => {
                self.resolve_expr(inner);
            }

            Expr::Tuple(elements) => {
                for elem in elements {
                    self.resolve_expr(elem);
                }
            }

            Expr::Block(block) => {
                // Block expressions create a new scope
                self.env.push_scope();
                for stmt in &block.stmts {
                    self.resolve_stmt(stmt);
                }
                self.env.pop_scope();
            }

            Expr::Try(inner) => {
                self.resolve_expr(inner);
            }

            Expr::Matches(matches_expr) => {
                // Resolve the scrutinee expression
                self.resolve_expr(&matches_expr.scrutinee);
                // The pattern doesn't need resolution (it's a pattern, not an expression)
                // Pattern variables are only bound within the matches! result context
            }

            Expr::Return(value) => {
                if let Some(ref expr) = value {
                    self.resolve_expr(expr);
                }
            }

            Expr::Break => {}
            Expr::Continue => {}

            Expr::Literal(_) => {}
        }
    }
}

impl Default for Resolver {
    fn default() -> Self {
        Self::new()
    }
}
