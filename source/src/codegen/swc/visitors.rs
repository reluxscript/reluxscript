//! Visitor Method Generation
//!
//! This module generates visitor methods for SWC's VisitMut/Visit traits.

use super::SwcGenerator;
use crate::parser::*;
use crate::codegen::swc_decorator::*;
use crate::codegen::decorated_ast::*;

impl SwcGenerator {
    pub(super) fn is_visitor_method(&self, f: &FnDecl) -> bool {
        // Must be named visit_*
        if !f.name.starts_with("visit_") {
            return false;
        }

        // Must have no return type
        if f.return_type.is_some() {
            return false;
        }

        // Must have at least one parameter that is a mutable reference
        if let Some(first_param) = f.params.first() {
            if let Type::Reference { mutable, .. } = &first_param.ty {
                return *mutable;
            }
        }

        false
    }
    pub(super) fn gen_decorated_visitor_method(&mut self, func: &DecoratedFnDecl) {
        // Generate visitor method using decorated AST (metadata-driven!)
        let swc_name = self.visitor_name_to_swc(&func.name);
        let swc_type = self.visitor_name_to_swc_type(&func.name);

        self.emit_line("");
        self.emit_line(&format!("fn {}(&mut self, n: &mut {}) {{", swc_name, swc_type));
        self.indent += 1;

        // Store parameter rename (e.g., "node" -> "n")
        for param in &func.params {
            self.param_renames.insert(param.name.clone(), "n".to_string());
        }

        // Generate function body from decorated AST
        self.gen_decorated_block(&func.body);

        // Clean up parameter renames
        for param in &func.params {
            self.param_renames.remove(&param.name);
        }

        self.indent -= 1;
        self.emit_line("}");
    }
    pub(super) fn gen_visitor_method(&mut self, f: &FnDecl) {
        let swc_name = self.visitor_name_to_swc(&f.name);
        let swc_type = self.visitor_name_to_swc_type(&f.name);

        self.emit_line("");
        self.emit_line(&format!("fn {}(&mut self, n: &mut {}) {{", swc_name, swc_type));
        self.indent += 1;

        // Set up parameter renames for visitor methods
        // The first parameter (typically "node") maps to "n"
        if let Some(first_param) = f.params.first() {
            self.param_renames.insert(first_param.name.clone(), "n".to_string());
        }

        // Track the parameter's type in the environment
        // Both "n" and the original parameter name should map to the swc_type
        self.type_env.push_scope();
        let param_ctx = TypeContext {
            reluxscript_type: swc_type.clone(),
            swc_type: swc_type.clone(),
            kind: super::type_context::SwcTypeKind::Struct,
            known_variant: None,
            needs_deref: false,
        };
        self.type_env.define("n", param_ctx.clone());
        if let Some(first_param) = f.params.first() {
            self.type_env.define(&first_param.name, param_ctx);
        }

        // Generate body
        self.gen_block(&f.body);

        // Clear environment and renames
        self.type_env.pop_scope();
        self.param_renames.clear();

        self.indent -= 1;
        self.emit_line("}");
    }
    pub(super) fn gen_visit_method(&mut self, f: &FnDecl) {
        // For Visit (read-only), parameters are & not &mut
        let mut swc_name = self.visitor_name_to_swc(&f.name);
        // Convert visit_mut_ to visit_ for Visit trait
        if swc_name.starts_with("visit_mut_") {
            swc_name = swc_name.replace("visit_mut_", "visit_");
        }
        let swc_type = self.visitor_name_to_swc_type(&f.name);

        self.emit_line("");
        self.emit_line(&format!("fn {}(&mut self, n: &{}) {{", swc_name, swc_type));
        self.indent += 1;

        // Set up parameter renames for visitor methods
        // The first parameter (typically "node") maps to "n"
        if let Some(first_param) = f.params.first() {
            self.param_renames.insert(first_param.name.clone(), "n".to_string());
        }

        // Track the parameter's type in the environment
        // Both "n" and the original parameter name should map to the swc_type
        self.type_env.push_scope();
        let param_ctx = TypeContext {
            reluxscript_type: swc_type.clone(),
            swc_type: swc_type.clone(),
            kind: super::type_context::SwcTypeKind::Struct,
            known_variant: None,
            needs_deref: false,
        };
        self.type_env.define("n", param_ctx.clone());
        if let Some(first_param) = f.params.first() {
            self.type_env.define(&first_param.name, param_ctx);
        }

        // Generate body - for visitor methods, all statements should have semicolons
        // (they return (), not an implicit value)
        for stmt in &f.body.stmts {
            self.gen_stmt(stmt);
        }

        // Clear environment and renames
        self.type_env.pop_scope();
        self.param_renames.clear();

        self.indent -= 1;
        self.emit_line("}");
    }
}
