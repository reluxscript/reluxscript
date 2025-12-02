/**
 * Analyze <Plugin name="..." state={...} /> JSX elements in React components
 * Detects plugin usage and extracts metadata for C# code generation
 *
 * Phase 3: Babel Plugin Integration
 *
 * Transforms:
 *   <Plugin name="Clock" state={currentTime} />
 *
 * To C# code:
 *   new PluginNode("Clock", currentTime)
 */

/**
 * Plugin metadata extracted from JSX
 */
pub struct PluginMetadata {
    pub plugin_name: Str,
    pub state_binding: StateBinding,
    pub version: Option<Str>,
}

/**
 * State binding information
 */
pub struct StateBinding {
    pub binding_type: Str,  // 'identifier', 'memberExpression', 'objectExpression', 'complexExpression'
    pub binding: Str,
    pub name: Option<Str>,
    pub state_type: Option<Str>,
}

/**
 * Component state metadata
 */
pub struct ComponentState {
    pub use_state: Vec<UseStateInfo>,
    pub props: Vec<PropInfo>,
    pub local_variables: Vec<LocalVarInfo>,
}

pub struct UseStateInfo {
    pub name: Str,
    pub setter_name: Str,
    pub var_type: Option<Str>,
}

pub struct PropInfo {
    pub name: Str,
    pub prop_type: Option<Str>,
}

pub struct LocalVarInfo {
    pub name: Str,
    pub var_type: Option<Str>,
}

/**
 * Analyze JSX tree for Plugin elements
 *
 * Traverses the component's JSX and extracts metadata for all <Plugin> elements
 *
 * @param func_node - Function declaration node for the component
 * @param component_state - Component metadata being built
 * @returns Vector of plugin usage metadata
 */
pub fn analyze_plugin_usage(func_node: &FunctionDeclaration, component_state: &ComponentState) -> Vec<PluginMetadata> {
    let mut plugin_usages = vec![];

    // Traverse the function body to find JSX elements
    traverse(func_node) {
        fn visit_jsx_element(jsx_node: &mut JSXElement, ctx: &Context) {
            let opening_element = &jsx_node.opening_element;

            // Check if this is a <Plugin> element
            if !is_plugin_element(opening_element) {
                return;
            }

            // Extract plugin metadata
            match extract_plugin_metadata(opening_element, component_state) {
                Ok(plugin_metadata) => {
                    self.plugin_usages.push(plugin_metadata);
                }
                Err(err_msg) => {
                    // In RustScript, we can't easily log to console
                    // Error will propagate up
                }
            }
        }
    }

    plugin_usages
}

/**
 * Check if JSX element is a <Plugin> component
 *
 * @param opening_element - JSX opening element
 * @returns true if it's a Plugin element
 */
pub fn is_plugin_element(opening_element: &JSXOpeningElement) -> bool {
    match &opening_element.name {
        // Check for <Plugin>
        JSXElementName::Identifier(ref id) => {
            id.name == "Plugin"
        }

        // Check for <Plugin.Something>
        JSXElementName::MemberExpression(ref member) => {
            if let JSXMemberExpression::Identifier(ref obj) = member.object {
                obj.name == "Plugin"
            } else {
                false
            }
        }

        _ => false,
    }
}

/**
 * Extract plugin metadata from JSX element
 *
 * @param opening_element - JSX opening element
 * @param component_state - Component metadata
 * @returns Plugin metadata or error
 */
pub fn extract_plugin_metadata(opening_element: &JSXOpeningElement, component_state: &ComponentState) -> Result<PluginMetadata, Str> {
    // Find required attributes
    let name_attr = find_attribute(&opening_element.attributes, "name");
    let state_attr = find_attribute(&opening_element.attributes, "state");
    let version_attr = find_attribute(&opening_element.attributes, "version");

    // Validate required attributes
    if name_attr.is_none() {
        return Err("Plugin element requires 'name' attribute");
    }

    if state_attr.is_none() {
        return Err("Plugin element requires 'state' attribute");
    }

    // Extract plugin name (must be a string literal)
    let plugin_name = extract_plugin_name(&name_attr.unwrap())?;

    // Extract state binding (can be expression or identifier)
    let state_binding = extract_state_binding(&state_attr.unwrap(), component_state)?;

    // Extract optional version
    let version = if let Some(ref v_attr) = version_attr {
        extract_version(v_attr)
    } else {
        None
    };

    Ok(PluginMetadata {
        plugin_name,
        state_binding,
        version,
    })
}

