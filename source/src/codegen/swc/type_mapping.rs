//! Type Name Conversions
//!
//! This module converts ReluxScript type names to SWC Rust types.

use super::SwcGenerator;
use crate::parser::*;
use crate::mapping::{get_node_mapping, get_node_mapping_by_visitor, get_typed_field_mapping, map_reluxscript_to_swc};
use crate::codegen::type_context::{TypeContext, TypeEnvironment};

impl SwcGenerator {
    pub(super) fn visitor_name_to_swc(&self, name: &str) -> String {
        // Use mapping module: visit_call_expression -> visit_mut_call_expr or visit_call_expr
        if let Some(mapping) = get_node_mapping_by_visitor(name) {
            // For writers (Visit trait), use visit_ prefix instead of visit_mut_
            let visitor_name = mapping.swc_visitor.to_string();
            // Check if this is being called in a Visit context (not VisitMut)
            // For now, we'll just return the mapping as-is and handle it in gen_visit_method
            return visitor_name;
        }
        // Fallback for unknown visitor methods
        let stripped = name.strip_prefix("visit_").unwrap_or(name);
        format!("visit_mut_{}", self.to_swc_node_name(stripped))
    }
    pub(super) fn visitor_name_to_swc_type(&self, name: &str) -> String {
        // Use mapping module to get the SWC type for a visitor method
        if let Some(mapping) = get_node_mapping_by_visitor(name) {
            return mapping.swc.to_string();
        }
        let stripped = name.strip_prefix("visit_").unwrap_or(name);
        self.reluxscript_to_swc_type(stripped)
    }
    pub(super) fn to_swc_node_name(&self, name: &str) -> String {
        // Convert snake_case ReluxScript node names to SWC visitor method suffixes
        // Try to find a mapping by converting snake_case to PascalCase
        let pascal = name.split('_')
            .map(|part| {
                let mut chars = part.chars();
                match chars.next() {
                    None => String::new(),
                    Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
                }
            })
            .collect::<String>();

        if let Some(mapping) = get_node_mapping(&pascal) {
            // Extract the suffix from swc_visitor (e.g., "visit_mut_call_expr" -> "call_expr")
            mapping.swc_visitor
                .strip_prefix("visit_mut_")
                .unwrap_or(&mapping.swc_visitor)
                .to_string()
        } else {
            // Fallback: apply common transformations
            name.replace("_expression", "_expr")
                .replace("_statement", "_stmt")
                .replace("_declaration", "_decl")
        }
    }
    pub(super) fn reluxscript_to_swc_type(&self, name: &str) -> String {
        // Convert ReluxScript node names to SWC AST types using mapping module
        // Handle both snake_case and PascalCase inputs

        // Handle primitive type aliases first
        match name {
            "Number" => return "i32".to_string(),
            "Bool" => return "bool".to_string(),
            _ => {}
        }

        // First try direct lookup (PascalCase)
        if let Some(mapping) = get_node_mapping(name) {
            return mapping.swc.to_string();
        }

        // Try converting snake_case to PascalCase
        let pascal: String = name.split('_')
            .map(|s| {
                let mut chars = s.chars();
                match chars.next() {
                    None => String::new(),
                    Some(c) => c.to_uppercase().chain(chars).collect(),
                }
            })
            .collect();

        if let Some(mapping) = get_node_mapping(&pascal) {
            return mapping.swc.to_string();
        }

        // Fallback: return the PascalCase conversion
        pascal
    }
    pub(super) fn type_to_rust(&self, ty: &Type) -> String {
        match ty {
            Type::Primitive(name) => {
                match name.as_str() {
                    "Str" => "String".to_string(),
                    "()" => "()".to_string(),
                    "Number" => "i32".to_string(),
                    "Bool" => "bool".to_string(),
                    _ => name.clone(),
                }
            }
            Type::Reference { mutable, inner } => {
                let inner_type = self.type_to_rust(inner);
                if *mutable {
                    format!("&mut {}", inner_type)
                } else {
                    format!("&{}", inner_type)
                }
            }
            Type::Container { name, type_args } => {
                let args: Vec<String> = type_args.iter().map(|t| self.type_to_rust(t)).collect();
                format!("{}<{}>", name, args.join(", "))
            }
            Type::Named(name) => {
                // Handle special types
                match name.as_str() {
                    "CodeBuilder" => "String".to_string(),
                    _ => {
                        // Map ReluxScript AST types to SWC types
                        // Don't lowercase - preserve original case for user-defined types
                        self.reluxscript_to_swc_type(name)
                    }
                }
            }
            Type::Array { element } => {
                format!("Vec<{}>", self.type_to_rust(element))
            }
            Type::Tuple(types) => {
                let inner: Vec<String> = types.iter().map(|t| self.type_to_rust(t)).collect();
                format!("({})", inner.join(", "))
            }
            Type::Optional(inner) => {
                format!("Option<{}>", self.type_to_rust(inner))
            }
            Type::Unit => "()".to_string(),
            Type::FnTrait { params, return_type } => {
                let param_types: Vec<String> = params.iter().map(|t| self.type_to_rust(t)).collect();
                let ret = self.type_to_rust(return_type);
                format!("Fn({}) -> {}", param_types.join(", "), ret)
            }
        }
    }

