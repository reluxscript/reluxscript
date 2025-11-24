/// Test: Basic tuple destructuring in let statements
///
/// Tests tuple destructuring without type annotations

plugin TupleDestructuringTest {
    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        // Simple tuple destructuring
        let (x, y) = get_coords();

        // Nested tuples
        let ((a, b), c) = get_nested();

        // In for loops (already tested)
        for (key, value) in pairs {
            let sum = key + value;
        }

        // Use the destructured values
        let sum = x + y + a + b + c;
    }

    fn get_coords() {
        // Returns a tuple
    }

    fn get_nested() {
        // Returns nested tuple
    }
}
