/// Test block ending with if-let expression

plugin BlockEndingIfLetTest {
    fn test(opt: Option<Str>) -> Str {
        let result: Str = {
            // Block contains only an if-let expression
            if let Some(val) = opt {
                val
            } else {
                return "default".to_string();
            }
        };

        result
    }
}
