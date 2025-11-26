//! ReluxScript Compiler CLI

use clap::{Parser as ClapParser, Subcommand};
use std::fs;
use std::path::PathBuf;

use reluxscript::{Lexer, Parser, analyze_with_base_dir, TokenRewriter};

#[cfg(feature = "codegen")]
use reluxscript::{generate, Target, lower};

#[derive(ClapParser)]
#[command(name = "reluxscript")]
#[command(about = "ReluxScript compiler - compile to Babel and SWC plugins")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new ReluxScript plugin
    New {
        /// Plugin name
        name: String,
    },
    /// Tokenize a ReluxScript file (for debugging)
    Lex {
        /// Input file
        file: PathBuf,
    },
    /// Parse a ReluxScript file (for debugging)
    Parse {
        /// Input file
        file: PathBuf,
    },
    /// Check a ReluxScript file for errors
    Check {
        /// Input file
        file: PathBuf,
        /// Automatically fix common issues (path-qualified if-let patterns)
        #[arg(long)]
        autofix: bool,
    },
    /// Build a ReluxScript project
    #[cfg(feature = "codegen")]
    Build {
        /// Input file
        file: PathBuf,
        /// Target platform (babel, swc, both)
        #[arg(short, long, default_value = "both")]
        target: String,
        /// Output directory
        #[arg(short, long, default_value = "dist")]
        output: PathBuf,
        /// Automatically fix common issues (path-qualified if-let patterns)
        #[arg(long)]
        autofix: bool,
        /// Dump decorated AST for SWC (debug mode - before rewriting)
        #[arg(long)]
        dump_decorated_ast: bool,
        /// Dump rewritten AST for SWC (debug mode - after rewriting)
        #[arg(long)]
        dump_rewritten_ast: bool,
    },
    /// Fix common issues in ReluxScript files (rewrites in-place)
    Fix {
        /// Input file(s)
        files: Vec<PathBuf>,
        /// Show what would be changed without writing
        #[arg(long)]
        dry_run: bool,
    },
}

