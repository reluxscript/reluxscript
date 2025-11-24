/// Minimact TSX to C# Transpiler
///
/// Transpiles React/TSX components to C# MinimactComponent classes.
/// Generates C# code, template JSON, and hook metadata.

use fs;
use json;

writer MinimactTranspiler {
    /// Writer state
    struct State {
        // Output builders
        csharp: CodeBuilder,
        templates: HashMap<Str, Template>,
        hooks: Vec<HookSignature>,

        // Traversal state
        current_component: Option<ComponentInfo>,

        // Collected components
        components: Vec<ComponentInfo>,
    }

    /// Initialize the transpiler
    fn init() -> State {
        let csharp = CodeBuilder::new();
        let templates = HashMap::new();
        let hooks = vec![];
        let components = vec![];

        return State {
            csharp: csharp,
            templates: templates,
            hooks: hooks,
            current_component: None,
            components: components,
        };
    }

    /// Visit function declarations to find React components
    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        let name = node.id.name.clone();

        // Check if this is a component (PascalCase name)
        if !is_pascal_case(&name) {
            return;
        }

        // Initialize component info
        let mut component = ComponentInfo::new(name.clone());

        // Extract props from parameters
        if node.params.len() > 0 {
            extract_props(&node.params[0], &mut component);
        }

        // Traverse body to extract hooks and render
        if let Some(body) = &node.body {
            traverse(body) capturing [&mut component] {
                fn visit_variable_declarator(decl: &VariableDeclarator) {
                    if let Some(init) = &decl.init {
                        if matches!(init, CallExpression) {
                            extract_hook_from_call(init, &decl.id, &mut component);
                        }
                    }
                }

                fn visit_return_statement(ret: &ReturnStatement) {
                    if let Some(arg) = &ret.argument {
                        if matches!(arg, JSXElement) {
                            component.render_body = Some(arg.clone());
                        }
                    }
                }
            }
        }

        // Store the component
        self.components.push(component);
    }

    /// Generate all output after traversal
    fn finish(&self) -> TranspilerOutput {
        let mut csharp_code = String::new();

        // Generate C# for each component
        for component in &self.components {
            let code = generate_csharp_class(&component);
            csharp_code.push_str(&code);
            csharp_code.push_str("\n");
        }

        // Build template map
        let mut all_templates: HashMap<Str, Template> = HashMap::new();
        for component in &self.components {
            for (key, template) in &component.templates {
                let full_key = format!("{}.{}", component.name, key);
                all_templates.insert(full_key, template.clone());
            }
        }

        TranspilerOutput {
            csharp: csharp_code,
            templates: json::to_string_pretty(&all_templates).unwrap(),
            hooks: json::to_string_pretty(&self.hooks).unwrap(),
        }
    }
}

// =============================================================================
// Data Structures
// =============================================================================

#[derive(Serialize)]
struct TranspilerOutput {
    csharp: Str,
    templates: Str,
    hooks: Str,
}

#[derive(Clone, Serialize)]
struct ComponentInfo {
    name: Str,
    props: Vec<PropInfo>,
    use_state: Vec<StateInfo>,
    use_effect: Vec<EffectInfo>,
    use_ref: Vec<RefInfo>,
    event_handlers: Vec<EventHandlerInfo>,
    render_body: Option<Box<Expr>>,
    templates: HashMap<Str, Template>,
}

fn component_info_new(name: Str) -> ComponentInfo {
    ComponentInfo {
        name: name,
        props: vec![],
        use_state: vec![],
        use_effect: vec![],
        use_ref: vec![],
        event_handlers: vec![],
        render_body: None,
        templates: HashMap::new(),
    }
}

#[derive(Clone, Serialize)]
struct PropInfo {
    name: Str,
    prop_type: Str,
    optional: bool,
}

#[derive(Clone, Serialize)]
struct StateInfo {
    name: Str,
    setter: Option<Str>,
    initial_value: Str,
    state_type: Str,
}

#[derive(Clone, Serialize)]
struct EffectInfo {
    dependencies: Vec<Str>,
    is_client_side: bool,
}

#[derive(Clone, Serialize)]
struct RefInfo {
    name: Str,
    initial_value: Str,
}

#[derive(Clone, Serialize)]
struct EventHandlerInfo {
    name: Str,
    params: Vec<Str>,
    is_async: bool,
}

#[derive(Clone, Serialize)]
struct Template {
    path: Str,
    template: Str,
    bindings: Vec<Str>,
}

#[derive(Clone, Serialize)]
struct HookSignature {
    name: Str,
    hook_type: Str,
}

// CodeBuilder is now a built-in type (see spec 3.4)

// =============================================================================
// Helper Functions
// =============================================================================

/// Check if a name is PascalCase (starts with uppercase)
fn is_pascal_case(name: &Str) -> bool {
    if name.len() == 0 {
        return false;
    }
    let first_char = name.chars().next().unwrap();
    return first_char.is_uppercase();
}

/// Extract props from function parameter
fn extract_props(param: &Pattern, component: &mut ComponentInfo) {
    // Handle destructured props: function Component({ name, age })
    if matches!(param, ObjectPattern) {
        for prop in &param.properties {
            if matches!(prop, ObjectPatternProperty) {
                let prop_name = prop.key.name.clone();
                component.props.push(PropInfo {
                    name: prop_name,
                    prop_type: "dynamic".to_string(),
                    optional: false,
                });
            }
        }
    }
    // Handle identifier props: function Component(props)
    else if matches!(param, Identifier) {
        // Props accessed via props.x - would need deeper analysis
    }
}

