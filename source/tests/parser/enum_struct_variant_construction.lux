/// Test: Enum struct variant construction
///
/// Tests creating instances of enum variants with struct-style fields

plugin EnumStructVariantConstructionTest {

    enum Message {
        Quit,
        Move { x: i32, y: i32 },
        Write(Str),
        ChangeColor(i32, i32, i32),
    }

    /// Test 1: Construct struct variant
    fn test_struct_variant_construction() -> Message {
        Message::Move { x: 10, y: 20 }
    }

    /// Test 2: Construct unit variant
    fn test_unit_variant() -> Message {
        Message::Quit
    }

    /// Test 3: Construct tuple variant
    fn test_tuple_variant() -> Message {
        Message::Write("hello".to_string())
    }

    /// Test 4: Use in variable
    fn test_in_variable() {
        let msg = Message::Move { x: 5, y: 15 };
    }

    /// Test 5: Pattern matching with struct variants
    fn test_pattern_match(msg: Message) {
        match msg {
            Message::Move { x, y } => {
                let sum = x + y;
            }
            Message::Quit => {}
            Message::Write(text) => {}
            Message::ChangeColor(r, g, b) => {}
        }
    }
}
