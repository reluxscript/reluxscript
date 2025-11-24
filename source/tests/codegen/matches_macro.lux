/// Test matches! macro codegen

plugin MatchesMacroTest {

    fn test_simple_matches(x: &Option<i32>) -> bool {
        matches!(x, Some(_))
    }

    fn test_or_pattern(x: &Option<i32>) -> bool {
        matches!(x, Some(1) | Some(2) | None)
    }

    fn test_ast_pattern(expr: &Expression) -> bool {
        matches!(expr, Expression::JSXElement(_) | Expression::JSXFragment(_))
    }
}
