//! Helper function generation for abstracting platform differences
//!
//! These helpers encapsulate common operations that differ between Babel and SWC.

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Helper function mapping
#[derive(Debug, Clone)]
pub struct HelperMapping {
    /// Helper name in ReluxScript
    pub name: &'static str,
    /// Babel implementation
    pub babel: &'static str,
    /// SWC implementation
    pub swc: &'static str,
    /// Parameters
    pub params: &'static [(&'static str, &'static str)],
    /// Return type
    pub return_type: &'static str,
}

/// All helper mappings
pub static HELPER_MAPPINGS: Lazy<Vec<HelperMapping>> = Lazy::new(|| vec![
    // === String/Atom Helpers ===
    HelperMapping {
        name: "get_identifier_name",
        params: &[("node", "&Identifier")],
        return_type: "String",
        babel: "node.name",
        swc: "node.sym.to_string()",
    },
    HelperMapping {
        name: "create_identifier",
        params: &[("name", "&str")],
        return_type: "Identifier",
        babel: "t.identifier(name)",
        swc: r#"Ident::new(name.into(), DUMMY_SP)"#,
    },
    HelperMapping {
        name: "get_string_value",
        params: &[("node", "&StringLiteral")],
        return_type: "String",
        babel: "node.value",
        swc: "node.value.to_string()",
    },
    HelperMapping {
        name: "create_string_literal",
        params: &[("value", "&str")],
        return_type: "StringLiteral",
        babel: "t.stringLiteral(value)",
        swc: r#"Str { span: DUMMY_SP, value: value.into(), raw: None }"#,
    },

    // === Node Construction ===
    HelperMapping {
        name: "create_call_expression",
        params: &[("callee", "Expression"), ("args", "Vec<Expression>")],
        return_type: "CallExpression",
        babel: "t.callExpression(callee, args)",
        swc: r#"CallExpr {
            span: DUMMY_SP,
            callee: Callee::Expr(Box::new(callee)),
            args: args.into_iter().map(|a| ExprOrSpread { spread: None, expr: Box::new(a) }).collect(),
            type_args: None,
        }"#,
    },
    HelperMapping {
        name: "create_member_expression",
        params: &[("object", "Expression"), ("property", "&str")],
        return_type: "MemberExpression",
        babel: "t.memberExpression(object, t.identifier(property))",
        swc: r#"MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(object),
            prop: MemberProp::Ident(IdentName::new(property.into(), DUMMY_SP)),
        }"#,
    },
    HelperMapping {
        name: "create_variable_declaration",
        params: &[("kind", "&str"), ("id", "Pattern"), ("init", "Option<Expression>")],
        return_type: "VariableDeclaration",
        babel: r#"t.variableDeclaration(kind, [t.variableDeclarator(id, init)])"#,
        swc: r#"VarDecl {
            span: DUMMY_SP,
            kind: match kind { "const" => VarDeclKind::Const, "let" => VarDeclKind::Let, _ => VarDeclKind::Var },
            declare: false,
            decls: vec![VarDeclarator {
                span: DUMMY_SP,
                name: id,
                init: init.map(Box::new),
                definite: false,
            }],
        }"#,
    },
    HelperMapping {
        name: "create_return_statement",
        params: &[("argument", "Option<Expression>")],
        return_type: "ReturnStatement",
        babel: "t.returnStatement(argument)",
        swc: r#"ReturnStmt { span: DUMMY_SP, arg: argument.map(Box::new) }"#,
    },
    HelperMapping {
        name: "create_expression_statement",
        params: &[("expression", "Expression")],
        return_type: "ExpressionStatement",
        babel: "t.expressionStatement(expression)",
        swc: r#"ExprStmt { span: DUMMY_SP, expr: Box::new(expression) }"#,
    },
    HelperMapping {
        name: "create_block_statement",
        params: &[("body", "Vec<Statement>")],
        return_type: "BlockStatement",
        babel: "t.blockStatement(body)",
        swc: r#"BlockStmt { span: DUMMY_SP, stmts: body }"#,
    },
    HelperMapping {
        name: "create_arrow_function",
        params: &[("params", "Vec<Pattern>"), ("body", "BlockOrExpression"), ("is_async", "bool")],
        return_type: "ArrowFunctionExpression",
        babel: "t.arrowFunctionExpression(params, body, is_async)",
        swc: r#"ArrowExpr {
            span: DUMMY_SP,
            params,
            body: Box::new(body),
            is_async,
            is_generator: false,
            type_params: None,
            return_type: None,
        }"#,
    },

    // === Type Checking Helpers ===
    HelperMapping {
        name: "is_identifier",
        params: &[("node", "&Expression")],
        return_type: "bool",
        babel: "t.isIdentifier(node)",
        swc: "matches!(node, Expr::Ident(_))",
    },
    HelperMapping {
        name: "is_call_expression",
        params: &[("node", "&Expression")],
        return_type: "bool",
        babel: "t.isCallExpression(node)",
        swc: "matches!(node, Expr::Call(_))",
    },
    HelperMapping {
        name: "is_member_expression",
        params: &[("node", "&Expression")],
        return_type: "bool",
        babel: "t.isMemberExpression(node)",
        swc: "matches!(node, Expr::Member(_))",
    },
    HelperMapping {
        name: "is_string_literal",
        params: &[("node", "&Expression")],
        return_type: "bool",
        babel: "t.isStringLiteral(node)",
        swc: "matches!(node, Expr::Lit(Lit::Str(_)))",
    },
    HelperMapping {
        name: "is_function_declaration",
        params: &[("node", "&Statement")],
        return_type: "bool",
        babel: "t.isFunctionDeclaration(node)",
        swc: "matches!(node, Stmt::Decl(Decl::Fn(_)))",
    },

    // === Field Access Helpers ===
    HelperMapping {
        name: "get_callee",
        params: &[("call", "&CallExpression")],
        return_type: "&Expression",
        babel: "call.callee",
        swc: r#"match &call.callee { Callee::Expr(e) => &**e, _ => panic!("Expected Expr callee") }"#,
    },
    HelperMapping {
        name: "get_member_object",
        params: &[("member", "&MemberExpression")],
        return_type: "&Expression",
        babel: "member.object",
        swc: "&*member.obj",
    },
    HelperMapping {
        name: "get_member_property_name",
        params: &[("member", "&MemberExpression")],
        return_type: "Option<String>",
        babel: "member.computed ? null : member.property.name",
        swc: r#"match &member.prop {
            MemberProp::Ident(ident) => Some(ident.sym.to_string()),
            _ => None,
        }"#,
    },
    HelperMapping {
        name: "get_arguments",
        params: &[("call", "&CallExpression")],
        return_type: "Vec<&Expression>",
        babel: "call.arguments",
        swc: "call.args.iter().map(|a| &*a.expr).collect()",
    },

    // === Node Replacement Helpers ===
    HelperMapping {
        name: "replace_with",
        params: &[("target", "&mut Expression"), ("replacement", "Expression")],
        return_type: "()",
        babel: "path.replaceWith(replacement)",
        swc: "*target = replacement",
    },
    HelperMapping {
        name: "remove_node",
        params: &[("target", "&mut Option<T>")],
        return_type: "()",
        babel: "path.remove()",
        swc: "*target = None",
    },

    // === Array/Vec Helpers ===
    HelperMapping {
        name: "push_statement",
        params: &[("body", "&mut Vec<Statement>"), ("stmt", "Statement")],
        return_type: "()",
        babel: "body.push(stmt)",
        swc: "body.push(stmt)",
    },
    HelperMapping {
        name: "insert_before",
        params: &[("body", "&mut Vec<Statement>"), ("index", "usize"), ("stmt", "Statement")],
        return_type: "()",
        babel: "path.insertBefore(stmt)", // Note: Babel path API differs
        swc: "body.insert(index, stmt)",
    },

    // === Span Helpers ===
    HelperMapping {
        name: "dummy_span",
        params: &[],
        return_type: "Span",
        babel: "null", // Babel doesn't use spans
        swc: "DUMMY_SP",
    },
    HelperMapping {
        name: "get_span",
        params: &[("node", "&dyn HasSpan")],
        return_type: "Span",
        babel: "node.loc",
        swc: "node.span",
    },

    // === Pattern Matching Helpers ===
    HelperMapping {
        name: "identifier_equals",
        params: &[("node", "&Identifier"), ("name", "&str")],
        return_type: "bool",
        babel: "node.name === name",
        swc: "&*node.sym == name",
    },
    HelperMapping {
        name: "string_literal_equals",
        params: &[("node", "&StringLiteral"), ("value", "&str")],
        return_type: "bool",
        babel: "node.value === value",
        swc: "&*node.value == value",
    },
]);

