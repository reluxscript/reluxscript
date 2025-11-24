/// Test: Tuple destructuring in let statements
///
/// ReluxScript should support tuple destructuring in various contexts

plugin TupleDestructuringTest {
    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        // Simple tuple destructuring
        let (x, y) = get_coords();

        // With types
        let (name, count): (Str, i32) = get_name_and_count();

        // Nested tuples
        let ((a, b), c) = get_nested();

        // In for loops (already tested but including for completeness)
        for (key, value) in get_map() {
            // use key and value
        }

        // With Option
        if let Some((first, second)) = get_optional_pair() {
            // use first and second
        }

        // Ignoring values with underscore
        let (result, _) = get_result_and_metadata();
        let (_, _, third) = get_triple();
    }
}

fn get_coords() -> (i32, i32) {
    (10, 20)
}

fn get_name_and_count() -> (Str, i32) {
    ("test", 5)
}

fn get_nested() -> ((i32, i32), i32) {
    ((1, 2), 3)
}

fn get_map() -> Vec<(Str, i32)> {
    vec![("a", 1), ("b", 2)]
}

fn get_optional_pair() -> Option<(i32, i32)> {
    Some((1, 2))
}

fn get_result_and_metadata() -> (Str, i32) {
    ("result", 0)
}

fn get_triple() -> (i32, i32, i32) {
    (1, 2, 3)
}
