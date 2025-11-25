const t = require('@babel/types');
const { extractLoopTemplate } = require('./extractLoopTemplate.cjs');

/**
 * Find .map() call expressions recursively
 */
function findMapExpressions(expr, loopTemplates) {
  if (!expr) return;

  // Direct .map() call: items.map(...)
  if (t.isCallExpression(expr) &&
      t.isMemberExpression(expr.callee) &&
      t.isIdentifier(expr.callee.property) &&
      expr.callee.property.name === 'map') {

    const loopTemplate = extractLoopTemplate(expr);
    if (loopTemplate) {
      loopTemplates.push(loopTemplate);
    }
  }

  // Chained operations: items.filter(...).map(...)
  if (t.isCallExpression(expr) &&
      t.isMemberExpression(expr.callee)) {
    findMapExpressions(expr.callee.object, loopTemplates);
  }

  // Wrapped in other expressions
  if (t.isLogicalExpression(expr) || t.isConditionalExpression(expr)) {
    findMapExpressions(expr.left || expr.test, loopTemplates);
    findMapExpressions(expr.right || expr.consequent, loopTemplates);
    if (expr.alternate) findMapExpressions(expr.alternate, loopTemplates);
  }
}

module.exports = { findMapExpressions };
