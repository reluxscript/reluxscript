#[path = "module_a.rs"]
mod module_a;
pub use module_a::{Point, distance_squared, origin};


#[derive(Clone, Debug, Default)]
pub struct Rectangle {
    pub top_left: Point,
    pub bottom_right: Point,
}


pub fn area(r: &Rectangle) -> i32 {
    let width = (r.bottom_right.x - r.top_left.x);
    let height = (r.bottom_right.y - r.top_left.y);
    (width * height)
}


pub fn is_at_origin(r: &Rectangle) -> bool {
    (distance_squared(&r.top_left) == 0)
}

