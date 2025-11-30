//! Code generation for expressions/statements/patterns
use super::SwcGenerator;
use crate::parser::*;
use crate::codegen::decorated_ast::*;
use crate::codegen::type_context::{TypeContext, TypeEnvironment};
use crate::mapping::{get_node_mapping, get_swc_variant_in_context, get_field_mapping};

impl SwcGenerator {
    pub(super) fn gen_decorated_expr(&mut self, expr: &DecoratedExpr) {
        match &expr.kind {
            DecoratedExprKind::Literal(lit) => self.gen_literal(lit),

            DecoratedExprKind::Ident { name, ident_metadata } => {
                // Metadata tells us if we need &*sym or just name
                if let Some(ref deref) = ident_metadata.deref_pattern {
                    self.emit(deref);
                }
                self.emit(name);
                if ident_metadata.use_sym {
                    self.emit(".sym");
                }
            }

            DecoratedExprKind::Binary { left, op, right, binary_metadata } => {
                self.emit("(");
                if binary_metadata.left_needs_deref {
                    self.emit("&*");
                }
                self.gen_decorated_expr(left);
                self.emit(&format!(" {} ", self.binary_op_to_rust(op)));
                if binary_metadata.right_needs_deref {
                    self.emit("&*");
                }
                self.gen_decorated_expr(right);
                self.emit(")");
            }

            DecoratedExprKind::Member { object, property, field_metadata, .. } => {
                self.gen_decorated_expr(object);
                self.emit(".");
                // Use the SWC field name from metadata!
                self.emit(&field_metadata.swc_field_name);

                // Apply accessor strategy from metadata
                match &field_metadata.accessor {
                    super::swc_metadata::FieldAccessor::BoxedAsRef => {
                        self.emit(".as_ref()");
                    }
                    _ => {
                        // Other accessors handled differently
                    }
                }
            }

            DecoratedExprKind::Unary { op, operand, unary_metadata } => {
                // Use override_op from metadata if present
                if let Some(ref override_op) = unary_metadata.override_op {
                    self.emit(override_op);
                } else {
                    match op {
                        crate::parser::UnaryOp::Not => self.emit("!"),
                        crate::parser::UnaryOp::Neg => self.emit("-"),
                        crate::parser::UnaryOp::Deref => self.emit("*"),
                        crate::parser::UnaryOp::Ref => self.emit("&"),
                        crate::parser::UnaryOp::RefMut => self.emit("&mut "),
                    }
                }
                self.gen_decorated_expr(operand);
            }

            DecoratedExprKind::Call(call) => {
                self.gen_decorated_expr(&call.callee);
                self.emit("(");
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 { self.emit(", "); }
                    self.gen_decorated_expr(arg);
                }
                self.emit(")");
            }

            DecoratedExprKind::Index { object, index } => {
                self.gen_decorated_expr(object);
                self.emit("[");
                self.gen_decorated_expr(index);
                self.emit("]");
            }

            DecoratedExprKind::VecInit(elements) => {
                self.emit("vec![");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 { self.emit(", "); }
                    self.gen_decorated_expr(elem);
                }
                self.emit("]");
            }

            DecoratedExprKind::Ref { mutable, expr } => {
                if *mutable {
                    self.emit("&mut ");
                } else {
                    self.emit("&");
                }
                self.gen_decorated_expr(expr);
            }

            DecoratedExprKind::Deref(expr) => {
                self.emit("*");
                self.gen_decorated_expr(expr);
            }

            DecoratedExprKind::Assign { left, right } => {
                self.gen_decorated_expr(left);
                self.emit(" = ");
                self.gen_decorated_expr(right);
            }

            DecoratedExprKind::CompoundAssign { left, op, right } => {
                self.gen_decorated_expr(left);
                self.emit(&format!(" {} ", self.compound_op_to_rust(op)));
                self.gen_decorated_expr(right);
            }

            DecoratedExprKind::Paren(expr) => {
                self.emit("(");
                self.gen_decorated_expr(expr);
                self.emit(")");
            }

            DecoratedExprKind::Block(block) => {
                self.gen_decorated_block(block);
            }

            DecoratedExprKind::Try(expr) => {
                self.gen_decorated_expr(expr);
                self.emit("?");
            }

            DecoratedExprKind::Tuple(exprs) => {
                self.emit("(");
                for (i, expr) in exprs.iter().enumerate() {
                    if i > 0 { self.emit(", "); }
                    self.gen_decorated_expr(expr);
                }
                if exprs.len() == 1 {
                    self.emit(","); // Single-element tuple needs trailing comma
                }
                self.emit(")");
            }

