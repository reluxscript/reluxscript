const t = require('@babel/types');

/**
 * Extract state key (root variable name) from expression
 */
function extractStateKey(expr, component) {
  if (t.isIdentifier(expr)) {
    return expr.name;
  } else if (t.isMemberExpression(expr)) {
    // Get root object: user.isLoggedIn → "user"
    let current = expr;
    while (t.isMemberExpression(current)) {
      current = current.object;
    }
    if (t.isIdentifier(current)) {
      return current.name;
    }
  } else if (t.isUnaryExpression(expr)) {
    return extractStateKey(expr.argument, component);
  }
  return null;
}

module.exports = { extractStateKey };
