//! TypeScript type helper functions
//!
//! These helpers are used in generated code to inspect TypeScript AST types.

/// TypeScript keyword type checks
pub struct TsTypeHelpers;

impl TsTypeHelpers {
    // ==========================================================================
    // Keyword Type Checks
    // ==========================================================================

    /// Check if a type annotation is TSStringKeyword
    ///
    /// Babel: `t.isTSStringKeyword(node)`
    /// SWC: `matches!(node, TsType::TsKeywordType(TsKeywordType { kind: TsKeywordTypeKind::TsStringKeyword, .. }))`
    pub fn is_ts_string_babel() -> &'static str {
        "t.isTSStringKeyword"
    }

    pub fn is_ts_string_swc() -> &'static str {
        "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsStringKeyword)"
    }

    /// Check if a type annotation is TSNumberKeyword
    pub fn is_ts_number_babel() -> &'static str {
        "t.isTSNumberKeyword"
    }

    pub fn is_ts_number_swc() -> &'static str {
        "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsNumberKeyword)"
    }

    /// Check if a type annotation is TSBooleanKeyword
    pub fn is_ts_boolean_babel() -> &'static str {
        "t.isTSBooleanKeyword"
    }

    pub fn is_ts_boolean_swc() -> &'static str {
        "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsBooleanKeyword)"
    }

    /// Check if a type annotation is TSAnyKeyword
    pub fn is_ts_any_babel() -> &'static str {
        "t.isTSAnyKeyword"
    }

    pub fn is_ts_any_swc() -> &'static str {
        "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsAnyKeyword)"
    }

    /// Check if a type annotation is TSVoidKeyword
    pub fn is_ts_void_babel() -> &'static str {
        "t.isTSVoidKeyword"
    }

    pub fn is_ts_void_swc() -> &'static str {
        "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsVoidKeyword)"
    }

    // ==========================================================================
    // Type Reference Helpers
    // ==========================================================================

    /// Check if a type is a TSTypeReference
    pub fn is_ts_type_reference_babel() -> &'static str {
        "t.isTSTypeReference"
    }

    pub fn is_ts_type_reference_swc() -> &'static str {
        "matches!(node, TsType::TsTypeRef(_))"
    }

    /// Get type name from a TSTypeReference
    ///
    /// Babel: `node.typeName.name` (if Identifier)
    /// SWC: `node.type_name` (TsEntityName)
    pub fn get_type_name_babel() -> &'static str {
        "node.typeName && t.isIdentifier(node.typeName) ? node.typeName.name : null"
    }

    pub fn get_type_name_swc() -> &'static str {
        r#"match &node.type_name {
            TsEntityName::Ident(id) => Some(id.sym.to_string()),
            TsEntityName::TsQualifiedName(q) => Some(format!("{}", q)),
        }"#
    }

    /// Get type arguments from a TSTypeReference
    ///
    /// Babel: `node.typeParameters?.params`
    /// SWC: `node.type_params.as_ref().map(|p| &p.params)`
    pub fn get_type_args_babel() -> &'static str {
        "node.typeParameters?.params || []"
    }

    pub fn get_type_args_swc() -> &'static str {
        "node.type_params.as_ref().map(|p| p.params.clone()).unwrap_or_default()"
    }

    // ==========================================================================
    // Array Type Helpers
    // ==========================================================================

    /// Check if a type is a TSArrayType
    pub fn is_ts_array_babel() -> &'static str {
        "t.isTSArrayType"
    }

    pub fn is_ts_array_swc() -> &'static str {
        "matches!(node, TsType::TsArrayType(_))"
    }

    /// Get element type from TSArrayType
    pub fn get_array_element_babel() -> &'static str {
        "node.elementType"
    }

    pub fn get_array_element_swc() -> &'static str {
        "&node.elem_type"
    }

    // ==========================================================================
    // Union Type Helpers
    // ==========================================================================

    /// Check if a type is a TSUnionType
    pub fn is_ts_union_babel() -> &'static str {
        "t.isTSUnionType"
    }

    pub fn is_ts_union_swc() -> &'static str {
        "matches!(node, TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(_)))"
    }

    /// Get types from TSUnionType
    pub fn get_union_types_babel() -> &'static str {
        "node.types"
    }

    pub fn get_union_types_swc() -> &'static str {
        "&node.types"
    }
}

/// Helper function mappings for generated code
#[derive(Debug, Clone)]
pub struct TsHelperMapping {
    pub name: &'static str,
    pub babel_code: &'static str,
    pub swc_code: &'static str,
    pub description: &'static str,
}

