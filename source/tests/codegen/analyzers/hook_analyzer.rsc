/**
 * Hook Analyzer Module
 *
 * Analyzes a custom hook to extract:
 * - useState calls → [State] fields
 * - Methods (arrow functions, function declarations)
 * - JSX elements → Render() method
 * - Return values → API surface
 */

use "./hook_detector.rsc" { get_hook_name, get_hook_parameters, get_hook_body, extract_type_string, HookParameter };

/**
 * Hook analysis result
 */
pub struct HookAnalysis {
    pub name: Str,
    pub class_name: Str,
    pub params: Vec<HookParameter>,
    pub states: Vec<StateInfo>,
    pub methods: Vec<MethodInfo>,
    pub jsx_elements: Vec<JSXInfo>,
    pub return_values: Vec<ReturnValueInfo>,
    pub event_handlers: Vec<EventHandlerInfo>,
}

/**
 * State information from useState
 */
pub struct StateInfo {
    pub var_name: Str,
    pub setter_name: Str,
    pub state_type: Str,
    pub initial_value: Option<Str>,
}

/**
 * Method information
 */
pub struct MethodInfo {
    pub name: Str,
    pub params: Vec<MethodParam>,
    pub return_type: Str,
    pub body: Str,
}

pub struct MethodParam {
    pub name: Str,
    pub param_type: Str,
}

/**
 * JSX information
 */
pub struct JSXInfo {
    pub jsx_type: Str,  // 'variable', 'inline', 'conditional'
    pub var_name: Option<Str>,
}

/**
 * Return value information
 */
pub struct ReturnValueInfo {
    pub index: i32,
    pub name: Str,
    pub value_type: Str,
}

/**
 * Event handler information
 */
pub struct EventHandlerInfo {
    pub name: Str,
    pub event_type: Str,
}

/**
 * Analyze a custom hook and extract all relevant information
 *
 * @param node - Statement node (function or variable declaration)
 * @returns Hook analysis result
 */
pub fn analyze_hook(node: &Statement) -> HookAnalysis {
    let name = get_hook_name(node).unwrap_or("UnknownHook".to_string());
    let params = get_hook_parameters(node);
    let body = get_hook_body(node);

    let mut analysis = HookAnalysis {
        name: name.clone(),
        class_name: format!("{}Hook", capitalize(&name)),
        params,
        states: vec![],
        methods: vec![],
        jsx_elements: vec![],
        return_values: vec![],
        event_handlers: vec![],
    };

    if let Some(ref body_block) = body {
        // Extract useState calls
        extract_use_state_calls(body_block, &mut analysis.states);

        // Extract methods
        extract_methods(body_block, &mut analysis.methods);

        // Extract return values and JSX
        extract_return_info(body_block, &mut analysis);
    }

    // Improve return value type inference
    infer_return_value_types(&mut analysis);

    analysis
}

/**
 * Extract useState calls from block statement
 */
fn extract_use_state_calls(block: &BlockStatement, states: &mut Vec<StateInfo>) {
    for stmt in &block.body {
        if let Statement::VariableDeclaration(ref var_decl) = stmt {
            for declarator in &var_decl.declarations {
                if let Some(state_info) = extract_state_from_declarator(declarator) {
                    states.push(state_info);
                }
            }
        }
    }
}

/**
 * Extract state from variable declarator with useState
 */
fn extract_state_from_declarator(declarator: &VariableDeclarator) -> Option<StateInfo> {
    // Pattern: const [value, setValue] = useState(initial);
    if let Pattern::ArrayPattern(ref array_pattern) = declarator.id {
        if array_pattern.elements.len() != 2 {
            return None;
        }

        // Get value and setter names
        let value_name = if let Some(ref elem) = array_pattern.elements[0] {
            if let Pattern::Identifier(ref id) = elem {
                id.name.clone()
            } else {
                return None;
            }
        } else {
            return None;
        };

        let setter_name = if let Some(ref elem) = array_pattern.elements[1] {
            if let Pattern::Identifier(ref id) = elem {
                id.name.clone()
            } else {
                return None;
            }
        } else {
            return None;
        };

        // Check if init is useState call
        if let Some(ref init) = declarator.init {
            if let Expression::CallExpression(ref call) = init {
                if let Expression::Identifier(ref callee) = call.callee {
                    if callee.name == "useState" {
                        // Extract initial value
                        let (initial_value, inferred_type) = if !call.arguments.is_empty() {
                            let init_expr = &call.arguments[0];
                            (Some(generate_expression_code(init_expr)), infer_type(init_expr))
                        } else {
                            (None, "any".to_string())
                        };

                        return Some(StateInfo {
                            var_name: value_name,
                            setter_name,
                            state_type: inferred_type,
                            initial_value,
                        });
                    }
                }
            }
        }
    }

    None
}