/**
 * Find attribute by name in JSX attributes
 *
 * @param attributes - JSX attributes
 * @param name - Attribute name to find
 * @returns Attribute if found
 */
pub fn find_attribute(attributes: &Vec<JSXAttribute>, name: &Str) -> Option<JSXAttribute> {
    for attr in attributes {
        if let JSXAttribute::Attribute(ref jsx_attr) = attr {
            if let JSXAttributeName::Identifier(ref attr_name) = jsx_attr.name {
                if attr_name.name == name {
                    return Some(attr.clone());
                }
            }
        }
    }
    None
}

/**
 * Extract plugin name from name attribute
 * Must be a string literal (e.g., name="Clock")
 *
 * @param name_attr - JSX attribute node
 * @returns Plugin name or error
 */
fn extract_plugin_name(name_attr: &JSXAttribute) -> Result<Str, Str> {
    if let JSXAttribute::Attribute(ref attr) = name_attr {
        if let Some(ref value) = attr.value {
            match value {
                // String literal: name="Clock"
                JSXAttributeValue::StringLiteral(ref str_lit) => {
                    return Ok(str_lit.value.clone());
                }

                // JSX expression: name={"Clock"}
                JSXAttributeValue::ExpressionContainer(ref container) => {
                    if let Expression::StringLiteral(ref str_lit) = container.expression {
                        return Ok(str_lit.value.clone());
                    }
                }

                _ => {}
            }
        }
    }

    Err("Plugin 'name' attribute must be a string literal (e.g., name=\"Clock\")")
}

/**
 * Extract state binding from state attribute
 * Can be an identifier or expression
 *
 * @param state_attr - JSX attribute node
 * @param component_state - Component metadata
 * @returns State binding metadata or error
 */
fn extract_state_binding(state_attr: &JSXAttribute, component_state: &ComponentState) -> Result<StateBinding, Str> {
    if let JSXAttribute::Attribute(ref attr) = state_attr {
        if let Some(ref value) = attr.value {
            if let JSXAttributeValue::ExpressionContainer(ref container) = value {
                let expr = &container.expression;

                // Simple identifier: state={currentTime}
                if let Expression::Identifier(ref id) = expr {
                    let name = id.name.clone();
                    let state_type = infer_state_type(&name, component_state);

                    return Ok(StateBinding {
                        binding_type: "identifier",
                        binding: name.clone(),
                        name: Some(name),
                        state_type,
                    });
                }

                // Member expression: state={this.state.time}
                if let Expression::MemberExpression(ref member) = expr {
                    let binding = generate_binding_path(member);
                    let state_type = infer_state_type(&binding, component_state);

                    return Ok(StateBinding {
                        binding_type: "memberExpression",
                        binding: binding.clone(),
                        name: None,
                        state_type,
                    });
                }

                // Object expression: state={{ hours: h, minutes: m }}
                if let Expression::ObjectExpression(_) = expr {
                    return Ok(StateBinding {
                        binding_type: "objectExpression",
                        binding: "__inline_object__",
                        name: None,
                        state_type: None,
                    });
                }

                // Any other expression (will be evaluated at runtime)
                return Ok(StateBinding {
                    binding_type: "complexExpression",
                    binding: "__complex__",
                    name: None,
                    state_type: None,
                });
            }
        }
    }

    Err("Plugin 'state' attribute must be a JSX expression (e.g., state={currentTime})")
}

/**
 * Extract version from version attribute
 *
 * @param version_attr - JSX attribute node
 * @returns Version string if valid
 */