/// All TypeScript helper mappings
pub static TS_HELPER_MAPPINGS: &[TsHelperMapping] = &[
    // Keyword checks
    TsHelperMapping {
        name: "is_ts_string",
        babel_code: "t.isTSStringKeyword(node)",
        swc_code: "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsStringKeyword)",
        description: "Check if type is string keyword",
    },
    TsHelperMapping {
        name: "is_ts_number",
        babel_code: "t.isTSNumberKeyword(node)",
        swc_code: "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsNumberKeyword)",
        description: "Check if type is number keyword",
    },
    TsHelperMapping {
        name: "is_ts_boolean",
        babel_code: "t.isTSBooleanKeyword(node)",
        swc_code: "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsBooleanKeyword)",
        description: "Check if type is boolean keyword",
    },
    TsHelperMapping {
        name: "is_ts_any",
        babel_code: "t.isTSAnyKeyword(node)",
        swc_code: "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsAnyKeyword)",
        description: "Check if type is any keyword",
    },
    TsHelperMapping {
        name: "is_ts_void",
        babel_code: "t.isTSVoidKeyword(node)",
        swc_code: "matches!(node, TsType::TsKeywordType(k) if k.kind == TsKeywordTypeKind::TsVoidKeyword)",
        description: "Check if type is void keyword",
    },

    // Type reference
    TsHelperMapping {
        name: "is_ts_type_reference",
        babel_code: "t.isTSTypeReference(node)",
        swc_code: "matches!(node, TsType::TsTypeRef(_))",
        description: "Check if type is a type reference",
    },
    TsHelperMapping {
        name: "get_type_name",
        babel_code: "node.typeName && t.isIdentifier(node.typeName) ? node.typeName.name : null",
        swc_code: r#"match &node.type_name { TsEntityName::Ident(id) => Some(id.sym.to_string()), _ => None }"#,
        description: "Get name from type reference",
    },
    TsHelperMapping {
        name: "get_type_args",
        babel_code: "node.typeParameters?.params || []",
        swc_code: "node.type_params.as_ref().map(|p| p.params.clone()).unwrap_or_default()",
        description: "Get type arguments from type reference",
    },

    // Array type
    TsHelperMapping {
        name: "is_ts_array",
        babel_code: "t.isTSArrayType(node)",
        swc_code: "matches!(node, TsType::TsArrayType(_))",
        description: "Check if type is an array type",
    },
    TsHelperMapping {
        name: "get_array_element",
        babel_code: "node.elementType",
        swc_code: "&node.elem_type",
        description: "Get element type from array type",
    },

    // Union type
    TsHelperMapping {
        name: "is_ts_union",
        babel_code: "t.isTSUnionType(node)",
        swc_code: "matches!(node, TsType::TsUnionOrIntersectionType(TsUnionOrIntersectionType::TsUnionType(_)))",
        description: "Check if type is a union type",
    },
    TsHelperMapping {
        name: "get_union_types",
        babel_code: "node.types",
        swc_code: "&node.types",
        description: "Get types from union type",
    },
];

/// Get a TypeScript helper mapping by name
pub fn get_ts_helper(name: &str) -> Option<&'static TsHelperMapping> {
    TS_HELPER_MAPPINGS.iter().find(|h| h.name == name)
}

/// Generate Babel code for a TypeScript helper
pub fn gen_ts_helper_babel(name: &str, node_var: &str) -> Option<String> {
    get_ts_helper(name).map(|h| h.babel_code.replace("node", node_var))
}

/// Generate SWC code for a TypeScript helper
pub fn gen_ts_helper_swc(name: &str, node_var: &str) -> Option<String> {
    get_ts_helper(name).map(|h| h.swc_code.replace("node", node_var))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_ts_helper() {
        let helper = get_ts_helper("is_ts_string");
        assert!(helper.is_some());
        assert_eq!(helper.unwrap().babel_code, "t.isTSStringKeyword(node)");
    }

    #[test]
    fn test_gen_ts_helper_babel() {
        let code = gen_ts_helper_babel("is_ts_string", "typeNode");
        assert_eq!(code, Some("t.isTSStringKeyword(typeNode)".to_string()));
    }

    #[test]
    fn test_gen_ts_helper_swc() {
        let code = gen_ts_helper_swc("is_ts_type_reference", "myType");
        assert_eq!(code, Some("matches!(myType, TsType::TsTypeRef(_))".to_string()));
    }
}
