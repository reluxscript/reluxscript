const { extractIdentifiers } = require('./extractIdentifiers.cjs');
const { analyzeBinaryExpression } = require('./analyzeBinaryExpression.cjs');
const { generateExpressionString } = require('./generateExpressionString.cjs');

/**
 * Extract template from binary expression
 *
 * Example: count * 2 + 1
 * →
 * {
 *   type: 'binaryExpression',
 *   bindings: ['count'],
 *   expression: 'count * 2 + 1',
 *   transform: {
 *     type: 'arithmetic',
 *     operations: [
 *       { op: '*', right: 2 },
 *       { op: '+', right: 1 }
 *     ]
 *   }
 * }
 */
function extractBinaryExpressionTemplate(binaryExpr, component, path) {
  // Extract all identifiers
  const identifiers = [];
  extractIdentifiers(binaryExpr, identifiers);

  if (identifiers.length === 0) return null;

  // For simple cases (single identifier with constant), extract transform
  if (identifiers.length === 1) {
    const binding = identifiers[0];
    const transform = analyzeBinaryExpression(binaryExpr, binding);

    if (transform) {
      const stateKey = binding.split('.')[0];
      return {
        type: 'binaryExpression',
        stateKey,
        bindings: [binding],
        transform,
        path
      };
    }
  }

  // Complex multi-variable expression - store as formula
  return {
    type: 'complexExpression',
    stateKey: identifiers[0].split('.')[0],
    bindings: identifiers,
    expression: generateExpressionString(binaryExpr),
    path
  };
}

module.exports = { extractBinaryExpressionTemplate };
