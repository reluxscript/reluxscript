//! Type checking pass for ReluxScript

use crate::parser::*;
use crate::semantic::{SemanticError, TypeEnv, TypeInfo, types::ast_type_to_type_info};

/// Type checker - validates types throughout the AST
pub struct TypeChecker {
    env: TypeEnv,
    errors: Vec<SemanticError>,
    /// Current function's return type (for return statement checking)
    current_return_type: Option<TypeInfo>,
}

impl TypeChecker {
    pub fn new(env: &TypeEnv) -> Self {
        Self {
            env: env.clone(),
            errors: Vec::new(),
            current_return_type: None,
        }
    }

    /// Run type checking
    pub fn check(&mut self, program: &Program) -> Result<(), Vec<SemanticError>> {
        match &program.decl {
            TopLevelDecl::Plugin(plugin) => self.check_plugin(plugin),
            TopLevelDecl::Writer(writer) => self.check_writer(writer),
            TopLevelDecl::Interface(_) => {} // Interfaces are type declarations, not code
            TopLevelDecl::Module(module) => self.check_module(module),
        }

        if self.errors.is_empty() {
            Ok(())
        } else {
            Err(std::mem::take(&mut self.errors))
        }
    }

    /// Get the type environment
    pub fn into_env(self) -> TypeEnv {
        self.env
    }

    fn check_plugin(&mut self, plugin: &PluginDecl) {
        // Don't push/pop scope - we want to retain all types for codegen
        for item in &plugin.body {
            if let PluginItem::Function(f) = item {
                self.check_function(f);
            }
        }
    }

    fn check_writer(&mut self, writer: &WriterDecl) {
        // Don't push/pop scope - we want to retain all types for codegen
        for item in &writer.body {
            if let PluginItem::Function(f) = item {
                self.check_function(f);
            }
        }
    }

    fn check_module(&mut self, module: &ModuleDecl) {
        // Don't push/pop scope - we want to retain all types for codegen
        for item in &module.items {
            if let PluginItem::Function(f) = item {
                self.check_function(f);
            }
        }
    }

    fn check_function(&mut self, f: &FnDecl) {
        let return_type = f
            .return_type
            .as_ref()
            .map(ast_type_to_type_info)
            .unwrap_or(TypeInfo::Unit);

        self.current_return_type = Some(return_type);
        // NOTE: We don't push/pop scope for functions because we want the TypeEnv
        // to retain all variable types (including parameters and narrowed types)
        // so they're available to the decorator during codegen.

        // Define parameters
        for param in &f.params {
            let ty = ast_type_to_type_info(&param.ty);
            self.env.define(param.name.clone(), ty);
        }

        // Check body
        self.check_block(&f.body);

        // Don't pop scope - keep all types for decorator
        self.current_return_type = None;
    }

    fn check_block(&mut self, block: &Block) {
        for stmt in &block.stmts {
            self.check_stmt(stmt);
        }
    }

    /// Infer the type of a block
    /// A block's type is the type of its final expression (if any), or Unit
    fn infer_block(&mut self, block: &Block) -> TypeInfo {
        println!("DEBUG infer_block: block has {} statements", block.stmts.len());

        // Check all statements except potentially the last
        for stmt in &block.stmts {
            match stmt {
                // If any statement is a return/break/continue, the block has type Never
                Stmt::Return(_) => {
                    eprintln!("DEBUG infer_block: found Return, returning Never");
                    return TypeInfo::Never;
                }
                Stmt::Break(_) => return TypeInfo::Never,
                Stmt::Continue(_) => return TypeInfo::Never,
                _ => self.check_stmt(stmt),
            }
        }

        // If the last statement is an expression statement (no semicolon),
        // the block evaluates to that expression's type
        // If/match statements can also be expressions
        if let Some(last_stmt) = block.stmts.last() {
            match last_stmt {
                Stmt::Expr(expr_stmt) => {
                    return self.infer_expr(&expr_stmt.expr);
                }
                Stmt::If(if_stmt) => {
                    // Treat if statement as expression
                    let if_expr = Box::new(IfExpr {
                        condition: if_stmt.condition.clone(),
                        pattern: if_stmt.pattern.clone(),
                        then_branch: if_stmt.then_branch.clone(),
                        else_branch: if_stmt.else_branch.clone(),
                        span: if_stmt.span,
                    });
                    return self.infer_expr(&Expr::If(if_expr));
                }
                _ => {}
            }
        }

        TypeInfo::Unit
    }

