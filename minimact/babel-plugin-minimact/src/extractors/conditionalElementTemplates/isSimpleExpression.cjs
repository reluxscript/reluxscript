const t = require('@babel/types');

/**
 * Check if expression is simple (identifier, member, literal)
 */
function isSimpleExpression(expr) {
  return t.isIdentifier(expr) ||
         t.isMemberExpression(expr) ||
         t.isStringLiteral(expr) ||
         t.isNumericLiteral(expr) ||
         t.isBooleanLiteral(expr) ||
         t.isNullLiteral(expr);
}

module.exports = { isSimpleExpression };
