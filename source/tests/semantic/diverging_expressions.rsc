/// Test: Diverging expressions should not affect type inference
///
/// When an if/else branch diverges (return, break, continue, panic!),
/// the type of that branch should be the "never" type, and the overall
/// expression type should be determined by the non-diverging branches.
///
/// Current bug: RS003 type mismatch - thinks expression returns ()

plugin DivergingExpressionTest {

    /// Test 1: if-let with diverging else
    fn test_if_let_diverging_else() -> Str {
        let opt = Some("value".to_string());

        // This should infer as Str, not ()
        let result: Str = if let Some(val) = opt {
            val
        } else {
            return "early".to_string();  // Diverges
        };

        result
    }

    /// Test 2: Nested if-let with diverging branches
    fn test_nested_diverging() -> Str {
        let opt = Some("value".to_string());

        let result: Str = if let Some(val) = opt {
            if val == "value" {
                val
            } else {
                return "early".to_string();  // Diverges
            }
        } else {
            return "none".to_string();  // Diverges
        };

        result
    }

    /// Test 3: The exact pattern from minimact
    fn test_minimact_pattern(elements: &Vec<Option<Pattern>>) -> Str {
        let state_var: Str = if let Some(ref elem) = elements[0] {
            if let Pattern::Identifier(ref id) = elem {
                id.name.clone()
            } else {
                return "invalid".to_string();  // Diverges
            }
        } else {
            return "empty".to_string();  // Diverges
        };

        state_var
    }

    /// Test 4: Multiple diverging paths
    fn test_multiple_diverging(val: i32) -> i32 {
        let result: i32 = if val > 0 {
            if val > 10 {
                return 100;  // Diverges
            } else {
                val * 2
            }
        } else {
            return 0;  // Diverges
        };

        result
    }

    /// Test 5: Without type annotation (should still work)
    fn test_no_annotation() -> Str {
        let opt = Some("value".to_string());

        let result = if let Some(val) = opt {
            val
        } else {
            return "early".to_string();
        };

        result
    }
}
