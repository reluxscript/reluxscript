/// Test nested if-let with diverging - the exact minimact pattern

plugin NestedIfLetDivergeTest {
    fn test(elements: &Vec<Option<Pattern>>) -> Str {
        // This is the exact pattern from minimact that fails
        let state_var: Str = if let Some(ref elem) = elements[0] {
            if let Pattern::Identifier(ref id) = elem {
                id.name.clone()
            } else {
                return "invalid".to_string();
            }
        } else {
            return "empty".to_string();
        };

        state_var
    }
}
