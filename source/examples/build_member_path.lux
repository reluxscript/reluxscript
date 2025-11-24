/// Build member expression path
///
/// Takes a MemberExpression and returns the dot-separated path string.
/// Example: console.log → "console.log"

plugin BuildMemberPath {
    /// Build member expression path from an expression
    pub fn build_member_path(expr: &Expr) -> Str {
        let mut parts = vec![];
        let mut current = expr.clone();

        // Walk up the member expression chain
        while matches!(current, MemberExpression) {
            let member = current.clone();
            let property = member.property.clone();
            let object = member.object.clone();

            // Extract property name if it's an identifier
            if matches!(property, Identifier) {
                let name = property.name.clone();
                parts.insert(0, name);
            }
            current = object;
        }

        // Handle the base identifier
        if matches!(current, Identifier) {
            let name = current.name.clone();
            parts.insert(0, name);
        }

        // Join with dots
        return parts.join(".");
    }
}
