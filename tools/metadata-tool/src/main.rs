//! metadata-tool - CLI for modifying struct literal fields across a codebase
//!
//! Usage:
//!   metadata-tool add SwcExprMetadata needs_to_string false --path ./src
//!   metadata-tool rename SwcExprMetadata old_field new_field --path ./src
//!   metadata-tool remove SwcExprMetadata deprecated_field --path ./src
//!
//! This tool uses span-based text manipulation to preserve formatting.

use clap::{Parser, Subcommand};
use std::fs;
use std::path::PathBuf;
use syn::spanned::Spanned;
use syn::visit::Visit;
use syn::{parse_file, Expr, ExprStruct, Member};
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

/// Represents an edit to be applied to the source
#[derive(Debug, Clone)]
enum Edit {
    /// Insert text at a byte offset
    Insert { offset: usize, text: String },
    /// Replace a range with new text
    Replace { start: usize, end: usize, text: String },
    /// Delete a range
    Delete { start: usize, end: usize },
}

/// Collects insertion points for adding fields
struct FieldAddCollector<'a> {
    struct_name: &'a str,
    field_name: &'a str,
    default_value: &'a str,
    source: &'a str,
    edits: Vec<Edit>,
}

impl<'ast, 'a> Visit<'ast> for FieldAddCollector<'a> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        // First recurse into child expressions
        syn::visit::visit_expr(self, expr);

        // Then check if this is a struct literal we care about
        if let Expr::Struct(expr_struct) = expr {
            if self.is_target_struct(expr_struct) {
                // Check if field already exists
                let field_exists = expr_struct.fields.iter().any(|f| {
                    if let Member::Named(ident) = &f.member {
                        ident == self.field_name
                    } else {
                        false
                    }
                });

                if !field_exists {
                    // Find insertion point - right before the closing brace
                    let span = expr_struct.brace_token.span.close();
                    let close_brace_offset = span.start().column;

                    // We need byte offset, not column. Use the span's byte range.
                    // The closing brace is at the end of the struct expression
                    let struct_span = expr_struct.span();
                    let end_offset = struct_span.end().column;

                    // Find the actual closing brace by searching backwards from end
                    let struct_text = &self.source[..];

                    // Get byte offset from proc_macro2 span
                    // Note: proc_macro2 spans in non-proc-macro context give line/column
                    // We need to convert to byte offset
                    let close_line = expr_struct.brace_token.span.close().end().line;
                    let close_col = expr_struct.brace_token.span.close().end().column;

                    if let Some(offset) = line_col_to_offset(self.source, close_line, close_col) {
                        // Check if there's already a trailing comma
                        let before_brace = self.source[..offset].trim_end();
                        let needs_comma = !before_brace.ends_with(',') && !expr_struct.fields.is_empty();

                        // Determine indentation by looking at existing fields
                        let indent = detect_field_indent(self.source, offset);

                        let insert_text = if needs_comma {
                            format!(",\n{}{}: {}", indent, self.field_name, self.default_value)
                        } else if expr_struct.fields.is_empty() {
                            format!("\n{}{}: {}\n", indent, self.field_name, self.default_value)
                        } else {
                            format!("\n{}{}: {},", indent, self.field_name, self.default_value)
                        };

                        self.edits.push(Edit::Insert {
                            offset,
                            text: insert_text,
                        });
                    }
                }
            }
        }
    }
}

impl<'a> FieldAddCollector<'a> {
    fn is_target_struct(&self, expr_struct: &ExprStruct) -> bool {
        if let Some(segment) = expr_struct.path.segments.last() {
            segment.ident == self.struct_name
        } else {
            false
        }
    }
}

/// Collects rename locations
struct FieldRenameCollector<'a> {
    struct_name: &'a str,
    old_name: &'a str,
    new_name: &'a str,
    source: &'a str,
    edits: Vec<Edit>,
}

