/// Test: Multi-line function parameter declarations
///
/// Tests that function parameters can span multiple lines
/// Relates to: hook_imports.rsc line 34

plugin MultilineFunctionParamsTest {
    // Single line params (should work)
    fn test_single_line(x: i32, y: i32) -> i32 {
        x + y
    }

    // Multi-line params
    pub fn test_multiline_params(
        first_param: &Str,
        second_param: i32
    ) -> Str {
        format!("{}: {}", first_param, second_param)
    }

    // Multi-line params with complex types
    pub fn test_complex_types(
        items: &Vec<Str>,
        mapping: &HashMap<Str, i32>
    ) -> i32 {
        items.len() + mapping.len()
    }
}