            DecoratedExprKind::Return(expr_opt) => {
                self.emit("return");
                if let Some(expr) = expr_opt {
                    self.emit(" ");
                    self.gen_decorated_expr(expr);
                }
            }

            DecoratedExprKind::Break => {
                self.emit("break");
            }

            DecoratedExprKind::Continue => {
                self.emit("continue");
            }

            DecoratedExprKind::Matches { expr, pattern } => {
                self.emit("matches!(");
                self.gen_decorated_expr(expr);
                self.emit(", ");
                self.gen_decorated_pattern(pattern);
                self.emit(")");
            }

            // Fallback for undecorated expressions
            DecoratedExprKind::StructInit(struct_init) => {
                // Not decorated yet, use original codegen
                self.emit(&struct_init.name);
                self.emit(" { ");
                for (i, (field_name, field_expr)) in struct_init.fields.iter().enumerate() {
                    if i > 0 { self.emit(", "); }
                    self.emit(field_name);
                    self.emit(": ");
                    self.gen_expr(field_expr);
                }
                self.emit(" }");
            }

            DecoratedExprKind::Closure(closure) => {
                // Not decorated yet, fallback
                self.emit("|");
                for (i, param) in closure.params.iter().enumerate() {
                    if i > 0 { self.emit(", "); }
                    self.emit(&param.name);
                }
                self.emit("| ");
                self.gen_expr(&closure.body);
            }

            DecoratedExprKind::If(if_expr) => {
                self.emit("if ");
                self.gen_decorated_expr(&if_expr.condition);
                self.emit(" ");
                self.gen_decorated_block(&if_expr.then_branch);
                if let Some(ref else_branch) = if_expr.else_branch {
                    self.emit(" else ");
                    self.gen_decorated_block(else_branch);
                }
            }

