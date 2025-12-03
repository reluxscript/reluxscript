//! metadata-tool - CLI for modifying struct literal fields across a codebase
//!
//! Usage:
//!   metadata-tool add SwcExprMetadata needs_to_string false --path ./src
//!   metadata-tool rename SwcExprMetadata old_field new_field --path ./src
//!   metadata-tool remove SwcExprMetadata deprecated_field --path ./src

use clap::{Parser, Subcommand};
use regex::Regex;
use std::fs;
use std::path::PathBuf;
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

/// Find matching brace for a struct literal starting at `{`
fn find_matching_brace(content: &str, start: usize) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut depth = 0;
    let mut i = start;

    while i < bytes.len() {
        match bytes[i] {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            b'"' => {
                // Skip string literals
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == b'\\' {
                        i += 2;
                        continue;
                    }
                    if bytes[i] == b'"' {
                        break;
                    }
                    i += 1;
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

fn add_field(content: &str, struct_name: &str, field_name: &str, default_value: &str) -> (String, usize) {
    // Pattern to find struct literals: StructName { field: or StructName { field,
    // Must have word boundary before struct name to avoid matching "FooSwcExprMetadata"
    // Must have a field pattern after { to distinguish from return types like `-> Type {`
    // Handles both `field:` and shorthand `field,` syntax
    let pattern = format!(r"\b{}\s*\{{\s*[a-zA-Z_][a-zA-Z0-9_]*\s*[,:]", regex::escape(struct_name));
    let re = Regex::new(&pattern).unwrap();

    let mut result = content.to_string();
    let mut edit_count = 0;

    // Collect all edit positions first to avoid issues with overlapping
    let mut edits: Vec<(usize, String)> = Vec::new();

    for cap in re.find_iter(content) {
        let match_start = cap.start();
        let brace_pos = content[match_start..].find('{').unwrap() + match_start;

        if let Some(close_brace) = find_matching_brace(content, brace_pos) {
            // Check if field already exists
            let struct_content = &content[brace_pos..=close_brace];
            let field_check = format!(r"\b{}\s*:", regex::escape(field_name));
            let field_re = Regex::new(&field_check).unwrap();

            if field_re.is_match(struct_content) {
                continue; // Field already exists
            }

            // Check what's right before the closing brace
            let before_close = content[brace_pos..close_brace].trim_end();
            let needs_comma = !before_close.ends_with(',') && !before_close.ends_with('{');

            // Determine if single-line or multi-line
            let is_single_line = !struct_content.contains('\n');

            let insert_text = if is_single_line {
                if needs_comma {
                    format!(", {}: {}", field_name, default_value)
                } else if before_close.ends_with('{') || before_close.is_empty() {
                    format!(" {}: {} ", field_name, default_value)
                } else {
                    format!(" {}: {},", field_name, default_value)
                }
            } else {
                // Multi-line: detect indentation from the struct content
                let indent = detect_indent(content, close_brace);
                if needs_comma {
                    format!(",\n{}{}: {}", indent, field_name, default_value)
                } else {
                    format!("\n{}{}: {},", indent, field_name, default_value)
                }
            };

            edits.push((close_brace, insert_text));
        }
    }

    // Apply edits from end to start to preserve positions
    edits.sort_by(|a, b| b.0.cmp(&a.0));

    for (pos, text) in edits {
        result.insert_str(pos, &text);
        edit_count += 1;
    }

    (result, edit_count)
}

fn rename_field(content: &str, struct_name: &str, old_name: &str, new_name: &str) -> (String, usize) {
    let pattern = format!(r"{}(\s*)\{{", regex::escape(struct_name));
    let re = Regex::new(&pattern).unwrap();

    let mut result = content.to_string();
    let mut edit_count = 0;

    // Process each struct literal
    for cap in re.find_iter(content) {
        let match_start = cap.start();
        let brace_pos = content[match_start..].find('{').unwrap() + match_start;

        if let Some(close_brace) = find_matching_brace(content, brace_pos) {
            // We need to rename fields only within this struct's braces
            // For simplicity, do a targeted replacement
            let _struct_range = brace_pos..=close_brace;
        }
    }

    // Simpler approach: find pattern "StructName { ... old_name: ... }"
    // and rename old_name to new_name within struct context
    let field_pattern = format!(
        r"({}[^{{]*\{{[^}}]*)\b{}\s*:",
        regex::escape(struct_name),
        regex::escape(old_name)
    );
    let field_re = Regex::new(&field_pattern).unwrap();

    // Use a loop to handle all occurrences
    loop {
        if let Some(cap) = field_re.find(&result) {
            // Find the field name position
            let search_start = cap.start();
            let in_struct = &result[search_start..cap.end()];

            // Find old_name position within the match
            let field_name_re = Regex::new(&format!(r"\b{}\s*:", regex::escape(old_name))).unwrap();
            if let Some(field_match) = field_name_re.find(in_struct) {
                let abs_start = search_start + field_match.start();
                let abs_end = abs_start + old_name.len();

                result.replace_range(abs_start..abs_end, new_name);
                edit_count += 1;
                continue;
            }
        }
        break;
    }

    (result, edit_count)
}

fn remove_field(content: &str, struct_name: &str, field_name: &str) -> (String, usize) {
    let pattern = format!(r"{}(\s*)\{{", regex::escape(struct_name));
    let re = Regex::new(&pattern).unwrap();

    let mut result = content.to_string();
    let mut edit_count = 0;

    // Pattern to match field with value and trailing comma/whitespace
    // Handles: field_name: value, or field_name: value (at end)
    let field_patterns = [
        // Field with trailing comma and optional newline+indent
        format!(r",?\s*\n?\s*{}\s*:[^,}}]+,?", regex::escape(field_name)),
        // Field at start with trailing comma
        format!(r"{}\s*:[^,}}]+,\s*", regex::escape(field_name)),
    ];

    for cap in re.find_iter(&content) {
        let match_start = cap.start();
        let brace_pos = result[match_start..].find('{');
        if brace_pos.is_none() {
            continue;
        }
    }

    // Simpler: just remove the field pattern globally within struct contexts
    // Match: StructName { ... field_name: value, ... }

    loop {
        // Find a struct that contains our field
        let struct_pattern = format!(r"{}[^{{]*\{{", regex::escape(struct_name));
        let struct_re = Regex::new(&struct_pattern).unwrap();

        let mut found = false;

        if let Some(struct_match) = struct_re.find(&result) {
            let brace_pos = struct_match.end() - 1;

            if let Some(close_brace) = find_matching_brace(&result, brace_pos) {
                let struct_content = &result[brace_pos..=close_brace];

                // Check if field exists in this struct
                let field_check = Regex::new(&format!(r"\b{}\s*:", regex::escape(field_name))).unwrap();

                if field_check.is_match(struct_content) {
                    // Remove the field - try different patterns

                    // Pattern 1: field in middle with leading comma
                    let p1 = format!(r",\s*{}\s*:[^,\}}]+", regex::escape(field_name));
                    // Pattern 2: field at start with trailing comma
                    let p2 = format!(r"{}\s*:[^,\}}]+,\s*", regex::escape(field_name));
                    // Pattern 3: field at end (no trailing comma)
                    let p3 = format!(r",?\s*{}\s*:[^,\}}]+", regex::escape(field_name));

                    for pattern in [&p1, &p2, &p3] {
                        let field_re = Regex::new(pattern).unwrap();
                        if let Some(field_match) = field_re.find(struct_content) {
                            let abs_start = brace_pos + field_match.start();
                            let abs_end = brace_pos + field_match.end();
                            result.replace_range(abs_start..abs_end, "");
                            edit_count += 1;
                            found = true;
                            break;
                        }
                    }
                }
            }
        }

        if !found {
            break;
        }
    }

    (result, edit_count)
}

fn detect_indent(content: &str, pos: usize) -> String {
    // Look backwards from pos to find the indentation of the previous line
    let before = &content[..pos];

    // Find the last newline
    if let Some(newline_pos) = before.rfind('\n') {
        let line = &before[newline_pos + 1..];
        // Extract leading whitespace
        let indent: String = line.chars().take_while(|c| c.is_whitespace()).collect();
        if !indent.is_empty() {
            return indent;
        }
    }

    "    ".to_string()
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
            let mut total_edits = 0;

            for file_path in files {
                let content = match fs::read_to_string(&file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let (output, edit_count) = add_field(&content, &struct_name, &field_name, &default_value);

                if edit_count > 0 {
                    total_modified += 1;
                    total_edits += edit_count;
                    println!("Modified: {} ({} edits)", file_path.display(), edit_count);

                    if !dry_run {
                        if let Err(e) = fs::write(&file_path, output) {
                            eprintln!("Failed to write {}: {}", file_path.display(), e);
                        }
                    }
                }
            }

            println!("\n{} {} files ({} total edits)",
                if dry_run { "Would modify" } else { "Modified" },
                total_modified,
                total_edits
            );
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
            let mut total_edits = 0;

            for file_path in files {
                let content = match fs::read_to_string(&file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let (output, edit_count) = rename_field(&content, &struct_name, &old_name, &new_name);

                if edit_count > 0 {
                    total_modified += 1;
                    total_edits += edit_count;
                    println!("Modified: {} ({} edits)", file_path.display(), edit_count);

                    if !dry_run {
                        if let Err(e) = fs::write(&file_path, output) {
                            eprintln!("Failed to write {}: {}", file_path.display(), e);
                        }
                    }
                }
            }

            println!("\n{} {} files ({} total edits)",
                if dry_run { "Would modify" } else { "Modified" },
                total_modified,
                total_edits
            );
        }

        Commands::Remove {
            struct_name,
            field_name,
            path,
            dry_run,
        } => {
            let files = find_rust_files(&path);
            let mut total_modified = 0;
            let mut total_edits = 0;

            for file_path in files {
                let content = match fs::read_to_string(&file_path) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Failed to read {}: {}", file_path.display(), e);
                        continue;
                    }
                };

                let (output, edit_count) = remove_field(&content, &struct_name, &field_name);

                if edit_count > 0 {
                    total_modified += 1;
                    total_edits += edit_count;
                    println!("Modified: {} ({} edits)", file_path.display(), edit_count);

                    if !dry_run {
                        if let Err(e) = fs::write(&file_path, output) {
                            eprintln!("Failed to write {}: {}", file_path.display(), e);
                        }
                    }
                }
            }

            println!("\n{} {} files ({} total edits)",
                if dry_run { "Would modify" } else { "Modified" },
                total_modified,
                total_edits
            );
        }
    }
}
