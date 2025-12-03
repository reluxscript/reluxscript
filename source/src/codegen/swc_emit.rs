//! SWC Emitter - Emits Rust code from decorated/rewritten AST
//!
//! This is the final stage of the pipeline:
//! 1. Receives transformed, decorated AST with all metadata
//! 2. Emits Rust code as strings
//! 3. NO semantic decisions - just string emission based on AST structure
//!
//! The emitter is "dumb" by design - all transformations happened in earlier stages.

use super::swc_decorator::{DecoratedProgram, DecoratedTopLevelDecl, DecoratedPlugin, DecoratedWriter, DecoratedModule, DecoratedPluginItem, DecoratedFnDecl, DecoratedImplBlock};
use super::decorated_ast::*;
use super::swc_metadata::*;
use crate::parser::*;

/// SwcEmitter generates Rust code from decorated AST
pub struct SwcEmitter {
    /// Output buffer
    output: String,

    /// Current indentation level
    indent: usize,

    /// Plugin/writer name
    name: String,

    /// Whether we're in a writer context
    is_writer: bool,

    /// Whether to add HashMap import
    uses_hashmap: bool,

    /// Whether to add HashSet import
    uses_hashset: bool,

    /// Whether json serialization is needed
    uses_json: bool,

    /// Whether fs module is used
    uses_fs: bool,

    /// Whether parser module is used
    uses_parser: bool,

    /// Whether codegen module is used
    uses_codegen: bool,

    /// Whether CodeBuilder type is used
    uses_codebuilder: bool,

    /// Whether regex captures helper is needed
    needs_regex_captures_helper: bool,

    /// Whether regex crate is used
    uses_regex: bool,

    /// Whether custom AST properties are used
    uses_custom_props: bool,

    /// Set of custom types used in custom properties (for CustomPropValue enum)
    custom_prop_types: std::collections::HashSet<String>,

    /// Whether this is a module file (not the main lib.rs)
    /// If true, use `use crate::...` instead of `mod ...` for imports
    is_module: bool,

    /// All module names to declare in lib.rs (for multi-file output)
    all_module_names: std::collections::HashSet<String>,

    /// Whether this is an inline module (skip headers/imports, just emit functions)
    is_inline: bool,

    /// Skip file module imports (for when modules are inlined)
    skip_file_imports: bool,
}

