//! Pattern-based SWC code generation
//!
//! This module implements the pattern rewriting for matches! macros
//! and flow-sensitive type narrowing.

use crate::parser::*;
use super::type_context::{TypeContext, TypeEnvironment, get_swc_variant, get_swc_variant_in_context, get_typed_field_mapping, SwcTypeKind};

/// Pattern-aware SWC code generator
pub struct SwcPatternGenerator {
    pub output: String,
    pub env: TypeEnvironment,
    indent_level: usize,
}

impl SwcPatternGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            env: TypeEnvironment::new(),
            indent_level: 0,
        }
    }

    /// Generate a while statement with type-aware pattern matching
    ///
    /// Transforms:
    /// ```reluxscript
    /// while matches!(current, MemberExpression) {
    ///     let member = current;
    ///     // ...
    /// }
    /// ```
    ///
    /// Into:
    /// ```rust
    /// while let Expr::Member(current) = current {
    ///     // current is now MemberExpr
    ///     // ...
    /// }
    /// ```
    pub fn gen_while_stmt_typed(&mut self, condition: &Expr, body: &Block) {
        // 1. Try to extract matches!(var, Type) pattern
        if let Some((var_name, type_target)) = self.extract_matches_pattern(condition) {
            // 2. Look up the variable's type to determine the correct context
            let var_type = self.env.lookup(&var_name)
                .map(|ctx| ctx.swc_type.clone())
                .unwrap_or_else(|| "Expr".to_string());

            // 3. Get SWC enum/variant mapping using context
            let (swc_enum, swc_variant, swc_struct) = get_swc_variant_in_context(&type_target, &var_type);

            // 4. Generate "while let" with shadowing
            self.emit_indent();
            // Output: while let Expr::Member(var_name) = var_name {
            self.output.push_str(&format!(
                "while let {}::{}({}) = {} {{\n",
                swc_enum, swc_variant, var_name, var_name
            ));

            self.indent_level += 1;
            self.env.push_scope();

            // 4. SHADOWING: Define var_name with narrowed type
            let narrowed_ctx = TypeContext::narrowed(&type_target, &swc_struct);
            self.env.define(&var_name, narrowed_ctx);

            // 5. Generate body
            self.gen_block_typed(body);

            self.env.pop_scope();
            self.indent_level -= 1;
            self.emit_indent();
            self.output.push_str("}\n");
        } else {
            // Fallback for standard while loops
            self.gen_standard_while(condition, body);
        }
    }

    /// Generate an if statement with type-aware pattern matching
    pub fn gen_if_stmt_typed(&mut self, condition: &Expr, then_branch: &Block, else_branch: Option<&Block>) {
        // Try to extract matches!(var, Type) pattern
        if let Some((var_name, type_target)) = self.extract_matches_pattern(condition) {
            // Look up the variable's type to determine the correct context
            let var_type = self.env.lookup(&var_name)
                .map(|ctx| ctx.swc_type.clone())
                .unwrap_or_else(|| "Expr".to_string());

            // Use the variable's type as context for determining the enum/variant
            let (swc_enum, swc_variant, swc_struct) = get_swc_variant_in_context(&type_target, &var_type);

            self.emit_indent();
            // Output: if let Expr::Ident(var_name) = var_name {
            // or: if let MemberProp::Ident(var_name) = var_name {
            self.output.push_str(&format!(
                "if let {}::{}({}) = {} {{\n",
                swc_enum, swc_variant, var_name, var_name
            ));

            self.indent_level += 1;
            self.env.push_scope();

            // Shadow with narrowed type
            let narrowed_ctx = TypeContext::narrowed(&type_target, &swc_struct);
            self.env.define(&var_name, narrowed_ctx);

            self.gen_block_typed(then_branch);

            self.env.pop_scope();
            self.indent_level -= 1;
            self.emit_indent();
            self.output.push_str("}");

            // Handle else branch
            if let Some(else_block) = else_branch {
                self.output.push_str(" else {\n");
                self.indent_level += 1;
                self.gen_block_typed(else_block);
                self.indent_level -= 1;
                self.emit_indent();
                self.output.push_str("}");
            }
            self.output.push('\n');
        } else {
            // Fallback for standard if statements
            self.gen_standard_if(condition, then_branch, else_branch);
        }
    }

    /// Generate a let statement with type awareness
    pub fn gen_let_stmt_typed(&mut self, stmt: &LetStmt) {
        // Infer the type of the initializer
        let init_type = self.infer_type(&stmt.init);

        self.emit_indent();
        if stmt.mutable {
            self.output.push_str("let mut ");
        } else {
            self.output.push_str("let ");
        }

        // Generate pattern (only simple identifiers supported)
        if let Pattern::Ident(name) = &stmt.pattern {
            self.output.push_str(name);
            self.output.push_str(" = ");

            // Generate the initializer with type context
            self.gen_expr_typed(&stmt.init);
            self.output.push_str(";\n");

            // Record the variable's type
            self.env.define(name, init_type);
        } else {
            // For complex patterns, just emit a placeholder
            self.output.push_str("_ = ");
            self.gen_expr_typed(&stmt.init);
            self.output.push_str(";\n");
        }
    }

    /// Generate an expression with type awareness
    pub fn gen_expr_typed(&mut self, expr: &Expr) {
        match expr {
            Expr::Ident(ident) => {
                self.output.push_str(&ident.name);
            }

            Expr::Member(mem) => {
                // Get the object's type to determine field mapping
                let obj_type = self.infer_type(&mem.object);

                // Generate object expression
                let needs_deref = obj_type.needs_deref;
                if needs_deref {
                    self.output.push_str("(*");
                }
                self.gen_expr_typed(&mem.object);
                if needs_deref {
                    self.output.push_str(")");
                }

                // Look up field mapping
                if let Some(mapping) = get_typed_field_mapping(&obj_type.swc_type, &mem.property) {
                    self.output.push('.');
                    self.output.push_str(mapping.swc_field);

                    // Apply read conversion if needed
                    if !mapping.read_conversion.is_empty() {
                        self.output.push_str(mapping.read_conversion);
                    }
                } else {
                    // Fallback: use field name directly
                    self.output.push('.');
                    self.output.push_str(&mem.property);
                }
            }

            Expr::Call(call) => {
                // Check for .clone() which we handle specially
                if let Expr::Member(mem) = call.callee.as_ref() {
                    if mem.property == "clone" && call.args.is_empty() {
                        // Just generate the object, clone is implicit in our patterns
                        self.gen_expr_typed(&mem.object);
                        self.output.push_str(".clone()");
                        return;
                    }
                }

                // Standard call
                self.gen_expr_typed(&call.callee);
                self.output.push('(');
                for (i, arg) in call.args.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.gen_expr_typed(arg);
                }
                self.output.push(')');
            }

            Expr::Literal(lit) => {
                match lit {
                    Literal::String(s) => {
                        self.output.push_str(&format!("\"{}\"", s));
                    }
                    Literal::Int(n) => {
                        self.output.push_str(&n.to_string());
                    }
                    Literal::Float(n) => {
                        self.output.push_str(&n.to_string());
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

            Expr::VecInit(vec_init) => {
                self.output.push_str("vec![");
                for (i, elem) in vec_init.elements.iter().enumerate() {
                    if i > 0 {
                        self.output.push_str(", ");
                    }
                    self.gen_expr_typed(elem);
                }
                self.output.push(']');
            }

            // Handle other expressions...
            _ => {
                // Fallback: generate as-is (would need full implementation)
                self.output.push_str("/* unhandled expr */");
            }
        }
    }

    /// Generate a block of statements
    pub fn gen_block_typed(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.gen_stmt_typed(stmt);
        }
    }

    /// Generate a statement with type awareness
    fn gen_stmt_typed(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_stmt) => {
                self.gen_let_stmt_typed(let_stmt);
            }
            Stmt::If(if_stmt) => {
                self.gen_if_stmt_typed(
                    &if_stmt.condition,
                    &if_stmt.then_branch,
                    if_stmt.else_branch.as_ref(),
                );
            }
            Stmt::While(while_stmt) => {
                self.gen_while_stmt_typed(&while_stmt.condition, &while_stmt.body);
            }
            Stmt::Return(ret) => {
                self.emit_indent();
                if let Some(value) = &ret.value {
                    self.output.push_str("return ");
                    self.gen_expr_typed(value);
                    self.output.push_str(";\n");
                } else {
                    self.output.push_str("return;\n");
                }
            }
            Stmt::Expr(expr_stmt) => {
                self.emit_indent();
                self.gen_expr_typed(&expr_stmt.expr);
                self.output.push_str(";\n");
            }
            _ => {
                // Handle other statement types
                self.emit_indent();
                self.output.push_str("/* unhandled stmt */;\n");
            }
        }
    }

    /// Infer the type of an expression
    fn infer_type(&self, expr: &Expr) -> TypeContext {
        match expr {
            Expr::Ident(ident) => {
                // Look up variable type
                self.env.lookup(&ident.name)
                    .cloned()
                    .unwrap_or(TypeContext::unknown())
            }

            Expr::Member(mem) => {
                // Get object type, then look up field type
                let obj_type = self.infer_type(&mem.object);
                if let Some(mapping) = get_typed_field_mapping(&obj_type.swc_type, &mem.property) {
                    let (_, kind) = super::type_context::map_reluxscript_to_swc(mapping.result_type_rs);
                    TypeContext {
                        reluxscript_type: mapping.result_type_rs.to_string(),
                        swc_type: mapping.result_type_swc.to_string(),
                        kind,
                        known_variant: None,
                        needs_deref: mapping.needs_deref,
                    }
                } else {
                    TypeContext::unknown()
                }
            }

            Expr::Call(call) => {
                // Handle .clone() - returns same type
                if let Expr::Member(mem) = call.callee.as_ref() {
                    if mem.property == "clone" {
                        return self.infer_type(&mem.object);
                    }
                }
                TypeContext::unknown()
            }

            _ => TypeContext::unknown(),
        }
    }

    /// Extract matches!(var, Type) pattern from an expression
    fn extract_matches_pattern(&self, expr: &Expr) -> Option<(String, String)> {
        if let Expr::Call(call) = expr {
            if let Expr::Ident(callee) = call.callee.as_ref() {
                if callee.name == "matches!" && call.args.len() == 2 {
                    let var_name = if let Expr::Ident(id) = &call.args[0] {
                        id.name.clone()
                    } else {
                        return None;
                    };

                    let type_target = if let Expr::Ident(id) = &call.args[1] {
                        id.name.clone()
                    } else {
                        return None;
                    };

                    return Some((var_name, type_target));
                }
            }
        }
        None
    }

    /// Generate standard while loop (fallback)
    fn gen_standard_while(&mut self, condition: &Expr, body: &Block) {
        self.emit_indent();
        self.output.push_str("while ");
        self.gen_expr_typed(condition);
        self.output.push_str(" {\n");
        self.indent_level += 1;
        self.gen_block_typed(body);
        self.indent_level -= 1;
        self.emit_indent();
        self.output.push_str("}\n");
    }

    /// Generate standard if statement (fallback)
    fn gen_standard_if(&mut self, condition: &Expr, then_branch: &Block, else_branch: Option<&Block>) {
        self.emit_indent();
        self.output.push_str("if ");
        self.gen_expr_typed(condition);
        self.output.push_str(" {\n");
        self.indent_level += 1;
        self.gen_block_typed(then_branch);
        self.indent_level -= 1;
        self.emit_indent();
        self.output.push_str("}");

        if let Some(else_block) = else_branch {
            self.output.push_str(" else {\n");
            self.indent_level += 1;
            self.gen_block_typed(else_block);
            self.indent_level -= 1;
            self.emit_indent();
            self.output.push_str("}");
        }
        self.output.push('\n');
    }

    /// Emit indentation
    fn emit_indent(&mut self) {
        for _ in 0..self.indent_level {
            self.output.push_str("    ");
        }
    }
}

