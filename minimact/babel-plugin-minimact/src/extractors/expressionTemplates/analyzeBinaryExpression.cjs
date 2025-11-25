const t = require('@babel/types');
const { isBindingExpression } = require('./isBindingExpression.cjs');

/**
 * Analyze binary expression to extract arithmetic operations
 *
 * Example: count * 2 + 1 with binding="count"
 * →
 * {
 *   type: 'arithmetic',
 *   operations: [
 *     { op: '*', value: 2 },
 *     { op: '+', value: 1 }
 *   ]
 * }
 */
function analyzeBinaryExpression(expr, targetBinding) {
  const operations = [];

  function analyze(node) {
    if (t.isBinaryExpression(node)) {
      const { left, operator, right } = node;

      // Check if one side is our target binding
      const leftIsTarget = isBindingExpression(left, targetBinding);
      const rightIsTarget = isBindingExpression(right, targetBinding);

      if (leftIsTarget && t.isNumericLiteral(right)) {
        operations.push({ op: operator, value: right.value, side: 'right' });
      } else if (rightIsTarget && t.isNumericLiteral(left)) {
        operations.push({ op: operator, value: left.value, side: 'left' });
      } else {
        // Recurse
        analyze(left);
        analyze(right);
      }
    }
  }

  analyze(expr);

  if (operations.length > 0) {
    return {
      type: 'arithmetic',
      operations
    };
  }

  return null;
}

module.exports = { analyzeBinaryExpression };
