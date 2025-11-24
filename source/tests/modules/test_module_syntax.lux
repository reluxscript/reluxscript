// Test different module import syntaxes

// Built-in module (simple identifier)
use fs;
use json;

// File module (string path)
use "./helpers.rsc";

// File module with alias
use "./utils/types.rsc" as types;

// File module with specific imports
use "./extractors/props.rsc" { extract_props, PropInfo };

// File module with alias AND imports
use "./extractors/hooks.rsc" as hooks { extract_useState };

plugin ModuleSyntaxTest {
    fn visit_function_declaration(node: &mut FunctionDeclaration, ctx: &Context) {
        // Test code
    }
}
