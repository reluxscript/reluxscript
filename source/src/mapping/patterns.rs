//! Pattern matching helpers for type checking
//!
//! Generates the correct type checking code for Babel and SWC.

use super::nodes::get_node_mapping;
use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Pattern for matching AST node types
#[derive(Debug, Clone)]
pub struct PatternMapping {
    /// ReluxScript node type
    pub node_type: &'static str,
    /// Babel pattern (using t.isXxx)
    pub babel_check: &'static str,
    /// SWC pattern (using matches! or if let)
    pub swc_pattern: &'static str,
    /// SWC binding variable (what gets bound in the pattern)
    pub swc_binding: &'static str,
}

/// All pattern mappings
pub static PATTERN_MAPPINGS: Lazy<Vec<PatternMapping>> = Lazy::new(|| vec![
    // === Expressions ===
    PatternMapping {
        node_type: "Identifier",
        babel_check: "t.isIdentifier(node)",
        swc_pattern: "Expr::Ident(ident)",
        swc_binding: "ident",
    },
    PatternMapping {
        node_type: "CallExpression",
        babel_check: "t.isCallExpression(node)",
        swc_pattern: "Expr::Call(call_expr)",
        swc_binding: "call_expr",
    },
    PatternMapping {
        node_type: "MemberExpression",
        babel_check: "t.isMemberExpression(node)",
        swc_pattern: "Expr::Member(member_expr)",
        swc_binding: "member_expr",
    },
    PatternMapping {
        node_type: "BinaryExpression",
        babel_check: "t.isBinaryExpression(node)",
        swc_pattern: "Expr::Bin(bin_expr)",
        swc_binding: "bin_expr",
    },
    PatternMapping {
        node_type: "StringLiteral",
        babel_check: "t.isStringLiteral(node)",
        swc_pattern: "Expr::Lit(Lit::Str(str_lit))",
        swc_binding: "str_lit",
    },
    PatternMapping {
        node_type: "NumericLiteral",
        babel_check: "t.isNumericLiteral(node)",
        swc_pattern: "Expr::Lit(Lit::Num(num_lit))",
        swc_binding: "num_lit",
    },
    PatternMapping {
        node_type: "BooleanLiteral",
        babel_check: "t.isBooleanLiteral(node)",
        swc_pattern: "Expr::Lit(Lit::Bool(bool_lit))",
        swc_binding: "bool_lit",
    },
    PatternMapping {
        node_type: "NullLiteral",
        babel_check: "t.isNullLiteral(node)",
        swc_pattern: "Expr::Lit(Lit::Null(_))",
        swc_binding: "_",
    },
    PatternMapping {
        node_type: "ArrayExpression",
        babel_check: "t.isArrayExpression(node)",
        swc_pattern: "Expr::Array(array_lit)",
        swc_binding: "array_lit",
    },
    PatternMapping {
        node_type: "ObjectExpression",
        babel_check: "t.isObjectExpression(node)",
        swc_pattern: "Expr::Object(object_lit)",
        swc_binding: "object_lit",
    },
    PatternMapping {
        node_type: "ArrowFunctionExpression",
        babel_check: "t.isArrowFunctionExpression(node)",
        swc_pattern: "Expr::Arrow(arrow_expr)",
        swc_binding: "arrow_expr",
    },
    PatternMapping {
        node_type: "FunctionExpression",
        babel_check: "t.isFunctionExpression(node)",
        swc_pattern: "Expr::Fn(fn_expr)",
        swc_binding: "fn_expr",
    },

    // === Statements ===
    PatternMapping {
        node_type: "ExpressionStatement",
        babel_check: "t.isExpressionStatement(node)",
        swc_pattern: "Stmt::Expr(expr_stmt)",
        swc_binding: "expr_stmt",
    },
    PatternMapping {
        node_type: "BlockStatement",
        babel_check: "t.isBlockStatement(node)",
        swc_pattern: "Stmt::Block(block_stmt)",
        swc_binding: "block_stmt",
    },
    PatternMapping {
        node_type: "ReturnStatement",
        babel_check: "t.isReturnStatement(node)",
        swc_pattern: "Stmt::Return(return_stmt)",
        swc_binding: "return_stmt",
    },
    PatternMapping {
        node_type: "IfStatement",
        babel_check: "t.isIfStatement(node)",
        swc_pattern: "Stmt::If(if_stmt)",
        swc_binding: "if_stmt",
    },
    PatternMapping {
        node_type: "ForStatement",
        babel_check: "t.isForStatement(node)",
        swc_pattern: "Stmt::For(for_stmt)",
        swc_binding: "for_stmt",
    },
    PatternMapping {
        node_type: "WhileStatement",
        babel_check: "t.isWhileStatement(node)",
        swc_pattern: "Stmt::While(while_stmt)",
        swc_binding: "while_stmt",
    },

    // === Declarations ===
    PatternMapping {
        node_type: "VariableDeclaration",
        babel_check: "t.isVariableDeclaration(node)",
        swc_pattern: "Stmt::Decl(Decl::Var(var_decl))",
        swc_binding: "var_decl",
    },
    PatternMapping {
        node_type: "FunctionDeclaration",
        babel_check: "t.isFunctionDeclaration(node)",
        swc_pattern: "Stmt::Decl(Decl::Fn(fn_decl))",
        swc_binding: "fn_decl",
    },
    PatternMapping {
        node_type: "ClassDeclaration",
        babel_check: "t.isClassDeclaration(node)",
        swc_pattern: "Stmt::Decl(Decl::Class(class_decl))",
        swc_binding: "class_decl",
    },

    // === JSX ===
    PatternMapping {
        node_type: "JSXElement",
        babel_check: "t.isJSXElement(node)",
        swc_pattern: "Expr::JSXElement(jsx_element)",
        swc_binding: "jsx_element",
    },
    PatternMapping {
        node_type: "JSXFragment",
        babel_check: "t.isJSXFragment(node)",
        swc_pattern: "Expr::JSXFragment(jsx_fragment)",
        swc_binding: "jsx_fragment",
    },
]);

