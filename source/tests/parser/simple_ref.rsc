plugin SimpleRefTest {
    fn test(expr: &Expression) {
        if let Expression::CallExpression(ref call) = expr {
            let x = 1;
        }
    }
}