impl<'ast, 'a> Visit<'ast> for FieldRenameCollector<'a> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        syn::visit::visit_expr(self, expr);

        if let Expr::Struct(expr_struct) = expr {
            if self.is_target_struct(expr_struct) {
                for field in &expr_struct.fields {
                    if let Member::Named(ident) = &field.member {
                        if ident == self.old_name {
                            let span = ident.span();
                            let start_line = span.start().line;
                            let start_col = span.start().column;
                            let end_line = span.end().line;
                            let end_col = span.end().column;

                            if let (Some(start), Some(end)) = (
                                line_col_to_offset(self.source, start_line, start_col),
                                line_col_to_offset(self.source, end_line, end_col),
                            ) {
                                self.edits.push(Edit::Replace {
                                    start,
                                    end,
                                    text: self.new_name.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<'a> FieldRenameCollector<'a> {
    fn is_target_struct(&self, expr_struct: &ExprStruct) -> bool {
        if let Some(segment) = expr_struct.path.segments.last() {
            segment.ident == self.struct_name
        } else {
            false
        }
    }
}

/// Collects removal locations
struct FieldRemoveCollector<'a> {
    struct_name: &'a str,
    field_name: &'a str,
    source: &'a str,
    edits: Vec<Edit>,
}

impl<'ast, 'a> Visit<'ast> for FieldRemoveCollector<'a> {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        syn::visit::visit_expr(self, expr);

        if let Expr::Struct(expr_struct) = expr {
            if self.is_target_struct(expr_struct) {
                for (i, field) in expr_struct.fields.iter().enumerate() {
                    if let Member::Named(ident) = &field.member {
                        if ident == self.field_name {
                            // Find the full field range including trailing comma
                            let field_span = field.span();
                            let start_line = field_span.start().line;
                            let start_col = field_span.start().column;
                            let end_line = field_span.end().line;
                            let end_col = field_span.end().column;

                            if let (Some(start), Some(end)) = (
                                line_col_to_offset(self.source, start_line, start_col),
                                line_col_to_offset(self.source, end_line, end_col),
                            ) {
                                // Extend to include trailing comma and whitespace
                                let mut delete_end = end;
                                let rest = &self.source[end..];
                                let trimmed = rest.trim_start();
                                if trimmed.starts_with(',') {
                                    delete_end = end + (rest.len() - trimmed.len()) + 1;
                                    // Also consume newline after comma if present
                                    let after_comma = &self.source[delete_end..];
                                    if after_comma.starts_with('\n') {
                                        delete_end += 1;
                                    } else if after_comma.starts_with("\r\n") {
                                        delete_end += 2;
                                    }
                                }

                                // Handle leading whitespace for cleaner removal
                                let mut delete_start = start;
                                if i > 0 {
                                    // Find start of line and include indentation
                                    let before = &self.source[..start];
                                    if let Some(newline_pos) = before.rfind('\n') {
                                        let line_start = newline_pos + 1;
                                        if self.source[line_start..start].trim().is_empty() {
                                            delete_start = line_start;
                                        }
                                    }
                                }

                                self.edits.push(Edit::Delete {
                                    start: delete_start,
                                    end: delete_end,
                                });
                            }
                        }
                    }
                }
            }
        }
    }
}

impl<'a> FieldRemoveCollector<'a> {
    fn is_target_struct(&self, expr_struct: &ExprStruct) -> bool {
        if let Some(segment) = expr_struct.path.segments.last() {
            segment.ident == self.struct_name
        } else {
            false
        }
    }
}

/// Convert line/column (1-indexed line, 0-indexed column) to byte offset
fn line_col_to_offset(source: &str, line: usize, col: usize) -> Option<usize> {
    let mut current_line = 1;
    let mut line_start = 0;

    for (i, c) in source.char_indices() {
        if current_line == line {
            // Found the line, now add column offset
            // But we need to count chars, not bytes for the column
            let line_content = &source[line_start..];
            let mut char_count = 0;
            for (j, _) in line_content.char_indices() {
                if char_count == col {
                    return Some(line_start + j);
                }
                char_count += 1;
            }
            // Column might be at end of line
            if char_count == col {
                return Some(line_start + line_content.len());
            }
            return None;
        }
        if c == '\n' {
            current_line += 1;
            line_start = i + 1;
        }
    }

    // Handle last line
    if current_line == line {
        let line_content = &source[line_start..];
        let mut char_count = 0;
        for (j, _) in line_content.char_indices() {
            if char_count == col {
                return Some(line_start + j);
            }
            char_count += 1;
        }
        if char_count == col {
            return Some(source.len());
        }
    }

    None
}

/// Detect indentation used for fields in the struct
fn detect_field_indent(source: &str, close_brace_offset: usize) -> String {
    // Look backwards for the last field's indentation
    let before = &source[..close_brace_offset];

    // Find the last newline before the closing brace
    if let Some(newline_pos) = before.rfind('\n') {
        let line = &before[newline_pos + 1..];
        // Extract leading whitespace
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        if !indent.is_empty() {
            return indent;
        }
    }

    // Default indentation
    "    ".to_string()
}

/// Apply edits to source, processing from end to start to preserve offsets
fn apply_edits(source: &str, mut edits: Vec<Edit>) -> String {
    // Sort edits by position, descending (process from end to preserve offsets)
    edits.sort_by(|a, b| {
        let pos_a = match a {
            Edit::Insert { offset, .. } => *offset,
            Edit::Replace { start, .. } => *start,
            Edit::Delete { start, .. } => *start,
        };
        let pos_b = match b {
            Edit::Insert { offset, .. } => *offset,
            Edit::Replace { start, .. } => *start,
            Edit::Delete { start, .. } => *start,
        };
        pos_b.cmp(&pos_a)
    });

    let mut result = source.to_string();

    for edit in edits {
        match edit {
            Edit::Insert { offset, text } => {
                result.insert_str(offset, &text);
            }
            Edit::Replace { start, end, text } => {
                result.replace_range(start..end, &text);
            }
            Edit::Delete { start, end } => {
                result.replace_range(start..end, "");
            }
        }
    }

    result
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
            let files = find_rust_files(&path);
            let mut total_modified = 0;

            for file_path in files {
                let content = match fs::read_to_string(&file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let ast = match parse_file(&content) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let mut collector = FieldAddCollector {
                    struct_name: &struct_name,
                    field_name: &field_name,
                    default_value: &default_value,
                    source: &content,
                    edits: Vec::new(),
                };

                collector.visit_file(&ast);

                if !collector.edits.is_empty() {
                    total_modified += 1;
                    println!("Modified: {} ({} edits)", file_path.display(), collector.edits.len());

                    if !dry_run {
                        let output = apply_edits(&content, collector.edits);
                        if let Err(e) = fs::write(&file_path, output) {
                            eprintln!("Failed to write {}: {}", file_path.display(), e);
                        }
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
                let content = match fs::read_to_string(&file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let ast = match parse_file(&content) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let mut collector = FieldRenameCollector {
                    struct_name: &struct_name,
                    old_name: &old_name,
                    new_name: &new_name,
                    source: &content,
                    edits: Vec::new(),
                };

                collector.visit_file(&ast);

                if !collector.edits.is_empty() {
                    total_modified += 1;
                    println!("Modified: {} ({} edits)", file_path.display(), collector.edits.len());

                    if !dry_run {
                        let output = apply_edits(&content, collector.edits);
                        if let Err(e) = fs::write(&file_path, output) {
                            eprintln!("Failed to write {}: {}", file_path.display(), e);
                        }
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
                let content = match fs::read_to_string(&file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let ast = match parse_file(&content) {
                    Ok(a) => a,
                    Err(e) => {
                        eprintln!("Failed to parse {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let mut collector = FieldRemoveCollector {
                    struct_name: &struct_name,
                    field_name: &field_name,
                    source: &content,
                    edits: Vec::new(),
                };

                collector.visit_file(&ast);

                if !collector.edits.is_empty() {
                    total_modified += 1;
                    println!("Modified: {} ({} edits)", file_path.display(), collector.edits.len());

                    if !dry_run {
                        let output = apply_edits(&content, collector.edits);
                        if let Err(e) = fs::write(&file_path, output) {
                            eprintln!("Failed to write {}: {}", file_path.display(), e);
                        }
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