            DecoratedExprKind::Match(match_expr) => {
                self.emit("match ");
                self.gen_decorated_expr(&match_expr.scrutinee);
                self.emit(" {\n");
                self.indent += 1;
                for arm in &match_expr.arms {
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
                self.emit("}");
            }

            DecoratedExprKind::Range { start, end, inclusive } => {
                if let Some(s) = start {
                    self.gen_decorated_expr(s);
                }
                if *inclusive {
                    self.emit("..=");
                } else {
                    self.emit("..");
                }
                if let Some(e) = end {
                    self.gen_decorated_expr(e);
                }
            }
        }
    }

    /// Generate if-let statement from decorated AST
    pub(super) fn gen_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Literal(lit) => self.gen_literal(lit),
            Expr::Ident(ident) => {
                match ident.name.as_str() {
                    "self" => self.emit("self"),
                    _ => {
                        // Check for parameter renames (e.g., "node" -> "n")
                        let output = if let Some(renamed) = self.param_renames.get(&ident.name) {
                            renamed.clone()
                        } else {
                            ident.name.clone()
                        };
                        // Check if this is a captured variable (needs self. prefix)
                        if self.captured_vars.contains(&ident.name) {
                            self.emit(&format!("self.{}", output));
                        } else {
                            self.emit(&output);
                        }
                    }
                }
            }
            Expr::Binary(bin) => {
                self.emit("(");
                self.gen_expr(&bin.left);
                self.emit(&format!(" {} ", self.binary_op_to_rust(&bin.op)));
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
                        self.emit("*");
                        self.gen_expr(&un.operand);
                    }
                    UnaryOp::Ref => {
                        self.emit("&");
                        self.gen_expr(&un.operand);
                    }
                    UnaryOp::RefMut => {
                        self.emit("&mut ");
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
                    // Check if this is a call to an associated function (needs Self:: prefix)
                    if self.associated_functions.contains(&ident.name) {
                        self.emit("Self::");
                        self.emit(&ident.name);
                        self.emit("(");
                        for (i, arg) in call.args.iter().enumerate() {
                            if i > 0 {
                                self.emit(", ");
                            }
                            self.gen_expr(arg);
                        }
                        self.emit(")");
                        return;
                    }
                    // Check for format! macro - pass through as-is
                    if (ident.name == "format!" || ident.name == "format") && !call.args.is_empty() {
                        self.emit("format!(");
                        for (i, arg) in call.args.iter().enumerate() {
                            if i > 0 {
                                self.emit(", ");
                            }
                            self.gen_expr(arg);
                        }
                        self.emit(")");
                        return;
                    }
                    // Check for vec! macro - pass through as-is
                    if ident.name == "vec!" {
                        self.emit("vec![");
                        for (i, arg) in call.args.iter().enumerate() {
                            if i > 0 {
                                self.emit(", ");
                            }
                            self.gen_expr(arg);
                        }
                        self.emit("]");
                        return;
                    }
                    // Check for panic! macro - pass through as-is
                    if ident.name == "panic!" {
                        self.emit("panic!(");
                        for (i, arg) in call.args.iter().enumerate() {
                            if i > 0 {
                                self.emit(", ");
                            }
                            self.gen_expr(arg);
                        }
                        self.emit(")");
                        return;
                    }
                }

                // Check for CodeBuilder methods
                if let Expr::Member(mem) = call.callee.as_ref() {
                    // CodeBuilder::new() -> String::new()
                    if let Expr::Ident(type_name) = mem.object.as_ref() {
                        if type_name.name == "CodeBuilder" && mem.property == "new" {
                            self.emit("String::new()");
                            return;
                        }
                    }

                    // builder.append(s) -> builder.push_str(s)
                    if mem.property == "append" {
                        // Check if object is CodeBuilder type
                        let obj_type = self.infer_type(&mem.object);
                        if obj_type.swc_type == "CodeBuilder" {
                            self.gen_expr(&mem.object);
                            self.emit(".push_str(");
                            if !call.args.is_empty() {
                                self.gen_expr(&call.args[0]);
                            }
                            self.emit(")");
                            return;
                        }
                    }

                    // builder.append_line(s) -> builder.push_str(s); builder.push_str("\n")
                    if mem.property == "append_line" {
                        let obj_type = self.infer_type(&mem.object);
                        if obj_type.swc_type == "CodeBuilder" {
                            self.emit("{ ");
                            self.gen_expr(&mem.object);
                            self.emit(".push_str(");
                            if !call.args.is_empty() {
                                self.gen_expr(&call.args[0]);
                            }
                            self.emit("); ");
                            self.gen_expr(&mem.object);
                            self.emit(".push_str(\"\\n\"); }");
                            return;
                        }
                    }

                    // builder.newline() -> builder.push_str("\n")
                    if mem.property == "newline" {
                        let obj_type = self.infer_type(&mem.object);
                        if obj_type.swc_type == "CodeBuilder" {
                            self.gen_expr(&mem.object);
                            self.emit(".push_str(\"\\n\")");
                            return;
                        }
                    }

                    // builder.indent() - no-op for now (would need state tracking)
                    if mem.property == "indent" {
                        let obj_type = self.infer_type(&mem.object);
                        if obj_type.swc_type == "CodeBuilder" {
                            // TODO: Implement indent tracking
                            self.emit("()");
                            return;
                        }
                    }

                    // builder.dedent() - no-op for now
                    if mem.property == "dedent" {
                        let obj_type = self.infer_type(&mem.object);
                        if obj_type.swc_type == "CodeBuilder" {
                            // TODO: Implement indent tracking
                            self.emit("()");
                            return;
                        }
                    }

                    // builder.to_string() -> builder.clone()
                    if mem.property == "to_string" {
                        let obj_type = self.infer_type(&mem.object);
                        if obj_type.swc_type == "CodeBuilder" {
                            self.gen_expr(&mem.object);
                            self.emit(".clone()");
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

                // Check for visitor traversal methods
                if let Expr::Member(mem) = call.callee.as_ref() {
                    let prop = &mem.property;
                    // visit_children(self) -> swc_ecma_visit::VisitWith::visit_children_with(node, self) for writers
                    //                      -> n.visit_mut_children_with(self) for plugins
                    if prop == "visit_children" {
                        if self.is_writer {
                            // Use fully qualified syntax for Visit trait
                            self.emit("swc_ecma_visit::VisitWith::visit_children_with(");
                            self.gen_expr(&mem.object);
                            self.emit(", self)");
                        } else {
                            self.gen_expr(&mem.object);
                            self.emit(".visit_mut_children_with(self)");
                        }
                        return;
                    }
                    // visit_with(self) -> n.visit_with(self) for writers, n.visit_mut_with(self) for plugins
                    if prop == "visit_with" {
                        self.gen_expr(&mem.object);
                        if self.is_writer {
                            self.emit(".visit_with(self)");
                        } else {
                            self.emit(".visit_mut_with(self)");
                        }
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
                // Check for nested member access that needs unwrapping
                // e.g., node.callee.name or member.key.name
                if mem.property == "name" || mem.property == "sym" {
                    if let Expr::Member(inner) = mem.object.as_ref() {
                        let inner_prop = &inner.property;
                        // Handle callee.name -> need to unwrap Callee::Expr then Expr::Ident
                        if inner_prop == "callee" {
                            // Generate: if let Callee::Expr(e) = &obj.callee { if let Expr::Ident(i) = e.as_ref() { i.sym.clone() } }
                            // For now, emit a simpler pattern that assumes Ident
                            self.emit("{ let __callee = &");
                            self.gen_expr(&inner.object);
                            self.emit(".callee; match __callee { Callee::Expr(e) => match e.as_ref() { Expr::Ident(i) => i.sym.clone(), _ => \"\".into() }, _ => \"\".into() } }");
                            return;
                        }
                        // Handle key.name -> need to unwrap Box<Expr> then Expr::Ident
                        if inner_prop == "key" {
                            self.emit("{ match ");
                            self.gen_expr(&inner.object);
                            self.emit(".key.as_ref() { Expr::Ident(i) => i.sym.clone(), _ => \"\".into() } }");
                            return;
                        }
                    }
                }

                // Special case: self.builder in writers becomes just "self"
                if let Expr::Ident(obj_ident) = mem.object.as_ref() {
                    if obj_ident.name == "self" && mem.property == "builder" {
                        // In writer context, self.builder -> self (writer has methods directly)
                        self.emit("self");
                        return;
                    }
                    // Special case: self.state in writers becomes self (State is flattened)
                    if obj_ident.name == "self" && mem.property == "state" {
                        self.emit("self");
                        return;
                    }
                }
                // Special case: self.state.builder becomes self (nested member access)
                if let Expr::Member(inner_mem) = mem.object.as_ref() {
                    if let Expr::Ident(obj_ident) = inner_mem.object.as_ref() {
                        if obj_ident.name == "self" && inner_mem.property == "state" && mem.property == "builder" {
                            self.emit("self");
                            return;
                        }
                    }
                }

                // Simple member access
                // Translate module names
                if let Expr::Ident(ident) = &*mem.object {
                    // Translate json:: to serde_json::
                    if ident.name == "json" && mem.is_path {
                        self.emit("serde_json");
                    } else {
                        self.gen_expr(&mem.object);
                    }
                } else {
                    self.gen_expr(&mem.object);
                }
                // Use :: for path expressions, . for member access
                self.emit(if mem.is_path { "::" } else { "." });
                // Map ReluxScript field names to SWC field names using type context
                let obj_type = self.infer_type(&mem.object);
                let is_ast_type = !matches!(obj_type.kind, super::type_context::SwcTypeKind::Unknown);
                let swc_field = if let Some(mapping) = get_typed_field_mapping(&obj_type.swc_type, &mem.property) {
                    mapping.swc_field
                } else if is_ast_type {
                    // Only apply fallback mappings for known AST types, not user-defined types
                    match mem.property.as_str() {
                        // Identifier.name -> Ident.sym
                        "name" => "sym",
                        // MemberExpression.property -> MemberExpr.prop
                        "property" => "prop",
                        // MemberExpression.object -> MemberExpr.obj (needs Box unwrap)
                        "object" => "obj",
                        // CallExpression.arguments -> CallExpr.args
                        "arguments" => "args",
                        // CallExpression.callee -> CallExpr.callee (needs Box unwrap)
                        "callee" => "callee",
                        // ArrayPattern.elements / ArrayExpression.elements -> elems
                        "elements" => "elems",
                        _ => &mem.property,
                    }
                } else {
                    // Unknown type (likely user-defined) - don't map field names
                    &mem.property
                };
                self.emit(swc_field);
            }
            Expr::Index(idx) => {
                // Check if this is a slice with range syntax
                if let Expr::Range(range) = idx.index.as_ref() {
                    // Convert name[start..end] to &name[start..end]
                    self.emit("&");
                    self.gen_expr(&idx.object);
                    self.emit("[");
                    if let Some(start) = &range.start {
                        self.gen_expr(start);
                    } else {
                        self.emit("0");
                    }
                    self.emit("..");
                    if let Some(end) = &range.end {
                        self.gen_expr(end);
                    }
                    self.emit("]");
                } else {
                    // Regular index access
                    self.gen_expr(&idx.object);
                    self.emit("[");
                    self.gen_expr(&idx.index);
                    self.emit("]");
                }
            }
            Expr::StructInit(init) => {
                // Map ReluxScript AST types to SWC types
                // Don't lowercase - preserve original case for user-defined types
                let swc_type = self.reluxscript_to_swc_type(&init.name);

                // Handle special SWC struct field mappings
                let is_ident = swc_type == "Ident";
                let is_call_expr = swc_type == "CallExpr";
                let is_literal = swc_type == "Lit";

                self.emit(&swc_type);
                self.emit(" { ");

                // Special case: Use Ident::new() constructor instead of struct literal
                if is_ident {
                    // Ident should use the constructor: Ident::new(name.into(), DUMMY_SP)
                    // This ensures all required fields (sym, span, ctxt, optional) are properly initialized
                    let mut name_value = None;
                    for (field_name, field_value) in &init.fields {
                        if field_name == "name" {
                            name_value = Some(field_value);
                            break;
                        }
                    }

                    if let Some(name_val) = name_value {
                        // Rewrite as Ident::new() call
                        self.output.truncate(self.output.len() - " { ".len()); // Remove " { "
                        self.emit("::new(");
                        self.gen_expr(name_val);
                        self.emit(".into(), DUMMY_SP, SyntaxContext::empty())");
                        return;
                    }
                }

                for (i, (name, value)) in init.fields.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }

                    // Map field names for SWC types
                    if is_ident && name == "name" {
                        // This branch should not be reached due to the special case above
                        self.emit("sym: ");
                        self.gen_expr(value);
                        self.emit(".into(), span: DUMMY_SP");
                    } else if is_call_expr && name == "callee" {
                        // CallExpr.callee is Callee enum
                        self.emit("callee: Callee::Expr(Box::new(Expr::Ident(Ident::new(");
                        // Extract the name from the nested Identifier
                        if let Expr::StructInit(nested) = value {
                            for (nested_name, nested_val) in &nested.fields {
                                if nested_name == "name" {
                                    self.gen_expr(nested_val);
                                }
                            }
                        } else {
                            self.gen_expr(value);
                        }
                        self.emit(".into(), DUMMY_SP, SyntaxContext::empty()))))");
                    } else if is_call_expr && name == "arguments" {
                        // CallExpr.args is Vec<ExprOrSpread>
                        self.emit("args: vec![]"); // Simplified for now
                        // TODO: properly handle ExprOrSpread
                    } else if is_literal && name == "value" {
                        // Literal needs proper Lit enum variant
                        self.emit("value: Lit::Num(Number { value: ");
                        self.gen_expr(value);
                        self.emit(".0, span: DUMMY_SP, raw: None })");
                    } else {
                        self.emit(name);
                        self.emit(": ");
                        self.gen_expr(value);
                    }
                }

                // Add required fields for CallExpr
                if is_call_expr {
                    self.emit(", span: DUMMY_SP, ..Default::default()");
                }

                self.emit(" }");
            }
            Expr::VecInit(vec) => {
                self.emit("vec![");
                for (i, elem) in vec.elements.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.gen_expr(elem);
                }
                self.emit("]");
            }
            Expr::If(if_expr) => {
                // Check if this is an if-let expression
                if let Some(pattern) = &if_expr.pattern {
                    self.emit("if let ");
                    self.gen_pattern(pattern);
                    self.emit(" = ");
                    self.gen_expr(&if_expr.condition);
                } else {
                    self.emit("if ");
                    self.gen_expr(&if_expr.condition);
                }
                self.emit(" {");
                // Check if this is a simple single-expression if (no semicolons needed)
                let is_simple = if_expr.then_branch.stmts.len() == 1
                    && matches!(if_expr.then_branch.stmts[0], Stmt::Expr(_))
                    && if_expr.else_branch.as_ref().map_or(true, |b| b.stmts.len() == 1 && matches!(b.stmts[0], Stmt::Expr(_)));

                if is_simple {
                    self.emit(" ");
                    // For simple if-expressions, just emit the expression without indentation
                    for stmt in &if_expr.then_branch.stmts {
                        if let Stmt::Expr(expr_stmt) = stmt {
                            self.gen_expr(&expr_stmt.expr);
                        }
                    }
                    self.emit(" }");
                    if let Some(else_block) = &if_expr.else_branch {
                        self.emit(" else { ");
                        for stmt in &else_block.stmts {
                            if let Stmt::Expr(expr_stmt) = stmt {
                                self.gen_expr(&expr_stmt.expr);
                            }
                        }
                        self.emit(" }");
                    }
                } else {
                    self.emit("\n");
                    self.indent += 1;
                    let then_len = if_expr.then_branch.stmts.len();
                    for (i, stmt) in if_expr.then_branch.stmts.iter().enumerate() {
                        self.gen_stmt_with_context(stmt, i == then_len - 1);
                    }
                    self.indent -= 1;
                    self.emit_indent();
                    self.emit("}");
                    if let Some(else_block) = &if_expr.else_branch {
                        self.emit(" else {\n");
                        self.indent += 1;
                        let else_len = else_block.stmts.len();
                        for (i, stmt) in else_block.stmts.iter().enumerate() {
                            self.gen_stmt_with_context(stmt, i == else_len - 1);
                        }
                        self.indent -= 1;
                        self.emit_indent();
                        self.emit("}");
                    }
                }
            }
            Expr::Match(match_expr) => {
                self.emit("match ");
                self.gen_expr(&match_expr.scrutinee);
                self.emit(" { ");
                for arm in &match_expr.arms {
                    self.gen_pattern(&arm.pattern);
                    self.emit(" => ");
                    self.gen_expr(&arm.body);
                    self.emit(", ");
                }
                self.emit("}");
            }
            Expr::Closure(closure) => {
                self.emit("|");
                self.emit(&closure.params.join(", "));
                self.emit("| ");
                self.gen_expr(&closure.body);
            }
            Expr::Ref(ref_expr) => {
                if ref_expr.mutable {
                    self.emit("&mut ");
                } else {
                    self.emit("&");
                }
                self.gen_expr(&ref_expr.expr);
            }
            Expr::Deref(deref) => {
                self.emit("*");
                self.gen_expr(&deref.expr);
            }
            Expr::Assign(assign) => {
                self.gen_expr(&assign.target);
                self.emit(" = ");
                self.gen_expr(&assign.value);
            }
            Expr::CompoundAssign(compound) => {
                self.gen_expr(&compound.target);
                self.emit(&format!(" {}= ", self.compound_op_to_rust(&compound.op)));
                self.gen_expr(&compound.value);
            }
            Expr::Range(range) => {
                if let Some(start) = &range.start {
                    self.gen_expr(start);
                }
                if range.inclusive {
                    self.emit("..=");
                } else {
                    self.emit("..");
                }
                if let Some(end) = &range.end {
                    self.gen_expr(end);
                }
            }
            Expr::Block(block) => {
                // Block expression
                self.emit("{\n");
                self.indent += 1;
                self.gen_block(block);
                self.indent -= 1;
                self.emit_indent();
                self.emit("}");
            }
            Expr::Try(inner) => {
                // Try operator: expr?
                self.gen_expr(inner);
                self.emit("?");
            }
            Expr::Paren(inner) => {
                self.emit("(");
                self.gen_expr(inner);
                self.emit(")");
            }
            Expr::Tuple(elements) => {
                // Tuples stay as tuples in Rust
                self.emit("(");
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        self.emit(", ");
                    }
                    self.gen_expr(elem);
                }
                self.emit(")");
            }

            Expr::Matches(matches_expr) => {
                // Generate matches! macro in Rust
                self.emit("matches!(");
                self.gen_expr(&matches_expr.scrutinee);
                self.emit(", ");
                self.gen_pattern(&matches_expr.pattern);
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
        }
    }
    pub(super) fn gen_literal(&mut self, lit: &Literal) {
        match lit {
            Literal::String(s) => {
                self.emit(&format!("\"{}\"", s.replace('\\', "\\\\").replace('"', "\\\"")));
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
                self.emit("None");
            }
            Literal::Unit => {
                self.emit("()");
            }
        }
    }
    pub(super) fn gen_matches_macro(&mut self, scrutinee: &Expr, pattern: &Expr) {
        // For SWC, we generate a boolean expression with nested if-let patterns
        // For simplicity, we'll generate a simpler check that works for common cases
        self.emit("{\n");
        self.indent += 1;
        self.emit_indent();
        self.emit("let __matched = ");
        self.gen_swc_pattern_check(scrutinee, pattern, 0);
        self.emit(";\n");
        self.emit_indent();
        self.emit("__matched\n");
        self.indent -= 1;
        self.emit_indent();
        self.emit("}");
    }
}
