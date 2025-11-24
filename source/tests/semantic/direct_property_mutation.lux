/// Test: Direct property mutation restrictions
///
/// ReluxScript semantic analyzer forbids direct field assignment in certain contexts
/// Error: RS002 - Direct property mutation not allowed

plugin PropertyMutationTest {

    struct ComponentInfo {
        name: Str,
        render_body: Option<Expression>,
        props: Vec<Str>,
    }

    /// Test 1: Direct mutation (should fail with RS002)
    fn test_direct_mutation_fails() {
        let mut comp = ComponentInfo {
            name: "Test".to_string(),
            render_body: None,
            props: vec![],
        };

        // This should trigger RS002
        comp.render_body = Some(Expression::Literal(Literal::String("body".to_string())));
        comp.name = "Updated".to_string();
    }

    /// Test 2: Whole-node replacement (should succeed)
    fn test_whole_node_replacement() {
        let comp = ComponentInfo {
            name: "Test".to_string(),
            render_body: None,
            props: vec![],
        };

        // Replace entire struct instead of mutating
        let comp = ComponentInfo {
            name: "Updated".to_string(),
            render_body: comp.render_body,
            props: comp.props,
        };
    }

    /// Test 3: Mutation inside traverse block (context-specific)
    fn test_mutation_in_traverse(node: &Expression) {
        let mut comp = ComponentInfo {
            name: "Test".to_string(),
            render_body: None,
            props: vec![],
        };

        // Does context matter for mutation restrictions?
        traverse(node) capturing [&mut comp] {
            fn visit_return_statement(ret: &ReturnStatement) {
                if let Some(arg) = &ret.argument {
                    // This is the actual pattern from minimact - does it work?
                    comp.render_body = Some(Box::new(arg.clone()));
                }
            }
        }
    }

    /// Test 4: Vec/HashMap mutations
    fn test_collection_mutations() {
        let mut items = vec![];

        // Are collection mutations allowed?
        items.push("item1".to_string());
        items.push("item2".to_string());
    }
}
