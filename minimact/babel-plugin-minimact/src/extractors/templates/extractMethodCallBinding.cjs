const t = require('@babel/types');
const { buildMemberPath, buildOptionalMemberPath } = require('./buildMemberPath.cjs');
const { extractIdentifiers } = require('./extractIdentifiers.cjs');

/**
 * Extract method call binding
 * Handles: price.toFixed(2), text.toLowerCase(), etc.
 * Returns: { transform: 'toFixed', binding: 'price', args: [2] }
 */
function extractMethodCallBinding(expr) {
  const callee = expr.callee;

  // Only handle method calls (obj.method()), not function calls (func())
  if (!t.isMemberExpression(callee) && !t.isOptionalMemberExpression(callee)) {
    return null;
  }

  const methodName = t.isIdentifier(callee.property) ? callee.property.name : null;
  if (!methodName) {
    return null;
  }

  // Supported transformation methods
  const transformMethods = [
    'toFixed', 'toString', 'toLowerCase', 'toUpperCase',
    'trim', 'trimStart', 'trimEnd'
  ];

  if (!transformMethods.includes(methodName)) {
    return null; // Unsupported method - mark as complex
  }

  // Extract the object being called (price from price.toFixed(2))
  let binding = null;
  if (t.isMemberExpression(callee.object)) {
    binding = buildMemberPath(callee.object);
  } else if (t.isOptionalMemberExpression(callee.object)) {
    binding = buildOptionalMemberPath(callee.object);
  } else if (t.isIdentifier(callee.object)) {
    binding = callee.object.name;
  } else if (t.isBinaryExpression(callee.object)) {
    // Handle expressions like (discount * 100).toFixed(0)
    // Extract all identifiers from the binary expression
    const identifiers = [];
    extractIdentifiers(callee.object, identifiers);
    binding = `__expr__:${identifiers.join(',')}`;
  }

  if (!binding) {
    return null; // Can't extract binding
  }

  // Extract method arguments (e.g., 2 from toFixed(2))
  const args = expr.arguments.map(arg => {
    if (t.isNumericLiteral(arg)) return arg.value;
    if (t.isStringLiteral(arg)) return arg.value;
    if (t.isBooleanLiteral(arg)) return arg.value;
    return null;
  }).filter(v => v !== null);

  // Return transform binding metadata
  return {
    transform: methodName,
    binding: binding,
    args: args
  };
}

module.exports = { extractMethodCallBinding };
