//! Node type mappings between ReluxScript, Babel, and SWC

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Mapping for a single AST node type
#[derive(Debug, Clone)]
pub struct NodeMapping {
    /// ReluxScript unified name
    pub reluxscript: &'static str,
    /// Babel/ESTree type name
    pub babel: &'static str,
    /// SWC Rust type name
    pub swc: &'static str,
    /// SWC enum variant (if wrapped in enum like Stmt, Expr, Decl)
    pub swc_enum: Option<&'static str>,
    /// Babel type checker function (e.g., "isIdentifier")
    pub babel_checker: &'static str,
    /// SWC pattern match (e.g., "Expr::Ident")
    pub swc_pattern: &'static str,
    /// Visitor method name in ReluxScript
    pub visitor_method: &'static str,
    /// SWC visitor method name
    pub swc_visitor: &'static str,
}

/// All node type mappings
pub static NODE_MAPPINGS: Lazy<Vec<NodeMapping>> = Lazy::new(|| vec![
    // === Declarations ===
    NodeMapping {
        reluxscript: "FunctionDeclaration",
        babel: "FunctionDeclaration",
        swc: "FnDecl",
        swc_enum: Some("Decl::Fn"),
        babel_checker: "isFunctionDeclaration",
        swc_pattern: "Decl::Fn(fn_decl)",
        visitor_method: "visit_function_declaration",
        swc_visitor: "visit_mut_fn_decl",
    },
    NodeMapping {
        reluxscript: "VariableDeclaration",
        babel: "VariableDeclaration",
        swc: "VarDecl",
        swc_enum: Some("Decl::Var"),
        babel_checker: "isVariableDeclaration",
        swc_pattern: "Decl::Var(var_decl)",
        visitor_method: "visit_variable_declaration",
        swc_visitor: "visit_mut_var_decl",
    },
    NodeMapping {
        reluxscript: "VariableDeclarator",
        babel: "VariableDeclarator",
        swc: "VarDeclarator",
        swc_enum: None,
        babel_checker: "isVariableDeclarator",
        swc_pattern: "VarDeclarator",
        visitor_method: "visit_variable_declarator",
        swc_visitor: "visit_mut_var_declarator",
    },
    NodeMapping {
        reluxscript: "ClassDeclaration",
        babel: "ClassDeclaration",
        swc: "ClassDecl",
        swc_enum: Some("Decl::Class"),
        babel_checker: "isClassDeclaration",
        swc_pattern: "Decl::Class(class_decl)",
        visitor_method: "visit_class_declaration",
        swc_visitor: "visit_mut_class_decl",
    },

    // === Statements ===
    NodeMapping {
        reluxscript: "ExpressionStatement",
        babel: "ExpressionStatement",
        swc: "ExprStmt",
        swc_enum: Some("Stmt::Expr"),
        babel_checker: "isExpressionStatement",
        swc_pattern: "Stmt::Expr(expr_stmt)",
        visitor_method: "visit_expression_statement",
        swc_visitor: "visit_mut_expr_stmt",
    },
    NodeMapping {
        reluxscript: "BlockStatement",
        babel: "BlockStatement",
        swc: "BlockStmt",
        swc_enum: Some("Stmt::Block"),
        babel_checker: "isBlockStatement",
        swc_pattern: "Stmt::Block(block_stmt)",
        visitor_method: "visit_block_statement",
        swc_visitor: "visit_mut_block_stmt",
    },
    NodeMapping {
        reluxscript: "ReturnStatement",
        babel: "ReturnStatement",
        swc: "ReturnStmt",
        swc_enum: Some("Stmt::Return"),
        babel_checker: "isReturnStatement",
        swc_pattern: "Stmt::Return(return_stmt)",
        visitor_method: "visit_return_statement",
        swc_visitor: "visit_mut_return_stmt",
    },
    NodeMapping {
        reluxscript: "IfStatement",
        babel: "IfStatement",
        swc: "IfStmt",
        swc_enum: Some("Stmt::If"),
        babel_checker: "isIfStatement",
        swc_pattern: "Stmt::If(if_stmt)",
        visitor_method: "visit_if_statement",
        swc_visitor: "visit_mut_if_stmt",
    },
    NodeMapping {
        reluxscript: "ForStatement",
        babel: "ForStatement",
        swc: "ForStmt",
        swc_enum: Some("Stmt::For"),
        babel_checker: "isForStatement",
        swc_pattern: "Stmt::For(for_stmt)",
        visitor_method: "visit_for_statement",
        swc_visitor: "visit_mut_for_stmt",
    },
    NodeMapping {
        reluxscript: "ForInStatement",
        babel: "ForInStatement",
        swc: "ForInStmt",
        swc_enum: Some("Stmt::ForIn"),
        babel_checker: "isForInStatement",
        swc_pattern: "Stmt::ForIn(for_in_stmt)",
        visitor_method: "visit_for_in_statement",
        swc_visitor: "visit_mut_for_in_stmt",
    },
    NodeMapping {
        reluxscript: "ForOfStatement",
        babel: "ForOfStatement",
        swc: "ForOfStmt",
        swc_enum: Some("Stmt::ForOf"),
        babel_checker: "isForOfStatement",
        swc_pattern: "Stmt::ForOf(for_of_stmt)",
        visitor_method: "visit_for_of_statement",
        swc_visitor: "visit_mut_for_of_stmt",
    },
    NodeMapping {
        reluxscript: "WhileStatement",
        babel: "WhileStatement",
        swc: "WhileStmt",
        swc_enum: Some("Stmt::While"),
        babel_checker: "isWhileStatement",
        swc_pattern: "Stmt::While(while_stmt)",
        visitor_method: "visit_while_statement",
        swc_visitor: "visit_mut_while_stmt",
    },
    NodeMapping {
        reluxscript: "SwitchStatement",
        babel: "SwitchStatement",
        swc: "SwitchStmt",
        swc_enum: Some("Stmt::Switch"),
        babel_checker: "isSwitchStatement",
        swc_pattern: "Stmt::Switch(switch_stmt)",
        visitor_method: "visit_switch_statement",
        swc_visitor: "visit_mut_switch_stmt",
    },
    NodeMapping {
        reluxscript: "TryStatement",
        babel: "TryStatement",
        swc: "TryStmt",
        swc_enum: Some("Stmt::Try"),
        babel_checker: "isTryStatement",
        swc_pattern: "Stmt::Try(try_stmt)",
        visitor_method: "visit_try_statement",
        swc_visitor: "visit_mut_try_stmt",
    },
    NodeMapping {
        reluxscript: "ThrowStatement",
        babel: "ThrowStatement",
        swc: "ThrowStmt",
        swc_enum: Some("Stmt::Throw"),
        babel_checker: "isThrowStatement",
        swc_pattern: "Stmt::Throw(throw_stmt)",
        visitor_method: "visit_throw_statement",
        swc_visitor: "visit_mut_throw_stmt",
    },

    // === Expressions ===
    NodeMapping {
        reluxscript: "Expression",
        babel: "Expression",
        swc: "Expr",
        swc_enum: None,
        babel_checker: "isExpression",
        swc_pattern: "Expr",
        visitor_method: "visit_expression",
        swc_visitor: "visit_mut_expr",
    },
    NodeMapping {
        reluxscript: "Identifier",
        babel: "Identifier",
        swc: "Ident",
        swc_enum: Some("Expr::Ident"),
        babel_checker: "isIdentifier",
        swc_pattern: "Expr::Ident(ident)",
        visitor_method: "visit_identifier",
        swc_visitor: "visit_mut_ident",
    },
    NodeMapping {
        reluxscript: "CallExpression",
        babel: "CallExpression",
        swc: "CallExpr",
        swc_enum: Some("Expr::Call"),
        babel_checker: "isCallExpression",
        swc_pattern: "Expr::Call(call_expr)",
        visitor_method: "visit_call_expression",
        swc_visitor: "visit_mut_call_expr",
    },
    NodeMapping {
        reluxscript: "MemberExpression",
        babel: "MemberExpression",
        swc: "MemberExpr",
        swc_enum: Some("Expr::Member"),
        babel_checker: "isMemberExpression",
        swc_pattern: "Expr::Member(member_expr)",
        visitor_method: "visit_member_expression",
        swc_visitor: "visit_mut_member_expr",
    },
    NodeMapping {
        reluxscript: "BinaryExpression",
        babel: "BinaryExpression",
        swc: "BinExpr",
        swc_enum: Some("Expr::Bin"),
        babel_checker: "isBinaryExpression",
        swc_pattern: "Expr::Bin(bin_expr)",
        visitor_method: "visit_binary_expression",
        swc_visitor: "visit_mut_bin_expr",
    },
    NodeMapping {
        reluxscript: "UnaryExpression",
        babel: "UnaryExpression",
        swc: "UnaryExpr",
        swc_enum: Some("Expr::Unary"),
        babel_checker: "isUnaryExpression",
        swc_pattern: "Expr::Unary(unary_expr)",
        visitor_method: "visit_unary_expression",
        swc_visitor: "visit_mut_unary_expr",
    },
    NodeMapping {
        reluxscript: "AssignmentExpression",
        babel: "AssignmentExpression",
        swc: "AssignExpr",
        swc_enum: Some("Expr::Assign"),
        babel_checker: "isAssignmentExpression",
        swc_pattern: "Expr::Assign(assign_expr)",
        visitor_method: "visit_assignment_expression",
        swc_visitor: "visit_mut_assign_expr",
    },
    NodeMapping {
        reluxscript: "ConditionalExpression",
        babel: "ConditionalExpression",
        swc: "CondExpr",
        swc_enum: Some("Expr::Cond"),
        babel_checker: "isConditionalExpression",
        swc_pattern: "Expr::Cond(cond_expr)",
        visitor_method: "visit_conditional_expression",
        swc_visitor: "visit_mut_cond_expr",
    },
    NodeMapping {
        reluxscript: "LogicalExpression",
        babel: "LogicalExpression",
        swc: "BinExpr",
        swc_enum: Some("Expr::Bin"),
        babel_checker: "isLogicalExpression",
        swc_pattern: "Expr::Bin(bin_expr)",
        visitor_method: "visit_logical_expression",
        swc_visitor: "visit_mut_bin_expr",
    },
    NodeMapping {
        reluxscript: "ArrayExpression",
        babel: "ArrayExpression",
        swc: "ArrayLit",
        swc_enum: Some("Expr::Array"),
        babel_checker: "isArrayExpression",
        swc_pattern: "Expr::Array(array_lit)",
        visitor_method: "visit_array_expression",
        swc_visitor: "visit_mut_array_lit",
    },
    NodeMapping {
        reluxscript: "ObjectExpression",
        babel: "ObjectExpression",
        swc: "ObjectLit",
        swc_enum: Some("Expr::Object"),
        babel_checker: "isObjectExpression",
        swc_pattern: "Expr::Object(object_lit)",
        visitor_method: "visit_object_expression",
        swc_visitor: "visit_mut_object_lit",
    },
    NodeMapping {
        reluxscript: "ArrowFunctionExpression",
        babel: "ArrowFunctionExpression",
        swc: "ArrowExpr",
        swc_enum: Some("Expr::Arrow"),
        babel_checker: "isArrowFunctionExpression",
        swc_pattern: "Expr::Arrow(arrow_expr)",
        visitor_method: "visit_arrow_function_expression",
        swc_visitor: "visit_mut_arrow_expr",
    },
    NodeMapping {
        reluxscript: "FunctionExpression",
        babel: "FunctionExpression",
        swc: "FnExpr",
        swc_enum: Some("Expr::Fn"),
        babel_checker: "isFunctionExpression",
        swc_pattern: "Expr::Fn(fn_expr)",
        visitor_method: "visit_function_expression",
        swc_visitor: "visit_mut_fn_expr",
    },
    NodeMapping {
        reluxscript: "NewExpression",
        babel: "NewExpression",
        swc: "NewExpr",
        swc_enum: Some("Expr::New"),
        babel_checker: "isNewExpression",
        swc_pattern: "Expr::New(new_expr)",
        visitor_method: "visit_new_expression",
        swc_visitor: "visit_mut_new_expr",
    },
    NodeMapping {
        reluxscript: "SequenceExpression",
        babel: "SequenceExpression",
        swc: "SeqExpr",
        swc_enum: Some("Expr::Seq"),
        babel_checker: "isSequenceExpression",
        swc_pattern: "Expr::Seq(seq_expr)",
        visitor_method: "visit_sequence_expression",
        swc_visitor: "visit_mut_seq_expr",
    },
    NodeMapping {
        reluxscript: "ThisExpression",
        babel: "ThisExpression",
        swc: "ThisExpr",
        swc_enum: Some("Expr::This"),
        babel_checker: "isThisExpression",
        swc_pattern: "Expr::This(this_expr)",
        visitor_method: "visit_this_expression",
        swc_visitor: "visit_mut_this_expr",
    },
    NodeMapping {
        reluxscript: "AwaitExpression",
        babel: "AwaitExpression",
        swc: "AwaitExpr",
        swc_enum: Some("Expr::Await"),
        babel_checker: "isAwaitExpression",
        swc_pattern: "Expr::Await(await_expr)",
        visitor_method: "visit_await_expression",
        swc_visitor: "visit_mut_await_expr",
    },
    NodeMapping {
        reluxscript: "YieldExpression",
        babel: "YieldExpression",
        swc: "YieldExpr",
        swc_enum: Some("Expr::Yield"),
        babel_checker: "isYieldExpression",
        swc_pattern: "Expr::Yield(yield_expr)",
        visitor_method: "visit_yield_expression",
        swc_visitor: "visit_mut_yield_expr",
    },

    // === Literals ===
    NodeMapping {
        reluxscript: "StringLiteral",
        babel: "StringLiteral",
        swc: "Str",
        swc_enum: Some("Lit::Str"),
        babel_checker: "isStringLiteral",
        swc_pattern: "Lit::Str(str_lit)",
        visitor_method: "visit_string_literal",
        swc_visitor: "visit_mut_str",
    },
    NodeMapping {
        reluxscript: "NumericLiteral",
        babel: "NumericLiteral",
        swc: "Number",
        swc_enum: Some("Lit::Num"),
        babel_checker: "isNumericLiteral",
        swc_pattern: "Lit::Num(num_lit)",
        visitor_method: "visit_numeric_literal",
        swc_visitor: "visit_mut_number",
    },
    NodeMapping {
        reluxscript: "BooleanLiteral",
        babel: "BooleanLiteral",
        swc: "Bool",
        swc_enum: Some("Lit::Bool"),
        babel_checker: "isBooleanLiteral",
        swc_pattern: "Lit::Bool(bool_lit)",
        visitor_method: "visit_boolean_literal",
        swc_visitor: "visit_mut_bool",
    },
    NodeMapping {
        reluxscript: "NullLiteral",
        babel: "NullLiteral",
        swc: "Null",
        swc_enum: Some("Lit::Null"),
        babel_checker: "isNullLiteral",
        swc_pattern: "Lit::Null(null_lit)",
        visitor_method: "visit_null_literal",
        swc_visitor: "visit_mut_null",
    },
    NodeMapping {
        reluxscript: "RegExpLiteral",
        babel: "RegExpLiteral",
        swc: "Regex",
        swc_enum: Some("Lit::Regex"),
        babel_checker: "isRegExpLiteral",
        swc_pattern: "Lit::Regex(regex_lit)",
        visitor_method: "visit_regexp_literal",
        swc_visitor: "visit_mut_regex",
    },
    NodeMapping {
        reluxscript: "TemplateLiteral",
        babel: "TemplateLiteral",
        swc: "Tpl",
        swc_enum: Some("Expr::Tpl"),
        babel_checker: "isTemplateLiteral",
        swc_pattern: "Expr::Tpl(tpl)",
        visitor_method: "visit_template_literal",
        swc_visitor: "visit_mut_tpl",
    },
    NodeMapping {
        reluxscript: "TemplateElement",
        babel: "TemplateElement",
        swc: "TplElement",
        swc_enum: None,
        babel_checker: "isTemplateElement",
        swc_pattern: "TplElement",
        visitor_method: "visit_template_element",
        swc_visitor: "visit_mut_tpl_element",
    },

    // === JSX ===
    NodeMapping {
        reluxscript: "JSXElement",
        babel: "JSXElement",
        swc: "JSXElement",
        swc_enum: Some("Expr::JSXElement"),
        babel_checker: "isJSXElement",
        swc_pattern: "Expr::JSXElement(jsx_element)",
        visitor_method: "visit_jsx_element",
        swc_visitor: "visit_mut_jsx_element",
    },
    NodeMapping {
        reluxscript: "JSXFragment",
        babel: "JSXFragment",
        swc: "JSXFragment",
        swc_enum: Some("Expr::JSXFragment"),
        babel_checker: "isJSXFragment",
        swc_pattern: "Expr::JSXFragment(jsx_fragment)",
        visitor_method: "visit_jsx_fragment",
        swc_visitor: "visit_mut_jsx_fragment",
    },
    NodeMapping {
        reluxscript: "JSXAttribute",
        babel: "JSXAttribute",
        swc: "JSXAttr",
        swc_enum: None,
        babel_checker: "isJSXAttribute",
        swc_pattern: "JSXAttrOrSpread::JSXAttr(jsx_attr)",
        visitor_method: "visit_jsx_attribute",
        swc_visitor: "visit_mut_jsx_attr",
    },
    NodeMapping {
        reluxscript: "JSXExpressionContainer",
        babel: "JSXExpressionContainer",
        swc: "JSXExprContainer",
        swc_enum: None,
        babel_checker: "isJSXExpressionContainer",
        swc_pattern: "JSXElementChild::JSXExprContainer(container)",
        visitor_method: "visit_jsx_expression_container",
        swc_visitor: "visit_mut_jsx_expr_container",
    },
    NodeMapping {
        reluxscript: "JSXText",
        babel: "JSXText",
        swc: "JSXText",
        swc_enum: None,
        babel_checker: "isJSXText",
        swc_pattern: "JSXElementChild::JSXText(jsx_text)",
        visitor_method: "visit_jsx_text",
        swc_visitor: "visit_mut_jsx_text",
    },
    NodeMapping {
        reluxscript: "JSXOpeningElement",
        babel: "JSXOpeningElement",
        swc: "JSXOpeningElement",
        swc_enum: None,
        babel_checker: "isJSXOpeningElement",
        swc_pattern: "JSXOpeningElement",
        visitor_method: "visit_jsx_opening_element",
        swc_visitor: "visit_mut_jsx_opening_element",
    },

    // === Module ===
    NodeMapping {
        reluxscript: "ImportDeclaration",
        babel: "ImportDeclaration",
        swc: "ImportDecl",
        swc_enum: Some("ModuleDecl::Import"),
        babel_checker: "isImportDeclaration",
        swc_pattern: "ModuleDecl::Import(import_decl)",
        visitor_method: "visit_import_declaration",
        swc_visitor: "visit_mut_import_decl",
    },
    NodeMapping {
        reluxscript: "ExportNamedDeclaration",
        babel: "ExportNamedDeclaration",
        swc: "ExportDecl",
        swc_enum: Some("ModuleDecl::ExportDecl"),
        babel_checker: "isExportNamedDeclaration",
        swc_pattern: "ModuleDecl::ExportDecl(export_decl)",
        visitor_method: "visit_export_named_declaration",
        swc_visitor: "visit_mut_export_decl",
    },
    NodeMapping {
        reluxscript: "ExportDefaultDeclaration",
        babel: "ExportDefaultDeclaration",
        swc: "ExportDefaultDecl",
        swc_enum: Some("ModuleDecl::ExportDefaultDecl"),
        babel_checker: "isExportDefaultDeclaration",
        swc_pattern: "ModuleDecl::ExportDefaultDecl(export_default_decl)",
        visitor_method: "visit_export_default_declaration",
        swc_visitor: "visit_mut_export_default_decl",
    },

    // === Program ===
    NodeMapping {
        reluxscript: "Program",
        babel: "Program",
        swc: "Program",
        swc_enum: None,
        babel_checker: "isProgram",
        swc_pattern: "Program",
        visitor_method: "visit_program",
        swc_visitor: "visit_mut_program",
    },

    // === TypeScript Declarations ===
    NodeMapping {
        reluxscript: "TSInterfaceDeclaration",
        babel: "TSInterfaceDeclaration",
        swc: "TsInterfaceDecl",
        swc_enum: Some("Decl::TsInterface"),
        babel_checker: "isTSInterfaceDeclaration",
        swc_pattern: "Decl::TsInterface(ts_interface_decl)",
        visitor_method: "visit_ts_interface_declaration",
        swc_visitor: "visit_mut_ts_interface_decl",
    },
    NodeMapping {
        reluxscript: "TSTypeAliasDeclaration",
        babel: "TSTypeAliasDeclaration",
        swc: "TsTypeAliasDecl",
        swc_enum: Some("Decl::TsTypeAlias"),
        babel_checker: "isTSTypeAliasDeclaration",
        swc_pattern: "Decl::TsTypeAlias(ts_type_alias_decl)",
        visitor_method: "visit_ts_type_alias_declaration",
        swc_visitor: "visit_mut_ts_type_alias_decl",
    },
    NodeMapping {
        reluxscript: "TSEnumDeclaration",
        babel: "TSEnumDeclaration",
        swc: "TsEnumDecl",
        swc_enum: Some("Decl::TsEnum"),
        babel_checker: "isTSEnumDeclaration",
        swc_pattern: "Decl::TsEnum(ts_enum_decl)",
        visitor_method: "visit_ts_enum_declaration",
        swc_visitor: "visit_mut_ts_enum_decl",
    },

    // === TypeScript Interface Members ===
    NodeMapping {
        reluxscript: "TSPropertySignature",
        babel: "TSPropertySignature",
        swc: "TsPropertySignature",
        swc_enum: Some("TsTypeElement::TsPropertySignature"),
        babel_checker: "isTSPropertySignature",
        swc_pattern: "TsTypeElement::TsPropertySignature(ts_prop_sig)",
        visitor_method: "visit_ts_property_signature",
        swc_visitor: "visit_mut_ts_property_signature",
    },
    NodeMapping {
        reluxscript: "TSMethodSignature",
        babel: "TSMethodSignature",
        swc: "TsMethodSignature",
        swc_enum: Some("TsTypeElement::TsMethodSignature"),
        babel_checker: "isTSMethodSignature",
        swc_pattern: "TsTypeElement::TsMethodSignature(ts_method_sig)",
        visitor_method: "visit_ts_method_signature",
        swc_visitor: "visit_mut_ts_method_signature",
    },
    NodeMapping {
        reluxscript: "TSIndexSignature",
        babel: "TSIndexSignature",
        swc: "TsIndexSignature",
        swc_enum: Some("TsTypeElement::TsIndexSignature"),
        babel_checker: "isTSIndexSignature",
        swc_pattern: "TsTypeElement::TsIndexSignature(ts_index_sig)",
        visitor_method: "visit_ts_index_signature",
        swc_visitor: "visit_mut_ts_index_signature",
    },

    // === TypeScript Types ===
    NodeMapping {
        reluxscript: "TSTypeReference",
        babel: "TSTypeReference",
        swc: "TsTypeRef",
        swc_enum: Some("TsType::TsTypeRef"),
        babel_checker: "isTSTypeReference",
        swc_pattern: "TsType::TsTypeRef(ts_type_ref)",
        visitor_method: "visit_ts_type_reference",
        swc_visitor: "visit_mut_ts_type_ref",
    },
    NodeMapping {
        reluxscript: "TSTypeAnnotation",
        babel: "TSTypeAnnotation",
        swc: "TsTypeAnn",
        swc_enum: None,
        babel_checker: "isTSTypeAnnotation",
        swc_pattern: "TsTypeAnn",
        visitor_method: "visit_ts_type_annotation",
        swc_visitor: "visit_mut_ts_type_ann",
    },
    NodeMapping {
        reluxscript: "TSArrayType",
        babel: "TSArrayType",
        swc: "TsArrayType",
        swc_enum: Some("TsType::TsArrayType"),
        babel_checker: "isTSArrayType",
        swc_pattern: "TsType::TsArrayType(ts_array_type)",
        visitor_method: "visit_ts_array_type",
        swc_visitor: "visit_mut_ts_array_type",
    },
    NodeMapping {
        reluxscript: "TSUnionType",
        babel: "TSUnionType",
        swc: "TsUnionType",
        swc_enum: Some("TsType::TsUnionOrIntersectionType"),
        babel_checker: "isTSUnionType",
        swc_pattern: "TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(ts_union))",
        visitor_method: "visit_ts_union_type",
        swc_visitor: "visit_mut_ts_union_type",
    },

    // === TypeScript Type Keywords ===
    NodeMapping {
        reluxscript: "TSStringKeyword",
        babel: "TSStringKeyword",
        swc: "TsKeywordType",
        swc_enum: Some("TsType::TsKeywordType"),
        babel_checker: "isTSStringKeyword",
        swc_pattern: "TsType::TsKeywordType(TsKeywordType { kind: TsKeywordTypeKind::TsStringKeyword, .. })",
        visitor_method: "visit_ts_string_keyword",
        swc_visitor: "visit_mut_ts_keyword_type",
    },
    NodeMapping {
        reluxscript: "TSNumberKeyword",
        babel: "TSNumberKeyword",
        swc: "TsKeywordType",
        swc_enum: Some("TsType::TsKeywordType"),
        babel_checker: "isTSNumberKeyword",
        swc_pattern: "TsType::TsKeywordType(TsKeywordType { kind: TsKeywordTypeKind::TsNumberKeyword, .. })",
        visitor_method: "visit_ts_number_keyword",
        swc_visitor: "visit_mut_ts_keyword_type",
    },
    NodeMapping {
        reluxscript: "TSBooleanKeyword",
        babel: "TSBooleanKeyword",
        swc: "TsKeywordType",
        swc_enum: Some("TsType::TsKeywordType"),
        babel_checker: "isTSBooleanKeyword",
        swc_pattern: "TsType::TsKeywordType(TsKeywordType { kind: TsKeywordTypeKind::TsBooleanKeyword, .. })",
        visitor_method: "visit_ts_boolean_keyword",
        swc_visitor: "visit_mut_ts_keyword_type",
    },
    NodeMapping {
        reluxscript: "TSAnyKeyword",
        babel: "TSAnyKeyword",
        swc: "TsKeywordType",
        swc_enum: Some("TsType::TsKeywordType"),
        babel_checker: "isTSAnyKeyword",
        swc_pattern: "TsType::TsKeywordType(TsKeywordType { kind: TsKeywordTypeKind::TsAnyKeyword, .. })",
        visitor_method: "visit_ts_any_keyword",
        swc_visitor: "visit_mut_ts_keyword_type",
    },
    NodeMapping {
        reluxscript: "TSVoidKeyword",
        babel: "TSVoidKeyword",
        swc: "TsKeywordType",
        swc_enum: Some("TsType::TsKeywordType"),
        babel_checker: "isTSVoidKeyword",
        swc_pattern: "TsType::TsKeywordType(TsKeywordType { kind: TsKeywordTypeKind::TsVoidKeyword, .. })",
        visitor_method: "visit_ts_void_keyword",
        swc_visitor: "visit_mut_ts_keyword_type",
    },

    // === TypeScript Type Parameters ===
    NodeMapping {
        reluxscript: "TSTypeParameterInstantiation",
        babel: "TSTypeParameterInstantiation",
        swc: "TsTypeParamInstantiation",
        swc_enum: None,
        babel_checker: "isTSTypeParameterInstantiation",
        swc_pattern: "TsTypeParamInstantiation",
        visitor_method: "visit_ts_type_parameter_instantiation",
        swc_visitor: "visit_mut_ts_type_param_instantiation",
    },
    NodeMapping {
        reluxscript: "TSTypeParameterDeclaration",
        babel: "TSTypeParameterDeclaration",
        swc: "TsTypeParamDecl",
        swc_enum: None,
        babel_checker: "isTSTypeParameterDeclaration",
        swc_pattern: "TsTypeParamDecl",
        visitor_method: "visit_ts_type_parameter_declaration",
        swc_visitor: "visit_mut_ts_type_param_decl",
    },

    // === Patterns ===
    NodeMapping {
        reluxscript: "Pattern",
        babel: "Pattern",
        swc: "Pat",
        swc_enum: None,
        babel_checker: "isPattern",
        swc_pattern: "Pat",
        visitor_method: "visit_pattern",
        swc_visitor: "visit_mut_pat",
    },
    NodeMapping {
        reluxscript: "ArrayPattern",
        babel: "ArrayPattern",
        swc: "ArrayPat",
        swc_enum: Some("Pat::Array"),
        babel_checker: "isArrayPattern",
        swc_pattern: "Pat::Array",
        visitor_method: "visit_array_pattern",
        swc_visitor: "visit_mut_array_pat",
    },
    NodeMapping {
        reluxscript: "ObjectPattern",
        babel: "ObjectPattern",
        swc: "ObjectPat",
        swc_enum: Some("Pat::Object"),
        babel_checker: "isObjectPattern",
        swc_pattern: "Pat::Object",
        visitor_method: "visit_object_pattern",
        swc_visitor: "visit_mut_object_pat",
    },
    NodeMapping {
        reluxscript: "RestElement",
        babel: "RestElement",
        swc: "RestPat",
        swc_enum: Some("Pat::Rest"),
        babel_checker: "isRestElement",
        swc_pattern: "Pat::Rest",
        visitor_method: "visit_rest_element",
        swc_visitor: "visit_mut_rest_pat",
    },
    NodeMapping {
        reluxscript: "AssignmentPattern",
        babel: "AssignmentPattern",
        swc: "AssignPat",
        swc_enum: Some("Pat::Assign"),
        babel_checker: "isAssignmentPattern",
        swc_pattern: "Pat::Assign",
        visitor_method: "visit_assignment_pattern",
        swc_visitor: "visit_mut_assign_pat",
    },
]);

