// Test multi-line imports
use "./helpers.rsc" {
    escape_string,
    get_component_name,
    is_component_name
};

plugin TestPlugin {

fn transform(node: &Node) {
    let name = get_component_name(node);
    let escaped = escape_string(name);
    let is_comp = is_component_name(name);
}

}
