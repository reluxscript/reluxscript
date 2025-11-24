/// Test: Generic Functions with Where Clauses
///
/// Tests syntactic support for:
/// - Type parameters: <F, T>
/// - Where clauses: where F: Fn(...)
/// - Function trait types: Fn(&Type, bool) -> Str

plugin GenericFunctionTest {

    /// Test 1: Simple generic function
    fn simple_generic<T>(value: T) -> T {
        value
    }

    /// Test 2: Multiple type parameters
    fn multiple_params<T, U>(first: T, second: U) -> T {
        first
    }

    /// Test 3: Generic with where clause
    fn with_where_clause<F>(callback: F) -> Str
    where
        F: Fn(i32) -> Str
    {
        callback(42)
    }

    /// Test 4: Complex function trait bound
    fn complex_bound<F>(node: &Expression, generator: F) -> Str
    where
        F: Fn(&Expression, bool) -> Str
    {
        generator(node, false)
    }

    /// Test 5: Multiple where predicates
    fn multiple_where<F, G>(f: F, g: G) -> Str
    where
        F: Fn(i32) -> Str,
        G: Fn(Str) -> bool
    {
        let s = f(10);
        if g(s.clone()) {
            s
        } else {
            "".to_string()
        }
    }
}
