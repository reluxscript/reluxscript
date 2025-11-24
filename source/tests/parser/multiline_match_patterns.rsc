/// Test: Multi-line match patterns with | operator
///
/// Tests that match patterns can span multiple lines with | operator
/// Relates to: prop_type_inference.rsc line 248

plugin MultilineMatchPatternsTest {
    fn test_single_line_or_pattern(method: &Str) -> bool {
        // Single line with multiple patterns (should work)
        match method.as_str() {
            "map" | "filter" | "reduce" => true,
            _ => false,
        }
    }

    fn test_multiline_or_pattern(method: &Str) -> bool {
        // Multi-line pattern with |
        match method.as_str() {
            "map" | "filter" | "forEach" | "find" |
            "reduce" | "sort" | "slice" => true,
            _ => false,
        }
    }

    fn test_very_long_pattern(method: &Str) -> bool {
        // Very long multi-line pattern
        match method.as_str() {
            "map" | "filter" | "forEach" | "find" | "some" | "every" |
            "reduce" | "reduceRight" | "sort" | "slice" | "splice" |
            "push" | "pop" | "shift" | "unshift" => true,
            _ => false,
        }
    }
}
