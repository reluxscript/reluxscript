/// Test: matches! macro with OR patterns

plugin MatchesOrPatternTest {

    fn test_simple_matches(x: &Option<i32>) -> bool {
        matches!(x, Some(_))
    }

    fn test_or_pattern(x: &Option<i32>) -> bool {
        matches!(x, Some(1) | Some(2) | None)
    }

    fn test_path_qualified_or(expr: &Expression) -> bool {
        matches!(expr, Expression::JSXElement(_) | Expression::JSXFragment(_))
    }
}
