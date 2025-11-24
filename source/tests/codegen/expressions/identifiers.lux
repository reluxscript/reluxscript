/**
 * Identifier and member expression handlers
 *
 * Generates C# code for JavaScript/TypeScript identifiers and property access
 */

/**
 * Component metadata for custom hooks
 */
pub struct ComponentMetadata {
    pub custom_hooks: Vec<HookInstance>,
}

pub struct HookInstance {
    pub hook_name: Str,
    pub namespace: Str,
    pub return_values: Vec<Str>,
    pub metadata: Option<HookMetadata>,
}

pub struct HookMetadata {
    pub return_values: Vec<ReturnValueMetadata>,
}

pub struct ReturnValueMetadata {
    pub name: Str,
    pub value_type: Str, // "state", "method", "jsx"
}

/**
 * Generate identifier expression
 *
 * @param node - Identifier node
 * @param current_component - Optional component metadata for custom hooks
 * @returns C# identifier code
 */
pub fn generate_identifier(
    node: &Identifier,
    current_component: Option<&ComponentMetadata>
) -> Str {
    // Special case: 'state' identifier (state proxy)
    // Note: This should only happen as part of member expression (state.key or state["key"])
    // Standalone 'state' reference is unusual - warn but transpile to 'State'
    if node.name == "state" {
        console_warn("[Babel Plugin] Naked state reference detected (should be state.key or state[\"key\"])");
        return "State".to_string();
    }

    // Check if this identifier is a custom hook return value
    if let Some(component) = current_component {
        for hook_instance in &component.custom_hooks {
            if let Some(ref metadata) = hook_instance.metadata {
                if let Some(return_value_index) = hook_instance.return_values.iter().position(|rv| rv == &node.name) {
                    if let Some(return_value_metadata) = metadata.return_values.get(return_value_index) {
                        // Skip UI return values (they're handled separately as VComponentWrapper)
                        if return_value_metadata.value_type == "jsx" {
                            return node.name.clone(); // Keep as-is for UI variables
                        }

                        // Replace with lifted state access for state/method returns
                        if return_value_metadata.value_type == "state" {
                            // Access the hook's lifted state: State["hookNamespace.stateVarName"]
                            let lifted_state_path = format!("{}.{}", hook_instance.namespace, return_value_metadata.name);
                            return format!("GetState<dynamic>(\"{}\")", lifted_state_path);
                        } else if return_value_metadata.value_type == "method" {
                            // For methods, we can't call them from parent - warn and keep as-is for now
                            console_warn(&format!(
                                "[Custom Hook] Cannot access method '{}' from hook '{}' in parent component",
                                node.name, hook_instance.hook_name
                            ));
                            return node.name.clone();
                        }
                    }
                }
            }
        }
    }

    node.name.clone()
}

/**
 * Generate member expression
 *
 * @param node - MemberExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @param in_interpolation - Whether inside string interpolation
 * @returns C# member access code
 */
pub fn generate_member_expression<F>(
    node: &MemberExpression,
    generate_csharp_expression: F,
    in_interpolation: bool
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    // Special case: state.key or state["key"] (state proxy)
    if let Expression::Identifier(ref id) = node.object {
        if id.name == "state" {
            if node.computed {
                // state["someKey"] or state["Child.key"] → State["someKey"] or State["Child.key"]
                let key = generate_csharp_expression(&node.property, in_interpolation);
                return format!("State[{}]", key);
            } else {
                // state.someKey → State["someKey"]
                if let Expression::Identifier(ref prop) = node.property {
                    return format!("State[\"{}\"]", prop.name);
                }
            }
        }
    }

    let object = generate_csharp_expression(&node.object, in_interpolation);

    // Get property name if it's an identifier
    let property_name = if let Expression::Identifier(ref id) = node.property {
        Some(id.name.clone())
    } else {
        None
    };

    // Handle ref.current → just ref (refs in C# are the value itself, not a container)
    if let Some(ref prop) = property_name {
        if prop == "current" && !node.computed {
            if let Expression::Identifier(ref obj_id) = node.object {
                // Check if the object is a ref variable (ends with "Ref")
                if obj_id.name.ends_with("Ref") {
                    return object; // Return just the ref variable name without .current
                }
            }
        }
    }

    // Handle JavaScript to C# API conversions
    if let Some(ref prop) = property_name {
        if !node.computed {
            match prop.as_str() {
                // array.length → array.Count
                "length" => return format!("{}.Count", object),

                // e.target → e.Target
                "target" => return format!("{}.Target", object),

                // e.value → e.Value (capitalize for C# property convention)
                "value" => return format!("{}.Value", object),

                // e.checked → e.Checked
                "checked" => return format!("{}.Checked", object),

                // err.message → err.Message
                "message" => return format!("{}.Message", object),

                // response.ok → response.IsSuccessStatusCode
                "ok" => return format!("{}.IsSuccessStatusCode", object),

                _ => {}
            }
        }
    }

    // Default member access
    let property = if node.computed {
        format!("[{}]", generate_csharp_expression(&node.property, in_interpolation))
    } else if let Some(prop) = property_name {
        format!(".{}", prop)
    } else {
        format!("[{}]", generate_csharp_expression(&node.property, in_interpolation))
    };

    format!("{}{}", object, property)
}

/**
 * Generate optional member expression (optional chaining)
 *
 * @param node - OptionalMemberExpression node
 * @param generate_csharp_expression - Function to generate C# expression code
 * @param in_interpolation - Whether inside string interpolation
 * @returns C# optional member access code
 */
pub fn generate_optional_member_expression<F>(
    node: &OptionalMemberExpression,
    generate_csharp_expression: F,
    in_interpolation: bool
) -> Str
where
    F: Fn(&Expression, bool) -> Str
{
    let object = generate_csharp_expression(&node.object, in_interpolation);

    // Get property name if it's an identifier
    let property_name = if let Expression::Identifier(ref id) = node.property {
        Some(id.name.clone())
    } else {
        None
    };

    // Capitalize first letter for C# property convention (userEmail → UserEmail)
    let csharp_property = property_name.as_ref().map(|prop| {
        if prop.is_empty() {
            prop.clone()
        } else {
            let mut chars = prop.chars();
            let first = chars.next().unwrap().to_uppercase().to_string();
            first + chars.as_str()
        }
    });

    // Generate property access
    let property = if node.computed {
        format!("?[{}]", generate_csharp_expression(&node.property, in_interpolation))
    } else if let Some(csharp_prop) = csharp_property {
        format!("?.{}", csharp_prop)
    } else {
        format!("?[{}]", generate_csharp_expression(&node.property, in_interpolation))
    };

    format!("{}{}", object, property)
}

/**
 * Console warning helper (placeholder for logging)
 */
fn console_warn(message: &str) {
    // In real implementation, this would log to console or collect warnings
    // For now, we'll just note it in comments or handle it at runtime
}