    fn check_stmt(&mut self, stmt: &Stmt) {
        eprintln!("DEBUG check_stmt: checking statement variant {:?}", std::mem::discriminant(stmt));
        match stmt {
            Stmt::Let(let_stmt) => {
                // If there's a type annotation, use it as the expected type for bidirectional inference
                let expected_type = let_stmt.ty.as_ref().map(ast_type_to_type_info);

                // Infer type from initializer if present
                let init_type = if let Some(ref init) = let_stmt.init {
                    Some(self.infer_expr_with_expected(init, expected_type.as_ref()))
                } else {
                    None
                };

                if let Some(declared_type) = expected_type {
                    // Has type annotation
                    if let Some(ref inferred) = init_type {
                        if !inferred.is_assignable_to(&declared_type) {
                            self.errors.push(SemanticError::new(
                                "RS003",
                                format!(
                                    "Type mismatch: expected {}, found {}",
                                    declared_type.display_name(),
                                    inferred.display_name()
                                ),
                                let_stmt.span,
                            ));
                        }
                    }
                    self.define_pattern_in_env(&let_stmt.pattern, declared_type);
                } else if let Some(inferred) = init_type {
                    // No type annotation, use inferred type
                    self.define_pattern_in_env(&let_stmt.pattern, inferred);
                } else {
                    // No type and no init - use Unknown
                    self.define_pattern_in_env(&let_stmt.pattern, TypeInfo::Unknown);
                }
            }

            Stmt::Const(const_stmt) => {
                let init_type = self.infer_expr(&const_stmt.init);

                if let Some(ref type_ann) = const_stmt.ty {
                    let declared_type = ast_type_to_type_info(type_ann);
                    if !init_type.is_assignable_to(&declared_type) {
                        self.errors.push(SemanticError::new(
                            "RS003",
                            format!(
                                "Type mismatch: expected {}, found {}",
                                declared_type.display_name(),
                                init_type.display_name()
                            ),
                            const_stmt.span,
                        ));
                    }
                    self.env.define(const_stmt.name.clone(), declared_type);
                } else {
                    self.env.define(const_stmt.name.clone(), init_type);
                }
            }

            Stmt::Expr(expr_stmt) => {
                self.infer_expr(&expr_stmt.expr);
            }

            Stmt::If(if_stmt) => {
                let cond_type = self.infer_expr(&if_stmt.condition);

                // For if-let, the condition is a pattern match expression, not a boolean
                // Only check for bool if there's no pattern
                if if_stmt.pattern.is_none() && !matches!(cond_type, TypeInfo::Bool | TypeInfo::Unknown) {
                    self.errors.push(SemanticError::new(
                        "RS003",
                        format!(
                            "Condition must be bool, found {}",
                            cond_type.display_name()
                        ),
                        if_stmt.span,
                    ));
                }

                // Type narrowing for if-let patterns (positive matches)
                // NOTE: Don't push/pop scope here - we want the narrowing to persist
                // in semantic_type_env so the decorator can see it

                // If there's a pattern, try to narrow the scrutinee type
                if let Some(ref pattern) = if_stmt.pattern {
                    eprintln!("DEBUG narrowing: found if-let pattern, attempting type narrowing");

                    // Extract the scrutinee variable name (only works for simple identifiers)
                    if let Some(var_name) = self.extract_simple_scrutinee(&if_stmt.condition) {
                        // Get the original type before narrowing
                        let original_type = self.env.lookup(&var_name).cloned();

                        // Get the narrowed type from the pattern
                        if let Some(narrowed_type) = self.pattern_to_concrete_type(pattern) {
                            // Unwrap Ref if the original type is a reference
                            let unwrapped_original = original_type.as_ref().and_then(|t| {
                                if let TypeInfo::Ref { inner, .. } = t {
                                    Some(inner.as_ref().clone())
                                } else {
                                    Some(t.clone())
                                }
                            });

                            // If narrowing from a parent enum, create NarrowedAstNode
                            let final_type = if let (Some(TypeInfo::AstNode(parent_enum)), TypeInfo::AstNode(variant_type)) = (&unwrapped_original, &narrowed_type) {
                                // Extract variant name from pattern
                                let variant_name = match pattern {
                                    Pattern::Variant { name, .. } => name.clone(),
                                    _ => variant_type.clone(),
                                };
                                eprintln!("DEBUG narrowing: creating NarrowedAstNode {{ current: {}, parent: {}, variant: {} }}", variant_type, parent_enum, variant_name);
                                TypeInfo::NarrowedAstNode {
                                    current_type: variant_type.clone(),
                                    parent_enum: parent_enum.clone(),
                                    variant: variant_name,
                                }
                            } else {
                                narrowed_type
                            };

                            eprintln!("DEBUG narrowing: shadowing '{}' with type {:?} in then-branch", var_name, final_type);
                            // Shadow the variable with the narrowed type - this will persist!
                            self.env.define(var_name, final_type);
                        }
                    }

                    // Also register bindings INSIDE the pattern (e.g., `arg` in `Some(arg)`)
                    // Get the type of the scrutinee to know what type the bindings should have
                    let scrutinee_type = self.infer_expr(&if_stmt.condition);
                    self.register_pattern_bindings(pattern, &scrutinee_type);
                }

                self.check_block(&if_stmt.then_branch);

                // Type narrowing for negated matches with early return
                // Pattern: if !matches!(x, Type) { return; }
                // After this pattern, we know x IS Type in the continuation
                if let Expr::Unary(unary) = &if_stmt.condition {
                    // Check if it's a NOT operation
                    if unary.op == UnaryOp::Not {
                        // Check if the operand is a Matches expression
                        if let Expr::Matches(matches_expr) = unary.operand.as_ref() {
                            // Check if the then-branch has an early return
                            if self.has_early_return(&if_stmt.then_branch) {
                                eprintln!("DEBUG narrowing: detected !matches!() with early return pattern");

                                // Extract scrutinee variable name
                                if let Some(var_name) = self.extract_simple_scrutinee(&matches_expr.scrutinee) {
                                    // Get the narrowed type from the pattern
                                    let narrowed_type = if let Pattern::Variant { name, .. } | Pattern::Ident(name) = &matches_expr.pattern {
                                        if name == "Some" {
                                            // Special case: Some pattern narrows Option<T> to T
                                            eprintln!("DEBUG narrowing: pattern 'Some' detected, extracting inner type from Option");
                                            let scrutinee_type = self.infer_expr(&matches_expr.scrutinee);

                                            // Unwrap Ref if present
                                            let scrutinee_type = match scrutinee_type {
                                                TypeInfo::Ref { inner, .. } => *inner,
                                                other => other,
                                            };

                                            if let TypeInfo::Option(inner) = scrutinee_type {
                                                eprintln!("DEBUG narrowing: extracted inner type from Option: {:?}", inner);
                                                Some(*inner)
                                            } else {
                                                eprintln!("DEBUG narrowing: scrutinee is not Option type, got: {:?}", scrutinee_type);
                                                None
                                            }
                                        } else {
                                            self.pattern_to_concrete_type(&matches_expr.pattern)
                                        }
                                    } else {
                                        self.pattern_to_concrete_type(&matches_expr.pattern)
                                    };

                                    if let Some(narrowed_type) = narrowed_type {
                                        eprintln!("DEBUG narrowing: negated matches with early return - narrowing '{}' to {:?} after if-statement", var_name, narrowed_type);
                                        // Narrow in the CURRENT scope (not a new scope)
                                        // After the early return, we know the variable has the narrowed type
                                        self.env.define(var_name, narrowed_type);
                                    }
                                }
                            }
                        }
                    }
                }

                for (cond, block) in &if_stmt.else_if_branches {
                    let cond_type = self.infer_expr(cond);
                    if !matches!(cond_type, TypeInfo::Bool | TypeInfo::Unknown) {
                        self.errors.push(SemanticError::new(
                            "RS003",
                            format!(
                                "Condition must be bool, found {}",
                                cond_type.display_name()
                            ),
                            if_stmt.span,
                        ));
                    }
                    self.env.push_scope();
                    self.check_block(block);
                    self.env.pop_scope();
                }

                if let Some(ref else_block) = if_stmt.else_branch {
                    self.env.push_scope();
                    self.check_block(else_block);
                    self.env.pop_scope();
                }
            }

            Stmt::Match(match_stmt) => {
                let _scrutinee_type = self.infer_expr(&match_stmt.scrutinee);
                for arm in &match_stmt.arms {
                    self.env.push_scope();
                    self.infer_expr(&arm.body);
                    self.env.pop_scope();
                }
            }

            Stmt::For(for_stmt) => {
                let iter_type = self.infer_expr(&for_stmt.iter);

                // Determine element type
                let elem_type = match &iter_type {
                    TypeInfo::Vec(inner) => (**inner).clone(),
                    TypeInfo::Ref { inner, .. } => {
                        if let TypeInfo::Vec(elem) = inner.as_ref() {
                            TypeInfo::Ref {
                                mutable: false,
                                inner: elem.clone(),
                            }
                        } else {
                            TypeInfo::Unknown
                        }
                    }
                    _ => TypeInfo::Unknown,
                };

                self.env.push_scope();
                // Define variables from pattern
                self.define_pattern_in_env(&for_stmt.pattern, elem_type);
                self.check_block(&for_stmt.body);
                self.env.pop_scope();
            }

            Stmt::While(while_stmt) => {
                let cond_type = self.infer_expr(&while_stmt.condition);
                if !matches!(cond_type, TypeInfo::Bool | TypeInfo::Unknown) {
                    self.errors.push(SemanticError::new(
                        "RS003",
                        format!(
                            "Condition must be bool, found {}",
                            cond_type.display_name()
                        ),
                        while_stmt.span,
                    ));
                }

                self.env.push_scope();
                self.check_block(&while_stmt.body);
                self.env.pop_scope();
            }

            Stmt::Loop(loop_stmt) => {
                self.env.push_scope();
                self.check_block(&loop_stmt.body);
                self.env.pop_scope();
            }

            Stmt::Return(return_stmt) => {
                if let Some(ref value) = return_stmt.value {
                    // Clone expected type to avoid borrow issues
                    let expected_return = self.current_return_type.clone();
                    // Pass expected return type for bidirectional inference
                    let value_type = self.infer_expr_with_expected(value, expected_return.as_ref());
                    if let Some(ref expected) = expected_return {
                        if !value_type.is_assignable_to(expected) {
                            self.errors.push(SemanticError::new(
                                "RS003",
                                format!(
                                    "Return type mismatch: expected {}, found {}",
                                    expected.display_name(),
                                    value_type.display_name()
                                ),
                                return_stmt.span,
                            ));
                        }
                    }
                } else if let Some(ref expected) = self.current_return_type {
                    if !matches!(expected, TypeInfo::Unit | TypeInfo::Unknown) {
                        self.errors.push(SemanticError::new(
                            "RS003",
                            format!(
                                "Return type mismatch: expected {}, found ()",
                                expected.display_name()
                            ),
                            return_stmt.span,
                        ));
                    }
                }
            }

            Stmt::Break(_) | Stmt::Continue(_) => {}

            Stmt::Traverse(traverse_stmt) => {
                // Check the target expression
                self.infer_expr(&traverse_stmt.target);

                // Check the traverse kind
                match &traverse_stmt.kind {
                    crate::parser::TraverseKind::Inline(inline) => {
                        self.env.push_scope();

                        // Check state variables
                        for let_stmt in &inline.state {
                            let init_type = if let Some(ref init) = let_stmt.init {
                                Some(self.infer_expr(init))
                            } else {
                                None
                            };
                            if let Some(ref type_ann) = let_stmt.ty {
                                let declared_type = ast_type_to_type_info(type_ann);
                                if let Some(ref inferred) = init_type {
                                    if !inferred.is_assignable_to(&declared_type) {
                                        self.errors.push(SemanticError::new(
                                            "RS003",
                                            format!(
                                                "Type mismatch: expected {}, found {}",
                                                declared_type.display_name(),
                                                inferred.display_name()
                                            ),
                                            let_stmt.span,
                                        ));
                                    }
                                }
                                self.define_pattern_in_env(&let_stmt.pattern, declared_type);
                            } else if let Some(inferred) = init_type {
                                self.define_pattern_in_env(&let_stmt.pattern, inferred);
                            } else {
                                self.define_pattern_in_env(&let_stmt.pattern, TypeInfo::Unknown);
                            }
                        }

                        // Check methods
                        for method in &inline.methods {
                            self.check_function(method);
                        }

                        self.env.pop_scope();
                    }
                    crate::parser::TraverseKind::Delegated(_visitor_name) => {
                        // Visitor name validation would happen here
                    }
                }
            }

            Stmt::Function(fn_decl) => {
                // Nested function declaration
                // Define the function in the current scope
                let param_types: Vec<TypeInfo> = fn_decl.params.iter()
                    .map(|p| ast_type_to_type_info(&p.ty))
                    .collect();
                let ret_type = fn_decl.return_type.as_ref()
                    .map(ast_type_to_type_info)
                    .unwrap_or(TypeInfo::Unit);
                let fn_type = TypeInfo::Function {
                    params: param_types,
                    ret: Box::new(ret_type),
                };
                self.env.define(fn_decl.name.clone(), fn_type);

                // Type check the function body in a new scope
                self.env.push_scope();
                // Define parameters
                for param in &fn_decl.params {
                    let param_type = ast_type_to_type_info(&param.ty);
                    self.env.define(param.name.clone(), param_type);
                }
                // Check the body
                self.check_block(&fn_decl.body);
                self.env.pop_scope();
            }

            Stmt::Verbatim(_) => {
                // Verbatim blocks are opaque to type checking
                // No analysis performed on raw code
            }

            Stmt::CustomPropAssignment(assign) => {
                // Check the node and value expressions
                let _node_type = self.infer_expr(&assign.node);
                let _value_type = self.infer_expr(&assign.value);
                // TODO: Implement custom property type tracking
                // For now, we just validate the expressions compile
            }

            Stmt::Unsafe(unsafe_block) => {
                // Type check statements inside the unsafe block
                for stmt in &unsafe_block.body.stmts {
                    self.check_stmt(stmt);
                }
            }

        }
    }

