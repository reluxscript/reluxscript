/// Test: ref keyword in patterns
///
/// Tests that patterns support the ref keyword for borrow bindings

plugin RefPatternTest {

    /// Test 1: ref with path-qualified pattern
    fn test_ref_path_qualified(expr: &Expression) {
        if let Expression::CallExpression(ref call) = expr {
            // call is &CallExpression
            let arg_count = call.arguments.len();
        }
    }

    /// Test 2: ref with simple variant pattern
    fn test_ref_variant(opt: &Option<i32>) {
        if let Some(ref value) = opt {
            // value is &i32
            let doubled = value * 2;
        }
    }

    /// Test 3: ref with tuple pattern element
    fn test_ref_tuple() {
        let pair = (1, 2);
        if let (ref x, y) = pair {
            // x is &i32, y is i32
            let sum = x + y;
        }
    }

    /// Test 4: Multiple refs in tuple
    fn test_ref_multiple_tuple() {
        let data = ("hello", 42);
        if let (ref s, ref n) = data {
            // Both are references
            let len = s.len();
        }
    }

    /// Test 5: ref with object pattern (from minimact)
    fn test_ref_object_pattern(param: &Pattern) {
        if let Pattern::ObjectPattern(ref obj_pat) = param {
            let prop_count = obj_pat.properties.len();
        }
    }
}
