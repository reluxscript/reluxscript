/// Test deep chain lowering
///
/// This tests the auto-unwrap chain transformation.
/// The expression `member.property.name` should be lowered to
/// explicit pattern matching.

plugin DeepChainTest {
    /// Extract property name from a member expression
    pub fn get_property_name(member: &MemberExpression) -> Str {
        // This deep chain should be auto-lowered:
        // member.property -> MemberProp (WrapperEnum)
        // .name -> needs unwrap to MemberProp::Ident
        let name = member.property.name.clone();
        return name;
    }

    /// Another test with matches! guard
    pub fn get_property_name_safe(member: &MemberExpression) -> Str {
        let property = member.property.clone();
        if matches!(property, Identifier) {
            let name = property.name.clone();
            return name;
        }
        return "unknown";
    }
}
