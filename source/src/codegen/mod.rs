//! Code generation for ReluxScript
//!
//! Generates both Babel (JavaScript) and SWC (Rust) plugin code from ReluxScript AST.

mod babel;
mod swc;
pub mod type_context;
pub mod swc_patterns;
pub mod swc_metadata;
pub mod decorated_ast;
pub mod swc_decorator;

pub use babel::BabelGenerator;
pub use swc::SwcGenerator;
pub use type_context::{TypeContext, TypeEnvironment, SwcTypeKind};
pub use swc_patterns::SwcPatternGenerator;
pub use swc_metadata::*;
pub use decorated_ast::*;
pub use swc_decorator::SwcDecorator;

/// Target platform for code generation
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Target {
    Babel,
    Swc,
    Both,
}

/// Result of code generation
#[derive(Debug)]
pub struct GeneratedCode {
    pub babel: Option<String>,
    pub swc: Option<String>,
}

/// Generate code for the given target(s)
pub fn generate(program: &crate::parser::Program, target: Target) -> GeneratedCode {
    let babel = if target == Target::Babel || target == Target::Both {
        Some(BabelGenerator::new().generate(program))
    } else {
        None
    };

    let swc = if target == Target::Swc || target == Target::Both {
        Some(SwcGenerator::new().generate(program))
    } else {
        None
    };

    GeneratedCode { babel, swc }
}
