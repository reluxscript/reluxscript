/// Test: Bool vs bool type consistency
///
/// Tests that ReluxScript type system properly handles Bool (ReluxScript type)
/// vs bool (Rust primitive type)

plugin BoolTypeTest {

    struct Config {
        is_enabled: Bool,
        is_active: Bool,
    }

    /// Test 1: Bool literal assignment
    fn test_bool_literal() -> Config {
        Config {
            is_enabled: true,   // Should this be Bool or bool?
            is_active: false,
        }
    }

    /// Test 2: Bool from expression
    fn test_bool_expression() -> Bool {
        let x = 5;
        let result = x > 3;  // Is this Bool or bool?
        result
    }

    /// Test 3: Bool in conditionals
    fn test_bool_conditional(flag: Bool) -> Str {
        if flag {
            "yes".to_string()
        } else {
            "no".to_string()
        }
    }

    /// Test 4: Comparison returning bool
    fn test_comparison() -> Config {
        let a = 10;
        let b = 20;
        Config {
            is_enabled: a < b,    // Comparison returns bool or Bool?
            is_active: a == b,
        }
    }
}
