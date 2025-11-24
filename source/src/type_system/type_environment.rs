//! Type environment for tracking variable types through scopes

use super::TypeContext;
use std::collections::HashMap;

/// Environment for tracking variable types through nested scopes
#[derive(Debug, Clone)]
pub struct TypeEnvironment {
    scopes: Vec<HashMap<String, TypeContext>>,
}

impl TypeEnvironment {
    pub fn new() -> Self {
        Self {
            scopes: vec![HashMap::new()],
        }
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn define(&mut self, name: &str, type_ctx: TypeContext) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name.to_string(), type_ctx);
        }
    }

    pub fn lookup(&self, name: &str) -> Option<&TypeContext> {
        for scope in self.scopes.iter().rev() {
            if let Some(type_ctx) = scope.get(name) {
                return Some(type_ctx);
            }
        }
        None
    }

    pub fn update(&mut self, name: &str, type_ctx: TypeContext) {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), type_ctx);
                return;
            }
        }
        // If not found, define in current scope
        self.define(name, type_ctx);
    }
}

impl Default for TypeEnvironment {
    fn default() -> Self {
        Self::new()
    }
}
