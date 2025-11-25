/**
 * Extract JSX element from callback function body
 *
 * Handles various callback formats in .map() calls:
 * - Arrow functions with direct JSX: (...) => <li>...</li>
 * - Functions with block bodies: (...) => { return <li>...</li>; }
 * - Conditional expressions: (...) => condition ? <div/> : <span/>
 * - Logical expressions: (...) => condition && <div/>
 */

/**
 * Extract the JSX element from a callback function
 *
 * @param callback - The callback function (ArrowFunctionExpression or FunctionExpression)
 * @returns The JSX element or None if not found
 */
pub fn extract_jsx_from_callback(callback: &Function) -> Option<JSXElement> {
    match callback {
        Function::ArrowFunctionExpression(ref arrow_fn) => {
            extract_jsx_from_arrow_body(&arrow_fn.body)
        }

        Function::FunctionExpression(ref func_expr) => {
            extract_jsx_from_block_body(&func_expr.body)
        }

        _ => None,
    }
}

/**
 * Extract JSX from arrow function body
 */
fn extract_jsx_from_arrow_body(body: &ArrowFunctionBody) -> Option<JSXElement> {
    match body {
        // Arrow function with direct JSX return: (...) => <li>...</li>
        ArrowFunctionBody::JSXElement(ref jsx_elem) => {
            Some(jsx_elem.clone())
        }

        // Arrow function with expression body
        ArrowFunctionBody::Expression(ref expr) => {
            extract_jsx_from_expression(expr)
        }

        // Arrow function with block body
        ArrowFunctionBody::BlockStatement(ref block) => {
            extract_jsx_from_block_body(block)
        }

        _ => None,
    }
}

/**
 * Extract JSX from block statement (function body with return)
 */
fn extract_jsx_from_block_body(block: &BlockStatement) -> Option<JSXElement> {
    // Find return statement with JSX
    for stmt in &block.body {
        if let Statement::ReturnStatement(ref return_stmt) = stmt {
            if let Some(ref argument) = return_stmt.argument {
                if let Expression::JSXElement(ref jsx_elem) = argument {
                    return Some(jsx_elem.clone());
                }
            }
        }
    }

    None
}

/**
 * Extract JSX from various expression forms
 */
fn extract_jsx_from_expression(expr: &Expression) -> Option<JSXElement> {
    match expr {
        // Direct JSX element
        Expression::JSXElement(ref jsx_elem) => {
            Some(jsx_elem.clone())
        }

        // Conditional: condition ? <div/> : <span/>
        // For now, take the consequent (true branch)
        Expression::ConditionalExpression(ref cond_expr) => {
            if let Expression::JSXElement(ref jsx_elem) = cond_expr.consequent {
                Some(jsx_elem.clone())
            } else {
                None
            }
        }

        // Logical AND: condition && <div/>
        Expression::LogicalExpression(ref log_expr) => {
            if log_expr.operator == LogicalOperator::And {
                if let Expression::JSXElement(ref jsx_elem) = log_expr.right {
                    return Some(jsx_elem.clone());
                }
            }
            None
        }

        _ => None,
    }
}
