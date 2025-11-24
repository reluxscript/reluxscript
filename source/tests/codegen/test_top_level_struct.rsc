// Test top-level struct outside plugin/writer
struct Point {
    x: i32,
    y: i32,
}

plugin TestPlugin {
    fn use_point() -> Point {
        Point { x: 5, y: 10 }
    }
}
