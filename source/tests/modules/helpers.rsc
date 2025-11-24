// Helper functions module (to be exported)

pub fn get_component_name(node: &FunctionDeclaration) -> Str {
    node.id.name.clone()
}

pub fn escape_string(s: &Str) -> Str {
    s.replace("\\", "\\\\")
     .replace("\"", "\\\"")
     .replace("\n", "\\n")
}

// Private function (not exported)
fn internal_helper() -> Str {
    "internal"
}

pub struct ComponentInfo {
    pub name: Str,
    pub props: Vec<Str>,
}
