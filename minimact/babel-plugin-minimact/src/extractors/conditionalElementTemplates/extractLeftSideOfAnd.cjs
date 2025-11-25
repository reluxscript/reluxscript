const t = require('@babel/types');

/**
 * Extract the left side of a chained AND expression
 * Example: myState1 && !myState2 && <div /> → returns myState1 && !myState2
 */
function extractLeftSideOfAnd(expr) {
  if (!t.isLogicalExpression(expr) || expr.operator !== '&&') {
    return expr;
  }

  const right = expr.right;

  // If right is JSX, left is the condition
  if (t.isJSXElement(right) || t.isJSXFragment(right)) {
    return expr.left;
  }

  // Otherwise, keep recursing
  return expr;
}

module.exports = { extractLeftSideOfAnd };
