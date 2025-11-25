const t = require('@babel/types');
const { buildMemberPath } = require('./buildMemberPath.cjs');

/**
 * Extract binding from expression
 */
function extractBinding(expr) {
  if (t.isIdentifier(expr)) {
    return expr.name;
  } else if (t.isMemberExpression(expr)) {
    return buildMemberPath(expr);
  }
  return null;
}

module.exports = { extractBinding };
