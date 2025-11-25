/**
 * Dependency Analyzer
 */

const t = require('@babel/types');

/**
 * Analyze dependencies in JSX expressions
 * Walk the AST manually to find identifier dependencies
 */
function analyzeDependencies(jsxExpr, component) {
  const deps = new Set();

  function walk(node) {
    if (!node) return;

    // Check if this is an identifier that's a state variable
    if (t.isIdentifier(node)) {
      const name = node.name;
      if (component.stateTypes.has(name)) {
        deps.add({
          name: name,
          type: component.stateTypes.get(name) // 'client' or 'server'
        });
      }
    }

    // Recursively walk the tree
    if (t.isConditionalExpression(node)) {
      walk(node.test);
      walk(node.consequent);
      walk(node.alternate);
    } else if (t.isLogicalExpression(node)) {
      walk(node.left);
      walk(node.right);
    } else if (t.isMemberExpression(node)) {
      walk(node.object);
      walk(node.property);
    } else if (t.isCallExpression(node)) {
      walk(node.callee);
      node.arguments.forEach(walk);
    } else if (t.isBinaryExpression(node)) {
      walk(node.left);
      walk(node.right);
    } else if (t.isUnaryExpression(node)) {
      walk(node.argument);
    } else if (t.isArrowFunctionExpression(node) || t.isFunctionExpression(node)) {
      walk(node.body);
    }
  }

  walk(jsxExpr);
  return deps;
}


module.exports = {
  analyzeDependencies
};
