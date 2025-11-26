//! Code generation for ReluxScript
//!
//! Generates both Babel (JavaScript) and SWC (Rust) plugin code from ReluxScript AST.

mod babel;
// Temporarily using swc_stub.rs to test rewriter pipeline
#[path = "swc_stub.rs"]
mod swc;
pub mod type_context;
pub mod swc_patterns;
pub mod swc_metadata;
pub mod decorated_ast;
pub mod swc_decorator;
pub mod swc_rewriter;
pub mod swc_emit;

pub use babel::BabelGenerator;
pub use swc::SwcGenerator;
pub use type_context::{TypeContext, TypeEnvironment, SwcTypeKind};
pub use swc_patterns::SwcPatternGenerator;
pub use swc_metadata::*;
pub use decorated_ast::*;
pub use swc_decorator::SwcDecorator;
pub use swc_rewriter::SwcRewriter;
pub use swc_emit::SwcEmitter;

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
        // NEW 3-STAGE PIPELINE: Decorate → Rewrite → Emit
        let mut decorator = SwcDecorator::new();
        let decorated_program = decorator.decorate_program(program);

        let mut rewriter = SwcRewriter::new();
        let rewritten_program = rewriter.rewrite_program(decorated_program);

        let mut emitter = SwcEmitter::new();
        Some(emitter.emit_program(&rewritten_program))
    } else {
        None
    };

    GeneratedCode { babel, swc }
}

/// Generate code with semantic type information (for better type inference)
pub fn generate_with_types(
    program: &crate::parser::Program,
    type_env: crate::semantic::TypeEnv,
    target: Target,
) -> GeneratedCode {
    let babel = if target == Target::Babel || target == Target::Both {
        Some(BabelGenerator::new().generate(program))
    } else {
        None
    };

    let swc = if target == Target::Swc || target == Target::Both {
        // NEW 3-STAGE PIPELINE: Decorate (with types) → Rewrite → Emit
        let mut decorator = SwcDecorator::with_semantic_types(type_env);
        let decorated_program = decorator.decorate_program(program);

        let mut rewriter = SwcRewriter::new();
        let rewritten_program = rewriter.rewrite_program(decorated_program);

        let mut emitter = SwcEmitter::new();
        Some(emitter.emit_program(&rewritten_program))
    } else {
        None
    };

    GeneratedCode { babel, swc }
}
