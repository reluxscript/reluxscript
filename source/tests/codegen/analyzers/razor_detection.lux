/**
 * Razor Syntax Detection and Variable Extraction
 *
 * Detects Razor-style syntax in markdown strings and extracts referenced variables.
 *
 * Supported Razor syntax:
 * - Variable references: @variableName, @variable.Property
 * - Inline expressions: @(expression)
 * - Conditionals: @if (condition) { ... } else { ... }
 * - Loops: @foreach (var item in items) { ... }
 * - For loops: @for (var i = 0; i < count; i++) { ... }
 * - Switch expressions: @switch (value) { case ...: ... }
 */

/**
 * Check if markdown string contains Razor syntax
 *
 * @param markdown - Markdown content to check
 * @returns True if Razor syntax detected
 */
pub fn contains_razor_syntax(markdown: &Str) -> bool {
    if markdown.is_empty() {
        return false;
    }

    // Razor patterns to detect:
    // 1. @variableName or @variable.Property (but not email addresses)
    // 2. @if, @foreach, @for, @switch, @while
    // 3. @(expression)

    // Check for @if, @foreach, etc.
    if markdown.contains("@if (") || markdown.contains("@foreach (") ||
       markdown.contains("@for (") || markdown.contains("@switch (") ||
       markdown.contains("@while (") {
        return true;
    }

    // Check for @(expression)
    if markdown.contains("@(") {
        return true;
    }

    // Check for @variableName pattern (letter after @)
    has_variable_pattern(markdown)
}

/**
 * Check if string has @variable pattern
 */
fn has_variable_pattern(s: &Str) -> bool {
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '@' && i + 1 < chars.len() {
            let next_char = chars[i + 1];
            // Check if next character is a letter or underscore
            if next_char.is_alphabetic() || next_char == '_' {
                return true;
            }
        }
        i += 1;
    }

    false
}

/**
 * Extract all variable references from Razor markdown
 *
 * Returns root variable names (before any '.' property access)
 * Excludes Razor keywords (if, else, foreach, etc.)
 *
 * @param markdown - Markdown content with Razor syntax
 * @returns Set of variable names referenced
 */
pub fn extract_razor_variables(markdown: &Str) -> HashSet<Str> {
    if markdown.is_empty() {
        return HashSet::new();
    }

    let mut variables = HashSet::new();
    let razor_keywords = get_razor_keywords();

    // Extract from different patterns
    extract_simple_variables(markdown, &razor_keywords, &mut variables);
    extract_expression_variables(markdown, &razor_keywords, &mut variables);
    extract_if_variables(markdown, &razor_keywords, &mut variables);
    extract_foreach_variables(markdown, &razor_keywords, &mut variables);
    extract_for_variables(markdown, &razor_keywords, &mut variables);
    extract_switch_variables(markdown, &razor_keywords, &mut variables);

    variables
}

/**
 * Get set of Razor keywords to exclude
 */
fn get_razor_keywords() -> HashSet<Str> {
    let keywords = vec![
        "if", "else", "foreach", "for", "while", "switch",
        "case", "default", "break", "var"
    ];

    let mut set = HashSet::new();
    for keyword in keywords {
        set.insert(keyword.to_string());
    }
    set
}

/**
 * Extract simple @variable or @variable.Property references
 */
fn extract_simple_variables(markdown: &Str, keywords: &HashSet<Str>, variables: &mut HashSet<Str>) {
    let chars: Vec<char> = markdown.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i] == '@' && i + 1 < chars.len() {
            let next_char = chars[i + 1];

            if next_char.is_alphabetic() || next_char == '_' {
                // Extract the identifier
                let mut var_name = String::new();
                let mut j = i + 1;

                while j < chars.len() {
                    let ch = chars[j];
                    if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                        var_name.push(ch);
                        j += 1;
                    } else {
                        break;
                    }
                }

                // Get root variable (before first '.')
                let root_var = var_name.split('.').next().unwrap_or("").to_string();

                if !root_var.is_empty() && !keywords.contains(&root_var) {
                    variables.insert(root_var);
                }

                i = j;
            } else {
                i += 1;
            }
        } else {
            i += 1;
        }
    }
}

/**
 * Extract variables from @(expression)
 */
fn extract_expression_variables(markdown: &Str, keywords: &HashSet<Str>, variables: &mut HashSet<Str>) {
    // Find all @(...)
    let chars: Vec<char> = markdown.chars().collect();
    let mut i = 0;

    while i < chars.len() - 1 {
        if chars[i] == '@' && chars[i + 1] == '(' {
            // Find matching closing paren
            let mut depth = 1;
            let mut j = i + 2;
            let mut expression = String::new();

            while j < chars.len() && depth > 0 {
                if chars[j] == '(' {
                    depth += 1;
                } else if chars[j] == ')' {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                }
                expression.push(chars[j]);
                j += 1;
            }

            // Extract identifiers from expression
            extract_identifiers_from_expression(&expression, keywords, variables);
            i = j;
        } else {
            i += 1;
        }
    }
}

