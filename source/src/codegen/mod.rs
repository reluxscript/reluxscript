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

/// A generated file with path and content
#[derive(Debug, Clone)]
pub struct GeneratedFile {
    /// Relative path for the file (e.g., "lib.rs" or "process_component.rs")
    pub path: String,
    /// Generated code content
    pub content: String,
}

/// Result of code generation
#[derive(Debug)]
pub struct GeneratedCode {
    pub babel: Option<String>,
    pub swc: Option<String>,
    /// Additional generated files (for multi-file output)
    pub babel_modules: Vec<GeneratedFile>,
    pub swc_modules: Vec<GeneratedFile>,
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

    GeneratedCode { babel, swc, babel_modules: vec![], swc_modules: vec![] }
}

/// Generate code with semantic type information (for better type inference)
pub fn generate_with_types(
    program: &crate::parser::Program,
    type_env: crate::semantic::TypeEnv,
    target: Target,
) -> GeneratedCode {
    generate_with_types_and_modules(program, type_env, target, &[])
}

/// Generate code with semantic type information and loaded modules (for multi-file output)
pub fn generate_with_types_and_modules(
    program: &crate::parser::Program,
    type_env: crate::semantic::TypeEnv,
    target: Target,
    loaded_modules: &[crate::semantic::LoadedModule],
) -> GeneratedCode {
    let mut babel_modules = Vec::new();
    let mut swc_modules = Vec::new();

    let babel = if target == Target::Babel || target == Target::Both {
        // Generate main file
        let main = BabelGenerator::new().generate(program);

        // Generate code for each loaded module
        for module in loaded_modules {
            let content = BabelGenerator::new().generate(&module.program);
            // Get filename from path (e.g., "process_component.lux" -> "process_component.js")
            let filename = module.path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("module");
            babel_modules.push(GeneratedFile {
                path: format!("{}.js", filename),
                content,
            });
        }

        Some(main)
    } else {
        None
    };

    let swc = if target == Target::Swc || target == Target::Both {
        // INLINE APPROACH: Generate all module code into a single lib.rs
        // This avoids Rust module system complexity (mod declarations, pub visibility, use crate::...)

        // First, process all loaded modules and collect their emitted code
        let mut module_codes: Vec<String> = Vec::new();

        for module in loaded_modules {
            let mut mod_decorator = SwcDecorator::with_semantic_types(type_env.clone());
            let mod_decorated = mod_decorator.decorate_program(&module.program);

            let mut mod_rewriter = SwcRewriter::new();
            let mod_rewritten = mod_rewriter.rewrite_program(mod_decorated);

            let mut mod_hoister = SwcHoister::new(type_env.clone());
            let mod_hoisted = mod_hoister.hoist_program(mod_rewritten);

            // Use new_inline() for inlined modules - skips header/imports, just emits functions
            let mut mod_emitter = SwcEmitter::new_inline();
            let content = mod_emitter.emit_program(&mod_hoisted);

            if !content.trim().is_empty() {
                let filename = module.path
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("module");
                module_codes.push(format!(
                    "// ============================================================\n\
                     // Module: {}\n\
                     // ============================================================\n\n\
                     {}",
                    filename, content
                ));
            }
        }

        // NEW 4-STAGE PIPELINE: Decorate (with types) → Rewrite → Hoist → Emit
        let mut decorator = SwcDecorator::with_semantic_types(type_env.clone());
        let decorated_program = decorator.decorate_program(program);

        let mut rewriter = SwcRewriter::new();
        let rewritten_program = rewriter.rewrite_program(decorated_program);

        let mut hoister = SwcHoister::new(type_env.clone());
        let hoisted_program = hoister.hoist_program(rewritten_program);

        // Main emitter generates full lib.rs with headers
        // Use new_with_inlined_modules() when there are modules to inline (skips file imports)
        let mut emitter = if !loaded_modules.is_empty() {
            SwcEmitter::new_with_inlined_modules()
        } else {
            SwcEmitter::new()
        };
        let mut main = emitter.emit_program(&hoisted_program);

        // Append all inlined module code at the end
        if !module_codes.is_empty() {
            main.push_str("\n\n");
            main.push_str("// ============================================================\n");
            main.push_str("// INLINED MODULES\n");
            main.push_str("// ============================================================\n\n");
            main.push_str(&module_codes.join("\n\n"));
        }

        Some(main)
    } else {
        None
    };

    GeneratedCode { babel, swc, babel_modules, swc_modules }
}