    /// Get a default value for initializing a type
    pub(super) fn get_default_value_for_type(&self, ty: &Type) -> String {
        match ty {
            Type::Primitive(name) => {
                match name.as_str() {
                    "Str" => "String::new()".to_string(),
                    "Number" => "0".to_string(),
                    "Bool" => "false".to_string(),
                    "()" => "()".to_string(),
                    "i32" | "i64" | "u32" | "u64" | "usize" | "isize" => "0".to_string(),
                    "f32" | "f64" => "0.0".to_string(),
                    "char" => "'\\0'".to_string(),
                    _ => "Default::default()".to_string(),
                }
            }
            Type::Container { name, .. } => {
                match name.as_str() {
                    "Vec" => "Vec::new()".to_string(),
                    "HashMap" => "HashMap::new()".to_string(),
                    "HashSet" => "HashSet::new()".to_string(),
                    "Option" => "None".to_string(),
                    _ => format!("{}::new()", name),
                }
            }
            Type::Optional(_) => "None".to_string(),
            Type::Array { .. } => "Vec::new()".to_string(),
            Type::Named(name) => {
                // Handle special types
                match name.as_str() {
                    "CodeBuilder" => "String::new()".to_string(),
                    _ => "Default::default()".to_string(),
                }
            }
            _ => "Default::default()".to_string(),
        }
    }
    pub(super) fn visitor_method_to_swc(&self, method_name: &str) -> String {
        // Convert visit_xxx_yyy to visit_mut_xxx_yyy
        if let Some(stripped) = method_name.strip_prefix("visit_") {
            format!("visit_mut_{}", stripped)
        } else {
            method_name.to_string()
        }
    }
    pub(super) fn reluxscript_type_to_swc(&self, type_name: &str) -> String {
        // Use mapping module to convert ReluxScript AST types to SWC types
        get_node_mapping(type_name)
            .map(|m| m.swc.to_string())
            .unwrap_or_else(|| type_name.to_string())
    }