/**
 * Extract methods from block statement
 */
fn extract_methods(block: &BlockStatement, methods: &mut Vec<MethodInfo>) {
    for stmt in &block.body {
        match stmt {
            // const method = () => { ... }
            Statement::VariableDeclaration(ref var_decl) => {
                for declarator in &var_decl.declarations {
                    if let Some(method_info) = extract_method_from_declarator(declarator) {
                        methods.push(method_info);
                    }
                }
            }

            // function method() { ... }
            Statement::FunctionDeclaration(ref func) => {
                if let Some(method_info) = extract_method_from_function(func) {
                    methods.push(method_info);
                }
            }

            _ => {}
        }
    }
}

/**
 * Extract method from variable declarator
 */
fn extract_method_from_declarator(declarator: &VariableDeclarator) -> Option<MethodInfo> {
    if let Pattern::Identifier(ref id) = declarator.id {
        if let Some(ref init) = declarator.init {
            match init {
                Expression::ArrowFunctionExpression(ref arrow) => {
                    let params = extract_method_params(&arrow.params);
                    let body = generate_method_body(&arrow.body);

                    return Some(MethodInfo {
                        name: id.name.clone(),
                        params,
                        return_type: "void".to_string(),
                        body,
                    });
                }

                Expression::FunctionExpression(ref func_expr) => {
                    let params = extract_method_params(&func_expr.params);
                    let body = if let Some(ref body_block) = func_expr.body {
                        generate_block_code(body_block)
                    } else {
                        "/* empty */".to_string()
                    };

                    return Some(MethodInfo {
                        name: id.name.clone(),
                        params,
                        return_type: "void".to_string(),
                        body,
                    });
                }

                _ => {}
            }
        }
    }

    None
}

/**
 * Extract method from function declaration
 */
fn extract_method_from_function(func: &FunctionDeclaration) -> Option<MethodInfo> {
    if let Some(ref id) = func.id {
        let params = extract_method_params(&func.params);
        let body = if let Some(ref body_block) = func.body {
            generate_block_code(body_block)
        } else {
            "/* empty */".to_string()
        };

        return Some(MethodInfo {
            name: id.name.clone(),
            params,
            return_type: "void".to_string(),
            body,
        });
    }

    None
}

/**
 * Extract method parameters
 */
fn extract_method_params(params: &Vec<Pattern>) -> Vec<MethodParam> {
    let mut method_params = vec![];

    for param in params {
        if let Pattern::Identifier(ref id) = param {
            let param_type = if let Some(ref type_ann) = id.type_annotation {
                extract_type_string(&type_ann.type_annotation)
            } else {
                "any".to_string()
            };

            method_params.push(MethodParam {
                name: id.name.clone(),
                param_type,
            });
        }
    }

    method_params
}

/**
 * Extract return information (JSX and return values)
 */
fn extract_return_info(block: &BlockStatement, analysis: &mut HookAnalysis) {
    // Find return statements
    for stmt in &block.body {
        if let Statement::ReturnStatement(ref ret_stmt) = stmt {
            if let Some(ref arg) = ret_stmt.argument {
                // Extract JSX
                if has_jsx(arg) {
                    let jsx_type = determine_jsx_type(arg);
                    analysis.jsx_elements.push(JSXInfo {
                        jsx_type,
                        var_name: None,
                    });
                }

                // Extract return values
                let return_vals = extract_return_values(arg);
                analysis.return_values = return_vals;
            }
        }
    }
}

/**
 * Extract return values from return expression
 */
fn extract_return_values(expr: &Expression) -> Vec<ReturnValueInfo> {
    let mut values = vec![];

    match expr {
        // Array: return [value, setValue, ui];
        Expression::ArrayExpression(ref array) => {
            for (index, element) in array.elements.iter().enumerate() {
                if let Some(ref elem) = element {
                    if let Expression::Identifier(ref id) = elem {
                        values.push(ReturnValueInfo {
                            index: index as i32,
                            name: id.name.clone(),
                            value_type: "unknown".to_string(),
                        });
                    } else if is_jsx_expression(elem) {
                        values.push(ReturnValueInfo {
                            index: index as i32,
                            name: format!("ui_{}", index),
                            value_type: "jsx".to_string(),
                        });
                    }
                }
            }
        }

        // Object: return { value, setValue };
        Expression::ObjectExpression(ref obj) => {
            for (index, prop) in obj.properties.iter().enumerate() {
                if let ObjectProperty::Property(ref obj_prop) = prop {
                    if let Expression::Identifier(ref key) = obj_prop.key {
                        values.push(ReturnValueInfo {
                            index: index as i32,
                            name: key.name.clone(),
                            value_type: "unknown".to_string(),
                        });
                    }
                }
            }
        }

        // Single value: return value;
        Expression::Identifier(ref id) => {
            values.push(ReturnValueInfo {
                index: 0,
                name: id.name.clone(),
                value_type: "unknown".to_string(),
            });
        }

        _ => {}
    }

    values
}

