const t = require('@babel/types');

/**
 * Extract literal value from node (string, number, boolean)
 */
function extractLiteralValue(node) {
  if (t.isStringLiteral(node)) {
    return node.value;
  } else if (t.isNumericLiteral(node)) {
    return node.value.toString();
  } else if (t.isBooleanLiteral(node)) {
    return node.value.toString();
  } else {
    return null;
  }
}

module.exports = { extractLiteralValue };
