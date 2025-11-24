// Test: Match with variant bindings
writer TestMatchBinding {
    fn get_value(expr: &Expression) -> Str {
        match expr {
            Expression::NumericLiteral(num) => {
                num.value.to_string()
            },
            Expression::StringLiteral(str) => {
                str.value.clone()
            },
            _ => {
                "unknown"
            }
        }
    }
}
