/**
 * Hook Detection Module
 *
 * Detects custom hooks based on the pattern:
 * function use{Name}(namespace: string, ...args) { ... }
 *
 * Requirements:
 * 1. Function name starts with 'use'
 * 2. First parameter is named 'namespace' (string type)
 * 3. Contains at least one useState call OR returns JSX
 */

/**
 * Check if a node represents a custom hook definition
 *
 * @param node - Function declaration or variable declarator
 * @returns True if this is a custom hook
 */
pub fn is_custom_hook(node: &Statement) -> bool {
    match node {
        // Handle function declaration: function useCounter(namespace, start) { ... }
        Statement::FunctionDeclaration(ref func) => {
            is_custom_hook_function(func)
        }

        // Handle variable declarator with arrow function
        Statement::VariableDeclaration(ref var_decl) => {
            for declarator in &var_decl.declarations {
                if let Some(ref init) = declarator.init {
                    if let Expression::ArrowFunctionExpression(ref arrow) = init {
                        if let Pattern::Identifier(ref id) = declarator.id {
                            return is_custom_hook_arrow(&id.name, &arrow.params);
                        }
                    } else if let Expression::FunctionExpression(ref func_expr) = init {
                        if let Pattern::Identifier(ref id) = declarator.id {
                            return is_custom_hook_arrow(&id.name, &func_expr.params);
                        }
                    }
                }
            }
            false
        }

        _ => false,
    }
}

/**
 * Check if function declaration is a custom hook
 */
fn is_custom_hook_function(func: &FunctionDeclaration) -> bool {
    // Must have a name
    let name = match &func.id {
        Some(ref id) => &id.name,
        None => return false,
    };

    // Must start with 'use'
    if !name.starts_with("use") {
        return false;
    }

    // Must have at least one parameter
    if func.params.is_empty() {
        return false;
    }

    // First parameter must be 'namespace'
    is_namespace_parameter(&func.params[0])
}

/**
 * Check if arrow function is a custom hook
 */
fn is_custom_hook_arrow(name: &Str, params: &Vec<Pattern>) -> bool {
    // Must start with 'use'
    if !name.starts_with("use") {
        return false;
    }

    // Must have at least one parameter
    if params.is_empty() {
        return false;
    }

    // First parameter must be 'namespace'
    is_namespace_parameter(&params[0])
}

/**
 * Check if a parameter is the required 'namespace' parameter
 *
 * @param param - Parameter pattern
 * @returns True if this is a valid namespace parameter
 */
pub fn is_namespace_parameter(param: &Pattern) -> bool {
    match param {
        // Handle simple identifier: namespace
        Pattern::Identifier(ref id) => {
            id.name == "namespace"
        }

        // Handle TypeScript annotation: namespace: string
        Pattern::AssignmentPattern(ref assign) => {
            // Check the left side
            if let Pattern::Identifier(ref id) = assign.left {
                if id.name == "namespace" {
                    // Optionally verify it's typed as string
                    if let Some(ref type_ann) = id.type_annotation {
                        if let TSType::TSStringKeyword = type_ann.type_annotation {
                            return true;
                        }
                    }
                    return true;
                }
            }
            false
        }

        _ => false,
    }
}

/**
 * Get the hook name from a statement
 *
 * @param node - Statement node (function or variable declaration)
 * @returns Hook name or None
 */
pub fn get_hook_name(node: &Statement) -> Option<Str> {
    match node {
        Statement::FunctionDeclaration(ref func) => {
            func.id.as_ref().map(|id| id.name.clone())
        }

        Statement::VariableDeclaration(ref var_decl) => {
            if let Some(ref declarator) = var_decl.declarations.get(0) {
                if let Pattern::Identifier(ref id) = declarator.id {
                    return Some(id.name.clone());
                }
            }
            None
        }

        _ => None,
    }
}

/**
 * Hook parameter information
 */
