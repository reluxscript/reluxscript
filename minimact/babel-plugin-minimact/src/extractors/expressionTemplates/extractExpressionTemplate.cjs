const t = require('@babel/types');
const { extractMethodCallTemplate } = require('./extractMethodCallTemplate.cjs');
const { extractBinaryExpressionTemplate } = require('./extractBinaryExpressionTemplate.cjs');
const { extractMemberExpressionTemplate } = require('./extractMemberExpressionTemplate.cjs');
const { extractUnaryExpressionTemplate } = require('./extractUnaryExpressionTemplate.cjs');

/**
 * Extract expression template from expression node
 */
function extractExpressionTemplate(expr, component, path) {
  // Skip if it's a simple identifier (no transformation)
  if (t.isIdentifier(expr)) {
    return null;
  }

  // Skip conditionals (handled by structural templates)
  if (t.isConditionalExpression(expr) || t.isLogicalExpression(expr)) {
    return null;
  }

  // Method call: price.toFixed(2)
  if (t.isCallExpression(expr) && t.isMemberExpression(expr.callee)) {
    return extractMethodCallTemplate(expr, component, path);
  }

  // Binary expression: count * 2 + 1
  if (t.isBinaryExpression(expr)) {
    return extractBinaryExpressionTemplate(expr, component, path);
  }

  // Member expression: user.name, items.length
  if (t.isMemberExpression(expr)) {
    return extractMemberExpressionTemplate(expr, component, path);
  }

  // Unary expression: -count, +value
  if (t.isUnaryExpression(expr)) {
    return extractUnaryExpressionTemplate(expr, component, path);
  }

  return null;
}

module.exports = { extractExpressionTemplate };
