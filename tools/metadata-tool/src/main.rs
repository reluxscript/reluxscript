//! metadata-tool - CLI for modifying struct literal fields across a codebase
//!
//! Usage:
//!   metadata-tool add SwcExprMetadata needs_to_string false --path ./src
//!   metadata-tool rename SwcExprMetadata old_field new_field --path ./src
//!   metadata-tool remove SwcExprMetadata deprecated_field --path ./src

use clap::{Parser, Subcommand};
use proc_macro2::TokenStream;
use quote::ToTokens;
use std::fs;
use std::path::PathBuf;
use syn::visit_mut::VisitMut;
use syn::{parse_file, Expr, ExprStruct, FieldValue, Ident, Member};
use walkdir::WalkDir;

#[derive(Parser)]
#[command(name = "metadata-tool")]
#[command(about = "Modify struct literal fields across a Rust codebase")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Add a new field to all struct literals of a given type
    Add {
        /// Name of the struct (e.g., SwcExprMetadata)
        struct_name: String,
        /// Name of the field to add
        field_name: String,
        /// Default value for the field (e.g., "false", "None", "0")
        default_value: String,
        /// Path to search (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        /// Dry run - show what would be changed without modifying files
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Rename a field in all struct literals of a given type
    Rename {
        /// Name of the struct (e.g., SwcExprMetadata)
        struct_name: String,
        /// Current field name
        old_name: String,
        /// New field name
        new_name: String,
        /// Path to search (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        /// Dry run - show what would be changed without modifying files
        #[arg(short, long)]
        dry_run: bool,
    },
    /// Remove a field from all struct literals of a given type
    Remove {
        /// Name of the struct (e.g., SwcExprMetadata)
        struct_name: String,
        /// Name of the field to remove
        field_name: String,
        /// Path to search (defaults to current directory)
        #[arg(short, long, default_value = ".")]
        path: PathBuf,
        /// Dry run - show what would be changed without modifying files
        #[arg(short, long)]
        dry_run: bool,
    },
}

struct FieldAdder {
    struct_name: String,
    field_name: String,
    default_value: TokenStream,
    modified: bool,
}

impl VisitMut for FieldAdder {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        // First recurse into child expressions
        syn::visit_mut::visit_expr_mut(self, expr);

        // Then check if this is a struct literal we care about
        if let Expr::Struct(expr_struct) = expr {
            if self.is_target_struct(expr_struct) {
                // Check if field already exists
                let field_exists = expr_struct.fields.iter().any(|f| {
                    if let Member::Named(ident) = &f.member {
                        ident == &self.field_name
                    } else {
                        false
                    }
                });

                if !field_exists {
                    // Add the new field
                    let field_name = Ident::new(&self.field_name, proc_macro2::Span::call_site());
                    let default_value = self.default_value.clone();

                    let new_field: FieldValue = syn::parse_quote! {
                        #field_name: #default_value
                    };

                    expr_struct.fields.push(new_field);
                    self.modified = true;
                }
            }
        }
    }
}

impl FieldAdder {
    fn is_target_struct(&self, expr_struct: &ExprStruct) -> bool {
        // Get the last segment of the path (the struct name)
        if let Some(segment) = expr_struct.path.segments.last() {
            segment.ident == self.struct_name
        } else {
            false
        }
    }
}

struct FieldRenamer {
    struct_name: String,
    old_name: String,
    new_name: String,
    modified: bool,
}

impl VisitMut for FieldRenamer {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, expr);

        if let Expr::Struct(expr_struct) = expr {
            if self.is_target_struct(expr_struct) {
                for field in expr_struct.fields.iter_mut() {
                    if let Member::Named(ident) = &mut field.member {
                        if ident == &self.old_name {
                            *ident = Ident::new(&self.new_name, ident.span());
                            self.modified = true;
                        }
                    }
                }
            }
        }
    }
}

impl FieldRenamer {
    fn is_target_struct(&self, expr_struct: &ExprStruct) -> bool {
        if let Some(segment) = expr_struct.path.segments.last() {
            segment.ident == self.struct_name
        } else {
            false
        }
    }
}

struct FieldRemover {
    struct_name: String,
    field_name: String,
    modified: bool,
}