pub struct HookParameter {
    pub name: Str,
    pub param_type: Str,
    pub default_value: Option<Expression>,
}

/**
 * Get all parameters of a hook (excluding namespace)
 *
 * @param node - Statement node (function or variable declaration)
 * @returns Vector of parameter information
 */
pub fn get_hook_parameters(node: &Statement) -> Vec<HookParameter> {
    let params = match node {
        Statement::FunctionDeclaration(ref func) => {
            &func.params
        }

        Statement::VariableDeclaration(ref var_decl) => {
            if let Some(ref declarator) = var_decl.declarations.get(0) {
                if let Some(ref init) = declarator.init {
                    match init {
                        Expression::ArrowFunctionExpression(ref arrow) => &arrow.params,
                        Expression::FunctionExpression(ref func_expr) => &func_expr.params,
                        _ => return vec![],
                    }
                } else {
                    return vec![];
                }
            } else {
                return vec![];
            }
        }

        _ => return vec![],
    };

    // Skip first parameter (namespace)
    if params.len() <= 1 {
        return vec![];
    }

    let mut hook_params = vec![];

    for param in &params[1..] {
        let param_info = extract_parameter_info(param);
        hook_params.push(param_info);
    }

    hook_params
}

/**
 * Extract parameter information from a pattern
 */
fn extract_parameter_info(param: &Pattern) -> HookParameter {
    match param {
        Pattern::Identifier(ref id) => {
            let param_type = if let Some(ref type_ann) = id.type_annotation {
                extract_type_string(&type_ann.type_annotation)
            } else {
                "any"
            };

            HookParameter {
                name: id.name.clone(),
                param_type,
                default_value: None,
            }
        }

        Pattern::AssignmentPattern(ref assign) => {
            // Has default value: start = 0
            let (name, param_type) = if let Pattern::Identifier(ref id) = assign.left {
                let param_type = if let Some(ref type_ann) = id.type_annotation {
                    extract_type_string(&type_ann.type_annotation)
                } else {
                    "any"
                };
                (id.name.clone(), param_type)
            } else {
                ("unknown", "any")
            };

            HookParameter {
                name,
                param_type,
                default_value: Some(assign.right.clone()),
            }
        }

        _ => HookParameter {
            name: "unknown",
            param_type: "any",
            default_value: None,
        },
    }
}

/**
 * Extract TypeScript type as string
 *
 * @param type_node - TypeScript type node
 * @returns Type as string
 */
pub fn extract_type_string(type_node: &TSType) -> Str {
    match type_node {
        TSType::TSStringKeyword => "string",
        TSType::TSNumberKeyword => "number",
        TSType::TSBooleanKeyword => "boolean",
        TSType::TSAnyKeyword => "any",

        TSType::TSArrayType(ref array_type) => {
            let element_type = extract_type_string(&array_type.element_type);
            format!("{}[]", element_type)
        }

        TSType::TSTypeReference(ref type_ref) => {
            if let TSEntityName::Identifier(ref id) = type_ref.type_name {
                id.name.clone()
            } else {
                "any"
            }
        }

        _ => "any",
    }
}

/**
 * Get the function body from a hook statement
 *
 * @param node - Statement node
 * @returns Function body or None
 */
pub fn get_hook_body(node: &Statement) -> Option<BlockStatement> {
    match node {
        Statement::FunctionDeclaration(ref func) => {
            func.body.clone()
        }

        Statement::VariableDeclaration(ref var_decl) => {
            if let Some(ref declarator) = var_decl.declarations.get(0) {
                if let Some(ref init) = declarator.init {
                    match init {
                        Expression::ArrowFunctionExpression(ref arrow) => {
                            if let Expression::BlockStatement(ref block) = arrow.body {
                                return Some(block.clone());
                            }
                        }
                        Expression::FunctionExpression(ref func_expr) => {
                            return func_expr.body.clone();
                        }
                        _ => {}
                    }
                }
            }
            None
        }

        _ => None,
    }
}
