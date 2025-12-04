#[derive(Clone, Debug, Default)]
pub struct Point {
    pub x: i32,
    pub y: i32,
}


pub fn distance_squared(p: &Point) -> i32 {
    ((p.x * p.x) + (p.y * p.y))
}


pub fn origin() -> Point {
    Point { x: 0, y: 0 }
}