    /// Define variables from a pattern in the current environment
    fn define_pattern_in_env(&mut self, pattern: &Pattern, type_info: TypeInfo) {
        match pattern {
            Pattern::Ident(name) => {
                eprintln!("[TYPE ENV DEFINE] Defining '{}' with type {:?}", name, type_info);
                self.env.define(name.clone(), type_info);
            }
            Pattern::Tuple(patterns) => {
                // Extract tuple element types if available
                match &type_info {
                    TypeInfo::Tuple(elem_types) => {
                        // Match each pattern with its corresponding type
                        for (i, pat) in patterns.iter().enumerate() {
                            let elem_type = elem_types.get(i)
                                .cloned()
                                .unwrap_or(TypeInfo::Unknown);
                            self.define_pattern_in_env(pat, elem_type);
                        }
                    }
                    _ => {
                        // If not a tuple type, give all elements Unknown type
                        for pat in patterns {
                            self.define_pattern_in_env(pat, TypeInfo::Unknown);
                        }
                    }
                }
            }
            Pattern::Array(_) => {
                // Array destructuring not yet implemented
            }
            Pattern::Object(_) => {
                // Object destructuring not yet implemented
            }
            Pattern::Rest(_) => {
                // Rest pattern not yet implemented
            }
            Pattern::Or(patterns) => {
                // For OR patterns, all branches must bind the same variables with same types
                // For now, just define variables from the first pattern
                if let Some(first) = patterns.first() {
                    self.define_pattern_in_env(first, type_info);
                }
            }
            Pattern::Literal(_) | Pattern::Wildcard => {
                // No variables to define
            }
            Pattern::Struct { .. } => {
                // Struct patterns not yet implemented
            }
            Pattern::Variant { .. } => {
                // Variant patterns not yet implemented
            }
            Pattern::Ref { pattern: inner, .. } => {
                // ref pattern - define variables from the inner pattern
                // The type remains the same (ref doesn't change the type in our IR)
                self.define_pattern_in_env(inner, type_info);
            }
        }
    }

