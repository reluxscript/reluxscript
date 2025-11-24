/// Simple test plugin
plugin SimplePlugin {
    struct State {
        count: i32,
        name: Str,
    }

    fn visit_identifier(node: &mut Identifier, ctx: &Context) {
        let name = node.name.clone();
        if name == "foo" {
            *node = Identifier {
                name: "bar",
            };
        }
    }

    fn is_hook_name(name: &Str) -> bool {
        return name.starts_with("use") && name.len() > 3;
    }
}
