plugin TestNodeMutation {
    pub fn visit_identifier(node: &Identifier) {
        // This should error - mutating AST node
        node.name = "newName";
    }
}