    /// Infer the type of an expression
    /// `expected` is an optional hint from the context (e.g., struct field type, variable annotation)
    fn infer_expr(&mut self, expr: &Expr) -> TypeInfo {
        self.infer_expr_with_expected(expr, None)
    }

    /// Infer expression type with an expected type hint for bidirectional inference
    fn infer_expr_with_expected(&mut self, expr: &Expr, expected: Option<&TypeInfo>) -> TypeInfo {
        match expr {
            Expr::Literal(lit) => match lit {
                Literal::String(_) => TypeInfo::Str,
                Literal::Int(_) => TypeInfo::I32,
                Literal::Float(_) => TypeInfo::F64,
                Literal::Bool(_) => TypeInfo::Bool,
                Literal::Null => TypeInfo::Null,
                Literal::Unit => TypeInfo::Unit,
            },

            Expr::Ident(ident) => {
                let ty = self.env
                    .lookup(&ident.name)
                    .cloned()
                    .unwrap_or(TypeInfo::Unknown);
                eprintln!("[TYPE INFER] Ident '{}' has type: {:?}", ident.name, ty);
                ty
            }

            Expr::Binary(binary) => {
                let left_type = self.infer_expr(&binary.left);
                let right_type = self.infer_expr(&binary.right);

                match binary.op {
                    // Comparison operators return bool
                    BinaryOp::Eq
                    | BinaryOp::NotEq
                    | BinaryOp::Lt
                    | BinaryOp::Gt
                    | BinaryOp::LtEq
                    | BinaryOp::GtEq => TypeInfo::Bool,

                    // Logical operators return bool
                    BinaryOp::And | BinaryOp::Or => TypeInfo::Bool,

                    // Arithmetic operators
                    BinaryOp::Add | BinaryOp::Sub | BinaryOp::Mul | BinaryOp::Div | BinaryOp::Mod => {
                        // If either is f64, result is f64
                        if matches!(left_type, TypeInfo::F64) || matches!(right_type, TypeInfo::F64)
                        {
                            TypeInfo::F64
                        } else if matches!(left_type, TypeInfo::Str) {
                            // String concatenation
                            TypeInfo::Str
                        } else {
                            TypeInfo::I32
                        }
                    }
                }
            }

            Expr::Unary(unary) => {
                let operand_type = self.infer_expr(&unary.operand);
                match unary.op {
                    UnaryOp::Not => TypeInfo::Bool,
                    UnaryOp::Neg => operand_type,
                    UnaryOp::Deref => operand_type.deref().cloned().unwrap_or(TypeInfo::Unknown),
                    UnaryOp::Ref => TypeInfo::Ref {
                        mutable: false,
                        inner: Box::new(operand_type),
                    },
                    UnaryOp::RefMut => TypeInfo::Ref {
                        mutable: true,
                        inner: Box::new(operand_type),
                    },
                }
            }

            Expr::Call(call) => {
                let callee_type = self.infer_expr(&call.callee);

                // Check arguments with expected parameter types for bidirectional inference
                match &callee_type {
                    TypeInfo::Function { params, ret } => {
                        for (i, arg) in call.args.iter().enumerate() {
                            let expected_param = params.get(i);
                            self.infer_expr_with_expected(arg, expected_param);
                        }
                        *ret.clone()
                    }
                    _ => {
                        // Method calls on known types
                        if let Expr::Member(member) = call.callee.as_ref() {
                            let obj_type = self.infer_expr(&member.object);
                            return self.infer_method_call(&obj_type, &member.property, &call.args);
                        }
                        // Unknown callee - just check arguments without expected types
                        for arg in &call.args {
                            self.infer_expr(arg);
                        }
                        TypeInfo::Unknown
                    }
                }
            }

            Expr::Member(member) => {
                // Check if this member expression has a narrowed type
                // e.g., after `if matches!(node.expr, CallExpression)`, `node.expr` is narrowed
                if let Expr::Ident(ident) = member.object.as_ref() {
                    let member_path = format!("{}.{}", ident.name, member.property);
                    if let Some(narrowed) = self.env.lookup(&member_path) {
                        eprintln!("[TYPE INFER] Member '{}' has narrowed type: {}", member_path, narrowed.display_name());
                        return narrowed.clone();
                    }
                }

                let obj_type = self.infer_expr(&member.object);
                self.get_field_type(&obj_type, &member.property)
            }

            Expr::Index(index) => {
                let obj_type = self.infer_expr(&index.object);
                self.infer_expr(&index.index);

                match obj_type {
                    TypeInfo::Vec(inner) => *inner,
                    TypeInfo::HashMap(_, v) => TypeInfo::Option(v),
                    _ => TypeInfo::Unknown,
                }
            }

            Expr::StructInit(init) => {
                // Check field types
                if let Some(fields) = self.env.get_struct_fields(&init.name) {
                    let fields = fields.clone();
                    for (field_name, value) in &init.fields {
                        // Pass expected field type for bidirectional inference
                        let field_expected = fields.get(field_name);
                        let value_type = self.infer_expr_with_expected(value, field_expected);
                        if let Some(expected_type) = field_expected {
                            if !value_type.is_assignable_to(expected_type) {
                                self.errors.push(SemanticError::new(
                                    "RS003",
                                    format!(
                                        "Field '{}' type mismatch: expected {}, found {}",
                                        field_name,
                                        expected_type.display_name(),
                                        value_type.display_name()
                                    ),
                                    init.span,
                                ));
                            }
                        }
                    }
                    TypeInfo::Struct {
                        name: init.name.clone(),
                        fields,
                    }
                } else {
                    // AST node type
                    for (_, value) in &init.fields {
                        self.infer_expr(value);
                    }
                    TypeInfo::AstNode(init.name.clone())
                }
            }

            Expr::VecInit(vec_init) => {
                if vec_init.elements.is_empty() {
                    // Use expected type if available (e.g., from struct field or variable annotation)
                    if let Some(TypeInfo::Vec(inner)) = expected {
                        TypeInfo::Vec(inner.clone())
                    } else {
                        TypeInfo::Vec(Box::new(TypeInfo::Unknown))
                    }
                } else {
                    // Infer from first element, but could also check against expected
                    let elem_type = self.infer_expr(&vec_init.elements[0]);
                    TypeInfo::Vec(Box::new(elem_type))
                }
            }

            Expr::If(if_expr) => {
                eprintln!("DEBUG Expr::If: inferring if expression");
                self.infer_expr(&if_expr.condition);

                // Infer type from the then branch
                self.env.push_scope();
                let then_type = self.infer_block(&if_expr.then_branch);
                eprintln!("DEBUG Expr::If: then_type = {:?}", then_type);
                self.env.pop_scope();

                // If there's an else branch, infer its type too
                if let Some(ref else_block) = if_expr.else_branch {
                    self.env.push_scope();
                    let else_type = self.infer_block(else_block);
                    eprintln!("DEBUG Expr::If: else_type = {:?}", else_type);
                    self.env.pop_scope();

                    // If both branches have the same type, use that
                    // If one is Never (!), use the other branch's type
                    // Otherwise, use Unit
                    let result_type = if then_type == else_type {
                        then_type
                    } else if matches!(then_type, TypeInfo::Never) {
                        else_type
                    } else if matches!(else_type, TypeInfo::Never) {
                        then_type
                    } else {
                        TypeInfo::Unit
                    };
                    eprintln!("DEBUG Expr::If: result_type = {:?}", result_type);
                    result_type
                } else {
                    // No else branch means the if-expression can be skipped entirely
                    // So it always evaluates to Unit
                    eprintln!("DEBUG Expr::If: no else branch, returning Unit");
                    TypeInfo::Unit
                }
            }

            Expr::Match(match_expr) => {
                self.infer_expr(&match_expr.scrutinee);

                // Infer first arm to establish expected type for other arms
                let first_arm_type = if !match_expr.arms.is_empty() {
                    self.env.push_scope();
                    let t = self.infer_expr_with_expected(&match_expr.arms[0].body, expected);
                    self.env.pop_scope();
                    t
                } else {
                    TypeInfo::Unknown
                };

                // Clone the type to avoid borrow issues
                let arm_expected_owned = if matches!(first_arm_type, TypeInfo::Unknown) {
                    expected.cloned()
                } else {
                    Some(first_arm_type.clone())
                };

                for arm in match_expr.arms.iter().skip(1) {
                    self.env.push_scope();
                    self.infer_expr_with_expected(&arm.body, arm_expected_owned.as_ref());
                    self.env.pop_scope();
                }

                first_arm_type
            }

            Expr::Closure(closure) => {
                self.env.push_scope();
                for param in &closure.params {
                    let var_type = self.env.fresh_var();
                    self.env.define(param.clone(), var_type);
                }
                let body_type = self.infer_expr(&closure.body);
                self.env.pop_scope();

                TypeInfo::Function {
                    params: vec![TypeInfo::Unknown; closure.params.len()],
                    ret: Box::new(body_type),
                }
            }

            Expr::Ref(ref_expr) => {
                let inner = self.infer_expr(&ref_expr.expr);
                TypeInfo::Ref {
                    mutable: ref_expr.mutable,
                    inner: Box::new(inner),
                }
            }

            Expr::Deref(deref_expr) => {
                let inner = self.infer_expr(&deref_expr.expr);
                inner.deref().cloned().unwrap_or(TypeInfo::Unknown)
            }

            Expr::Assign(assign) => {
                self.infer_expr(&assign.target);
                self.infer_expr(&assign.value);
                TypeInfo::Unit
            }

            Expr::CompoundAssign(compound) => {
                self.infer_expr(&compound.target);
                self.infer_expr(&compound.value);
                TypeInfo::Unit
            }

            Expr::Range(range) => {
                if let Some(ref start) = range.start {
                    self.infer_expr(start);
                }
                if let Some(ref end) = range.end {
                    self.infer_expr(end);
                }
                TypeInfo::Unknown // Range type
            }

            Expr::Paren(inner) => self.infer_expr(inner),

            Expr::Tuple(elements) => {
                let element_types: Vec<_> = elements.iter().map(|e| self.infer_expr(e)).collect();
                TypeInfo::Tuple(element_types)
            }

            Expr::Block(block) => {
                // Type of a block is the type of its last expression (if any)
                self.env.push_scope();
                let result_type = self.infer_block(block);
                self.env.pop_scope();
                result_type
            }

            Expr::Try(inner) => {
                // Type of expr? is the Ok variant of Result<T, E>
                let inner_type = self.infer_expr(inner);
                // If inner is Result<T, E>, type is T
                // For now, just return the inner type (simplified)
                inner_type
            }

            Expr::Matches(_) => {
                // matches! macro always returns bool
                TypeInfo::Bool
            }

            Expr::Return(value) => {
                // Return expression never produces a value (it diverges)
                if let Some(ref expr) = value {
                    self.infer_expr(expr);
                }
                TypeInfo::Unit  // Never type, but using Unit for now
            }

            Expr::Break => TypeInfo::Unit,  // Diverges
            Expr::Continue => TypeInfo::Unit,  // Diverges

            Expr::RegexCall(regex_call) => {
                use crate::parser::RegexMethod;
                // Return type depends on method
                match regex_call.method {
                    RegexMethod::Matches => TypeInfo::Bool,
                    RegexMethod::Find => TypeInfo::Option(Box::new(TypeInfo::Str)),
                    RegexMethod::FindAll => TypeInfo::Vec(Box::new(TypeInfo::Str)),
                    RegexMethod::Captures => {
                        // Return Option<Captures> where Captures has a get(i32) -> Str method
                        let mut fields = std::collections::HashMap::new();
                        fields.insert("get".to_string(), TypeInfo::Function {
                            params: vec![TypeInfo::I32],
                            ret: Box::new(TypeInfo::Str),
                        });
                        TypeInfo::Option(Box::new(TypeInfo::Struct {
                            name: "Captures".to_string(),
                            fields,
                        }))
                    }
                    RegexMethod::Replace | RegexMethod::ReplaceAll => TypeInfo::Str,
                }
            }

            Expr::CustomPropAccess(_access) => {
                // Custom properties always return Option<T>
                // TODO: Track property types and return Option<ActualType>
                // For now, return Option<Unknown>
                TypeInfo::Option(Box::new(TypeInfo::Unknown))
            }

            Expr::Path(_path) => {
                // Path expressions like std::ptr::null() are Rust stdlib paths
                // Type depends on the path - for now treat as Unknown
                TypeInfo::Unknown
            }
        }
    }

