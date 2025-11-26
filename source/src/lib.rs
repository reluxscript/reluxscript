//! ReluxScript Compiler
//!
//! A language that compiles to both Babel (JavaScript) and SWC (Rust) plugins.

pub mod lexer;
pub mod parser;
pub mod type_system;
pub mod semantic;
#[cfg(feature = "codegen")]
pub mod codegen;
pub mod mapping;
pub mod autofix;
// pub mod error;
// pub mod prelude;

pub use lexer::{Lexer, Token, TokenKind, Span};
pub use parser::{Parser, Program, ParseError};
pub use semantic::{analyze, analyze_with_base_dir, lower, SemanticError, SemanticResult, UnwrapHoister};
#[cfg(feature = "codegen")]
pub use codegen::{generate, generate_with_types, Target, GeneratedCode, SwcDecorator, SwcRewriter};
pub use mapping::{
    NodeMapping, FieldMapping, HelperMapping, PatternMapping,
    get_node_mapping, get_field_mapping, get_helper_for_field, get_pattern_check,
};
pub use autofix::TokenRewriter;

// WASM bindings for playground
#[cfg(feature = "wasm")]
pub mod wasm;
