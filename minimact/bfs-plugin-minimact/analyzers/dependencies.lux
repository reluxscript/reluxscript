/**
 * Dependency Analyzer
 *
 * Analyzes dependencies in JSX expressions
 * Walks the AST to find identifier dependencies and their types
 */

/**
 * Dependency information
 */
pub struct Dependency {
    pub name: Str,
    pub dep_type: Str,  // 'client' or 'server'
}

/**
 * Component state types tracker
 */
pub struct ComponentState {
    pub state_types: HashMap<Str, Str>,  // name -> 'client' or 'server'
}

/**
 * Analyze dependencies in JSX expressions
 *
 * Walks the AST manually to find identifier dependencies and classify them
 * as client-side or server-side based on component state.
 *
 * @param jsx_expr - JSX expression to analyze
 * @param component - Component state with type information
 * @returns Set of dependencies
 */
pub fn analyze_dependencies(jsx_expr: &Expression, component: &ComponentState) -> HashSet<Dependency> {
    let mut deps = HashSet::new();
    walk_expression(jsx_expr, component, &mut deps);
    deps
}

/**
 * Walk an expression to find dependencies
 */
fn walk_expression(node: &Expression, component: &ComponentState, deps: &mut HashSet<Dependency>) {
    match node {
        // Check if this is an identifier that's a state variable
        Expression::Identifier(ref id) => {
            let name = &id.name;
            if let Some(dep_type) = component.state_types.get(name) {
                deps.insert(Dependency {
                    name: name.clone(),
                    dep_type: dep_type.clone(),
                });
            }
        }

        // Conditional expression: test ? consequent : alternate
        Expression::ConditionalExpression(ref cond) => {
            walk_expression(&cond.test, component, deps);
            walk_expression(&cond.consequent, component, deps);
            walk_expression(&cond.alternate, component, deps);
        }

        // Logical expression: left && right, left || right
        Expression::LogicalExpression(ref logical) => {
            walk_expression(&logical.left, component, deps);
            walk_expression(&logical.right, component, deps);
        }

        // Member expression: object.property
        Expression::MemberExpression(ref member) => {
            walk_expression(&member.object, component, deps);
            if !member.computed {
                // For non-computed, property is an identifier but we don't walk it
            } else {
                // For computed (e.g., obj[expr]), walk the property expression
                walk_expression(&member.property, component, deps);
            }
        }

        // Call expression: callee(arg1, arg2, ...)
        Expression::CallExpression(ref call) => {
            walk_expression(&call.callee, component, deps);
            for arg in &call.arguments {
                walk_expression(arg, component, deps);
            }
        }

        // Binary expression: left + right, left === right, etc.
        Expression::BinaryExpression(ref binary) => {
            walk_expression(&binary.left, component, deps);
            walk_expression(&binary.right, component, deps);
        }

        // Unary expression: !arg, -arg, etc.
        Expression::UnaryExpression(ref unary) => {
            walk_expression(&unary.argument, component, deps);
        }

        // Arrow function: (args) => body
        Expression::ArrowFunctionExpression(ref arrow) => {
            walk_expression(&arrow.body, component, deps);
        }

        // Function expression: function(args) { body }
        Expression::FunctionExpression(ref func) => {
            if let Some(ref body) = func.body {
                walk_block_statement(body, component, deps);
            }
        }

        // Array expression: [elem1, elem2, ...]
        Expression::ArrayExpression(ref array) => {
            for element in &array.elements {
                if let Some(ref elem) = element {
                    walk_expression(elem, component, deps);
                }
            }
        }

        // Object expression: { key: value, ... }
        Expression::ObjectExpression(ref obj) => {
            for prop in &obj.properties {
                walk_object_property(prop, component, deps);
            }
        }

        // Template literal: `text ${expr} text`
        Expression::TemplateLiteral(ref template) => {
            for expr in &template.expressions {
                walk_expression(expr, component, deps);
            }
        }

        // Other expression types don't have dependencies we care about
        _ => {}
    }
}

/**
 * Walk a block statement
 */
fn walk_block_statement(block: &BlockStatement, component: &ComponentState, deps: &mut HashSet<Dependency>) {
    for stmt in &block.body {
        walk_statement(stmt, component, deps);
    }
}

/**
 * Walk a statement
 */
fn walk_statement(stmt: &Statement, component: &ComponentState, deps: &mut HashSet<Dependency>) {
    match stmt {
        Statement::ExpressionStatement(ref expr_stmt) => {
            walk_expression(&expr_stmt.expression, component, deps);
        }
        Statement::ReturnStatement(ref ret) => {
            if let Some(ref arg) = ret.argument {
                walk_expression(arg, component, deps);
            }
        }
        Statement::IfStatement(ref if_stmt) => {
            walk_expression(&if_stmt.test, component, deps);
            walk_statement(&if_stmt.consequent, component, deps);
            if let Some(ref alt) = if_stmt.alternate {
                walk_statement(alt, component, deps);
            }
        }
        // Add more statement types as needed
        _ => {}
    }
}

/**
 * Walk an object property
 */
fn walk_object_property(prop: &ObjectProperty, component: &ComponentState, deps: &mut HashSet<Dependency>) {
    match prop {
        ObjectProperty::Property(ref obj_prop) => {
            walk_expression(&obj_prop.value, component, deps);
        }
        ObjectProperty::SpreadElement(ref spread) => {
            walk_expression(&spread.argument, component, deps);
        }
    }
}
