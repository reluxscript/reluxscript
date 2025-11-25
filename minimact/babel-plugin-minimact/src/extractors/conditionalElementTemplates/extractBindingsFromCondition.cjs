const t = require('@babel/types');
const { buildMemberPath } = require('./buildMemberPath.cjs');

/**
 * Extract all state bindings from a condition expression
 * Example: myState1 && !myState2 → ["myState1", "myState2"]
 */
function extractBindingsFromCondition(expr) {
  const bindings = new Set();

  function traverse(node) {
    if (t.isIdentifier(node)) {
      bindings.add(node.name);
    } else if (t.isLogicalExpression(node)) {
      traverse(node.left);
      traverse(node.right);
    } else if (t.isUnaryExpression(node)) {
      traverse(node.argument);
    } else if (t.isBinaryExpression(node)) {
      traverse(node.left);
      traverse(node.right);
    } else if (t.isMemberExpression(node)) {
      const path = buildMemberPath(node);
      if (path) bindings.add(path);
    }
  }

  traverse(expr);
  return Array.from(bindings);
}

module.exports = { extractBindingsFromCondition };
