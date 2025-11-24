/// Test matches! macro code generation

plugin MatchesMacroTest {

    fn test_simple_matches(expr: &Expression) -> bool {
        matches!(expr, Expression::Identifier(_))
    }

    fn test_or_pattern(expr: &Expression) -> bool {
        matches!(expr, Expression::NullLiteral | Expression::BooleanLiteral(_))
    }

    fn test_with_value(opt: &Option<i32>) -> bool {
        if matches!(opt, Some(_)) {
            true
        } else {
            false
        }
    }

    fn test_multiple_checks(node: &Statement) {
        let is_return = matches!(node, Statement::ReturnStatement(_));
        let is_if = matches!(node, Statement::IfStatement(_));

        if is_return || is_if {
            // Do something
        }
    }
}
