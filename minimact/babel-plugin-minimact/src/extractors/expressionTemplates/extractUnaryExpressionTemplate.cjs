const { extractBinding } = require('./extractBinding.cjs');
const { extractStateKey } = require('./extractStateKey.cjs');

/**
 * Extract template from unary expression
 *
 * Example: -count, +value
 */
function extractUnaryExpressionTemplate(unaryExpr, component, path) {
  const { operator, argument } = unaryExpr;

  const binding = extractBinding(argument);
  if (!binding) return null;

  if (operator === '-' || operator === '+') {
    const stateKey = extractStateKey(argument, component);

    return {
      type: 'unaryExpression',
      stateKey: stateKey || binding,
      binding,
      operator,
      transform: {
        type: 'unary',
        operator
      },
      path
    };
  }

  return null;
}

module.exports = { extractUnaryExpressionTemplate };
