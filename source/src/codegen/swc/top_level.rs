//! Top-Level Declaration Generation
//!
//! This module generates plugins, writers, and modules.

use super::SwcGenerator;
use crate::parser::*;
use crate::codegen::swc_decorator::*;
use crate::codegen::decorated_ast::*;

impl SwcGenerator {
    pub(super) fn gen_plugin(&mut self, plugin: &PluginDecl) {
        // Generate struct definitions first
        for item in &plugin.body {
            if let PluginItem::Struct(s) = item {
                self.gen_struct(s);
            }
        }

        // Generate enum definitions
        for item in &plugin.body {
            if let PluginItem::Enum(e) = item {
                self.gen_enum(e);
            }
        }

        // Check if there's a State struct defined and get its fields
        let state_struct = plugin.body.iter().find_map(|item| {
            if let PluginItem::Struct(s) = item {
                if s.name == "State" {
                    return Some(s);
                }
            }
            None
        });

        // Generate the main plugin struct
        self.emit_line(&format!("pub struct {} {{", plugin.name));
        self.indent += 1;
        if state_struct.is_some() {
            self.emit_line("pub state: State,");
        } else {
            self.emit_line("// Plugin state");
        }
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // Generate impl block with helper functions
        self.emit_line(&format!("impl {} {{", plugin.name));
        self.indent += 1;

        self.emit_line("pub fn new() -> Self {");
        self.indent += 1;
        if let Some(state) = state_struct {
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
        } else {
            self.emit_line("Self {}");
        }
        self.indent -= 1;
        self.emit_line("}");

        // Generate helper functions (non-visitor)
        for item in &plugin.body {
            if let PluginItem::Function(f) = item {
                if !self.is_visitor_method(f) {
                    self.emit_line("");
                    self.gen_helper_function(f);
                }
            }
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // Generate VisitMut impl
        self.emit_line(&format!("impl VisitMut for {} {{", plugin.name));
        self.indent += 1;

        for item in &plugin.body {
            if let PluginItem::Function(f) = item {
                if self.is_visitor_method(f) {
                    self.gen_visitor_method(f);
                }
            }
        }

        self.indent -= 1;
        self.emit_line("}");
    }
    pub(super) fn gen_writer(&mut self, writer: &WriterDecl) {
        // Mark that we're generating a writer (uses Visit, not VisitMut)
        self.is_writer = true;

        // Separate items by type
        let mut pre_hook: Option<&FnDecl> = None;
        let mut exit_hook: Option<&FnDecl> = None;
        let mut methods = Vec::new();
        let mut structs = Vec::new();

        for item in &writer.body {
            match item {
                PluginItem::PreHook(f) => pre_hook = Some(f),
                PluginItem::ExitHook(f) => exit_hook = Some(f),
                PluginItem::Function(f) => {
                    // Treat init() and finish() as aliases for pre/exit hooks
                    if f.name == "init" && pre_hook.is_none() {
                        pre_hook = Some(f);
                    } else if f.name == "finish" && exit_hook.is_none() {
                        exit_hook = Some(f);
                    } else {
                        methods.push(f);
                    }
                },
                PluginItem::Struct(s) => structs.push(s),
                _ => {}
            }
        }

        // For writers, use Visit instead of VisitMut
        self.emit_line("use swc_ecma_visit::Visit;");
        self.emit_line("");

        // Find State struct (if any) to flatten its fields into main struct
        let state_struct = structs.iter().find(|s| s.name == "State").cloned();

        // Generate other structs (not State, as we'll flatten it)
        for struct_decl in &structs {
            if struct_decl.name != "State" {
                self.gen_struct(struct_decl);
                self.emit_line("");
            }
        }

        // Generate the writer struct with CodeBuilder + State fields
        self.emit_line(&format!("pub struct {} {{", writer.name));
        self.indent += 1;
        self.emit_line("output: String,");
        self.emit_line("indent_level: usize,");

        // Flatten State struct fields into main struct
        if let Some(state) = state_struct {
            for field in &state.fields {
                let rust_type = self.type_to_rust(&field.ty);
                self.emit_line(&format!("{}: {},", field.name, rust_type));
            }
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // Generate impl block
        self.emit_line(&format!("impl {} {{", writer.name));
        self.indent += 1;

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

        // CodeBuilder methods
        self.emit_line("fn append(&mut self, s: &str) {");
        self.indent += 1;
        self.emit_line("self.output.push_str(s);");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        self.emit_line("fn newline(&mut self) {");
        self.indent += 1;
        self.emit_line("self.output.push('\\n');");
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
        self.emit_line("self.indent_level -= 1;");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // Generate finish() method
        if let Some(exit_fn) = exit_hook {
            // Use exit hook as finish method
            self.emit_line("/// Finalize output (from exit hook)");
            self.emit_line("pub fn finish(mut self) -> String {");
            self.indent += 1;

            // Generate exit hook body
            self.gen_block(&exit_fn.body);

            // Always return the output at the end
            self.emit_line("self.output");
            self.indent -= 1;
            self.emit_line("}");
        } else {
            // Default finish
            self.emit_line("pub fn finish(self) -> String {");
            self.indent += 1;
            self.emit_line("self.output");
            self.indent -= 1;
            self.emit_line("}");
        }
        self.emit_line("");

        // Emit comment about pre-hook if present (not supported in SWC)
        if pre_hook.is_some() {
            self.emit_line("// Note: pre() hook not supported in SWC (no source access)");
            self.emit_line("");
        }

        // Generate helper functions (non-visitor methods)
        for method in &methods {
            if !method.name.starts_with("visit_") {
                self.gen_helper_function(method);
                self.emit_line("");
            }
        }

        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // Generate Visit impl
        self.emit_line(&format!("impl Visit for {} {{", writer.name));
        self.indent += 1;

        for method in methods {
            if method.name.starts_with("visit_") {
                self.gen_visit_method(method);
            }
        }

        self.indent -= 1;
        self.emit_line("}");
    }

    /// Generate decorated plugin
    pub(super) fn gen_decorated_plugin(&mut self, plugin: &DecoratedPlugin) {
        self.emit_line(&format!("pub struct {} {{}}", plugin.name));
        self.emit_line("");
        self.emit_line(&format!("impl {} {{", plugin.name));
        self.indent += 1;
        self.emit_indent();
        self.emit_line("pub fn new() -> Self { Self {} }");
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");

        // Generate VisitMut implementation
        self.emit_line(&format!("impl VisitMut for {} {{", plugin.name));
        self.indent += 1;

        // TODO: Generate visitor methods from decorated plugin items
        for item in &plugin.body {
            match item {
                DecoratedPluginItem::Function(func) => {
                    self.gen_decorated_visitor_method(func);
                }
                _ => {
                    // TODO: Handle other items
                }
            }
        }

        self.indent -= 1;
        self.emit_line("}");
    }

    /// Generate decorated writer
    pub(super) fn gen_decorated_writer(&mut self, writer: &DecoratedWriter) {
        // TODO: Similar to plugin but uses Visit instead of VisitMut
        self.emit_line(&format!("// TODO: Decorated writer {}", writer.name));
    }

    /// Generate a visitor method from decorated function
    pub(super) fn gen_module(&mut self, module: &ModuleDecl) {
        // Generate standalone module with pub exports
        self.emit_line("//! Generated by ReluxScript compiler");
        self.emit_line("//! Do not edit manually");
        self.emit_line("");

        // Generate structs
        for item in &module.items {
            if let PluginItem::Struct(s) = item {
                self.gen_struct(s);
            }
        }

        // Generate enums
        for item in &module.items {
            if let PluginItem::Enum(e) = item {
                self.gen_enum(e);
            }
        }

        // Generate functions (only pub functions are exported)
        for item in &module.items {
            if let PluginItem::Function(f) = item {
                self.gen_helper_function(f);
                self.emit_line("");
            }
        }
    }
    
}
