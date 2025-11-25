const t = require('@babel/types');
const { buildMemberPath } = require('./buildMemberPath.cjs');

/**
 * Generate expression string for complex expressions
 */
function generateExpressionString(expr) {
  if (t.isIdentifier(expr)) {
    return expr.name;
  } else if (t.isNumericLiteral(expr)) {
    return String(expr.value);
  } else if (t.isBinaryExpression(expr)) {
    const left = generateExpressionString(expr.left);
    const right = generateExpressionString(expr.right);
    return `${left} ${expr.operator} ${right}`;
  } else if (t.isUnaryExpression(expr)) {
    const arg = generateExpressionString(expr.argument);
    return `${expr.operator}${arg}`;
  } else if (t.isMemberExpression(expr)) {
    return buildMemberPath(expr);
  }
  return '?';
}

module.exports = { generateExpressionString };