/**
 * Extract identifiers from an expression string
 */
fn extract_identifiers_from_expression(expr: &Str, keywords: &HashSet<Str>, variables: &mut HashSet<Str>) {
    let chars: Vec<char> = expr.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        if chars[i].is_alphabetic() || chars[i] == '_' {
            let mut identifier = String::new();
            let mut j = i;

            while j < chars.len() && (chars[j].is_alphanumeric() || chars[j] == '_') {
                identifier.push(chars[j]);
                j += 1;
            }

            if !is_csharp_keyword(&identifier) && !keywords.contains(&identifier) {
                variables.insert(identifier);
            }

            i = j;
        } else {
            i += 1;
        }
    }
}

/**
 * Extract variables from @if (condition)
 */
fn extract_if_variables(markdown: &Str, keywords: &HashSet<Str>, variables: &mut HashSet<Str>) {
    // Find @if (...) patterns
    if let Some(start) = markdown.find("@if (") {
        let rest = &markdown[start + 5..];
        if let Some(end) = rest.find(')') {
            let condition = &rest[0..end];
            extract_identifiers_from_expression(condition, keywords, variables);
        }
    }
}

/**
 * Extract variables from @foreach (var item in collection)
 */
fn extract_foreach_variables(markdown: &Str, keywords: &HashSet<Str>, variables: &mut HashSet<Str>) {
    // Find @foreach (...) patterns
    if let Some(start) = markdown.find("@foreach (") {
        let rest = &markdown[start + 10..];
        if let Some(end) = rest.find(')') {
            let loop_expr = &rest[0..end];

            // Extract collection after "in"
            if let Some(in_pos) = loop_expr.find(" in ") {
                let collection = &loop_expr[in_pos + 4..].trim();
                let root_var = collection.split('.').next().unwrap_or("").to_string();

                if !root_var.is_empty() && !keywords.contains(&root_var) {
                    variables.insert(root_var);
                }
            }
        }
    }
}

/**
 * Extract variables from @for (var i = 0; i < count; i++)
 */
fn extract_for_variables(markdown: &Str, keywords: &HashSet<Str>, variables: &mut HashSet<Str>) {
    // Find @for (...) patterns
    if let Some(start) = markdown.find("@for (") {
        let rest = &markdown[start + 6..];
        if let Some(end) = rest.find(')') {
            let for_expr = &rest[0..end];
            // Skip common loop variables like i, j, k
            let skip_vars = vec!["i", "j", "k"];

            extract_identifiers_from_expression(for_expr, keywords, variables);

            // Remove common loop variables
            for var in skip_vars {
                variables.remove(var);
            }
        }
    }
}

/**
 * Extract variables from @switch (expression)
 */
fn extract_switch_variables(markdown: &Str, keywords: &HashSet<Str>, variables: &mut HashSet<Str>) {
    // Find @switch (...) patterns
    if let Some(start) = markdown.find("@switch (") {
        let rest = &markdown[start + 9..];
        if let Some(end) = rest.find(')') {
            let switch_expr = &rest[0..end];
            extract_identifiers_from_expression(switch_expr, keywords, variables);
        }
    }
}

/**
 * Check if identifier is a C# keyword
 */
pub fn is_csharp_keyword(identifier: &Str) -> bool {
    let keywords = vec![
        "abstract", "as", "base", "bool", "break", "byte", "case", "catch", "char",
        "checked", "class", "const", "continue", "decimal", "default", "delegate",
        "do", "double", "else", "enum", "event", "explicit", "extern", "false",
        "finally", "fixed", "float", "for", "foreach", "goto", "if", "implicit",
        "in", "int", "interface", "internal", "is", "lock", "long", "namespace",
        "new", "null", "object", "operator", "out", "override", "params", "private",
        "protected", "public", "readonly", "ref", "return", "sbyte", "sealed",
        "short", "sizeof", "stackalloc", "static", "string", "struct", "switch",
        "this", "throw", "true", "try", "typeof", "uint", "ulong", "unchecked",
        "unsafe", "ushort", "using", "virtual", "void", "volatile", "while",
        // C# contextual keywords
        "add", "and", "alias", "ascending", "async", "await", "by", "descending",
        "dynamic", "equals", "from", "get", "global", "group", "into", "join",
        "let", "nameof", "not", "notnull", "on", "or", "orderby", "partial",
        "record", "remove", "select", "set", "unmanaged", "value", "var", "when",
        "where", "with", "yield"
    ];

    keywords.contains(&identifier)
}
