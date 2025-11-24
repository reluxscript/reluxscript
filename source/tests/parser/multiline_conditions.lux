/// Test: Multi-line conditions with || operator
///
/// Tests that conditions can span multiple lines with || operator
/// Relates to: detection.rsc line 99, razor_detection.rsc line 32

plugin MultilineConditionTest {
    fn test_multiline_or() -> bool {
        let x = 5;
        let y = 10;

        // Multi-line condition with ||
        if x > 3 ||
           y > 8 {
            return true;
        }
        false
    }

    fn test_multiline_matches() -> bool {
        let expr = Expression::Literal(Literal::String("test".to_string()));

        // Multi-line matches! with ||
        if matches!(expr, Expression::Literal(_)) ||
           matches!(expr, Expression::Identifier(_)) {
            return true;
        }
        false
    }
}
