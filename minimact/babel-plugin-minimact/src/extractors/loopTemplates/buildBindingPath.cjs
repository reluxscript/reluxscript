const t = require('@babel/types');
const { buildMemberExpressionPath } = require('./buildMemberExpressionPath.cjs');
const {
  extractLoopBinaryExpression,
  extractLoopLogicalExpression,
  extractLoopUnaryExpression,
  extractLoopCallExpression
} = require('./extractLoopExpressions.cjs');

/**
 * Build binding path from expression relative to item variable
 *
 * Examples:
 * - todo → null (just the item itself)
 * - todo.text → "item.text"
 * - todo.author.name → "item.author.name"
 * - index → "index"
 * - todo.priority + 1 → "__expr__:item.priority"
 * - todo.text.toUpperCase() → "__expr__:item.text"
 * - index * 2 + 1 → "__expr__:index"
 */
function buildBindingPath(expr, itemVar, indexVar) {
  if (t.isIdentifier(expr)) {
    // Just the item variable itself
    if (expr.name === itemVar) {
      return null; // Can't template the entire item object
    }
    // Index variable
    if (expr.name === 'index' || expr.name === indexVar) {
      return 'index';
    }
    // Other identifier (likely a closure variable)
    return null;
  }

  if (t.isMemberExpression(expr)) {
    const path = buildMemberExpressionPath(expr);
    if (path && path.startsWith(itemVar + '.')) {
      // Replace item variable with "item" prefix
      return 'item' + path.substring(itemVar.length);
    }
  }

  // Handle binary expressions: todo.priority + 1, price * quantity, etc.
  if (t.isBinaryExpression(expr)) {
    return extractLoopBinaryExpression(expr, itemVar, indexVar);
  }

  // Handle logical expressions: todo.dueDate || 'No due date'
  if (t.isLogicalExpression(expr)) {
    return extractLoopLogicalExpression(expr, itemVar, indexVar);
  }

  // Handle unary expressions: !todo.completed, -value
  if (t.isUnaryExpression(expr)) {
    return extractLoopUnaryExpression(expr, itemVar, indexVar);
  }

  // Handle call expressions: todo.text.toUpperCase(), array.concat()
  if (t.isCallExpression(expr)) {
    return extractLoopCallExpression(expr, itemVar, indexVar);
  }

  return null;
}

module.exports = { buildBindingPath };
