//! Type system for ReluxScript semantic analysis

use std::collections::HashMap;

/// Type information
#[derive(Debug, Clone, PartialEq)]
pub enum TypeInfo {
    /// Primitive types
    Str,
    I32,
    U32,
    F64,
    Bool,
    Unit,
    Null,

    /// Reference type
    Ref {
        mutable: bool,
        inner: Box<TypeInfo>,
    },

    /// Container types
    Vec(Box<TypeInfo>),
    Option(Box<TypeInfo>),
    Result(Box<TypeInfo>, Box<TypeInfo>),
    HashMap(Box<TypeInfo>, Box<TypeInfo>),
    HashSet(Box<TypeInfo>),

    /// Tuple type
    Tuple(Vec<TypeInfo>),

    /// Function type
    Function {
        params: Vec<TypeInfo>,
        ret: Box<TypeInfo>,
    },

    /// Struct type
    Struct {
        name: String,
        fields: HashMap<String, TypeInfo>,
    },

    /// Enum type
    Enum {
        name: String,
        variants: HashMap<String, Option<Vec<TypeInfo>>>,
    },

    /// AST node type (from Unified AST)
    AstNode(String),

    /// Module type (for imports like fs, json)
    Module {
        name: String,
    },

    /// Type variable (for inference)
    Var(usize),

    /// Unknown type (error recovery)
    Unknown,

    /// Never type (for functions that don't return)
    Never,
}

impl TypeInfo {
    /// Check if type is copyable (doesn't need clone)
    pub fn is_copy(&self) -> bool {
        matches!(
            self,
            TypeInfo::I32
                | TypeInfo::U32
                | TypeInfo::F64
                | TypeInfo::Bool
                | TypeInfo::Unit
                | TypeInfo::Null
        )
    }

    /// Check if type is a reference
    pub fn is_ref(&self) -> bool {
        matches!(self, TypeInfo::Ref { .. })
    }

    /// Get inner type of reference
    pub fn deref(&self) -> Option<&TypeInfo> {
        match self {
            TypeInfo::Ref { inner, .. } => Some(inner),
            _ => None,
        }
    }

    /// Check if types are compatible for assignment
    pub fn is_assignable_to(&self, target: &TypeInfo) -> bool {
        match (self, target) {
            // Structs with the same name are compatible (nominal typing)
            // Must come before general equality check to avoid comparing field hashmaps
            (TypeInfo::Struct { name: n1, .. }, TypeInfo::Struct { name: n2, .. }) => n1 == n2,

            // Struct can be assigned to AstNode with the same name (nominal typing)
            (TypeInfo::Struct { name, .. }, TypeInfo::AstNode(node_name)) => name == node_name,

            // Enums with the same name are compatible (nominal typing)
            (TypeInfo::Enum { name: n1, .. }, TypeInfo::Enum { name: n2, .. }) => n1 == n2,

            // Enum can be assigned to AstNode with the same name (nominal typing)
            (TypeInfo::Enum { name, .. }, TypeInfo::AstNode(node_name)) => name == node_name,

            // Same types are always assignable
            (a, b) if a == b => true,

            // Null is assignable to Option
            (TypeInfo::Null, TypeInfo::Option(_)) => true,

            // Numeric coercion (i32 to f64)
            (TypeInfo::I32, TypeInfo::F64) => true,

            // Reference compatibility
            (
                TypeInfo::Ref {
                    mutable: m1,
                    inner: i1,
                },
                TypeInfo::Ref {
                    mutable: m2,
                    inner: i2,
                },
            ) => {
                // Mutable ref can coerce to immutable ref
                (*m1 || !*m2) && i1.is_assignable_to(i2)
            }

            // Vec<Unknown> is assignable to any Vec<T> (inference fallback)
            (TypeInfo::Vec(elem), TypeInfo::Vec(expected_elem)) => {
                match elem.as_ref() {
                    TypeInfo::Unknown => true,
                    _ => elem.is_assignable_to(expected_elem),
                }
            }

            // HashMap<Unknown, Unknown> is assignable to any HashMap<K, V>
            (TypeInfo::HashMap(k, v), TypeInfo::HashMap(ek, ev)) => {
                let k_ok = matches!(k.as_ref(), TypeInfo::Unknown) || k.is_assignable_to(ek);
                let v_ok = matches!(v.as_ref(), TypeInfo::Unknown) || v.is_assignable_to(ev);
                k_ok && v_ok
            }

            // HashSet<Unknown> is assignable to any HashSet<T>
            (TypeInfo::HashSet(elem), TypeInfo::HashSet(expected_elem)) => {
                match elem.as_ref() {
                    TypeInfo::Unknown => true,
                    _ => elem.is_assignable_to(expected_elem),
                }
            }

            // Option<Unknown> is assignable to any Option<T>
            (TypeInfo::Option(inner), TypeInfo::Option(expected_inner)) => {
                match inner.as_ref() {
                    TypeInfo::Unknown => true,
                    _ => inner.is_assignable_to(expected_inner),
                }
            }

            // Result<Unknown, Unknown> is assignable to any Result<T, E>
            (TypeInfo::Result(ok, err), TypeInfo::Result(eok, eerr)) => {
                let ok_ok = matches!(ok.as_ref(), TypeInfo::Unknown) || ok.is_assignable_to(eok);
                let err_ok = matches!(err.as_ref(), TypeInfo::Unknown) || err.is_assignable_to(eerr);
                ok_ok && err_ok
            }

            // Unknown matches anything (for error recovery)
            (TypeInfo::Unknown, _) | (_, TypeInfo::Unknown) => true,

            _ => false,
        }
    }

