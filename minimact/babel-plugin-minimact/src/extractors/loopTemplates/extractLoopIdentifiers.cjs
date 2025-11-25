const t = require('@babel/types');
const { buildMemberExpressionPath } = require('./buildMemberExpressionPath.cjs');

/**
 * Extract identifiers from expression, converting item references to "item" prefix
 */
function extractLoopIdentifiers(expr, result, itemVar, indexVar) {
  if (t.isIdentifier(expr)) {
    if (expr.name === itemVar) {
      // Don't add raw item variable
      return;
    } else if (expr.name === 'index' || expr.name === indexVar) {
      result.push('index');
    } else {
      result.push(expr.name);
    }
  } else if (t.isBinaryExpression(expr) || t.isLogicalExpression(expr)) {
    extractLoopIdentifiers(expr.left, result, itemVar, indexVar);
    extractLoopIdentifiers(expr.right, result, itemVar, indexVar);
  } else if (t.isUnaryExpression(expr)) {
    extractLoopIdentifiers(expr.argument, result, itemVar, indexVar);
  } else if (t.isMemberExpression(expr)) {
    const path = buildMemberExpressionPath(expr);
    if (path) {
      if (path.startsWith(itemVar + '.')) {
        // Replace item variable with "item" prefix
        result.push('item' + path.substring(itemVar.length));
      } else {
        result.push(path);
      }
    } else {
      // Complex member expression (e.g., (a + b).toFixed())
      // Extract from both object and property
      extractLoopIdentifiers(expr.object, result, itemVar, indexVar);
      if (t.isIdentifier(expr.property)) {
        result.push(expr.property.name);
      }
    }
  } else if (t.isCallExpression(expr)) {
    // Extract from callee
    extractLoopIdentifiers(expr.callee, result, itemVar, indexVar);
    // Extract from arguments
    for (const arg of expr.arguments) {
      extractLoopIdentifiers(arg, result, itemVar, indexVar);
    }
  }
}

module.exports = { extractLoopIdentifiers };
