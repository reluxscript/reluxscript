/**
 * Prop Type Inference
 *
 * Infers C# types for props based on how they're used in the component
 * Analyzes prop usage patterns to determine the most appropriate type
 */

/**
 * Prop usage tracking
 */
pub struct PropUsage {
    pub used_as_boolean: bool,
    pub used_as_number: bool,
    pub used_as_string: bool,
    pub used_as_array: bool,
    pub used_as_object: bool,
    pub has_array_methods: bool,
    pub has_number_operations: bool,
}

impl PropUsage {
    pub fn new() -> PropUsage {
        PropUsage {
            used_as_boolean: false,
            used_as_number: false,
            used_as_string: false,
            used_as_array: false,
            used_as_object: false,
            has_array_methods: false,
            has_number_operations: false,
        }
    }
}

/**
 * Prop information
 */
pub struct Prop {
    pub name: Str,
    pub prop_type: Str,
}

/**
 * Component information
 */
pub struct Component {
    pub props: Vec<Prop>,
}

/**
 * Infer prop types from usage in the component body
 *
 * Analyzes how props are used throughout the component to determine
 * their most likely types
 *
 * @param component - Component with props to analyze
 * @param body - Function body to analyze for prop usage
 */
pub fn infer_prop_types(component: &mut Component, body: &Statement) {
    let mut prop_usage: HashMap<Str, PropUsage> = HashMap::new();

    // Initialize tracking for each prop
    for prop in &component.props {
        prop_usage.insert(prop.name.clone(), PropUsage::new());
    }

    // Traverse the body to analyze prop usage
    analyze_prop_usage_in_statement(body, &mut prop_usage);

    // Now infer types based on usage patterns
    for prop in &mut component.props {
        if prop.prop_type != "dynamic" {
            // Already has explicit type from TypeScript, don't override
            continue;
        }

        if let Some(usage) = prop_usage.get(&prop.name) {
            prop.prop_type = determine_type_from_usage(usage);
        }
    }
}

/**
 * Analyze prop usage in a statement
 */
fn analyze_prop_usage_in_statement(stmt: &Statement, prop_usage: &mut HashMap<Str, PropUsage>) {
    match stmt {
        Statement::BlockStatement(ref block) => {
            for statement in &block.body {
                analyze_prop_usage_in_statement(statement, prop_usage);
            }
        }

        Statement::VariableDeclaration(ref var_decl) => {
            for declarator in &var_decl.declarations {
                if let Some(ref init) = declarator.init {
                    analyze_prop_usage_in_expression(init, prop_usage);
                }
            }
        }

        Statement::ReturnStatement(ref ret) => {
            if let Some(ref arg) = ret.argument {
                analyze_prop_usage_in_expression(arg, prop_usage);
            }
        }

        Statement::ExpressionStatement(ref expr_stmt) => {
            analyze_prop_usage_in_expression(&expr_stmt.expression, prop_usage);
        }

        _ => {}
    }
}

/**
 * Analyze prop usage in an expression
 */
