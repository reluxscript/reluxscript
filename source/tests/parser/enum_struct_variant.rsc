/// Test: Enum with struct variants
///
/// Tests that enums can have struct-style variants with named fields

pub enum Message {
    Quit,
    Move { x: i32, y: i32 },
    Write { message: Str },
    ChangeColor(i32, i32, i32),
}

pub fn test_struct_variant() {
    // TODO: struct variant construction syntax not yet supported
}
