//! Type Inference (DEPRECATED)
//!
//! ⚠️ This module contains flow-sensitive type inference that will be REMOVED
//! once the decorator metadata system is complete.
//!
//! DO NOT add new code to this module. Use SwcDecorator instead.

use super::SwcGenerator;
use crate::parser::*;
use crate::codegen::type_context::{TypeContext, TypeEnvironment};

impl SwcGenerator {
    pub(super) fn extract_matches_pattern(&self, expr: &Expr) -> Option<(String, String, Option<String>, String)> {
        if let Expr::Call(call) = expr {
            if let Expr::Ident(ident) = call.callee.as_ref() {
                if ident.name == "matches!" && call.args.len() == 2 {
                    // First arg can be variable or member expression
                    let (var_name, field_path, match_expr) = match &call.args[0] {
                        Expr::Ident(id) => (id.name.clone(), None, id.name.clone()),
                        Expr::Member(mem) => {
                            // For obj.field, extract the full path for refinement
                            // but use just the field name for the pattern binding
                            if let Expr::Ident(obj_id) = &*mem.object {
                                let path = format!("{}.{}", obj_id.name, mem.property);

                                // Translate field name to SWC using object's type
                                let obj_type = self.type_env.lookup(&obj_id.name)
                                    .map(|ctx| ctx.swc_type.clone())
                                    .unwrap_or_else(|| "Unknown".to_string());
                                let swc_field = get_typed_field_mapping(&obj_type, &mem.property)
                                    .map(|m| m.swc_field.to_string())
                                    .unwrap_or_else(|| mem.property.clone());

                                let match_expr = format!("{}.{}", obj_id.name, swc_field);
                                // Use the field name as the binding variable
                                (mem.property.clone(), Some(path), match_expr)
                            } else {
                                return None;
                            }
                        }
                        _ => return None,
                    };

                    // Second arg should be the type name
                    let type_name = if let Expr::Ident(id) = &call.args[1] {
                        id.name.clone()
                    } else {
                        return None;
                    };

                    return Some((var_name, type_name, field_path, match_expr));
                }
            }
        }
        None
    }

    /// Infer the type of an expression based on context
    pub(super) fn type_from_ast(&self, ty: &Type) -> TypeContext {
        match ty {
            Type::Named(name) => TypeContext::from_reluxscript(name),
            Type::Primitive(name) => TypeContext::from_reluxscript(name),
            Type::Reference { inner, .. } => self.type_from_ast(inner),
            Type::Container { name, type_args } => {
                // Handle Vec, Option, etc. - preserve type arguments
                if type_args.is_empty() {
                    TypeContext::from_reluxscript(name)
                } else {
                    // Build type with arguments, e.g., Vec<i32>
                    let args: Vec<String> = type_args.iter().map(|t| {
                        let ctx = self.type_from_ast(t);
                        ctx.swc_type
                    }).collect();
                    let full_type = format!("{}<{}>", name, args.join(", "));
                    TypeContext {
                        reluxscript_type: full_type.clone(),
                        swc_type: full_type,
                        kind: super::type_context::SwcTypeKind::Unknown,
                        known_variant: None,
                        needs_deref: false,
                    }
                }
            }
            Type::Array { element } => {
                let _elem_type = self.type_from_ast(element);
                TypeContext::unknown() // Array type - could be improved
            }
            Type::Tuple(_) => TypeContext::unknown(),
            Type::Optional(inner) => self.type_from_ast(inner),
            Type::Unit => TypeContext::unknown(),
            Type::FnTrait { .. } => TypeContext::unknown(),
        }
    }

    /// Get the element type from a collection type (Vec, array, etc.)
}
