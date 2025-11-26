//! Module: structures
use super::SwcGenerator;
use crate::parser::*;

impl SwcGenerator {
    pub(super) fn gen_struct(&mut self, s: &StructDecl) {
        if self.uses_json {
            self.emit_line("#[derive(Debug, Clone, Serialize, Deserialize)]");
        } else {
            self.emit_line("#[derive(Debug, Clone)]");
        }
        self.emit_line(&format!("pub struct {} {{", s.name));
        self.indent += 1;
        for field in &s.fields {
            self.emit_indent();
            self.emit(&format!("pub {}: {},\n", field.name, self.type_to_rust(&field.ty)));
        }
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }
    pub(super) fn gen_enum(&mut self, e: &EnumDecl) {
        if self.uses_json {
            self.emit_line("#[derive(Debug, Clone, Serialize, Deserialize)]");
        } else {
            self.emit_line("#[derive(Debug, Clone)]");
        }
        self.emit_line(&format!("pub enum {} {{", e.name));
        self.indent += 1;
        for variant in &e.variants {
            self.emit_indent();
            match &variant.fields {
                EnumVariantFields::Tuple(fields) => {
                    let types: Vec<String> = fields.iter().map(|t| self.type_to_rust(t)).collect();
                    self.emit(&format!("{}({}),\n", variant.name, types.join(", ")));
                }
                EnumVariantFields::Struct(named_fields) => {
                    self.emit(&format!("{} {{\n", variant.name));
                    self.indent += 1;
                    for (field_name, field_type) in named_fields {
                        self.emit_indent();
                        self.emit(&format!("{}: {},\n", field_name, self.type_to_rust(field_type)));
                    }
                    self.indent -= 1;
                    self.emit_indent();
                    self.emit("},\n");
                }
                EnumVariantFields::Unit => {
                    self.emit(&format!("{},\n", variant.name));
                }
            }
        }
        self.indent -= 1;
        self.emit_line("}");
        self.emit_line("");
    }
    pub(super) fn gen_helper_function(&mut self, f: &FnDecl) {
        let pub_str = if f.is_pub { "pub " } else { "" };

        // Check if this is an associated function (no self parameter)
        let has_self = f.params.iter().any(|p| p.name == "self");
        if !has_self {
            self.associated_functions.insert(f.name.clone());
        }

        // Generate type parameters: <F, T>
        let type_params = if !f.type_params.is_empty() {
            let params: Vec<String> = f.type_params.iter().map(|p| p.name.clone()).collect();
            format!("<{}>", params.join(", "))
        } else {
            String::new()
        };

        let params: Vec<String> = f.params.iter().map(|p| {
            format!("{}: {}", p.name, self.type_to_rust(&p.ty))
        }).collect();

        let ret_type = f.return_type.as_ref()
            .map(|t| format!(" -> {}", self.type_to_rust(t)))
            .unwrap_or_default();

        // Generate where clause
        let where_clause = if !f.where_clause.is_empty() {
            let predicates: Vec<String> = f.where_clause.iter().map(|p| {
                format!("    {}: {}", p.target, self.type_to_rust(&p.bound))
            }).collect();
            format!("\nwhere\n{}", predicates.join(",\n"))
        } else {
            String::new()
        };

        self.emit_line(&format!("{}fn {}{}({}){}{} {{", pub_str, f.name, type_params, params.join(", "), ret_type, where_clause));
        self.indent += 1;

        // Track parameter types in the environment
        self.type_env.push_scope();
        for param in &f.params {
            let param_ctx = self.type_from_ast(&param.ty);
            #[cfg(debug_assertions)]
            eprintln!("[swc] param {} : {:?} -> swc_type={}",
                param.name, param.ty, param_ctx.swc_type);
            self.type_env.define(&param.name, param_ctx);
        }

        self.gen_block(&f.body);

        self.type_env.pop_scope();
        self.indent -= 1;
        self.emit_line("}");
    }