    /// Get the type of a field access
    fn get_field_type(&self, obj_type: &TypeInfo, field: &str) -> TypeInfo {
        match obj_type {
            TypeInfo::Struct { fields, .. } => {
                fields.get(field).cloned().unwrap_or(TypeInfo::Unknown)
            }
            TypeInfo::Ref { inner, .. } => self.get_field_type(inner, field),
            TypeInfo::NarrowedAstNode { current_type, .. } => {
                // Use the current narrowed type for field lookup
                self.get_field_type(&TypeInfo::AstNode(current_type.clone()), field)
            }
            TypeInfo::AstNode(node_type) => {
                // Try to get field type from AST schema
                use crate::codegen::type_context::{get_typed_field_mapping, map_reluxscript_to_swc};
                eprintln!("[GET FIELD TYPE] node_type='{}', field='{}'", node_type, field);

                // Map Babel/ReluxScript type name to SWC type name
                let (swc_type, _) = map_reluxscript_to_swc(node_type);
                eprintln!("[GET FIELD TYPE] Mapped '{}' -> '{}'", node_type, swc_type);

                if let Some(mapping) = get_typed_field_mapping(&swc_type, field) {
                    eprintln!("[GET FIELD TYPE] Found mapping: result_type_swc='{}'", mapping.result_type_swc);
                    // Parse the result_type_swc to create proper TypeInfo
                    let type_str = mapping.result_type_swc;
                    if type_str.starts_with("Option<") && type_str.ends_with(">") {
                        // Extract inner type from Option<T>
                        let inner_str = &type_str[7..type_str.len()-1]; // Remove "Option<" and ">"
                        let inner_type = if inner_str.starts_with("Box<") && inner_str.ends_with(">") {
                            // Option<Box<T>>
                            let inner_inner = &inner_str[4..inner_str.len()-1]; // Remove "Box<" and ">"
                            Box::new(TypeInfo::AstNode(inner_inner.to_string()))
                        } else {
                            Box::new(TypeInfo::AstNode(inner_str.to_string()))
                        };
                        return TypeInfo::Option(inner_type);
                    } else if type_str.starts_with("Vec<") && type_str.ends_with(">") {
                        let inner_str = &type_str[4..type_str.len()-1];
                        return TypeInfo::Vec(Box::new(TypeInfo::AstNode(inner_str.to_string())));
                    } else {
                        return TypeInfo::AstNode(type_str.to_string());
                    }
                }

                // Fallback for unmapped fields
                match field {
                    "name" | "value" | "operator" | "kind" => TypeInfo::Str,
                    "body" | "params" | "arguments" | "elements" | "properties" => {
                        TypeInfo::Vec(Box::new(TypeInfo::Unknown))
                    }
                    "id" | "init" | "left" | "right" | "object" | "property" | "callee"
                    | "argument" | "test" | "consequent" | "alternate" => TypeInfo::Unknown,
                    _ => TypeInfo::Unknown,
                }
            }
            _ => TypeInfo::Unknown,
        }
    }

