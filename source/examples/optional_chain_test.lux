/// Test optional chaining syntax
///
/// This tests the ?. operator for safe property access.

plugin OptionalChainTest {
    /// Safely get a property that might not exist
    pub fn safe_get_name(node: &MemberExpression) -> Str {
        // Use optional chaining for safe access on member expression
        let prop = node.property.clone();
        let name = prop?.name.clone();
        return name;
    }

    /// Regular access for comparison
    pub fn regular_get_name(node: &MemberExpression) -> Str {
        let name = node.property.name.clone();
        return name;
    }
}
