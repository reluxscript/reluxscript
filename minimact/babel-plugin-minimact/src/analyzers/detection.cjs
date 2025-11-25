/**
 * Pattern Detection
 */

const t = require('@babel/types');


/**
 * Detect if attributes contain spread operators
 */
function hasSpreadProps(attributes) {
  return attributes.some(attr => t.isJSXSpreadAttribute(attr));
}

/**
 * Detect if children contain dynamic patterns (like .map())
 */
function hasDynamicChildren(children) {
  return children.some(child => {
    if (!t.isJSXExpressionContainer(child)) return false;
    const expr = child.expression;

    // Check for .map() calls
    if (t.isCallExpression(expr) &&
        t.isMemberExpression(expr.callee) &&
        t.isIdentifier(expr.callee.property, { name: 'map' })) {
      return true;
    }

    // Check for array expressions from LINQ/Select
    if (t.isCallExpression(expr) &&
        t.isMemberExpression(expr.callee) &&
        (t.isIdentifier(expr.callee.property, { name: 'Select' }) ||
         t.isIdentifier(expr.callee.property, { name: 'ToArray' }))) {
      return true;
    }

    // Check for conditionals with JSX: {condition ? <A/> : <B/>}
    if (t.isConditionalExpression(expr)) {
      if (t.isJSXElement(expr.consequent) || t.isJSXFragment(expr.consequent) ||
          t.isJSXElement(expr.alternate) || t.isJSXFragment(expr.alternate)) {
        return true;
      }
    }

    // Check for logical expressions with JSX: {condition && <Element/>}
    if (t.isLogicalExpression(expr)) {
      if (t.isJSXElement(expr.right) || t.isJSXFragment(expr.right)) {
        return true;
      }
    }

    return false;
  });
}

/**
 * Detect if props contain complex expressions
 */
function hasComplexProps(attributes) {
  return attributes.some(attr => {
    if (!t.isJSXAttribute(attr)) return false;
    const value = attr.value;

    if (!t.isJSXExpressionContainer(value)) return false;
    const expr = value.expression;

    // Check for conditional spread: {...(condition && { prop: value })}
    if (t.isConditionalExpression(expr) || t.isLogicalExpression(expr)) {
      return true;
    }

    return false;
  });
}

module.exports = {
  hasSpreadProps,
  hasDynamicChildren,
  hasComplexProps
};