    /// Infer return type of a method call
    fn infer_method_call(&self, obj_type: &TypeInfo, method: &str, _args: &[Expr]) -> TypeInfo {
        let result = match (obj_type, method) {
            // String methods
            (TypeInfo::Str, "clone") => TypeInfo::Str,
            (TypeInfo::Str, "len") => TypeInfo::I32,
            (TypeInfo::Str, "is_empty") => TypeInfo::Bool,
            (TypeInfo::Str, "starts_with" | "ends_with" | "contains") => TypeInfo::Bool,
            (TypeInfo::Str, "to_uppercase" | "to_lowercase") => TypeInfo::Str,
            (TypeInfo::Str, "chars") => TypeInfo::Vec(Box::new(TypeInfo::Str)),

            // Vec methods
            (TypeInfo::Vec(inner), "clone") => TypeInfo::Vec(inner.clone()),
            (TypeInfo::Vec(_), "len") => TypeInfo::I32,
            (TypeInfo::Vec(_), "is_empty") => TypeInfo::Bool,
            (TypeInfo::Vec(_), "push") => TypeInfo::Unit,
            (TypeInfo::Vec(inner), "pop") => TypeInfo::Option(inner.clone()),
            (TypeInfo::Vec(inner), "get") => TypeInfo::Option(Box::new(TypeInfo::Ref {
                mutable: false,
                inner: inner.clone(),
            })),
            (TypeInfo::Vec(inner), "iter") => TypeInfo::Vec(inner.clone()),
            (TypeInfo::Vec(_), "collect") => TypeInfo::Vec(Box::new(TypeInfo::Unknown)),

            // Option methods
            (TypeInfo::Option(inner), "unwrap") => (**inner).clone(),
            (TypeInfo::Option(inner), "unwrap_or") => (**inner).clone(),
            (TypeInfo::Option(inner), "unwrap_or_else") => (**inner).clone(),
            (TypeInfo::Option(_), "is_some" | "is_none") => TypeInfo::Bool,
            (TypeInfo::Option(inner), "map") => TypeInfo::Option(inner.clone()),
            (TypeInfo::Option(inner), "and_then") => TypeInfo::Option(inner.clone()),

            // HashMap methods
            (TypeInfo::HashMap(_, v), "get") => TypeInfo::Option(Box::new(TypeInfo::Ref {
                mutable: false,
                inner: v.clone(),
            })),
            (TypeInfo::HashMap(_, _), "insert") => TypeInfo::Unit,
            (TypeInfo::HashMap(_, _), "contains_key") => TypeInfo::Bool,
            (TypeInfo::HashMap(_, _), "len") => TypeInfo::I32,

            // Reference dereferencing for method calls
            (TypeInfo::Ref { inner, .. }, method) => self.infer_method_call(inner, method, _args),

            // AST node methods
            (TypeInfo::AstNode(_), "clone") => obj_type.clone(),
            (TypeInfo::AstNode(_), "visit_children") => TypeInfo::Unit,

            _ => TypeInfo::Unknown,
        };
        eprintln!("[METHOD CALL] {}.{}() -> {:?}", obj_type.display_name(), method, result);
        result
    }

