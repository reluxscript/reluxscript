//! SWC Emitter - Emits Rust code from decorated/rewritten AST
//!
//! This is the final stage of the pipeline:
//! 1. Receives transformed, decorated AST with all metadata
//! 2. Emits Rust code as strings
//! 3. NO semantic decisions - just string emission based on AST structure
//!
//! The emitter is "dumb" by design - all transformations happened in earlier stages.

use super::swc_decorator::{DecoratedProgram, DecoratedTopLevelDecl, DecoratedPlugin, DecoratedWriter, DecoratedPluginItem, DecoratedFnDecl, DecoratedImplBlock};
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

    /// Whether regex captures helper is needed
    needs_regex_captures_helper: bool,

    /// Whether regex crate is used
    uses_regex: bool,
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
            needs_regex_captures_helper: false,
            uses_regex: false,
        }
    }

    /// Main entry point: emit entire program
    pub fn emit_program(&mut self, program: &DecoratedProgram) -> String {
        // Detect what imports we need
        self.detect_imports(program);

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
                for item in &writer.body {
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
            DecoratedTopLevelDecl::Undecorated(_) => {
                // Undecorated code (interfaces, modules) - skip
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
            _ => {}
        }
    }

    fn detect_regex_usage_in_block(&mut self, block: &crate::codegen::decorated_ast::DecoratedBlock) {
        use crate::codegen::decorated_ast::{DecoratedStmt, DecoratedExprKind};
        for stmt in &block.stmts {
            match stmt {
                DecoratedStmt::Let(let_stmt) => {
                    self.detect_regex_usage_in_expr(&let_stmt.init);
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
        if uses.is_empty() {
            return;
        }

        for use_stmt in uses {
            let is_file_module = use_stmt.path.starts_with("./") || use_stmt.path.starts_with("../");

            if is_file_module {
                // File module: convert path to module name
                // e.g., "./helpers.lux" -> "helpers"
                // e.g., "../utils/types.lux" -> "types" (just use the filename)
                let module_name = self.extract_module_name_from_path(&use_stmt.path);

                // Emit mod declaration
                self.emit_line(&format!("mod {};", module_name));

                // Emit use statement for imports
                if !use_stmt.imports.is_empty() {
                    // Named imports: use helpers::{get_component_name, escape_string};
                    let imports = use_stmt.imports.join(", ");
                    self.emit_line(&format!("use {}::{{{}}};", module_name, imports));
                } else if let Some(alias) = &use_stmt.alias {
                    // Aliased import: use helpers as h;
                    self.emit_line(&format!("use {} as {};", module_name, alias));
                } else {
                    // Full import: use helpers;
                    self.emit_line(&format!("use {};", module_name));
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
            DecoratedTopLevelDecl::Undecorated(_) => {
                self.emit_line("// Undecorated top-level declaration (not yet supported)");
            }
        }
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

        // Emit impl block with constructor if has state
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
                self.emit_line(&format!("impl {} {{", plugin.name));
                self.indent += 1;

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

                self.indent -= 1;
                self.emit_line("},");
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("}");

                self.indent -= 1;
                self.emit_line("}");
                self.emit_line("");
            }
        }

        // Emit helper functions outside VisitMut impl
        for item in &plugin.body {
            if let DecoratedPluginItem::Function(func) = item {
                if !func.name.starts_with("visit_") {
                    self.emit_plugin_item(item);
                }
            }
        }
    }

    fn emit_writer(&mut self, writer: &DecoratedWriter) {
        self.name = writer.name.clone();
        self.is_writer = true;

        // 1. Emit hoisted structs at module level (before the writer struct)
        for struct_decl in &writer.hoisted_structs {
            self.emit_struct(struct_decl);
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

        // Emit user-defined methods
        for item in &writer.body {
            self.emit_plugin_item(item);
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
        }
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
            self.emit_line("#[derive(Clone, Debug)]");
        }

        self.emit_line(&format!("struct {} {{", struct_decl.name));
        self.indent += 1;

        for field in &struct_decl.fields {
            let type_str = self.type_to_string(&field.ty);
            self.emit_line(&format!("{}: {},", field.name, type_str));
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
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
        self.emit_line(&format!("impl {} {{", impl_block.target));
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
        self.emit_function_with_visibility(func, false);
    }

    fn emit_function_with_visibility(&mut self, func: &DecoratedFnDecl, is_public: bool) {
        // Function signature
        let visibility = if is_public { "pub " } else { "" };
        let mut sig = format!("{}fn {}", visibility, func.name);

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

        // For SWC visitor methods (visit_mut_*) and writer helper methods, add &mut self as first parameter
        let needs_self = func.name.starts_with("visit_") ||
                        (self.is_writer && func.name.starts_with("extract_"));

        // Check if first parameter is already a self parameter
        let first_is_self = func.params.first()
            .map(|p| p.name == "self")
            .unwrap_or(false);

        if needs_self && !first_is_self {
            sig.push_str("&mut self");
            if !func.params.is_empty() {
                sig.push_str(", ");
            }
        }

        for (i, param) in func.params.iter().enumerate() {
            // Skip first parameter if it's self and we already added &mut self
            if needs_self && first_is_self && i == 0 {
                // Replace self parameter with &mut self
                sig.push_str("&mut self");
                if func.params.len() > 1 {
                    sig.push_str(", ");
                }
                continue;
            }

            if i > 0 {
                sig.push_str(", ");
            }
            sig.push_str(&param.name);
            sig.push_str(": ");
            // Apply lifetime to parameter types if the function has a lifetime parameter
            sig.push_str(&self.type_to_string_with_lifetime(&param.ty, needs_lifetime));
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
                self.output.push_str(" = ");
                self.emit_expr(&let_stmt.init);
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
                self.emit_block(&for_stmt.body);
                self.indent -= 1;
                self.emit_line("}");
            }

            DecoratedStmt::While(while_stmt) => {
                self.emit_indent();
                self.output.push_str("while ");
                self.emit_expr(&while_stmt.condition);
                self.output.push_str(" {\n");
                self.indent += 1;
                self.emit_block(&while_stmt.body);
                self.indent -= 1;
                self.emit_line("}");
            }

            DecoratedStmt::Loop(loop_body) => {
                self.emit_line("loop {");
                self.indent += 1;
                self.emit_block(loop_body);
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
                self.emit_line(&format!("// Traverse: {:?}", traverse));
            }

            DecoratedStmt::Function(func_decl) => {
                self.emit_line(&format!("// Nested function: {}", func_decl.name));
            }

            DecoratedStmt::Verbatim(verbatim) => {
                self.emit_line(&verbatim.code);
            }

            DecoratedStmt::CustomPropAssignment(assign) => {
                // TODO: Emit as self.state.set_custom_prop(node, "prop", value)
                // For now, just emit a comment
                self.emit_indent();
                self.output.push_str("// TODO: CustomPropAssignment: ");
                self.output.push_str(&assign.property);
                self.output.push_str("\n");
            }
        }
    }

    fn emit_if_stmt(&mut self, if_stmt: &DecoratedIfStmt) {
        self.emit_indent();

        if let Some(ref pattern) = if_stmt.pattern {
            // if-let statement
            self.output.push_str("if let ");
            self.emit_pattern(pattern);
            self.output.push_str(" = ");
            self.emit_expr(&if_stmt.condition);
        } else {
            // Regular if
            self.output.push_str("if ");
            self.emit_expr(&if_stmt.condition);
        }

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

    fn emit_match_stmt(&mut self, match_stmt: &DecoratedMatchStmt) {
        self.emit_indent();
        self.output.push_str("match ");
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

                // Check if the metadata contains parentheses (meaning it has a binding)
                if pattern.metadata.swc_pattern.contains('(') {
                    // It already has the binding, use it as-is
                    self.output.push_str(&pattern.metadata.swc_pattern);
                } else if let Some(ref inner_pattern) = inner {
                    // No binding in metadata, emit pattern with inner
                    self.output.push_str(&pattern.metadata.swc_pattern);
                    self.output.push('(');
                    self.emit_pattern(inner_pattern);
                    self.output.push(')');
                } else {
                    // No inner pattern, just emit the variant name
                    self.output.push_str(&pattern.metadata.swc_pattern);
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
            }

            DecoratedExprKind::Ident { name, ident_metadata } => {
                // Check for deref pattern
                if let Some(ref deref) = ident_metadata.deref_pattern {
                    self.output.push_str(deref);
                }
                self.output.push_str(name);

                // Check if we need .sym
                if ident_metadata.use_sym {
                    self.output.push_str(".sym");
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
                // Check for operator override
                if let Some(ref override_op) = unary_metadata.override_op {
                    self.output.push_str(override_op);
                } else {
                    self.output.push_str(&self.unary_op_to_string(op));
                }
                self.emit_expr(operand);
            }

            DecoratedExprKind::Member { object, property: _, optional, computed: _, is_path, field_metadata } => {
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
                if !field_metadata.read_conversion.is_empty() {
                    self.output.push_str(&field_metadata.read_conversion);
                }
            }

            DecoratedExprKind::Call(call) => {
                // Check if callee is a Member with read_conversion that already includes ()
                let skip_parens = if let DecoratedExprKind::Member { ref field_metadata, .. } = call.callee.kind {
                    !field_metadata.read_conversion.is_empty() &&
                    field_metadata.read_conversion.ends_with("()")
                } else {
                    false
                };

                self.emit_expr(&call.callee);

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

                    // Check if this is a string literal - add .into() for String fields
                    let is_string_literal = matches!(field_expr, Expr::Literal(Literal::String(_)));
                    if is_string_literal {
                        self.output.push('(');
                    }
                    self.emit_undecorated_expr(field_expr);
                    if is_string_literal {
                        self.output.push_str(").into()");
                    }
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
                self.output.push('&');
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
                self.emit_expr(right);
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
                // TODO: Emit as self.state.get_custom_prop(node, "prop")
                // For now, just emit a comment
                self.output.push_str("/* TODO: CustomPropAccess: ");
                self.output.push_str(&access.property);
                self.output.push_str(" */");
            }

            DecoratedExprKind::Closure(closure) => {
                // Emit Rust closure syntax: |params| body
                self.output.push('|');
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(param);
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
            _ => {
                // For other expression types, emit a placeholder
                self.output.push_str("/* complex expr */");
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
                // For simplicity, only handle simple identifier patterns
                if let crate::parser::Pattern::Ident(ref name) = let_stmt.pattern {
                    self.output.push_str(name);
                } else {
                    self.output.push_str("/* complex pattern */");
                }
                self.output.push_str(" = ");
                self.emit_parser_expr(&let_stmt.init);
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
            Expr::StructInit(struct_init) => {
                self.output.push_str(&struct_init.name);
                self.output.push_str(" { ");
                for (i, (field_name, field_expr)) in struct_init.fields.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.output.push_str(field_name);
                    self.output.push_str(": ");
                    self.emit_undecorated_expr(field_expr);
                }
                self.output.push_str(" }");
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
}
