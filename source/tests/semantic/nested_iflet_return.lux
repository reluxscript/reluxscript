/// Test: Nested if-let with returns

plugin NestedIfLetTest {
    fn test_nested(opt: Option<Pattern>) -> Str {
        // Nested if-let like in minimact
        let name = if let Some(ref elem) = opt {
            if let Pattern::Identifier(ref id) = elem {
                id.name.clone()
            } else {
                return "error".to_string();
            }
        } else {
            return "none".to_string();
        };

        name  // Should be Str, not ()
    }
}
