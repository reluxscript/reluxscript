/// Test: Function calls as struct field initializers
///
/// ReluxScript should support calling functions directly in struct literal
/// field initializer position.

plugin FunctionCallInStructTest {
    struct StringBuilder {
        lines: Vec<Str>,
    }

    struct State {
        csharp: StringBuilder,
        templates: HashMap<Str, Str>,
        imports: HashSet<Str>,
    }

    /// Test function call in struct literal
    pub fn test_direct_call_in_struct() {
        // Should work - function calls as field values
        let state = State {
            csharp: StringBuilder::new(),
            templates: HashMap::new(),
            imports: HashSet::new(),
        };
    }

    /// Test with custom functions
    pub fn create_builder() -> StringBuilder {
        return StringBuilder {
            lines: vec![],
        };
    }

    pub fn test_custom_function_in_struct() {
        let state = State {
            csharp: create_builder(),
            templates: HashMap::new(),
            imports: HashSet::new(),
        };
    }

    /// Test nested calls
    pub fn test_nested_calls() {
        let state = State {
            csharp: StringBuilder::new(),
            templates: create_empty_map(),
            imports: HashSet::new(),
        };
    }

    pub fn create_empty_map() -> HashMap<Str, Str> {
        return HashMap::new();
    }

    /// Test in visitor context
    pub fn visit_program(node: &Program) {
        let state = State {
            csharp: StringBuilder::new(),
            templates: HashMap::new(),
            imports: HashSet::new(),
        };

        // Use state...
    }

    impl StringBuilder {
        fn new() -> StringBuilder {
            return StringBuilder {
                lines: vec![],
            };
        }
    }
}
