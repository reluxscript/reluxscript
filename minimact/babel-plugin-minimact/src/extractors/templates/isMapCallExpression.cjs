const t = require('@babel/types');

/**
 * Check if expression is a .map() call (including chained calls like .filter().map())
 */
function isMapCallExpression(expr) {
  if (!t.isCallExpression(expr)) {
    return false;
  }

  // Check if it's a direct .map() call
  if (t.isMemberExpression(expr.callee) &&
      t.isIdentifier(expr.callee.property) &&
      expr.callee.property.name === 'map') {
    return true;
  }

  // Check if it's a chained call ending in .map()
  // e.g., items.filter(...).map(...), items.slice(0, 10).map(...)
  let current = expr;
  while (t.isCallExpression(current)) {
    if (t.isMemberExpression(current.callee) &&
        t.isIdentifier(current.callee.property) &&
        current.callee.property.name === 'map') {
      return true;
    }
    // Move to the next call in the chain
    if (t.isMemberExpression(current.callee)) {
      current = current.callee.object;
    } else {
      break;
    }
  }

  return false;
}

module.exports = { isMapCallExpression };
