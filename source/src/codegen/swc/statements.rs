//! Code generation for expressions/statements/patterns
use super::SwcGenerator;
use crate::parser::*;
use crate::codegen::decorated_ast::*;
use crate::codegen::type_context::{TypeContext, TypeEnvironment};
use crate::mapping::{get_node_mapping, get_swc_variant_in_context, get_field_mapping};

impl SwcGenerator {
    pub(super) fn gen_block(&mut self, block: &Block) {
        let len = block.stmts.len();
        for (i, stmt) in block.stmts.iter().enumerate() {
            let is_last = i == len - 1;
            self.gen_stmt_with_context(stmt, is_last);
        }
    }
    pub(super) fn gen_stmt_with_context(&mut self, stmt: &Stmt, is_last_in_block: bool) {
        // If this is the last statement in a block and it's an expression,
        // check if it's actually a return value or just a statement
        match stmt {
            Stmt::Expr(expr_stmt) if is_last_in_block => {
                // Check if this expression produces a meaningful return value
                // Calls to push(), insert(), etc. return () so they need semicolons
                let needs_semicolon = match &expr_stmt.expr {
                    Expr::Call(call) => {
                        // Check if it's a method call to a mutating method
                        if let Expr::Member(mem) = call.callee.as_ref() {
                            matches!(mem.property.as_str(),
                                "push" | "insert" | "remove" | "clear" | "append" |
                                "pop" | "push_str" | "extend" | "drain")
                        } else {
                            false
                        }
                    }
                    Expr::Assign(_) => true,  // Assignments return ()
                    _ => false,
                };

                self.emit_indent();
                self.gen_expr(&expr_stmt.expr);
                if needs_semicolon {
                    self.emit(";");
                }
                self.emit("\n");
            }
            _ => self.gen_stmt(stmt),
        }
    }
    pub(super) fn gen_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_stmt) => {
                // Determine the type: use explicit annotation if available, else infer
                let var_type = if let Some(ty) = &let_stmt.ty {
                    // Use explicit type annotation
                    self.type_from_ast(ty)
                } else {
                    // Infer the type from the initializer expression
                    self.infer_type(&let_stmt.init)
                };

                self.emit_indent();
                if let_stmt.mutable {
                    self.emit("let mut ");
                } else {
                    self.emit("let ");
                }
                self.gen_pattern(&let_stmt.pattern);
                // Don't emit type annotations for internal temp variables
                // The type environment tracking is what matters
                self.emit(" = ");
                self.gen_expr(&let_stmt.init);
                self.emit(";\n");

                // Track the variable's type in the environment (only for simple identifiers)
                if let Pattern::Ident(name) = &let_stmt.pattern {
                    self.type_env.define(name, var_type);
                }
            }
            Stmt::Const(const_stmt) => {
                self.emit_indent();
                self.emit("const ");
                self.emit(&const_stmt.name);
                if let Some(ty) = &const_stmt.ty {
                    self.emit(&format!(": {}", self.type_to_rust(ty)));
                }
                self.emit(" = ");
                self.gen_expr(&const_stmt.init);
                self.emit(";\n");
            }
            Stmt::Expr(expr_stmt) => {
                self.emit_indent();
                self.gen_expr(&expr_stmt.expr);
                self.emit(";\n");
            }
            Stmt::If(if_stmt) => {
                // Check for if-let pattern first
                if let Some(pattern) = &if_stmt.pattern {
                    self.gen_if_let_stmt(if_stmt, pattern);
                } else if let Some((var_name, type_name, field_path, match_expr)) = self.extract_matches_pattern(&if_stmt.condition) {
                    // Check if condition is matches!(var, Type) or matches!(obj.field, Type)
                    // Look up the variable's type to determine the correct context
                    // Extract the first argument to infer its type
                    let var_type = if let Expr::Call(call) = &if_stmt.condition {
                        if call.args.len() >= 1 {
                            self.infer_type(&call.args[0]).swc_type.clone()
                        } else {
                            "Expr".to_string()
                        }
                    } else {
                        self.type_env.lookup(&var_name)
                            .map(|ctx| ctx.swc_type.clone())
                            .unwrap_or_else(|| "Expr".to_string())
                    };

                    // Generate if let with type narrowing using context
                    let (swc_enum, swc_variant, swc_struct) = get_swc_variant_in_context(&type_name, &var_type);
                    #[cfg(debug_assertions)]
                    eprintln!("[swc] matches!({}, {}) -> var_type={}, enum={}, variant={}, struct={}",
                        match_expr, type_name, var_type, swc_enum, swc_variant, swc_struct);

                    self.emit_indent();
                    // Handle nested patterns like Expr::Lit(Lit::Str(x))
                    let extra_close = if swc_variant.contains('(') { ")" } else { "" };
                    self.emit(&format!("if let {}::{}({}){} = &{} {{\n",
                        swc_enum, swc_variant, var_name, extra_close, match_expr));

                    self.indent += 1;
                    self.type_env.push_scope();

                    // Shadow the variable with narrowed type
                    let narrowed_ctx = TypeContext::narrowed(&type_name, &swc_struct);
                    self.type_env.define(&var_name, narrowed_ctx.clone());

                    // If this was a field access, also register the field refinement
                    if let Some(path) = &field_path {
                        self.type_env.refine_field(path, narrowed_ctx);
                    }

                    self.gen_block(&if_stmt.then_branch);

                    self.type_env.pop_scope();
                    self.indent -= 1;

                    // Handle else-if branches - each might also be a matches! pattern
                    for (cond, block) in &if_stmt.else_if_branches {
                        if let Some((else_var_name, else_type_name, else_field_path, else_match_expr)) = self.extract_matches_pattern(cond) {
                            // This else-if is also a matches! pattern
                            // Infer type from the first argument of matches!
                            let else_var_type = if let Expr::Call(call) = cond {
                                if call.args.len() >= 1 {
                                    self.infer_type(&call.args[0]).swc_type.clone()
                                } else {
                                    "Expr".to_string()
                                }
                            } else {
                                self.type_env.lookup(&else_var_name)
                                    .map(|ctx| ctx.swc_type.clone())
                                    .unwrap_or_else(|| "Expr".to_string())
                            };

                            let (else_swc_enum, else_swc_variant, else_swc_struct) =
                                get_swc_variant_in_context(&else_type_name, &else_var_type);

                            #[cfg(debug_assertions)]
                            eprintln!("[swc] else if matches!({}, {}) -> var_type={}, enum={}, variant={}, struct={}",
                                else_match_expr, else_type_name, else_var_type, else_swc_enum, else_swc_variant, else_swc_struct);

                            self.emit_indent();
                            // Handle nested patterns like Expr::Lit(Lit::Str(x))
                            let else_extra_close = if else_swc_variant.contains('(') { ")" } else { "" };
                            self.emit(&format!("}} else if let {}::{}({}){} = &{} {{\n",
                                else_swc_enum, else_swc_variant, else_var_name, else_extra_close, else_match_expr));

                            self.indent += 1;
                            self.type_env.push_scope();

                            let else_narrowed_ctx = TypeContext::narrowed(&else_type_name, &else_swc_struct);
                            self.type_env.define(&else_var_name, else_narrowed_ctx.clone());

                            // Register field refinement if needed
                            if let Some(path) = &else_field_path {
                                self.type_env.refine_field(path, else_narrowed_ctx);
                            }

                            self.gen_block(block);

                            self.type_env.pop_scope();
                            self.indent -= 1;
                        } else {
                            // Regular else-if condition
                            self.emit_indent();
                            self.emit("} else if ");
                            self.gen_expr(cond);
                            self.emit(" {\n");
                            self.indent += 1;
                            self.gen_block(block);
                            self.indent -= 1;
                        }
                    }

                    // Handle final else branch
                    if let Some(else_block) = &if_stmt.else_branch {
                        self.emit_indent();
                        self.emit("} else {\n");
                        self.indent += 1;
                        self.gen_block(else_block);
                        self.indent -= 1;
                    }

                    self.emit_line("}");
                } else {
                    // Standard if statement
                    self.emit_indent();
                    self.emit("if ");
                    self.gen_expr(&if_stmt.condition);
                    self.emit(" {\n");
                    self.indent += 1;
                    self.gen_block(&if_stmt.then_branch);
                    self.indent -= 1;

                    for (cond, block) in &if_stmt.else_if_branches {
                        self.emit_indent();
                        self.emit("} else if ");
                        self.gen_expr(cond);
                        self.emit(" {\n");
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
            Stmt::Match(match_stmt) => {
                self.emit_indent();
                self.emit("match ");
                self.gen_expr(&match_stmt.scrutinee);
                self.emit(" {\n");
                self.indent += 1;
                for arm in &match_stmt.arms {
                    self.emit_indent();
                    self.gen_pattern(&arm.pattern);
                    self.emit(" => ");
                    self.gen_expr(&arm.body);
                    self.emit(",\n");
                }
                self.indent -= 1;
                self.emit_line("}");
            }
            Stmt::For(for_stmt) => {
                self.emit_indent();
                self.emit("for ");
                self.gen_pattern(&for_stmt.pattern);
                self.emit(" in ");
                self.gen_expr(&for_stmt.iter);
                self.emit(" {\n");
                self.indent += 1;

                // Infer the type of the loop variable from the iterator (only for simple identifiers)
                self.type_env.push_scope();
                let iter_type = self.infer_type(&for_stmt.iter);
                let elem_type = self.get_element_type(&iter_type);
                if let Pattern::Ident(var_name) = &for_stmt.pattern {
                    #[cfg(debug_assertions)]
                    eprintln!("[swc] for {} in {:?} -> iter_type={:?}, elem_type={:?}",
                        var_name, for_stmt.iter, iter_type, elem_type);
                    self.type_env.define(var_name, elem_type);
                }

                self.gen_block(&for_stmt.body);

                self.type_env.pop_scope();
                self.indent -= 1;
                self.emit_line("}");
            }
            Stmt::While(while_stmt) => {
                // Check if condition is matches!(var, Type)
                if let Some((var_name, type_name, field_path, match_expr)) = self.extract_matches_pattern(&while_stmt.condition) {
                    // Look up the variable's type to determine the correct context
                    // Infer type from the first argument of matches!
                    let var_type = if let Expr::Call(call) = &while_stmt.condition {
                        if call.args.len() >= 1 {
                            self.infer_type(&call.args[0]).swc_type.clone()
                        } else {
                            "Expr".to_string()
                        }
                    } else {
                        self.type_env.lookup(&var_name)
                            .map(|ctx| ctx.swc_type.clone())
                            .unwrap_or_else(|| "Expr".to_string())
                    };

                    // Generate while let with type narrowing using context
                    let (swc_enum, swc_variant, swc_struct) = get_swc_variant_in_context(&type_name, &var_type);

                    self.emit_indent();
                    self.emit(&format!("while let {}::{}({}) = {} {{\n",
                        swc_enum, swc_variant, var_name, match_expr));

                    self.indent += 1;
                    self.type_env.push_scope();

                    // Shadow the variable with narrowed type
                    let narrowed_ctx = TypeContext::narrowed(&type_name, &swc_struct);
                    self.type_env.define(&var_name, narrowed_ctx.clone());

                    // Register field refinement if needed
                    if let Some(path) = &field_path {
                        self.type_env.refine_field(path, narrowed_ctx);
                    }

                    self.gen_block(&while_stmt.body);

                    self.type_env.pop_scope();
                    self.indent -= 1;
                    self.emit_line("}");
                } else {
                    // Standard while loop
                    self.emit_indent();
                    self.emit("while ");
                    self.gen_expr(&while_stmt.condition);
                    self.emit(" {\n");
                    self.indent += 1;
                    self.gen_block(&while_stmt.body);
                    self.indent -= 1;
                    self.emit_line("}");
                }
            }
            Stmt::Loop(loop_stmt) => {
                self.emit_line("loop {");
                self.indent += 1;
                self.gen_block(&loop_stmt.body);
                self.indent -= 1;
                self.emit_line("}");
            }
            Stmt::Return(ret) => {
                self.emit_indent();
                if let Some(value) = &ret.value {
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
            Stmt::Traverse(traverse_stmt) => {
                self.gen_traverse_stmt(traverse_stmt);
            }
            Stmt::Function(fn_decl) => {
                // Generate nested function
                self.emit_indent();
                self.emit("fn ");
                self.emit(&fn_decl.name);
                self.emit("(");
                for (i, param) in fn_decl.params.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.emit(&param.name);
                    self.emit(&format!(": {}", self.type_to_rust(&param.ty)));
                }
                self.emit(")");
                if let Some(return_type) = &fn_decl.return_type {
                    self.emit(&format!(" -> {}", self.type_to_rust(return_type)));
                }
                self.emit(" {\n");
                self.indent += 1;
                self.gen_block(&fn_decl.body);
                self.indent -= 1;
                self.emit_indent();
                self.emit("}\n");
            }
            Stmt::Verbatim(verbatim) => {
                // Emit raw code only for Rust target
                match verbatim.target {
                    VerbatimTarget::Rust => {
                        self.emit_indent();
                        self.emit(&verbatim.code);
                        if !verbatim.code.ends_with(';') && !verbatim.code.ends_with('}') {
                            self.emit(";");
                        }
                        self.emit("\n");
                    }
                    VerbatimTarget::JavaScript => {
                        // Skip - this is Babel-only code
                        self.emit_indent();
                        self.emit("// Babel-only code omitted\n");
                    }
                }
            }
        }
    }
    pub(super) fn gen_if_let_stmt(&mut self, if_stmt: &IfStmt, pattern: &Pattern) {
        // Check if this is a nested enum pattern that needs desugaring
        // e.g., Callee::MemberExpression -> Callee::Expr + Expr::Member
        if let Pattern::Variant { name, inner } = pattern {
            if name == "Callee::MemberExpression" {
                // Desugar into nested if-lets:
                // if let Callee::Expr(__callee_expr) = &node.callee {
                //     if let Expr::Member(ref member) = __callee_expr.as_ref() {
                self.emit_indent();
                self.emit("if let Callee::Expr(__callee_expr) = &");
                self.gen_expr(&if_stmt.condition);
                self.emit(" {\n");
                self.indent += 1;

                self.emit_indent();
                self.emit("if let Expr::Member(");
                if let Some(inner_pat) = inner {
                    self.gen_pattern(inner_pat);
                } else {
                    self.emit("_");
                }
                self.emit(") = __callee_expr.as_ref() {\n");

                self.indent += 1;
                self.type_env.push_scope();

                // Register the binding in type environment
                if let Some(inner_pat) = inner {
                    if let Pattern::Ref { pattern: inner_ident, .. } = inner_pat.as_ref() {
                        if let Pattern::Ident(binding) = inner_ident.as_ref() {
                            let member_type = TypeContext::narrowed("MemberExpression", "MemberExpr");
                            self.type_env.define(binding, member_type);
                        }
                    }
                }

                self.gen_block(&if_stmt.then_branch);

                self.type_env.pop_scope();
                self.indent -= 1;
                self.emit_indent();
                self.emit("}\n");

                self.indent -= 1;

                // Handle else branch
                if let Some(else_block) = &if_stmt.else_branch {
                    self.emit_indent();
                    self.emit("} else {\n");
                    self.indent += 1;
                    self.gen_block(else_block);
                    self.indent -= 1;
                }

                self.emit_line("}");
                return;
            }
        }

        // Standard if-let pattern matching for SWC/Rust
        self.emit_indent();
        self.emit("if let ");
        self.gen_pattern(pattern);
        self.emit(" = ");
        self.gen_expr(&if_stmt.condition);
        self.emit(" {\n");

        self.indent += 1;
        self.type_env.push_scope();

        // If pattern binds a variable, add it to the environment
        if let Pattern::Variant { name, inner } = pattern {
            if let Some(inner_pat) = inner {
                if let Pattern::Ident(binding) = inner_pat.as_ref() {
                    // For Some(x), the binding 'x' gets the inner type
                    // Infer the type of the condition and unwrap it
                    let cond_type = self.infer_type(&if_stmt.condition);
                    let cond_type_str = cond_type.swc_type.clone();
                    let inner_type = if name == "Some" {
                        // Unwrap Option<T> to get T
                        cond_type.unwrap_generic()
                    } else {
                        cond_type
                    };
                    #[cfg(debug_assertions)]
                    eprintln!("[swc] if let {}({}) = ... -> cond_type={}, inner_type={}",
                        name, binding, cond_type_str, inner_type.swc_type);
                    self.type_env.define(binding, inner_type);
                }
            }
        } else if let Pattern::Ident(binding) = pattern {
            let ctx = TypeContext::unknown();
            self.type_env.define(binding, ctx);
        }

        self.gen_block(&if_stmt.then_branch);

        self.type_env.pop_scope();
        self.indent -= 1;

        // Handle else branch
        if let Some(else_block) = &if_stmt.else_branch {
            self.emit_indent();
            self.emit("} else {\n");
            self.indent += 1;
            self.gen_block(else_block);
            self.indent -= 1;
        }

        self.emit_line("}");
    }
    pub(super) fn gen_traverse_stmt(&mut self, traverse_stmt: &crate::parser::TraverseStmt) {
        match &traverse_stmt.kind {
            crate::parser::TraverseKind::Inline(inline) => {
                // Generate a unique struct name for this inline visitor
                let struct_name = format!("__InlineVisitor_{}", self.hoisted_visitors.len());

                // Check if we have captures (need lifetime parameter)
                let has_captures = !traverse_stmt.captures.is_empty();
                let lifetime = if has_captures { "<'a>" } else { "" };

                // Build the hoisted struct definition
                let mut struct_def = String::new();
                struct_def.push_str(&format!("struct {}{} {{\n", struct_name, lifetime));

                // Add captured variables as reference fields
                for capture in &traverse_stmt.captures {
                    let ref_type = if capture.mutable {
                        "&'a mut"
                    } else {
                        "&'a"
                    };
                    // Look up type from environment, default to i32
                    // For simple integer variables initialized with literals, default to i32
                    let captured_type = if let Some(type_ctx) = self.type_env.lookup(&capture.name) {
                        if type_ctx.swc_type == "Unknown" {
                            "i32".to_string() // Default unknown types to i32
                        } else {
                            type_ctx.swc_type.clone()
                        }
                    } else {
                        "i32".to_string()
                    };
                    struct_def.push_str(&format!("    {}: {} {},\n", capture.name, ref_type, captured_type));
                }

                // Add local state fields
                for let_stmt in &inline.state {
                    // Only add simple identifier patterns as state fields
                    if let Pattern::Ident(name) = &let_stmt.pattern {
                        let ty = if let Some(ref ty) = let_stmt.ty {
                            self.type_to_rust(ty)
                        } else {
                            "i32".to_string() // Default type, should be inferred
                        };
                        struct_def.push_str(&format!("    {}: {},\n", name, ty));
                    }
                }
                struct_def.push_str("}\n\n");

                // Generate impl VisitMut for the struct
                let impl_lifetime = if has_captures { "<'a>" } else { "" };
                struct_def.push_str(&format!("impl{} VisitMut for {}{} {{\n", impl_lifetime, struct_name, lifetime));
                for method in &inline.methods {
                    // Convert visit_xxx to visit_mut_xxx
                    let swc_method = self.visitor_method_to_swc(&method.name);

                    // Get parameter type
                    let param_type = if !method.params.is_empty() {
                        if let crate::parser::Type::Reference { inner, .. } = &method.params[0].ty {
                            if let crate::parser::Type::Named(name) = inner.as_ref() {
                                self.reluxscript_type_to_swc(name)
                            } else {
                                "Expr".to_string()
                            }
                        } else {
                            "Expr".to_string()
                        }
                    } else {
                        "Expr".to_string()
                    };

                    // Get the original parameter name from the method
                    let param_name = if !method.params.is_empty() {
                        &method.params[0].name
                    } else {
                        "n"
                    };

                    struct_def.push_str(&format!("    fn {}(&mut self, {}: &mut {}) {{\n", swc_method, param_name, param_type));

                    // Generate method body with captured variables marked for self. prefix
                    let mut body_gen = SwcGenerator::new();
                    body_gen.indent = 2;
                    // Mark captured variables so they generate as self.var
                    for capture in &traverse_stmt.captures {
                        body_gen.captured_vars.insert(capture.name.clone());
                    }
                    // Also mark local state variables
                    for let_stmt in &inline.state {
                        if let Pattern::Ident(name) = &let_stmt.pattern {
                            body_gen.captured_vars.insert(name.clone());
                        }
                    }
                    body_gen.gen_block(&method.body);
                    struct_def.push_str(&body_gen.output);

                    struct_def.push_str("    }\n");
                }
                struct_def.push_str("}\n");

                self.hoisted_visitors.push(struct_def);

                // Generate instantiation and call at the usage site
                self.emit_indent();
                self.emit(&format!("let mut __visitor = {} {{\n", struct_name));
                self.indent += 1;

                // Initialize captured variables (pass references)
                for capture in &traverse_stmt.captures {
                    self.emit_indent();
                    self.emit(&capture.name);
                    self.emit(": ");
                    if capture.mutable {
                        self.emit("&mut ");
                    } else {
                        self.emit("&");
                    }
                    self.emit(&capture.name);
                    self.emit(",\n");
                }

                // Initialize local state
                for let_stmt in &inline.state {
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
                self.emit("};\n");

                // Generate visit_mut_with call
                self.emit_indent();
                self.gen_expr(&traverse_stmt.target);
                self.emit(".visit_mut_with(&mut __visitor);\n");
            }
            crate::parser::TraverseKind::Delegated(visitor_name) => {
                // Generate delegation to another visitor
                self.emit_indent();
                self.emit(&format!("let mut __visitor = {}::default();\n", visitor_name));
                self.emit_indent();
                self.gen_expr(&traverse_stmt.target);
                self.emit(".visit_mut_with(&mut __visitor);\n");
            }
        }
    }
    pub(super) fn gen_decorated_if_let_stmt(&mut self, if_stmt: &DecoratedIfStmt) {
        self.emit_indent();
        self.emit("if let ");

        if let Some(ref pattern) = if_stmt.pattern {
            self.gen_decorated_pattern(pattern);
        }

        self.emit(" = ");
        self.gen_decorated_expr(&if_stmt.condition);
        self.emit(" {\n");

        self.indent += 1;
        self.gen_decorated_block(&if_stmt.then_branch);
        self.indent -= 1;

        if let Some(ref else_branch) = if_stmt.else_branch {
            self.emit_indent();
            self.emit("} else {\n");
            self.indent += 1;
            self.gen_decorated_block(else_branch);
            self.indent -= 1;
        }

        self.emit_line("}");
    }

    /// Generate block from decorated AST
    pub(super) fn gen_decorated_block(&mut self, block: &DecoratedBlock) {
        for stmt in &block.stmts {
            self.gen_decorated_stmt(stmt);
        }
    }

    /// Generate statement from decorated AST
    pub(super) fn gen_decorated_stmt(&mut self, stmt: &DecoratedStmt) {
        match stmt {
            DecoratedStmt::Let(let_stmt) => {
                self.emit_indent();
                if let_stmt.mutable {
                    self.emit("let mut ");
                } else {
                    self.emit("let ");
                }
                self.gen_decorated_pattern(&let_stmt.pattern);
                self.emit(" = ");
                self.gen_decorated_expr(&let_stmt.init);
                self.emit(";\n");
            }

            DecoratedStmt::Const(const_stmt) => {
                self.emit_indent();
                self.emit("const ");
                self.emit(&const_stmt.name);
                self.emit(": ");
                // TODO: type annotation from metadata
                self.emit(" = ");
                self.gen_decorated_expr(&const_stmt.init);
                self.emit(";\n");
            }

            DecoratedStmt::Expr(expr) => {
                self.emit_indent();
                self.gen_decorated_expr(expr);
                self.emit(";\n");
            }

            DecoratedStmt::If(if_stmt) => {
                self.gen_decorated_if_let_stmt(if_stmt);
            }

            DecoratedStmt::Match(match_stmt) => {
                self.emit_indent();
                self.emit("match ");
                self.gen_decorated_expr(&match_stmt.scrutinee);
                self.emit(" {\n");
                self.indent += 1;
                for arm in &match_stmt.arms {
                    self.emit_indent();
                    self.gen_decorated_pattern(&arm.pattern);
                    if let Some(ref guard) = arm.guard {
                        self.emit(" if ");
                        self.gen_decorated_expr(guard);
                    }
                    self.emit(" => ");
                    self.gen_decorated_expr(&arm.body);
                    self.emit(",\n");
                }
                self.indent -= 1;
                self.emit_indent();
                self.emit("}\n");
            }

            DecoratedStmt::For(for_stmt) => {
                self.emit_indent();
                self.emit("for ");
                self.gen_decorated_pattern(&for_stmt.pattern);
                self.emit(" in ");
                self.gen_decorated_expr(&for_stmt.iter);
                self.emit(" {\n");
                self.indent += 1;
                self.gen_decorated_block(&for_stmt.body);
                self.indent -= 1;
                self.emit_indent();
                self.emit("}\n");
            }

            DecoratedStmt::While(while_stmt) => {
                self.emit_indent();
                self.emit("while ");
                self.gen_decorated_expr(&while_stmt.condition);
                self.emit(" {\n");
                self.indent += 1;
                self.gen_decorated_block(&while_stmt.body);
                self.indent -= 1;
                self.emit_indent();
                self.emit("}\n");
            }

            DecoratedStmt::Loop(block) => {
                self.emit_indent();
                self.emit("loop {\n");
                self.indent += 1;
                self.gen_decorated_block(block);
                self.indent -= 1;
                self.emit_indent();
                self.emit("}\n");
            }

            DecoratedStmt::Return(expr_opt) => {
                self.emit_indent();
                self.emit("return");
                if let Some(expr) = expr_opt {
                    self.emit(" ");
                    self.gen_decorated_expr(expr);
                }
                self.emit(";\n");
            }

            DecoratedStmt::Break => {
                self.emit_indent();
                self.emit("break;\n");
            }

            DecoratedStmt::Continue => {
                self.emit_indent();
                self.emit("continue;\n");
            }

            // Fallback for undecorated statements
            DecoratedStmt::Traverse(traverse) => {
                // Not decorated yet
                self.gen_traverse_stmt(traverse);
            }

            DecoratedStmt::Function(func) => {
                // Not decorated yet
                self.gen_helper_function(func);
            }

            DecoratedStmt::Verbatim(verbatim) => {
                self.emit_indent();
                self.emit(&verbatim.code);
                if !verbatim.code.ends_with('\n') {
                    self.emit("\n");
                }
            }
        }
    }

    // ============================================================================
    // ORIGINAL AST GENERATION (Old, will be deprecated)
    // ============================================================================
}
