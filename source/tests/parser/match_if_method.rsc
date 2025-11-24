/// Test match arm with if expression and method call

plugin MatchIfMethodTest {
    fn test(lit: &Literal) -> Str {
        match lit {
            Literal::Bool(val) => {
                if val.value { "true" } else { "false" }.to_string()
            },
            _ => "other".to_string()
        }
    }
}
