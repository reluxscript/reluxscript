const t = require('@babel/types');

/**
 * Extract array binding from member expression
 *
 * Examples:
 * - todos.map(...) → "todos"
 * - this.state.items.map(...) → "items"
 * - [...todos].map(...) → "todos"
 */
function extractArrayBinding(expr) {
  if (t.isIdentifier(expr)) {
    return expr.name;
  } else if (t.isMemberExpression(expr)) {
    // Get the last property name
    if (t.isIdentifier(expr.property)) {
      return expr.property.name;
    }
  } else if (t.isCallExpression(expr)) {
    // Handle array methods like .reverse(), .slice()
    if (t.isMemberExpression(expr.callee)) {
      return extractArrayBinding(expr.callee.object);
    }
  } else if (t.isArrayExpression(expr)) {
    // Spread array: [...todos]
    if (expr.elements.length > 0 && t.isSpreadElement(expr.elements[0])) {
      return extractArrayBinding(expr.elements[0].argument);
    }
  }
  return null;
}

module.exports = { extractArrayBinding };
