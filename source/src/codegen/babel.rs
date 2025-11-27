//! Babel (JavaScript) code generator for ReluxScript

use crate::parser::*;
use crate::mapping::{get_node_mapping, get_node_mapping_by_visitor, get_field_mapping};

/// Context for tracking traverse block state
#[derive(Clone)]
struct TraverseContext {
    /// State variable names defined in this traverse block
    state_vars: std::collections::HashSet<String>,
    /// Unique ID for this traverse block (for generating unique visitor names)
    id: usize,
    /// The path variable name that should be used for traverse calls
    path_var: String,
}

/// Tracks whether a variable holds a path or a node
#[derive(Clone, Debug, PartialEq)]
enum VarKind {
    /// Variable holds a Babel path
    Path,
    /// Variable holds a node, with optional info about which path and property it came from
    Node {
        /// The path variable this node came from (e.g., "path")
        from_path: Option<String>,
        /// The property accessed (e.g., "body" for path.node.body)
        property: Option<String>,
    },
    /// Variable is a loop iteration variable over a node array
    LoopItem {
        /// The path that contains the array
        from_path: String,
        /// The array property (e.g., "body")
        array_prop: String,
        /// The loop index variable name
        index_var: String,
    },
}

/// Generator for Babel plugin JavaScript code
pub struct BabelGenerator {
    output: String,
    indent: usize,
    /// Maps parameter names to their aliases (e.g., "func" -> "node")
    param_aliases: std::collections::HashMap<String, String>,
    /// Stack of traverse contexts (for nested traverse blocks)
    traverse_stack: Vec<TraverseContext>,
    /// Counter for generating unique traverse visitor names
    traverse_counter: usize,
    /// Tracks what kind of value each variable holds (path vs node)
    var_kinds: std::collections::HashMap<String, VarKind>,
    /// Counter for generating unique loop index variables
    loop_index_counter: usize,
    /// Counter for generating unique if-let temporary variables
    iflet_counter: usize,
    /// Whether the codegen module is used (requires @babel/generator import)
    uses_codegen: bool,
    /// Track loop nesting depth (for correct return handling in match expressions)
    loop_depth: usize,
}