/// Index for fast lookup
pub static PATTERN_MAP: Lazy<HashMap<&'static str, &'static PatternMapping>> = Lazy::new(|| {
    PATTERN_MAPPINGS
        .iter()
        .map(|m| (m.node_type, m))
        .collect()
});

/// Get pattern check for a node type
pub fn get_pattern_check(node_type: &str) -> Option<&'static PatternMapping> {
    PATTERN_MAP.get(node_type).copied()
}

/// Generate Babel type check
pub fn gen_babel_type_check(node_var: &str, node_type: &str) -> String {
    if let Some(mapping) = get_pattern_check(node_type) {
        mapping.babel_check.replace("node", node_var)
    } else if let Some(node_mapping) = get_node_mapping(node_type) {
        format!("t.{}({})", node_mapping.babel_checker, node_var)
    } else {
        format!("t.is{}({})", node_type, node_var)
    }
}

/// Generate SWC type check (matches! pattern)
pub fn gen_swc_type_check(node_var: &str, node_type: &str) -> String {
    if let Some(mapping) = get_pattern_check(node_type) {
        format!("matches!({}, {})", node_var, mapping.swc_pattern)
    } else if let Some(node_mapping) = get_node_mapping(node_type) {
        format!("matches!({}, {})", node_var, node_mapping.swc_pattern)
    } else {
        format!("matches!({}, {})", node_var, node_type)
    }
}

/// Generate SWC if-let pattern for extracting value
pub fn gen_swc_if_let(node_var: &str, node_type: &str, body: &str) -> String {
    if let Some(mapping) = get_pattern_check(node_type) {
        format!(
            "if let {} = {} {{\n    {}\n}}",
            mapping.swc_pattern, node_var, body
        )
    } else {
        format!("// Unknown pattern for {}", node_type)
    }
}

/// Generate SWC match arm
pub fn gen_swc_match_arm(node_type: &str, body: &str) -> String {
    if let Some(mapping) = get_pattern_check(node_type) {
        format!("{} => {{\n    {}\n}}", mapping.swc_pattern, body)
    } else if let Some(node_mapping) = get_node_mapping(node_type) {
        format!("{} => {{\n    {}\n}}", node_mapping.swc_pattern, body)
    } else {
        format!("{} => {{\n    {}\n}}", node_type, body)
    }
}

/// Generate comprehensive type check with field assertions (for matches! with fields)
pub fn gen_babel_matches(node_var: &str, node_type: &str, field_checks: &[(&str, &str)]) -> String {
    let mut checks = vec![gen_babel_type_check(node_var, node_type)];

    for (field, value) in field_checks {
        checks.push(format!("{}.{} === {}", node_var, field, value));
    }

    checks.join(" && ")
}

/// Generate SWC matches! with field checks
pub fn gen_swc_matches(node_var: &str, node_type: &str, field_checks: &[(&str, &str)]) -> String {
    if let Some(mapping) = get_pattern_check(node_type) {
        if field_checks.is_empty() {
            format!("matches!({}, {})", node_var, mapping.swc_pattern)
        } else {
            // For complex matches, we need nested if-let
            let binding = mapping.swc_binding;
            let mut result = format!("if let {} = {} {{\n", mapping.swc_pattern, node_var);
            for (field, value) in field_checks {
                result.push_str(&format!("    {} == {} &&\n",
                    format!("{}.{}", binding, field), value));
            }
            result.push_str("    true\n} else { false }");
            result
        }
    } else {
        format!("matches!({}, {})", node_var, node_type)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_babel_type_check() {
        let result = gen_babel_type_check("expr", "Identifier");
        assert_eq!(result, "t.isIdentifier(expr)");
    }

    #[test]
    fn test_swc_type_check() {
        let result = gen_swc_type_check("expr", "Identifier");
        assert!(result.contains("matches!"));
        assert!(result.contains("Expr::Ident"));
    }

    #[test]
    fn test_swc_if_let() {
        let result = gen_swc_if_let("expr", "CallExpression", "// body");
        assert!(result.contains("if let Expr::Call(call_expr)"));
    }

    #[test]
    fn test_nested_pattern() {
        let result = gen_swc_type_check("node", "StringLiteral");
        assert!(result.contains("Expr::Lit(Lit::Str"));
    }
}
