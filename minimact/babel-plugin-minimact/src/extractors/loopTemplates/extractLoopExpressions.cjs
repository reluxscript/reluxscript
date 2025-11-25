const { extractLoopIdentifiers } = require('./extractLoopIdentifiers.cjs');

/**
 * Extract binding from binary expression in loop
 * Examples: todo.priority + 1, price * quantity, index * 2 + 1
 */
function extractLoopBinaryExpression(expr, itemVar, indexVar) {
  const identifiers = [];
  extractLoopIdentifiers(expr, identifiers, itemVar, indexVar);

  if (identifiers.length === 0) {
    return null;
  }

  // Use __expr__ prefix to indicate this is a computed expression
  return `__expr__:${identifiers.join(',')}`;
}

/**
 * Extract binding from logical expression in loop
 * Examples: todo.dueDate || 'No due date', condition && value
 */
function extractLoopLogicalExpression(expr, itemVar, indexVar) {
  const identifiers = [];
  extractLoopIdentifiers(expr, identifiers, itemVar, indexVar);

  if (identifiers.length === 0) {
    return null;
  }

  // Use __expr__ prefix to indicate this is a computed expression
  return `__expr__:${identifiers.join(',')}`;
}

/**
 * Extract binding from unary expression in loop
 * Examples: !todo.completed, -value
 */
function extractLoopUnaryExpression(expr, itemVar, indexVar) {
  const identifiers = [];
  extractLoopIdentifiers(expr, identifiers, itemVar, indexVar);

  if (identifiers.length === 0) {
    return null;
  }

  // Use __expr__ prefix to indicate this is a computed expression
  return `__expr__:${identifiers.join(',')}`;
}

/**
 * Extract binding from call expression in loop
 * Examples: todo.text.toUpperCase(), todo.text.substring(0, 10)
 */
function extractLoopCallExpression(expr, itemVar, indexVar) {
  const identifiers = [];
  extractLoopIdentifiers(expr, identifiers, itemVar, indexVar);

  if (identifiers.length === 0) {
    return null;
  }

  // Use __expr__ prefix to indicate this is a computed expression
  return `__expr__:${identifiers.join(',')}`;
}

module.exports = {
  extractLoopBinaryExpression,
  extractLoopLogicalExpression,
  extractLoopUnaryExpression,
  extractLoopCallExpression
};
