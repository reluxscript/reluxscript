/// Test conversion of extract_use_state from generated-swc
///
/// Original Rust function:
/// pub fn extract_use_state(call: &CallExpr, var_name: &Pat, component: &mut Component, hook_type: &str)

plugin ExtractUseStateTest {
    struct UseStateInfo {
        var_name: Str,
        setter_name: Option<Str>,
        initial_value: Str,
        state_type: Str,
        is_client_state: bool,
    }

    struct Component {
        use_state: Vec<UseStateInfo>,
        use_client_state: Vec<UseStateInfo>,
    }

    /// Extract useState or useClientState hook
    /// const [value, setValue] = useState(initial)
    pub fn extract_use_state(call: &CallExpression, var_name: &Pattern, component: &mut Component, hook_type: &Str) {
        // Check if var_name is an ArrayPattern
        if matches!(var_name, ArrayPattern) {
            let arr = var_name.clone();

            // Extract state variable name from first element
            let mut state_var: Option<Str> = None;
            if arr.elements.len() > 0 {
                let first_elem = &arr.elements[0];
                if matches!(first_elem, Identifier) {
                    state_var = Some(first_elem.name.clone());
                }
            }

            // Extract setter name from second element
            let mut setter_var: Option<Str> = None;
            if arr.elements.len() > 1 {
                let second_elem = &arr.elements[1];
                if matches!(second_elem, Identifier) {
                    setter_var = Some(second_elem.name.clone());
                }
            }

            // Only proceed if we got a state variable name
            if let Some(name) = state_var {
                // Get initial value from first argument
                let mut initial_value = "null";
                let mut state_type = "dynamic";

                if call.arguments.len() > 0 {
                    let first_arg = &call.arguments[0];
                    initial_value = generate_csharp_expression(first_arg);
                    state_type = infer_csharp_type(first_arg);
                }

                // Create UseStateInfo
                let info = UseStateInfo {
                    var_name: name.clone(),
                    setter_name: setter_var,
                    initial_value: initial_value.to_string(),
                    state_type: state_type.to_string(),
                    is_client_state: hook_type == "useClientState",
                };

                // Push to appropriate list
                if hook_type == "useClientState" {
                    component.use_client_state.push(info);
                } else {
                    component.use_state.push(info);
                }
            }
        }
    }

    /// Generate C# expression from AST node (stub)
    fn generate_csharp_expression(expr: &Expression) -> Str {
        if matches!(expr, StringLiteral) {
            let s = expr.value.clone();
            return format!("\"{}\"", s);
        } else if matches!(expr, NumericLiteral) {
            return expr.value.to_string();
        } else if matches!(expr, BooleanLiteral) {
            return expr.value.to_string();
        } else if matches!(expr, NullLiteral) {
            return "null";
        } else if matches!(expr, Identifier) {
            return expr.name.clone();
        }
        return "null";
    }

    /// Infer C# type from expression (stub)
    fn infer_csharp_type(expr: &Expression) -> Str {
        if matches!(expr, StringLiteral) {
            return "string";
        } else if matches!(expr, NumericLiteral) {
            return "int";
        } else if matches!(expr, BooleanLiteral) {
            return "bool";
        } else if matches!(expr, ArrayExpression) {
            return "List<dynamic>";
        } else if matches!(expr, ObjectExpression) {
            return "Dictionary<string, dynamic>";
        }
        return "dynamic";
    }
}
