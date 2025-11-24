/// Test: vec! macro for vector creation
///
/// ReluxScript should support vec! macro for creating vectors
/// that compiles to arrays in Babel and Vec in Rust.

plugin VecMacroTest {
    struct HookInfo {
        name: Str,
        hook_type: Str,
    }

    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        // Empty vec
        let mut items: Vec<i32> = vec![];

        // Vec with initial values
        let numbers = vec![1, 2, 3, 4, 5];

        // Vec of strings
        let names = vec!["useState", "useEffect", "useRef"];

        // Vec of structs
        let hooks = vec![
            HookInfo {
                name: "useState",
                hook_type: "state",
            },
            HookInfo {
                name: "useEffect",
                hook_type: "effect",
            },
        ];

        // Nested vecs
        let matrix: Vec<Vec<i32>> = vec![
            vec![1, 2, 3],
            vec![4, 5, 6],
        ];
    }
}
