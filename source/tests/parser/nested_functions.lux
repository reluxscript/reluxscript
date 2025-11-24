/// Test: Nested function definitions
///
/// Tests that functions can be defined inside other functions

plugin NestedFunctionTest {

    /// Test 1: Simple nested function
    fn outer() -> i32 {
        fn inner() -> i32 {
            42
        }

        inner()
    }

    /// Test 2: Nested function with parameters
    fn outer_with_params(x: i32) -> i32 {
        fn add_ten(n: i32) -> i32 {
            n + 10
        }

        add_ten(x)
    }

    /// Test 3: Nested function accessing outer scope
    fn outer_with_closure_like() -> i32 {
        let multiplier = 5;

        fn multiply(n: i32) -> i32 {
            n * multiplier  // Access outer variable (if supported)
        }

        multiply(3)
    }

    /// Test 4: Multiple nested functions
    fn outer_multiple() -> i32 {
        fn helper1() -> i32 {
            10
        }

        fn helper2() -> i32 {
            20
        }

        helper1() + helper2()
    }
}
