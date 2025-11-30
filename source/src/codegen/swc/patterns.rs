//! Code generation for expressions/statements/patterns
use super::SwcGenerator;
use crate::parser::*;
use crate::codegen::decorated_ast::*;
use crate::codegen::type_context::{TypeContext, TypeEnvironment};
use crate::mapping::{get_node_mapping, get_swc_variant_in_context, get_field_mapping};

impl SwcGenerator {
    pub(super) fn gen_decorated_pattern(&mut self, pattern: &DecoratedPattern) {
        // Use metadata.swc_pattern directly - no inference needed!
        self.emit(&pattern.metadata.swc_pattern);

        // Handle inner patterns recursively
        match &pattern.kind {
            DecoratedPatternKind::Variant { inner: Some(inner_pat), .. } => {
                self.emit("(");
                self.gen_decorated_pattern(inner_pat);
                self.emit(")");
            }
            DecoratedPatternKind::Tuple(patterns) => {
                self.emit("(");
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 { self.emit(", "); }
                    self.gen_decorated_pattern(p);
                }
                self.emit(")");
            }
            DecoratedPatternKind::Struct { fields, .. } => {
                self.emit(" { ");
                for (i, (name, p)) in fields.iter().enumerate() {
                    if i > 0 { self.emit(", "); }
                    self.emit(name);
                    self.emit(": ");
                    self.gen_decorated_pattern(p);
                }
                self.emit(" }");
            }
            _ => {
                // Other pattern kinds don't need special handling
            }
        }
    }

    /// Generate expression from decorated AST (uses metadata)
    pub(super) fn gen_pattern(&mut self, pattern: &Pattern) {
        match pattern {
            Pattern::Literal(lit) => self.gen_literal(lit),
            Pattern::Ident(name) => self.emit(name),
            Pattern::Wildcard => self.emit("_"),
            Pattern::Tuple(patterns) => {
                self.emit("(");
                for (i, pat) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.gen_pattern(pat);
                }
                self.emit(")");
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
            Pattern::Object(_) => {
                // Rust doesn't have object destructuring, use wildcard
                self.emit("_");
            }
            Pattern::Rest(_) => {
                // Rest patterns in Rust use ..
                self.emit("..");
            }
            Pattern::Struct { name, fields } => {
                self.emit(name);
                self.emit(" { ");
                for (i, (fname, fpat)) in fields.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.emit(fname);
                    self.emit(": ");
                    self.gen_pattern(fpat);
                }
                self.emit(" }");
            }
            Pattern::Variant { name, inner } => {
                // Map ReluxScript type names to SWC types (e.g., Expression::Identifier -> Expr::Ident)
                let swc_name = if name.contains("::") {
                    let parts: Vec<&str> = name.split("::").collect();
                    if parts.len() == 2 {
                        let enum_name = self.reluxscript_to_swc_type(parts[0]);
                        let variant_name = self.reluxscript_to_swc_type(parts[1]);
                        format!("{}::{}", enum_name, variant_name)
                    } else {
                        name.clone()
                    }
                } else {
                    // Handle standalone variant names (Some, None, Ok, Err)
                    name.clone()
                };
                self.emit(&swc_name);
                if let Some(inner_pat) = inner {
                    self.emit("(");
                    self.gen_pattern(inner_pat);
                    self.emit(")");
                }
            }
            Pattern::Or(patterns) => {
                for (i, p) in patterns.iter().enumerate() {
                    if i > 0 {
                        self.emit(" | ");
                    }
                    self.gen_pattern(p);
                }
            }
            Pattern::Ref { pattern: inner, .. } => {
                // Preserve ref in Rust
                self.emit("ref ");
                self.gen_pattern(inner);
            }
        }
    }
    pub(super) fn gen_swc_pattern_check(&mut self, scrutinee: &Expr, pattern: &Expr, depth: usize) {
        match pattern {
            Expr::StructInit(init) => {
                // Check if this is a wildcard pattern TypeName(_)
                if init.fields.len() == 1 && init.fields[0].0 == "_wildcard" {
                    // Wildcard pattern - just check the type
                    self.emit("matches!(");
                    self.gen_expr(scrutinee);
                    self.emit(", ");

                    // Use mapping module for SWC enum variants
                    if let Some(mapping) = get_node_mapping(&init.name) {
                        // Extract the enum variant pattern (e.g., "Expr::Call(_)")
                        let pattern_parts: Vec<&str> = mapping.swc_pattern.split('(').collect();
                        if let Some(variant) = pattern_parts.get(0) {
                            self.emit(&format!("{}(_)", variant));
                        } else {
                            self.emit(&format!("{}(_)", mapping.swc_pattern));
                        }
                    } else {
                        // Fallback for unknown types
                        self.emit(&format!("{}(_)", init.name));
                    }
                    self.emit(")");
                    return;
                }

                let swc_type = self.reluxscript_to_swc_type(&init.name.to_lowercase());

                // For nested patterns, we need to check the type and fields
                // Generate: matches!(scrutinee, SomeType { .. }) && field checks

                self.emit("matches!(");
                self.gen_expr(scrutinee);
                self.emit(", ");

                // Use mapping module for SWC enum variants
                if let Some(mapping) = get_node_mapping(&init.name) {
                    // Use the swc_pattern from mapping (e.g., "Expr::Ident(ident)")
                    // but we need just the pattern without binding
                    let pattern = mapping.swc_pattern.replace(|c: char| c.is_lowercase() || c == '_', "");
                    let pattern = if pattern.ends_with("()") {
                        pattern.replace("()", "(_)")
                    } else {
                        format!("{}(_)", mapping.swc_pattern.split('(').next().unwrap_or(&mapping.swc_pattern))
                    };
                    self.emit(&pattern);
                } else if init.name == "StringLiteral" {
                    self.emit("Expr::Lit(Lit::Str(_))");
                } else {
                    self.emit(&format!("{}{{ .. }}", swc_type));
                }
                self.emit(")");

                // Generate field checks with && chains
                for (field_name, field_pattern) in &init.fields {
                    self.emit(" && ");

                    // Generate field access - need to unwrap the enum first
                    let unwrap_prefix = if let Some(mapping) = get_node_mapping(&init.name) {
                        // Generate: if let Pattern = &expr { ... }
                        format!("if let {} = &", mapping.swc_pattern)
                    } else {
                        String::new()
                    };

                    if !unwrap_prefix.is_empty() {
                        self.emit("{ ");
                        self.emit(&unwrap_prefix);
                        self.gen_expr(scrutinee);
                        self.emit(" { ");
                    }

                    let field_var = format!("__f{}", depth);
                    match field_pattern {
                        Expr::Literal(Literal::String(s)) => {
                            // String equality check on field
                            // Use field mapping to get correct SWC field name
                            let swc_field = get_field_mapping(&init.name, field_name)
                                .map(|m| m.swc)
                                .unwrap_or(field_name.as_str());

                            // Get the variable name from the pattern binding
                            let var_name = get_node_mapping(&init.name)
                                .map(|m| {
                                    // Extract binding name from pattern like "Expr::Ident(ident)" -> "ident"
                                    m.swc_pattern
                                        .split('(')
                                        .nth(1)
                                        .and_then(|s| s.strip_suffix(')'))
                                        .unwrap_or("n")
                                })
                                .unwrap_or("n");

                            self.emit(&format!("&*{}.{} == \"{}\"", var_name, swc_field, s));
                        }
                        Expr::StructInit(nested) => {
                            // Nested pattern - recursive check
                            // Get the variable name from the pattern binding
                            let obj_var = get_node_mapping(&init.name)
                                .map(|m| {
                                    m.swc_pattern
                                        .split('(')
                                        .nth(1)
                                        .and_then(|s| s.strip_suffix(')'))
                                        .unwrap_or("n")
                                })
                                .unwrap_or("n");

                            // Use field mapping for the field name
                            let swc_field = get_field_mapping(&init.name, field_name)
                                .map(|m| m.swc)
                                .unwrap_or(field_name.as_str());

                            // Handle MemberProp specially - it's not an Expr
                            if init.name == "MemberExpression" && field_name == "property" {
                                // MemberProp needs special handling
                                if nested.name == "Identifier" {
                                    // MemberProp::Ident
                                    self.emit(&format!("matches!({}.prop, MemberProp::Ident(_))", obj_var));
                                    // Check name if specified
                                    for (nested_field, nested_val) in &nested.fields {
                                        if nested_field == "name" {
                                            if let Expr::Literal(Literal::String(s)) = nested_val {
                                                self.emit(&format!(" && {{ if let MemberProp::Ident(id) = &{}.prop {{ &*id.sym == \"{}\" }} else {{ false }} }}", obj_var, s));
                                            }
                                        }
                                    }
                                } else {
                                    self.emit("true /* unsupported MemberProp pattern */");
                                }
                            } else {
                                // Create a synthetic expression for the field access
                                let field_access = Expr::Ident(IdentExpr {
                                    name: format!("(*{}.{})", obj_var, swc_field),
                                    span: crate::lexer::Span::new(0, 0, 0, 0),
                                });
                                self.gen_swc_pattern_check(&field_access, &Expr::StructInit(nested.clone()), depth + 1);
                            }
                        }
                        _ => {
                            self.emit("true");
                        }
                    }

                    if !unwrap_prefix.is_empty() {
                        self.emit(" } else { false } }");
                    }
                }
            }
            Expr::Ident(ident) => {
                // Type check for a simple type name (e.g., MemberExpression)
                let type_name = &ident.name;

                // Use mapping to get the correct SWC pattern
                if let Some(mapping) = get_node_mapping(type_name) {
                    // Generate: matches!(scrutinee, Expr::Member(_))
                    self.emit("matches!(");
                    self.gen_expr(scrutinee);
                    self.emit(", ");

                    // Extract just the pattern part (e.g., "Expr::Member(_)")
                    let pattern_str = mapping.swc_pattern;
                    // Replace any binding variable with _
                    let pattern = if pattern_str.contains('(') {
                        let parts: Vec<&str> = pattern_str.split('(').collect();
                        format!("{}(_)", parts[0])
                    } else {
                        format!("{}{{ .. }}", pattern_str)
                    };
                    self.emit(&pattern);
                    self.emit(")");
                } else {
                    // Fallback: try common mappings
                    let swc_pattern = match type_name.as_str() {
                        "MemberExpression" => "Expr::Member(_)",
                        "CallExpression" => "Expr::Call(_)",
                        "Identifier" => "Expr::Ident(_)",
                        "FunctionDeclaration" => "Decl::Fn(_)",
                        "VariableDeclaration" => "Decl::Var(_)",
                        "ReturnStatement" => "Stmt::Return(_)",
                        "IfStatement" => "Stmt::If(_)",
                        "BlockStatement" => "Stmt::Block(_)",
                        _ => {
                            // Generate a placeholder
                            self.gen_expr(scrutinee);
                            self.emit(&format!(".is_{}()", type_name.to_lowercase()));
                            return;
                        }
                    };
                    self.emit(&format!("matches!("));
                    self.gen_expr(scrutinee);
                    self.emit(&format!(", {})", swc_pattern));
                }
            }
            _ => {
                // Literal or other pattern - generate equality check
                self.gen_expr(scrutinee);
                self.emit(" == ");
                self.gen_expr(pattern);
            }
        }
    }
}

impl Default for SwcGenerator {
}
