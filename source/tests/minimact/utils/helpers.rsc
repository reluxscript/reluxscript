// Utility Helpers
//
// General utility functions used throughout the plugin.

/// Escape special characters for C# string literals
pub fn escape_csharp_string(s: &Str) -> Str {
    s.replace("\\", "\\\\")
     .replace("\"", "\\\"")
     .replace("\n", "\\n")
     .replace("\r", "\\r")
     .replace("\t", "\\t")
}

/// Get component name from a function declaration node
/// Supports FunctionDeclaration with id, and arrow functions in VariableDeclarator
pub fn get_component_name(node: &FunctionDeclaration) -> Option<Str> {
    // Direct function declaration: function Component() {}
    if let Some(ref id) = node.id {
        return Some(id.name.clone());
    }

    None
}

/// Get component name from variable declarator (for arrow functions)
/// const Component = () => {}
pub fn get_component_name_from_declarator(decl: &VariableDeclarator) -> Option<Str> {
    // Check if id is an Identifier
    if let Pattern::Ident(ref name) = decl.id {
        return Some(name.clone());
    }

    None
}

/// Check if a name follows PascalCase (first letter uppercase)
/// Used to detect React components
pub fn is_component_name(name: &Str) -> bool {
    if name.is_empty() {
        return false;
    }

    // Get first character
    let first_char = name.chars().next();
    if let Some(c) = first_char {
        return c.is_uppercase();
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_escape_csharp_string() {
        let input = "Hello \"World\"\n\tTest\\Path";
        let expected = "Hello \\\"World\\\"\\n\\tTest\\\\Path";
        let result = escape_csharp_string(&input);
        // assert_eq!(result, expected);
    }

    fn test_is_component_name() {
        // assert!(is_component_name(&"Component"));
        // assert!(is_component_name(&"MyButton"));
        // assert!(!is_component_name(&"useState"));
        // assert!(!is_component_name(&"handleClick"));
        // assert!(!is_component_name(&""));
    }
}