/// Index for fast lookup
pub static HELPER_MAP: Lazy<HashMap<&'static str, &'static HelperMapping>> = Lazy::new(|| {
    HELPER_MAPPINGS
        .iter()
        .map(|m| (m.name, m))
        .collect()
});

/// Get helper by name
pub fn get_helper(name: &str) -> Option<&'static HelperMapping> {
    HELPER_MAP.get(name).copied()
}

/// Get helper for a field access pattern
pub fn get_helper_for_field(node_type: &str, field_name: &str) -> Option<&'static str> {
    match (node_type, field_name) {
        ("Identifier", "name") => Some("get_identifier_name"),
        ("StringLiteral", "value") => Some("get_string_value"),
        ("CallExpression", "callee") => Some("get_callee"),
        ("CallExpression", "arguments") => Some("get_arguments"),
        ("MemberExpression", "object") => Some("get_member_object"),
        ("MemberExpression", "property") => Some("get_member_property_name"),
        _ => None,
    }
}

/// Generate Babel helper call
pub fn gen_babel_helper(name: &str, args: &[&str]) -> String {
    if let Some(helper) = get_helper(name) {
        // For simple field accesses, return the babel code with args substituted
        let mut result = helper.babel.to_string();
        for (i, (param_name, _)) in helper.params.iter().enumerate() {
            if i < args.len() {
                result = result.replace(param_name, args[i]);
            }
        }
        result
    } else {
        format!("{}({})", name, args.join(", "))
    }
}

/// Generate SWC helper call
pub fn gen_swc_helper(name: &str, args: &[&str]) -> String {
    if let Some(helper) = get_helper(name) {
        let mut result = helper.swc.to_string();
        for (i, (param_name, _)) in helper.params.iter().enumerate() {
            if i < args.len() {
                result = result.replace(param_name, args[i]);
            }
        }
        result
    } else {
        format!("{}({})", name, args.join(", "))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_identifier_name() {
        let helper = get_helper("get_identifier_name").unwrap();
        assert_eq!(helper.babel, "node.name");
        assert_eq!(helper.swc, "node.sym.to_string()");
    }

    #[test]
    fn test_is_identifier() {
        let helper = get_helper("is_identifier").unwrap();
        assert_eq!(helper.babel, "t.isIdentifier(node)");
        assert!(helper.swc.contains("matches!"));
    }

    #[test]
    fn test_gen_babel_helper() {
        let result = gen_babel_helper("create_identifier", &["\"foo\""]);
        assert!(result.contains("t.identifier"));
    }
}
