#[derive(Clone, Debug, Default)]
pub struct Counter {
    pub value: i32,
}


pub fn increment(c: &mut Counter) {
    c.value = (c.value + 1);
}


pub fn is_positive(n: i32) -> bool {
    (n > 0)
}