/// Index for fast lookup by ReluxScript name
pub static NODE_MAP: Lazy<HashMap<&'static str, &'static NodeMapping>> = Lazy::new(|| {
    NODE_MAPPINGS
        .iter()
        .map(|m| (m.reluxscript, m))
        .collect()
});

/// Get node mapping by ReluxScript name
pub fn get_node_mapping(reluxscript_name: &str) -> Option<&'static NodeMapping> {
    NODE_MAP.get(reluxscript_name).copied()
}

/// Get node mapping by visitor method name
pub fn get_node_mapping_by_visitor(visitor_method: &str) -> Option<&'static NodeMapping> {
    // First try exact match
    if let Some(mapping) = NODE_MAPPINGS.iter().find(|m| m.visitor_method == visitor_method) {
        return Some(mapping);
    }

    // Try matching without ts_ prefix (e.g., visit_interface_declaration -> visit_ts_interface_declaration)
    let with_ts = if visitor_method.starts_with("visit_") {
        format!("visit_ts_{}", &visitor_method[6..])
    } else {
        return None;
    };

    NODE_MAPPINGS.iter().find(|m| m.visitor_method == with_ts)
}

/// Get SWC type from ReluxScript type
pub fn reluxscript_to_swc(reluxscript_name: &str) -> String {
    get_node_mapping(reluxscript_name)
        .map(|m| m.swc.to_string())
        .unwrap_or_else(|| reluxscript_name.to_string())
}

/// Get Babel type from ReluxScript type
pub fn reluxscript_to_babel(reluxscript_name: &str) -> String {
    get_node_mapping(reluxscript_name)
        .map(|m| m.babel.to_string())
        .unwrap_or_else(|| reluxscript_name.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_node_mapping() {
        let mapping = get_node_mapping("Identifier").unwrap();
        assert_eq!(mapping.swc, "Ident");
        assert_eq!(mapping.babel, "Identifier");
    }

    #[test]
    fn test_function_declaration_mapping() {
        let mapping = get_node_mapping("FunctionDeclaration").unwrap();
        assert_eq!(mapping.swc, "FnDecl");
        assert_eq!(mapping.swc_visitor, "visit_mut_fn_decl");
    }
}
