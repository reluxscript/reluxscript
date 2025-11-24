// Test nested struct inside writer
writer TestWriter {
    struct Point {
        x: i32,
        y: i32,
    }

    struct Rectangle {
        top_left: Point,
        bottom_right: Point,
    }

    fn make_rect() -> Rectangle {
        Rectangle {
            top_left: Point { x: 0, y: 0 },
            bottom_right: Point { x: 10, y: 10 },
        }
    }
}