    /// Get a human-readable name for the type
    pub fn display_name(&self) -> String {
        match self {
            TypeInfo::Str => "Str".to_string(),
            TypeInfo::I32 => "i32".to_string(),
            TypeInfo::U32 => "u32".to_string(),
            TypeInfo::F64 => "f64".to_string(),
            TypeInfo::Bool => "bool".to_string(),
            TypeInfo::Unit => "()".to_string(),
            TypeInfo::Null => "null".to_string(),
            TypeInfo::Ref { mutable, inner } => {
                if *mutable {
                    format!("&mut {}", inner.display_name())
                } else {
                    format!("&{}", inner.display_name())
                }
            }
            TypeInfo::Vec(inner) => format!("Vec<{}>", inner.display_name()),
            TypeInfo::Option(inner) => format!("Option<{}>", inner.display_name()),
            TypeInfo::Result(ok, err) => {
                format!("Result<{}, {}>", ok.display_name(), err.display_name())
            }
            TypeInfo::HashMap(k, v) => {
                format!("HashMap<{}, {}>", k.display_name(), v.display_name())
            }
            TypeInfo::HashSet(inner) => format!("HashSet<{}>", inner.display_name()),
            TypeInfo::Tuple(elements) => {
                let elems_str = elements
                    .iter()
                    .map(|e| e.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("({})", elems_str)
            }
            TypeInfo::Function { params, ret } => {
                let params_str = params
                    .iter()
                    .map(|p| p.display_name())
                    .collect::<Vec<_>>()
                    .join(", ");
                format!("fn({}) -> {}", params_str, ret.display_name())
            }
            TypeInfo::Struct { name, .. } => name.clone(),
            TypeInfo::Enum { name, .. } => name.clone(),
            TypeInfo::AstNode(name) => name.clone(),
            TypeInfo::Module { name } => format!("module {}", name),
            TypeInfo::Var(id) => format!("?{}", id),
            TypeInfo::Unknown => "unknown".to_string(),
            TypeInfo::Never => "!".to_string(),
        }
    }
}

/// Type environment - stores type information for all symbols
#[derive(Debug, Clone, Default)]
pub struct TypeEnv {
    /// Type bindings by scope level
    scopes: Vec<HashMap<String, TypeInfo>>,
    /// Struct definitions
    structs: HashMap<String, HashMap<String, TypeInfo>>,
    /// Enum definitions
    enums: HashMap<String, HashMap<String, Option<Vec<TypeInfo>>>>,
    /// Function signatures
    functions: HashMap<String, (Vec<TypeInfo>, TypeInfo)>,
    /// Next type variable ID
    next_var: usize,
}

impl TypeEnv {
    pub fn new() -> Self {
        let mut env = Self {
            scopes: vec![HashMap::new()],
            structs: HashMap::new(),
            enums: HashMap::new(),
            functions: HashMap::new(),
            next_var: 0,
        };

        // Add built-in AST node types
        env.add_ast_types();

        env
    }

    fn add_ast_types(&mut self) {
        let ast_types = [
            "Program",
            "FunctionDeclaration",
            "VariableDeclaration",
            "ExpressionStatement",
            "ReturnStatement",
            "IfStatement",
            "ForStatement",
            "WhileStatement",
            "BlockStatement",
            "Identifier",
            "Literal",
            "BinaryExpression",
            "UnaryExpression",
            "CallExpression",
            "MemberExpression",
            "ArrayExpression",
            "ObjectExpression",
            "JSXElement",
            "JSXFragment",
            "JSXAttribute",
            "JSXText",
            "JSXExpressionContainer",
            "StringLiteral",
            "NumericLiteral",
            "BooleanLiteral",
            "NullLiteral",
            "ArrowFunctionExpression",
        ];

        for name in ast_types {
            // Add as a type in the global scope
            self.scopes[0].insert(name.to_string(), TypeInfo::AstNode(name.to_string()));
        }
    }

