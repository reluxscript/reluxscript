/// Test ownership checking
plugin OwnershipTest {
    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        // Good: using .clone()
        let name = node.name.clone();

        // Good: using method that returns owned value
        let len = name.len();

        // Good: statement lowering pattern
        if name == "foo" {
            *node = Identifier {
                name: "bar",
            };
        }
    }

    fn bad_ownership(node: &Identifier) {
        // Bad: implicit borrow without clone
        let name = node.name;

        // Bad: direct property mutation
        node.name = "bad";
    }
}
