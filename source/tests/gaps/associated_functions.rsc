/// Test: Associated function calls with :: on custom types
///
/// ReluxScript should support calling associated functions on custom types
/// using the :: syntax, not just on built-in types.

plugin AssociatedFunctionsTest {
    struct StringBuilder {
        lines: Vec<Str>,
    }

    struct Template {
        path: Str,
        bindings: Vec<Str>,
    }

    /// Test :: syntax on custom type (should work like HashMap::new())
    pub fn test_associated_call() {
        // This works (built-in)
        let map = HashMap::new();
        let str = String::new();

        // This should also work (custom type)
        let builder = StringBuilder::new();
        let template = Template::empty();
    }

    /// Define the associated functions
    impl StringBuilder {
        fn new() -> StringBuilder {
            return StringBuilder {
                lines: vec![],
            };
        }

        fn with_capacity(cap: usize) -> StringBuilder {
            return StringBuilder {
                lines: vec![],
            };
        }
    }

    impl Template {
        fn empty() -> Template {
            return Template {
                path: "",
                bindings: vec![],
            };
        }

        fn from_path(path: Str) -> Template {
            return Template {
                path: path,
                bindings: vec![],
            };
        }
    }

    /// Test in visitor context
    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        // Should work
        let builder = StringBuilder::new();

        // Should also work
        let template = Template::from_path("0.1");
    }
}
