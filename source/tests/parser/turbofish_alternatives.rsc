/// Test alternatives to turbofish syntax parse::<i32>()

plugin TurbofishAlternatives {

    /// Test 1: Turbofish syntax (should fail to parse)
    fn test_turbofish(s: &Str) -> bool {
        // s.parse::<i32>().is_ok()  // This would fail to parse
        false
    }

    /// Test 2: Type annotation on variable
    fn test_type_annotation(s: &Str) -> bool {
        let result: Result<i32, Str> = s.parse();
        result.is_ok()
    }

    /// Test 3: Type annotation in match
    fn test_match_annotation(s: &Str) -> i32 {
        let num: i32 = match s.parse() {
            Ok(n) => n,
            Err(_) => 0,
        };
        num
    }

    /// Test 4: Type annotation in if let
    fn test_if_let_annotation(s: &Str) -> Option<i32> {
        let result: Result<i32, Str> = s.parse();
        if let Ok(n) = result {
            Some(n)
        } else {
            None
        }
    }

    /// Test 5: Inline type annotation
    fn test_inline_annotation(s: &Str) -> bool {
        let result = s.parse();
        let num: i32 = result.unwrap_or(0);
        num > 0
    }

    /// Test 6: Using is_err() on typed result
    fn test_is_err_typed(s: &Str) -> bool {
        let result: Result<i32, Str> = s.parse();
        if result.is_err() {
            return false;
        }
        true
    }
}
