/// Simplest possible diverging expression test

plugin SimpleDivergeTest {
    fn test() -> Str {
        let x = true;

        // Simplest case: if-else with return in else
        let result: Str = if x {
            "value".to_string()
        } else {
            return "early".to_string();
        };

        result
    }
}