fn main() {
    let cli = <Cli as ClapParser>::parse();

    match cli.command {
        Commands::New { name } => {
            // Create plugin file
            let plugin_file = PathBuf::from(format!("{}.lux", name));

            if plugin_file.exists() {
                eprintln!("Error: {} already exists", plugin_file.display());
                std::process::exit(1);
            }

            // Create a basic plugin template
            let template = format!(r#"// {name} - A ReluxScript plugin
// Edit this file to implement your AST transformation

plugin {name} {{
    fn visit_call_expression(node: &mut CallExpression, ctx: &Context) {{
        // Example: Remove console.log calls
        // if matches!(node.callee, "console.log") {{
        //     *node = Statement::empty();
        // }}
    }}
}}
"#, name = name);

            if let Err(e) = fs::write(&plugin_file, template) {
                eprintln!("Error creating plugin file: {}", e);
                std::process::exit(1);
            }

            println!("Created new plugin: {}", plugin_file.display());
            println!("\nNext steps:");
            println!("  1. Edit {} to implement your transformation", plugin_file.display());
            println!("  2. Build to Babel: relux build {} --target babel", plugin_file.display());
            println!("  3. Build to SWC: relux build {} --target swc", plugin_file.display());
        }
        Commands::Lex { file } => {
            let source = match fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            };

            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();

            println!("Tokens for {:?}:", file);
            println!("{:-<60}", "");
            for token in &tokens {
                println!(
                    "{:>4}:{:<3} {:?}",
                    token.span.line,
                    token.span.column,
                    token.kind
                );
            }
            println!("{:-<60}", "");
            println!("Total tokens: {}", tokens.len());
        }
        Commands::Parse { file } => {
            let source = match fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            };

            let mut lexer = Lexer::new(&source);
            let tokens = lexer.tokenize();
            let mut parser = Parser::new_with_source(tokens, source.clone());

            match parser.parse() {
                Ok(program) => {
                    println!("Successfully parsed {:?}", file);
                    println!("{:-<60}", "");
                    println!("{:#?}", program);
                }
                Err(e) => {
                    eprintln!("Parse error at {}:{}: {}", e.span.line, e.span.column, e.message);
                    std::process::exit(1);
                }
            }
        }
        Commands::Check { file, autofix } => {
            let source = match fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            };

            let mut lexer = Lexer::new(&source);
            let mut tokens = lexer.tokenize();

            // Apply autofix if requested
            if autofix {
                let rewriter = TokenRewriter::new(tokens);
                let (fixed_tokens, fixes_applied) = rewriter.rewrite();
                tokens = fixed_tokens;
                if fixes_applied > 0 {
                    println!("Autofix: Applied {} fix(es)", fixes_applied);
                }
            }

            let mut parser = Parser::new_with_source(tokens, source.clone());

            let program = match parser.parse() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Parse error at {}:{}: {}", e.span.line, e.span.column, e.message);
                    std::process::exit(1);
                }
            };

            // Get base directory from file path
            let base_dir = file.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf();
            let result = analyze_with_base_dir(&program, base_dir);

            // Print errors
            for error in &result.errors {
                eprintln!(
                    "error[{}]: {} at {}:{}",
                    error.code, error.message, error.span.line, error.span.column
                );
                if let Some(ref hint) = error.hint {
                    eprintln!("  help: {}", hint);
                }
            }

            // Print warnings
            for warning in &result.warnings {
                eprintln!(
                    "warning[{}]: {} at {}:{}",
                    warning.code, warning.message, warning.span.line, warning.span.column
                );
                if let Some(ref hint) = warning.hint {
                    eprintln!("  help: {}", hint);
                }
            }

            if result.errors.is_empty() {
                println!("Check passed: {:?}", file);
                if !result.warnings.is_empty() {
                    println!("  {} warning(s)", result.warnings.len());
                }
            } else {
                eprintln!("Check failed: {} error(s)", result.errors.len());
                std::process::exit(1);
            }
        }
        #[cfg(feature = "codegen")]
        Commands::Build { file, target, output, autofix, dump_decorated_ast, dump_rewritten_ast } => {
            let source = match fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("Error reading file: {}", e);
                    std::process::exit(1);
                }
            };

            // Parse
            let mut lexer = Lexer::new(&source);
            let mut tokens = lexer.tokenize();

            // Apply autofix if requested
            if autofix {
                let rewriter = TokenRewriter::new(tokens);
                let (fixed_tokens, fixes_applied) = rewriter.rewrite();
                tokens = fixed_tokens;
                if fixes_applied > 0 {
                    println!("Autofix: Applied {} fix(es)", fixes_applied);
                }
            }

            let mut parser = Parser::new_with_source(tokens, source.clone());

            let mut program = match parser.parse() {
                Ok(p) => p,
                Err(e) => {
                    eprintln!("Parse error at {}:{}: {}", e.span.line, e.span.column, e.message);
                    std::process::exit(1);
                }
            };

            // Semantic analysis
            let base_dir = file.parent().unwrap_or_else(|| std::path::Path::new(".")).to_path_buf();
            let result = analyze_with_base_dir(&program, base_dir);
            if !result.errors.is_empty() {
                for error in &result.errors {
                    eprintln!(
                        "error[{}]: {} at {}:{}",
                        error.code, error.message, error.span.line, error.span.column
                    );
                }
                eprintln!("Build failed: {} error(s)", result.errors.len());
                std::process::exit(1);
            }

            // AST lowering (transform deep chains to pattern matching)
            lower(&mut program);

            // Determine target
            let target_enum = match target.as_str() {
                "babel" => Target::Babel,
                "swc" => Target::Swc,
                "both" => Target::Both,
                _ => {
                    eprintln!("Unknown target: {}. Use 'babel', 'swc', or 'both'", target);
                    std::process::exit(1);
                }
            };

            // Dump decorated AST if requested (SWC only) - skip codegen
            if dump_decorated_ast {
                if target_enum == Target::Babel {
                    eprintln!("Error: --dump-decorated-ast only works with --target swc");
                    std::process::exit(1);
                }

                use reluxscript::SwcDecorator;
                // Use semantic type environment for decoration
                let mut decorator = SwcDecorator::with_semantic_types(result.type_env);
                let decorated = decorator.decorate_program(&program);

                println!("\n=== DECORATED AST FOR SWC (BEFORE REWRITING) ===");
                println!("{:#?}", decorated);
                println!("=== END DECORATED AST ===\n");

                // Exit early - don't run codegen
                return;
            }

            // Dump rewritten AST if requested (SWC only) - skip codegen
            if dump_rewritten_ast {
                if target_enum == Target::Babel {
                    eprintln!("Error: --dump-rewritten-ast only works with --target swc");
                    std::process::exit(1);
                }

                use reluxscript::{SwcDecorator, SwcRewriter};
                // Use semantic type environment for decoration
                let mut decorator = SwcDecorator::with_semantic_types(result.type_env);
                let decorated = decorator.decorate_program(&program);

                // Rewrite the decorated AST
                let mut rewriter = SwcRewriter::new();
                let rewritten = rewriter.rewrite_program(decorated);

                println!("\n=== REWRITTEN AST FOR SWC (AFTER PATTERN DESUGARING) ===");
                println!("{:#?}", rewritten);
                println!("=== END REWRITTEN AST ===\n");

                // Exit early - don't run codegen
                return;
            }

            // Generate code (use generate_with_types to get proper SWC mappings)
            let generated = reluxscript::generate_with_types(&program, result.type_env.clone(), target_enum);

            // Create output directory
            if let Err(e) = fs::create_dir_all(&output) {
                eprintln!("Error creating output directory: {}", e);
                std::process::exit(1);
            }

            // Write generated files
            if let Some(babel_code) = generated.babel {
                let babel_path = output.join("index.js");
                if let Err(e) = fs::write(&babel_path, babel_code) {
                    eprintln!("Error writing Babel output: {}", e);
                    std::process::exit(1);
                }
                println!("Generated Babel plugin: {:?}", babel_path);

                // Validate generated JS syntax with node --check
                let node_check = std::process::Command::new("node")
                    .arg("--check")
                    .arg(&babel_path)
                    .output();

                match node_check {
                    Ok(output) if !output.status.success() => {
                        eprintln!("\n[VALIDATION ERROR] Generated Babel plugin has syntax errors:");
                        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                        eprintln!("\nCodegen produced invalid JavaScript. This is a compiler bug.");
                        std::process::exit(1);
                    }
                    Ok(_) => {
                        println!("✓ Babel output validated successfully");
                    }
                    Err(_) => {
                        // Node.js not available, skip validation with warning
                        eprintln!("Warning: Could not validate JS syntax (node not found)");
                    }
                }
            }

            if let Some(swc_code) = generated.swc {
                let swc_path = output.join("lib.rs");
                if let Err(e) = fs::write(&swc_path, &swc_code) {
                    eprintln!("Error writing SWC output: {}", e);
                    std::process::exit(1);
                }
                println!("Generated SWC plugin: {:?}", swc_path);

                // Generate a minimal Cargo.toml for validation
                let cargo_toml_path = output.join("Cargo.toml");
                let needs_cargo_toml = !cargo_toml_path.exists();

                if needs_cargo_toml {
                    let cargo_toml_content = r#"[package]
name = "swc-plugin-temp"
version = "0.1.0"
edition = "2021"

[lib]
path = "lib.rs"
crate-type = ["cdylib", "rlib"]

[dependencies]
swc_common = "17"
swc_ecma_ast = "18"
swc_ecma_visit = "18"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
"#;
                    if let Err(e) = fs::write(&cargo_toml_path, cargo_toml_content) {
                        eprintln!("Warning: Could not create Cargo.toml for validation: {}", e);
                    }
                }

                // Validate generated Rust code with cargo check
                let cargo_check = std::process::Command::new("cargo")
                    .arg("check")
                    .arg("--manifest-path")
                    .arg(&cargo_toml_path)
                    .arg("--lib")
                    .output();

                match cargo_check {
                    Ok(output) if !output.status.success() => {
                        eprintln!("\n[VALIDATION ERROR] Generated SWC plugin has compilation errors:");
                        eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                        eprintln!("\nCodegen produced invalid Rust code. This is a compiler bug.");

                        // Clean up temporary Cargo.toml if we created it
                        if needs_cargo_toml {
                            let _ = fs::remove_file(&cargo_toml_path);
                        }

                        std::process::exit(1);
                    }
                    Ok(_) => {
                        println!("✓ SWC output validated successfully");

                        // Clean up temporary Cargo.toml if we created it
                        if needs_cargo_toml {
                            let _ = fs::remove_file(&cargo_toml_path);
                        }
                    }
                    Err(_) => {
                        // Cargo not available, skip validation with warning
                        eprintln!("Warning: Could not validate Rust syntax (cargo not found)");

                        // Clean up temporary Cargo.toml if we created it
                        if needs_cargo_toml {
                            let _ = fs::remove_file(&cargo_toml_path);
                        }
                    }
                }
            }

            println!("Build complete!");
        }
        Commands::Fix { files, dry_run } => {
            let mut total_fixes = 0;
            let mut files_changed = 0;

            for file in &files {
                let source = match fs::read_to_string(file) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Error reading {:?}: {}", file, e);
                        continue;
                    }
                };

                // Tokenize
                let mut lexer = Lexer::new(&source);
                let tokens = lexer.tokenize();

                // Apply fixes
                let rewriter = TokenRewriter::new(tokens);
                let (fixed_tokens, fixes_applied) = rewriter.rewrite();

                if fixes_applied == 0 {
                    if files.len() == 1 {
                        println!("No fixes needed for {:?}", file);
                    }
                    continue;
                }

                files_changed += 1;
                total_fixes += fixes_applied;

                println!("{:?}: {} fix(es) applied", file, fixes_applied);

                if !dry_run {
                    // We need to regenerate source from tokens
                    // For now, we'll use a simple approach that works for our use case
                    // In a production system, you'd want a proper token-to-source converter

                    // Since we're rewriting if-let to match, we need to actually parse and
                    // verify it works, then write it back
                    // For simplicity, let's parse with the fixed tokens to verify it works
                    let mut parser = Parser::new_with_source(fixed_tokens, source.clone());

                    match parser.parse() {
                        Ok(_) => {
                            // The fix worked! But we can't write it back yet because
                            // we need a token-to-source converter
                            println!("  Warning: File NOT rewritten - token-to-source conversion not yet implemented");
                            println!("  The fixes would have been applied, but source regeneration is needed");
                        }
                        Err(e) => {
                            eprintln!("  Error: Fix validation failed: {}", e.message);
                            eprintln!("  File NOT modified");
                        }
                    }
                } else {
                    println!("  (dry-run: file not modified)");
                }
            }

            println!("\n{} file(s) processed, {} total fix(es)", files.len(), total_fixes);
            if files_changed > 0 {
                if dry_run {
                    println!("Run without --dry-run to apply changes");
                } else {
                    println!("Note: Actual file rewriting requires token-to-source conversion (not yet implemented)");
                    println!("Use --autofix with check/build commands to apply fixes during compilation");
                }
            }
        }
    }
}
