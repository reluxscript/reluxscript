/**
 * Pattern Detection
 *
 * Detects various patterns in JSX code including:
 * - Spread props
 * - Dynamic children (map, conditionals, etc.)
 * - Complex props (conditional expressions, logical expressions)
 */

/**
 * Detect if attributes contain spread operators
 *
 * @param attributes - JSX attributes array
 * @returns true if any spread attributes are found
 */
pub fn has_spread_props(attributes: &Vec<JSXAttribute>) -> bool {
    for attr in attributes {
        if matches!(attr, JSXAttribute::SpreadAttribute(_)) {
            return true;
        }
    }
    false
}

/**
 * Detect if children contain dynamic patterns (like .map())
 *
 * Checks for:
 * - Array map() calls
 * - LINQ-style Select/ToArray calls
 * - Conditional expressions with JSX
 * - Logical expressions with JSX
 *
 * @param children - JSX children array
 * @returns true if dynamic children are found
 */
pub fn has_dynamic_children(children: &Vec<JSXChild>) -> bool {
    for child in children {
        if let JSXChild::ExpressionContainer(ref container) = child {
            let expr = &container.expression;

            // Check for .map() calls
            if let Expression::CallExpression(ref call) = expr {
                if let Expression::MemberExpression(ref member) = call.callee {
                    if let Expression::Identifier(ref prop) = member.property {
                        if prop.name == "map" {
                            return true;
                        }
                    }
                }

                // Check for LINQ-style Select/ToArray calls
                if let Expression::MemberExpression(ref member) = call.callee {
                    if let Expression::Identifier(ref prop) = member.property {
                        if prop.name == "Select" || prop.name == "ToArray" {
                            return true;
                        }
                    }
                }
            }

            // Check for conditionals with JSX: {condition ? <A/> : <B/>}
            if let Expression::ConditionalExpression(ref cond) = expr {
                if is_jsx_expression(&cond.consequent) || is_jsx_expression(&cond.alternate) {
                    return true;
                }
            }

            // Check for logical expressions with JSX: {condition && <Element/>}
            if let Expression::LogicalExpression(ref logical) = expr {
                if is_jsx_expression(&logical.right) {
                    return true;
                }
            }
        }
    }

    false
}

/**
 * Detect if props contain complex expressions
 *
 * Checks for:
 * - Conditional expressions in props
 * - Logical expressions in props
 *
 * @param attributes - JSX attributes array
 * @returns true if complex props are found
 */
pub fn has_complex_props(attributes: &Vec<JSXAttribute>) -> bool {
    for attr in attributes {
        if let JSXAttribute::Attribute(ref jsx_attr) = attr {
            if let Some(ref value) = jsx_attr.value {
                if let JSXAttributeValue::ExpressionContainer(ref container) = value {
                    let expr = &container.expression;

                    // Check for conditional or logical expressions
                    if matches!(expr, Expression::ConditionalExpression(_)) ||
                       matches!(expr, Expression::LogicalExpression(_)) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

/**
 * Helper: Check if an expression is JSX
 */
fn is_jsx_expression(expr: &Expression) -> bool {
    matches!(expr, Expression::JSXElement(_)) || matches!(expr, Expression::JSXFragment(_))
}
