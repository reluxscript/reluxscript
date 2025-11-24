// Test: Iterator methods like chars() and enumerate()
writer TestIterators {
    fn iterate_string(s: &str) {
        // Test chars()
        for ch in s.chars() {
            // Do something
        }
    }

    fn iterate_with_index(items: &Vec<Str>) {
        // Test enumerate()
        for (i, item) in items.iter().enumerate() {
            // Do something
        }
    }
}
