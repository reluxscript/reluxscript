const t = require('@babel/types');
const { isSimpleExpression } = require('./isSimpleExpression.cjs');

/**
 * Check if condition can be evaluated client-side
 * Only simple boolean logic is supported (&&, ||, !, comparisons)
 */
function isConditionEvaluableClientSide(expr, bindings) {
  // Simple identifier: myState1
  if (t.isIdentifier(expr)) {
    return true;
  }

  // Unary: !myState1
  if (t.isUnaryExpression(expr) && expr.operator === '!') {
    return isConditionEvaluableClientSide(expr.argument, bindings);
  }

  // Logical: myState1 && myState2, myState1 || myState2
  if (t.isLogicalExpression(expr)) {
    return isConditionEvaluableClientSide(expr.left, bindings) &&
           isConditionEvaluableClientSide(expr.right, bindings);
  }

  // Binary comparisons: count > 0, name === "admin"
  if (t.isBinaryExpression(expr)) {
    // Simple comparisons are evaluable
    const operators = ['==', '===', '!=', '!==', '<', '>', '<=', '>='];
    if (operators.includes(expr.operator)) {
      return isSimpleExpression(expr.left) && isSimpleExpression(expr.right);
    }
  }

  // Member expressions: user.isAdmin
  if (t.isMemberExpression(expr)) {
    return true;
  }

  // Complex expressions require server evaluation
  return false;
}

module.exports = { isConditionEvaluableClientSide };