fn extract_version(version_attr: &JSXAttribute) -> Option<Str> {
    if let JSXAttribute::Attribute(ref attr) = version_attr {
        if let Some(ref value) = attr.value {
            match value {
                JSXAttributeValue::StringLiteral(ref str_lit) => {
                    return Some(str_lit.value.clone());
                }

                JSXAttributeValue::ExpressionContainer(ref container) => {
                    if let Expression::StringLiteral(ref str_lit) = container.expression {
                        return Some(str_lit.value.clone());
                    }
                }

                _ => {}
            }
        }
    }

    None
}

/**
 * Generate binding path from member expression
 * e.g., this.state.time -> "state.time"
 *
 * @param expr - Member expression AST node
 * @returns Binding path string
 */
fn generate_binding_path(expr: &MemberExpression) -> Str {
    let mut parts = vec![];

    fn traverse_member(node: &Expression, parts: &mut Vec<Str>) {
        match node {
            Expression::Identifier(ref id) => {
                // Skip 'this' prefix
                if id.name != "this" {
                    parts.insert(0, id.name.clone());
                }
            }

            Expression::MemberExpression(ref member) => {
                if let Expression::Identifier(ref prop) = member.property {
                    parts.insert(0, prop.name.clone());
                }
                traverse_member(&member.object, parts);
            }

            _ => {}
        }
    }

    traverse_member(&Expression::MemberExpression(expr.clone()), &mut parts);
    parts.join(".")
}

/**
 * Infer state type from binding name and component metadata
 *
 * @param binding_name - Name of the state binding
 * @param component_state - Component metadata
 * @returns Inferred type or "object" as default
 */
fn infer_state_type(binding_name: &Str, component_state: &ComponentState) -> Option<Str> {
    // Check useState declarations
    for state_decl in &component_state.use_state {
        if state_decl.name == binding_name || state_decl.setter_name == binding_name {
            return state_decl.var_type.clone().or(Some("object"));
        }
    }

    // Check props
    for prop in &component_state.props {
        if prop.name == binding_name {
            return prop.prop_type.clone().or(Some("object"));
        }
    }

    // Check local variables
    for local_var in &component_state.local_variables {
        if local_var.name == binding_name {
            return local_var.var_type.clone().or(Some("object"));
        }
    }

    // Default to object if we can't infer
    Some("object")
}

/**
 * Validate plugin usage (called after analysis)
 *
 * @param plugin_usages - Array of plugin usage metadata
 * @returns Result with error message if validation fails
 */
pub fn validate_plugin_usage(plugin_usages: &Vec<PluginMetadata>) -> Result<(), Str> {
    for plugin in plugin_usages {
        // Validate plugin name format (starts with letter, only letters and numbers)
        if !is_valid_plugin_name(&plugin.plugin_name) {
            return Err(format!(
                "Invalid plugin name '{}'. Plugin names must start with a letter and contain only letters and numbers.",
                plugin.plugin_name
            ));
        }

        // Validate version format if provided
        if let Some(ref version) = plugin.version {
            if !is_valid_semver(version) {
                // Just warn, don't fail
                // In RustScript, we can't easily log warnings
            }
        }
    }

    Ok(())
}

/**
 * Check if plugin name is valid (alphanumeric, starts with letter)
 */
fn is_valid_plugin_name(name: &Str) -> bool {
    if name.is_empty() {
        return false;
    }

    let chars: Vec<char> = name.chars().collect();

    // First character must be a letter
    if !chars[0].is_alphabetic() {
        return false;
    }

    // All characters must be alphanumeric
    for ch in chars {
        if !ch.is_alphanumeric() {
            return false;
        }
    }

    true
}

/**
 * Check if version string is valid semver (X.Y.Z)
 */
fn is_valid_semver(version: &Str) -> bool {
    let parts: Vec<&str> = version.split('.').collect();

    if parts.len() != 3 {
        return false;
    }

    // Each part should be a number
    for part in parts {
        if part.parse::<i32>().is_err() {
            return false;
        }
    }

    true
}