    // ========================================================================
    // Type Narrowing Support
    // ========================================================================

    /// Check if a block ends with an early return statement
    fn has_early_return(&self, block: &Block) -> bool {
        if let Some(last_stmt) = block.stmts.last() {
            matches!(last_stmt, Stmt::Return(_))
        } else {
            false
        }
    }

    /// Register bindings from a pattern with their appropriate types
    /// For example, in `if let Some(x) = opt`, register `x` with the inner type of Option
    fn register_pattern_bindings(&mut self, pattern: &Pattern, scrutinee_type: &TypeInfo) {
        eprintln!("[PATTERN BINDING DEBUG] pattern={:?}, scrutinee_type={:?}", pattern, scrutinee_type);
        match pattern {
            Pattern::Ident(name) => {
                // Simple binding - use the scrutinee type
                eprintln!("[PATTERN BINDING] Registering '{}' with type {:?}", name, scrutinee_type);
                self.env.define(name.clone(), scrutinee_type.clone());
            }
            Pattern::Variant { name, inner } => {
                // Variant pattern like Some(x) or JSXElement
                if name == "Some" {
                    // Option unwrapping - extract inner type
                    if let TypeInfo::Option(inner_type) = scrutinee_type {
                        if let Some(inner_pattern) = inner {
                            eprintln!("[PATTERN BINDING] Option::Some pattern, inner type: {:?}", inner_type);
                            self.register_pattern_bindings(inner_pattern, inner_type);
                        }
                    } else if let TypeInfo::Ref { inner: ref_inner, .. } = scrutinee_type {
                        // Handle &Option<T> case
                        if let TypeInfo::Option(inner_type) = ref_inner.as_ref() {
                            if let Some(inner_pattern) = inner {
                                eprintln!("[PATTERN BINDING] &Option::Some pattern, inner type: {:?}", inner_type);
                                // Create a reference to the inner type
                                let inner_ref_type = TypeInfo::Ref {
                                    mutable: false,
                                    inner: inner_type.clone(),
                                };
                                self.register_pattern_bindings(inner_pattern, &inner_ref_type);
                            }
                        }
                    }
                } else if let Some(inner_pattern) = inner {
                    // Other variant patterns - the binding gets the narrowed type
                    if let Some(narrowed_type) = self.pattern_to_concrete_type(pattern) {
                        self.register_pattern_bindings(inner_pattern, &narrowed_type);
                    }
                }
            }
            _ => {
                // Other patterns - skip for now
            }
        }
    }

