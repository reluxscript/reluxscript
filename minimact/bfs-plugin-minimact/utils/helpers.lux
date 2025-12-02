/**
 * Utility Helpers
 *
 * General utility functions used throughout the plugin.
 *
 * Functions:
 * - escape_csharp_string(str) - Escapes special characters for C# strings
 * - get_component_name(node, parent) - Extracts component name from function/class declaration
 *
 * Utilities:
 * - escape_csharp_string: Handles \, ", \n, \r, \t escaping
 * - get_component_name: Supports FunctionDeclaration, ArrowFunctionExpression, etc.
 *
 * Returns processed string or component name
 */

/**
 * Escape C# string
 * Handles special characters: \, ", \n, \r, \t
 */
pub fn escape_csharp_string(s: &Str) -> Str {
    s.replace("\\", "\\\\")
     .replace("\"", "\\\"")
     .replace("\n", "\\n")
     .replace("\r", "\\r")
     .replace("\t", "\\t")
}

/**
 * Get component name from function declaration
 * Supports:
 * - Direct function declarations with id
 * - Variable declarators
 * - Export named declarations
 */
pub fn get_component_name(node: &FunctionDeclaration, parent: Option<&Statement>) -> Option<Str> {
    // Check if node has an id (name)
    if let Some(ref id) = node.id {
        return Some(id.name.clone());
    }

    // Check parent context if provided
    if let Some(parent_stmt) = parent {
        // Check if parent is a VariableDeclarator
        if let Statement::VariableDeclaration(ref var_decl) = parent_stmt {
            if !var_decl.declarations.is_empty() {
                if let Some(ref first_decl) = var_decl.declarations.get(0) {
                    if let Pattern::Identifier(ref id) = first_decl.id {
                        return Some(id.name.clone());
                    }
                }
            }
        }

        // Check if parent is ExportNamedDeclaration
        if let Statement::ExportNamedDeclaration(ref export) = parent_stmt {
            if let Some(ref decl) = export.declaration {
                if let Declaration::FunctionDeclaration(ref func) = decl {
                    if let Some(ref id) = func.id {
                        return Some(id.name.clone());
                    }
                }
            }
        }
    }

    None
}

/**
 * Check if a name is a component name (starts with uppercase letter)
 */
pub fn is_component_name(name: &Str) -> bool {
    if name.is_empty() {
        return false;
    }

    if let Some(first_char) = name.chars().next() {
        first_char.is_uppercase()
    } else {
        false
    }
}
