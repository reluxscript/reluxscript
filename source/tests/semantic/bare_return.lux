/// Test: Bare return (return without value)

plugin BareReturnTest {
    fn outer_function() {
        let name = if let Some(ref value) = get_option() {
            if matches_pattern(value) {
                extract_name(value)
            } else {
                return;  // Bare return - returns from outer_function
            }
        } else {
            return;  // Bare return - returns from outer_function
        };

        use_name(name);  // name should be Str, not ()
    }

    fn get_option() -> Option<Str> {
        Some("test".to_string())
    }

    fn matches_pattern(s: &Str) -> bool {
        true
    }

    fn extract_name(s: &Str) -> Str {
        s.clone()
    }

    fn use_name(s: Str) {
        // Use the name
    }
}
