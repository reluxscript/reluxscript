/// Test if expression with method call

plugin IfMethodChainTest {
    fn test(flag: bool) -> Str {
        let result = if flag { "true" } else { "false" }.to_string();
        result
    }
}