fn analyze_prop_usage_in_expression(expr: &Expression, prop_usage: &mut HashMap<Str, PropUsage>) {
    match expr {
        // Conditional expression (test ? consequent : alternate) - test is boolean
        Expression::ConditionalExpression(ref cond) => {
            if let Some(test_name) = extract_prop_name(&cond.test) {
                if let Some(usage) = prop_usage.get_mut(&test_name) {
                    usage.used_as_boolean = true;
                }
            }
            analyze_prop_usage_in_expression(&cond.consequent, prop_usage);
            analyze_prop_usage_in_expression(&cond.alternate, prop_usage);
        }

        // Logical expression (left && right, left || right) - left is boolean
        Expression::LogicalExpression(ref logical) => {
            if let Some(left_name) = extract_prop_name(&logical.left) {
                if let Some(usage) = prop_usage.get_mut(&left_name) {
                    usage.used_as_boolean = true;
                }
            }
            analyze_prop_usage_in_expression(&logical.right, prop_usage);
        }

        // Call expression - check for array methods
        Expression::CallExpression(ref call) => {
            if let Expression::MemberExpression(ref member) = call.callee {
                if let Some(object_name) = extract_prop_name(&member.object) {
                    if let Expression::Identifier(ref prop) = member.property {
                        let method_name = &prop.name;

                        // Array methods
                        if is_array_method(method_name) {
                            if let Some(usage) = prop_usage.get_mut(&object_name) {
                                usage.used_as_array = true;
                                usage.has_array_methods = true;
                            }
                        }
                    }
                }
            }

            // Recurse into arguments
            for arg in &call.arguments {
                analyze_prop_usage_in_expression(arg, prop_usage);
            }
        }

        // Binary expression - check for number operations
        Expression::BinaryExpression(ref binary) => {
            if is_number_operator(&binary.operator) {
                if let Some(left_name) = extract_prop_name(&binary.left) {
                    if let Some(usage) = prop_usage.get_mut(&left_name) {
                        usage.used_as_number = true;
                        usage.has_number_operations = true;
                    }
                }
                if let Some(right_name) = extract_prop_name(&binary.right) {
                    if let Some(usage) = prop_usage.get_mut(&right_name) {
                        usage.used_as_number = true;
                        usage.has_number_operations = true;
                    }
                }
            }

            analyze_prop_usage_in_expression(&binary.left, prop_usage);
            analyze_prop_usage_in_expression(&binary.right, prop_usage);
        }

        // Member expression - check for property access
        Expression::MemberExpression(ref member) => {
            if let Some(object_name) = extract_prop_name(&member.object) {
                if let Expression::Identifier(ref prop) = member.property {
                    let property_name = &prop.name;

                    if let Some(usage) = prop_usage.get_mut(&object_name) {
                        if property_name == "length" {
                            // Could be array or string
                            usage.used_as_array = true;
                            usage.used_as_string = true;
                        } else {
                            // Accessing a property implies object
                            usage.used_as_object = true;
                        }
                    }
                }
            }

            analyze_prop_usage_in_expression(&member.object, prop_usage);
            if member.computed {
                analyze_prop_usage_in_expression(&member.property, prop_usage);
            }
        }

        // Arrow function
        Expression::ArrowFunctionExpression(ref arrow) => {
            analyze_prop_usage_in_expression(&arrow.body, prop_usage);
        }

        // Array expression
        Expression::ArrayExpression(ref array) => {
            for element in &array.elements {
                if let Some(ref elem) = element {
                    analyze_prop_usage_in_expression(elem, prop_usage);
                }
            }
        }

        _ => {}
    }
}

/**
 * Extract prop name from an expression (unwrap identifiers and member expressions)
 */
fn extract_prop_name(expr: &Expression) -> Option<Str> {
    match expr {
        Expression::Identifier(ref id) => Some(id.name.clone()),

        Expression::MemberExpression(ref member) => extract_prop_name(&member.object),

        _ => None,
    }
}

/**
 * Check if a method name is an array method
 */
fn is_array_method(method_name: &Str) -> bool {
    match method_name.as_str() {
        "map" | "filter" | "forEach" | "find" | "some" | "every" |
        "reduce" | "sort" | "slice" => true,
        _ => false,
    }
}

/**
 * Check if an operator is a number operator
 */
fn is_number_operator(operator: &Str) -> bool {
    match operator.as_str() {
        "+" | "-" | "*" | "/" | "%" | ">" | "<" | ">=" | "<=" => true,
        _ => false,
    }
}

/**
 * Determine type from usage patterns
 */
fn determine_type_from_usage(usage: &PropUsage) -> Str {
    if usage.has_array_methods {
        // Definitely an array if array methods are called
        return "List<dynamic>";
    }

    if usage.used_as_array && !usage.has_number_operations {
        // Used as array (e.g., .length on array)
        return "List<dynamic>";
    }

    if usage.used_as_boolean && !usage.used_as_number &&
       !usage.used_as_string && !usage.used_as_object && !usage.used_as_array {
        // Used only as boolean
        return "bool";
    }

    if usage.has_number_operations && !usage.used_as_boolean && !usage.used_as_array {
        // Used in arithmetic operations
        return "double";
    }

    if usage.used_as_object && !usage.used_as_array && !usage.used_as_boolean {
        // Used as object with property access
        return "dynamic";
    }

    // Keep as dynamic for complex cases
    "dynamic"
}