impl VisitMut for FieldRemover {
    fn visit_expr_mut(&mut self, expr: &mut Expr) {
        syn::visit_mut::visit_expr_mut(self, expr);

        if let Expr::Struct(expr_struct) = expr {
            if self.is_target_struct(expr_struct) {
                let original_len = expr_struct.fields.len();
                expr_struct.fields = expr_struct
                    .fields
                    .iter()
                    .filter(|f| {
                        if let Member::Named(ident) = &f.member {
                            ident != &self.field_name
                        } else {
                            true
                        }
                    })
                    .cloned()
                    .collect();

                if expr_struct.fields.len() != original_len {
                    self.modified = true;
                }
            }
        }
    }
}

impl FieldRemover {
    fn is_target_struct(&self, expr_struct: &ExprStruct) -> bool {
        if let Some(segment) = expr_struct.path.segments.last() {
            segment.ident == self.struct_name
        } else {
            false
        }
    }
}

fn process_file<V: VisitMut>(path: &PathBuf, visitor: &mut V) -> Result<(bool, String), String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read {}: {}", path.display(), e))?;

    let mut ast = parse_file(&content)
        .map_err(|e| format!("Failed to parse {}: {}", path.display(), e))?;

    visitor.visit_file_mut(&mut ast);

    let output = ast.to_token_stream().to_string();

    // Use prettyplease-style formatting by re-parsing and using quote
    // For now, just return the token stream output
    Ok((true, output))
}

fn find_rust_files(path: &PathBuf) -> Vec<PathBuf> {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().map_or(false, |ext| ext == "rs")
                && !e.path().to_string_lossy().contains("target")
        })
        .map(|e| e.path().to_path_buf())
        .collect()
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Add {
            struct_name,
            field_name,
            default_value,
            path,
            dry_run,
        } => {
            let default_tokens: TokenStream = default_value
                .parse()
                .expect("Failed to parse default value as tokens");

            let files = find_rust_files(&path);
            let mut total_modified = 0;

            for file_path in files {
                let mut adder = FieldAdder {
                    struct_name: struct_name.clone(),
                    field_name: field_name.clone(),
                    default_value: default_tokens.clone(),
                    modified: false,
                };

                match process_file(&file_path, &mut adder) {
                    Ok((_, output)) => {
                        if adder.modified {
                            total_modified += 1;
                            println!("Modified: {}", file_path.display());

                            if !dry_run {
                                if let Err(e) = fs::write(&file_path, output) {
                                    eprintln!("Failed to write {}: {}", file_path.display(), e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }

            if dry_run {
                println!("\nDry run: would modify {} files", total_modified);
            } else {
                println!("\nModified {} files", total_modified);
            }
        }

        Commands::Rename {
            struct_name,
            old_name,
            new_name,
            path,
            dry_run,
        } => {
            let files = find_rust_files(&path);
            let mut total_modified = 0;

            for file_path in files {
                let mut renamer = FieldRenamer {
                    struct_name: struct_name.clone(),
                    old_name: old_name.clone(),
                    new_name: new_name.clone(),
                    modified: false,
                };

                match process_file(&file_path, &mut renamer) {
                    Ok((_, output)) => {
                        if renamer.modified {
                            total_modified += 1;
                            println!("Modified: {}", file_path.display());

                            if !dry_run {
                                if let Err(e) = fs::write(&file_path, output) {
                                    eprintln!("Failed to write {}: {}", file_path.display(), e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }

            if dry_run {
                println!("\nDry run: would modify {} files", total_modified);
            } else {
                println!("\nModified {} files", total_modified);
            }
        }

        Commands::Remove {
            struct_name,
            field_name,
            path,
            dry_run,
        } => {
            let files = find_rust_files(&path);
            let mut total_modified = 0;

            for file_path in files {
                let mut remover = FieldRemover {
                    struct_name: struct_name.clone(),
                    field_name: field_name.clone(),
                    modified: false,
                };

                match process_file(&file_path, &mut remover) {
                    Ok((_, output)) => {
                        if remover.modified {
                            total_modified += 1;
                            println!("Modified: {}", file_path.display());

                            if !dry_run {
                                if let Err(e) = fs::write(&file_path, output) {
                                    eprintln!("Failed to write {}: {}", file_path.display(), e);
                                }
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: {}", e);
                    }
                }
            }

            if dry_run {
                println!("\nDry run: would modify {} files", total_modified);
            } else {
                println!("\nModified {} files", total_modified);
            }
        }
    }
}
