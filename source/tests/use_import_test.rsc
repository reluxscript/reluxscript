// Test use statement with imports but no alias
use "./helpers.rsc" { get_name, format_code };

plugin TestPlugin {

fn transform(node: &Node) {
    let name = get_name(node);
    let code = format_code(name);
}

}
