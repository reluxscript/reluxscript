/// Match with number OR pattern

plugin MatchNumberOr {
    fn test(x: i32) -> bool {
        match x {
            1 | 2 | 3 => true,
            _ => false,
        }
    }
}
