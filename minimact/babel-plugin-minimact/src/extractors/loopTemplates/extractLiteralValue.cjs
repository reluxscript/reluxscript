const t = require('@babel/types');

/**
 * Extract literal value from expression
 */
function extractLiteralValue(expr) {
  if (t.isStringLiteral(expr)) {
    return expr.value;
  } else if (t.isNumericLiteral(expr)) {
    return expr.value;
  } else if (t.isBooleanLiteral(expr)) {
    return expr.value;
  } else if (t.isNullLiteral(expr)) {
    return null;
  }
  return null; // Complex expression
}

module.exports = { extractLiteralValue };
