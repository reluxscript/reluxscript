//! LUXON - Lux Object Notation
//!
//! Manifest format for compiled ReluxScript modules.
//! Similar to .d.ts files in TypeScript or .pdb files in C#.
//!
//! A .luxon file describes the exported symbols from a compiled module,
//! allowing other modules to import them without parsing the source.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// LUXON manifest for a compiled module
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuxonManifest {
    /// Module name
    pub name: String,
    /// Version (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    /// Exported structs
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub structs: HashMap<String, LuxonStruct>,
    /// Exported enums
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub enums: HashMap<String, LuxonEnum>,
    /// Exported functions
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub functions: HashMap<String, LuxonFunction>,
}

/// Struct definition in LUXON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuxonStruct {
    /// Field name -> type
    pub fields: HashMap<String, String>,
    /// Derive attributes (Clone, Debug, etc.)
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derives: Vec<String>,
}

/// Enum definition in LUXON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuxonEnum {
    /// Variant name -> optional tuple/struct fields
    pub variants: HashMap<String, LuxonVariant>,
    /// Derive attributes
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub derives: Vec<String>,
}

/// Enum variant in LUXON
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum LuxonVariant {
    /// Unit variant: `Foo`
    Unit,
    /// Tuple variant: `Foo(i32, String)`
    Tuple(Vec<String>),
    /// Struct variant: `Foo { x: i32, y: String }`
    Struct(HashMap<String, String>),
}

/// Function definition in LUXON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuxonFunction {
    /// Function parameters
    pub params: Vec<LuxonParam>,
    /// Return type (None = void/unit)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub returns: Option<String>,
    /// Type parameters for generics
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub type_params: Vec<String>,
}

/// Function parameter in LUXON
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LuxonParam {
    /// Parameter name
    pub name: String,
    /// Parameter type
    #[serde(rename = "type")]
    pub ty: String,
}

impl LuxonManifest {
    /// Create a new empty manifest
    pub fn new(name: String) -> Self {
        Self {
            name,
            version: None,
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
        }
    }

    /// Load a manifest from a .luxon file
    pub fn load(path: &Path) -> Result<Self, String> {
        let content = fs::read_to_string(path)
            .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;
        serde_json::from_str(&content)
            .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))
    }

    /// Save the manifest to a .luxon file
    pub fn save(&self, path: &Path) -> Result<(), String> {
        let content = serde_json::to_string_pretty(self)
            .map_err(|e| format!("Failed to serialize manifest: {}", e))?;
        fs::write(path, content)
            .map_err(|e| format!("Failed to write {}: {}", path.display(), e))
    }

    /// Add a struct export
    pub fn add_struct(&mut self, name: String, fields: HashMap<String, String>, derives: Vec<String>) {
        self.structs.insert(name, LuxonStruct { fields, derives });
    }

    /// Add an enum export
    pub fn add_enum(&mut self, name: String, variants: HashMap<String, LuxonVariant>, derives: Vec<String>) {
        self.enums.insert(name, LuxonEnum { variants, derives });
    }

    /// Add a function export
    pub fn add_function(&mut self, name: String, params: Vec<LuxonParam>, returns: Option<String>, type_params: Vec<String>) {
        self.functions.insert(name, LuxonFunction { params, returns, type_params });
    }
}