/// Extract hook from a call expression
fn extract_hook_from_call(call: &CallExpression, binding: &Pattern, component: &mut ComponentInfo) {
    if !matches!(call.callee, Identifier) {
        return;
    }

    let callee_name = call.callee.name.clone();

    if callee_name == "useState" || callee_name == "useClientState" {
        extract_use_state(call, binding, component);
    } else if callee_name == "useEffect" {
        extract_use_effect(call, component);
    } else if callee_name == "useRef" {
        extract_use_ref(call, binding, component);
    } else if callee_name.starts_with("use") {
        // Custom hook - record for later processing
    }
}

/// Extract useState hook
fn extract_use_state(call: &CallExpression, binding: &Pattern, component: &mut ComponentInfo) {
    // Pattern: const [value, setValue] = useState(initial)
    if !matches!(binding, ArrayPattern) {
        return;
    }

    let arr = binding.clone();
    if arr.elements.len() < 1 {
        return;
    }

    let state_var = arr.elements[0].name.clone();
    let setter_var = if arr.elements.len() > 1 {
        Some(arr.elements[1].name.clone())
    } else {
        None
    };

    // Get initial value
    let initial_value = if call.arguments.len() > 0 {
        expr_to_csharp(&call.arguments[0])
    } else {
        "null".to_string()
    };

    // Infer type from initial value
    let state_type = if call.arguments.len() > 0 {
        infer_csharp_type(&call.arguments[0])
    } else {
        "dynamic".to_string()
    };

    component.use_state.push(StateInfo {
        name: state_var,
        setter: setter_var,
        initial_value: initial_value,
        state_type: state_type,
    });
}

/// Extract useEffect hook
fn extract_use_effect(call: &CallExpression, component: &mut ComponentInfo) {
    let mut deps: Vec<Str> = vec![];

    // Get dependency array (second argument)
    if call.arguments.len() > 1 {
        let deps_arg = &call.arguments[1];
        if matches!(deps_arg, ArrayExpression) {
            for elem in &deps_arg.elements {
                if matches!(elem, Identifier) {
                    deps.push(elem.name.clone());
                }
            }
        }
    }

    component.use_effect.push(EffectInfo {
        dependencies: deps,
        is_client_side: false,
    });
}

/// Extract useRef hook
fn extract_use_ref(call: &CallExpression, binding: &Pattern, component: &mut ComponentInfo) {
    if !matches!(binding, Identifier) {
        return;
    }

    let ref_name = binding.name.clone();

    let initial_value = if call.arguments.len() > 0 {
        expr_to_csharp(&call.arguments[0])
    } else {
        "null".to_string()
    };

    component.use_ref.push(RefInfo {
        name: ref_name,
        initial_value: initial_value,
    });
}

/// Convert expression to C# code string
fn expr_to_csharp(expr: &Expr) -> Str {
    if matches!(expr, StringLiteral) {
        return format!("\"{}\"", expr.value);
    } else if matches!(expr, NumericLiteral) {
        return expr.value.to_string();
    } else if matches!(expr, BooleanLiteral) {
        return if expr.value { "true" } else { "false" }.to_string();
    } else if matches!(expr, NullLiteral) {
        return "null".to_string();
    } else if matches!(expr, Identifier) {
        return expr.name.clone();
    } else if matches!(expr, ArrayExpression) {
        return "new List<dynamic>()".to_string();
    } else if matches!(expr, ObjectExpression) {
        return "new Dictionary<string, dynamic>()".to_string();
    }

    return "null".to_string();
}

/// Infer C# type from expression
fn infer_csharp_type(expr: &Expr) -> Str {
    if matches!(expr, StringLiteral) {
        return "string".to_string();
    } else if matches!(expr, NumericLiteral) {
        // Check if it's an integer or float
        let val = expr.value;
        if val == val.floor() {
            return "int".to_string();
        } else {
            return "double".to_string();
        }
    } else if matches!(expr, BooleanLiteral) {
        return "bool".to_string();
    } else if matches!(expr, ArrayExpression) {
        return "List<dynamic>".to_string();
    } else if matches!(expr, ObjectExpression) {
        return "Dictionary<string, dynamic>".to_string();
    }

    return "dynamic".to_string();
}

// =============================================================================
// C# Code Generation
// =============================================================================

/// Generate C# class for a component
fn generate_csharp_class(component: &ComponentInfo) -> Str {
    let mut builder = CodeBuilder::new();

    // Usings
    builder.append_line(&"using Minimact.Core;");
    builder.append_line(&"using Minimact.VDom;");
    builder.append_line(&"using System.Collections.Generic;");
    builder.newline();

    // Class declaration
    builder.append_line(&format!("public class {} : MinimactComponent", component.name));
    builder.append_line(&"{");
    builder.indent();

    // Generate state fields
    for state in &component.use_state {
        builder.append_line(&format!(
            "[State] private {} {} = {};",
            state.state_type,
            state.name,
            state.initial_value
        ));
    }

    // Generate ref fields
    for ref_info in &component.use_ref {
        builder.append_line(&format!(
            "[Ref] private object {} = {};",
            ref_info.name,
            ref_info.initial_value
        ));
    }

    if component.use_state.len() > 0 || component.use_ref.len() > 0 {
        builder.newline();
    }

    // Generate Render method
    builder.append_line(&"protected override VNode Render()");
    builder.append_line(&"{");
    builder.indent();

    if component.render_body.is_some() {
        // TODO: Generate VNode tree from JSX
        builder.append_line(&"// TODO: Generate VNode from JSX");
        builder.append_line(&"return new VNull();");
    } else {
        builder.append_line(&"return new VNull();");
    }

    builder.dedent();
    builder.append_line(&"}");

    builder.dedent();
    builder.append_line(&"}");

    return builder.to_string();
}
