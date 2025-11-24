/// Hex path generator for JSX elements
/// Stub based on babel-plugin-minimact/src/utils/hexPath.cjs

struct HexPathGenerator {
    counter: i32,
}

impl HexPathGenerator {
    fn new() -> HexPathGenerator {
        HexPathGenerator { counter: 0 }
    }

    fn next(&mut self, parent_path: &Str) -> Str {
        // Stub: Generate next hex path
        self.counter = self.counter + 1;
        format!("{}", self.counter)
    }

    fn build_path(&self, parent: &Str, child: &Str) -> Str {
        // Stub: Build full path from parent and child
        if parent.is_empty() {
            child.clone()
        } else {
            format!("{}/{}", parent, child)
        }
    }

    /// Update this generator with values from another
    /// Workaround for ReluxScript's no-direct-mutation rule
    fn update_from(&mut self, other: &HexPathGenerator) {
        self.counter = other.counter;
    }
}
