// Test: Return statements in match expressions
writer TestReturn {
    fn get_type(expr: &Expression) -> Str {
        match expr {
            Expression::NumericLiteral(num) => {
                return "number";
            },
            Expression::StringLiteral(str) => {
                return "string";
            },
            _ => {
                return "other";
            }
        }
    }
}
