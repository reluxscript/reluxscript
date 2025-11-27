//! Ownership checking pass for ReluxScript
//!
//! Enforces the Clone-to-Own principle: explicit .clone() required for value extraction

use crate::parser::*;
use crate::semantic::SemanticError;

/// Ownership checker - validates borrow and ownership rules
pub struct OwnershipChecker {
    errors: Vec<SemanticError>,
    warnings: Vec<SemanticError>,
}

impl OwnershipChecker {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Run ownership checking
    pub fn check(&mut self, program: &Program) -> (Vec<SemanticError>, Vec<SemanticError>) {
        match &program.decl {
            TopLevelDecl::Plugin(plugin) => self.check_plugin(plugin),
            TopLevelDecl::Writer(writer) => self.check_writer(writer),
            TopLevelDecl::Interface(_) => {} // Interfaces don't have ownership semantics
            TopLevelDecl::Module(module) => self.check_module(module),
        }

        (
            std::mem::take(&mut self.errors),
            std::mem::take(&mut self.warnings),
        )
    }

    fn check_plugin(&mut self, plugin: &PluginDecl) {
        for item in &plugin.body {
            if let PluginItem::Function(f) = item {
                self.check_function(f);
            }
        }
    }

    fn check_writer(&mut self, writer: &WriterDecl) {
        for item in &writer.body {
            if let PluginItem::Function(f) = item {
                self.check_function(f);
            }
        }
    }

    fn check_module(&mut self, module: &ModuleDecl) {
        for item in &module.items {
            if let PluginItem::Function(f) = item {
                self.check_function(f);
            }
        }
    }

    fn check_function(&mut self, f: &FnDecl) {
        self.check_block(&f.body);
    }

