/// Test: HashMap.len() codegen for Babel
///
/// ReluxScript should generate map.size (not map.length) for Babel/JavaScript
/// when calling .len() on a HashMap.

plugin HashMapLengthTest {
    /// Test basic HashMap.len()
    pub fn test_hashmap_len() -> usize {
        let mut map: HashMap<Str, Str> = HashMap::new();
        map.insert("a", "1");
        map.insert("b", "2");

        // Should generate map.size in JavaScript
        let count = map.len();
        return count;
    }

    /// Test HashSet.len()
    pub fn test_hashset_len() -> usize {
        let mut set: HashSet<Str> = HashSet::new();
        set.insert("a");
        set.insert("b");

        // Should generate set.size in JavaScript
        let count = set.len();
        return count;
    }

    /// Test in conditional
    pub fn test_len_in_condition() {
        let map: HashMap<Str, i32> = HashMap::new();

        if map.len() > 0 {
            // Map has items
        }

        if map.is_empty() {
            // Map is empty
        }
    }

    /// Test in visitor context
    pub fn visit_program(node: &Program) {
        let mut templates: HashMap<Str, Str> = HashMap::new();

        for item in &node.body {
            if matches!(item, FunctionDeclaration) {
                templates.insert(item.id.name.clone(), "template");
            }
        }

        // Should generate templates.size in JavaScript
        let template_count = templates.len();

        if template_count > 0 {
            // Generate output
        }
    }

    /// Test Vec.len() for comparison (should generate .length)
    pub fn test_vec_len() -> usize {
        let vec: Vec<Str> = vec!["a", "b", "c"];

        // Should generate vec.length in JavaScript (correct)
        let count = vec.len();
        return count;
    }
}