impl SwcEmitter {
    /// Create new emitter
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            name: String::new(),
            is_writer: false,
            uses_hashmap: false,
            uses_hashset: false,
            uses_json: false,
            uses_fs: false,
            uses_parser: false,
            uses_codegen: false,
            uses_codebuilder: false,
            needs_regex_captures_helper: false,
            uses_regex: false,
            uses_custom_props: false,
            custom_prop_types: std::collections::HashSet::new(),
            is_module: false,
            all_module_names: std::collections::HashSet::new(),
            is_inline: false,
            skip_file_imports: false,
        }
    }

    /// Create new emitter for when modules are inlined (skip file-based use statements)
    pub fn new_with_inlined_modules() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            name: String::new(),
            is_writer: false,
            uses_hashmap: false,
            uses_hashset: false,
            uses_json: false,
            uses_fs: false,
            uses_parser: false,
            uses_codegen: false,
            uses_codebuilder: false,
            needs_regex_captures_helper: false,
            uses_regex: false,
            uses_custom_props: false,
            custom_prop_types: std::collections::HashSet::new(),
            is_module: false,
            all_module_names: std::collections::HashSet::new(),
            is_inline: false,
            skip_file_imports: true,
        }
    }

    /// Create new emitter for inlined modules (skip headers/imports, just emit functions)
    pub fn new_inline() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            name: String::new(),
            is_writer: false,
            uses_hashmap: false,
            uses_hashset: false,
            uses_json: false,
            uses_fs: false,
            uses_parser: false,
            uses_codegen: false,
            uses_codebuilder: false,
            needs_regex_captures_helper: false,
            uses_regex: false,
            uses_custom_props: false,
            custom_prop_types: std::collections::HashSet::new(),
            is_module: false,
            all_module_names: std::collections::HashSet::new(),
            is_inline: true,
            skip_file_imports: true,
        }
    }

    /// Create new emitter with all module names to declare (for multi-file lib.rs)
    #[allow(dead_code)]
    pub fn new_with_all_modules(all_module_names: std::collections::HashSet<String>) -> Self {
        Self {
            output: String::new(),
            indent: 0,
            name: String::new(),
            is_writer: false,
            uses_hashmap: false,
            uses_hashset: false,
            uses_json: false,
            uses_fs: false,
            uses_parser: false,
            uses_codegen: false,
            uses_codebuilder: false,
            needs_regex_captures_helper: false,
            uses_regex: false,
            uses_custom_props: false,
            custom_prop_types: std::collections::HashSet::new(),
            is_module: false,
            all_module_names,
            is_inline: false,
            skip_file_imports: false,
        }
    }

    /// Create new emitter for a module file (uses `use crate::...` instead of `mod ...`)
    #[allow(dead_code)]
    pub fn new_module() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            name: String::new(),
            is_writer: false,
            uses_hashmap: false,
            uses_hashset: false,
            uses_json: false,
            uses_fs: false,
            uses_parser: false,
            uses_codegen: false,
            uses_codebuilder: false,
            needs_regex_captures_helper: false,
            uses_regex: false,
            uses_custom_props: false,
            custom_prop_types: std::collections::HashSet::new(),
            is_module: true,
            all_module_names: std::collections::HashSet::new(),
            is_inline: false,
            skip_file_imports: false,
        }
    }

    /// Main entry point: emit entire program
    pub fn emit_program(&mut self, program: &DecoratedProgram) -> String {
        // ALWAYS detect imports first - this populates critical emitter state
        self.detect_imports(program);

        // For inline mode, just emit the functions without headers/imports/helpers
        if self.is_inline {
            self.emit_top_level_decl(&program.decl);
            return std::mem::take(&mut self.output);
        }

        // Emit header with conditional imports
        self.emit_header();

        // Emit user module imports (from use statements)
        self.emit_user_imports(&program.uses);

        // Emit the main code
        self.emit_top_level_decl(&program.decl);

        // Emit helper function modules if needed
        if self.uses_parser {
            self.emit_line("");
            self.emit_parser_helpers();
        }

        if self.uses_codegen {
            self.emit_line("");
            self.emit_codegen_helpers();
        }

        if self.uses_codebuilder {
            self.emit_line("");
            self.emit_codebuilder_helper();
        }

        if self.needs_regex_captures_helper {
            self.emit_line("");
            self.emit_regex_helpers();
        }

        std::mem::take(&mut self.output)
    }

    // ========================================================================
    // IMPORT DETECTION
    // ========================================================================

    fn detect_imports(&mut self, program: &DecoratedProgram) {
        // Scan use statements to detect which modules are imported
        for use_stmt in &program.uses {
            match use_stmt.path.as_str() {
                "codegen" => self.uses_codegen = true,
                "parser" => self.uses_parser = true,
                "fs" => self.uses_fs = true,
                "json" => self.uses_json = true,
                "HashMap" => self.uses_hashmap = true,
                "HashSet" => self.uses_hashset = true,
                _ => {
                    // File modules or unknown modules - ignore for now
                }
            }
        }

        // Walk AST to detect regex usage
        self.detect_regex_usage_in_decl(&program.decl);
    }

    fn detect_regex_usage_in_decl(&mut self, decl: &crate::codegen::swc_decorator::DecoratedTopLevelDecl) {
        use crate::codegen::swc_decorator::{DecoratedTopLevelDecl, DecoratedPluginItem};
        match decl {
            DecoratedTopLevelDecl::Plugin(plugin) => {
                for item in &plugin.body {
                    match item {
                        DecoratedPluginItem::Function(func) => {
                            self.detect_regex_usage_in_block(&func.body);
                        }
                        DecoratedPluginItem::Struct(struct_decl) => {
                            self.detect_hashmap_hashset_in_struct(struct_decl);
                        }
                        _ => {}
                    }
                }
            }
            DecoratedTopLevelDecl::Writer(writer) => {
                // Check hoisted structs (module-level structs)
                for struct_decl in &writer.hoisted_structs {
                    self.detect_hashmap_hashset_in_struct(struct_decl);
                }

                // Check State struct
                if let Some(state_struct) = &writer.state_struct {
                    self.detect_hashmap_hashset_in_struct(state_struct);
                }

                // Check items in writer body
                for item in &writer.body {
                    match item {
                        DecoratedPluginItem::Function(func) => {
                            self.detect_regex_usage_in_block(&func.body);
                        }
                        DecoratedPluginItem::PreHook(func) => {
                            self.detect_regex_usage_in_block(&func.body);
                        }
                        DecoratedPluginItem::ExitHook(func) => {
                            self.detect_regex_usage_in_block(&func.body);
                        }
                        DecoratedPluginItem::Struct(struct_decl) => {
                            self.detect_hashmap_hashset_in_struct(struct_decl);
                        }
                        _ => {}
                    }
                }
            }
            DecoratedTopLevelDecl::Module(module) => {
                // Scan module items for usage
                for item in &module.items {
                    match item {
                        DecoratedPluginItem::Function(func) => {
                            self.detect_regex_usage_in_block(&func.body);
                        }
                        DecoratedPluginItem::Struct(struct_decl) => {
                            self.detect_hashmap_hashset_in_struct(struct_decl);
                        }
                        _ => {}
                    }
                }
            }
            DecoratedTopLevelDecl::Undecorated(top_level) => {
                // Scan module-level items for HashMap/HashSet usage
                use crate::parser::{TopLevelDecl, PluginItem};
                if let TopLevelDecl::Module(module) = top_level {
                    for item in &module.items {
                        if let PluginItem::Struct(struct_decl) = item {
                            self.detect_hashmap_hashset_in_struct(struct_decl);
                        }
                    }
                }
            }
        }
    }

    fn detect_hashmap_hashset_in_struct(&mut self, struct_decl: &crate::parser::StructDecl) {
        for field in &struct_decl.fields {
            self.detect_hashmap_hashset_in_type(&field.ty);
        }
    }

    fn detect_hashmap_hashset_in_type(&mut self, ty: &crate::parser::Type) {
        use crate::parser::Type;
        match ty {
            Type::Container { name, type_args } => {
                match name.as_str() {
                    "HashMap" => self.uses_hashmap = true,
                    "HashSet" => self.uses_hashset = true,
                    "CodeBuilder" => self.uses_codebuilder = true,
                    _ => {}
                }
                // Recursively check type arguments
                for ty_arg in type_args {
                    self.detect_hashmap_hashset_in_type(ty_arg);
                }
            }
            Type::Reference { inner, .. } => {
                self.detect_hashmap_hashset_in_type(inner);
            }
            Type::Optional(inner) => {
                self.detect_hashmap_hashset_in_type(inner);
            }
            Type::Array { element } => {
                self.detect_hashmap_hashset_in_type(element);
            }
            Type::Tuple(types) => {
                for ty in types {
                    self.detect_hashmap_hashset_in_type(ty);
                }
            }
            Type::Named(name) => {
                if name == "CodeBuilder" {
                    self.uses_codebuilder = true;
                }
            }
            _ => {}
        }
    }

    fn detect_regex_usage_in_block(&mut self, block: &crate::codegen::decorated_ast::DecoratedBlock) {
        use crate::codegen::decorated_ast::{DecoratedStmt, DecoratedExprKind};
        for stmt in &block.stmts {
            match stmt {
                DecoratedStmt::Let(let_stmt) => {
                    if let Some(ref init) = let_stmt.init {
                        self.detect_regex_usage_in_expr(init);
                    }
                }
                DecoratedStmt::Expr(expr) => {
                    self.detect_regex_usage_in_expr(expr);
                }
                DecoratedStmt::If(if_stmt) => {
                    self.detect_regex_usage_in_expr(&if_stmt.condition);
                    self.detect_regex_usage_in_block(&if_stmt.then_branch);
                    if let Some(ref else_branch) = if_stmt.else_branch {
                        self.detect_regex_usage_in_block(else_branch);
                    }
                }
                DecoratedStmt::Match(match_stmt) => {
                    self.detect_regex_usage_in_expr(&match_stmt.expr);
                    for arm in &match_stmt.arms {
                        self.detect_regex_usage_in_block(&arm.body);
                    }
                }
                DecoratedStmt::Return(Some(expr)) => {
                    self.detect_regex_usage_in_expr(expr);
                }
                _ => {}
            }
        }
    }

    fn detect_regex_usage_in_expr(&mut self, expr: &crate::codegen::decorated_ast::DecoratedExpr) {
        use crate::codegen::decorated_ast::DecoratedExprKind;

        // Check if this expression is a regex call
        if matches!(expr.kind, DecoratedExprKind::RegexCall(_)) {
            self.uses_regex = true;
            if let DecoratedExprKind::RegexCall(ref regex_call) = expr.kind {
                if regex_call.metadata.needs_helper {
                    self.needs_regex_captures_helper = true;
                }
            }
            return;
        }

        // Recursively check child expressions
        match &expr.kind {
            DecoratedExprKind::Call(call) => {
                // Check for CodeBuilder::new() calls
                if let DecoratedExprKind::Member { object, property, .. } = &call.callee.kind {
                    if let DecoratedExprKind::Ident { name, .. } = &object.kind {
                        if name == "CodeBuilder" && property == "new" {
                            self.uses_codebuilder = true;
                        }
                    }

                    // Check for custom prop method calls
                    if let DecoratedExprKind::Member { property: state_prop, .. } = &object.kind {
                        if state_prop == "state" && (property == "set_custom_prop" || property == "get_custom_prop" || property == "delete_custom_prop") {
                            self.uses_custom_props = true;
                            self.uses_hashmap = true; // Custom props use HashMap
                        }
                    }
                } else if let DecoratedExprKind::Ident { name, .. } = &call.callee.kind {
                    // Check for CodeBuilder::new() call (path-qualified calls become single ident)
                    if name == "CodeBuilder::new" || name == "CodeBuilder" {
                        self.uses_codebuilder = true;
                    }
                }

                self.detect_regex_usage_in_expr(&call.callee);
                for arg in &call.args {
                    self.detect_regex_usage_in_expr(arg);
                }
            }
            DecoratedExprKind::Binary { left, right, .. } => {
                self.detect_regex_usage_in_expr(left);
                self.detect_regex_usage_in_expr(right);
            }
            DecoratedExprKind::Member { object, .. } => {
                self.detect_regex_usage_in_expr(object);
            }
            DecoratedExprKind::If(if_expr) => {
                self.detect_regex_usage_in_expr(&if_expr.condition);
                self.detect_regex_usage_in_block(&if_expr.then_branch);
                if let Some(ref else_branch) = if_expr.else_branch {
                    self.detect_regex_usage_in_block(else_branch);
                }
            }
            DecoratedExprKind::Match(match_expr) => {
                self.detect_regex_usage_in_expr(&match_expr.expr);
                for arm in &match_expr.arms {
                    self.detect_regex_usage_in_block(&arm.body);
                }
            }
            _ => {}
        }
    }

    // ========================================================================
    // HEADER
    // ========================================================================

    fn emit_header(&mut self) {
        self.emit_line("// Generated by ReluxScript compiler");
        self.emit_line("// Do not edit manually");
        self.emit_line("// NOTE: SWC plugins require nightly Rust");
        self.emit_line("");
        self.emit_line("use swc_common::{Span, DUMMY_SP, SyntaxContext};");
        self.emit_line("use swc_ecma_ast::*;");
        self.emit_line("use swc_ecma_visit::{Visit, VisitMut, VisitMutWith, VisitWith};");

        // Add conditional imports
        if self.uses_hashmap && self.uses_hashset {
            self.emit_line("use std::collections::{HashMap, HashSet};");
        } else if self.uses_hashmap {
            self.emit_line("use std::collections::HashMap;");
        } else if self.uses_hashset {
            self.emit_line("use std::collections::HashSet;");
        }

        if self.uses_json {
            self.emit_line("use serde::{Serialize, Deserialize};");
            self.emit_line("use serde_json;");
        }

        if self.uses_fs {
            self.emit_line("use std::fs;");
            self.emit_line("use std::path::Path;");
        }

        if self.uses_parser {
            // Parser imports needed
            self.emit_line("use std::sync::Arc;");
            self.emit_line("use swc_common::{SourceMap, FileName};");
            self.emit_line("use swc_ecma_parser::{Parser, Syntax, TsConfig, EsConfig, StringInput};");
        }

        if self.uses_codegen {
            self.emit_line("use swc_common::SourceMap;");
            self.emit_line("use swc_ecma_codegen::{Emitter, text_writer::JsWriter, Config as CodegenConfig, Node};");
        }

        if self.uses_regex {
            self.emit_line("use regex::Regex as RegexPattern;");
        }

        self.emit_line("");
    }

    /// Emit user module imports (from use statements)
    fn emit_user_imports(&mut self, uses: &[crate::parser::UseStmt]) {
        // In lib.rs with multi-file output, first emit ALL module declarations
        if !self.is_module && !self.all_module_names.is_empty() {
            let mut sorted_names: Vec<_> = self.all_module_names.iter().cloned().collect();
            sorted_names.sort();
            for module_name in &sorted_names {
                // Handle Rust keywords - "mod" is a reserved keyword
                if module_name == "mod" {
                    self.emit_line("mod r#mod;");
                } else {
                    self.emit_line(&format!("mod {};", module_name));
                }
            }
            self.emit_line("");
        }

        if uses.is_empty() {
            return;
        }

        for use_stmt in uses {
            let is_file_module = use_stmt.path.starts_with("./") || use_stmt.path.starts_with("../");

            // Skip file module imports when modules are inlined (functions are in the same file)
            if is_file_module && self.skip_file_imports {
                continue;
            }

            if is_file_module {
                // File module: convert path to module name
                // e.g., "./helpers.lux" -> "helpers"
                // e.g., "../utils/types.lux" -> "types" (just use the filename)
                let module_name = self.extract_module_name_from_path(&use_stmt.path);

                // Handle Rust keywords
                let rust_module_name = if module_name == "mod" {
                    "r#mod".to_string()
                } else {
                    module_name.clone()
                };

                if self.is_module {
                    // In a module file, don't emit mod declarations - use crate:: paths
                    // The mod declarations are only in lib.rs
                    if !use_stmt.imports.is_empty() {
                        // Named imports: use crate::helpers::{get_component_name, escape_string};
                        let imports = use_stmt.imports.join(", ");
                        self.emit_line(&format!("use crate::{}::{{{}}};", rust_module_name, imports));
                    } else if let Some(alias) = &use_stmt.alias {
                        // Aliased import: use crate::helpers as h;
                        self.emit_line(&format!("use crate::{} as {};", rust_module_name, alias));
                    } else {
                        // Full import: use crate::helpers;
                        self.emit_line(&format!("use crate::{};", rust_module_name));
                    }
                } else {
                    // In lib.rs without all_module_names, emit mod declarations inline
                    if self.all_module_names.is_empty() {
                        self.emit_line(&format!("mod {};", rust_module_name));
                    }

                    // Emit use statement for imports
                    if !use_stmt.imports.is_empty() {
                        // Named imports: use helpers::{get_component_name, escape_string};
                        let imports = use_stmt.imports.join(", ");
                        self.emit_line(&format!("use {}::{{{}}};", rust_module_name, imports));
                    } else if let Some(alias) = &use_stmt.alias {
                        // Aliased import: use helpers as h;
                        self.emit_line(&format!("use {} as {};", rust_module_name, alias));
                    } else {
                        // Full import: use helpers;
                        self.emit_line(&format!("use {};", rust_module_name));
                    }
                }
            }
            // Skip built-in modules - they're handled by detect_imports
        }

        self.emit_line("");
    }

    /// Extract module name from file path
    fn extract_module_name_from_path(&self, path: &str) -> String {
        // Remove .lux or .rsc extension
        let path = path.replace(".lux", "").replace(".rsc", "");

        // Extract just the filename from the path
        // e.g., "./helpers" -> "helpers"
        // e.g., "../utils/types" -> "types"
        path.split('/').last().unwrap_or(&path).to_string()
    }

    // ========================================================================
    // TOP-LEVEL DECLARATIONS
    // ========================================================================

    fn emit_top_level_decl(&mut self, decl: &DecoratedTopLevelDecl) {
        match decl {
            DecoratedTopLevelDecl::Plugin(plugin) => {
                self.is_writer = false;
                self.emit_plugin(plugin);
            }
            DecoratedTopLevelDecl::Writer(writer) => {
                self.is_writer = true;
                self.emit_writer(writer);
            }
            DecoratedTopLevelDecl::Module(module) => {
                self.is_writer = false;
                self.emit_module(module);
            }
            DecoratedTopLevelDecl::Undecorated(top_level) => {
                self.emit_undecorated(top_level);
            }
        }
    }

    /// Emit a decorated module (standalone file with functions/structs/enums)
    fn emit_module(&mut self, module: &DecoratedModule) {
        for item in &module.items {
            self.emit_plugin_item(item);
        }
    }

    /// Emit an undecorated top-level declaration (Module)
    fn emit_undecorated(&mut self, decl: &crate::parser::TopLevelDecl) {
        use crate::parser::{TopLevelDecl, PluginItem};

        match decl {
            TopLevelDecl::Module(module) => {
                // Emit each item in the module
                for item in &module.items {
                    self.emit_module_item(item);
                }
            }
            _ => {
                self.emit_line("// Undecorated top-level declaration (not yet supported)");
            }
        }
    }

    /// Emit a module item (for Module top-level declarations)
    fn emit_module_item(&mut self, item: &crate::parser::PluginItem) {
        use crate::parser::{PluginItem, EnumVariantFields};

        match item {
            PluginItem::Function(func) => {
                // Emit as pub fn for module exports
                self.emit_module_function(func);
            }
            PluginItem::Struct(struct_decl) => {
                // Emit struct with pub visibility
                self.emit_line("#[derive(Clone, Debug)]");
                self.emit_line(&format!("pub struct {} {{", struct_decl.name));
                self.indent += 1;
                for field in &struct_decl.fields {
                    let rust_type = self.type_to_string(&field.ty);
                    self.emit_line(&format!("pub {}: {},", field.name, rust_type));
                }
                self.indent -= 1;
                self.emit_line("}");
                self.emit_line("");
            }
            PluginItem::Enum(enum_decl) => {
                // Emit enum with pub visibility
                self.emit_line("#[derive(Clone, Debug)]");
                self.emit_line(&format!("pub enum {} {{", enum_decl.name));
                self.indent += 1;
                for variant in &enum_decl.variants {
                    match &variant.fields {
                        EnumVariantFields::Tuple(types) => {
                            let type_strs: Vec<_> = types.iter().map(|t| self.type_to_string(t)).collect();
                            self.emit_line(&format!("{}({}),", variant.name, type_strs.join(", ")));
                        }
                        EnumVariantFields::Struct(fields) => {
                            self.emit_line(&format!("{} {{", variant.name));
                            self.indent += 1;
                            for (name, ty) in fields {
                                let rust_type = self.type_to_string(ty);
                                self.emit_line(&format!("{}: {},", name, rust_type));
                            }
                            self.indent -= 1;
                            self.emit_line("},");
                        }
                        EnumVariantFields::Unit => {
                            self.emit_line(&format!("{},", variant.name));
                        }
                    }
                }
                self.indent -= 1;
                self.emit_line("}");
                self.emit_line("");
            }
            _ => {
                // Skip other items for now
            }
        }
    }

    /// Emit a function from a module (with pub visibility)
    fn emit_module_function(&mut self, func: &crate::parser::FnDecl) {
        // Build function signature
        let mut params = Vec::new();
        for param in &func.params {
            let rust_type = self.type_to_string(&param.ty);
            params.push(format!("{}: {}", param.name, rust_type));
        }

        let return_type = if let Some(ref ret_ty) = func.return_type {
            format!(" -> {}", self.type_to_string(ret_ty))
        } else {
            String::new()
        };

        // Emit function header with pub
        let pub_kw = if func.is_pub { "pub " } else { "" };
        self.emit_line(&format!("{}fn {}({}){} {{", pub_kw, func.name, params.join(", "), return_type));
        self.indent += 1;

        // Emit function body - for now just emit a todo!() since we can't easily
        // emit decorated function bodies from undecorated AST
        self.emit_line("todo!(\"Module function body not yet implemented\")");

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    fn emit_plugin(&mut self, plugin: &DecoratedPlugin) {
        self.name = plugin.name.clone();

        // Check if there's a State struct
        let has_state = plugin.body.iter().any(|item| {
            if let DecoratedPluginItem::Struct(s) = item {
                s.name == "State"
            } else {
                false
            }
        });

        // If custom props are used, emit the CustomPropValue enum first
        if self.uses_custom_props && has_state {
            self.emit_custom_prop_value_enum();
        }

        // Emit structs, enums, and impl blocks FIRST (at module level)
        for item in &plugin.body {
            match item {
                DecoratedPluginItem::Struct(_) |
                DecoratedPluginItem::Enum(_) |
                DecoratedPluginItem::Impl(_) => {
                    self.emit_plugin_item(item);
                }
                _ => {}
            }
        }

        // Plugin struct
        self.emit_line(&format!("pub struct {} {{", plugin.name));
        self.indent += 1;
        if has_state {
            self.emit_line("pub state: State,");
        }
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // Impl VisitMut (only visitor methods)
        self.emit_line(&format!("impl VisitMut for {} {{", plugin.name));
        self.indent += 1;

        // Emit only visitor methods (visit_*)
        for item in &plugin.body {
            if let DecoratedPluginItem::Function(func) = item {
                if func.name.starts_with("visit_") {
                    self.emit_plugin_item(item);
                }
            }
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // Emit impl block with constructor and helper functions
        self.emit_line(&format!("impl {} {{", plugin.name));
        self.indent += 1;

        // Emit constructor if has state
        if has_state {
            // Get the State struct to initialize fields
            let state_struct = plugin.body.iter().find_map(|item| {
                if let DecoratedPluginItem::Struct(s) = item {
                    if s.name == "State" {
                        return Some(s);
                    }
                }
                None
            });

            if let Some(state) = state_struct {
                self.emit_line("pub fn new() -> Self {");
                self.indent += 1;
                self.emit_line("Self {");
                self.indent += 1;
                self.emit_line("state: State {");
                self.indent += 1;

                // Initialize state fields with default values
                for field in &state.fields {
                    let default_value = self.get_default_value_for_type(&field.ty);
                    self.emit_line(&format!("{}: {},", field.name, default_value));
                }

                // Initialize __custom_props if used
                if self.uses_custom_props {
                    self.emit_line("__custom_props: std::collections::HashMap::new(),");
                }

                self.indent -= 1;
                self.emit_line("},");
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("}");
                self.emit_line("");
            }
        } else {
            // Emit simple constructor for plugins without state
            self.emit_line("pub fn new() -> Self {");
            self.indent += 1;
            self.emit_line("Self {}");
            self.indent -= 1;
            self.emit_line("}");
            self.emit_line("");
        }

        // Emit helper functions inside impl block
        for item in &plugin.body {
            if let DecoratedPluginItem::Function(func) = item {
                if !func.name.starts_with("visit_") {
                    self.emit_plugin_item(item);
                }
            }
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    fn emit_writer(&mut self, writer: &DecoratedWriter) {
        self.name = writer.name.clone();
        self.is_writer = true;

        // 1. Emit hoisted structs and impl blocks at module level (before the writer struct)
        for struct_decl in &writer.hoisted_structs {
            self.emit_struct(struct_decl);
        }

        // Emit hoisted impl blocks from body at module level
        for item in &writer.body {
            if let DecoratedPluginItem::Impl(impl_block) = item {
                self.emit_impl_block(impl_block);
            }
        }

        // 2. Emit the writer struct with output field + flattened State fields
        self.emit_line(&format!("pub struct {} {{", writer.name));
        self.indent += 1;
        self.emit_line("output: String,");
        self.emit_line("indent_level: usize,");

        // Flatten State struct fields into main struct
        if let Some(ref state) = writer.state_struct {
            for field in &state.fields {
                let type_str = self.type_to_string(&field.ty);
                self.emit_line(&format!("{}: {},", field.name, type_str));
            }
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // 3. Empty Visit implementation (writers use immutable Visit, not VisitMut)
        self.emit_line(&format!("impl Visit for {} {{}}", writer.name));
        self.emit_line("");

        // 4. Impl block with new(), CodeBuilder methods, and user methods
        self.emit_line(&format!("impl {} {{", writer.name));
        self.indent += 1;

        // Generate new() constructor
        self.emit_writer_constructor(&writer.state_struct);

        // Generate CodeBuilder helper methods
        self.emit_codebuilder_methods();

        // Emit user-defined methods (skip Impl blocks as they were emitted at module level)
        for item in &writer.body {
            if !matches!(item, DecoratedPluginItem::Impl(_)) {
                self.emit_plugin_item(item);
            }
        }

        self.indent -= 1;
        self.emit_line("}");
    }

    fn emit_plugin_item(&mut self, item: &DecoratedPluginItem) {
        match item {
            DecoratedPluginItem::Function(func) => {
                self.emit_function(func);
            }
            DecoratedPluginItem::Struct(struct_decl) => {
                self.emit_struct(struct_decl);
            }
            DecoratedPluginItem::Enum(enum_decl) => {
                self.emit_enum(enum_decl);
            }
            DecoratedPluginItem::Impl(impl_block) => {
                self.emit_impl_block(impl_block);
            }
            DecoratedPluginItem::PreHook(func) => {
                self.emit_comment("Pre-hook");
                self.emit_function(func);
            }
            DecoratedPluginItem::ExitHook(func) => {
                self.emit_comment("Exit-hook");
                self.emit_function_with_visibility(func, true);
            }
            DecoratedPluginItem::Static(static_decl) => {
                self.emit_static(static_decl);
            }
            DecoratedPluginItem::PubUse(use_stmt) => {
                // Skip file module re-exports when modules are inlined
                let is_file_module = use_stmt.path.starts_with("./") || use_stmt.path.starts_with("../");
                if is_file_module && self.skip_file_imports {
                    // Skip - functions are already available from inlined code
                    return;
                }

                // Emit pub use as Rust re-export
                self.emit_indent();
                self.output.push_str("pub use ");
                self.output.push_str(&use_stmt.path);
                if !use_stmt.imports.is_empty() {
                    self.output.push_str("::{");
                    self.output.push_str(&use_stmt.imports.join(", "));
                    self.output.push_str("}");
                }
                self.output.push_str(";\n");
            }
        }
    }

    fn emit_static(&mut self, static_decl: &super::swc_decorator::DecoratedStaticDecl) {
        self.emit_indent();
        if static_decl.is_mut {
            self.output.push_str("static mut ");
        } else {
            self.output.push_str("static ");
        }
        self.output.push_str(&static_decl.name);
        self.output.push_str(": ");
        let type_str = self.type_to_string_with_lifetime(&static_decl.ty, false);
        self.output.push_str(&type_str);
        self.output.push_str(" = ");
        self.emit_expr(&static_decl.init);
        self.output.push_str(";\n");
    }

    // ========================================================================
    // STRUCTURES
    // ========================================================================

    fn emit_struct(&mut self, struct_decl: &StructDecl) {
        // Emit derives if any, or default to Clone + Debug for SWC
        if !struct_decl.derives.is_empty() {
            self.emit_line(&format!("#[derive({})]", struct_decl.derives.join(", ")));
        } else {
            // Default derives for user structs in SWC
            // Don't derive Clone if struct has mutable reference fields (can't be cloned)
            let has_mut_refs = struct_decl.fields.iter().any(|f| {
                matches!(f.ty, Type::Reference { mutable: true, .. })
            });
            if has_mut_refs {
                self.emit_line("#[derive(Debug)]");
            } else {
                self.emit_line("#[derive(Clone, Debug)]");
            }
        }

        // Emit struct with optional lifetime parameters
        let lifetimes_str = if !struct_decl.lifetimes.is_empty() {
            format!("<{}>", struct_decl.lifetimes.join(", "))
        } else {
            String::new()
        };
        self.emit_line(&format!("struct {}{} {{", struct_decl.name, lifetimes_str));
        self.indent += 1;

        // If struct has lifetimes, add lifetime annotations to reference types
        let has_lifetimes = !struct_decl.lifetimes.is_empty();
        let has_serialize = struct_decl.derives.iter().any(|d| d == "Serialize");

        for field in &struct_decl.fields {
            // If field contains AST types and struct derives Serialize, skip serialization only
            // (not deserialization, so we don't need Default implementation)
            if has_serialize && self.contains_ast_type(&field.ty) {
                self.emit_line("#[serde(skip_serializing)]");
            }

            let type_str = self.type_to_string_with_lifetime(&field.ty, has_lifetimes);
            self.emit_line(&format!("{}: {},", field.name, type_str));
        }

        // If this is the State struct and custom props are used, inject the __custom_props field
        if struct_decl.name == "State" && self.uses_custom_props {
            self.emit_line("// Auto-generated: Custom AST property storage");
            self.emit_line("__custom_props: std::collections::HashMap<usize, std::collections::HashMap<String, CustomPropValue>>,");
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // If this is the State struct and custom props are used, emit helper methods
        if struct_decl.name == "State" && self.uses_custom_props {
            self.emit_custom_prop_helpers();
        }
    }

    fn emit_enum(&mut self, enum_decl: &EnumDecl) {
        self.emit_line(&format!("enum {} {{", enum_decl.name));
        self.indent += 1;

        for variant in &enum_decl.variants {
            match &variant.fields {
                EnumVariantFields::Unit => {
                    self.emit_line(&format!("{},", variant.name));
                }
                EnumVariantFields::Tuple(types) => {
                    let type_strs: Vec<String> = types.iter()
                        .map(|ty| self.type_to_string(ty))
                        .collect();
                    self.emit_line(&format!("{}({}),", variant.name, type_strs.join(", ")));
                }
                EnumVariantFields::Struct(fields) => {
                    self.emit_line(&format!("{} {{", variant.name));
                    self.indent += 1;
                    for (field_name, field_type) in fields {
                        let type_str = self.type_to_string(field_type);
                        self.emit_line(&format!("{}: {},", field_name, type_str));
                    }
                    self.indent -= 1;
                    self.emit_line("},");
                }
            }
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    fn emit_impl_block(&mut self, impl_block: &DecoratedImplBlock) {
        // Emit impl with optional lifetime parameters
        let lifetimes_str = if !impl_block.lifetimes.is_empty() {
            format!("<{}>", impl_block.lifetimes.join(", "))
        } else {
            String::new()
        };
        self.emit_line(&format!("impl{} {} {{", lifetimes_str, impl_block.target));
        self.indent += 1;

        for method in &impl_block.items {
            self.emit_function(method);
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    // ========================================================================
    // FUNCTIONS
    // ========================================================================

    fn emit_function(&mut self, func: &DecoratedFnDecl) {
        self.emit_function_with_visibility(func, func.is_pub);
    }

    fn emit_function_with_visibility(&mut self, func: &DecoratedFnDecl, is_public: bool) {
        // Function signature
        let visibility = if is_public { "pub " } else { "" };

        // Transform visit_X -> visit_mut_X for VisitMut trait methods
        let method_name = if func.name.starts_with("visit_") && !func.name.starts_with("visit_mut_") {
            func.name.replace("visit_", "visit_mut_")
        } else {
            func.name.clone()
        };

        let mut sig = format!("{}fn {}", visibility, method_name);

        // Check if we need lifetime parameter
        let needs_lifetime = func.return_type.as_ref()
            .map(|ty| self.type_has_reference(ty))
            .unwrap_or(false);

        // Add lifetime parameter if needed
        if needs_lifetime {
            sig.push_str("<'a>");
        }

        // Parameters
        sig.push('(');

        // Check if first parameter is already a self parameter
        let first_is_self = func.params.first()
            .map(|p| p.name == "self")
            .unwrap_or(false);

        // Only add &mut self if:
        // 1. It's a visitor method (visit_*), OR
        // 2. The function already has self as first parameter in source
        let needs_self = func.name.starts_with("visit_") || first_is_self;

        if needs_self && !first_is_self {
            sig.push_str("&mut self");
            if !func.params.is_empty() {
                sig.push_str(", ");
            }
        }

        let mut emitted_self = false;
        for (i, param) in func.params.iter().enumerate() {
            // Skip first parameter if it's self and we already added &mut self
            if needs_self && first_is_self && i == 0 {
                // Replace self parameter with &mut self
                sig.push_str("&mut self");
                emitted_self = true;
                continue;
            }

            // Add comma before parameter if needed
            if emitted_self || i > 0 {
                sig.push_str(", ");
            }
            sig.push_str(&param.name);
            sig.push_str(": ");

            // For visitor methods in plugins (not writers), make references mutable
            let param_type_str = if needs_self && !self.is_writer {
                // This is a visitor method in a plugin - need &mut references
                self.make_reference_mutable(&param.ty, needs_lifetime)
            } else {
                // Writer or non-visitor method - use type as-is
                self.type_to_string_with_lifetime(&param.ty, needs_lifetime)
            };
            sig.push_str(&param_type_str);
        }
        sig.push(')');

        // Return type
        if let Some(ref ret_ty) = func.return_type {
            sig.push_str(" -> ");
            sig.push_str(&self.type_to_string_with_lifetime(ret_ty, needs_lifetime));
        }

        sig.push_str(" {");
        self.emit_line(&sig);

        // Function body
        // If function has no return type or returns (), all statements need semicolons
        let force_semicolons = func.return_type.is_none();
        self.indent += 1;
        self.emit_block_with_context(&func.body, force_semicolons);
        self.indent -= 1;

        self.emit_line("}");
        self.emit_line("");
    }

    // ========================================================================
    // BLOCKS AND STATEMENTS
    // ========================================================================

    fn emit_block(&mut self, block: &DecoratedBlock) {
        self.emit_block_with_context(block, false);
    }

    fn emit_block_with_context(&mut self, block: &DecoratedBlock, force_semicolons: bool) {
        let len = block.stmts.len();
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i == len - 1;
            self.emit_stmt_with_context(stmt, is_last, force_semicolons);
        }
    }

    fn emit_stmt_with_context(&mut self, stmt: &DecoratedStmt, is_last_in_block: bool, force_semicolons: bool) {
        // If it's the last statement in a block and it's an expression,
        // don't add a semicolon UNLESS force_semicolons is true (e.g., function returns ())
        if is_last_in_block && !force_semicolons {
            if let DecoratedStmt::Expr(expr) = stmt {
                self.emit_indent();
                self.emit_expr(expr);
                self.output.push('\n');
                return;
            }
        }

        // Otherwise, emit normally
        self.emit_stmt(stmt);
    }

    fn emit_stmt(&mut self, stmt: &DecoratedStmt) {
        match stmt {
            DecoratedStmt::Let(let_stmt) => {
                self.emit_indent();
                if let_stmt.mutable {
                    self.output.push_str("let mut ");
                } else {
                    self.output.push_str("let ");
                }
                self.emit_pattern(&let_stmt.pattern);
                if let Some(ref init) = let_stmt.init {
                    self.output.push_str(" = ");
                    self.emit_expr(init);
                }
                self.output.push_str(";\n");
            }

            DecoratedStmt::Const(const_stmt) => {
                self.emit_indent();
                self.output.push_str("const ");
                self.output.push_str(&const_stmt.name);
                if let Some(ref ty) = const_stmt.ty {
                    self.output.push_str(": ");
                    self.output.push_str(&self.type_to_string(ty));
                }
                self.output.push_str(" = ");
                self.emit_expr(&const_stmt.init);
                self.output.push_str(";\n");
            }

            DecoratedStmt::Expr(expr) => {
                self.emit_indent();
                self.emit_expr(expr);
                self.output.push_str(";\n");
            }

            DecoratedStmt::If(if_stmt) => {
                self.emit_if_stmt(if_stmt);
            }

            DecoratedStmt::Match(match_stmt) => {
                self.emit_match_stmt(match_stmt);
            }

            DecoratedStmt::For(for_stmt) => {
                self.emit_indent();
                self.output.push_str("for ");
                self.emit_pattern(&for_stmt.pattern);
                self.output.push_str(" in ");
                self.emit_expr(&for_stmt.iter);
                self.output.push_str(" {\n");
                self.indent += 1;
                self.emit_block_with_context(&for_stmt.body, true); // Force semicolons in for loop body
                self.indent -= 1;
                self.emit_line("}");
            }

            DecoratedStmt::While(while_stmt) => {
                self.emit_indent();
                self.output.push_str("while ");
                self.emit_expr(&while_stmt.condition);
                self.output.push_str(" {\n");
                self.indent += 1;
                self.emit_block_with_context(&while_stmt.body, true); // Force semicolons in while loop body
                self.indent -= 1;
                self.emit_line("}");
            }

            DecoratedStmt::Loop(loop_body) => {
                self.emit_line("loop {");
                self.indent += 1;
                self.emit_block_with_context(loop_body, true); // Force semicolons in loop body
                self.indent -= 1;
                self.emit_line("}");
            }

            DecoratedStmt::Return(ret_expr) => {
                self.emit_indent();
                self.output.push_str("return");
                if let Some(ref expr) = ret_expr {
                    self.output.push(' ');
                    self.emit_expr(expr);
                }
                self.output.push_str(";\n");
            }

            DecoratedStmt::Break => {
                self.emit_line("break;");
            }

            DecoratedStmt::Continue => {
                self.emit_line("continue;");
            }

            DecoratedStmt::Traverse(traverse) => {
                self.emit_traverse_stmt(traverse);
            }

            DecoratedStmt::Function(func_decl) => {
                self.emit_line(&format!("// Nested function: {}", func_decl.name));
            }

            DecoratedStmt::Verbatim(verbatim) => {
                self.emit_line(&verbatim.code);
            }

            DecoratedStmt::CustomPropAssignment(assign) => {
                // This should have been transformed by the rewriter into a Call expression
                // If we reach here, the rewriter didn't run
                panic!("CustomPropAssignment should have been rewritten by SwcRewriter");
            }

            DecoratedStmt::Unsafe(unsafe_block) => {
                // Emit unsafe block - wraps statements in an unsafe block
                self.emit_line("unsafe {");
                self.indent += 1;
                for stmt in &unsafe_block.stmts {
                    self.emit_stmt(stmt);
                }
                self.indent -= 1;
                self.emit_line("}");
            }

        }
    }

    fn emit_if_stmt(&mut self, if_stmt: &DecoratedIfStmt) {
        if let Some(ref pattern) = if_stmt.pattern {
            // Emit ALL if-let patterns as match statements
            // This provides proper type narrowing in the generated code
            eprintln!("[EMIT] if-let pattern detected: {:?}, emitting as match", pattern.kind);
            self.emit_if_let_as_match(if_stmt, pattern);
            return;
        }

        // Regular if statement (no pattern)
        self.emit_indent();
        self.output.push_str("if ");
        self.emit_expr(&if_stmt.condition);
        self.output.push_str(" {\n");

        // Then branch
        self.indent += 1;
        self.emit_block(&if_stmt.then_branch);
        self.indent -= 1;

        // Else branch
        if let Some(ref else_branch) = if_stmt.else_branch {
            self.emit_line("} else {");
            self.indent += 1;
            self.emit_block(else_branch);
            self.indent -= 1;
            self.emit_line("}");
        } else {
            self.emit_line("}");
        }
    }

    /// Check if a pattern contains :: (path qualifier)
    fn is_path_qualified_pattern(&self, pattern: &DecoratedPattern) -> bool {
        // Check the swc_pattern in metadata, not the name in kind
        // The kind name is the original ReluxScript name ("ObjectPattern")
        // The metadata swc_pattern is the mapped SWC name ("Pat::Object")
        let is_qualified = pattern.metadata.swc_pattern.contains("::");
        if is_qualified {
            eprintln!("[EMIT PATH QUALIFIED] Pattern: {}", pattern.metadata.swc_pattern);
        }
        is_qualified
    }

    /// Emit if-let with path-qualified pattern as match statement
    /// if let Pat::Object(x) = expr { body } else { else_body }
    /// becomes:
    /// match expr { Pat::Object(x) => { body }, _ => { else_body } }
    fn emit_if_let_as_match(&mut self, if_stmt: &DecoratedIfStmt, pattern: &DecoratedPattern) {
        self.emit_indent();
        self.output.push_str("match ");

        // If the condition is a Box<T>, we need to dereference it
        // Use &* instead of .as_ref() to avoid double-calling as_ref()
        if if_stmt.condition.metadata.is_boxed {
            eprintln!("[EMIT IF-LET MATCH] Condition is boxed, emitting &* prefix");
            self.output.push_str("&*");
        }
        self.emit_expr(&if_stmt.condition);

        self.output.push_str(" {\n");
        self.indent += 1;

        // Match arm for the pattern
        self.emit_indent();
        self.emit_pattern(pattern);
        self.output.push_str(" => {\n");
        self.indent += 1;

        // Special case: if pattern is a guard pattern for Option<ExprOrSpread> with Expr variant,
        // insert nested if-let to match the Expr enum
        eprintln!("[EMIT MATCH ARM] pattern.swc_pattern='{}', condition.swc_type='{}'",
            pattern.metadata.swc_pattern, if_stmt.condition.metadata.swc_type);

        let needs_nested_expr_match = pattern.metadata.swc_pattern.contains("if s.spread.is_none()")
            && if_stmt.condition.metadata.swc_type.contains("Option<ExprOrSpread");

        if needs_nested_expr_match {
            eprintln!("[EMIT MATCH ARM] Inserting nested if-let for Expr::Ident match");
            self.emit_line("if let Expr::Ident(elem) = s.expr.as_ref() {");
            self.indent += 1;
        }

        self.emit_block(&if_stmt.then_branch);

        if needs_nested_expr_match {
            self.indent -= 1;
            self.emit_line("}");
        }

        self.indent -= 1;
        self.emit_line("}");

        // Wildcard arm for else branch (or empty block if no else)
        self.emit_indent();
        self.output.push_str("_ => ");
        if let Some(ref else_branch) = if_stmt.else_branch {
            self.output.push_str("{\n");
            self.indent += 1;
            self.emit_block(else_branch);
            self.indent -= 1;
            self.emit_line("}");
        } else {
            self.output.push_str("{}\n");
        }

        self.indent -= 1;
        self.emit_line("}");
    }

    fn emit_match_stmt(&mut self, match_stmt: &DecoratedMatchStmt) {
        self.emit_indent();
        self.output.push_str("match ");

        // If the scrutinee is a Box<T>, we need to dereference it with &*
        // This allows matching against the inner type
        eprintln!("[EMIT MATCH] scrutinee type='{}', is_boxed={}", match_stmt.expr.metadata.swc_type, match_stmt.expr.metadata.is_boxed);
        if match_stmt.expr.metadata.is_boxed {
            eprintln!("[EMIT MATCH] Emitting &* for boxed scrutinee");
            self.output.push_str("&*");
        }

        self.emit_expr(&match_stmt.expr);
        self.output.push_str(" {\n");

        self.indent += 1;

        for arm in &match_stmt.arms {
            self.emit_indent();
            self.emit_pattern(&arm.pattern);

            if let Some(ref guard) = arm.guard {
                self.output.push_str(" if ");
                self.emit_expr(guard);
            }

            self.output.push_str(" => {\n");
            self.indent += 1;
            self.emit_block(&arm.body);
            self.indent -= 1;
            self.emit_line("}");
        }

        self.indent -= 1;
        self.emit_line("}");
    }

    // ========================================================================
    // PATTERNS
    // ========================================================================

    fn emit_pattern(&mut self, pattern: &DecoratedPattern) {
        // Use the swc_pattern from metadata - it's already been mapped!
        match &pattern.kind {
            DecoratedPatternKind::Literal(lit) => {
                self.emit_literal(lit);
            }

            DecoratedPatternKind::Ident(name) => {
                self.output.push_str(name);
            }

            DecoratedPatternKind::Wildcard => {
                self.output.push('_');
            }

            DecoratedPatternKind::Variant { name: _, inner } => {
                // The swc_pattern metadata already contains the full pattern
                // For example: "Callee::Expr(__callee_expr)" or "Expr::Ident"
                // We only need the base pattern name, not the inner binding

                // Strip "UserDefined::" prefix if present (it's a marker, not a real type)
                let swc_pattern = if pattern.metadata.swc_pattern.starts_with("UserDefined::") {
                    pattern.metadata.swc_pattern.strip_prefix("UserDefined::").unwrap()
                } else if pattern.metadata.swc_pattern.starts_with("Unknown::") {
                    pattern.metadata.swc_pattern.strip_prefix("Unknown::").unwrap()
                } else {
                    &pattern.metadata.swc_pattern
                };

                // Check if the metadata contains parentheses (meaning it has a binding)
                if swc_pattern.contains('(') {
                    // It already has the binding, use it as-is
                    self.output.push_str(swc_pattern);
                } else if let Some(ref inner_pattern) = inner {
                    // No binding in metadata, emit pattern with inner
                    self.output.push_str(swc_pattern);
                    self.output.push('(');
                    self.emit_pattern(inner_pattern);
                    self.output.push(')');
                } else {
                    // No inner pattern - emit with wildcard for tuple variants
                    // For example: Some -> Some(_), None -> None (no wildcard needed)
                    self.output.push_str(swc_pattern);
                    // Check if this is a tuple variant (like Some, Ok, Err) that needs a wildcard
                    let needs_wildcard = matches!(
                        swc_pattern,
                        "Some" | "Ok" | "Err"
                    );
                    if needs_wildcard {
                        self.output.push_str("(_)");
                    }
                }
            }

            DecoratedPatternKind::Tuple(patterns) => {
                self.output.push('(');
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_pattern(pat);
                }
                self.output.push(')');
            }

            DecoratedPatternKind::Struct { name, fields } => {
                self.output.push_str(name);
                self.output.push_str(" { ");
                if fields.is_empty() {
                    // Empty fields = wildcard struct pattern
                    self.output.push_str("..");
                } else {
                    for (i, (field_name, field_pat)) in fields.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.output.push_str(field_name);
                        self.output.push_str(": ");
                        self.emit_pattern(field_pat);
                    }
                }
                self.output.push_str(" }");
            }

            DecoratedPatternKind::Array(patterns) => {
                self.output.push('[');
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_pattern(pat);
                }
                self.output.push(']');
            }

            DecoratedPatternKind::Object(_) => {
                self.output.push_str("/* object pattern */");
            }

            DecoratedPatternKind::Rest(inner) => {
                self.output.push_str("..");
                self.emit_pattern(inner);
            }

            DecoratedPatternKind::Or(patterns) => {
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.emit_pattern(pat);
                }
            }

            DecoratedPatternKind::Ref { is_mut, pattern: inner } => {
                self.output.push('&');
                if *is_mut {
                    self.output.push_str("mut ");
                }
                self.emit_pattern(inner);
            }
        }
    }

    // ========================================================================
    // EXPRESSIONS
    // ========================================================================

    fn emit_expr(&mut self, expr: &DecoratedExpr) {
        match &expr.kind {
            DecoratedExprKind::Literal(lit) => {
                self.emit_literal(lit);

                // String literals need .to_string() when used as owned String values
                // This happens in contexts like Err("str"), Ok("str"), Some("str"), etc.
                if matches!(lit, Literal::String(_)) && expr.metadata.swc_type == "String" {
                    self.output.push_str(".to_string()");
                }
            }

            DecoratedExprKind::Ident { name, ident_metadata } => {
                // Special handling for CLOSURE_UNWRAPPER marker
                if let Some(unwrapper) = name.strip_prefix("CLOSURE_UNWRAPPER:") {
                    // Emit the closure directly: |v| unwrapper_pattern
                    self.output.push_str("|v| ");
                    self.output.push_str(unwrapper);
                    return;
                }

                // Check for deref pattern
                if let Some(ref deref) = ident_metadata.deref_pattern {
                    self.output.push_str(deref);
                }

                // Transform built-in module names
                if name == "json" {
                    self.output.push_str("serde_json");
                } else {
                    self.output.push_str(name);
                }

                // Check if we need .sym
                if ident_metadata.use_sym {
                    self.output.push_str(".sym");
                }

                // Check if we need Option unwrap (from narrowing)
                if let Some((parent_enum, variant_name)) = &expr.metadata.needs_enum_unwrap {
                    if parent_enum == "Option" && variant_name == "unwrap" {
                        self.output.push_str(".as_ref().unwrap()");
                    }
                }
            }

            DecoratedExprKind::Binary { left, op, right, binary_metadata } => {
                self.output.push('(');

                // Left side - check for sym deref
                if binary_metadata.left_needs_deref {
                    self.output.push_str("&*");
                }
                self.emit_expr(left);

                // Operator
                self.output.push(' ');
                self.output.push_str(&self.binary_op_to_string(op));
                self.output.push(' ');

                // Right side - check for sym deref
                if binary_metadata.right_needs_deref {
                    self.output.push_str("&*");
                }
                self.emit_expr(right);

                self.output.push(')');
            }

            DecoratedExprKind::Unary { op, operand, unary_metadata } => {
                // Special handling for & on Box fields
                if matches!(op, crate::parser::UnaryOp::Ref) {
                    // Check if operand is a known Box field
                    let is_known_box_field = if let DecoratedExprKind::Member { field_metadata, .. } = &operand.kind {
                        field_metadata.field_type.starts_with("Box<") ||
                        field_metadata.swc_field_name == "obj" ||
                        field_metadata.swc_field_name == "expr"
                    } else {
                        false
                    };

                    if is_known_box_field {
                        // Emit &* for Box fields
                        self.output.push_str("&*");
                        self.emit_expr(operand);
                        return;
                    }
                }

                // Normal unary operator
                if let Some(ref override_op) = unary_metadata.override_op {
                    self.output.push_str(override_op);
                } else {
                    self.output.push_str(&self.unary_op_to_string(op));
                }
                self.emit_expr(operand);
            }

            DecoratedExprKind::Member { object, property: _, optional, computed: _, is_path, field_metadata } => {
                eprintln!("[EMIT MEMBER] field={}, accessor={:?}, object_type={}",
                    field_metadata.swc_field_name, field_metadata.accessor, object.metadata.swc_type);

                // Special case: if object has read_conversion that unwraps to Expr enum,
                // and we're accessing .sym, generate a match expression
                let needs_enum_match = if let DecoratedExprKind::Member { field_metadata: obj_meta, .. } = &object.kind {
                    obj_meta.read_conversion == ".as_expr().unwrap()" && field_metadata.swc_field_name == "sym"
                } else {
                    false
                };

                if needs_enum_match {
                    // Generate: match obj.as_ref() { Expr::Ident(i) => i.sym, _ => todo!() }
                    self.output.push_str("match ");
                    self.emit_expr(object);
                    self.output.push_str(".as_ref() { Expr::Ident(i) => i.sym");
                    // Apply the read_conversion for sym (.to_string())
                    if !field_metadata.read_conversion.is_empty() {
                        self.output.push_str(&field_metadata.read_conversion);
                    }
                    self.output.push_str(", _ => \"\".into() }");
                    return;
                }

                // Special case: PropName enum accessing .sym
                if object.metadata.swc_type == "PropName" && field_metadata.swc_field_name == "sym" {
                    eprintln!("[EMIT MEMBER] Generating PropName match for .sym access");
                    self.output.push_str("match &");
                    self.emit_expr(object);
                    self.output.push_str(" { PropName::Ident(ident) => ident.sym");
                    // Apply the read_conversion for sym (.to_string())
                    if !field_metadata.read_conversion.is_empty() {
                        self.output.push_str(&field_metadata.read_conversion);
                    }
                    self.output.push_str(", _ => \"\".into() }");
                    return;
                }

                // Special case: Option<Pat> accessing .sym (from .name field in ReluxScript)
                // This needs unwrap + Pat::Ident destructure + .id.sym access
                eprintln!("[EMIT MEMBER CHECK] object_type='{}', swc_field_name='{}', contains Option<Pat>={}",
                          object.metadata.swc_type, field_metadata.swc_field_name, object.metadata.swc_type.contains("Option<Pat>"));
                if object.metadata.swc_type.contains("Option<Pat>") && field_metadata.swc_field_name == "sym" {
                    eprintln!("[EMIT MEMBER] Generating Option<Pat> unwrap + destructure for .sym access");
                    // Check if this is an indexed access (arr.elems[0])
                    // Generate: ({ let Pat::Ident(ident) = &obj.clone().unwrap() else { return; }; ident.id.sym })
                    self.output.push_str("({ let Pat::Ident(__pat_ident) = &");
                    self.emit_expr(object);
                    self.output.push_str(".clone().unwrap() else { return; }; __pat_ident.id.sym");
                    // Apply the read_conversion for sym (.to_string())
                    if !field_metadata.read_conversion.is_empty() {
                        self.output.push_str(&field_metadata.read_conversion);
                    }
                    self.output.push_str(" })");
                    return;
                }

                // Emit prefix for Utf8Lossy accessor
                if let FieldAccessor::Utf8Lossy = field_metadata.accessor {
                    eprintln!("[EMIT UTF8LOSSY] Emitting wrapper prefix");
                    self.output.push_str("String::from_utf8_lossy(");
                }

                self.emit_expr(object);

                if *optional {
                    self.output.push('?');
                }

                // Use :: for path expressions (module::function), . for field access
                if *is_path {
                    self.output.push_str("::");
                } else {
                    self.output.push('.');
                }

                // Use the SWC field name from metadata!
                self.output.push_str(&field_metadata.swc_field_name);

                // Apply accessor strategy
                match &field_metadata.accessor {
                    FieldAccessor::Direct => {
                        // Nothing to add
                    }
                    FieldAccessor::BoxedAsRef => {
                        self.output.push_str(".as_ref()");
                    }
                    FieldAccessor::BoxedRefDeref => {
                        // Handled by unary deref
                    }
                    FieldAccessor::DerefDisplay => {
                        // Emit .as_ref() for Atom types to get &str for Display
                        self.output.push_str(".as_ref()");
                    }
                    FieldAccessor::Utf8Lossy => {
                        // String::from_utf8_lossy wrapper is emitted as prefix
                        // Emit .as_bytes() here, then close the wrapper paren
                        self.output.push_str(".as_bytes())");
                    }
                    FieldAccessor::EnumField { .. } => {
                        // No special handling needed
                    }
                    FieldAccessor::Optional { .. } => {
                        // No special handling needed
                    }
                    FieldAccessor::Replace { .. } => {
                        // Replacement already handled by rewriter (self.builder → self)
                        // Should not reach here
                    }
                }

                // Apply read conversion if present (e.g., .to_string() for Atom → String)
                // Skip for Utf8Lossy accessor - it's already included in the wrapper
                if !field_metadata.read_conversion.is_empty() && !matches!(field_metadata.accessor, FieldAccessor::Utf8Lossy) {
                    self.output.push_str(&field_metadata.read_conversion);
                }
            }

            DecoratedExprKind::Call(call) => {
                // Check if this is obj.clone() where obj is a narrowed enum variant
                // If so, wrap it: Expr::JSXElement(obj.clone())
                if let DecoratedExprKind::Member { object, property, .. } = &call.callee.kind {
                    if property == "clone" && call.args.is_empty() {
                        // Check if object has needs_enum_unwrap metadata
                        if let Some((parent_enum, variant)) = &object.metadata.needs_enum_unwrap {
                            eprintln!("[EMIT CLONE WRAP] Wrapping {}.clone() in {}::{}",
                                     parent_enum, parent_enum, variant);
                            // Emit: Box::new(Expr::JSXElement(obj.clone()))
                            self.output.push_str("Box::new(");
                            self.output.push_str(parent_enum);
                            self.output.push_str("::");
                            self.output.push_str(variant);
                            self.output.push('(');
                            self.emit_expr(object);
                            self.output.push_str(".clone()))");
                            return;
                        }
                    }
                }

                // CodeBuilder method call transformations (no longer needed - we generate real CodeBuilder)
                if let DecoratedExprKind::Member { object, property, .. } = &call.callee.kind {
                    if let DecoratedExprKind::Ident { name, .. } = &object.kind {
                        // Old code that mapped CodeBuilder to String is now removed
                        // CodeBuilder is a real struct now
                    }

                    // Old CodeBuilder method call transformations (no longer needed)
                    if false && (object.metadata.swc_type == "String" || object.metadata.swc_type == "CodeBuilder") {
                        match property.as_str() {
                            "append_line" => {
                                // builder.append_line(s) -> { builder.push_str(s); builder.push_str("\n"); }
                                self.output.push_str("{ ");
                                self.emit_expr(object);
                                self.output.push_str(".push_str(");
                                if !call.args.is_empty() {
                                    self.emit_expr(&call.args[0]);
                                }
                                self.output.push_str("); ");
                                self.emit_expr(object);
                                self.output.push_str(".push_str(\"\\n\"); }");
                                return;
                            }
                            "newline" => {
                                // builder.newline() -> builder.push_str("\n")
                                self.emit_expr(object);
                                self.output.push_str(".push_str(\"\\n\")");
                                return;
                            }
                            "indent" | "dedent" => {
                                // builder.indent() / builder.dedent() -> () (no-op for local CodeBuilder)
                                self.output.push_str("()");
                                return;
                            }
                            "to_string" => {
                                // builder.to_string() -> builder.clone()
                                self.emit_expr(object);
                                self.output.push_str(".clone()");
                                return;
                            }
                            _ => {}
                        }
                    }
                }

                // Check if callee is a Member with read_conversion that already includes ()
                let skip_parens = if let DecoratedExprKind::Member { ref field_metadata, .. } = call.callee.kind {
                    !field_metadata.read_conversion.is_empty() &&
                    field_metadata.read_conversion.ends_with("()")
                } else {
                    false
                };

                self.emit_expr(&call.callee);

                // Emit turbofish type arguments if present: ::<Type1, Type2>
                if !call.type_args.is_empty() {
                    self.output.push_str("::<");
                    for (i, ty) in call.type_args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.emit_ts_type_as_rust(ty);
                    }
                    self.output.push('>');
                }

                // Add ! suffix for macro calls
                if call.is_macro {
                    self.output.push('!');
                }

                // Don't add () if the callee already has it from read_conversion
                if !skip_parens {
                    self.output.push('(');
                    for (i, arg) in call.args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.emit_expr(arg);
                    }
                    self.output.push(')');
                }
            }

            DecoratedExprKind::Paren(inner) => {
                self.output.push('(');
                self.emit_expr(inner);
                self.output.push(')');
            }

            DecoratedExprKind::Block(block) => {
                self.output.push_str("{\n");
                self.indent += 1;
                self.emit_block(block);
                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');
            }

            DecoratedExprKind::Index { object, index } => {
                self.emit_expr(object);
                self.output.push('[');
                self.emit_expr(index);
                self.output.push(']');
            }

            DecoratedExprKind::StructInit(struct_init) => {
                // Use the SWC type from metadata (e.g., Identifier → Ident)
                self.output.push_str(&expr.metadata.swc_type);
                self.output.push_str(" { ");
                for (i, (field_name, field_expr)) in struct_init.fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(field_name);
                    self.output.push_str(": ");

                    // Emit the decorated field expression
                    self.emit_expr(field_expr);
                }
                self.output.push_str(" }");
            }

            DecoratedExprKind::VecInit(elements) => {
                self.output.push_str("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(elem);
                }
                self.output.push(']');
            }

            DecoratedExprKind::If(if_expr) => {
                self.output.push_str("if ");
                self.emit_expr(&if_expr.condition);
                self.output.push_str(" {\n");
                self.indent += 1;
                self.emit_block(&if_expr.then_branch);
                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');

                if let Some(ref else_branch) = if_expr.else_branch {
                    self.output.push_str(" else {\n");
                    self.indent += 1;
                    self.emit_block(else_branch);
                    self.indent -= 1;
                    self.emit_indent();
                    self.output.push('}');
                }
            }

            DecoratedExprKind::Match(match_expr) => {
                self.output.push_str("match ");
                self.emit_expr(&match_expr.expr);
                self.output.push_str(" {\n");
                self.indent += 1;

                for arm in &match_expr.arms {
                    self.emit_indent();
                    self.emit_pattern(&arm.pattern);

                    if let Some(ref guard) = arm.guard {
                        self.output.push_str(" if ");
                        self.emit_expr(guard);
                    }

                    self.output.push_str(" => {\n");
                    self.indent += 1;

                    // Emit statements, but skip semicolon on the last expression
                    for (i, stmt) in arm.body.stmts.iter().enumerate() {
                        let is_last = i == arm.body.stmts.len() - 1;

                        if is_last {
                            if let DecoratedStmt::Expr(expr) = stmt {
                                // Last statement is an expression - emit without semicolon
                                self.emit_indent();
                                self.emit_expr(expr);
                                self.output.push('\n');
                                continue;
                            }
                        }

                        // Normal statement emission
                        self.emit_stmt(stmt);
                    }

                    self.indent -= 1;
                    self.emit_line("}");
                }

                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');
            }

            DecoratedExprKind::Ref { mutable, expr: inner } => {
                // Check if inner is a boxed field access that needs dereferencing
                let needs_deref = matches!(&inner.kind,
                    DecoratedExprKind::Member { field_metadata, .. }
                    if matches!(&field_metadata.accessor, FieldAccessor::BoxedAsRef | FieldAccessor::BoxedRefDeref)
                );

                // FALLBACK: Check for known Box field names (for pattern-bound variables)
                let is_known_box_field = if let DecoratedExprKind::Member { field_metadata, .. } = &inner.kind {
                    // Check if field type is Box<T>
                    field_metadata.field_type.starts_with("Box<") ||
                    // ALSO check known SWC field names (for when type info is missing)
                    field_metadata.swc_field_name == "obj" ||     // MemberExpr.obj: Box<Expr>
                    field_metadata.swc_field_name == "expr"       // Common Box<Expr> field
                } else {
                    false
                };

                if needs_deref || is_known_box_field {
                    // Emit &* for boxed field access (e.g., &member.obj → &*member.obj)
                    self.output.push_str("&*");
                } else {
                    // Normal reference
                    self.output.push('&');
                }

                if *mutable {
                    self.output.push_str("mut ");
                }
                self.emit_expr(inner);
            }

            DecoratedExprKind::Deref(inner) => {
                self.output.push('*');
                self.emit_expr(inner);
            }

            DecoratedExprKind::Assign { left, right } => {
                self.emit_expr(left);
                self.output.push_str(" = ");

                // Check if we need to wrap the RHS in the parent enum constructor
                // This happens when assigning a narrowed type back to a field expecting the parent enum
                let needs_enum_wrap = if let Some((parent_enum, variant)) = &right.metadata.needs_enum_unwrap {
                    // The RHS is a narrowed type, check if LHS expects the parent enum
                    left.metadata.swc_type.contains(parent_enum) || left.metadata.swc_type.contains("Expr")
                } else {
                    false
                };

                if needs_enum_wrap {
                    if let Some((parent_enum, variant)) = &right.metadata.needs_enum_unwrap {
                        // Emit: Box::new(ParentEnum::Variant(rhs))
                        self.output.push_str("Box::new(");
                        self.output.push_str(parent_enum);
                        self.output.push_str("::");
                        self.output.push_str(variant);
                        self.output.push('(');
                        self.emit_expr(right);
                        self.output.push_str("))");
                    } else {
                        self.emit_expr(right);
                    }
                } else {
                    self.emit_expr(right);
                }
            }

            DecoratedExprKind::CompoundAssign { left, op, right } => {
                self.emit_expr(left);
                self.output.push(' ');
                self.output.push_str(&self.compound_op_to_string(op));
                self.output.push_str("= ");
                self.emit_expr(right);
            }

            DecoratedExprKind::Range { start, end, inclusive } => {
                if let Some(ref start_expr) = start {
                    self.emit_expr(start_expr);
                }
                if *inclusive {
                    self.output.push_str("..=");
                } else {
                    self.output.push_str("..");
                }
                if let Some(ref end_expr) = end {
                    self.emit_expr(end_expr);
                }
            }

            DecoratedExprKind::Try(inner) => {
                self.emit_expr(inner);
                self.output.push('?');
            }

            DecoratedExprKind::Tuple(elements) => {
                self.output.push('(');
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_expr(elem);
                }
                self.output.push(')');
            }

            DecoratedExprKind::Matches { expr: scrutinee, pattern } => {
                // matches! should have been expanded by rewriter
                self.output.push_str("matches!(");
                self.emit_expr(scrutinee);
                self.output.push_str(", ");
                self.emit_pattern(pattern);
                self.output.push(')');
            }

            DecoratedExprKind::Return(value) => {
                self.output.push_str("return");
                if let Some(ref expr) = value {
                    self.output.push(' ');
                    self.emit_expr(expr);
                }
            }

            DecoratedExprKind::Break => {
                self.output.push_str("break");
            }

            DecoratedExprKind::Continue => {
                self.output.push_str("continue");
            }

            DecoratedExprKind::RegexCall(regex_call) => {
                self.emit_regex_call(regex_call);
            }

            DecoratedExprKind::CustomPropAccess(access) => {
                // This should have been transformed by the rewriter into a Call expression
                // If we reach here, the rewriter didn't run
                panic!("CustomPropAccess should have been rewritten by SwcRewriter");
            }

            DecoratedExprKind::Closure(closure) => {
                // Emit Rust closure syntax: |params| body
                self.output.push('|');
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    match param {
                        ClosureParam::Ident(name) => self.output.push_str(name),
                        ClosureParam::Tuple(names) => {
                            self.output.push('(');
                            self.output.push_str(&names.join(", "));
                            self.output.push(')');
                        }
                        ClosureParam::Typed { name, ty } => {
                            self.output.push_str(name);
                            self.output.push_str(": ");
                            self.output.push_str(&format!("{:?}", ty)); // TODO: proper type emission
                        }
                    }
                }
                self.output.push('|');
                self.output.push(' ');

                // Emit the body - closure uses parser Expr, not DecoratedExpr
                self.emit_parser_expr(&closure.body);
            }
        }
    }

    // ========================================================================
    // LITERALS
    // ========================================================================

    fn emit_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::String(s) => {
                self.output.push('"');
                // Properly escape the string for Rust
                for ch in s.chars() {
                    match ch {
                        '\\' => self.output.push_str("\\\\"),
                        '"' => self.output.push_str("\\\""),
                        '\n' => self.output.push_str("\\n"),
                        '\r' => self.output.push_str("\\r"),
                        '\t' => self.output.push_str("\\t"),
                        '\0' => self.output.push_str("\\0"),
                        _ => self.output.push(ch),
                    }
                }
                self.output.push('"');
            }
            Literal::Int(n) => {
                self.output.push_str(&n.to_string());
            }
            Literal::Float(f) => {
                self.output.push_str(&f.to_string());
            }
            Literal::Bool(b) => {
                self.output.push_str(if *b { "true" } else { "false" });
            }
            Literal::Null => {
                self.output.push_str("None");
            }
            Literal::Unit => {
                self.output.push_str("()");
            }
        }
    }

    // ========================================================================
    // TYPE CONVERSIONS
    // ========================================================================

    /// Check if a type contains any references
    fn type_has_reference(&self, ty: &Type) -> bool {
        match ty {
            Type::Reference { .. } => true,
            Type::Container { type_args, .. } => type_args.iter().any(|t| self.type_has_reference(t)),
            Type::Optional(inner) => self.type_has_reference(inner),
            Type::Array { element } => self.type_has_reference(element),
            Type::Tuple(types) => types.iter().any(|t| self.type_has_reference(t)),
            _ => false,
        }
    }

    fn type_to_string(&self, ty: &Type) -> String {
        self.type_to_string_with_lifetime(ty, false)
    }

    /// Convert immutable references to mutable references for plugin visitor methods
    fn make_reference_mutable(&self, ty: &Type, add_lifetime: bool) -> String {
        match ty {
            Type::Reference { mutable: false, inner } => {
                // Convert &T to &mut T
                format!(
                    "&{}mut {}",
                    if add_lifetime { "'a " } else { "" },
                    self.type_to_string_with_lifetime(inner, add_lifetime)
                )
            }
            Type::Reference { mutable: true, inner } => {
                // Already mutable
                format!(
                    "&{}mut {}",
                    if add_lifetime { "'a " } else { "" },
                    self.type_to_string_with_lifetime(inner, add_lifetime)
                )
            }
            _ => {
                // Not a reference type - return as-is
                self.type_to_string_with_lifetime(ty, add_lifetime)
            }
        }
    }

    /// Check if a type contains AST node types (which can't be serialized)
    fn contains_ast_type(&self, ty: &Type) -> bool {
        match ty {
            Type::Named(name) => {
                // Common AST node type names
                matches!(name.as_str(),
                    "Expr" | "Stmt" | "Pattern" | "Declaration" |
                    "FunctionDeclaration" | "VariableDeclarator" | "CallExpression" |
                    "MemberExpression" | "Identifier" | "Literal" |
                    "JSXElement" | "JSXFragment" | "ArrayExpression" | "ObjectExpression" |
                    "BinaryExpression" | "UnaryExpression" | "AssignmentExpression" |
                    "ReturnStatement" | "IfStatement" | "WhileStatement" |
                    "BlockStatement" | "ExpressionStatement"
                )
            }
            Type::Container { type_args, .. } => {
                // Check if any type argument contains AST types
                type_args.iter().any(|t| self.contains_ast_type(t))
            }
            Type::Optional(inner) => self.contains_ast_type(inner),
            Type::Reference { inner, .. } => self.contains_ast_type(inner),
            Type::Array { element } => self.contains_ast_type(element),
            Type::Tuple(types) => types.iter().any(|t| self.contains_ast_type(t)),
            _ => false,
        }
    }

    fn type_to_string_with_lifetime(&self, ty: &Type, add_lifetime: bool) -> String {
        match ty {
            Type::Primitive(name) => {
                // Map ReluxScript/Babel types to Rust types for SWC
                match name.as_str() {
                    "Number" => "i32".to_string(),
                    "Str" => "String".to_string(),
                    "Bool" | "Boolean" => "bool".to_string(),
                    _ => name.clone(),
                }
            }
            Type::Named(name) => {
                // Map common type names to Rust equivalents
                match name.as_str() {
                    "Bool" | "Boolean" => "bool".to_string(),
                    "Str" => "String".to_string(),
                    _ => name.clone(),
                }
            }
            Type::Reference { mutable, inner } => {
                format!(
                    "&{}{}{}",
                    if add_lifetime { "'a " } else { "" },
                    if *mutable { "mut " } else { "" },
                    self.type_to_string_with_lifetime(inner, add_lifetime)
                )
            }
            Type::Container { name, type_args } => {
                if type_args.is_empty() {
                    name.clone()
                } else {
                    format!(
                        "{}<{}>",
                        name,
                        type_args.iter()
                            .map(|t| self.type_to_string_with_lifetime(t, add_lifetime))
                            .collect::<Vec<_>>()
                            .join(", ")
                    )
                }
            }
            Type::Array { element } => {
                format!("[{}]", self.type_to_string_with_lifetime(element, add_lifetime))
            }
            Type::Tuple(types) => {
                format!(
                    "({})",
                    types.iter()
                        .map(|t| self.type_to_string_with_lifetime(t, add_lifetime))
                        .collect::<Vec<_>>()
                        .join(", ")
                )
            }
            Type::Optional(inner) => {
                format!("Option<{}>", self.type_to_string_with_lifetime(inner, add_lifetime))
            }
            Type::Unit => {
                "()".to_string()
            }
            Type::FnTrait { params, return_type } => {
                format!(
                    "fn({}) -> {}",
                    params.iter()
                        .map(|t| self.type_to_string(t))
                        .collect::<Vec<_>>()
                        .join(", "),
                    self.type_to_string(return_type)
                )
            }
            Type::RawPointer { mutable, inner } => {
                format!(
                    "*{} {}",
                    if *mutable { "mut" } else { "const" },
                    self.type_to_string_with_lifetime(inner, add_lifetime)
                )
            }
        }
    }

    /// Emit a TsType as a Rust type string for turbofish syntax
    fn emit_ts_type_as_rust(&mut self, ty: &crate::parser::TsType) {
        use crate::parser::TsType;
        match ty {
            TsType::String => self.output.push_str("String"),
            TsType::Number => self.output.push_str("f64"),
            TsType::Boolean => self.output.push_str("bool"),
            TsType::Any => self.output.push('_'),  // Type inference placeholder
            TsType::Void => self.output.push_str("()"),
            TsType::Null | TsType::Undefined => self.output.push_str("()"),
            TsType::Never => self.output.push('!'),
            TsType::Unknown => self.output.push('_'),
            TsType::Array(inner) => {
                self.output.push_str("Vec<");
                self.emit_ts_type_as_rust(inner);
                self.output.push('>');
            }
            TsType::Tuple(types) => {
                self.output.push('(');
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_ts_type_as_rust(t);
                }
                self.output.push(')');
            }
            TsType::Union(types) => {
                // For unions, just use the first type (simplification)
                if !types.is_empty() {
                    self.emit_ts_type_as_rust(&types[0]);
                } else {
                    self.output.push('_');
                }
            }
            TsType::Intersection(types) => {
                // For intersections, just use the first type (simplification)
                if !types.is_empty() {
                    self.emit_ts_type_as_rust(&types[0]);
                } else {
                    self.output.push('_');
                }
            }
            TsType::TypeReference { name, type_args } => {
                // Map common type names to Rust equivalents
                let rust_name = match name.as_str() {
                    "Str" => "String",
                    "Bool" | "Boolean" => "bool",
                    "Float" => "f64",
                    "Int" | "Number" => "i32",
                    "f32" | "f64" | "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => name.as_str(),
                    _ => name.as_str(),
                };
                self.output.push_str(rust_name);
                if !type_args.is_empty() {
                    self.output.push('<');
                    for (i, arg) in type_args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.emit_ts_type_as_rust(arg);
                    }
                    self.output.push('>');
                }
            }
            TsType::FunctionType { params, return_type } => {
                self.output.push_str("fn(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_ts_type_as_rust(p);
                }
                self.output.push_str(") -> ");
                self.emit_ts_type_as_rust(return_type);
            }
            TsType::LiteralString(_) | TsType::LiteralNumber(_) | TsType::LiteralBoolean(_) => {
                // Literal types just use the base type
                self.output.push('_');
            }
        }
    }

    fn binary_op_to_string(&self, op: &BinaryOp) -> String {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
        .to_string()
    }

    fn unary_op_to_string(&self, op: &UnaryOp) -> String {
        match op {
            UnaryOp::Not => "!",
            UnaryOp::Neg => "-",
            UnaryOp::Deref => "*",
            UnaryOp::Ref => "&",
            UnaryOp::RefMut => "&mut ",
        }
        .to_string()
    }

    fn compound_op_to_string(&self, op: &CompoundAssignOp) -> String {
        match op {
            CompoundAssignOp::AddAssign => "+",
            CompoundAssignOp::SubAssign => "-",
            CompoundAssignOp::MulAssign => "*",
            CompoundAssignOp::DivAssign => "/",
        }
        .to_string()
    }

    // ========================================================================
    // WRITER-SPECIFIC HELPERS
    // ========================================================================

    fn emit_writer_constructor(&mut self, state_struct: &Option<StructDecl>) {
        self.emit_line("pub fn new() -> Self {");
        self.indent += 1;
        self.emit_line("Self {");
        self.indent += 1;
        self.emit_line("output: String::new(),");
        self.emit_line("indent_level: 0,");

        // Initialize State fields with defaults
        if let Some(state) = state_struct {
            for field in &state.fields {
                let default_value = self.get_default_value_for_type(&field.ty);
                self.emit_line(&format!("{}: {},", field.name, default_value));
            }
        }

        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    fn emit_codebuilder_methods(&mut self) {
        // append method
        self.emit_line("fn append(&mut self, s: &str) {");
        self.indent += 1;
        self.emit_line("self.output.push_str(s);");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // append_line method
        self.emit_line("fn append_line(&mut self, s: &str) {");
        self.indent += 1;
        self.emit_line("for _ in 0..self.indent_level {");
        self.indent += 1;
        self.emit_line("self.output.push_str(\"    \");");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("self.output.push_str(s);");
        self.emit_line("self.output.push('\\n');");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // indent method
        self.emit_line("fn indent(&mut self) {");
        self.indent += 1;
        self.emit_line("self.indent_level += 1;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // dedent method
        self.emit_line("fn dedent(&mut self) {");
        self.indent += 1;
        self.emit_line("if self.indent_level > 0 {");
        self.indent += 1;
        self.emit_line("self.indent_level -= 1;");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // newline method
        self.emit_line("fn newline(&mut self) {");
        self.indent += 1;
        self.emit_line("self.output.push('\\n');");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // to_string method (for finish/exit hooks)
        self.emit_line("pub fn to_string(&self) -> String {");
        self.indent += 1;
        self.emit_line("self.output.clone()");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    // ========================================================================
    // UNDECORATED EXPRESSION EMISSION (for closures, etc.)
    // ========================================================================

    fn emit_parser_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => self.emit_literal(lit),
            Expr::Ident(ident) => self.output.push_str(&ident.name),
            Expr::Binary(bin) => {
                self.output.push('(');
                self.emit_parser_expr(&bin.left);
                self.output.push(' ');
                self.output.push_str(&self.binary_op_to_string(&bin.op));
                self.output.push(' ');
                self.emit_parser_expr(&bin.right);
                self.output.push(')');
            }
            Expr::Unary(un) => {
                self.output.push_str(&self.unary_op_to_string(&un.op));
                self.emit_parser_expr(&un.operand);
            }
            Expr::Member(mem) => {
                self.emit_parser_expr(&mem.object);
                self.output.push('.');
                self.output.push_str(&mem.property);
            }
            Expr::Call(call) => {
                self.emit_parser_expr(&call.callee);
                self.output.push('(');
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_parser_expr(arg);
                }
                self.output.push(')');
            }
            Expr::Block(block) => {
                self.output.push_str("{\n");
                self.indent += 1;
                for stmt in &block.stmts {
                    self.emit_parser_stmt(stmt);
                }
                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');
            }
            Expr::If(if_expr) => {
                self.output.push_str("if ");
                self.emit_parser_expr(&if_expr.condition);
                self.output.push_str(" {\n");
                self.indent += 1;
                for stmt in &if_expr.then_branch.stmts {
                    self.emit_parser_stmt(stmt);
                }
                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');
                if let Some(ref else_branch) = if_expr.else_branch {
                    self.output.push_str(" else {\n");
                    self.indent += 1;
                    for stmt in &else_branch.stmts {
                        self.emit_parser_stmt(stmt);
                    }
                    self.indent -= 1;
                    self.emit_indent();
                    self.output.push('}');
                }
            }
            Expr::StructInit(struct_init) => {
                self.output.push_str(&struct_init.name);
                self.output.push_str(" { ");
                for (i, (field_name, field_expr)) in struct_init.fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(field_name);
                    self.output.push_str(": ");
                    self.emit_parser_expr(field_expr);
                }
                self.output.push_str(" }");
            }
            Expr::Deref(deref) => {
                self.output.push('*');
                self.emit_parser_expr(&deref.expr);
            }
            Expr::Ref(ref_expr) => {
                self.output.push('&');
                if ref_expr.mutable {
                    self.output.push_str("mut ");
                }
                self.emit_parser_expr(&ref_expr.expr);
            }
            Expr::Closure(closure) => {
                self.output.push('|');
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    match param {
                        crate::parser::ClosureParam::Ident(name) => {
                            self.output.push_str(name);
                        }
                        crate::parser::ClosureParam::Tuple(names) => {
                            self.output.push('(');
                            for (j, name) in names.iter().enumerate() {
                                if j > 0 {
                                    self.output.push_str(", ");
                                }
                                self.output.push_str(name);
                            }
                            self.output.push(')');
                        }
                        crate::parser::ClosureParam::Typed { name, ty } => {
                            self.output.push_str(name);
                            self.output.push_str(": ");
                            self.emit_parser_type(ty);
                        }
                    }
                }
                self.output.push_str("| ");
                self.emit_parser_expr(&closure.body);
            }
            Expr::Index(idx) => {
                self.emit_parser_expr(&idx.object);
                self.output.push('[');
                self.emit_parser_expr(&idx.index);
                self.output.push(']');
            }
            Expr::VecInit(vec_init) => {
                self.output.push_str("vec![");
                for (i, elem) in vec_init.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_parser_expr(elem);
                }
                self.output.push(']');
            }
            Expr::Match(match_expr) => {
                self.output.push_str("match ");
                self.emit_parser_expr(&match_expr.scrutinee);
                self.output.push_str(" {\n");
                self.indent += 1;
                for arm in &match_expr.arms {
                    self.emit_indent();
                    self.emit_parser_pattern(&arm.pattern);
                    self.output.push_str(" => ");
                    self.emit_parser_expr(&arm.body);
                    self.output.push_str(",\n");
                }
                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');
            }
            Expr::Assign(assign) => {
                self.emit_parser_expr(&assign.target);
                self.output.push_str(" = ");
                self.emit_parser_expr(&assign.value);
            }
            Expr::CompoundAssign(compound) => {
                self.emit_parser_expr(&compound.target);
                self.output.push(' ');
                self.output.push_str(&self.compound_op_to_string(&compound.op));
                self.output.push(' ');
                self.emit_parser_expr(&compound.value);
            }
            Expr::Range(range) => {
                if let Some(ref start) = range.start {
                    self.emit_parser_expr(start);
                }
                self.output.push_str("..");
                if let Some(ref end) = range.end {
                    self.emit_parser_expr(end);
                }
            }
            Expr::Paren(inner) => {
                self.output.push('(');
                self.emit_parser_expr(inner);
                self.output.push(')');
            }
            Expr::Try(inner) => {
                self.emit_parser_expr(inner);
                self.output.push('?');
            }
            Expr::Tuple(exprs) => {
                self.output.push('(');
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_parser_expr(expr);
                }
                // Add trailing comma for single-element tuples
                if exprs.len() == 1 {
                    self.output.push(',');
                }
                self.output.push(')');
            }
            Expr::Path(path) => {
                self.output.push_str(&path.segments.join("::"));
            }
            Expr::Return(ret) => {
                self.output.push_str("return");
                if let Some(ref expr) = ret {
                    self.output.push(' ');
                    self.emit_parser_expr(expr);
                }
            }
            Expr::Break => {
                self.output.push_str("break");
            }
            Expr::Continue => {
                self.output.push_str("continue");
            }
            Expr::Matches(matches) => {
                self.output.push_str("matches!(");
                self.emit_parser_expr(&matches.scrutinee);
                self.output.push_str(", ");
                self.emit_parser_pattern(&matches.pattern);
                self.output.push(')');
            }
            Expr::RegexCall(_) | Expr::CustomPropAccess(_) => {
                // These should be handled by the decorated AST path
                self.output.push_str("/* special expr */");
            }
        }
    }

    fn emit_parser_type(&mut self, ty: &crate::parser::Type) {
        use crate::parser::Type;
        match ty {
            Type::Primitive(name) => self.output.push_str(name),
            Type::Named(name) => self.output.push_str(name),
            Type::Container { name, type_args } => {
                self.output.push_str(name);
                if !type_args.is_empty() {
                    self.output.push('<');
                    for (i, arg) in type_args.iter().enumerate() {
                        if i > 0 {
                            self.output.push_str(", ");
                        }
                        self.emit_parser_type(arg);
                    }
                    self.output.push('>');
                }
            }
            Type::Reference { mutable, inner } => {
                self.output.push('&');
                if *mutable {
                    self.output.push_str("mut ");
                }
                self.emit_parser_type(inner);
            }
            Type::Tuple(types) => {
                self.output.push('(');
                for (i, t) in types.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_parser_type(t);
                }
                self.output.push(')');
            }
            Type::FnTrait { params, return_type } => {
                self.output.push_str("Fn(");
                for (i, p) in params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_parser_type(p);
                }
                self.output.push_str(") -> ");
                self.emit_parser_type(return_type);
            }
            Type::Array { element } => {
                self.output.push('[');
                self.emit_parser_type(element);
                self.output.push(']');
            }
            Type::Optional(inner) => {
                self.output.push_str("Option<");
                self.emit_parser_type(inner);
                self.output.push('>');
            }
            Type::RawPointer { mutable, inner } => {
                self.output.push('*');
                if *mutable {
                    self.output.push_str("mut ");
                } else {
                    self.output.push_str("const ");
                }
                self.emit_parser_type(inner);
            }
            Type::Unit => self.output.push_str("()"),
        }
    }

    fn emit_parser_pattern(&mut self, pattern: &crate::parser::Pattern) {
        use crate::parser::Pattern;
        match pattern {
            Pattern::Literal(lit) => self.emit_literal(lit),
            Pattern::Ident(name) => self.output.push_str(name),
            Pattern::Wildcard => self.output.push('_'),
            Pattern::Tuple(patterns) => {
                self.output.push('(');
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_parser_pattern(p);
                }
                self.output.push(')');
            }
            Pattern::Struct { name, fields } => {
                self.output.push_str(name);
                self.output.push_str(" { ");
                for (i, (field_name, field_pattern)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(field_name);
                    self.output.push_str(": ");
                    self.emit_parser_pattern(field_pattern);
                }
                self.output.push_str(" }");
            }
            Pattern::Variant { name, inner } => {
                self.output.push_str(name);
                if let Some(ref inner_pattern) = inner {
                    self.output.push('(');
                    self.emit_parser_pattern(inner_pattern);
                    self.output.push(')');
                }
            }
            Pattern::Array(patterns) => {
                self.output.push('[');
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_parser_pattern(p);
                }
                self.output.push(']');
            }
            Pattern::Object(props) => {
                use crate::parser::ObjectPatternProp;
                self.output.push_str("{ ");
                for (i, prop) in props.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    match prop {
                        ObjectPatternProp::Shorthand(name) => {
                            self.output.push_str(name);
                        }
                        ObjectPatternProp::KeyValue { key, value } => {
                            self.output.push_str(key);
                            self.output.push_str(": ");
                            self.emit_parser_pattern(value);
                        }
                        ObjectPatternProp::Rest(name) => {
                            self.output.push_str("..");
                            self.output.push_str(name);
                        }
                        ObjectPatternProp::Or(patterns) => {
                            for (j, p) in patterns.iter().enumerate() {
                                if j > 0 {
                                    self.output.push_str(" | ");
                                }
                                self.emit_parser_pattern(p);
                            }
                        }
                    }
                }
                self.output.push_str(" }");
            }
            Pattern::Rest(inner) => {
                self.output.push_str("..");
                self.emit_parser_pattern(inner);
            }
            Pattern::Or(patterns) => {
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(" | ");
                    }
                    self.emit_parser_pattern(p);
                }
            }
            Pattern::Ref { is_mut, pattern } => {
                self.output.push_str("ref ");
                if *is_mut {
                    self.output.push_str("mut ");
                }
                self.emit_parser_pattern(pattern);
            }
        }
    }

    fn emit_parser_stmt(&mut self, stmt: &Stmt) {
        self.emit_parser_stmt_with_context(stmt, false);
    }

    fn emit_parser_stmt_with_context(&mut self, stmt: &Stmt, is_last_in_block: bool) {
        match stmt {
            Stmt::Expr(expr_stmt) => {
                self.emit_indent();
                self.emit_parser_expr(&expr_stmt.expr);
                // Last expression in a block is the implicit return value (no semicolon)
                // Also, if/block/match expressions don't need semicolons
                let is_control_flow = matches!(expr_stmt.expr, Expr::If(_) | Expr::Block(_) | Expr::Match(_));
                let needs_semicolon = !is_last_in_block && !is_control_flow;
                if needs_semicolon {
                    self.output.push(';');
                }
                self.output.push('\n');
            }
            Stmt::Return(ret) => {
                self.emit_indent();
                self.output.push_str("return");
                if let Some(ref expr) = ret.value {
                    self.output.push(' ');
                    self.emit_parser_expr(expr);
                }
                self.output.push_str(";\n");
            }
            Stmt::Let(let_stmt) => {
                self.emit_indent();
                self.output.push_str("let ");
                if let_stmt.mutable {
                    self.output.push_str("mut ");
                }
                self.emit_parser_pattern(&let_stmt.pattern);
                if let Some(ref ty) = let_stmt.ty {
                    self.output.push_str(": ");
                    self.emit_parser_type(ty);
                }
                if let Some(ref init) = let_stmt.init {
                    self.output.push_str(" = ");
                    self.emit_parser_expr(init);
                }
                self.output.push_str(";\n");
            }
            Stmt::If(if_stmt) => {
                self.emit_indent();
                self.output.push_str("if ");
                self.emit_parser_expr(&if_stmt.condition);
                self.output.push_str(" {\n");
                self.indent += 1;
                let then_stmts_len = if_stmt.then_branch.stmts.len();
                for (i, stmt) in if_stmt.then_branch.stmts.iter().enumerate() {
                    let is_last = i == then_stmts_len - 1;
                    self.emit_parser_stmt_with_context(stmt, is_last);
                }
                self.indent -= 1;
                self.emit_indent();
                self.output.push('}');

                // Handle else-if branches
                for (condition, block) in &if_stmt.else_if_branches {
                    self.output.push_str(" else if ");
                    self.emit_parser_expr(condition);
                    self.output.push_str(" {\n");
                    self.indent += 1;
                    let stmts_len = block.stmts.len();
                    for (i, stmt) in block.stmts.iter().enumerate() {
                        let is_last = i == stmts_len - 1;
                        self.emit_parser_stmt_with_context(stmt, is_last);
                    }
                    self.indent -= 1;
                    self.emit_indent();
                    self.output.push('}');
                }

                // Handle else branch
                if let Some(ref else_branch) = if_stmt.else_branch {
                    self.output.push_str(" else {\n");
                    self.indent += 1;
                    let stmts_len = else_branch.stmts.len();
                    for (i, stmt) in else_branch.stmts.iter().enumerate() {
                        let is_last = i == stmts_len - 1;
                        self.emit_parser_stmt_with_context(stmt, is_last);
                    }
                    self.indent -= 1;
                    self.emit_indent();
                    self.output.push('}');
                }
                self.output.push('\n');
            }
            _ => {
                self.emit_indent();
                self.output.push_str("/* complex stmt */\n");
            }
        }
    }

    // ========================================================================
    // OUTPUT UTILITIES
    // ========================================================================

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("    ");
        }
    }

    fn emit_line(&mut self, s: &str) {
        self.emit_indent();
        self.output.push_str(s);
        self.output.push('\n');
    }

    fn emit_comment(&mut self, s: &str) {
        self.emit_line(&format!("// {}", s));
    }

    // ========================================================================
    // UNDECORATED EXPRESSION EMISSION (for StructInit fields, etc.)
    // ========================================================================

    /// Emit undecorated parser Expr (fallback for expressions that aren't decorated yet)
    /// This is only used for closures and other edge cases that haven't been fully decorated
    fn emit_undecorated_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(ident) => {
                self.output.push_str(&ident.name);
            }
            Expr::Literal(lit) => {
                self.emit_literal(lit);
            }
            Expr::Binary(bin) => {
                self.output.push('(');
                self.emit_undecorated_expr(&bin.left);
                self.output.push(' ');
                self.output.push_str(&self.binary_op_to_string(&bin.op));
                self.output.push(' ');
                self.emit_undecorated_expr(&bin.right);
                self.output.push(')');
            }
            Expr::Member(mem) => {
                self.emit_undecorated_expr(&mem.object);
                self.output.push('.');
                self.output.push_str(&mem.property);
            }
            Expr::Call(call) => {
                self.emit_undecorated_expr(&call.callee);
                self.output.push('(');
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_undecorated_expr(arg);
                }
                self.output.push(')');
            }
            Expr::VecInit(vec_init) => {
                self.output.push_str("vec![");
                for (i, elem) in vec_init.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.emit_undecorated_expr(elem);
                }
                self.output.push(']');
            }
            _ => {
                self.output.push_str("/* undecorated expr */");
            }
        }
    }

    // ========================================================================
    // HELPER MODULES
    // ========================================================================

    fn emit_parser_helpers(&mut self) {
        self.emit_line("// Parser module helper functions");
        self.emit_line("mod parser {");
        self.indent += 1;
        self.emit_line("use super::*;");
        self.emit_line("");

        // parser::parse_file
        self.emit_line("pub fn parse_file(path: &str) -> Result<Program, String> {");
        self.indent += 1;
        self.emit_line("let source_map = Arc::new(SourceMap::default());");
        self.emit_line("let code = std::fs::read_to_string(path)");
        self.indent += 1;
        self.emit_line(".map_err(|e| format!(\"Failed to read file: {}\", e))?;");
        self.indent -= 1;
        self.emit_line("let file = source_map.new_source_file(");
        self.indent += 1;
        self.emit_line("FileName::Real(path.into()),");
        self.emit_line("code,");
        self.indent -= 1;
        self.emit_line(");");
        self.emit_line("let syntax = Syntax::Typescript(TsConfig {");
        self.indent += 1;
        self.emit_line("tsx: true,");
        self.emit_line("decorators: false,");
        self.emit_line("..Default::default()");
        self.indent -= 1;
        self.emit_line("});");
        self.emit_line("let mut parser = Parser::new(syntax, StringInput::from(&*file), None);");
        self.emit_line("parser.parse_program()");
        self.indent += 1;
        self.emit_line(".map_err(|e| format!(\"Parse error: {:?}\", e))");
        self.indent -= 1;
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // parser::parse
        self.emit_line("pub fn parse(code: &str) -> Result<Program, String> {");
        self.indent += 1;
        self.emit_line("let source_map = Arc::new(SourceMap::default());");
        self.emit_line("let file = source_map.new_source_file(");
        self.indent += 1;
        self.emit_line("FileName::Anon,");
        self.emit_line("code.to_string(),");
        self.indent -= 1;
        self.emit_line(");");
        self.emit_line("let syntax = Syntax::Typescript(TsConfig {");
        self.indent += 1;
        self.emit_line("tsx: true,");
        self.emit_line("decorators: false,");
        self.emit_line("..Default::default()");
        self.indent -= 1;
        self.emit_line("});");
        self.emit_line("let mut parser = Parser::new(syntax, StringInput::from(&*file), None);");
        self.emit_line("parser.parse_program()");
        self.indent += 1;
        self.emit_line(".map_err(|e| format!(\"Parse error: {:?}\", e))");
        self.indent -= 1;
        self.indent -= 1;
        self.emit_line("}");

        self.indent -= 1;
        self.emit_line("}");
    }

    fn emit_regex_call(&mut self, regex_call: &crate::codegen::decorated_ast::DecoratedRegexCall) {
        use crate::parser::RegexMethod;

        // Mark that regex crate is used
        self.uses_regex = true;

        match regex_call.method {
            RegexMethod::Matches => {
                // Regex::matches(text, pattern) -> RegexPattern::new(r"pattern").unwrap().is_match(text)
                self.output.push_str("RegexPattern::new(r\"");
                self.output.push_str(&regex_call.pattern);
                self.output.push_str("\").unwrap().is_match(");
                self.emit_expr(&regex_call.text_arg);
                self.output.push(')');
            }

            RegexMethod::Find => {
                // Regex::find(text, pattern) -> RegexPattern::new(r"pattern").unwrap().find(text).map(|m| m.as_str().to_string())
                self.output.push_str("RegexPattern::new(r\"");
                self.output.push_str(&regex_call.pattern);
                self.output.push_str("\").unwrap().find(");
                self.emit_expr(&regex_call.text_arg);
                self.output.push_str(").map(|m| m.as_str().to_string())");
            }

            RegexMethod::FindAll => {
                // Regex::find_all(text, pattern) -> RegexPattern::new(r"pattern").unwrap().find_iter(text).map(|m| m.as_str().to_string()).collect::<Vec<String>>()
                self.output.push_str("RegexPattern::new(r\"");
                self.output.push_str(&regex_call.pattern);
                self.output.push_str("\").unwrap().find_iter(");
                self.emit_expr(&regex_call.text_arg);
                self.output.push_str(").map(|m| m.as_str().to_string()).collect::<Vec<String>>()");
            }

            RegexMethod::Captures => {
                // Regex::captures(text, pattern) -> __regex_captures(text, r"pattern")
                // Mark that we need the helper function
                self.needs_regex_captures_helper = true;
                self.output.push_str("__regex_captures(");
                self.emit_expr(&regex_call.text_arg);
                self.output.push_str(", r\"");
                self.output.push_str(&regex_call.pattern);
                self.output.push_str("\")");
            }

            RegexMethod::Replace => {
                // Regex::replace(text, pattern, replacement) -> RegexPattern::new(r"pattern").unwrap().replace(text, replacement).to_string()
                self.output.push_str("RegexPattern::new(r\"");
                self.output.push_str(&regex_call.pattern);
                self.output.push_str("\").unwrap().replace(");
                self.emit_expr(&regex_call.text_arg);
                self.output.push_str(", ");
                if let Some(ref replacement) = regex_call.replacement_arg {
                    self.emit_expr(replacement);
                }
                self.output.push_str(").to_string()");
            }

            RegexMethod::ReplaceAll => {
                // Regex::replace_all(text, pattern, replacement) -> RegexPattern::new(r"pattern").unwrap().replace_all(text, replacement).to_string()
                self.output.push_str("RegexPattern::new(r\"");
                self.output.push_str(&regex_call.pattern);
                self.output.push_str("\").unwrap().replace_all(");
                self.emit_expr(&regex_call.text_arg);
                self.output.push_str(", ");
                if let Some(ref replacement) = regex_call.replacement_arg {
                    self.emit_expr(replacement);
                }
                self.output.push_str(").to_string()");
            }
        }
    }

    fn emit_codegen_helpers(&mut self) {
        self.emit_line("// Codegen helper functions");
        self.emit_line("fn codegen_to_string<N: Node>(node: &N) -> String {");
        self.indent += 1;
        self.emit_line("let mut buf = vec![];");
        self.emit_line("{");
        self.indent += 1;
        self.emit_line("let cm = swc_common::sync::Lrc::new(SourceMap::default());");
        self.emit_line("let mut emitter = Emitter {");
        self.indent += 1;
        self.emit_line("cfg: CodegenConfig::default(),");
        self.emit_line("cm: cm.clone(),");
        self.emit_line("comments: None,");
        self.emit_line("wr: Box::new(JsWriter::new(cm.clone(), \"\\n\", &mut buf, None)),");
        self.indent -= 1;
        self.emit_line("};");
        self.emit_line("node.emit_with(&mut emitter).unwrap();");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("String::from_utf8(buf).unwrap()");
        self.indent -= 1;
        self.emit_line("}");
    }

    fn emit_codebuilder_helper(&mut self) {
        self.emit_line("// CodeBuilder type for code generation");
        self.emit_line("struct CodeBuilder {");
        self.indent += 1;
        self.emit_line("buffer: String,");
        self.emit_line("indent_level: usize,");
        self.emit_line("indent_string: String,");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("impl CodeBuilder {");
        self.indent += 1;
        self.emit_line("fn new() -> Self {");
        self.indent += 1;
        self.emit_line("Self {");
        self.indent += 1;
        self.emit_line("buffer: String::new(),");
        self.emit_line("indent_level: 0,");
        self.emit_line("indent_string: \"    \".to_string(),");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("fn append(&mut self, s: &str) {");
        self.indent += 1;
        self.emit_line("self.buffer.push_str(s);");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("fn append_line(&mut self, s: &str) {");
        self.indent += 1;
        self.emit_line("for _ in 0..self.indent_level {");
        self.indent += 1;
        self.emit_line("self.buffer.push_str(&self.indent_string);");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("self.buffer.push_str(s);");
        self.emit_line("self.buffer.push('\\n');");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("fn newline(&mut self) {");
        self.indent += 1;
        self.emit_line("self.buffer.push('\\n');");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("fn indent(&mut self) {");
        self.indent += 1;
        self.emit_line("self.indent_level += 1;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("fn dedent(&mut self) {");
        self.indent += 1;
        self.emit_line("if self.indent_level > 0 {");
        self.indent += 1;
        self.emit_line("self.indent_level -= 1;");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
        self.emit_line("fn to_string(self) -> String {");
        self.indent += 1;
        self.emit_line("self.buffer");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
    }

    fn emit_regex_helpers(&mut self) {
        self.emit_line("// Regex helper functions");
        self.emit_line("fn __regex_captures(text: &str, pattern: &str) -> Option<__Captures> {");
        self.indent += 1;
        self.emit_line("let re = RegexPattern::new(pattern).unwrap();");
        self.emit_line("re.captures(text).map(|caps| __Captures { inner: caps })");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        self.emit_line("struct __Captures<'a> {");
        self.indent += 1;
        self.emit_line("inner: regex::Captures<'a>,");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        self.emit_line("impl<'a> __Captures<'a> {");
        self.indent += 1;
        self.emit_line("fn get(&self, index: usize) -> String {");
        self.indent += 1;
        self.emit_line("self.inner.get(index)");
        self.indent += 1;
        self.emit_line(".map(|m| m.as_str().to_string())");
        self.emit_line(".unwrap_or_default()");
        self.indent -= 2;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");
    }

    fn get_default_value_for_type(&self, ty: &Type) -> String {
        match ty {
            Type::Primitive(name) => {
                match name.as_str() {
                    "Str" => "String::new()".to_string(),
                    "Number" => "0".to_string(),
                    "Bool" => "false".to_string(),
                    "()" => "()".to_string(),
                    "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => "0".to_string(),
                    "f32" | "f64" => "0.0".to_string(),
                    "char" => "'\\0'".to_string(),
                    _ => "Default::default()".to_string(),
                }
            }
            Type::Container { name, .. } => {
                match name.as_str() {
                    "Vec" => "Vec::new()".to_string(),
                    "HashMap" => "HashMap::new()".to_string(),
                    "HashSet" => "HashSet::new()".to_string(),
                    "Option" => "None".to_string(),
                    _ => format!("{}::new()", name),
                }
            }
            Type::Optional(_) => "None".to_string(),
            Type::Array { .. } => "Vec::new()".to_string(),
            Type::Named(name) => {
                // Handle special types
                match name.as_str() {
                    "CodeBuilder" => "String::new()".to_string(),
                    _ => "Default::default()".to_string(),
                }
            }
            _ => "Default::default()".to_string(),
        }
    }

    // ========================================================================
    // CUSTOM AST PROPERTIES - INFRASTRUCTURE GENERATION
    // ========================================================================

    fn emit_custom_prop_value_enum(&mut self) {
        self.emit_line("#[derive(Clone, Debug)]");
        self.emit_line("enum CustomPropValue {");
        self.indent += 1;
        self.emit_line("Bool(bool),");
        self.emit_line("I32(i32),");
        self.emit_line("I64(i64),");
        self.emit_line("F64(f64),");
        self.emit_line("Str(String),");
        // TODO: Add Vec, Map, and user-defined types if needed
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    fn emit_custom_prop_helpers(&mut self) {
        self.emit_line("impl State {");
        self.indent += 1;

        // get_node_id: Generate unique ID for AST nodes
        self.emit_line("fn get_node_id<T>(&self, node: &T) -> usize {");
        self.indent += 1;
        self.emit_line("// Use node memory address as ID");
        self.emit_line("node as *const T as usize");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // set_custom_prop: Set a custom property
        self.emit_line("fn set_custom_prop<T>(&mut self, node: &T, prop: &str, value: CustomPropValue) {");
        self.indent += 1;
        self.emit_line("let node_id = self.get_node_id(node);");
        self.emit_line("self.__custom_props");
        self.indent += 1;
        self.emit_line(".entry(node_id)");
        self.emit_line(".or_insert_with(std::collections::HashMap::new)");
        self.emit_line(".insert(prop.to_string(), value);");
        self.indent -= 1;
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // get_custom_prop: Get a custom property
        self.emit_line("fn get_custom_prop<T>(&self, node: &T, prop: &str) -> Option<&CustomPropValue> {");
        self.indent += 1;
        self.emit_line("let node_id = self.get_node_id(node);");
        self.emit_line("self.__custom_props");
        self.indent += 1;
        self.emit_line(".get(&node_id)");
        self.emit_line(".and_then(|m| m.get(prop))");
        self.indent -= 1;
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // delete_custom_prop: Remove a custom property
        self.emit_line("fn delete_custom_prop<T>(&mut self, node: &T, prop: &str) {");
        self.indent += 1;
        self.emit_line("let node_id = self.get_node_id(node);");
        self.emit_line("if let Some(props) = self.__custom_props.get_mut(&node_id) {");
        self.indent += 1;
        self.emit_line("props.remove(prop);");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("}");

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    fn emit_traverse_stmt(&mut self, _traverse: &Box<DecoratedTraverseStmt>) {
        // Traverse statements should have been transformed by the Hoister stage
        // If we see one here, it's a placeholder that should emit a comment
        self.emit_line("// Traverse statement (should have been hoisted)");
    }
}