    /// Enter a new scope
    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    /// Exit the current scope
    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    /// Define a variable in the current scope
    pub fn define(&mut self, name: String, ty: TypeInfo) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, ty);
        }
    }

    /// Look up a variable's type
    pub fn lookup(&self, name: &str) -> Option<&TypeInfo> {
        for scope in self.scopes.iter().rev() {
            if let Some(ty) = scope.get(name) {
                return Some(ty);
            }
        }
        None
    }

    /// Check if a name is defined in the current scope
    pub fn is_defined_in_current_scope(&self, name: &str) -> bool {
        self.scopes
            .last()
            .map(|s| s.contains_key(name))
            .unwrap_or(false)
    }

    /// Define a struct
    pub fn define_struct(&mut self, name: String, fields: HashMap<String, TypeInfo>) {
        self.structs.insert(name.clone(), fields.clone());
        self.define(name.clone(), TypeInfo::Struct { name, fields });
    }

    /// Get struct fields
    pub fn get_struct_fields(&self, name: &str) -> Option<&HashMap<String, TypeInfo>> {
        self.structs.get(name)
    }

    /// Define an enum
    pub fn define_enum(&mut self, name: String, variants: HashMap<String, Option<Vec<TypeInfo>>>) {
        self.enums.insert(name.clone(), variants.clone());
        self.define(name.clone(), TypeInfo::Enum { name, variants });
    }

    /// Define a function
    pub fn define_function(&mut self, name: String, params: Vec<TypeInfo>, ret: TypeInfo) {
        self.functions.insert(name.clone(), (params.clone(), ret.clone()));
        self.define(
            name,
            TypeInfo::Function {
                params,
                ret: Box::new(ret),
            },
        );
    }

    /// Get function signature
    pub fn get_function(&self, name: &str) -> Option<&(Vec<TypeInfo>, TypeInfo)> {
        self.functions.get(name)
    }

    /// Create a fresh type variable
    pub fn fresh_var(&mut self) -> TypeInfo {
        let id = self.next_var;
        self.next_var += 1;
        TypeInfo::Var(id)
    }
}

/// Convert AST Type to TypeInfo
pub fn ast_type_to_type_info(ty: &crate::parser::Type) -> TypeInfo {
    match ty {
        crate::parser::Type::Primitive(name) => match name.as_str() {
            "Str" => TypeInfo::Str,
            "i32" => TypeInfo::I32,
            "u32" => TypeInfo::U32,
            "f64" | "Number" => TypeInfo::F64,  // Number is an alias for f64
            "bool" | "Bool" => TypeInfo::Bool,  // Accept both lowercase and uppercase
            "()" => TypeInfo::Unit,
            _ => TypeInfo::Unknown,
        },
        crate::parser::Type::Reference { mutable, inner } => TypeInfo::Ref {
            mutable: *mutable,
            inner: Box::new(ast_type_to_type_info(inner)),
        },
        crate::parser::Type::Container { name, type_args } => {
            match name.as_str() {
                "Vec" => {
                    let inner = type_args.first().map(ast_type_to_type_info).unwrap_or(TypeInfo::Unknown);
                    TypeInfo::Vec(Box::new(inner))
                }
                "Option" => {
                    let inner = type_args.first().map(ast_type_to_type_info).unwrap_or(TypeInfo::Unknown);
                    TypeInfo::Option(Box::new(inner))
                }
                "Result" => {
                    let ok = type_args.first().map(ast_type_to_type_info).unwrap_or(TypeInfo::Unknown);
                    let err = type_args.get(1).map(ast_type_to_type_info).unwrap_or(TypeInfo::Str);
                    TypeInfo::Result(Box::new(ok), Box::new(err))
                }
                "HashMap" => {
                    let k = type_args.first().map(ast_type_to_type_info).unwrap_or(TypeInfo::Unknown);
                    let v = type_args.get(1).map(ast_type_to_type_info).unwrap_or(TypeInfo::Unknown);
                    TypeInfo::HashMap(Box::new(k), Box::new(v))
                }
                "HashSet" => {
                    let inner = type_args.first().map(ast_type_to_type_info).unwrap_or(TypeInfo::Unknown);
                    TypeInfo::HashSet(Box::new(inner))
                }
                _ => TypeInfo::Unknown,
            }
        }
        crate::parser::Type::Named(name) => {
            // Check if it's a primitive type name (some may be parsed as Named)
            match name.as_str() {
                "bool" | "Bool" => TypeInfo::Bool,
                "Str" | "String" => TypeInfo::Str,
                "i32" => TypeInfo::I32,
                "u32" => TypeInfo::U32,
                "f64" | "Number" => TypeInfo::F64,
                _ => {
                    // Otherwise, treat as AST node type
                    TypeInfo::AstNode(name.clone())
                }
            }
        }
        crate::parser::Type::Array { element } => {
            TypeInfo::Vec(Box::new(ast_type_to_type_info(element)))
        }
        crate::parser::Type::Tuple(types) => {
            // For now, treat tuples as unknown
            let _ = types;
            TypeInfo::Unknown
        }
        crate::parser::Type::Optional(inner) => {
            TypeInfo::Option(Box::new(ast_type_to_type_info(inner)))
        }
        crate::parser::Type::Unit => TypeInfo::Unit,
        crate::parser::Type::FnTrait { .. } => {
            // Function trait types are treated as unknown for type checking purposes
            // They're mainly syntactic for the Rust target
            TypeInfo::Unknown
        }
    }
}
