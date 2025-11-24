/// Simplest match with | pattern

plugin SimpleMatchOr {
    fn test(x: &Str) -> bool {
        match x.as_str() {
            "a" | "b" | "c" => true,
            _ => false,
        }
    }
}