    /// Extract a simple scrutinee variable name from an expression
    /// Returns Some(name) for identifiers, references, or member accesses
    fn extract_simple_scrutinee(&self, expr: &Expr) -> Option<String> {
        match expr {
            Expr::Ident(ident) => {
                eprintln!("DEBUG narrowing: extracted scrutinee '{}'", ident.name);
                Some(ident.name.clone())
            }
            Expr::Ref(ref_expr) => {
                // Handle &x and &mut x
                if let Expr::Ident(ident) = ref_expr.expr.as_ref() {
                    eprintln!("DEBUG narrowing: extracted scrutinee '{}' from reference", ident.name);
                    Some(ident.name.clone())
                } else {
                    eprintln!("DEBUG narrowing: reference to non-identifier ({:?}), cannot narrow", ref_expr.expr);
                    None
                }
            }
            Expr::Member(mem) => {
                // Handle member access like node.expr
                // Build the full path: object.property
                if let Expr::Ident(ident) = mem.object.as_ref() {
                    let path = format!("{}.{}", ident.name, mem.property);
                    eprintln!("DEBUG narrowing: extracted scrutinee '{}' from member access", path);
                    Some(path)
                } else {
                    eprintln!("DEBUG narrowing: member access with non-ident object, cannot narrow");
                    None
                }
            }
            _ => {
                eprintln!("DEBUG narrowing: complex expression (variant {:?}), cannot narrow", std::mem::discriminant(expr));
                None
            }
        }
    }

    /// Map a pattern variant name to its concrete AST type
    /// E.g., "ArrayPattern" -> TypeInfo::AstNode("ArrayPat")
    fn pattern_to_concrete_type(&self, pattern: &Pattern) -> Option<TypeInfo> {
        // Helper function to map pattern names to SWC types
        let map_pattern_name = |name: &str| -> Option<String> {
            let concrete_type = match name {
                // Expression patterns
                "ArrayExpression" => "ArrayLit",
                "ArrowFunctionExpression" => "ArrowExpr",
                "AssignmentExpression" => "AssignExpr",
                "BinaryExpression" => "BinExpr",
                "CallExpression" => "CallExpr",
                "ConditionalExpression" => "CondExpr",
                "FunctionExpression" => "FnExpr",
                "Identifier" => "Ident",
                "MemberExpression" => "MemberExpr",
                "NewExpression" => "NewExpr",
                "ObjectExpression" => "ObjectLit",
                "SequenceExpression" => "SeqExpr",
                "StringLiteral" => "Str",
                "NumericLiteral" => "Number",
                "BooleanLiteral" => "Bool",
                "NullLiteral" => "Null",
                "RegExpLiteral" => "Regex",
                "TemplateLiteral" => "Tpl",
                "UnaryExpression" => "UnaryExpr",
                "UpdateExpression" => "UpdateExpr",

                // Pattern patterns
                "ArrayPattern" => "ArrayPat",
                "ObjectPattern" => "ObjectPat",
                "ObjectPatternProperty" => "KeyValuePatProp",
                "RestElement" => "RestPat",
                "AssignmentPattern" => "AssignPat",

                // Statement patterns
                "BlockStatement" => "BlockStmt",
                "ExpressionStatement" => "ExprStmt",
                "ReturnStatement" => "ReturnStmt",
                "IfStatement" => "IfStmt",
                "ForStatement" => "ForStmt",
                "WhileStatement" => "WhileStmt",
                "BreakStatement" => "BreakStmt",
                "ContinueStatement" => "ContinueStmt",

                // Other common types
                "VariableDeclaration" => "VarDecl",
                "FunctionDeclaration" => "FnDecl",
                "ClassDeclaration" => "ClassDecl",

                // Not a known pattern - check if it's a user-defined type
                _ => return None,
            };
            Some(concrete_type.to_string())
        };

        match pattern {
            Pattern::Variant { name, .. } | Pattern::Ident(name) => {
                // Both Variant and Ident patterns can be AST pattern names
                if let Some(concrete_type) = map_pattern_name(name) {
                    eprintln!("DEBUG narrowing: pattern '{}' narrows to AstNode('{}')", name, concrete_type);
                    Some(TypeInfo::AstNode(concrete_type))
                } else {
                    // Check if it's a user-defined type in the environment
                    if let Some(ty) = self.env.lookup(name) {
                        eprintln!("DEBUG narrowing: pattern '{}' found as user type {:?}", name, ty);
                        Some(ty.clone())
                    } else {
                        eprintln!("DEBUG narrowing: pattern '{}' not recognized", name);
                        None
                    }
                }
            }
            _ => {
                eprintln!("DEBUG narrowing: pattern type not supported for narrowing");
                None
            }
        }
    }
}