/// Convert a parser Type to a LUXON type string
pub fn type_to_luxon_string(ty: &crate::parser::Type) -> String {
    use crate::parser::Type;

    match ty {
        Type::Primitive(name) => name.clone(),
        Type::Named(name) => name.clone(),
        Type::Reference { mutable, inner } => {
            if *mutable {
                format!("&mut {}", type_to_luxon_string(inner))
            } else {
                format!("&{}", type_to_luxon_string(inner))
            }
        }
        Type::Optional(inner) => format!("Option<{}>", type_to_luxon_string(inner)),
        Type::Container { name, type_args } => {
            if type_args.is_empty() {
                name.clone()
            } else {
                let args: Vec<String> = type_args.iter().map(type_to_luxon_string).collect();
                format!("{}<{}>", name, args.join(", "))
            }
        }
        Type::Tuple(types) => {
            let inner: Vec<String> = types.iter().map(type_to_luxon_string).collect();
            format!("({})", inner.join(", "))
        }
        Type::Array { element } => {
            format!("[{}]", type_to_luxon_string(element))
        }
        Type::FnTrait { params, return_type } => {
            let params_str: Vec<String> = params.iter().map(type_to_luxon_string).collect();
            format!("Fn({}) -> {}", params_str.join(", "), type_to_luxon_string(return_type))
        }
        Type::RawPointer { mutable, inner } => {
            if *mutable {
                format!("*mut {}", type_to_luxon_string(inner))
            } else {
                format!("*const {}", type_to_luxon_string(inner))
            }
        }
        Type::AstNode(name) => name.clone(),
        Type::Unit => "()".to_string(),
    }
}

/// Extract a LUXON manifest from a parsed program
pub fn extract_manifest(program: &crate::parser::Program, name: String) -> LuxonManifest {
    use crate::parser::{TopLevelDecl, PluginItem, EnumVariantFields};

    let mut manifest = LuxonManifest::new(name);

    let items: &[PluginItem] = match &program.decl {
        TopLevelDecl::Plugin(p) => &p.body,
        TopLevelDecl::Writer(w) => &w.body,
        TopLevelDecl::Module(m) => &m.items,
        TopLevelDecl::Interface(_) => return manifest,
    };

    for item in items {
        match item {
            PluginItem::Struct(s) => {
                let fields: HashMap<String, String> = s.fields.iter()
                    .map(|f| (f.name.clone(), type_to_luxon_string(&f.ty)))
                    .collect();
                manifest.add_struct(s.name.clone(), fields, s.derives.clone());
            }
            PluginItem::Enum(e) => {
                let variants: HashMap<String, LuxonVariant> = e.variants.iter()
                    .map(|v| {
                        let variant = match &v.fields {
                            EnumVariantFields::Unit => LuxonVariant::Unit,
                            EnumVariantFields::Tuple(types) => {
                                LuxonVariant::Tuple(types.iter().map(type_to_luxon_string).collect())
                            }
                            EnumVariantFields::Struct(fields) => {
                                LuxonVariant::Struct(fields.iter()
                                    .map(|(name, ty)| (name.clone(), type_to_luxon_string(ty)))
                                    .collect())
                            }
                        };
                        (v.name.clone(), variant)
                    })
                    .collect();
                manifest.add_enum(e.name.clone(), variants, e.derives.clone());
            }
            PluginItem::Function(f) => {
                let params: Vec<LuxonParam> = f.params.iter()
                    .map(|p| LuxonParam {
                        name: p.name.clone(),
                        ty: type_to_luxon_string(&p.ty),
                    })
                    .collect();
                let returns = f.return_type.as_ref().map(type_to_luxon_string);
                let type_params: Vec<String> = f.type_params.iter()
                    .map(|p| p.name.clone())
                    .collect();
                manifest.add_function(f.name.clone(), params, returns, type_params);
            }
            _ => {}
        }
    }

    manifest
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manifest_roundtrip() {
        let mut manifest = LuxonManifest::new("test_module".to_string());
        manifest.version = Some("1.0.0".to_string());

        let mut fields = HashMap::new();
        fields.insert("value".to_string(), "i32".to_string());
        manifest.add_struct("Counter".to_string(), fields, vec!["Clone".to_string(), "Debug".to_string()]);

        manifest.add_function(
            "increment".to_string(),
            vec![LuxonParam { name: "c".to_string(), ty: "&mut Counter".to_string() }],
            None,
            vec![],
        );

        let json = serde_json::to_string_pretty(&manifest).unwrap();
        let parsed: LuxonManifest = serde_json::from_str(&json).unwrap();

        assert_eq!(parsed.name, "test_module");
        assert_eq!(parsed.structs.len(), 1);
        assert_eq!(parsed.functions.len(), 1);
    }
}