    /// Generate code for codegen::generate() and codegen::generate_with_options()
    pub(super) fn gen_codegen_call(&mut self, function_name: &str, args: &[Expr]) {
        match function_name {
            "generate" => {
                // codegen::generate(node) -> codegen_to_string(node)
                if args.is_empty() {
                    self.emit("String::new()");
                    return;
                }

                self.emit("codegen_to_string(");
                self.gen_expr(&args[0]);
                self.emit(")");
            }
            "generate_with_options" => {
                // codegen::generate_with_options(node, options) -> codegen_to_string_with_config(node, config)
                if args.is_empty() {
                    self.emit("String::new()");
                    return;
                }

                self.emit("codegen_to_string_with_config(");
                self.gen_expr(&args[0]);

                // If there's a second argument (options), convert it
                if args.len() > 1 {
                    self.emit(", ");
                    self.gen_codegen_config(&args[1]);
                } else {
                    self.emit(", CodegenConfig::default()");
                }

                self.emit(")");
            }
            _ => {
                // Unknown codegen function, emit as-is
                self.emit(&format!("codegen::{}(", function_name));
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

    /// Convert ReluxScript CodegenOptions struct to SWC codegen Config
    pub(super) fn gen_codegen_config(&mut self, options_expr: &Expr) {
        // Expected to be a StructInit for CodegenOptions
        if let Expr::StructInit(init) = options_expr {
            if init.name == "CodegenOptions" {
                self.emit("CodegenConfig { ");

                for (field_name, field_value) in &init.fields {
                    // Map ReluxScript field names to SWC codegen Config fields
                    match field_name.as_str() {
                        "minified" => {
                            self.emit("minify: ");
                            self.gen_expr(field_value);
                            self.emit(", ");
                        }
                        "compact" | "semicolons" => {
                            // SWC doesn't have direct equivalents for these
                            // We can add them as comments or ignore
                        }
                        "quotes" => {
                            // SWC doesn't have a quotes option in the same way
                            // Could potentially use it if we extend the config
                        }
                        _ => {
                            // Pass through unknown options
                        }
                    }
                }

                self.emit("..Default::default() }");
                return;
            }
        }

        // Fallback: just use default config
        self.emit("CodegenConfig::default()");
    }

    /// Generate helper functions for the parser module
    pub(super) fn gen_parser_module_helpers(&mut self) {
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
        self.emit_line("");

        // parser::parse_with_syntax
        self.emit_line("pub fn parse_with_syntax(code: &str, syntax_type: &str) -> Result<Program, String> {");
        self.indent += 1;
        self.emit_line("let source_map = Arc::new(SourceMap::default());");
        self.emit_line("let file = source_map.new_source_file(");
        self.indent += 1;
        self.emit_line("FileName::Anon,");
        self.emit_line("code.to_string(),");
        self.indent -= 1;
        self.emit_line(");");
        self.emit_line("let syntax = match syntax_type {");
        self.indent += 1;
        self.emit_line("\"TypeScript\" => Syntax::Typescript(TsConfig {");
        self.indent += 1;
        self.emit_line("tsx: true,");
        self.emit_line("decorators: false,");
        self.emit_line("..Default::default()");
        self.indent -= 1;
        self.emit_line("}),");
        self.emit_line("\"JSX\" => Syntax::Es(EsConfig {");
        self.indent += 1;
        self.emit_line("jsx: true,");
        self.emit_line("..Default::default()");
        self.indent -= 1;
        self.emit_line("}),");
        self.emit_line("_ => Syntax::Es(EsConfig::default()),");
        self.indent -= 1;
        self.emit_line("};");
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

    /// Generate helper functions for the codegen module
    pub(super) fn gen_codegen_module_helpers(&mut self) {
        // Generate a helper function that converts an AST node to a string
        self.emit_line("fn codegen_to_string<N: swc_ecma_visit::Node>(node: &N) -> String {");
        self.indent += 1;
        self.emit_line("let mut buf = vec![];");
        self.emit_line("{");
        self.indent += 1;
        self.emit_line("let cm = Arc::new(SourceMap::default());");
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
        self.emit_line("");

        // Generate helper function with config
        self.emit_line("fn codegen_to_string_with_config<N: swc_ecma_visit::Node>(node: &N, cfg: CodegenConfig) -> String {");
        self.indent += 1;
        self.emit_line("let mut buf = vec![];");
        self.emit_line("{");
        self.indent += 1;
        self.emit_line("let cm = Arc::new(SourceMap::default());");
        self.emit_line("let mut emitter = Emitter {");
        self.indent += 1;
        self.emit_line("cfg,");
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
}
