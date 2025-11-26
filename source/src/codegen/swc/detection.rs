//! Module: detection
use super::SwcGenerator;
use crate::parser::*;

impl SwcGenerator {
    pub(super) fn detect_std_collections(&self, program: &Program) -> (bool, bool) {
        let mut uses_hashmap = false;
        let mut uses_hashset = false;

        // Check the program for HashMap/HashSet usage
        match &program.decl {
            TopLevelDecl::Plugin(plugin) => {
                for item in &plugin.body {
                    self.detect_collections_in_item(item, &mut uses_hashmap, &mut uses_hashset);
                }
            }
            TopLevelDecl::Writer(writer) => {
                for item in &writer.body {
                    self.detect_collections_in_item(item, &mut uses_hashmap, &mut uses_hashset);
                }
            }
            TopLevelDecl::Module(module) => {
                for item in &module.items {
                    self.detect_collections_in_item(item, &mut uses_hashmap, &mut uses_hashset);
                }
            }
            TopLevelDecl::Interface(_) => {}
        }

        (uses_hashmap, uses_hashset)
    }
    pub(super) fn detect_collections_in_item(&self, item: &PluginItem, uses_hashmap: &mut bool, uses_hashset: &mut bool) {
        match item {
            PluginItem::Function(func) => {
                self.detect_collections_in_block(&func.body, uses_hashmap, uses_hashset);
            }
            PluginItem::Struct(s) => {
                // Check struct field types for HashMap/HashSet
                for field in &s.fields {
                    self.detect_collections_in_type(&field.ty, uses_hashmap, uses_hashset);
                }
            }
            _ => {}
        }
    }
    pub(super) fn detect_collections_in_type(&self, ty: &Type, uses_hashmap: &mut bool, uses_hashset: &mut bool) {
        match ty {
            Type::Container { name, type_args } => {
                if name == "HashMap" {
                    *uses_hashmap = true;
                } else if name == "HashSet" {
                    *uses_hashset = true;
                }
                // Recursively check type arguments
                for arg in type_args {
                    self.detect_collections_in_type(arg, uses_hashmap, uses_hashset);
                }
            }
            Type::Array { element } => {
                self.detect_collections_in_type(element, uses_hashmap, uses_hashset);
            }
            Type::Tuple(elements) => {
                for elem in elements {
                    self.detect_collections_in_type(elem, uses_hashmap, uses_hashset);
                }
            }
            Type::Optional(inner) | Type::Reference { inner, .. } => {
                self.detect_collections_in_type(inner, uses_hashmap, uses_hashset);
            }
            _ => {}
        }
    }
    pub(super) fn detect_collections_in_block(&self, block: &Block, uses_hashmap: &mut bool, uses_hashset: &mut bool) {
        for stmt in &block.stmts {
            self.detect_collections_in_stmt(stmt, uses_hashmap, uses_hashset);
        }
    }
    pub(super) fn detect_collections_in_stmt(&self, stmt: &Stmt, uses_hashmap: &mut bool, uses_hashset: &mut bool) {
        match stmt {
            Stmt::Let(let_stmt) => {
                self.detect_collections_in_expr(&let_stmt.init, uses_hashmap, uses_hashset);
            }
            Stmt::Const(const_stmt) => {
                self.detect_collections_in_expr(&const_stmt.init, uses_hashmap, uses_hashset);
            }
            Stmt::Expr(expr_stmt) => {
                self.detect_collections_in_expr(&expr_stmt.expr, uses_hashmap, uses_hashset);
            }
            Stmt::If(if_stmt) => {
                self.detect_collections_in_expr(&if_stmt.condition, uses_hashmap, uses_hashset);
                self.detect_collections_in_block(&if_stmt.then_branch, uses_hashmap, uses_hashset);
                for (cond, block) in &if_stmt.else_if_branches {
                    self.detect_collections_in_expr(cond, uses_hashmap, uses_hashset);
                    self.detect_collections_in_block(block, uses_hashmap, uses_hashset);
                }
                if let Some(else_block) = &if_stmt.else_branch {
                    self.detect_collections_in_block(else_block, uses_hashmap, uses_hashset);
                }
            }
            Stmt::For(for_stmt) => {
                self.detect_collections_in_expr(&for_stmt.iter, uses_hashmap, uses_hashset);
                self.detect_collections_in_block(&for_stmt.body, uses_hashmap, uses_hashset);
            }
            Stmt::While(while_stmt) => {
                self.detect_collections_in_expr(&while_stmt.condition, uses_hashmap, uses_hashset);
                self.detect_collections_in_block(&while_stmt.body, uses_hashmap, uses_hashset);
            }
            Stmt::Loop(loop_stmt) => {
                self.detect_collections_in_block(&loop_stmt.body, uses_hashmap, uses_hashset);
            }
            Stmt::Return(ret_stmt) => {
                if let Some(expr) = &ret_stmt.value {
                    self.detect_collections_in_expr(expr, uses_hashmap, uses_hashset);
                }
            }
            _ => {}
        }
    }
    pub(super) fn detect_collections_in_expr(&self, expr: &Expr, uses_hashmap: &mut bool, uses_hashset: &mut bool) {
        match expr {
            Expr::Ident(ident) => {
                if ident.name == "HashMap" {
                    *uses_hashmap = true;
                } else if ident.name == "HashSet" {
                    *uses_hashset = true;
                }
            }
            Expr::Member(mem) => {
                self.detect_collections_in_expr(&mem.object, uses_hashmap, uses_hashset);
            }
            Expr::Call(call) => {
                self.detect_collections_in_expr(&call.callee, uses_hashmap, uses_hashset);
                for arg in &call.args {
                    self.detect_collections_in_expr(arg, uses_hashmap, uses_hashset);
                }
            }
            Expr::Binary(bin) => {
                self.detect_collections_in_expr(&bin.left, uses_hashmap, uses_hashset);
                self.detect_collections_in_expr(&bin.right, uses_hashmap, uses_hashset);
            }
            Expr::Unary(un) => {
                self.detect_collections_in_expr(&un.operand, uses_hashmap, uses_hashset);
            }
            Expr::Ref(ref_expr) => {
                self.detect_collections_in_expr(&ref_expr.expr, uses_hashmap, uses_hashset);
            }
            Expr::If(if_expr) => {
                self.detect_collections_in_expr(&if_expr.condition, uses_hashmap, uses_hashset);
                self.detect_collections_in_block(&if_expr.then_branch, uses_hashmap, uses_hashset);
                if let Some(else_block) = &if_expr.else_branch {
                    self.detect_collections_in_block(else_block, uses_hashmap, uses_hashset);
                }
            }
            Expr::Closure(closure) => {
                self.detect_collections_in_expr(&closure.body, uses_hashmap, uses_hashset);
            }
            Expr::Block(block) => {
                self.detect_collections_in_block(block, uses_hashmap, uses_hashset);
            }
            Expr::Try(inner) => {
                self.detect_collections_in_expr(inner, uses_hashmap, uses_hashset);
            }
            Expr::VecInit(vec_init) => {
                for elem in &vec_init.elements {
                    self.detect_collections_in_expr(elem, uses_hashmap, uses_hashset);
                }
            }
            Expr::StructInit(struct_init) => {
                for (_, field_expr) in &struct_init.fields {
                    self.detect_collections_in_expr(field_expr, uses_hashmap, uses_hashset);
                }
            }
            Expr::Index(idx) => {
                self.detect_collections_in_expr(&idx.object, uses_hashmap, uses_hashset);
                self.detect_collections_in_expr(&idx.index, uses_hashmap, uses_hashset);
            }
            Expr::Assign(assign) => {
                self.detect_collections_in_expr(&assign.target, uses_hashmap, uses_hashset);
                self.detect_collections_in_expr(&assign.value, uses_hashmap, uses_hashset);
            }
            Expr::Paren(inner) => {
                self.detect_collections_in_expr(inner, uses_hashmap, uses_hashset);
            }
            _ => {}
        }
    }
}
