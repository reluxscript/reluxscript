const { buildBindingPath } = require('./buildBindingPath.cjs');
const { extractLiteralValue } = require('./extractLiteralValue.cjs');

/**
 * Extract conditional template from ternary expression
 *
 * Example: todo.done ? 'completed' : 'pending'
 * →
 * {
 *   template: "{0}",
 *   bindings: ["item.done"],
 *   conditionalTemplates: { "true": "completed", "false": "pending" },
 *   conditionalBindingIndex: 0
 * }
 */
function extractConditionalTemplate(conditionalExpr, itemVar, indexVar) {
  const test = conditionalExpr.test;
  const consequent = conditionalExpr.consequent;
  const alternate = conditionalExpr.alternate;

  // Extract binding from test expression
  const binding = buildBindingPath(test, itemVar, indexVar);
  if (!binding) return null;

  // Extract literal values from consequent and alternate
  const trueValue = extractLiteralValue(consequent);
  const falseValue = extractLiteralValue(alternate);

  if (trueValue === null || falseValue === null) {
    // Complex expressions in branches - can't template it
    return null;
  }

  return {
    template: '{0}',
    bindings: [binding],
    slots: [0],
    conditionalTemplates: {
      'true': trueValue,
      'false': falseValue
    },
    conditionalBindingIndex: 0,
    type: 'conditional'
  };
}

module.exports = { extractConditionalTemplate };
