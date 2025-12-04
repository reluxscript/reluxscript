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
pub mod swc_hoister;
pub mod swc_emit;

pub use babel::BabelGenerator;
pub use swc::SwcGenerator;
pub use type_context::{TypeContext, TypeEnvironment, SwcTypeKind};
pub use swc_patterns::SwcPatternGenerator;
pub use swc_metadata::*;
pub use decorated_ast::*;
pub use swc_decorator::SwcDecorator;
pub use swc_rewriter::SwcRewriter;
pub use swc_hoister::SwcHoister;
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

/// Extended result with module information for multi-file output
#[derive(Debug)]
pub struct GeneratedCodeWithModules {
    pub babel: Option<String>,
    pub swc: Option<String>,
    /// Imported modules: (module_name, code, imported_symbols)
    pub swc_modules: Vec<(String, String, Vec<String>)>,
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
    let result = generate_with_types_and_base_dir(program, type_env, target, std::path::PathBuf::from("."));
    GeneratedCode {
        babel: result.babel,
        swc: result.swc,
    }
}

/// Generate code with semantic type information and base directory for module resolution
pub fn generate_with_types_and_base_dir(
    program: &crate::parser::Program,
    type_env: crate::semantic::TypeEnv,
    target: Target,
    base_dir: std::path::PathBuf,
) -> GeneratedCodeWithModules {
    let babel = if target == Target::Babel || target == Target::Both {
        Some(BabelGenerator::new().generate(program))
    } else {
        None
    };

    let (swc, swc_modules) = if target == Target::Swc || target == Target::Both {
        // NEW 4-STAGE PIPELINE: Decorate (with types) → Rewrite → Hoist → Emit
        let mut decorator = SwcDecorator::with_semantic_types(type_env.clone());
        let decorated_program = decorator.decorate_program(program);

        let mut rewriter = SwcRewriter::new();
        let rewritten_program = rewriter.rewrite_program(decorated_program);

        let mut hoister = SwcHoister::new(type_env);
        let hoisted_program = hoister.hoist_program(rewritten_program);

        let mut emitter = SwcEmitter::with_base_dir(base_dir);
        let code = emitter.emit_program(&hoisted_program);

        // Get imported modules and inject mod declarations + use statements
        let modules = emitter.get_imported_modules().to_vec();
        let final_code = if !modules.is_empty() {
            let mod_decls = emitter.generate_mod_declarations();
            let use_stmts = emitter.generate_use_statements();
            inject_module_imports(&code, &mod_decls, &use_stmts)
        } else {
            code
        };

        (Some(final_code), modules)
    } else {
        (None, Vec::new())
    };

    GeneratedCodeWithModules { babel, swc, swc_modules }
}

/// Inject mod declarations and use statements into the generated code
fn inject_module_imports(code: &str, mod_decls: &str, use_stmts: &str) -> String {
    // Find the position after the use statements in the generated code
    let lines: Vec<&str> = code.lines().collect();
    let mut insert_pos = 0;

    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.starts_with("use ") || trimmed.starts_with("//") || trimmed.is_empty() {
            insert_pos = i + 1;
        } else {
            break;
        }
    }

    // Build the result
    let mut result = String::new();
    for (i, line) in lines.iter().enumerate() {
        result.push_str(line);
        result.push('\n');
        if i + 1 == insert_pos {
            // Insert mod declarations and use statements
            result.push_str(mod_decls);
            result.push_str(use_stmts);
            result.push('\n');
        }
    }

    result
}
