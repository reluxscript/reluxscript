const { extractBinding } = require('./extractBinding.cjs');

/**
 * Check if expression is our target binding
 */
function isBindingExpression(expr, targetBinding) {
  const binding = extractBinding(expr);
  return binding === targetBinding;
}

module.exports = { isBindingExpression };
