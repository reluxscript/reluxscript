/// Test: Simple capturing syntax
///
/// Basic test for the capturing clause in traverse blocks.

plugin CapturingTest {
    /// Test basic capture with mutable counter
    pub fn visit_program(node: &Program) -> i32 {
        let mut count = 0;
        let mut total = 0;

        traverse(node.body) capturing [&mut count, &mut total] {
            fn visit_identifier(id: &Identifier) {
                count += 1;
                total += 1;
            }
        }

        // count and total should be modified
        return count;
    }

    /// Test immutable capture
    pub fn analyze_all(node: &Program) -> i32 {
        let mut count = 0;

        traverse(node.body) capturing [&mut count] {
            fn visit_identifier(id: &Identifier) {
                count += 1;
            }
        }

        return count;
    }
}