impl BabelGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            indent: 0,
            param_aliases: std::collections::HashMap::new(),
            traverse_stack: Vec::new(),
            traverse_counter: 0,
            var_kinds: std::collections::HashMap::new(),
            loop_index_counter: 0,
            iflet_counter: 0,
            uses_codegen: false,
            loop_depth: 0,
        }
    }

    /// Register a variable as holding a path
    fn register_path_var(&mut self, name: &str) {
        self.var_kinds.insert(name.to_string(), VarKind::Path);
    }

    /// Register a variable as holding a node
    fn register_node_var(&mut self, name: &str, from_path: Option<&str>, property: Option<&str>) {
        self.var_kinds.insert(name.to_string(), VarKind::Node {
            from_path: from_path.map(|s| s.to_string()),
            property: property.map(|s| s.to_string()),
        });
    }

    /// Register a variable as a loop item over a node array
    fn register_loop_item(&mut self, name: &str, from_path: &str, array_prop: &str, index_var: &str) {
        self.var_kinds.insert(name.to_string(), VarKind::LoopItem {
            from_path: from_path.to_string(),
            array_prop: array_prop.to_string(),
            index_var: index_var.to_string(),
        });
    }

    /// Get the VarKind for a variable
    fn get_var_kind(&self, name: &str) -> Option<&VarKind> {
        self.var_kinds.get(name)
    }

    /// Generate the path expression to traverse for a given variable and property
    fn gen_traverse_path(&self, var_name: &str, property: &str) -> String {
        match self.get_var_kind(var_name) {
            Some(VarKind::Path) => {
                // Direct path variable - just get the property
                format!("{}.get('{}')", var_name, property)
            }
            Some(VarKind::Node { from_path: Some(path), property: Some(node_prop) }) => {
                // Node from a path property - chain the gets
                format!("{}.get('{}').get('{}')", path, node_prop, property)
            }
            Some(VarKind::Node { from_path: Some(path), property: None }) => {
                // Node directly from path.node
                format!("{}.get('{}')", path, property)
            }
            Some(VarKind::LoopItem { from_path, array_prop, index_var }) => {
                // Loop item - need to index into the array, then get the property
                format!("{}.get(`{}.${{{}}}.{}`)", from_path, array_prop, index_var, property)
            }
            _ => {
                // Unknown - fall back to direct traverse (may not work)
                format!("{}.get('{}')", var_name, property)
            }
        }
    }

    /// Get a unique loop index variable name
    fn next_loop_index(&mut self) -> String {
        let name = format!("__idx_{}", self.loop_index_counter);
        self.loop_index_counter += 1;
        name
    }

    /// Check if we're currently inside a traverse block
    fn in_traverse(&self) -> bool {
        !self.traverse_stack.is_empty()
    }

    /// Check if a variable is a state variable in the current traverse
    fn is_traverse_state_var(&self, name: &str) -> bool {
        if let Some(ctx) = self.traverse_stack.last() {
            ctx.state_vars.contains(name)
        } else {
            false
        }
    }

    /// Get a unique visitor name for a traverse block
    fn next_visitor_name(&mut self) -> String {
        let name = format!("__visitor_{}", self.traverse_counter);
        self.traverse_counter += 1;
        name
    }

    /// Generate JavaScript code for a Babel plugin
    pub fn generate(&mut self, program: &Program) -> String {
        // First, generate the plugin body to determine what imports are needed
        match &program.decl {
            TopLevelDecl::Plugin(plugin) => self.gen_plugin(plugin),
            TopLevelDecl::Writer(writer) => self.gen_writer(writer),
            TopLevelDecl::Interface(_iface) => {
                // TODO: Generate TypeScript interface
            }
            TopLevelDecl::Module(module) => self.gen_module(module),
        }

        // Now prepend the imports based on what was used
        let body = std::mem::take(&mut self.output);
        let mut imports = String::new();

        // Generate use statements (require)
        if !program.uses.is_empty() {
            let mut temp_gen = BabelGenerator::new();
            temp_gen.gen_use_statements(&program.uses);
            imports.push_str(&temp_gen.output);
        }

        // If codegen module was used, add the @babel/generator import
        if self.uses_codegen {
            if !imports.is_empty() && !imports.trim_end().ends_with('\n') {
                imports.push('\n');
            }
            if imports.is_empty() || !imports.contains("// Module imports") {
                imports.push_str("// Module imports\n");
            }
            imports.push_str("const generate = require('@babel/generator').default;\n\n");
        }

        // Combine imports and body
        imports.push_str(&body);
        imports
    }

    fn emit(&mut self, s: &str) {
        self.output.push_str(s);
    }

    fn emit_indent(&mut self) {
        for _ in 0..self.indent {
            self.output.push_str("  ");
        }
    }

    fn emit_line(&mut self, s: &str) {
        self.emit_indent();
        self.emit(s);
        self.emit("\n");
    }

    /// Generate require() statements for use declarations
    fn gen_use_statements(&mut self, uses: &[UseStmt]) {
        if uses.is_empty() {
            return;
        }

        self.emit_line("// Module imports");

        for use_stmt in uses {
            self.gen_use_statement(use_stmt);
        }

        self.emit_line("");
    }

    /// Generate a single require() statement
    fn gen_use_statement(&mut self, use_stmt: &UseStmt) {
        let is_file_module = use_stmt.path.starts_with("./") || use_stmt.path.starts_with("../");

        if is_file_module {
            // File module: convert .lux to .js
            let js_path = use_stmt.path.replace(".lux", ".js");

            // Determine the variable name
            let var_name = if let Some(alias) = &use_stmt.alias {
                alias.clone()
            } else if !use_stmt.imports.is_empty() {
                // If we have specific imports, we'll use destructuring
                String::new() // Will be handled below
            } else {
                // Extract module name from path (e.g., "./helpers.lux" -> "helpers")
                self.extract_module_name_from_path(&use_stmt.path)
            };

            if !use_stmt.imports.is_empty() {
                // Destructuring import: const { foo, bar } = require('./helpers.js');
                let imports = use_stmt.imports.join(", ");

                if var_name.is_empty() {
                    // Only destructuring, no alias
                    self.emit_line(&format!("const {{ {} }} = require('{}');", imports, js_path));
                } else {
                    // Both alias and destructuring (questionable design, but we support it)
                    self.emit_line(&format!("const {} = require('{}');", var_name, js_path));
                    self.emit_line(&format!("const {{ {} }} = {};", imports, var_name));
                }
            } else {
                // Simple import: const helpers = require('./helpers.js');
                self.emit_line(&format!("const {} = require('{}');", var_name, js_path));
            }
        } else {
            // Built-in module
            self.gen_builtin_module(use_stmt);
        }
    }

    /// Generate require() for built-in modules (fs, json, path)
    fn gen_builtin_module(&mut self, use_stmt: &UseStmt) {
        match use_stmt.path.as_str() {
            "fs" => {
                let var_name = use_stmt.alias.as_ref().unwrap_or(&use_stmt.path);
                self.emit_line(&format!("const {} = require('fs');", var_name));
            }
            "json" => {
                // JSON is built-in to JavaScript, no require needed
                // But we might use it as a namespace for json::stringify, etc.
                let var_name = use_stmt.alias.as_ref().unwrap_or(&use_stmt.path);
                self.emit_line(&format!("const {} = {{", var_name));
                self.indent += 1;
                self.emit_line("stringify: (obj) => JSON.stringify(obj, null, 2),");
                self.emit_line("parse: (str) => JSON.parse(str)");
                self.indent -= 1;
                self.emit_line("};");
            }
            "path" => {
                let var_name = use_stmt.alias.as_ref().unwrap_or(&use_stmt.path);
                self.emit_line(&format!("const {} = require('path');", var_name));
            }
            "parser" => {
                // Parser module for runtime AST parsing
                let var_name = use_stmt.alias.as_ref().unwrap_or(&use_stmt.path);
                self.emit_line("const babel = require('@babel/core');");
                self.emit_line("const fs = require('fs');");
                self.emit_line(&format!("const {} = {{", var_name));
                self.indent += 1;

                // parser::parse_file(path: &Str) -> Result<Program, Str>
                self.emit_line("parse_file: (path) => {");
                self.indent += 1;
                self.emit_line("try {");
                self.indent += 1;
                self.emit_line("const code = fs.readFileSync(path, 'utf-8');");
                self.emit_line("const ast = babel.parseSync(code, {");
                self.indent += 1;
                self.emit_line("filename: path,");
                self.emit_line("presets: ['@babel/preset-typescript'],");
                self.emit_line("plugins: ['@babel/plugin-syntax-jsx'],");
                self.indent -= 1;
                self.emit_line("});");
                self.emit_line("return { ok: true, value: ast };");
                self.indent -= 1;
                self.emit_line("} catch (error) {");
                self.indent += 1;
                self.emit_line("return { ok: false, error: error.message };");
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("},");

                // parser::parse(code: &Str) -> Result<Program, Str>
                self.emit_line("parse: (code) => {");
                self.indent += 1;
                self.emit_line("try {");
                self.indent += 1;
                self.emit_line("const ast = babel.parseSync(code, {");
                self.indent += 1;
                self.emit_line("presets: ['@babel/preset-typescript'],");
                self.emit_line("plugins: ['@babel/plugin-syntax-jsx'],");
                self.indent -= 1;
                self.emit_line("});");
                self.emit_line("return { ok: true, value: ast };");
                self.indent -= 1;
                self.emit_line("} catch (error) {");
                self.indent += 1;
                self.emit_line("return { ok: false, error: error.message };");
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("},");

                // parser::parse_with_syntax(code: &Str, syntax: Syntax) -> Result<Program, Str>
                self.emit_line("parse_with_syntax: (code, syntax) => {");
                self.indent += 1;
                self.emit_line("try {");
                self.indent += 1;
                self.emit_line("let options = {};");
                self.emit_line("if (syntax === 'TypeScript') {");
                self.indent += 1;
                self.emit_line("options = {");
                self.indent += 1;
                self.emit_line("presets: ['@babel/preset-typescript'],");
                self.emit_line("plugins: ['@babel/plugin-syntax-jsx'],");
                self.indent -= 1;
                self.emit_line("};");
                self.indent -= 1;
                self.emit_line("} else if (syntax === 'JSX') {");
                self.indent += 1;
                self.emit_line("options = {");
                self.indent += 1;
                self.emit_line("plugins: ['@babel/plugin-syntax-jsx'],");
                self.indent -= 1;
                self.emit_line("};");
                self.indent -= 1;
                self.emit_line("} else {");
                self.indent += 1;
                self.emit_line("options = {};");
                self.indent -= 1;
                self.emit_line("}");
                self.emit_line("const ast = babel.parseSync(code, options);");
                self.emit_line("return { ok: true, value: ast };");
                self.indent -= 1;
                self.emit_line("} catch (error) {");
                self.indent += 1;
                self.emit_line("return { ok: false, error: error.message };");
                self.indent -= 1;
                self.emit_line("}");
                self.indent -= 1;
                self.emit_line("},");

                self.indent -= 1;
                self.emit_line("};");
            }
            "codegen" => {
                // Codegen module - no require needed, we inject @babel/generator automatically
                // when codegen functions are used
            }
            other => {
                // Unknown module - just try to require it
                let var_name = use_stmt.alias.as_ref().unwrap_or(&use_stmt.path);
                self.emit_line(&format!("const {} = require('{}');", var_name, other));
            }
        }
    }

    /// Extract module name from file path
    /// "./helpers.lux" -> "helpers"
    /// "./utils/types.lux" -> "types"
    /// "../foo/bar.lux" -> "bar"
    fn extract_module_name_from_path(&self, path: &str) -> String {
        // Remove .lux extension
        let without_ext = path.trim_end_matches(".lux");

        // Get the last component after '/'
        let name = without_ext.split('/').last().unwrap_or(without_ext);

        name.to_string()
    }

    fn gen_plugin(&mut self, plugin: &PluginDecl) {
        // Generate Babel plugin module structure
        self.emit_line("// Generated by ReluxScript compiler");
        self.emit_line("// Do not edit manually");
        self.emit_line("");
        self.emit_line("module.exports = function({ types: t }) {");
        self.indent += 1;

        // Generate helper structs as classes
        for item in &plugin.body {
            if let PluginItem::Struct(s) = item {
                self.gen_struct_class(s);
            }
        }

        // Generate helper functions (but not hooks)
        for item in &plugin.body {
            if let PluginItem::Function(f) = item {
                if !f.name.starts_with("visit_") {
                    self.gen_helper_function(f);
                }
            }
        }

        // Generate pre hook function if present
        for item in &plugin.body {
            if let PluginItem::PreHook(f) = item {
                self.emit_line("");
                self.emit_line("// Pre-transformation hook");
                self.gen_helper_function(f);
            }
        }

        // Generate exit hook function if present
        for item in &plugin.body {
            if let PluginItem::ExitHook(f) = item {
                self.emit_line("");
                self.emit_line("// Post-transformation hook");
                self.gen_helper_function(f);
            }
        }

        // Generate visitor state initialization
        self.emit_line("");
        self.emit_line("let state = {};");
        self.emit_line("");

        // Generate return with visitor
        self.emit_line("return {");
        self.indent += 1;

        // Add pre() hook if present
        let has_pre_hook = plugin.body.iter()
            .any(|item| matches!(item, PluginItem::PreHook(_)));

        if has_pre_hook {
            self.emit_line("pre(file) {");
            self.indent += 1;
            self.emit_line("pre(file);");
            self.indent -= 1;
            self.emit_line("},");
            self.emit_line("");
        }

        self.emit_line("visitor: {");
        self.indent += 1;

        // Add Program exit hook if present
        let has_exit_hook = plugin.body.iter()
            .any(|item| matches!(item, PluginItem::ExitHook(_)));

        if has_exit_hook {
            self.emit_line("Program: {");
            self.indent += 1;
            self.emit_line("exit(path, state) {");
            self.indent += 1;
            self.emit_line("exit(path.node, state);");
            self.indent -= 1;
            self.emit_line("}");
            self.indent -= 1;
            self.emit_line("},");
            self.emit_line("");
        }

        // Generate visitor methods
        let mut first = true;
        for item in &plugin.body {
            if let PluginItem::Function(f) = item {
                if f.name.starts_with("visit_") {
                    if !first {
                        self.emit(",\n");
                    }
                    first = false;
                    self.gen_visitor_method(f);
                }
            }
        }

        self.emit("\n");
        self.indent -= 1;
        self.emit_line("}");
        self.indent -= 1;
        self.emit_line("};");

        self.indent -= 1;
        self.emit_line("};");
    }

    fn gen_writer(&mut self, writer: &WriterDecl) {
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

        self.emit_line("// Generated by ReluxScript compiler");
        self.emit_line("// Do not edit manually");
        self.emit_line("");

        // Generate as Babel plugin that returns visitor + hooks
        self.emit(&format!("module.exports = function({{ types: t }}) {{\n"));
        self.indent += 1;

        // Generate CodeBuilder initialization
        self.emit_line("const builder = {");
        self.indent += 1;
        self.emit_line("_output: [],");
        self.emit_line("_indentLevel: 0,");
        self.emit_line("append(s) { this._output.push(s); },");
        self.emit_line("newline() { this._output.push('\\n'); },");
        self.emit_line("indent() { this._indentLevel++; },");
        self.emit_line("dedent() { this._indentLevel--; },");
        self.emit_line("toString() { return this._output.join(''); }");
        self.indent -= 1;
        self.emit_line("};");
        self.emit_line("");

        // Generate structs if any
        for struct_decl in structs {
            self.gen_struct_class(struct_decl);
            self.emit_line("");
        }

        // Generate helper functions (non-visitor methods)
        for method in &methods {
            if !method.name.starts_with("visit_") {
                self.gen_helper_function(method);
                self.emit_line("");
            }
        }

        // Generate pre hook if present
        if let Some(pre_fn) = pre_hook {
            self.emit_line("// Pre-hook");
            self.gen_helper_function(pre_fn);
            self.emit_line("");
        }

        // Generate exit hook if present
        if let Some(exit_fn) = exit_hook {
            self.emit_line("// Exit hook");
            self.gen_helper_function(exit_fn);
            self.emit_line("");
        }

        // Generate return object with visitor + hooks
        self.emit_line("return {");
        self.indent += 1;

        // Add pre() if present
        if let Some(pre_fn) = pre_hook {
            self.emit_line("pre(file) {");
            self.indent += 1;
            self.emit_line(&format!("{}(file);", pre_fn.name));
            self.indent -= 1;
            self.emit_line("},");
            self.emit_line("");
        }

        // Generate visitor object
        self.emit_line("visitor: {");
        self.indent += 1;

        // Add exit hook wrapper in Program visitor if present
        if let Some(exit_fn) = exit_hook {
            self.emit_line("Program: {");
            self.indent += 1;
            self.emit_line("exit(path, state) {");
            self.indent += 1;
            self.emit_line(&format!("{}(path.node, state, builder);", exit_fn.name));
            self.indent -= 1;
            self.emit_line("}");
            self.indent -= 1;
            self.emit_line("},");
            self.emit_line("");
        }

        // Generate visitor methods
        for method in methods {
            if method.name.starts_with("visit_") {
                // Convert visit_xxx to PascalCase node type
                let node_type = self.visitor_method_to_node_type(&method.name);
                self.emit_indent();
                self.emit(&format!("{}(path) {{\n", node_type));
                self.indent += 1;

                // Alias the node
                self.emit_line("const node = path.node;");

                // Generate method body with builder in scope
                self.gen_block(&method.body);

                self.indent -= 1;
                self.emit_indent();
                self.emit("},\n");
            }
        }

        self.indent -= 1;
        self.emit_line("}");  // Close visitor

        self.indent -= 1;
        self.emit_line("};");  // Close return object

        self.indent -= 1;
        self.emit_line("};");  // Close module.exports function
    }

    fn gen_module(&mut self, module: &ModuleDecl) {
        // Generate standalone module with exports
        self.emit_line("// Generated by ReluxScript compiler");
        self.emit_line("// Do not edit manually");
        self.emit_line("");

        // Collect exported items
        let mut exports = Vec::new();

        // Generate all items
        for item in &module.items {
            match item {
                PluginItem::Function(f) => {
                    self.gen_helper_function(f);
                    if f.is_pub {
                        exports.push(f.name.clone());
                    }
                }
                PluginItem::Struct(s) => {
                    self.gen_struct_class(s);
                    // Structs are always exported if defined at module level
                    exports.push(s.name.clone());
                }
                PluginItem::Enum(_e) => {
                    // TODO: Generate enum
                }
                PluginItem::Impl(_impl) => {
                    // Impl blocks don't export anything directly
                }
                PluginItem::PreHook(_) | PluginItem::ExitHook(_) => {
                    // Hooks are only valid in plugins, not modules
                    // This should not happen if the parser/semantic analyzer is correct
                }
            }
        }

        // Generate module.exports
        if !exports.is_empty() {
            self.emit_line("");
            self.emit_line("module.exports = {");
            self.indent += 1;
            for (i, name) in exports.iter().enumerate() {
                let comma = if i < exports.len() - 1 { "," } else { "" };
                self.emit_line(&format!("{}{}", name, comma));
            }
            self.indent -= 1;
            self.emit_line("};");
        }
    }

    fn gen_struct_class(&mut self, s: &StructDecl) {
        self.emit_line("");
        self.emit(&format!("  class {} {{\n", s.name));
        self.indent += 1;

        // Constructor
        let params: Vec<&str> = s.fields.iter().map(|f| f.name.as_str()).collect();
        self.emit(&format!("  constructor({}) {{\n", params.join(", ")));
        self.indent += 1;
        for field in &s.fields {
            self.emit_line(&format!("this.{} = {};", field.name, field.name));
        }
        self.indent -= 1;
        self.emit_line("}");

        self.indent -= 1;
        self.emit_line("}");
    }

    fn gen_helper_function(&mut self, f: &FnDecl) {
        self.emit_line("");
        // Filter out 'self' parameter - JavaScript methods don't have explicit 'this' parameter
        let params: Vec<String> = f.params.iter()
            .filter(|p| p.name != "self")
            .map(|p| p.name.clone())
            .collect();
        self.emit_line(&format!("function {}({}) {{", f.name, params.join(", ")));
        self.indent += 1;
        self.gen_block_with_implicit_return(&f.body);
        self.indent -= 1;
        self.emit_line("}");
    }

    fn gen_method(&mut self, f: &FnDecl) {
        // Filter out 'self' parameter - JavaScript methods don't have explicit 'this' parameter
        let params: Vec<String> = f.params.iter()
            .filter(|p| p.name != "self")
            .map(|p| p.name.clone())
            .collect();
        self.emit_line(&format!("{}({}) {{", f.name, params.join(", ")));
        self.indent += 1;
        self.gen_block_with_implicit_return(&f.body);
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }

    fn gen_visitor_method(&mut self, f: &FnDecl) {
        // Convert visit_call_expression to CallExpression
        let node_type = self.visitor_name_to_babel_type(&f.name);

        self.emit_indent();
        self.emit(&format!("{}(path) {{", node_type));
        self.emit("\n");
        self.indent += 1;

        // Add node alias and track the parameter rename
        self.emit_line("const node = path.node;");

        // Register path and node variable kinds
        self.register_path_var("path");
        self.register_node_var("node", Some("path"), None);

        // Set up parameter alias: original param name -> "node"
        if !f.params.is_empty() {
            let original_name = &f.params[0].name;
            self.param_aliases.insert(original_name.clone(), "node".to_string());
            // Also register the original name as a node
            self.register_node_var(original_name, Some("path"), None);
        }

        // Set up alias for context parameter (second param) -> "path"
        if f.params.len() >= 2 {
            let ctx_param_name = &f.params[1].name;
            self.param_aliases.insert(ctx_param_name.clone(), "path".to_string());
        }

        self.gen_block(&f.body);

        // Clear the aliases after generating the method
        if !f.params.is_empty() {
            self.param_aliases.remove(&f.params[0].name);
        }
        if f.params.len() >= 2 {
            self.param_aliases.remove(&f.params[1].name);
        }

        self.indent -= 1;
        self.emit_indent();
        self.emit("}");
    }

    fn visitor_name_to_babel_type(&self, name: &str) -> String {
        // Use mapping module: visit_call_expression -> CallExpression
        if let Some(mapping) = get_node_mapping_by_visitor(name) {
            return mapping.babel.to_string();
        }

        // Fallback: convert snake_case to PascalCase
        let stripped = name.strip_prefix("visit_").unwrap_or(name);
        stripped
            .split('_')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                }
            })
            .collect()
    }

    fn gen_block(&mut self, block: &Block) {
        self.gen_block_inner(block, false);
    }

    fn gen_block_with_implicit_return(&mut self, block: &Block) {
        self.gen_block_inner(block, true);
    }

    fn gen_block_inner(&mut self, block: &Block, implicit_return: bool) {
        let num_stmts = block.stmts.len();
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i == num_stmts - 1;

            // If this is the last statement and we want implicit returns
            if is_last && implicit_return {
                match stmt {
                    Stmt::Expr(expr_stmt) => {
                        // Generate return for implicit return expression
                        self.emit_indent();
                        self.emit("return ");
                        self.gen_expr(&expr_stmt.expr);
                        self.emit(";\n");
                        continue;
                    }
                    Stmt::If(if_stmt) => {
                        // Generate if with implicit returns in branches
                        self.gen_if_stmt_with_implicit_return(if_stmt);
                        continue;
                    }
                    _ => {}
                }
            }

            self.gen_stmt(stmt);
        }
    }

    fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_stmt) => {
                // Check if the initializer uses the ? operator (Try expression)
                if let Expr::Try(inner) = &let_stmt.init {
                    // Generate proper Result unwrapping in JavaScript
                    // const result = func();
                    // if (!result.ok) { return { ok: false, error: result.error }; }
                    // const varName = result.value;

                    let temp_var = "__result";

                    // Step 1: Call the function and store in temp variable
                    self.emit_indent();
                    self.emit(&format!("const {} = ", temp_var));
                    self.gen_expr(inner);
                    self.emit(";\n");

                    // Step 2: Check if result is error and early return
                    self.emit_indent();
                    self.emit(&format!("if (!{}.ok) {{\n", temp_var));
                    self.indent += 1;
                    self.emit_indent();
                    self.emit(&format!("return {{ ok: false, error: {}.error }};\n", temp_var));
                    self.indent -= 1;
                    self.emit_indent();
                    self.emit("}\n");

                    // Step 3: Extract the value
                    self.emit_indent();
                    if let_stmt.mutable {
                        self.emit("let ");
                    } else {
                        self.emit("const ");
                    }
                    self.gen_pattern(&let_stmt.pattern);
                    self.emit(&format!(" = {}.value;\n", temp_var));
                } else {
                    // Normal let statement
                    self.emit_indent();
                    if let_stmt.mutable {
                        self.emit("let ");
                    } else {
                        self.emit("const ");
                    }
                    self.gen_pattern(&let_stmt.pattern);
                    self.emit(" = ");
                    self.gen_expr(&let_stmt.init);
                    self.emit(";\n");
                }

                // Track var_kind for the new variable (only for simple identifier patterns)
                // If initializing from another variable, copy its var_kind
                if let Pattern::Ident(name) = &let_stmt.pattern {
                    if let Expr::Ident(ident) = &let_stmt.init {
                        if let Some(var_kind) = self.get_var_kind(&ident.name).cloned() {
                            self.var_kinds.insert(name.clone(), var_kind);
                        }
                    } else if let Expr::Call(call) = &let_stmt.init {
                        // Check for .clone() calls
                        if let Expr::Member(mem) = call.callee.as_ref() {
                            if mem.property == "clone" && call.args.is_empty() {
                                if let Expr::Ident(obj) = mem.object.as_ref() {
                                    if let Some(var_kind) = self.get_var_kind(&obj.name).cloned() {
                                        self.var_kinds.insert(name.clone(), var_kind);
                                    }
                                }
                            }
                        }
                    }
                }
            }
            Stmt::Const(const_stmt) => {
                self.emit_indent();
                self.emit("const ");
                self.emit(&const_stmt.name);
                self.emit(" = ");
                self.gen_expr(&const_stmt.init);
                self.emit(";\n");
            }
            Stmt::Expr(expr_stmt) => {
                // Check if this is Ok(value) or Err(error) as an implicit return
                if let Expr::Call(call) = &expr_stmt.expr {
                    if let Expr::Ident(ident) = call.callee.as_ref() {
                        if ident.name == "Ok" {
                            // Convert Ok(value) to return { ok: true, value: value }
                            self.emit_indent();
                            self.emit("return { ok: true, value: ");
                            if let Some(arg) = call.args.first() {
                                self.gen_expr(arg);
                            } else {
                                self.emit("undefined");
                            }
                            self.emit(" };\n");
                            return;
                        } else if ident.name == "Err" {
                            // Convert Err(error) to return { ok: false, error: error }
                            self.emit_indent();
                            self.emit("return { ok: false, error: ");
                            if let Some(arg) = call.args.first() {
                                self.gen_expr(arg);
                            } else {
                                self.emit("undefined");
                            }
                            self.emit(" };\n");
                            return;
                        }
                    }
                }
                // Check if this is a match expression (which returns a value via IIFE)
                if matches!(&expr_stmt.expr, Expr::Match(_)) {
                    self.emit_indent();
                    self.emit("return ");
                    self.gen_expr(&expr_stmt.expr);
                    self.emit(";\n");
                    return;
                }
                // Regular expression statement
                self.emit_indent();
                self.gen_expr(&expr_stmt.expr);
                self.emit(";\n");
            }
            Stmt::If(if_stmt) => {
                if let Some(pattern) = &if_stmt.pattern {
                    // if-let pattern matching
                    self.gen_if_let_stmt(if_stmt, pattern);
                } else {
                    // Regular if statement
                    self.emit_indent();
                    self.emit("if (");
                    self.gen_expr(&if_stmt.condition);
                    self.emit(") {\n");
                    self.indent += 1;
                    self.gen_block(&if_stmt.then_branch);
                    self.indent -= 1;

                    for (cond, block) in &if_stmt.else_if_branches {
                        self.emit_indent();
                        self.emit("} else if (");
                        self.gen_expr(cond);
                        self.emit(") {\n");
                        self.indent += 1;
                        self.gen_block(block);
                        self.indent -= 1;
                    }

                    if let Some(else_block) = &if_stmt.else_branch {
                        self.emit_indent();
                        self.emit("} else {\n");
                        self.indent += 1;
                        self.gen_block(else_block);
                        self.indent -= 1;
                    }

                    self.emit_line("}");
                }
            }
            Stmt::For(for_stmt) => {
                // Check if we're iterating over a range (e.g., for i in 0..10)
                if let Expr::Range(range) = &for_stmt.iter {
                    // Generate C-style for loop: for (let i = start; i < end; i++)
                    let var_name = if let Pattern::Ident(name) = &for_stmt.pattern {
                        name.clone()
                    } else {
                        "i".to_string()
                    };

                    self.emit_indent();
                    self.emit(&format!("for (let {} = ", var_name));
                    if let Some(start) = &range.start {
                        self.gen_expr(start);
                    } else {
                        self.emit("0");
                    }
                    self.emit(&format!("; {} < ", var_name));
                    if let Some(end) = &range.end {
                        self.gen_expr(end);
                    } else {
                        self.emit("Infinity");
                    }
                    self.emit(&format!("; {}++) {{\n", var_name));
                    self.indent += 1;
                    self.loop_depth += 1;
                    self.gen_block(&for_stmt.body);
                    self.loop_depth -= 1;
                    self.indent -= 1;
                    self.emit_line("}");
                    return;
                }

                // Check if we're iterating over a node property (e.g., node.body or &node.body)
                // This is needed for proper path tracking when using traverse inside the loop

                // Unwrap reference if present
                let iter_expr = match &for_stmt.iter {
                    Expr::Unary(unary) if matches!(unary.op, UnaryOp::Ref | UnaryOp::RefMut) => {
                        unary.operand.as_ref()
                    }
                    other => other,
                };

                let loop_item_info = if let Expr::Member(mem) = iter_expr {
                    if let Expr::Ident(obj_ident) = mem.object.as_ref() {
                        // Check if the object is a node variable
                        let obj_name = if let Some(alias) = self.param_aliases.get(&obj_ident.name) {
                            alias.clone()
                        } else {
                            obj_ident.name.clone()
                        };

                        // Find the path this node came from
                        match self.get_var_kind(&obj_name).cloned() {
                            Some(VarKind::Node { from_path: Some(path), property: Some(node_prop) }) => {
                                // Node from path.node.property - chain the properties
                                Some((path, format!("{}.{}", node_prop, mem.property)))
                            }
                            Some(VarKind::Node { from_path: Some(path), property: None }) => {
                                // Node from path.node - use the array property directly
                                Some((path, mem.property.clone()))
                            }
                            _ => None
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Extract variable name from pattern (only works for simple identifiers)
                let var_name = if let Pattern::Ident(name) = &for_stmt.pattern {
                    name.clone()
                } else {
                    // For complex patterns, generate a temporary name
                    "_item".to_string()
                };

                if let Some((from_path, array_prop)) = loop_item_info {
                    // Generate indexed for loop to track which item we're on
                    let index_var = self.next_loop_index();

                    self.emit_indent();
                    self.gen_expr(&for_stmt.iter);
                    self.emit(&format!(".forEach(({}, {}) => {{\n", var_name, index_var));
                    self.indent += 1;
                    self.loop_depth += 1;

                    // Register the loop variable as a loop item
                    self.register_loop_item(&var_name, &from_path, &array_prop, &index_var);

                    // If pattern is complex, destructure it
                    if !matches!(&for_stmt.pattern, Pattern::Ident(_)) {
                        self.emit_indent();
                        self.emit("const ");
                        self.gen_pattern(&for_stmt.pattern);
                        self.emit(&format!(" = {};\n", var_name));
                    }

                    self.gen_block(&for_stmt.body);

                    self.loop_depth -= 1;
                    self.indent -= 1;
                    self.emit_line("});");
                } else {
                    // Standard for-of loop
                    self.emit_indent();
                    self.emit("for (const ");
                    self.gen_pattern(&for_stmt.pattern);
                    self.emit(" of ");
                    self.gen_expr(&for_stmt.iter);
                    self.emit(") {\n");
                    self.indent += 1;
                    self.loop_depth += 1;
                    self.gen_block(&for_stmt.body);
                    self.loop_depth -= 1;
                    self.indent -= 1;
                    self.emit_line("}");
                }
            }
            Stmt::While(while_stmt) => {
                self.emit_indent();
                self.emit("while (");
                self.gen_expr(&while_stmt.condition);
                self.emit(") {\n");
                self.indent += 1;
                self.loop_depth += 1;
                self.gen_block(&while_stmt.body);
                self.loop_depth -= 1;
                self.indent -= 1;
                self.emit_line("}");
            }
            Stmt::Loop(loop_stmt) => {
                self.emit_line("while (true) {");
                self.indent += 1;
                self.loop_depth += 1;
                self.gen_block(&loop_stmt.body);
                self.loop_depth -= 1;
                self.indent -= 1;
                self.emit_line("}");
            }
            Stmt::Return(ret) => {
                self.emit_indent();
                if let Some(value) = &ret.value {
                    // Check if this is returning Ok(value) - convert to Result object
                    if let Expr::Call(call) = value {
                        if let Expr::Ident(ident) = call.callee.as_ref() {
                            if ident.name == "Ok" {
                                // Convert Ok(value) to { ok: true, value: value }
                                self.emit("return { ok: true, value: ");
                                if let Some(arg) = call.args.first() {
                                    self.gen_expr(arg);
                                } else {
                                    self.emit("undefined");
                                }
                                self.emit(" };\n");
                                return;
                            } else if ident.name == "Err" {
                                // Convert Err(error) to { ok: false, error: error }
                                self.emit("return { ok: false, error: ");
                                if let Some(arg) = call.args.first() {
                                    self.gen_expr(arg);
                                } else {
                                    self.emit("undefined");
                                }
                                self.emit(" };\n");
                                return;
                            }
                        }
                    }
                    // Regular return
                    self.emit("return ");
                    self.gen_expr(value);
                    self.emit(";\n");
                } else {
                    self.emit("return;\n");
                }
            }
            Stmt::Break(_) => {
                self.emit_line("break;");
            }
            Stmt::Continue(_) => {
                self.emit_line("continue;");
            }
            Stmt::Match(match_stmt) => {
                self.gen_match_stmt(match_stmt);
            }
            Stmt::Traverse(traverse_stmt) => {
                self.gen_traverse_stmt(traverse_stmt);
            }
            Stmt::Function(fn_decl) => {
                // Generate nested function
                self.emit_indent();
                self.emit("function ");
                self.emit(&fn_decl.name);
                self.emit("(");
                for (i, param) in fn_decl.params.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.emit(&param.name);
                }
                self.emit(") {\n");
                self.indent += 1;
                self.gen_block(&fn_decl.body);
                self.indent -= 1;
                self.emit_indent();
                self.emit("}\n");
            }
            Stmt::Verbatim(verbatim) => {
                // Emit raw code only for JavaScript target
                match verbatim.target {
                    VerbatimTarget::JavaScript => {
                        self.emit_indent();
                        self.emit(&verbatim.code);
                        if !verbatim.code.ends_with(';') && !verbatim.code.ends_with('}') {
                            self.emit(";");
                        }
                        self.emit("\n");
                    }
                    VerbatimTarget::Rust => {
                        // Skip - this is SWC-only code
                        self.emit_indent();
                        self.emit("/* SWC-only code omitted */\n");
                    }
                }
            }
        }
    }

    fn gen_traverse_stmt(&mut self, traverse_stmt: &crate::parser::TraverseStmt) {
        match &traverse_stmt.kind {
            crate::parser::TraverseKind::Inline(inline) => {
                // Generate unique visitor name
                let visitor_name = self.next_visitor_name();

                // Create traverse context with state variables
                let mut state_vars = std::collections::HashSet::new();
                for let_stmt in &inline.state {
                    // Only track simple identifier patterns as state vars
                    if let Pattern::Ident(name) = &let_stmt.pattern {
                        state_vars.insert(name.clone());
                    }
                }

                // Determine the path variable for traverse call
                // If target is an identifier bound from if-let, we need to find its path
                let path_var = self.determine_traverse_path(&traverse_stmt.target);

                let ctx = TraverseContext {
                    state_vars,
                    id: self.traverse_counter - 1,
                    path_var: path_var.clone(),
                };

                // Note: In JavaScript, captured variables are automatically available
                // via closure semantics. The captures list is for documentation.
                if !traverse_stmt.captures.is_empty() {
                    self.emit_indent();
                    let capture_names: Vec<_> = traverse_stmt.captures.iter()
                        .map(|c| if c.mutable { format!("&mut {}", c.name) } else { format!("&{}", c.name) })
                        .collect();
                    self.emit(&format!("// Captures: [{}]\n", capture_names.join(", ")));
                }

                // Generate inline visitor object
                self.emit_indent();
                self.emit(&format!("const {} = {{\n", visitor_name));
                self.indent += 1;

                // Generate state
                if !inline.state.is_empty() {
                    self.emit_indent();
                    self.emit("state: {\n");
                    self.indent += 1;
                    for let_stmt in &inline.state {
                        // Only emit simple identifier patterns
                        if let Pattern::Ident(name) = &let_stmt.pattern {
                            self.emit_indent();
                            self.emit(name);
                            self.emit(": ");
                            self.gen_expr(&let_stmt.init);
                            self.emit(",\n");
                        }
                    }
                    self.indent -= 1;
                    self.emit_indent();
                    self.emit("},\n");
                }

                // Push context before generating methods
                self.traverse_stack.push(ctx);

                // Generate visitor methods
                for method in &inline.methods {
                    // Convert visit_xxx to PascalCase node type
                    let node_type = self.visitor_method_to_node_type(&method.name);
                    self.emit_indent();
                    self.emit(&format!("{}(path) {{\n", node_type));
                    self.indent += 1;

                    // Generate method body
                    // First, alias the node
                    self.emit_indent();
                    if !method.params.is_empty() {
                        self.emit(&format!("const {} = path.node;\n", method.params[0].name));
                    }

                    self.gen_block(&method.body);
                    self.indent -= 1;
                    self.emit_indent();
                    self.emit("},\n");
                }

                // Pop context after generating methods
                self.traverse_stack.pop();

                self.indent -= 1;
                self.emit_indent();
                self.emit("};\n");

                // Generate traversal call using path.traverse()
                self.emit_indent();
                self.emit(&format!("{}.traverse({});\n", path_var, visitor_name));
            }
            crate::parser::TraverseKind::Delegated(visitor_name) => {
                // Generate delegation to another visitor
                let path_var = self.determine_traverse_path(&traverse_stmt.target);
                self.emit_indent();
                self.emit(&format!("{}.traverse({});\n", path_var, visitor_name));
            }
        }
    }

    /// Determine the path variable to use for traverse calls
    /// In Babel, we need to call path.traverse(), not node.traverse()
    fn determine_traverse_path(&self, target: &Expr) -> String {
        match target {
            Expr::Ident(ident) => {
                let name = &ident.name;

                // Check alias first
                let actual_name = if let Some(alias) = self.param_aliases.get(name) {
                    alias.clone()
                } else {
                    name.clone()
                };

                // Check if this is "node" - use path directly
                if actual_name == "node" {
                    return "path".to_string();
                }

                // Look up the variable kind to determine the correct path
                match self.get_var_kind(&actual_name) {
                    Some(VarKind::Path) => {
                        // Direct path variable
                        actual_name
                    }
                    Some(VarKind::Node { from_path: Some(path), property: Some(prop) }) => {
                        // Node from path.node.property - use path.get(property)
                        format!("{}.get('{}')", path, prop)
                    }
                    Some(VarKind::Node { from_path: Some(path), property: None }) => {
                        // Node from path.node - use path directly
                        path.clone()
                    }
                    Some(VarKind::LoopItem { from_path, array_prop, index_var }) => {
                        // Loop item - index into the array path
                        format!("{}.get(`{}.${{{}}}`)", from_path, array_prop, index_var)
                    }
                    _ => {
                        // Unknown - check common patterns
                        if name == "body" || name.ends_with("_body") {
                            format!("path.get('body')")
                        } else {
                            // Fallback to scope lookup
                            format!("path.scope.getBinding('{}')?.path || path", name)
                        }
                    }
                }
            }
            Expr::Member(mem) => {
                // For member expressions like func.body, generate path.get() chain
                if let Expr::Ident(obj_ident) = mem.object.as_ref() {
                    let obj_name = if let Some(alias) = self.param_aliases.get(&obj_ident.name) {
                        alias.clone()
                    } else {
                        obj_ident.name.clone()
                    };

                    let prop = &mem.property;

                    // Use gen_traverse_path which handles VarKind properly
                    self.gen_traverse_path(&obj_name, prop)
                } else {
                    // Complex object expression - fallback
                    let obj_str = self.expr_to_string(&mem.object);
                    let prop = &mem.property;

                    if obj_str == "node" {
                        format!("path.get('{}')", prop)
                    } else {
                        format!("path.get('{}')", prop)
                    }
                }
            }
            _ => {
                // Fallback: use path directly
                "path".to_string()
            }
        }
    }

    fn gen_if_stmt_with_implicit_return(&mut self, if_stmt: &IfStmt) {
        if let Some(pattern) = &if_stmt.pattern {
            // if-let pattern matching - not supported with implicit return yet
            self.gen_if_let_stmt(if_stmt, pattern);
        } else {
            // Regular if statement with implicit returns in branches
            self.emit_indent();
            self.emit("if (");
            self.gen_expr(&if_stmt.condition);
            self.emit(") {\n");
            self.indent += 1;
            self.gen_block_with_implicit_return(&if_stmt.then_branch);
            self.indent -= 1;

            for (cond, block) in &if_stmt.else_if_branches {
                self.emit_indent();
                self.emit("} else if (");
                self.gen_expr(cond);
                self.emit(") {\n");
                self.indent += 1;
                self.gen_block_with_implicit_return(block);
                self.indent -= 1;
            }

            if let Some(else_block) = &if_stmt.else_branch {
                self.emit_indent();
                self.emit("} else {\n");
                self.indent += 1;
                self.gen_block_with_implicit_return(else_block);
                self.indent -= 1;
            }

            self.emit_line("}");
        }
    }

    fn gen_if_let_stmt(&mut self, if_stmt: &IfStmt, pattern: &Pattern) {
        // Generate if-let as:
        // const __temp = expr;
        // if (__temp !== null && __temp !== undefined) {
        //     const binding = __temp; // or destructure
        //     ...then_branch
        // } else { ...else_branch }

        let temp_var = format!("__iflet_{}", self.iflet_counter);
        self.iflet_counter += 1;

        // Generate temp variable
        self.emit_indent();
        self.emit(&format!("const {} = ", temp_var));
        self.gen_expr(&if_stmt.condition);
        self.emit(";\n");

        // Generate condition check based on pattern
        self.emit_indent();
        match pattern {
            Pattern::Variant { name, inner } => {
                match name.as_str() {
                    "Some" => {
                        // if (__temp !== null && __temp !== undefined)
                        self.emit(&format!("if ({} !== null && {} !== undefined) {{\n", temp_var, temp_var));
                        self.indent += 1;

                        // Bind inner pattern
                        if let Some(inner_pat) = inner {
                            // Unwrap Ref patterns to get to the actual binding
                            let binding_pat = match inner_pat.as_ref() {
                                Pattern::Ref { pattern: inner, .. } => inner.as_ref(),
                                other => other,
                            };

                            if let Pattern::Ident(binding) = binding_pat {
                                self.emit_indent();
                                self.emit(&format!("const {} = {};\n", binding, temp_var));
                            }
                        }
                    }
                    "None" => {
                        // if (__temp === null || __temp === undefined)
                        self.emit(&format!("if ({} === null || {} === undefined) {{\n", temp_var, temp_var));
                        self.indent += 1;
                    }
                    "Ok" => {
                        // For Result types - check for success
                        self.emit(&format!("if ({} && !{}.error) {{\n", temp_var, temp_var));
                        self.indent += 1;

                        if let Some(inner_pat) = inner {
                            // Unwrap Ref patterns to get to the actual binding
                            let binding_pat = match inner_pat.as_ref() {
                                Pattern::Ref { pattern: inner, .. } => inner.as_ref(),
                                other => other,
                            };

                            if let Pattern::Ident(binding) = binding_pat {
                                self.emit_indent();
                                self.emit(&format!("const {} = {}.value;\n", binding, temp_var));
                            }
                        }
                    }
                    "Err" => {
                        // For Result types - check for error
                        self.emit(&format!("if ({} && {}.error) {{\n", temp_var, temp_var));
                        self.indent += 1;

                        if let Some(inner_pat) = inner {
                            // Unwrap Ref patterns to get to the actual binding
                            let binding_pat = match inner_pat.as_ref() {
                                Pattern::Ref { pattern: inner, .. } => inner.as_ref(),
                                other => other,
                            };

                            if let Pattern::Ident(binding) = binding_pat {
                                self.emit_indent();
                                self.emit(&format!("const {} = {}.error;\n", binding, temp_var));
                            }
                        }
                    }
                    _ => {
                        // Generic variant check for AST node types
                        // Extract variant name from qualified path (e.g., "Expression::ArrayExpression" -> "ArrayExpression")
                        let variant_name = if name.contains("::") {
                            name.split("::").last().unwrap_or(name)
                        } else {
                            name.as_str()
                        };
                        self.emit(&format!("if ({} !== null) {{\n", temp_var));
                        self.indent += 1;

                        // Bind inner pattern if present
                        if let Some(inner_pat) = inner {
                            // Unwrap Ref patterns to get to the actual binding
                            let binding_pat = match inner_pat.as_ref() {
                                Pattern::Ref { pattern: inner, .. } => inner.as_ref(),
                                other => other,
                            };

                            if let Pattern::Ident(binding) = binding_pat {
                                self.emit_indent();
                                self.emit(&format!("const {} = {};\n", binding, temp_var));
                            }
                        }
                    }
                }
            }
            Pattern::Ident(binding) => {
                // Simple binding: if let x = expr
                self.emit(&format!("if ({} !== null && {} !== undefined) {{\n", temp_var, temp_var));
                self.indent += 1;
                self.emit_indent();
                self.emit(&format!("const {} = {};\n", binding, temp_var));
            }
            _ => {
                // Fallback for other patterns
                self.emit(&format!("if ({} !== null) {{\n", temp_var));
                self.indent += 1;
            }
        }

        // Generate then branch
        self.gen_block(&if_stmt.then_branch);
        self.indent -= 1;

        // Generate else branch
        if let Some(else_block) = &if_stmt.else_branch {
            self.emit_indent();
            self.emit("} else {\n");
            self.indent += 1;
            self.gen_block(else_block);
            self.indent -= 1;
        }

        self.emit_line("}");
    }

    fn visitor_method_to_node_type(&self, method_name: &str) -> String {
        // Use mapping module: visit_xxx_yyy -> XxxYyy
        if let Some(mapping) = get_node_mapping_by_visitor(method_name) {
            return mapping.babel.to_string();
        }

        // Fallback: convert visit_xxx_yyy to XxxYyy
        if let Some(stripped) = method_name.strip_prefix("visit_") {
            stripped
                .split('_')
                .map(|part| {
                    let mut chars = part.chars();
                    match chars.next() {
                        None => String::new(),
                        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                    }
                })
                .collect()
        } else {
            method_name.to_string()
        }
    }

    fn gen_match_stmt(&mut self, match_stmt: &MatchStmt) {
        // Convert match to switch or if-else chain
        // For now, generate if-else chain for flexibility
        let mut first = true;
        for arm in &match_stmt.arms {
            self.emit_indent();
            if first {
                self.emit("if (");
                first = false;
            } else {
                self.emit("} else if (");
            }
            self.gen_pattern_condition(&arm.pattern, &match_stmt.scrutinee);
            self.emit(") {\n");
            self.indent += 1;

            // Extract bindings from pattern
            self.gen_pattern_bindings(&arm.pattern, &match_stmt.scrutinee);

            // Handle block bodies specially - unwrap them instead of generating IIFE
            if let Expr::Block(block) = &arm.body {
                // Generate block statements inline, with last expression as return (unless in loop)
                for (i, stmt) in block.stmts.iter().enumerate() {
                    let is_last = i == block.stmts.len() - 1;
                    if is_last && self.loop_depth == 0 {
                        // Last statement - if it's an expression and we're not in a loop, return it
                        if let Stmt::Expr(expr_stmt) = stmt {
                            self.emit_indent();
                            self.emit("return ");
                            self.gen_expr(&expr_stmt.expr);
                            self.emit(";\n");
                        } else {
                            // It's a statement, generate it normally
                            self.gen_stmt(stmt);
                        }
                    } else {
                        // Not last, or inside a loop - generate normally
                        self.gen_stmt(stmt);
                    }
                }
            } else {
                // Non-block body, return directly (unless in loop)
                self.emit_indent();
                if self.loop_depth == 0 {
                    self.emit("return ");
                }
                self.gen_expr(&arm.body);
                self.emit(";\n");
            }
            self.indent -= 1;
        }
        self.emit_line("}");
    }

    fn gen_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Ident(name) => {
                self.emit(name);
            }
            Pattern::Tuple(patterns) => {
                self.emit("[");
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.gen_pattern(pat);
                }
                self.emit("]");
            }
            Pattern::Array(patterns) => {
                self.emit("[");
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.gen_pattern(pat);
                }
                self.emit("]");
            }
            Pattern::Object(props) => {
                self.emit("{");
                for (i, prop) in props.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    match prop {
                        ObjectPatternProp::KeyValue { key, value } => {
                            self.emit(key);
                            self.emit(": ");
                            self.gen_pattern(value);
                        }
                        ObjectPatternProp::Shorthand(name) => {
                            self.emit(name);
                        }
                        ObjectPatternProp::Rest(name) => {
                            self.emit("...");
                            self.emit(name);
                        }
                        ObjectPatternProp::Or(_) => {
                            // Or patterns in object props don't make sense in JS
                            self.emit("_");
                        }
                    }
                }
                self.emit("}");
            }
            Pattern::Wildcard => {
                self.emit("_");
            }
            Pattern::Rest(inner) => {
                self.emit("...");
                self.gen_pattern(inner);
            }
            Pattern::Literal(lit) => {
                // Literals in destructuring don't make sense in JS, emit placeholder
                self.gen_literal(lit);
            }
            Pattern::Ref { pattern: inner, .. } => {
                // JavaScript doesn't have ref - just emit the inner pattern
                self.gen_pattern(inner);
            }
            Pattern::Struct { .. } | Pattern::Variant { .. } | Pattern::Or(_) => {
                // Complex patterns that require match/if-let - emit placeholder
                self.emit("_");
            }
        }
    }

    fn gen_pattern_condition(&mut self, pattern: &Pattern, scrutinee: &Expr) {
        match pattern {
            Pattern::Literal(lit) => {
                self.gen_expr(scrutinee);
                self.emit(" === ");
                self.gen_literal(lit);
            }
            Pattern::Ident(name) => {
                // Binding pattern - always true, but we bind the value
                self.emit("true");
                // The binding is handled separately
                let _ = name; // suppress unused warning
            }
            Pattern::Wildcard => {
                self.emit("true");
            }
            Pattern::Or(patterns) => {
                self.emit("(");
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.emit(" || ");
                    }
                    self.gen_pattern_condition(p, scrutinee);
                }
                self.emit(")");
            }
            Pattern::Struct { name, fields } => {
                // Type check with field pattern matching
                // Use mapping to get the correct Babel type checker
                let checker = get_node_mapping(name)
                    .map(|m| m.babel_checker.to_string())
                    .unwrap_or_else(|| format!("is{}", name));
                self.emit(&format!("t.{}(", checker));
                self.gen_expr(scrutinee);
                self.emit(")");
                for (field_name, field_pattern) in fields {
                    self.emit(" && ");
                    // Create nested scrutinee for field
                    let field_access = format!("{}.{}", self.expr_to_string(scrutinee), field_name);
                    self.gen_pattern_condition_str(field_pattern, &field_access);
                }
            }
            Pattern::Tuple(_) | Pattern::Array(_) | Pattern::Object(_) | Pattern::Rest(_) => {
                // Complex patterns that aren't fully supported in condition checks
                self.emit("true");
            }
            Pattern::Variant { name, inner: _ } => {
                // Variant pattern matching for Option/Result types and AST nodes
                let scrutinee_str = self.expr_to_string(scrutinee);
                match name.as_str() {
                    "Some" => {
                        self.emit(&format!("{} !== null && {} !== undefined", scrutinee_str, scrutinee_str));
                    }
                    "None" => {
                        self.emit(&format!("{} === null || {} === undefined", scrutinee_str, scrutinee_str));
                    }
                    "Ok" => {
                        self.emit(&format!("{} && !{}.error", scrutinee_str, scrutinee_str));
                    }
                    "Err" => {
                        self.emit(&format!("{} && {}.error", scrutinee_str, scrutinee_str));
                    }
                    _ => {
                        // For other variants (like AST node types), check the type property
                        // Extract variant name from qualified path (e.g., "Expression::NumericLiteral" -> "NumericLiteral")
                        let variant_name = if name.contains("::") {
                            name.split("::").last().unwrap_or(name)
                        } else {
                            name.as_str()
                        };
                        self.emit(&format!("{}.type === \"{}\"", scrutinee_str, variant_name));
                    }
                }
            }
            Pattern::Ref { pattern: inner, .. } => {
                // ref doesn't affect the condition - check the inner pattern
                self.gen_pattern_condition(inner, scrutinee);
            }
        }
    }

    fn gen_pattern_bindings(&mut self, pattern: &Pattern, scrutinee: &Expr) {
        // Extract bindings from pattern and emit const declarations
        match pattern {
            Pattern::Variant { name: _, inner } => {
                if let Some(inner_pattern) = inner {
                    if let Pattern::Ident(binding_name) = inner_pattern.as_ref() {
                        // Generate: const binding_name = scrutinee;
                        self.emit_indent();
                        self.emit("const ");
                        self.emit(binding_name);
                        self.emit(" = ");
                        self.gen_expr(scrutinee);
                        self.emit(";\n");
                    }
                }
            }
            Pattern::Ref { pattern: inner, .. } => {
                // ref doesn't affect bindings - extract from inner pattern
                self.gen_pattern_bindings(inner, scrutinee);
            }
            _ => {
                // Other patterns don't introduce bindings at this level
            }
        }
    }

    fn gen_pattern_condition_str(&mut self, pattern: &Pattern, scrutinee: &str) {
        match pattern {
            Pattern::Literal(lit) => {
                self.emit(scrutinee);
                self.emit(" === ");
                self.gen_literal(lit);
            }
            Pattern::Ident(_) => {
                self.emit("true");
            }
            Pattern::Wildcard => {
                self.emit("true");
            }
            _ => {
                self.emit("true"); // Simplified for now
            }
        }
    }

    fn expr_to_string(&self, expr: &Expr) -> String {
        let mut gen = BabelGenerator::new();
        gen.gen_expr(expr);
        gen.output
    }

    fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => self.gen_literal(lit),
            Expr::Ident(ident) => {
                // Handle special cases
                match ident.name.as_str() {
                    "self" => self.emit("this"),
                    "None" => self.emit("null"),
                    // Handle Default::default() which the hoister uses as a placeholder
                    name if name.starts_with("Default::default") => self.emit("undefined"),
                    _ => {
                        // Check if this is a traverse state variable
                        if self.is_traverse_state_var(&ident.name) {
                            self.emit(&format!("this.state.{}", ident.name));
                        } else if let Some(alias) = self.param_aliases.get(&ident.name).cloned() {
                            // Check if this identifier has an alias
                            self.emit(&alias);
                        } else {
                            self.emit(&ident.name);
                        }
                    }
                }
            }
            Expr::Binary(bin) => {
                self.emit("(");
                self.gen_expr(&bin.left);
                self.emit(&format!(" {} ", self.binary_op_to_js(&bin.op)));
                self.gen_expr(&bin.right);
                self.emit(")");
            }
            Expr::Unary(un) => {
                match un.op {
                    UnaryOp::Not => {
                        self.emit("!");
                        self.gen_expr(&un.operand);
                    }
                    UnaryOp::Neg => {
                        self.emit("-");
                        self.gen_expr(&un.operand);
                    }
                    UnaryOp::Deref => {
                        // In JS, dereference is a no-op
                        self.gen_expr(&un.operand);
                    }
                    UnaryOp::Ref | UnaryOp::RefMut => {
                        // In JS, references are passed directly
                        self.gen_expr(&un.operand);
                    }
                }
            }
            Expr::Call(call) => {
                // Check for matches! macro
                if let Expr::Ident(ident) = call.callee.as_ref() {
                    if ident.name == "matches!" && call.args.len() >= 2 {
                        self.gen_matches_macro(&call.args[0], &call.args[1]);
                        return;
                    }
                    // Check for panic! macro -> throw new Error()
                    if ident.name == "panic!" {
                        self.emit("throw new Error(");
                        if let Some(arg) = call.args.first() {
                            self.gen_expr(arg);
                        } else {
                            self.emit("\"panic\"");
                        }
                        self.emit(")");
                        return;
                    }
                    // Check for format! macro -> template literal
                    if ident.name == "format" && !call.args.is_empty() {
                        self.gen_format_macro(&call.args);
                        return;
                    }
                    // Check for Some(x) -> x (unwrap Option in JavaScript)
                    if ident.name == "Some" && call.args.len() == 1 {
                        self.gen_expr(&call.args[0]);
                        return;
                    }
                    // Check for Ok(x) -> { ok: true, value: x }
                    if ident.name == "Ok" {
                        self.emit("{ ok: true, value: ");
                        if let Some(arg) = call.args.first() {
                            self.gen_expr(arg);
                        } else {
                            self.emit("undefined");
                        }
                        self.emit(" }");
                        return;
                    }
                    // Check for Err(x) -> { ok: false, error: x }
                    if ident.name == "Err" {
                        self.emit("{ ok: false, error: ");
                        if let Some(arg) = call.args.first() {
                            self.gen_expr(arg);
                        } else {
                            self.emit("undefined");
                        }
                        self.emit(" }");
                        return;
                    }
                    // Check for Default::default() -> undefined or appropriate default
                    if ident.name == "Default::default()" || ident.name == "Default::default" {
                        self.emit("undefined");
                        return;
                    }
                }
                // Check for Type::new() patterns
                if let Expr::Member(mem) = call.callee.as_ref() {
                    if mem.property == "new" {
                        if let Expr::Ident(type_ident) = mem.object.as_ref() {
                            match type_ident.name.as_str() {
                                // HashMap::new() -> new Map()
                                "HashMap" => {
                                    self.emit("new Map()");
                                    return;
                                }
                                // HashSet::new() -> new Set()
                                "HashSet" => {
                                    self.emit("new Set()");
                                    return;
                                }
                                // String::new() -> ""
                                "String" => {
                                    self.emit("\"\"");
                                    return;
                                }
                                // CodeBuilder::new() -> []
                                "CodeBuilder" => {
                                    self.emit("[]");
                                    return;
                                }
                                _ => {}
                            }
                        }
                    }

                    // CodeBuilder method calls - only for old class-based writers
                    // For new plugin-based writers, builder is an object with actual methods
                    // so we don't need special handling

                    // Check if this is builder.method() or self.builder.method()
                    let is_builder_method = match mem.object.as_ref() {
                        Expr::Ident(ident) if ident.name == "builder" => true,
                        Expr::Member(inner_mem) => {
                            // Check for self.builder
                            if let Expr::Ident(obj) = inner_mem.object.as_ref() {
                                obj.name == "self" && inner_mem.property == "builder"
                            } else {
                                false
                            }
                        }
                        _ => false
                    };

                    if !is_builder_method {
                        // Old-style CodeBuilder array methods (for backward compatibility)
                        if mem.property == "append" {
                            // builder.append(s) -> builder.push(s)
                            self.gen_expr(&mem.object);
                            self.emit(".push(");
                            if !call.args.is_empty() {
                                self.gen_expr(&call.args[0]);
                            }
                            self.emit(")");
                            return;
                        }
                        if mem.property == "append_line" {
                            // builder.append_line(s) -> (builder.push(s), builder.push("\n"))
                            self.emit("(");
                            self.gen_expr(&mem.object);
                            self.emit(".push(");
                            if !call.args.is_empty() {
                                self.gen_expr(&call.args[0]);
                            }
                            self.emit("), ");
                            self.gen_expr(&mem.object);
                            self.emit(".push(\"\\n\"))");
                            return;
                        }
                        if mem.property == "newline" {
                            // builder.newline() -> builder.push("\n")
                            self.gen_expr(&mem.object);
                            self.emit(".push(\"\\n\")");
                            return;
                        }
                        if mem.property == "to_string" && call.args.is_empty() {
                            // Check if this is CodeBuilder/array vs primitive type
                            // builder.to_string() -> builder.join("")
                            // num.to_string() -> num.toString()
                            // For primitives (numbers, booleans), use .toString()
                            // For arrays/strings, use .join("") or keep as-is
                            let obj_str = self.expr_to_string(&mem.object);
                            if obj_str.contains("builder") || obj_str.contains("lines") || obj_str.contains("result") && !obj_str.contains(".value") {
                                // Likely an array/builder
                                self.gen_expr(&mem.object);
                                self.emit(".join(\"\")");
                            } else {
                                // Likely a primitive (number, boolean, etc.)
                                self.gen_expr(&mem.object);
                                self.emit(".toString()");
                            }
                            return;
                        }
                        if mem.property == "indent" || mem.property == "dedent" {
                            // indent/dedent are no-ops for now in Babel (would need state tracking)
                            self.emit("undefined");
                            return;
                        }
                    }

                    // Check for codegen::generate() and codegen::generate_with_options()
                    if mem.is_path {
                        if let Expr::Ident(module_ident) = mem.object.as_ref() {
                            if module_ident.name == "codegen" {
                                self.gen_codegen_call(&mem.property, &call.args);
                                return;
                            }
                        }
                    }
                }
                // Also check as a standalone identifier (no parens in name)
                if let Expr::Ident(ident) = call.callee.as_ref() {
                    if ident.name.starts_with("Default::default") {
                        self.emit("undefined");
                        return;
                    }
                }

                // Check if this is a method call that should be a property in JS
                if let Expr::Member(mem) = call.callee.as_ref() {
                    let prop = &mem.property;
                    // len() -> .length for arrays, .size for Map/Set
                    // Since we can't easily distinguish, we'll use .length which works for arrays
                    // Map/Set should use .size but that requires type tracking
                    if prop == "len" {
                        self.gen_expr(&mem.object);
                        self.emit(".length");
                        return;
                    }
                    // keys() and values() for Maps
                    if prop == "keys" || prop == "values" {
                        self.gen_expr(&mem.object);
                        self.emit(&format!(".{}()", prop));
                        return;
                    }
                    // is_empty() -> .length === 0
                    if prop == "is_empty" {
                        self.emit("(");
                        self.gen_expr(&mem.object);
                        self.emit(".length === 0)");
                        return;
                    }
                    // chars() -> just the string itself (or .split('') for actual char array)
                    if prop == "chars" {
                        self.gen_expr(&mem.object);
                        return;
                    }
                    // iter() -> just the array itself
                    if prop == "iter" {
                        self.gen_expr(&mem.object);
                        return;
                    }
                    // enumerate() on an iterator -> .entries()
                    if prop == "enumerate" {
                        self.gen_expr(&mem.object);
                        self.emit(".entries()");
                        return;
                    }
                    // clone() -> just the value (no-op in JS)
                    if prop == "clone" {
                        self.gen_expr(&mem.object);
                        return;
                    }
                    // remove() on Context -> path.remove() in Babel
                    if prop == "remove" && call.args.is_empty() {
                        self.gen_expr(&mem.object);
                        self.emit(".remove()");
                        return;
                    }
                    // next() -> [0] (get first element from iterator/string)
                    if prop == "next" {
                        self.gen_expr(&mem.object);
                        self.emit("[0]");
                        return;
                    }
                    // unwrap() -> just the value (no-op in JS, Option/Result unwrapping)
                    if prop == "unwrap" {
                        self.gen_expr(&mem.object);
                        return;
                    }
                    // collect() -> just the value (no-op in JS, iterators already return arrays)
                    if prop == "collect" {
                        self.gen_expr(&mem.object);
                        return;
                    }
                    // unwrap_or(default) -> value ?? default (nullish coalescing)
                    if prop == "unwrap_or" {
                        self.emit("(");
                        self.gen_expr(&mem.object);
                        self.emit(" ?? ");
                        if let Some(arg) = call.args.first() {
                            self.gen_expr(arg);
                        } else {
                            self.emit("undefined");
                        }
                        self.emit(")");
                        return;
                    }
                    // is_uppercase() -> char === char.toUpperCase()
                    if prop == "is_uppercase" {
                        self.emit("(");
                        let obj_str = self.expr_to_string(&mem.object);
                        self.gen_expr(&mem.object);
                        self.emit(&format!(" === {}.toUpperCase())", obj_str));
                        return;
                    }
                    // insert() has different meanings for Vec vs HashMap vs HashSet
                    // We need to distinguish based on arg count and types
                    if prop == "insert" {
                        match call.args.len() {
                            // set.insert(v) -> set.add(v)
                            1 => {
                                self.gen_expr(&mem.object);
                                self.emit(".add(");
                                self.gen_expr(&call.args[0]);
                                self.emit(")");
                                return;
                            }
                            // 2 args: could be Vec.insert(idx, val) or Map.insert(key, val)
                            // Vec.insert uses numeric index, Map uses any key type
                            // Check if first arg is a numeric literal for Vec case
                            2 => {
                                if let Expr::Literal(Literal::Int(idx)) = &call.args[0] {
                                    // Vec.insert(0, x) -> unshift(x)
                                    if *idx == 0 {
                                        self.gen_expr(&mem.object);
                                        self.emit(".unshift(");
                                        self.gen_expr(&call.args[1]);
                                        self.emit(")");
                                        return;
                                    }
                                    // Vec.insert(n, x) -> splice(n, 0, x)
                                    self.gen_expr(&mem.object);
                                    self.emit(".splice(");
                                    self.gen_expr(&call.args[0]);
                                    self.emit(", 0, ");
                                    self.gen_expr(&call.args[1]);
                                    self.emit(")");
                                    return;
                                }
                                // Map.insert(key, val) -> map.set(key, val)
                                self.gen_expr(&mem.object);
                                self.emit(".set(");
                                self.gen_expr(&call.args[0]);
                                self.emit(", ");
                                self.gen_expr(&call.args[1]);
                                self.emit(")");
                                return;
                            }
                            _ => {}
                        }
                    }
                    // map.get(&k) -> map.get(k)
                    if prop == "get" {
                        self.gen_expr(&mem.object);
                        self.emit(".get(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(")");
                        return;
                    }
                    // map.get_mut(&k) -> map.get(k)
                    if prop == "get_mut" {
                        self.gen_expr(&mem.object);
                        self.emit(".get(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(")");
                        return;
                    }
                    // map.contains_key(&k) -> map.has(k)
                    if prop == "contains_key" {
                        self.gen_expr(&mem.object);
                        self.emit(".has(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(")");
                        return;
                    }
                    // set.contains(&v) -> set.has(v)
                    if prop == "contains" {
                        self.gen_expr(&mem.object);
                        self.emit(".has(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(")");
                        return;
                    }
                    // map.remove(&k) -> map.delete(k)
                    if prop == "remove" {
                        self.gen_expr(&mem.object);
                        self.emit(".delete(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(")");
                        return;
                    }
                    // s.as_str() -> s (no-op in JS, strings are already strings)
                    if prop == "as_str" {
                        self.gen_expr(&mem.object);
                        return;
                    }
                    // .into() -> just the value (no-op in JS, type conversions are implicit)
                    if prop == "into" {
                        self.gen_expr(&mem.object);
                        return;
                    }
                    // s.push_str(&t) -> s += t
                    if prop == "push_str" && call.args.len() == 1 {
                        self.gen_expr(&mem.object);
                        self.emit(" += ");
                        self.gen_expr(&call.args[0]);
                        return;
                    }
                    // fs::write(path, content) -> fs.writeFileSync(path, content)
                    if prop == "write" {
                        self.gen_expr(&mem.object);
                        self.emit(".writeFileSync(");
                        for (i, arg) in call.args.iter().enumerate() {
                            if i > 0 {
                                self.emit(", ");
                            }
                            self.gen_expr(arg);
                        }
                        self.emit(")");
                        return;
                    }
                    // fs::read_to_string(path) -> fs.readFileSync(path, 'utf8')
                    if prop == "read_to_string" {
                        self.gen_expr(&mem.object);
                        self.emit(".readFileSync(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(", 'utf8')");
                        return;
                    }
                    // fs::exists(path) -> fs.existsSync(path)
                    if prop == "exists" {
                        self.gen_expr(&mem.object);
                        self.emit(".existsSync(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(")");
                        return;
                    }
                    // fs::create_dir_all(path) -> fs.mkdirSync(path, { recursive: true })
                    if prop == "create_dir_all" {
                        self.gen_expr(&mem.object);
                        self.emit(".mkdirSync(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(", { recursive: true })");
                        return;
                    }
                    // fs::remove_file(path) -> fs.unlinkSync(path)
                    if prop == "remove_file" {
                        self.gen_expr(&mem.object);
                        self.emit(".unlinkSync(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(")");
                        return;
                    }
                    // json::to_string(&v) -> JSON.stringify(v)
                    if prop == "to_string" {
                        // Check if it's json module
                        if let Expr::Ident(ident) = mem.object.as_ref() {
                            if ident.name == "json" {
                                self.emit("JSON.stringify(");
                                if !call.args.is_empty() {
                                    self.gen_expr(&call.args[0]);
                                }
                                self.emit(")");
                                return;
                            }
                        }
                    }
                    // json::to_string_pretty(&v) -> JSON.stringify(v, null, 2)
                    if prop == "to_string_pretty" {
                        self.emit("JSON.stringify(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(", null, 2)");
                        return;
                    }
                    // json::from_str(&s) -> JSON.parse(s)
                    if prop == "from_str" {
                        self.emit("JSON.parse(");
                        if !call.args.is_empty() {
                            self.gen_expr(&call.args[0]);
                        }
                        self.emit(")");
                        return;
                    }
                    // to_string() -> no-op in JS (strings are already strings)
                    if prop == "to_string" {
                        self.gen_expr(&mem.object);
                        return;
                    }
                    // visit_children(self) -> Babel auto-traverses, so this is a no-op
                    // We emit nothing since Babel handles traversal automatically
                    if prop == "visit_children" {
                        self.emit("/* Babel auto-traverses */");
                        return;
                    }
                    // visit_with(self) -> for manual iteration, also skip
                    if prop == "visit_with" {
                        self.emit("/* manual traversal handled by Babel */");
                        return;
                    }
                }

                self.gen_expr(&call.callee);
                self.emit("(");
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.gen_expr(arg);
                }
                self.emit(")");
            }
            Expr::Member(mem) => {
                // Special case: self.builder in writers should become just "builder"
                if let Expr::Ident(obj_ident) = mem.object.as_ref() {
                    if obj_ident.name == "self" && mem.property == "builder" {
                        // In writer context, self.builder -> builder
                        self.emit("builder");
                        return;
                    }
                }

                self.gen_expr(&mem.object);
                // Convert Rust-style method names to JS
                let js_name = self.convert_method_name(&mem.property);
                if !js_name.is_empty() {
                    self.emit(".");
                    // Map SWC field names back to Babel field names
                    let babel_field = match js_name.as_str() {
                        "sym" => "name", // Ident.sym -> Identifier.name
                        _ => &js_name,
                    };
                    self.emit(babel_field);
                }
                // If empty (like .clone()), we skip the member access entirely
            }
            Expr::Index(idx) => {
                // Check if this is a slice with range syntax
                if let Expr::Range(range) = idx.index.as_ref() {
                    // Convert name[start..end] to name.slice(start, end)
                    self.gen_expr(&idx.object);
                    self.emit(".slice(");
                    if let Some(start) = &range.start {
                        self.gen_expr(start);
                    } else {
                        self.emit("0");
                    }
                    if let Some(end) = &range.end {
                        self.emit(", ");
                        self.gen_expr(end);
                    }
                    self.emit(")");
                } else {
                    // Regular index access
                    self.gen_expr(&idx.object);
                    self.emit("[");
                    self.gen_expr(&idx.index);
                    self.emit("]");
                }
            }
            Expr::StructInit(init) => {
                // Check if this is an AST node type that should use Babel builders
                if let Some(babel_builder) = self.ast_type_to_babel_builder(&init.name) {
                    self.gen_babel_node_construction(&babel_builder, &init.fields);
                } else {
                    // Generate as object literal
                    self.emit("{ ");
                    for (i, (name, value)) in init.fields.iter().enumerate() {
                        if i > 0 {
                            self.emit(", ");
                        }
                        self.emit(name);
                        self.emit(": ");
                        self.gen_expr(value);
                    }
                    self.emit(" }");
                }
            }
            Expr::VecInit(vec) => {
                self.emit("[");
                for (i, elem) in vec.elements.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.gen_expr(elem);
                }
                self.emit("]");
            }
            Expr::If(if_expr) => {
                self.emit("(");
                self.gen_expr(&if_expr.condition);
                self.emit(" ? ");
                // Simplified - assumes single expression in then branch
                if let Some(stmt) = if_expr.then_branch.stmts.first() {
                    if let Stmt::Expr(expr_stmt) = stmt {
                        self.gen_expr(&expr_stmt.expr);
                    }
                }
                self.emit(" : ");
                if let Some(else_block) = &if_expr.else_branch {
                    if let Some(stmt) = else_block.stmts.first() {
                        if let Stmt::Expr(expr_stmt) = stmt {
                            self.gen_expr(&expr_stmt.expr);
                        }
                    }
                } else {
                    self.emit("undefined");
                }
                self.emit(")");
            }
            Expr::Match(match_expr) => {
                // Generate as IIFE with switch
                self.emit("(() => { ");
                let mut first = true;
                for arm in &match_expr.arms {
                    if first {
                        self.emit("if (");
                        first = false;
                    } else {
                        self.emit(" else if (");
                    }
                    self.gen_pattern_condition(&arm.pattern, &match_expr.scrutinee);
                    self.emit(") { ");

                    // Extract bindings if needed
                    if let Pattern::Variant { inner: Some(_), .. } = &arm.pattern {
                        // Save current indent and temporarily use no indent for inline bindings
                        let saved_indent = self.indent;
                        self.indent = 0;
                        self.gen_pattern_bindings(&arm.pattern, &match_expr.scrutinee);
                        self.indent = saved_indent;
                        // If there was a binding, we need to separate it from the return
                        self.emit(" ");
                    }

                    // Handle block bodies specially - unwrap them instead of generating IIFE
                    if let Expr::Block(block) = &arm.body {
                        // Generate block statements inline, with last expression as return
                        for (i, stmt) in block.stmts.iter().enumerate() {
                            let is_last = i == block.stmts.len() - 1;
                            if is_last {
                                // Last statement - if it's an expression, return it
                                if let Stmt::Expr(expr_stmt) = stmt {
                                    self.emit("return ");
                                    self.gen_expr(&expr_stmt.expr);
                                    self.emit(";");
                                } else {
                                    // It's a statement, generate it normally
                                    self.gen_stmt(stmt);
                                }
                            } else {
                                // Not last, generate normally
                                self.gen_stmt(stmt);
                                self.emit(" ");
                            }
                        }
                    } else {
                        // Non-block body, return directly
                        self.emit("return ");
                        self.gen_expr(&arm.body);
                        self.emit(";");
                    }
                    self.emit(" }");
                }
                self.emit(" })()");
            }
            Expr::Closure(closure) => {
                self.emit("(");
                self.emit(&closure.params.join(", "));
                self.emit(") => ");
                self.gen_expr(&closure.body);
            }
            Expr::Ref(ref_expr) => {
                // In JS, just pass the value directly
                self.gen_expr(&ref_expr.expr);
            }
            Expr::Deref(deref) => {
                // In JS, dereference is a no-op
                self.gen_expr(&deref.expr);
            }
            Expr::Assign(assign) => {
                // Statement lowering: *node = ... becomes path.replaceWith(...)
                // This applies to any deref assignment, not just "node"
                if let Expr::Deref(deref) = assign.target.as_ref() {
                    if let Expr::Ident(_ident) = deref.expr.as_ref() {
                        // Any dereference assignment in a visitor should use path.replaceWith
                        self.emit("path.replaceWith(");
                        self.gen_expr(&assign.value);
                        self.emit(")");
                        return;
                    }
                }
                // Regular assignment
                self.gen_expr(&assign.target);
                self.emit(" = ");
                self.gen_expr(&assign.value);
            }
            Expr::CompoundAssign(compound) => {
                // Check if target is a traverse state variable
                if let Expr::Ident(ident) = compound.target.as_ref() {
                    if self.is_traverse_state_var(&ident.name) {
                        self.emit(&format!("this.state.{}", ident.name));
                        self.emit(&format!(" {}= ", self.compound_op_to_js(&compound.op)));
                        self.gen_expr(&compound.value);
                        return;
                    }
                }
                self.gen_expr(&compound.target);
                self.emit(&format!(" {}= ", self.compound_op_to_js(&compound.op)));
                self.gen_expr(&compound.value);
            }
            Expr::Range(range) => {
                // Generate array range (simplified)
                self.emit("/* range not fully supported */");
                if let Some(start) = &range.start {
                    self.gen_expr(start);
                }
                self.emit("..");
                if let Some(end) = &range.end {
                    self.gen_expr(end);
                }
            }
            Expr::Block(block) => {
                // Block expression: generate an IIFE with implicit return
                self.emit("(() => {\n");
                self.indent += 1;
                self.gen_block_with_implicit_return(block);
                self.indent -= 1;
                self.emit_indent();
                self.emit("})()");
            }
            Expr::Try(inner) => {
                // Try operator: expr?
                // In JavaScript, we can't easily emulate this without async/throw semantics
                // For now, just emit the inner expression with a comment
                self.emit("(/* ? */ ");
                self.gen_expr(inner);
                self.emit(")");
            }
            Expr::Paren(inner) => {
                self.emit("(");
                self.gen_expr(inner);
                self.emit(")");
            }
            Expr::Tuple(elements) => {
                // Tuples become arrays in JavaScript
                self.emit("[");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.gen_expr(elem);
                }
                self.emit("]");
            }

            Expr::Matches(matches_expr) => {
                // Generate matches! macro as pattern matching check
                self.emit("(");
                self.gen_matches_pattern_new(&matches_expr.scrutinee, &matches_expr.pattern);
                self.emit(")");
            }

            Expr::Return(value) => {
                self.emit("return");
                if let Some(ref expr) = value {
                    self.emit(" ");
                    self.gen_expr(expr);
                }
            }

            Expr::Break => {
                self.emit("break");
            }

            Expr::Continue => {
                self.emit("continue");
            }

            Expr::RegexCall(_regex_call) => {
                // TODO: Implement regex call generation for Babel
                self.emit("/* TODO: Regex call */");
            }
        }
    }

    fn gen_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::String(s) => {
                // Escape special characters for JavaScript strings
                let escaped = s
                    .replace('\\', "\\\\")  // Backslash must be first
                    .replace('"', "\\\"")   // Double quote
                    .replace('\n', "\\n")   // Newline
                    .replace('\r', "\\r")   // Carriage return
                    .replace('\t', "\\t");  // Tab
                self.emit(&format!("\"{}\"", escaped));
            }
            Literal::Int(n) => {
                self.emit(&n.to_string());
            }
            Literal::Float(n) => {
                self.emit(&n.to_string());
            }
            Literal::Bool(b) => {
                self.emit(if *b { "true" } else { "false" });
            }
            Literal::Null => {
                self.emit("null");
            }
            Literal::Unit => {
                self.emit("undefined");
            }
        }
    }

    fn binary_op_to_js(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "===",
            BinaryOp::NotEq => "!==",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }

    fn compound_op_to_js(&self, op: &CompoundAssignOp) -> &'static str {
        match op {
            CompoundAssignOp::AddAssign => "+",
            CompoundAssignOp::SubAssign => "-",
            CompoundAssignOp::MulAssign => "*",
            CompoundAssignOp::DivAssign => "/",
        }
    }

    fn convert_method_name(&self, name: &str) -> String {
        // Convert Rust-style methods to JS equivalents
        match name {
            "starts_with" => "startsWith".to_string(),
            "ends_with" => "endsWith".to_string(),
            "contains" => "includes".to_string(),
            "len" => "length".to_string(),
            "is_empty" => "length === 0".to_string(), // Special case
            "to_uppercase" => "toUpperCase".to_string(),
            "to_lowercase" => "toLowerCase".to_string(),
            "push" => "push".to_string(),
            "clone" => "".to_string(), // Clone is no-op in JS, strip it
            "unwrap_or" => "??".to_string(), // Will need special handling
            // ReluxScript to Babel field mappings
            "stmts" => "body".to_string(), // BlockStatement.stmts -> BlockStatement.body
            "is_if_statement" => "isIfStatement".to_string(),
            "is_abstract" => "abstract".to_string(),
            _ => name.to_string(),
        }
    }

    /// Generate matches! macro as Babel type checks
    fn gen_matches_macro(&mut self, scrutinee: &Expr, pattern: &Expr) {
        self.emit("(");
        self.gen_matches_pattern(scrutinee, pattern);
        self.emit(")");
    }

    /// Generate format! macro as JavaScript template literal
    fn gen_format_macro(&mut self, args: &[Expr]) {
        if args.is_empty() {
            self.emit("\"\"");
            return;
        }

        // First argument is the format string
        let format_str = match &args[0] {
            Expr::Literal(Literal::String(s)) => s.clone(),
            _ => {
                // If not a string literal, fall back to regular function call
                self.emit("String(");
                self.gen_expr(&args[0]);
                self.emit(")");
                return;
            }
        };

        // Parse the format string and replace {} with ${arg}
        let remaining_args = &args[1..];
        let mut arg_index = 0;
        let mut result = String::new();
        let mut chars = format_str.chars().peekable();

        while let Some(ch) = chars.next() {
            if ch == '{' {
                if chars.peek() == Some(&'{') {
                    // Escaped {{ -> {
                    chars.next();
                    result.push('{');
                } else if chars.peek() == Some(&'}') {
                    // {} placeholder
                    chars.next();
                    if arg_index < remaining_args.len() {
                        result.push_str("${");
                        result.push_str(&self.expr_to_string(&remaining_args[arg_index]));
                        result.push('}');
                        arg_index += 1;
                    } else {
                        result.push_str("{}");
                    }
                } else {
                    result.push(ch);
                }
            } else if ch == '}' {
                if chars.peek() == Some(&'}') {
                    // Escaped }} -> }
                    chars.next();
                    result.push('}');
                } else {
                    result.push(ch);
                }
            } else if ch == '`' {
                // Escape backticks in template literal
                result.push_str("\\`");
            } else if ch == '$' {
                // Escape $ to prevent unintended interpolation
                result.push_str("\\$");
            } else {
                result.push(ch);
            }
        }

        self.emit("`");
        self.emit(&result);
        self.emit("`");
    }

    /// Generate code for codegen::generate() and codegen::generate_with_options()
    fn gen_codegen_call(&mut self, function_name: &str, args: &[Expr]) {
        // Mark that we use the codegen module
        self.uses_codegen = true;

        match function_name {
            "generate" => {
                // codegen::generate(node) -> generate(node).code
                // We use the @babel/generator package
                if args.is_empty() {
                    self.emit("\"\"");
                    return;
                }

                self.emit("generate(");
                self.gen_expr(&args[0]);
                self.emit(").code");
            }
            "generate_with_options" => {
                // codegen::generate_with_options(node, options) -> generate(node, options).code
                if args.is_empty() {
                    self.emit("\"\"");
                    return;
                }

                self.emit("generate(");
                self.gen_expr(&args[0]);

                // If there's a second argument (options), convert it
                if args.len() > 1 {
                    self.emit(", ");
                    self.gen_codegen_options(&args[1]);
                }

                self.emit(").code");
            }
            _ => {
                // Unknown codegen function, emit as-is
                self.emit(&format!("codegen.{}(", function_name));
                for (i, arg) in args.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.gen_expr(arg);
                }
                self.emit(")");
            }
        }
    }

    /// Convert ReluxScript CodegenOptions struct to Babel generator options
    fn gen_codegen_options(&mut self, options_expr: &Expr) {
        // Expected to be a StructInit for CodegenOptions
        if let Expr::StructInit(init) = options_expr {
            if init.name == "CodegenOptions" {
                self.emit("{");
                let mut first = true;

                for (field_name, field_value) in &init.fields {
                    if !first {
                        self.emit(", ");
                    }
                    first = false;

                    // Map ReluxScript field names to Babel generator options
                    match field_name.as_str() {
                        "compact" => {
                            self.emit("compact: ");
                            self.gen_expr(field_value);
                        }
                        "minified" => {
                            self.emit("minified: ");
                            self.gen_expr(field_value);
                        }
                        "quotes" => {
                            // quotes: QuoteStyle::Single -> quotes: "single"
                            self.emit("quotes: ");
                            if let Expr::Member(mem) = field_value {
                                if let Expr::Ident(enum_name) = mem.object.as_ref() {
                                    if enum_name.name == "QuoteStyle" {
                                        let quote_val = match mem.property.as_str() {
                                            "Single" => "\"single\"",
                                            "Double" => "\"double\"",
                                            _ => "\"double\"",
                                        };
                                        self.emit(quote_val);
                                    } else {
                                        self.gen_expr(field_value);
                                    }
                                } else {
                                    self.gen_expr(field_value);
                                }
                            } else {
                                self.gen_expr(field_value);
                            }
                        }
                        "semicolons" => {
                            // Note: Babel doesn't have a direct semicolons option
                            // We could emit it anyway for future compatibility
                            self.emit("semicolons: ");
                            self.gen_expr(field_value);
                        }
                        _ => {
                            // Pass through unknown options
                            self.emit(field_name);
                            self.emit(": ");
                            self.gen_expr(field_value);
                        }
                    }
                }

                self.emit("}");
                return;
            }
        }

        // Fallback: just emit the expression as-is
        self.gen_expr(options_expr);
    }

    /// Recursively generate type checks for a pattern
    fn gen_matches_pattern(&mut self, scrutinee: &Expr, pattern: &Expr) {
        match pattern {
            Expr::StructInit(init) => {
                // Check if this is a wildcard pattern TypeName(_)
                if init.fields.len() == 1 && init.fields[0].0 == "_wildcard" {
                    // Wildcard pattern - just check the type
                    let type_name = &init.name;

                    // Use mapping to get the correct Babel type checker
                    let checker = get_node_mapping(type_name)
                        .map(|m| m.babel_checker.to_string())
                        .unwrap_or_else(|| format!("is{}", type_name));
                    self.emit(&format!("t.{}(", checker));
                    self.gen_expr(scrutinee);
                    self.emit(")");
                    return;
                }

                // Generate t.isType(scrutinee, { field: value, ... })
                let type_name = &init.name;

                // Use mapping to get the correct Babel type checker
                let checker = get_node_mapping(type_name)
                    .map(|m| m.babel_checker.to_string())
                    .unwrap_or_else(|| format!("is{}", type_name));
                self.emit(&format!("t.{}(", checker));
                self.gen_expr(scrutinee);
                self.emit(")");

                // Generate field checks
                for (field_name, field_pattern) in &init.fields {
                    self.emit(" && ");

                    // Create the field access expression
                    let field_scrutinee = format!("{}.{}", self.expr_to_string(scrutinee), field_name);

                    match field_pattern {
                        Expr::Literal(Literal::String(s)) => {
                            // Simple string equality check
                            self.emit(&format!("{} === \"{}\"", field_scrutinee, s));
                        }
                        Expr::StructInit(_) => {
                            // Nested struct pattern - recurse
                            // We need to create a fake Ident expression for the field access
                            let field_expr = Expr::Ident(IdentExpr {
                                name: field_scrutinee.clone(),
                                span: crate::lexer::Span::new(0, 0, 0, 0),
                            });
                            self.gen_matches_pattern(&field_expr, field_pattern);
                        }
                        _ => {
                            // Other patterns - generate equality
                            self.emit(&field_scrutinee);
                            self.emit(" === ");
                            self.gen_expr(field_pattern);
                        }
                    }
                }
            }
            Expr::Ident(ident) => {
                // Check if this is a type name (AST node type)
                // If so, generate t.isTypeName(scrutinee)
                let type_name = &ident.name;

                // Use mapping to get the correct Babel type checker
                if let Some(mapping) = get_node_mapping(type_name) {
                    self.emit(&format!("t.{}(", mapping.babel_checker));
                    self.gen_expr(scrutinee);
                    self.emit(")");
                } else {
                    // Fallback: assume it's an AST type and generate isTypeName
                    self.emit(&format!("t.is{}(", type_name));
                    self.gen_expr(scrutinee);
                    self.emit(")");
                }
            }
            _ => {
                // For other patterns (literals, etc.), generate equality check
                self.gen_expr(scrutinee);
                self.emit(" === ");
                self.gen_expr(pattern);
            }
        }
    }

    /// Generate type checks for a pattern (new version that works with Pattern AST)
    fn gen_matches_pattern_new(&mut self, scrutinee: &Expr, pattern: &Pattern) {
        match pattern {
            Pattern::Wildcard => {
                // Wildcard matches everything
                self.emit("true");
            }
            Pattern::Ident(_) => {
                // Identifier pattern matches everything (binds a variable)
                self.emit("true");
            }
            Pattern::Literal(lit) => {
                // Literal pattern: equality check
                self.gen_expr(scrutinee);
                self.emit(" === ");
                self.emit(&format!("{:?}", lit)); // Quick hack - should use gen_literal
            }
            Pattern::Variant { name, inner } => {
                // Variant pattern: check type and optionally inner pattern
                // Use mapping to get the correct Babel type checker
                let checker = get_node_mapping(name)
                    .map(|m| m.babel_checker.to_string())
                    .unwrap_or_else(|| format!("is{}", name));
                self.emit(&format!("t.{}(", checker));
                self.gen_expr(scrutinee);
                self.emit(")");

                // If there's an inner pattern, we'd need to check it too
                // For now, wildcard inner patterns are ignored
            }
            Pattern::Struct { name, .. } => {
                // Struct pattern: check type
                let checker = get_node_mapping(name)
                    .map(|m| m.babel_checker.to_string())
                    .unwrap_or_else(|| format!("is{}", name));
                self.emit(&format!("t.{}(", checker));
                self.gen_expr(scrutinee);
                self.emit(")");
            }
            Pattern::Or(patterns) => {
                // OR pattern: any of the patterns can match
                self.emit("(");
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.emit(" || ");
                    }
                    self.gen_matches_pattern_new(scrutinee, pat);
                }
                self.emit(")");
            }
            Pattern::Tuple(_) => {
                // Tuple pattern - not commonly used in matches!, just check truthiness
                self.emit("true");
            }
            Pattern::Ref { pattern, .. } => {
                // Ref pattern - check inner pattern
                self.gen_matches_pattern_new(scrutinee, pattern);
            }
            Pattern::Array(_) => {
                // Array pattern - check if it's an array
                self.emit("Array.isArray(");
                self.gen_expr(scrutinee);
                self.emit(")");
            }
            Pattern::Object(_) => {
                // Object pattern - check if it's an object
                self.emit("typeof ");
                self.gen_expr(scrutinee);
                self.emit(" === 'object'");
            }
            Pattern::Rest(_) => {
                // Rest pattern matches everything remaining
                self.emit("true");
            }
        }
    }

    /// Convert ReluxScript AST type names to Babel builder function names
    fn ast_type_to_babel_builder(&self, type_name: &str) -> Option<String> {
        // Use mapping module to get the Babel builder name
        // The builder name is typically camelCase of the type name
        if let Some(mapping) = get_node_mapping(type_name) {
            // Convert PascalCase to camelCase for builder function
            let babel_name = mapping.babel;
            let mut chars = babel_name.chars();
            let builder = match chars.next() {
                None => return None,
                Some(c) => c.to_lowercase().collect::<String>() + chars.as_str(),
            };
            return Some(builder);
        }

        // Fallback for types not in mapping (like literals)
        let builder = match type_name {
            "StringLiteral" => "stringLiteral",
            "NumericLiteral" => "numericLiteral",
            "BooleanLiteral" => "booleanLiteral",
            "NullLiteral" => "nullLiteral",
            "SpreadElement" => "spreadElement",
            "VariableDeclarator" => "variableDeclarator",
            "CatchClause" => "catchClause",
            "ObjectProperty" => "objectProperty",
            "ObjectMethod" => "objectMethod",
            "ClassMethod" => "classMethod",
            "ClassProperty" => "classProperty",
            "RestElement" => "restElement",
            "AssignmentPattern" => "assignmentPattern",
            _ => return None,
        };
        Some(builder.to_string())
    }

    /// Generate Babel node construction call
    fn gen_babel_node_construction(&mut self, builder: &str, fields: &[(String, Expr)]) {
        self.emit(&format!("t.{}(", builder));

        // Different builders take different argument orders
        // We'll generate based on common patterns
        match builder {
            "identifier" => {
                // t.identifier(name)
                if let Some((_, value)) = fields.iter().find(|(k, _)| k == "name") {
                    self.gen_expr(value);
                }
            }
            "stringLiteral" => {
                // t.stringLiteral(value)
                if let Some((_, value)) = fields.iter().find(|(k, _)| k == "value") {
                    self.gen_expr(value);
                }
            }
            "numericLiteral" => {
                // t.numericLiteral(value)
                if let Some((_, value)) = fields.iter().find(|(k, _)| k == "value") {
                    self.gen_expr(value);
                }
            }
            "booleanLiteral" => {
                // t.booleanLiteral(value)
                if let Some((_, value)) = fields.iter().find(|(k, _)| k == "value") {
                    self.gen_expr(value);
                }
            }
            "callExpression" => {
                // t.callExpression(callee, arguments)
                if let Some((_, callee)) = fields.iter().find(|(k, _)| k == "callee") {
                    self.gen_expr(callee);
                }
                self.emit(", ");
                if let Some((_, args)) = fields.iter().find(|(k, _)| k == "arguments") {
                    self.gen_expr(args);
                } else {
                    self.emit("[]");
                }
            }
            "memberExpression" => {
                // t.memberExpression(object, property, computed, optional)
                if let Some((_, obj)) = fields.iter().find(|(k, _)| k == "object") {
                    self.gen_expr(obj);
                }
                self.emit(", ");
                if let Some((_, prop)) = fields.iter().find(|(k, _)| k == "property") {
                    self.gen_expr(prop);
                }
                // Add computed flag if present
                if let Some((_, computed)) = fields.iter().find(|(k, _)| k == "computed") {
                    self.emit(", ");
                    self.gen_expr(computed);
                }
            }
            "expressionStatement" => {
                // t.expressionStatement(expression)
                if let Some((_, expr)) = fields.iter().find(|(k, _)| k == "expression") {
                    self.gen_expr(expr);
                }
            }
            "returnStatement" => {
                // t.returnStatement(argument)
                if let Some((_, arg)) = fields.iter().find(|(k, _)| k == "argument") {
                    self.gen_expr(arg);
                } else {
                    self.emit("null");
                }
            }
            "blockStatement" => {
                // t.blockStatement(body)
                if let Some((_, body)) = fields.iter().find(|(k, _)| k == "body") {
                    self.gen_expr(body);
                } else {
                    self.emit("[]");
                }
            }
            "variableDeclaration" => {
                // t.variableDeclaration(kind, declarations)
                if let Some((_, kind)) = fields.iter().find(|(k, _)| k == "kind") {
                    self.gen_expr(kind);
                } else {
                    self.emit("\"const\"");
                }
                self.emit(", ");
                if let Some((_, decls)) = fields.iter().find(|(k, _)| k == "declarations") {
                    self.gen_expr(decls);
                } else {
                    self.emit("[]");
                }
            }
            "variableDeclarator" => {
                // t.variableDeclarator(id, init)
                if let Some((_, id)) = fields.iter().find(|(k, _)| k == "id") {
                    self.gen_expr(id);
                }
                if let Some((_, init)) = fields.iter().find(|(k, _)| k == "init") {
                    self.emit(", ");
                    self.gen_expr(init);
                }
            }
            "jsxElement" => {
                // t.jsxElement(openingElement, closingElement, children)
                if let Some((_, open)) = fields.iter().find(|(k, _)| k == "openingElement") {
                    self.gen_expr(open);
                }
                self.emit(", ");
                if let Some((_, close)) = fields.iter().find(|(k, _)| k == "closingElement") {
                    self.gen_expr(close);
                } else {
                    self.emit("null");
                }
                self.emit(", ");
                if let Some((_, children)) = fields.iter().find(|(k, _)| k == "children") {
                    self.gen_expr(children);
                } else {
                    self.emit("[]");
                }
            }
            "jsxIdentifier" => {
                // t.jsxIdentifier(name)
                if let Some((_, name)) = fields.iter().find(|(k, _)| k == "name") {
                    self.gen_expr(name);
                }
            }
            "jsxAttribute" => {
                // t.jsxAttribute(name, value)
                if let Some((_, name)) = fields.iter().find(|(k, _)| k == "name") {
                    self.gen_expr(name);
                }
                if let Some((_, value)) = fields.iter().find(|(k, _)| k == "value") {
                    self.emit(", ");
                    self.gen_expr(value);
                }
            }
            "jsxExpressionContainer" => {
                // t.jsxExpressionContainer(expression)
                if let Some((_, expr)) = fields.iter().find(|(k, _)| k == "expression") {
                    self.gen_expr(expr);
                }
            }
            _ => {
                // For other builders, generate as object with fields
                self.emit("{ ");
                for (i, (name, value)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.emit(name);
                    self.emit(": ");
                    self.gen_expr(value);
                }
                self.emit(" }");
            }
        }

        self.emit(")");
    }
}

impl Default for BabelGenerator {
    fn default() -> Self {
        Self::new()
    }
}
