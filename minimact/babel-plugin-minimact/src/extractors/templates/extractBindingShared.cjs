const t = require('@babel/types');
const { buildMemberPath } = require('./buildMemberPath.cjs');
const { extractIdentifiers } = require('./extractIdentifiers.cjs');

/**
 * Extract method call binding (shared version)
 * Handles: price.toFixed(2), text.toLowerCase(), etc.
 */
function extractMethodCallBindingShared(expr) {
  const callee = expr.callee;

  if (!t.isMemberExpression(callee) && !t.isOptionalMemberExpression(callee)) {
    return null;
  }

  const methodName = t.isIdentifier(callee.property) ? callee.property.name : null;
  if (!methodName) return null;

  const transformMethods = [
    'toFixed', 'toString', 'toLowerCase', 'toUpperCase',
    'trim', 'trimStart', 'trimEnd'
  ];

  if (!transformMethods.includes(methodName)) {
    return null;
  }

  let binding = null;
  if (t.isMemberExpression(callee.object)) {
    binding = buildMemberPath(callee.object);
  } else if (t.isIdentifier(callee.object)) {
    binding = callee.object.name;
  } else if (t.isBinaryExpression(callee.object)) {
    const identifiers = [];
    extractIdentifiers(callee.object, identifiers);
    binding = `__expr__:${identifiers.join(',')}`;
  }

  if (!binding) return null;

  const args = expr.arguments.map(arg => {
    if (t.isNumericLiteral(arg)) return arg.value;
    if (t.isStringLiteral(arg)) return arg.value;
    if (t.isBooleanLiteral(arg)) return arg.value;
    return null;
  }).filter(v => v !== null);

  return {
    transform: methodName,
    binding: binding,
    args: args
  };
}

/**
 * Extract binding from binary expression
 * Examples: todo.priority + 1, price * quantity, index * 2 + 1
 */
function extractBinaryExpressionBinding(expr) {
  const identifiers = [];
  extractIdentifiers(expr, identifiers);

  // Use __expr__ prefix to indicate this is a computed expression
  return `__expr__:${identifiers.join(',')}`;
}

/**
 * Extract binding from logical expression
 * Examples: todo.dueDate || 'No due date', condition && value
 */
function extractLogicalExpressionBinding(expr) {
  const identifiers = [];
  extractIdentifiers(expr, identifiers);

  // Use __expr__ prefix to indicate this is a computed expression
  return `__expr__:${identifiers.join(',')}`;
}

/**
 * Extract binding from unary expression
 * Examples: !todo.completed, -value
 */
function extractUnaryExpressionBinding(expr) {
  const identifiers = [];
  extractIdentifiers(expr, identifiers);

  // Use __expr__ prefix to indicate this is a computed expression
  return `__expr__:${identifiers.join(',')}`;
}

/**
 * Extract binding from complex call expression (non-transform methods)
 * Examples: todo.text.substring(0, 10).toUpperCase(), array.concat(other)
 */
function extractComplexCallExpression(expr) {
  const identifiers = [];
  extractIdentifiers(expr, identifiers);

  if (identifiers.length === 0) {
    return null;
  }

  // Use __expr__ prefix to indicate this is a computed expression
  return `__expr__:${identifiers.join(',')}`;
}

/**
 * Shared helper: Extract binding from expression
 */
function extractBindingShared(expr, component) {
  if (t.isIdentifier(expr)) {
    return expr.name;
  } else if (t.isMemberExpression(expr)) {
    return buildMemberPath(expr);
  } else if (t.isCallExpression(expr)) {
    // First try method call binding (toFixed, etc.)
    const methodBinding = extractMethodCallBindingShared(expr);
    if (methodBinding) {
      return methodBinding;
    }

    // Otherwise, handle chained method calls: todo.text.substring(0, 10).toUpperCase()
    return extractComplexCallExpression(expr);
  } else if (t.isBinaryExpression(expr)) {
    // Handle binary expressions: todo.priority + 1, price * quantity, etc.
    return extractBinaryExpressionBinding(expr);
  } else if (t.isLogicalExpression(expr)) {
    // Handle logical expressions: todo.dueDate || 'No due date'
    return extractLogicalExpressionBinding(expr);
  } else if (t.isUnaryExpression(expr)) {
    // Handle unary expressions: !todo.completed
    return extractUnaryExpressionBinding(expr);
  } else {
    return null;
  }
}

module.exports = {
  extractBindingShared,
  extractMethodCallBindingShared,
  extractBinaryExpressionBinding,
  extractLogicalExpressionBinding,
  extractUnaryExpressionBinding,
  extractComplexCallExpression
};
