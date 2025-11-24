/// Minimal test for path-qualified patterns

plugin SimplePathTest {
    fn test(expr: &Expression) -> bool {
        if let Expression::StringLiteral(lit) = expr {
            true
        } else {
            false
        }
    }
}
