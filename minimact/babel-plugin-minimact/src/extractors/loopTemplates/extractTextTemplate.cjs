const t = require('@babel/types');
const { buildBindingPath } = require('./buildBindingPath.cjs');
const { extractConditionalTemplate } = require('./extractConditionalTemplate.cjs');
const { extractTemplateFromTemplateLiteral } = require('./extractTemplateFromTemplateLiteral.cjs');

/**
 * Extract text template from expression
 *
 * Handles:
 * - Simple binding: {todo.text} → { template: "{0}", bindings: ["item.text"] }
 * - Conditional: {todo.done ? '✓' : '○'} → conditional template
 * - Binary expressions: {todo.count + 1} → expression template
 * - Method calls: {todo.text.toUpperCase()} → expression template
 * - Logical expressions: {todo.date || 'N/A'} → expression template
 */
function extractTextTemplate(expr, itemVar, indexVar) {
  // Template literal: {`${user.firstName} ${user.lastName}`}
  if (t.isTemplateLiteral(expr)) {
    const templateLiteralResult = extractTemplateFromTemplateLiteral(expr, itemVar, indexVar);
    if (templateLiteralResult) {
      return {
        type: 'Text',
        ...templateLiteralResult
      };
    }
  }

  // Conditional expression: {todo.done ? '✓' : '○'}
  if (t.isConditionalExpression(expr)) {
    const conditionalTemplate = extractConditionalTemplate(expr, itemVar, indexVar);
    if (conditionalTemplate) {
      return {
        type: 'Text',
        ...conditionalTemplate
      };
    }
  }

  // Try to extract binding (handles simple, binary, method calls, etc.)
  const binding = buildBindingPath(expr, itemVar, indexVar);
  if (binding) {
    return {
      type: 'Text',
      template: '{0}',
      bindings: [binding],
      slots: [0]
    };
  }

  // No binding found
  return null;
}

module.exports = { extractTextTemplate };
