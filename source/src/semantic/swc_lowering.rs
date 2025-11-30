//! SWC-specific AST Lowering Pass
//!
//! This pass transforms ReluxScript constructs into their final AST form
//! before semantic analysis runs. This is critical for features where
//! the transformation creates variable bindings that need to be validated.
//!
//! Transformations:
//! 1. matches! macro → if-let patterns
//!    `if matches!(expr, Pattern(binding))` → `if let Pattern(binding) = expr`
//!
//! This must run BEFORE semantic analysis so that pattern bindings are
//! visible to the resolver.

use crate::parser::{
    Program, TopLevelDecl, PluginDecl, WriterDecl, ModuleDecl, InterfaceDecl,
    PluginItem, FnDecl, Block, Stmt, Expr, IfStmt, Pattern,
};
use crate::lexer::Span;

/// SWC-specific lowering pass that transforms matches! and other constructs
pub struct SwcLowering {
}

impl SwcLowering {
    pub fn new() -> Self {
        Self {}
    }

    /// Run the lowering pass on a program
    pub fn run(&mut self, program: &mut Program) {
        match &mut program.decl {
            TopLevelDecl::Plugin(plugin) => self.lower_plugin(plugin),
            TopLevelDecl::Writer(writer) => self.lower_writer(writer),
            TopLevelDecl::Module(module) => self.lower_module(module),
            TopLevelDecl::Interface(interface) => self.lower_interface(interface),
        }
    }

    fn lower_plugin(&mut self, plugin: &mut PluginDecl) {
        for item in &mut plugin.body {
            match item {
                PluginItem::Function(func) => self.lower_function(func),
                PluginItem::Struct(_) => {}
                PluginItem::Enum(_) => {}
                PluginItem::Impl(impl_block) => {
                    for method in &mut impl_block.items {
                        self.lower_function(method);
                    }
                }
                PluginItem::PreHook(func) => self.lower_function(func),
                PluginItem::ExitHook(func) => self.lower_function(func),
            }
        }
    }

    fn lower_writer(&mut self, writer: &mut WriterDecl) {
        for item in &mut writer.body {
            match item {
                PluginItem::Function(func) => self.lower_function(func),
                PluginItem::Struct(_) => {}
                PluginItem::Enum(_) => {}
                PluginItem::Impl(impl_block) => {
                    for method in &mut impl_block.items {
                        self.lower_function(method);
                    }
                }
                PluginItem::PreHook(func) => self.lower_function(func),
                PluginItem::ExitHook(func) => self.lower_function(func),
            }
        }
    }

    fn lower_module(&mut self, module: &mut ModuleDecl) {
        for item in &mut module.items {
            match item {
                PluginItem::Function(func) => self.lower_function(func),
                PluginItem::Impl(impl_block) => {
                    for method in &mut impl_block.items {
                        self.lower_function(method);
                    }
                }
                _ => {}
            }
        }
    }

    fn lower_interface(&mut self, _interface: &mut InterfaceDecl) {
        // Interfaces don't have implementations to lower
    }

    fn lower_function(&mut self, func: &mut FnDecl) {
        self.lower_block(&mut func.body);
    }

    fn lower_block(&mut self, block: &mut Block) {
        for stmt in &mut block.stmts {
            self.lower_stmt(stmt);
        }
    }

    fn lower_stmt(&mut self, stmt: &mut Stmt) {
        match stmt {
            Stmt::If(if_stmt) => self.lower_if_stmt(if_stmt),
            Stmt::Match(match_stmt) => {
                self.lower_expr(&mut match_stmt.scrutinee);
                for arm in &mut match_stmt.arms {
                    self.lower_expr(&mut arm.body);
                }
            }
            Stmt::For(for_stmt) => {
                self.lower_expr(&mut for_stmt.iter);
                self.lower_block(&mut for_stmt.body);
            }
            Stmt::While(while_stmt) => {
                self.lower_expr(&mut while_stmt.condition);
                self.lower_block(&mut while_stmt.body);
            }
            Stmt::Return(ret_stmt) => {
                if let Some(ref mut value) = ret_stmt.value {
                    self.lower_expr(value);
                }
            }
            Stmt::Let(let_stmt) => {
                self.lower_expr(&mut let_stmt.init);
            }
            Stmt::Expr(expr_stmt) => {
                self.lower_expr(&mut expr_stmt.expr);
            }
            Stmt::Break(_) | Stmt::Continue(_) => {}
            Stmt::Const(_) | Stmt::Loop(_) | Stmt::Traverse(_) | Stmt::Function(_) | Stmt::Verbatim(_) | Stmt::CustomPropAssignment(_) => {
                // These statements don't need lowering or aren't supported yet
            }
        }
    }

