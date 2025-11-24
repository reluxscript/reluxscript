/// Test nested function definitions

plugin NestedFunctionTest {
    fn outer_function() -> i32 {
        let x = 5;

        fn inner_function(y: i32) -> i32 {
            y * 2
        }

        inner_function(x)
    }
}
