/// Test: Closure syntax in method chains
///
/// Tests that closures work as arguments in method call chains

plugin ClosureMethodChainTest {

    /// Test 1: Simple closure in method call
    fn test_unwrap_or_else() -> Str {
        let opt: Option<Str> = None;
        opt.unwrap_or_else(|| "default".to_string())
    }

    /// Test 2: Closure with parameter
    fn test_map() -> Option<i32> {
        let opt: Option<i32> = Some(5);
        opt.map(|x| x * 2)
    }

    /// Test 3: Chained method calls with closures
    fn test_chained_closures() -> i32 {
        let nums = vec![1, 2, 3];
        nums.iter()
            .map(|x| x * 2)
            .filter(|x| x > 2)
            .sum()
    }

    /// Test 4: Closure in method chain after field access
    fn test_field_then_method_with_closure(component: &Component) -> Str {
        let result = component.name
            .unwrap_or_else(|| "default".to_string());
        result
    }

    /// Test 5: Multi-line closure in method chain
    fn test_multiline_closure() -> Option<Str> {
        let opt: Option<Str> = Some("test".to_string());
        opt.map(|x| {
            let upper = x.to_uppercase();
            upper
        })
    }
}
