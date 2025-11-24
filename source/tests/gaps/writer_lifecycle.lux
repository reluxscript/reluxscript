/// Test: Writer lifecycle (init and finish)
///
/// ReluxScript should support writer blocks with init() and finish() lifecycle
/// methods. init() creates the initial state, and finish() produces the final output.

use fs;
use json;

writer WriterLifecycleTest {
    /// Writer state that persists across visits
    struct State {
        components: Vec<ComponentInfo>,
        output_dir: Str,
        total_hooks: i32,
    }

    /// Initialize the writer state
    /// This should be called once before any visits
    fn init() -> State {
        return State {
            components: vec![],
            output_dir: "dist",
            total_hooks: 0,
        };
    }

    /// Visit function declarations to collect components
    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        let name = node.id.name.clone();

        // Check if it's a component (PascalCase)
        if !is_component(&name) {
            return;
        }

        let component = ComponentInfo {
            name: name,
            hook_count: 0,
        };

        // Store in writer state
        self.components.push(component);
    }

    /// Visit call expressions to count hooks
    pub fn visit_call_expression(call: &CallExpression) {
        if matches!(call.callee, Identifier) {
            let name = call.callee.name.clone();
            if name.starts_with("use") {
                self.total_hooks += 1;
            }
        }
    }

    /// Generate output after all visits complete
    /// This should be called once after traversal finishes
    fn finish(&self) -> WriterOutput {
        let mut code = String::new();

        // Generate C# for each component
        for component in &self.components {
            code.push_str(&format!("public class {} {{\n", component.name));
            code.push_str("    // Generated code\n");
            code.push_str("}\n\n");
        }

        // Generate metadata JSON
        let metadata = Metadata {
            component_count: self.components.len(),
            total_hooks: self.total_hooks,
        };

        let json_str = json::to_string_pretty(&metadata).unwrap();

        return WriterOutput {
            csharp_code: code,
            metadata_json: json_str,
        };
    }
}

// =============================================================================
// Data Structures
// =============================================================================

struct ComponentInfo {
    name: Str,
    hook_count: i32,
}

#[derive(Serialize)]
struct Metadata {
    component_count: usize,
    total_hooks: i32,
}

#[derive(Serialize)]
struct WriterOutput {
    csharp_code: Str,
    metadata_json: Str,
}

// =============================================================================
// Helper Functions
// =============================================================================

fn is_component(name: &Str) -> bool {
    if name.len() == 0 {
        return false;
    }
    let first = name.chars().next().unwrap();
    return first.is_uppercase();
}
