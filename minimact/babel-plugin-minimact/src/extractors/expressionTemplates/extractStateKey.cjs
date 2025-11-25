const t = require('@babel/types');

/**
 * Extract state key (root variable)
 */
function extractStateKey(expr, component) {
  if (t.isIdentifier(expr)) {
    return expr.name;
  } else if (t.isMemberExpression(expr)) {
    let current = expr;
    while (t.isMemberExpression(current)) {
      current = current.object;
    }
    if (t.isIdentifier(current)) {
      return current.name;
    }
  }
  return null;
}

module.exports = { extractStateKey };
