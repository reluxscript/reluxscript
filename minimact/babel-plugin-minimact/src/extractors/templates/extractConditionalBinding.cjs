const t = require('@babel/types');
const { extractLiteralValue } = require('./extractLiteralValue.cjs');

/**
 * Extract conditional binding from ternary expression
 * Returns object with test identifier and consequent/alternate values
 * Example: isExpanded ? 'Hide' : 'Show'
 * Returns: { conditional: 'isExpanded', trueValue: 'Hide', falseValue: 'Show' }
 */
function extractConditionalBinding(expr) {
  // Check if test is a simple identifier
  if (!t.isIdentifier(expr.test)) {
    // Complex test condition - mark as complex
    return null;
  }

  // Check if consequent and alternate are literals
  const trueValue = extractLiteralValue(expr.consequent);
  const falseValue = extractLiteralValue(expr.alternate);

  if (trueValue === null || falseValue === null) {
    // Not simple literals - mark as complex
    return null;
  }

  // Return conditional template metadata
  return {
    conditional: expr.test.name,
    trueValue,
    falseValue
  };
}

module.exports = { extractConditionalBinding };