impl Default for SwcPatternGenerator {
    fn default() -> Self {
        Self::new()
    }
}

// Helper function to access private map function
mod super_access {
    pub fn map_reluxscript_to_swc(rs_type: &str) -> (String, super::super::type_context::SwcTypeKind) {
        // Re-implement or make the original public
        match rs_type {
            "Expr" | "Expression" => ("Expr".into(), super::super::type_context::SwcTypeKind::Enum),
            "MemberExpression" => ("MemberExpr".into(), super::super::type_context::SwcTypeKind::Struct),
            "Identifier" => ("Ident".into(), super::super::type_context::SwcTypeKind::Struct),
            _ => (rs_type.to_string(), super::super::type_context::SwcTypeKind::Unknown),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_matches_pattern() {
        let gen = SwcPatternGenerator::new();

        // Create a matches!(current, MemberExpression) call
        let expr = Expr::Call(CallExpr {
            callee: Box::new(Expr::Ident(IdentExpr {
                name: "matches!".to_string(),
                span: crate::lexer::Span::new(0, 0, 0, 0),
            })),
            args: vec![
                Expr::Ident(IdentExpr {
                    name: "current".to_string(),
                    span: crate::lexer::Span::new(0, 0, 0, 0),
                }),
                Expr::Ident(IdentExpr {
                    name: "MemberExpression".to_string(),
                    span: crate::lexer::Span::new(0, 0, 0, 0),
                }),
            ],
            type_args: Vec::new(),
            optional: false,
            is_macro: true,
            span: crate::lexer::Span::new(0, 0, 0, 0),
        });

        let result = gen.extract_matches_pattern(&expr);
        assert_eq!(result, Some(("current".to_string(), "MemberExpression".to_string())));
    }
}
