//! Autofix system for ReluxScript
//!
//! Operates on token streams to fix common patterns that don't parse
//! but have clear alternatives.

pub mod token_rewriter;

pub use token_rewriter::TokenRewriter;
