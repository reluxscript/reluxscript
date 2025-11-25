//! WebAssembly bindings for ReluxScript playground

use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

use crate::{Lexer, Parser, analyze_with_base_dir};

#[cfg(feature = "codegen")]
use crate::{generate, Target, lower};

#[derive(Serialize, Deserialize)]
pub struct CompileResult {
    pub success: bool,
    pub babel: Option<String>,
    pub swc: Option<String>,
    pub errors: Vec<CompileError>,
}

#[derive(Serialize, Deserialize)]
pub struct CompileError {
    pub message: String,
    pub line: usize,
    pub column: usize,
}

/// Compile ReluxScript source to Babel and SWC
#[wasm_bindgen]
pub fn compile_reluxscript(source: &str) -> JsValue {
    // Set panic hook for better error messages
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();

    let result = compile_internal(source);
    serde_wasm_bindgen::to_value(&result).unwrap()
}

#[cfg(feature = "codegen")]
fn compile_internal(source: &str) -> CompileResult {
    // Lex
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize();

    // Parse
    let mut parser = Parser::new_with_source(tokens, source.to_string());
    let mut program = match parser.parse() {
        Ok(p) => p,
        Err(e) => {
            return CompileResult {
                success: false,
                babel: None,
                swc: None,
                errors: vec![CompileError {
                    message: format!("Parse error: {}", e.message),
                    line: e.span.line,
                    column: e.span.column,
                }],
            };
        }
    };

    // Analyze
    let base_dir = std::path::PathBuf::from(".");
    let analysis = analyze_with_base_dir(&program, base_dir);

    if !analysis.errors.is_empty() {
        return CompileResult {
            success: false,
            babel: None,
            swc: None,
            errors: analysis.errors.iter().map(|e| CompileError {
                message: e.message.clone(),
                line: e.span.line,
                column: e.span.column,
            }).collect(),
        };
    }

    // Lower
    lower(&mut program);

    // Generate
    let generated = generate(&program, Target::Both);

    CompileResult {
        success: true,
        babel: generated.babel,
        swc: generated.swc,
        errors: vec![],
    }
}

#[cfg(not(feature = "codegen"))]
fn compile_internal(_source: &str) -> CompileResult {
    CompileResult {
        success: false,
        babel: None,
        swc: None,
        errors: vec![CompileError {
            message: "Codegen feature not enabled".to_string(),
            line: 0,
            column: 0,
        }],
    }
}
