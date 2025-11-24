/// Test: HashMap and HashSet Support
///
/// ReluxScript should support HashMap<K, V> and HashSet<T> with full
/// collection operations.

plugin HashMapHashSetTest {
    struct Template {
        path: Str,
        bindings: Vec<Str>,
    }

    /// Test HashMap operations
    pub fn test_hashmap_basic() {
        let mut map: HashMap<Str, Str> = HashMap::new();

        // Insert
        map.insert("key1", "value1");
        map.insert("key2", "value2");

        // Get
        let value = map.get(&"key1");

        // Contains
        if map.contains_key(&"key1") {
            // found
        }

        // Length
        let count = map.len();
    }

    /// Test HashSet operations
    pub fn test_hashset_basic() {
        let mut set: HashSet<Str> = HashSet::new();

        // Insert
        set.insert("item1");
        set.insert("item2");

        // Contains
        if set.contains(&"item1") {
            // found
        }

        // Length
        let count = set.len();
    }

    /// Test iteration
    pub fn test_iteration() {
        let mut map: HashMap<Str, Str> = HashMap::new();
        map.insert("a", "1");
        map.insert("b", "2");

        // Iteration
        for key in map.keys() {
            let k = key.clone();
        }

        for value in map.values() {
            let v = value.clone();
        }
    }
}
