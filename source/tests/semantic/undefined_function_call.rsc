/// Test: Calling undefined function in if-expression

plugin UndefinedFunctionTest {
    struct MyStruct {
        value: Str,
    }

    fn test_undefined_call(items: Vec<Str>) -> MyStruct {
        // Call undefined function in if-expression
        let result = if items.len() > 0 {
            undefined_function(&items[0])  // This function doesn't exist
        } else {
            "default".to_string()
        };

        MyStruct {
            value: result  // Should this be Str or ()?
        }
    }
}
