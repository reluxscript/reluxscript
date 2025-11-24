/// Test: Variables resolving to unit type () instead of proper types
///
/// This tests various cases where the type inference might incorrectly
/// infer () (unit type) instead of the actual type

plugin UnitTypeInferenceTest {

    struct MyStruct {
        value: Str,
    }

    /// Test 1: If-let with early return in else branch
    fn test_iflet_with_return(opt: Option<Str>) -> MyStruct {
        // The type of state_var should be Str, not ()
        // Even though the else branch has return
        let state_var = if let Some(ref value) = opt {
            value.clone()
        } else {
            return MyStruct { value: "default".to_string() };
        };

        MyStruct {
            value: state_var  // Should be Str, not ()
        }
    }

    /// Test 2: If-let with nested returns
    fn test_nested_iflet_return(opt: Option<Pattern>) -> Str {
        let name = if let Some(ref pat) = opt {
            if let Pattern::Identifier(ref id) = pat {
                id.name.clone()
            } else {
                return "error".to_string();
            }
        } else {
            return "none".to_string();
        };

        name  // Should be Str, not ()
    }

    /// Test 3: Multiple early returns
    fn test_multiple_returns(items: Vec<Str>) -> Str {
        let first = if items.len() > 0 {
            items[0].clone()
        } else {
            return "empty".to_string();
        };

        let second = if items.len() > 1 {
            items[1].clone()
        } else {
            return first;
        };

        second  // Should be Str, not ()
    }

    /// Test 4: Return type should be Never (!), not Unit
    fn test_return_type() {
        let x: Str = if true {
            "hello".to_string()
        } else {
            return;  // This should have type Never (!), allowing the if-expr to be Str
        };
    }

    /// Test 5: If-let with Option return in else
    fn test_iflet_option_return(opt: Option<Str>) -> Option<Str> {
        let value = if let Some(ref v) = opt {
            v.clone()
        } else {
            return None;  // Return None, not return ()
        };

        Some(value)  // value should be Str, not ()
    }
}
