// Test: Match expression on enum variants
writer TestMatch {
    fn test_match(expr: &Expression) -> Str {
        match expr {
            Expression::NumericLiteral(num) => "number",
            Expression::StringLiteral(str) => "string",
            _ => "other"
        }
    }
}
