mod resolver;
mod type_checker;
mod ownership;
mod types;
pub mod hoist_unwraps;

pub use resolver::Resolver;
pub use type_checker::TypeChecker;
pub use ownership::OwnershipChecker;
pub use types::{TypeInfo, TypeEnv};
pub use hoist_unwraps::UnwrapHoister;

use crate::parser::Program;
use crate::lexer::Span;

/// Semantic error
#[derive(Debug, Clone)]
pub struct SemanticError {
    pub code: &'static str,
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

impl SemanticError {
    pub fn new(code: &'static str, message: impl Into<String>, span: Span) -> Self {
        Self {
            code,
            message: message.into(),
            span,
            hint: None,
        }
    }

    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }
}

/// Result of semantic analysis
pub struct SemanticResult {
    pub errors: Vec<SemanticError>,
    pub warnings: Vec<SemanticError>,
    pub type_env: TypeEnv,
}

/// Run all semantic analysis passes
pub fn analyze(program: &Program) -> SemanticResult {
    analyze_with_base_dir(program, std::path::PathBuf::from("."))
}

/// Run all semantic analysis passes with a specific base directory for module resolution
pub fn analyze_with_base_dir(program: &Program, base_dir: std::path::PathBuf) -> SemanticResult {
    let mut errors = Vec::new();
    let mut warnings = Vec::new();

    // Pass 1: Name resolution
    let mut resolver = Resolver::with_base_dir(base_dir);
    if let Err(e) = resolver.resolve(program) {
        errors.extend(e);
    }

    // Pass 2: Type checking
    let mut type_checker = TypeChecker::new(resolver.get_env());
    if let Err(e) = type_checker.check(program) {
        errors.extend(e);
    }

    // Pass 3: Ownership checking
    let mut ownership_checker = OwnershipChecker::new();
    let (ownership_errors, ownership_warnings) = ownership_checker.check(program);
    errors.extend(ownership_errors);
    warnings.extend(ownership_warnings);

    SemanticResult {
        errors,
        warnings,
        type_env: type_checker.into_env(),
    }
}

/// Run the AST lowering pass (transforms deep chains into explicit pattern matching)
pub fn lower(program: &mut Program) {
    let mut hoister = UnwrapHoister::new();
    hoister.run(program);
}