    fn check_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        match stmt {
            Stmt::Let(let_stmt) => {
                // Check if assigning from a member access without clone
                self.check_needs_clone(&let_stmt.init, let_stmt.span);
                self.check_expr(&let_stmt.init);
            }

            Stmt::Const(const_stmt) => {
                self.check_needs_clone(&const_stmt.init, const_stmt.span);
                self.check_expr(&const_stmt.init);
            }

            Stmt::Expr(expr_stmt) => {
                self.check_expr(&expr_stmt.expr);
                // Check for direct property mutation
                self.check_property_mutation(&expr_stmt.expr);
            }

            Stmt::If(if_stmt) => {
                self.check_expr(&if_stmt.condition);
                self.check_block(&if_stmt.then_branch);
                for (cond, block) in &if_stmt.else_if_branches {
                    self.check_expr(cond);
                    self.check_block(block);
                }
                if let Some(ref else_block) = if_stmt.else_branch {
                    self.check_block(else_block);
                }
            }

            Stmt::Match(match_stmt) => {
                self.check_expr(&match_stmt.scrutinee);
                for arm in &match_stmt.arms {
                    self.check_expr(&arm.body);
                }
            }

            Stmt::For(for_stmt) => {
                self.check_expr(&for_stmt.iter);
                self.check_block(&for_stmt.body);
            }

            Stmt::While(while_stmt) => {
                self.check_expr(&while_stmt.condition);
                self.check_block(&while_stmt.body);
            }

            Stmt::Loop(loop_stmt) => {
                self.check_block(&loop_stmt.body);
            }

            Stmt::Return(return_stmt) => {
                if let Some(ref value) = return_stmt.value {
                    self.check_expr(value);
                }
            }

            Stmt::Break(_) | Stmt::Continue(_) => {}

            Stmt::Traverse(traverse_stmt) => {
                // Check the target expression
                self.check_expr(&traverse_stmt.target);

                // Check the traverse kind
                match &traverse_stmt.kind {
                    crate::parser::TraverseKind::Inline(inline) => {
                        // Check state variables
                        for let_stmt in &inline.state {
                            self.check_needs_clone(&let_stmt.init, let_stmt.span);
                            self.check_expr(&let_stmt.init);
                        }

                        // Check methods
                        for method in &inline.methods {
                            self.check_function(method);
                        }
                    }
                    crate::parser::TraverseKind::Delegated(_) => {
                        // No ownership checks needed for delegated traversal
                    }
                }
            }

            Stmt::Function(fn_decl) => {
                // Check nested function
                self.check_function(fn_decl);
            }

            Stmt::Verbatim(_) => {
                // Verbatim blocks are opaque to ownership checking
                // No analysis performed on raw code
            }
        }
    }

    fn check_expr(&mut self, expr: &Expr) {
        match expr {
            Expr::Binary(binary) => {
                self.check_expr(&binary.left);
                self.check_expr(&binary.right);
            }

            Expr::Unary(unary) => {
                self.check_expr(&unary.operand);
            }

            Expr::Call(call) => {
                self.check_expr(&call.callee);
                for arg in &call.args {
                    self.check_expr(arg);
                }
            }

            Expr::Member(member) => {
                self.check_expr(&member.object);
            }

            Expr::Index(index) => {
                self.check_expr(&index.object);
                self.check_expr(&index.index);
            }

            Expr::StructInit(init) => {
                for (_, value) in &init.fields {
                    self.check_expr(value);
                }
            }

            Expr::VecInit(vec_init) => {
                for elem in &vec_init.elements {
                    self.check_expr(elem);
                }
            }

            Expr::If(if_expr) => {
                self.check_expr(&if_expr.condition);
                self.check_block(&if_expr.then_branch);
                if let Some(ref else_block) = if_expr.else_branch {
                    self.check_block(else_block);
                }
            }

            Expr::Match(match_expr) => {
                self.check_expr(&match_expr.scrutinee);
                for arm in &match_expr.arms {
                    self.check_expr(&arm.body);
                }
            }

            Expr::Closure(closure) => {
                self.check_expr(&closure.body);
            }

            Expr::Ref(ref_expr) => {
                self.check_expr(&ref_expr.expr);
            }

            Expr::Deref(deref_expr) => {
                self.check_expr(&deref_expr.expr);
            }

            Expr::Assign(assign) => {
                self.check_expr(&assign.target);
                self.check_expr(&assign.value);

                // Check for node replacement (statement lowering) - this is OK
                if matches!(assign.target.as_ref(), Expr::Deref(_)) {
                    // *node = ... is the pattern for node replacement - this is allowed
                } else {
                    // Regular assignment, check for property mutation
                    self.check_property_mutation(&Expr::Assign(assign.clone()));
                }
            }

            Expr::CompoundAssign(compound) => {
                self.check_expr(&compound.target);
                self.check_expr(&compound.value);
            }

            Expr::Range(range) => {
                if let Some(ref start) = range.start {
                    self.check_expr(start);
                }
                if let Some(ref end) = range.end {
                    self.check_expr(end);
                }
            }

            Expr::Paren(inner) => {
                self.check_expr(inner);
            }

            Expr::Tuple(elements) => {
                for elem in elements {
                    self.check_expr(elem);
                }
            }

            Expr::Block(block) => {
                for stmt in &block.stmts {
                    self.check_stmt(stmt);
                }
            }

            Expr::Try(inner) => {
                self.check_expr(inner);
            }

            Expr::Matches(matches_expr) => {
                // Check the scrutinee expression
                self.check_expr(&matches_expr.scrutinee);
                // Pattern doesn't need ownership checking
            }

            Expr::Return(value) => {
                if let Some(ref expr) = value {
                    self.check_expr(expr);
                }
            }

            Expr::Break => {}
            Expr::Continue => {}

            Expr::RegexCall(regex_call) => {
                self.check_expr(&regex_call.text_arg);
                if let Some(ref repl) = regex_call.replacement_arg {
                    self.check_expr(repl);
                }
            }

            Expr::Literal(_) | Expr::Ident(_) => {}
        }
    }

    /// Check if an expression needs .clone() for ownership transfer
    fn check_needs_clone(&mut self, expr: &Expr, span: crate::lexer::Span) {
        match expr {
            // Member access needs clone (unless it's a method call)
            Expr::Member(member) => {
                // This is borrowing from a field - needs clone
                self.errors.push(
                    SemanticError::new(
                        "RS001",
                        "Implicit borrow not allowed",
                        span,
                    )
                    .with_hint(format!(
                        "use explicit clone: {}.{}.clone()",
                        self.expr_to_string(&member.object),
                        member.property
                    )),
                );
            }

            // Call is OK if it's a .clone() call
            Expr::Call(call) => {
                if let Expr::Member(member) = call.callee.as_ref() {
                    if member.property != "clone" {
                        // Method call that returns a value - check the object
                        // But this is generally OK as the method returns ownership
                    }
                }
            }

            // Parenthesized expression - check inner
            Expr::Paren(inner) => {
                self.check_needs_clone(inner, span);
            }

            // Tuple expressions produce owned values (newly created tuple)
            Expr::Tuple(_) => {}

            // Block expressions produce owned values
            Expr::Block(_) => {}

            // Try expressions produce owned values (unwrapped from Result)
            Expr::Try(_) => {}

            // Return/break/continue don't produce values (they diverge)
            Expr::Return(_) | Expr::Break | Expr::Continue => {}

            // These are OK - they produce owned values
            Expr::Literal(_)
            | Expr::Ident(_)
            | Expr::Binary(_)
            | Expr::Unary(_)
            | Expr::StructInit(_)
            | Expr::VecInit(_)
            | Expr::If(_)
            | Expr::Match(_)
            | Expr::Closure(_) => {}

            // Index might need clone
            Expr::Index(index) => {
                self.errors.push(
                    SemanticError::new(
                        "RS001",
                        "Implicit borrow not allowed",
                        span,
                    )
                    .with_hint(format!(
                        "use explicit clone: {}[...].clone()",
                        self.expr_to_string(&index.object)
                    )),
                );
            }

            _ => {}
        }
    }

    /// Check for forbidden property mutation
    fn check_property_mutation(&mut self, expr: &Expr) {
        if let Expr::Assign(assign) = expr {
            // Check if target is a member expression (direct property mutation)
            if let Expr::Member(member) = assign.target.as_ref() {
                // Allow mutations on 'self' (writer/plugin state)
                // This should work for self.field, self.state.field, etc.
                if self.starts_with_self(&member.object) {
                    return; // self.* = value is allowed
                }

                // Direct property mutation on AST nodes is not allowed
                let target_name = self.expr_to_string(assign.target.as_ref());
                self.errors.push(
                    SemanticError::new(
                        "RS002",
                        &format!("Direct property mutation not allowed on '{}'", target_name),
                        member.span,
                    )
                    .with_hint("AST node mutation can break Babel's scope tracker. Use the clone-and-rebuild pattern: `*node = NodeType { field: new_value, ..node.clone() }`"),
                );
            }
        }
    }

    /// Check if an expression starts with 'self'
    fn starts_with_self(&self, expr: &Expr) -> bool {
        match expr {
            Expr::Ident(ident) => ident.name == "self",
            Expr::Member(member) => self.starts_with_self(&member.object),
            _ => false,
        }
    }

    /// Convert expression to string for error messages
    fn expr_to_string(&self, expr: &Expr) -> String {
        match expr {
            Expr::Ident(ident) => ident.name.clone(),
            Expr::Member(member) => {
                format!("{}.{}", self.expr_to_string(&member.object), member.property)
            }
            Expr::Deref(deref) => format!("*{}", self.expr_to_string(&deref.expr)),
            _ => "expr".to_string(),
        }
    }
}

impl Default for OwnershipChecker {
    fn default() -> Self {
        Self::new()
    }
}
