// Test pre and exit hooks
plugin TestHooks {
    fn pre(file: &File) {
        // This should run before any visitors
        let original_code = file.code;
    }

    fn visit_jsx_element(node: &mut JSXElement) {
        // Regular visitor
    }

    fn exit(program: &mut Program, state: &PluginState) {
        // This should run after all visitors
        let output_path = "test.out";
    }
}
