const t = require('@babel/types');
const { buildMemberPath } = require('./buildMemberPath.cjs');
const { extractIdentifiers } = require('./extractIdentifiers.cjs');
const { extractMethodCallBinding } = require('./extractMethodCallBinding.cjs');
const { extractConditionalBinding } = require('./extractConditionalBinding.cjs');
const { extractOptionalChainBinding } = require('./extractOptionalChainBinding.cjs');

/**
 * Extract binding name from expression
 * Supports:
 * - Identifiers: {count}
 * - Member expressions: {user.name}
 * - Simple operations: {count + 1}
 * - Conditionals: {isExpanded ? 'Hide' : 'Show'}
 * - Method calls: {price.toFixed(2)}
 * - Optional chaining: {viewModel?.userEmail}
 */
function extractBinding(expr, component) {
  if (t.isIdentifier(expr)) {
    return expr.name;
  } else if (t.isMemberExpression(expr)) {
    return buildMemberPath(expr);
  } else if (t.isOptionalMemberExpression(expr)) {
    // Optional chaining (viewModel?.userEmail)
    return extractOptionalChainBinding(expr);
  } else if (t.isCallExpression(expr)) {
    // Method calls (price.toFixed(2))
    return extractMethodCallBinding(expr);
  } else if (t.isBinaryExpression(expr) || t.isUnaryExpression(expr)) {
    // Simple operations - extract all identifiers
    const identifiers = [];
    extractIdentifiers(expr, identifiers);
    return identifiers.join('.');
  } else if (t.isConditionalExpression(expr)) {
    // Ternary expression: {isExpanded ? 'Hide' : 'Show'}
    return extractConditionalBinding(expr);
  } else {
    // Complex expression
    return null;
  }
}

module.exports = { extractBinding };
