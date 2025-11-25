const t = require('@babel/types');
const { buildMemberPath } = require('./buildMemberPath.cjs');

/**
 * Extract binding from expression
 */
function extractBinding(expr, component) {
  if (t.isIdentifier(expr)) {
    return expr.name;
  } else if (t.isMemberExpression(expr)) {
    return buildMemberPath(expr);
  } else if (t.isUnaryExpression(expr) && expr.operator === '!') {
    // Handle !isLoading
    const binding = extractBinding(expr.argument, component);
    return binding ? `!${binding}` : null;
  }
  return null;
}

module.exports = { extractBinding };
