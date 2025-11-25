const t = require('@babel/types');
const { buildMemberPath } = require('./buildMemberPath.cjs');

/**
 * Extract all identifiers from expression
 */
function extractIdentifiers(expr, result) {
  if (t.isIdentifier(expr)) {
    result.push(expr.name);
  } else if (t.isBinaryExpression(expr)) {
    extractIdentifiers(expr.left, result);
    extractIdentifiers(expr.right, result);
  } else if (t.isUnaryExpression(expr)) {
    extractIdentifiers(expr.argument, result);
  } else if (t.isMemberExpression(expr)) {
    const path = buildMemberPath(expr);
    if (path) result.push(path);
  }
}

module.exports = { extractIdentifiers };
