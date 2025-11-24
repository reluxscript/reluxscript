// Test matches! macro
plugin MatchesTest {

fn test(expr: &CallExpression) {
    if matches!(expr, CallExpression(_)) {
        // matched
    }
}

}
