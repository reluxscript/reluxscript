/// Extract interface property names from TSX
///
/// This plugin demonstrates ReluxScript's ability to:
/// 1. Visit TSInterfaceDeclaration nodes
/// 2. Extract property names
/// 3. Return a list of strings

plugin InterfaceExtractor {
    /// Main visitor function for interface declarations
    /// Returns a joined string of property names from the interface
    pub fn visit_interface_declaration(node: &TSInterfaceDeclaration) -> Str {
        let mut parts = vec![];

        // Get interface name
        let interface_name = node.id.name.clone();
        parts.push(interface_name);

        // Iterate over interface body members
        for member in &node.body.body {
            // Check if this is a property signature
            if matches!(member, TSPropertySignature) {
                let prop_name = member.key.name.clone();
                parts.push(prop_name);
            }
        }

        return parts.join(",");
    }

    /// Extract useState call information
    /// Returns the callee name if it's a useState call
    pub fn visit_call_expression(node: &CallExpression) -> Str {
        // Check if this is a useState call
        if matches!(node.callee, Identifier) {
            let callee_name = node.callee.name.clone();
            if callee_name == "useState" {
                return callee_name;
            }
        }
        return "";
    }
}
