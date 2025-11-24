/// Minimact TSX to C# Transpiler (Refactored)
///
/// Mirrors the logic of babel-plugin-minimact/src/processComponent.cjs
/// Uses converted helper modules from reluxscript-plugin-minimact/

use fs;
use json;

// ✅ Import converted helper modules
use "./reluxscript-plugin-minimact/utils/helpers.rsc" {
    escape_csharp_string,
    is_component_name,
    get_component_name
};
use "./reluxscript-plugin-minimact/utils/hex_path.rsc" { HexPathGenerator };
use "./reluxscript-plugin-minimact/types/type_conversion.rsc" {
    infer_type,
    ts_type_to_csharp_type
};
use "./reluxscript-plugin-minimact/analyzers/classification.rsc" {
    classify_node,
    is_static,
    is_hybrid,
    Dependency
};
use "./reluxscript-plugin-minimact/analyzers/detection.rsc" {
    has_spread_props,
    has_dynamic_children
};
use "./reluxscript-plugin-minimact/analyzers/hook_detector.rsc" { is_custom_hook };

writer MinimactTranspiler {
    /// Writer state - mirrors babel-plugin-minimact state
    struct State {
        // Output builders
        csharp: CodeBuilder,
        templates: HashMap<Str, Template>,
        hooks: Vec<HookSignature>,

        // Hex path generator (mirrors babel-plugin-minimact)
        hex_path_gen: HexPathGenerator,

        // Current component being processed
        current_component: Option<ComponentInfo>,

        // All collected components
        components: Vec<ComponentInfo>,

        // Track external imports (lodash, dayjs, etc.)
        external_imports: HashSet<Str>,

        // Hot reload detection
        is_hot_reload: bool,
    }

    /// Initialize the transpiler
    fn init() -> State {
        State {
            csharp: CodeBuilder::new(),
            templates: HashMap::new(),
            hooks: vec![],
            hex_path_gen: HexPathGenerator::default(),
            current_component: None,
            components: vec![],
            external_imports: HashSet::new(),
            is_hot_reload: false,
        }
    }

    /// Visit function declarations to find React components
    /// Mirrors processComponent.cjs line 34-46
    pub fn visit_function_declaration(node: &FunctionDeclaration) {
        // 1. Get component name (line 35)
        let name_opt = get_component_name(node, None);
        if name_opt.is_none() {
            return;
        }
        let name = name_opt.unwrap();

        // 2. Check if this is a custom hook (line 40-43)
        // TODO: Implement isCustomHook check when available
        // if is_custom_hook(node) {
        //     return self.process_custom_hook(node);
        // }

        // 3. Validate component name - must be PascalCase (line 45)
        if !is_component_name(&name) {
            return;
        }

        // 4. Initialize component info (line 63-90)
        let mut component = ComponentInfo {
            name: name.clone(),
            props: vec![],
            use_state: vec![],
            use_client_state: vec![],
            use_effect: vec![],
            use_ref: vec![],
            custom_hooks: vec![],
            event_handlers: vec![],
            local_variables: vec![],
            helper_functions: vec![],
            render_body: None,
            templates: HashMap::new(),
            dependencies: HashMap::new(),
            external_imports: HashSet::new(),
        };

        // 5. Extract props from parameters (line 123-160)
        if node.params.len() > 0 {
            extract_props(&node.params[0], &mut component);
        }

        // 6. Traverse body to extract hooks and render (line 168-226)
        if let Some(body) = &node.body {
            // Create local mutable reference for capturing
            let mut comp = component;

            traverse(body) capturing [&mut comp] {
                fn visit_variable_declarator(decl: &VariableDeclarator) {
                    if let Some(init) = &decl.init {
                        if matches!(init, Expression::CallExpression(_)) {
                            extract_hook_from_call(init, &decl.id, &mut comp);
                        }
                    }
                }

                fn visit_return_statement(ret: &ReturnStatement) {
                    if let Some(arg) = &ret.argument {
                        comp.render_body = Some(Box::new(arg.clone()));
                    }
                }
            }

            // Reassign back
            component = comp;
        }

        // 7. Assign hex paths to JSX (line 234-247)
        if component.render_body.is_some() {
            let mut render_body = component.render_body.unwrap();
            let mut hex_gen = self.hex_path_gen;

            assign_paths_to_jsx(&mut *render_body, "", &mut hex_gen);

            component.render_body = Some(render_body);
            self.hex_path_gen = hex_gen;
        }

        // 8. Extract templates from JSX
        // TODO: Implement template extraction
        // extract_templates(&mut component);
        // extract_loop_templates(&mut component);
        // extract_conditional_templates(&mut component);

        // 9. Store the component
        self.components.push(component);
    }

    /// Generate all output after traversal
    /// Mirrors index.cjs Program exit (line 90+)
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
// Data Structures - Mirror babel-plugin-minimact component structure
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
    use_client_state: Vec<StateInfo>,
    use_effect: Vec<EffectInfo>,
    use_ref: Vec<RefInfo>,
    custom_hooks: Vec<CustomHookInfo>,
    event_handlers: Vec<EventHandlerInfo>,
    local_variables: Vec<LocalVarInfo>,
    helper_functions: Vec<HelperFunctionInfo>,
    render_body: Option<Box<Expression>>,
    templates: HashMap<Str, Template>,
    dependencies: HashMap<Str, Vec<Dependency>>,
    external_imports: HashSet<Str>,
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
struct CustomHookInfo {
    name: Str,
    namespace: Str,
    args: Vec<Str>,
}

#[derive(Clone, Serialize)]
struct EventHandlerInfo {
    name: Str,
    params: Vec<Str>,
    is_async: bool,
}

#[derive(Clone, Serialize)]
struct LocalVarInfo {
    name: Str,
    var_type: Str,
    initial_value: Str,
}

#[derive(Clone, Serialize)]
struct HelperFunctionInfo {
    name: Str,
    params: Vec<ParamInfo>,
    return_type: Str,
    is_async: bool,
}

#[derive(Clone, Serialize)]
struct ParamInfo {
    name: Str,
    param_type: Str,
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

// =============================================================================
// Helper Functions - Use converted modules
// =============================================================================

/// Extract props from function parameter
/// Mirrors processComponent.cjs line 123-160
fn extract_props(param: &Pattern, component: &mut ComponentInfo) {
    // Handle destructured props: function Component({ name, age })
    if let Pattern::ObjectPattern(ref obj_pat) = param {
        for property in &obj_pat.properties {
            if let Some(key) = property.key.as_identifier() {
                let prop_name = key.name.clone();

                // Try to get TypeScript type annotation
                let prop_type = if let Some(ref type_ann) = property.type_annotation {
                    ts_type_to_csharp_type(type_ann)
                } else {
                    "dynamic".to_string()
                };

                component.props.push(PropInfo {
                    name: prop_name,
                    prop_type: prop_type,
                    optional: property.optional.unwrap_or(false),
                });
            }
        }
    }
    // Handle identifier props: function Component(props)
    else if let Pattern::Identifier(ref id) = param {
        component.props.push(PropInfo {
            name: id.name.clone(),
            prop_type: "dynamic".to_string(),
            optional: false,
        });
    }
}

/// Extract hook from a call expression
/// Mirrors processComponent.cjs line 169-170 (extractHook call)
fn extract_hook_from_call(call: &Expression, binding: &Pattern, component: &mut ComponentInfo) {
    if let Expression::CallExpression(ref call_expr) = call {
        if let Expression::Identifier(ref callee) = &call_expr.callee {
            let callee_name = &callee.name;

            if callee_name == "useState" || callee_name == "useClientState" {
                extract_use_state(call_expr, binding, component, callee_name);
            } else if callee_name == "useEffect" {
                extract_use_effect(call_expr, component);
            } else if callee_name == "useRef" {
                extract_use_ref(call_expr, binding, component);
            } else if callee_name.starts_with("use") {
                // Custom hook - record for later processing
                extract_custom_hook(call_expr, binding, component, callee_name);
            }
        }
    }
}

/// Extract useState or useClientState hook
/// Mirrors processComponent.cjs useState extraction logic
fn extract_use_state(call: &CallExpression, binding: &Pattern, component: &mut ComponentInfo, hook_name: &Str) {
    // Pattern: const [value, setValue] = useState(initial)
    if let Pattern::ArrayPattern(ref arr_pat) = binding {
        if arr_pat.elements.len() < 1 {
            return;
        }

        let state_var = if let Some(ref elem) = arr_pat.elements[0] {
            if let Pattern::Identifier(ref id) = elem {
                id.name.clone()
            } else {
                return;
            }
        } else {
            return;
        };

        let setter_var = if arr_pat.elements.len() > 1 {
            if let Some(ref elem) = arr_pat.elements[1] {
                if let Pattern::Identifier(ref id) = elem {
                    Some(id.name.clone())
                } else {
                    None
                }
            } else {
                None
            }
        } else {
            None
        };

        // Get initial value
        let initial_value = if call.arguments.len() > 0 {
            expr_to_csharp(&call.arguments[0])
        } else {
            "null".to_string()
        };

        // ✅ Use infer_type from type_conversion.rsc
        let state_type = if call.arguments.len() > 0 {
            infer_type(&call.arguments[0])
        } else {
            "dynamic".to_string()
        };

        let state_info = StateInfo {
            name: state_var,
            setter: setter_var,
            initial_value: initial_value,
            state_type: state_type,
        };

        if hook_name == "useClientState" {
            component.use_client_state.push(state_info);
        } else {
            component.use_state.push(state_info);
        }
    }
}

/// Extract useEffect hook
fn extract_use_effect(call: &CallExpression, component: &mut ComponentInfo) {
    let mut deps: Vec<Str> = vec![];

    // Get dependency array (second argument)
    if call.arguments.len() > 1 {
        if let Expression::ArrayExpression(ref arr_expr) = &call.arguments[1] {
            for elem in &arr_expr.elements {
                if let Expression::Identifier(ref id) = elem {
                    deps.push(id.name.clone());
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
    if let Pattern::Identifier(ref id) = binding {
        let ref_name = id.name.clone();

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
}

/// Extract custom hook
fn extract_custom_hook(call: &CallExpression, binding: &Pattern, component: &mut ComponentInfo, hook_name: &Str) {
    // Custom hooks typically have a namespace as first parameter
    let namespace = if call.arguments.len() > 0 {
        if let Expression::StringLiteral(ref lit) = &call.arguments[0] {
            lit.value.clone()
        } else {
            "default".to_string()
        }
    } else {
        "default".to_string()
    };

    let args = call.arguments.iter()
        .map(|arg| expr_to_csharp(arg))
        .collect();

    component.custom_hooks.push(CustomHookInfo {
        name: hook_name.clone(),
        namespace: namespace,
        args: args,
    });
}

/// Assign hex paths to JSX tree
/// Mirrors babel-plugin-minimact/src/utils/pathAssignment.cjs
fn assign_paths_to_jsx(node: &mut Expression, parent_path: &Str, path_gen: &mut HexPathGenerator) {
    if let Expression::JSXElement(ref mut elem) = node {
        // Generate hex path for this element
        let hex_code = path_gen.next(parent_path);
        let full_path = if parent_path.is_empty() {
            hex_code.clone()
        } else {
            path_gen.build_path(parent_path, &hex_code)
        };

        // Store path as a data attribute (would need AST node modification)
        // elem.data_hex_path = Some(full_path.clone());

        // Recursively assign paths to children
        for child in &mut elem.children {
            if let JSXChild::ExpressionContainer(ref mut container) = child {
                assign_paths_to_jsx(&mut container.expression, &full_path, path_gen);
            } else if let JSXChild::Element(ref mut child_elem) = child {
                assign_paths_to_jsx(
                    &mut Expression::JSXElement(child_elem.clone()),
                    &full_path,
                    path_gen
                );
            }
        }
    }
}

/// Convert expression to C# code string
/// ✅ Updated to use escape_csharp_string
fn expr_to_csharp(expr: &Expression) -> Str {
    match expr {
        Expression::StringLiteral(lit) => {
            // ✅ Use helper module for proper escaping
            format!("\"{}\"", escape_csharp_string(&lit.value))
        }
        Expression::NumericLiteral(lit) => {
            lit.value.to_string()
        }
        Expression::BooleanLiteral(lit) => {
            if lit.value { "true" } else { "false" }.to_string()
        }
        Expression::NullLiteral(_) => {
            "null".to_string()
        }
        Expression::Identifier(id) => {
            id.name.clone()
        }
        Expression::ArrayExpression(_) => {
            "new List<dynamic>()".to_string()
        }
        Expression::ObjectExpression(_) => {
            "new Dictionary<string, dynamic>()".to_string()
        }
        _ => "null".to_string()
    }
}

// =============================================================================
// C# Code Generation
// =============================================================================

/// Generate C# class for a component
/// Mirrors babel-plugin-minimact/src/generators/csharpFile.cjs
fn generate_csharp_class(component: &ComponentInfo) -> Str {
    let mut builder = CodeBuilder::new();

    // Usings
    builder.append_line("using Minimact.Core;");
    builder.append_line("using Minimact.VDom;");
    builder.append_line("using System.Collections.Generic;");
    builder.newline();

    // Class declaration
    builder.append_line(&format!("public class {} : MinimactComponent", component.name));
    builder.append_line("{");
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

    // Generate client state fields
    for state in &component.use_client_state {
        builder.append_line(&format!(
            "[ClientState] private {} {} = {};",
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

    if component.use_state.len() > 0 || component.use_ref.len() > 0 || component.use_client_state.len() > 0 {
        builder.newline();
    }

    // Generate Render method
    builder.append_line("protected override VNode Render()");
    builder.append_line("{");
    builder.indent();

    if component.render_body.is_some() {
        // TODO: Generate VNode tree from JSX
        builder.append_line("// TODO: Generate VNode from JSX");
        builder.append_line("return new VNull();");
    } else {
        builder.append_line("return new VNull();");
    }

    builder.dedent();
    builder.append_line("}");

    builder.dedent();
    builder.append_line("}");

    builder.to_string()
}
