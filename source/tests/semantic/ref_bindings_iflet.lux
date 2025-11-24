/// Test: ref bindings in if-let patterns should be tracked as defined variables
///
/// Currently fails with RS006: Undefined variable
/// The semantic analyzer should recognize that `ref x` creates a binding

plugin RefBindingsTest {

    /// Test 1: Simple ref binding
    fn test_simple_ref(opt: &Option<Str>) -> Str {
        if let Some(ref value) = opt {
            // value should be recognized as defined variable
            return value.clone();
        }
        "".to_string()
    }

    /// Test 2: Nested ref bindings
    fn test_nested_ref(node: &Expression) -> Str {
        if let Expression::CallExpression(call) = node {
            if let Some(ref arg) = call.arguments.get(0) {
                // arg should be recognized as defined
                return arg.to_string();
            }
        }
        "".to_string()
    }

    /// Test 3: ref in pattern matching with path-qualified patterns
    fn test_ref_with_path_pattern(param: &Pattern) -> Str {
        if let Pattern::ObjectPattern(ref obj_pat) = param {
            // obj_pat should be recognized as defined
            let count = obj_pat.properties.len();
            return format!("{}", count);
        }
        "0".to_string()
    }

    /// Test 4: Multiple ref bindings
    fn test_multiple_refs(expr: &Expression) {
        if let Expression::BinaryExpression(ref bin) = expr {
            // Both should be recognized
            let left_str = format!("{:?}", bin.left);
            let right_str = format!("{:?}", bin.right);
        }
    }
}