    /// Extract matches!(var, Type) pattern from an expression
    /// Returns (var_name, type_name, field_path, match_expr) if found
    /// - var_name: the binding variable name (just "id" for "decl.id")
    /// - type_name: the type being matched against
    /// - field_path: Some("decl.id") for field refinement, None for simple var
    /// - match_expr: the full expression to match against ("decl.id" or "var")
    pub(super) fn infer_type(&self, expr: &Expr) -> TypeContext {
        match expr {
            Expr::Ident(ident) => {
                // Look up variable type from environment
                self.type_env.lookup(&ident.name)
                    .cloned()
                    .unwrap_or(TypeContext::unknown())
            }

            Expr::Member(mem) => {
                // 1. Check Refinements First
                if let Expr::Ident(obj) = &*mem.object {
                    if let Some(refined) = self.type_env.lookup_field_refinement(&obj.name, &mem.property) {
                        return refined.clone();
                    }
                }

                // 2. Standard Inference - Get object type, then look up field type
                let obj_type = self.infer_type(&mem.object);
                #[cfg(debug_assertions)]
                eprintln!("[swc] Member: {}.{} -> obj_type={}",
                    match &*mem.object {
                        Expr::Ident(i) => i.name.clone(),
                        _ => "?".to_string(),
                    },
                    mem.property,
                    obj_type.swc_type);
                if let Some(mapping) = get_typed_field_mapping(&obj_type.swc_type, &mem.property) {
                    let (_, kind) = map_reluxscript_to_swc(mapping.result_type_rs);
                    #[cfg(debug_assertions)]
                    eprintln!("[swc]   -> result_type={}", mapping.result_type_swc);
                    TypeContext {
                        reluxscript_type: mapping.result_type_rs.to_string(),
                        swc_type: mapping.result_type_swc.to_string(),
                        kind,
                        known_variant: None,
                        needs_deref: mapping.needs_deref,
                    }
                } else {
                    // Unknown field access
                    #[cfg(debug_assertions)]
                    eprintln!("[swc]   -> no mapping found");
                    TypeContext::unknown()
                }
            }

            Expr::Call(call) => {
                // Check for .clone() which preserves type
                if let Expr::Member(mem) = call.callee.as_ref() {
                    if mem.property == "clone" && call.args.is_empty() {
                        let result = self.infer_type(&mem.object);
                        #[cfg(debug_assertions)]
                        eprintln!("[swc] Call .clone() -> inferred type={}", result.swc_type);
                        return result;
                    }
                }
                // Default: unknown return type
                TypeContext::unknown()
            }

            Expr::VecInit(_) => {
                // Vec initialization - could track element type but for now unknown
                TypeContext::unknown()
            }

            Expr::Ref(ref_expr) => {
                // Reference expression - infer the inner type
                self.infer_type(&ref_expr.expr)
            }

            Expr::Unary(un) => {
                // Unary expression - for Ref/RefMut, infer the inner type
                match un.op {
                    crate::parser::UnaryOp::Ref | crate::parser::UnaryOp::RefMut => {
                        self.infer_type(&un.operand)
                    }
                    _ => TypeContext::unknown(),
                }
            }

            Expr::Index(idx) => {
                // Array/Vec indexing - get the element type
                let container_type = self.infer_type(&idx.object);
                let elem_type = self.get_element_type(&container_type);
                #[cfg(debug_assertions)]
                eprintln!("[swc] Index: container_type={}, elem_type={}",
                    container_type.swc_type, elem_type.swc_type);
                elem_type
            }

            _ => TypeContext::unknown(),
        }
    }

    /// Convert an AST type to a TypeContext
    pub(super) fn get_element_type(&self, container_type: &TypeContext) -> TypeContext {
        // Check if the swc_type represents a Vec or array
        let swc_type = &container_type.swc_type;

        // Handle Vec<T> - extract T
        if swc_type.starts_with("Vec<") && swc_type.ends_with(">") {
            let inner = &swc_type[4..swc_type.len()-1];
            return TypeContext {
                reluxscript_type: inner.to_string(),
                swc_type: inner.to_string(),
                kind: super::type_context::classify_swc_type(inner),
                known_variant: None,
                needs_deref: false,
            };
        }

        // For unknown collections, return unknown
        TypeContext::unknown()
    }

    // ============================================================================
    // DECORATED AST GENERATION (New, metadata-driven)
    // ============================================================================

    /// Generate pattern from decorated AST (uses metadata)
    pub(super) fn binary_op_to_rust(&self, op: &BinaryOp) -> &'static str {
        match op {
            BinaryOp::Add => "+",
            BinaryOp::Sub => "-",
            BinaryOp::Mul => "*",
            BinaryOp::Div => "/",
            BinaryOp::Mod => "%",
            BinaryOp::Eq => "==",
            BinaryOp::NotEq => "!=",
            BinaryOp::Lt => "<",
            BinaryOp::Gt => ">",
            BinaryOp::LtEq => "<=",
            BinaryOp::GtEq => ">=",
            BinaryOp::And => "&&",
            BinaryOp::Or => "||",
        }
    }
    pub(super) fn compound_op_to_rust(&self, op: &CompoundAssignOp) -> &'static str {
        match op {
            CompoundAssignOp::AddAssign => "+",
            CompoundAssignOp::SubAssign => "-",
            CompoundAssignOp::MulAssign => "*",
            CompoundAssignOp::DivAssign => "/",
        }
    }

    /// Generate matches! macro for SWC
}