/**
 * Infer return value types based on analysis context
 */
fn infer_return_value_types(analysis: &mut HookAnalysis) {
    for ret_val in &mut analysis.return_values {
        ret_val.value_type = infer_return_value_type(&ret_val.name, analysis);
    }
}

/**
 * Infer return value type from name and context
 */
fn infer_return_value_type(name: &Str, analysis: &HookAnalysis) -> Str {
    // Check if it's a state setter
    for state in &analysis.states {
        if state.setter_name == name {
            return "setter".to_string();
        }
        if state.var_name == name {
            return "state".to_string();
        }
    }

    // Check if it's a method
    for method in &analysis.methods {
        if method.name == name {
            return "method".to_string();
        }
    }

    // Name-based inference
    if name.starts_with("set") {
        return "setter".to_string();
    }
    if name == "ui" || name.ends_with("UI") {
        return "jsx".to_string();
    }
    if name.contains("handle") || name.contains("on") {
        return "method".to_string();
    }

    "state".to_string()
}

/**
 * Helper: Capitalize first letter
 */
fn capitalize(s: &Str) -> Str {
    if s.is_empty() {
        return s.clone();
    }

    let mut chars = s.chars();
    let first = chars.next().unwrap().to_uppercase().to_string();
    let rest: String = chars.collect();

    format!("{}{}", first, rest)
}

/**
 * Helper: Infer type from expression
 */
fn infer_type(expr: &Expression) -> Str {
    match expr {
        Expression::NumericLiteral(_) => "number",
        Expression::StringLiteral(_) => "string",
        Expression::BooleanLiteral(_) => "boolean",
        Expression::ArrayExpression(_) => "any[]",
        Expression::ObjectExpression(_) => "object",
        Expression::NullLiteral => "object",
        _ => "any",
    }
}

/**
 * Helper: Check if expression contains JSX
 */
fn has_jsx(expr: &Expression) -> bool {
    is_jsx_expression(expr) || contains_jsx_in_array(expr) || contains_jsx_in_logical(expr)
}

/**
 * Helper: Check if expression is JSX
 */
fn is_jsx_expression(expr: &Expression) -> bool {
    matches!(expr, Expression::JSXElement(_)) || matches!(expr, Expression::JSXFragment(_))
}

/**
 * Helper: Check if array contains JSX
 */
fn contains_jsx_in_array(expr: &Expression) -> bool {
    if let Expression::ArrayExpression(ref array) = expr {
        for element in &array.elements {
            if let Some(ref elem) = element {
                if is_jsx_expression(elem) {
                    return true;
                }
            }
        }
    }
    false
}

/**
 * Helper: Check if logical expression contains JSX
 */
fn contains_jsx_in_logical(expr: &Expression) -> bool {
    if let Expression::LogicalExpression(ref logical) = expr {
        return is_jsx_expression(&logical.right);
    }
    false
}

/**
 * Helper: Determine JSX type
 */
fn determine_jsx_type(expr: &Expression) -> Str {
    if is_jsx_expression(expr) {
        return "inline".to_string();
    }
    if matches!(expr, Expression::LogicalExpression(_)) {
        return "conditional".to_string();
    }
    if let Expression::Identifier(_) = expr {
        return "variable".to_string();
    }
    "unknown".to_string()
}

/**
 * Helper: Generate expression code (simplified)
 */
fn generate_expression_code(expr: &Expression) -> Str {
    match expr {
        Expression::NumericLiteral(ref num) => num.value.to_string(),
        Expression::StringLiteral(ref str_lit) => format!("\"{}\"", str_lit.value),
        Expression::BooleanLiteral(ref bool_lit) => {
            if bool_lit.value { "true" } else { "false" }
        }
        Expression::NullLiteral => "null",
        Expression::Identifier(ref id) => id.name.clone(),
        _ => "/* complex expression */".to_string(),
    }
}

/**
 * Helper: Generate method body code
 */
fn generate_method_body(body: &Expression) -> Str {
    // Simplified - in full version would use transpiler
    "/* method body */".to_string()
}

/**
 * Helper: Generate block code
 */
fn generate_block_code(block: &BlockStatement) -> Str {
    // Simplified - in full version would use transpiler
    "/* block code */".to_string()
}
