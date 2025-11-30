//! Field mappings between ReluxScript, Babel, and SWC
//!
//! This module handles the field-level divergence between platforms.
//! For example: `node.name` in Babel vs `node.sym` in SWC.

use std::collections::HashMap;
use once_cell::sync::Lazy;

/// Field mapping for a specific node type
#[derive(Debug, Clone)]
pub struct FieldMapping {
    /// Node type this field belongs to
    pub node_type: &'static str,
    /// ReluxScript field name
    pub reluxscript: &'static str,
    /// Babel field access
    pub babel: &'static str,
    /// SWC field access (may include conversions)
    pub swc: &'static str,
    /// SWC type of this field
    pub swc_type: &'static str,
    /// Whether this field needs Box unwrapping in SWC
    pub needs_box_unwrap: bool,
    /// Whether this field is optional
    pub optional: bool,
    /// Conversion needed when reading (e.g., JsWord → String)
    pub read_conversion: Option<&'static str>,
    /// Conversion needed when writing (e.g., String → JsWord)
    pub write_conversion: Option<&'static str>,
}

/// All field mappings
pub static FIELD_MAPPINGS: Lazy<Vec<FieldMapping>> = Lazy::new(|| vec![
    // === Identifier ===
    FieldMapping {
        node_type: "Identifier",
        reluxscript: "name",
        babel: "name",
        swc: "sym",
        swc_type: "Atom",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: Some(".to_string()"),
        write_conversion: Some(".into()"),
    },

    // === FunctionDeclaration ===
    FieldMapping {
        node_type: "FunctionDeclaration",
        reluxscript: "id",
        babel: "id",
        swc: "ident",
        swc_type: "Ident",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "FunctionDeclaration",
        reluxscript: "params",
        babel: "params",
        swc: "function.params",
        swc_type: "Vec<Param>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "FunctionDeclaration",
        reluxscript: "body",
        babel: "body",
        swc: "function.body",
        swc_type: "Option<BlockStmt>",
        needs_box_unwrap: false,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "FunctionDeclaration",
        reluxscript: "async",
        babel: "async",
        swc: "function.is_async",
        swc_type: "bool",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "FunctionDeclaration",
        reluxscript: "generator",
        babel: "generator",
        swc: "function.is_generator",
        swc_type: "bool",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === CallExpression ===
    FieldMapping {
        node_type: "CallExpression",
        reluxscript: "callee",
        babel: "callee",
        swc: "callee",
        swc_type: "Callee",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: Some(".as_expr().unwrap()"),  // Callee::Expr(...) -> &Expr
        write_conversion: None,
    },
    FieldMapping {
        node_type: "CallExpression",
        reluxscript: "arguments",
        babel: "arguments",
        swc: "args",
        swc_type: "Vec<ExprOrSpread>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === MemberExpression ===
    FieldMapping {
        node_type: "MemberExpression",
        reluxscript: "object",
        babel: "object",
        swc: "obj",
        swc_type: "Box<Expr>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "MemberExpression",
        reluxscript: "property",
        babel: "property",
        swc: "prop",
        swc_type: "MemberProp",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "MemberExpression",
        reluxscript: "computed",
        babel: "computed",
        swc: "computed", // Inferred from MemberProp variant
        swc_type: "bool",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: Some("matches!(prop, MemberProp::Computed(_))"),
        write_conversion: None,
    },

    // === BinaryExpression ===
    FieldMapping {
        node_type: "BinaryExpression",
        reluxscript: "left",
        babel: "left",
        swc: "left",
        swc_type: "Box<Expr>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "BinaryExpression",
        reluxscript: "right",
        babel: "right",
        swc: "right",
        swc_type: "Box<Expr>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "BinaryExpression",
        reluxscript: "operator",
        babel: "operator",
        swc: "op",
        swc_type: "BinaryOp",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: Some("op_to_string()"),
        write_conversion: Some("string_to_op()"),
    },

    // === AssignmentExpression ===
    FieldMapping {
        node_type: "AssignmentExpression",
        reluxscript: "left",
        babel: "left",
        swc: "left",
        swc_type: "AssignTarget",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "AssignmentExpression",
        reluxscript: "right",
        babel: "right",
        swc: "right",
        swc_type: "Box<Expr>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "AssignmentExpression",
        reluxscript: "operator",
        babel: "operator",
        swc: "op",
        swc_type: "AssignOp",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === VariableDeclaration ===
    FieldMapping {
        node_type: "VariableDeclaration",
        reluxscript: "kind",
        babel: "kind",
        swc: "kind",
        swc_type: "VarDeclKind",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "VariableDeclaration",
        reluxscript: "declarations",
        babel: "declarations",
        swc: "decls",
        swc_type: "Vec<VarDeclarator>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === VariableDeclarator ===
    FieldMapping {
        node_type: "VariableDeclarator",
        reluxscript: "id",
        babel: "id",
        swc: "name",
        swc_type: "Pat",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "VariableDeclarator",
        reluxscript: "init",
        babel: "init",
        swc: "init",
        swc_type: "Option<Box<Expr>>",
        needs_box_unwrap: true,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },

    // === ReturnStatement ===
    FieldMapping {
        node_type: "ReturnStatement",
        reluxscript: "argument",
        babel: "argument",
        swc: "arg",
        swc_type: "Option<Box<Expr>>",
        needs_box_unwrap: true,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },

    // === IfStatement ===
    FieldMapping {
        node_type: "IfStatement",
        reluxscript: "test",
        babel: "test",
        swc: "test",
        swc_type: "Box<Expr>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "IfStatement",
        reluxscript: "consequent",
        babel: "consequent",
        swc: "cons",
        swc_type: "Box<Stmt>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "IfStatement",
        reluxscript: "alternate",
        babel: "alternate",
        swc: "alt",
        swc_type: "Option<Box<Stmt>>",
        needs_box_unwrap: true,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },

    // === BlockStatement ===
    FieldMapping {
        node_type: "BlockStatement",
        reluxscript: "body",
        babel: "body",
        swc: "stmts",
        swc_type: "Vec<Stmt>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === ArrowFunctionExpression ===
    FieldMapping {
        node_type: "ArrowFunctionExpression",
        reluxscript: "params",
        babel: "params",
        swc: "params",
        swc_type: "Vec<Pat>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "ArrowFunctionExpression",
        reluxscript: "body",
        babel: "body",
        swc: "body",
        swc_type: "Box<BlockStmtOrExpr>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "ArrowFunctionExpression",
        reluxscript: "async",
        babel: "async",
        swc: "is_async",
        swc_type: "bool",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === StringLiteral ===
    FieldMapping {
        node_type: "StringLiteral",
        reluxscript: "value",
        babel: "value",
        swc: "value",
        swc_type: "Atom",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: Some(".to_string()"),
        write_conversion: Some(".into()"),
    },

    // === NumericLiteral ===
    FieldMapping {
        node_type: "NumericLiteral",
        reluxscript: "value",
        babel: "value",
        swc: "value",
        swc_type: "f64",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === ArrayExpression ===
    FieldMapping {
        node_type: "ArrayExpression",
        reluxscript: "elements",
        babel: "elements",
        swc: "elems",
        swc_type: "Vec<Option<ExprOrSpread>>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === ArrayPattern ===
    FieldMapping {
        node_type: "ArrayPattern",
        reluxscript: "elements",
        babel: "elements",
        swc: "elems",
        swc_type: "Vec<Option<Pat>>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === ObjectExpression ===
    FieldMapping {
        node_type: "ObjectExpression",
        reluxscript: "properties",
        babel: "properties",
        swc: "props",
        swc_type: "Vec<PropOrSpread>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === ImportDeclaration ===
    FieldMapping {
        node_type: "ImportDeclaration",
        reluxscript: "source",
        babel: "source",
        swc: "src",
        swc_type: "Box<Str>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "ImportDeclaration",
        reluxscript: "specifiers",
        babel: "specifiers",
        swc: "specifiers",
        swc_type: "Vec<ImportSpecifier>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === JSXElement ===
    FieldMapping {
        node_type: "JSXElement",
        reluxscript: "openingElement",
        babel: "openingElement",
        swc: "opening",
        swc_type: "JSXOpeningElement",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "JSXElement",
        reluxscript: "children",
        babel: "children",
        swc: "children",
        swc_type: "Vec<JSXElementChild>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "JSXElement",
        reluxscript: "closingElement",
        babel: "closingElement",
        swc: "closing",
        swc_type: "Option<JSXClosingElement>",
        needs_box_unwrap: false,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },

    // === JSXOpeningElement ===
    FieldMapping {
        node_type: "JSXOpeningElement",
        reluxscript: "name",
        babel: "name",
        swc: "name",
        swc_type: "JSXElementName",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "JSXOpeningElement",
        reluxscript: "attributes",
        babel: "attributes",
        swc: "attrs",
        swc_type: "Vec<JSXAttrOrSpread>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "JSXOpeningElement",
        reluxscript: "selfClosing",
        babel: "selfClosing",
        swc: "self_closing",
        swc_type: "bool",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === JSXAttribute ===
    FieldMapping {
        node_type: "JSXAttribute",
        reluxscript: "name",
        babel: "name",
        swc: "name",
        swc_type: "JSXAttrName",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "JSXAttribute",
        reluxscript: "value",
        babel: "value",
        swc: "value",
        swc_type: "Option<JSXAttrValue>",
        needs_box_unwrap: false,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },

    // === TSInterfaceDeclaration ===
    FieldMapping {
        node_type: "TSInterfaceDeclaration",
        reluxscript: "id",
        babel: "id",
        swc: "id",
        swc_type: "Ident",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSInterfaceDeclaration",
        reluxscript: "body",
        babel: "body",
        swc: "body",
        swc_type: "TsInterfaceBody",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSInterfaceDeclaration",
        reluxscript: "members",
        babel: "body.body",
        swc: "body.body",
        swc_type: "Vec<TsTypeElement>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSInterfaceDeclaration",
        reluxscript: "extends",
        babel: "extends",
        swc: "extends",
        swc_type: "Vec<TsExprWithTypeArgs>",
        needs_box_unwrap: false,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSInterfaceDeclaration",
        reluxscript: "typeParameters",
        babel: "typeParameters",
        swc: "type_params",
        swc_type: "Option<Box<TsTypeParamDecl>>",
        needs_box_unwrap: true,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },

    // === TSPropertySignature ===
    FieldMapping {
        node_type: "TSPropertySignature",
        reluxscript: "key",
        babel: "key",
        swc: "key",
        swc_type: "Box<Expr>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSPropertySignature",
        reluxscript: "typeAnnotation",
        babel: "typeAnnotation",
        swc: "type_ann",
        swc_type: "Option<Box<TsTypeAnn>>",
        needs_box_unwrap: true,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSPropertySignature",
        reluxscript: "optional",
        babel: "optional",
        swc: "optional",
        swc_type: "bool",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSPropertySignature",
        reluxscript: "readonly",
        babel: "readonly",
        swc: "readonly",
        swc_type: "bool",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === TSTypeAnnotation ===
    FieldMapping {
        node_type: "TSTypeAnnotation",
        reluxscript: "typeAnnotation",
        babel: "typeAnnotation",
        swc: "type_ann",
        swc_type: "Box<TsType>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === TSTypeReference ===
    FieldMapping {
        node_type: "TSTypeReference",
        reluxscript: "typeName",
        babel: "typeName",
        swc: "type_name",
        swc_type: "TsEntityName",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSTypeReference",
        reluxscript: "typeParameters",
        babel: "typeParameters",
        swc: "type_params",
        swc_type: "Option<Box<TsTypeParamInstantiation>>",
        needs_box_unwrap: true,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },

    // === TSTypeParameterInstantiation ===
    FieldMapping {
        node_type: "TSTypeParameterInstantiation",
        reluxscript: "params",
        babel: "params",
        swc: "params",
        swc_type: "Vec<Box<TsType>>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === TSArrayType ===
    FieldMapping {
        node_type: "TSArrayType",
        reluxscript: "elementType",
        babel: "elementType",
        swc: "elem_type",
        swc_type: "Box<TsType>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === CallExpression typeParameters ===
    FieldMapping {
        node_type: "CallExpression",
        reluxscript: "typeArguments",
        babel: "typeParameters.params",
        swc: "type_args.as_ref().map(|t| &t.params)",
        swc_type: "Option<Vec<Box<TsType>>>",
        needs_box_unwrap: false,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },

    // === TemplateLiteral ===
    FieldMapping {
        node_type: "TemplateLiteral",
        reluxscript: "quasis",
        babel: "quasis",
        swc: "quasis",
        swc_type: "Vec<TplElement>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TemplateLiteral",
        reluxscript: "expressions",
        babel: "expressions",
        swc: "exprs",
        swc_type: "Vec<Box<Expr>>",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === TemplateElement ===
    FieldMapping {
        node_type: "TemplateElement",
        reluxscript: "value",
        babel: "value",
        swc: "raw",  // SWC has raw and cooked directly on TplElement
        swc_type: "Atom",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TemplateElement",
        reluxscript: "tail",
        babel: "tail",
        swc: "tail",
        swc_type: "bool",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },

    // === TSPropertySignature ===
    FieldMapping {
        node_type: "TSPropertySignature",
        reluxscript: "key",
        babel: "key",
        swc: "key",
        swc_type: "Box<Expr>",
        needs_box_unwrap: true,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSPropertySignature",
        reluxscript: "optional",
        babel: "optional",
        swc: "optional",
        swc_type: "bool",
        needs_box_unwrap: false,
        optional: false,
        read_conversion: None,
        write_conversion: None,
    },
    FieldMapping {
        node_type: "TSPropertySignature",
        reluxscript: "typeAnnotation",
        babel: "typeAnnotation",
        swc: "type_ann",
        swc_type: "Option<Box<TsTypeAnn>>",
        needs_box_unwrap: true,
        optional: true,
        read_conversion: None,
        write_conversion: None,
    },
]);

/// Index for fast lookup by (node_type, field_name)
pub static FIELD_MAP: Lazy<HashMap<(&'static str, &'static str), &'static FieldMapping>> = Lazy::new(|| {
    FIELD_MAPPINGS
        .iter()
        .map(|m| ((m.node_type, m.reluxscript), m))
        .collect()
});

/// Get field mapping
pub fn get_field_mapping(node_type: &str, field_name: &str) -> Option<&'static FieldMapping> {
    FIELD_MAP.get(&(node_type, field_name)).copied()
}

/// Get all fields for a node type
pub fn get_fields_for_node(node_type: &str) -> Vec<&'static FieldMapping> {
    FIELD_MAPPINGS
        .iter()
        .filter(|m| m.node_type == node_type)
        .collect()
}

/// Generate Babel field access code
pub fn gen_babel_field_access(node_var: &str, node_type: &str, field_name: &str) -> String {
    if let Some(mapping) = get_field_mapping(node_type, field_name) {
        format!("{}.{}", node_var, mapping.babel)
    } else {
        format!("{}.{}", node_var, field_name)
    }
}

/// Generate SWC field access code
pub fn gen_swc_field_access(node_var: &str, node_type: &str, field_name: &str) -> String {
    if let Some(mapping) = get_field_mapping(node_type, field_name) {
        let base = format!("{}.{}", node_var, mapping.swc);
        if let Some(conversion) = mapping.read_conversion {
            format!("{}{}", base, conversion)
        } else if mapping.needs_box_unwrap {
            format!("&*{}", base)
        } else {
            base
        }
    } else {
        format!("{}.{}", node_var, field_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_identifier_name_mapping() {
        let mapping = get_field_mapping("Identifier", "name").unwrap();
        assert_eq!(mapping.babel, "name");
        assert_eq!(mapping.swc, "sym");
        assert_eq!(mapping.read_conversion, Some(".to_string()"));
    }

    #[test]
    fn test_call_expression_arguments() {
        let mapping = get_field_mapping("CallExpression", "arguments").unwrap();
        assert_eq!(mapping.babel, "arguments");
        assert_eq!(mapping.swc, "args");
    }

    #[test]
    fn test_gen_field_access() {
        let babel = gen_babel_field_access("node", "Identifier", "name");
        let swc = gen_swc_field_access("node", "Identifier", "name");
        assert_eq!(babel, "node.name");
        assert_eq!(swc, "node.sym.to_string()");
    }
}