    /// Transform if statements with matches! conditions
    fn lower_if_stmt(&mut self, if_stmt: &mut IfStmt) {
        // First, recursively lower the condition and branches
        self.lower_expr(&mut if_stmt.condition);
        self.lower_block(&mut if_stmt.then_branch);

        for (cond, block) in &mut if_stmt.else_if_branches {
            self.lower_expr(cond);
            self.lower_block(block);
        }

        if let Some(ref mut else_block) = if_stmt.else_branch {
            self.lower_block(else_block);
        }

        // Transform: if matches!(expr, Pattern) → if let Pattern = expr
        if if_stmt.pattern.is_none() {
            if let Expr::Matches(matches_expr) = &if_stmt.condition {
                eprintln!("[LOWERING] Transforming matches! to if-let");

                // Extract scrutinee and pattern
                let scrutinee = matches_expr.scrutinee.clone();
                let mut pattern = matches_expr.pattern.clone();

                // If the pattern is an identifier (like StringLiteral), wrap it as a variant with a binding
                // This allows field access on the matched value
                // Example: StringLiteral → StringLiteral(__inner)
                eprintln!("[LOWERING] Pattern: {:?}", pattern);
                match &pattern {
                    Pattern::Ident(name) => {
                        eprintln!("[LOWERING] Pattern is Ident({}), wrapping with __inner binding", name);
                        pattern = Pattern::Variant {
                            name: name.clone(),
                            inner: Some(Box::new(Pattern::Ident("__inner".to_string()))),
                        };
                    }
                    Pattern::Variant { name, inner } if inner.is_none() => {
                        eprintln!("[LOWERING] Pattern is Variant without binding, adding __inner");
                        pattern = Pattern::Variant {
                            name: name.clone(),
                            inner: Some(Box::new(Pattern::Ident("__inner".to_string()))),
                        };
                    }
                    _ => {
                        eprintln!("[LOWERING] Pattern already has binding or is complex, keeping as-is");
                    }
                }

                // Set the if-let pattern and replace condition
                if_stmt.pattern = Some(pattern);
                if_stmt.condition = *scrutinee;

                eprintln!("[LOWERING] Transformation complete");
            }
        }
    }

    fn lower_expr(&mut self, expr: &mut Expr) {
        match expr {
            Expr::Binary(binary) => {
                self.lower_expr(&mut binary.left);
                self.lower_expr(&mut binary.right);
            }
            Expr::Unary(unary) => {
                self.lower_expr(&mut unary.operand);
            }
            Expr::Call(call) => {
                self.lower_expr(&mut call.callee);
                for arg in &mut call.args {
                    self.lower_expr(arg);
                }
            }
            Expr::Member(member) => {
                self.lower_expr(&mut member.object);
            }
            Expr::Index(index) => {
                self.lower_expr(&mut index.object);
                self.lower_expr(&mut index.index);
            }
            Expr::Paren(inner) => {
                self.lower_expr(inner);
            }
            Expr::Block(block) => {
                self.lower_block(block);
            }
            Expr::If(if_expr) => {
                self.lower_expr(&mut if_expr.condition);
                self.lower_block(&mut if_expr.then_branch);
                if let Some(ref mut else_branch) = if_expr.else_branch {
                    self.lower_block(else_branch);
                }
            }
            Expr::Match(match_expr) => {
                self.lower_expr(&mut match_expr.scrutinee);
                for arm in &mut match_expr.arms {
                    self.lower_expr(&mut arm.body);
                }
            }
            Expr::Matches(matches_expr) => {
                // Lower the scrutinee
                self.lower_expr(&mut matches_expr.scrutinee);
                // The pattern itself doesn't need lowering (it's a pattern, not an expression)
            }
            Expr::StructInit(struct_init) => {
                for (_, field_expr) in &mut struct_init.fields {
                    self.lower_expr(field_expr);
                }
            }
            Expr::Assign(assign) => {
                self.lower_expr(&mut assign.target);
                self.lower_expr(&mut assign.value);
            }
            Expr::CompoundAssign(compound) => {
                self.lower_expr(&mut compound.target);
                self.lower_expr(&mut compound.value);
            }
            Expr::Closure(closure) => {
                self.lower_expr(&mut closure.body);
            }
            // Leaf expressions and other expressions that don't need recursion
            _ => {}
        }
    }
}
