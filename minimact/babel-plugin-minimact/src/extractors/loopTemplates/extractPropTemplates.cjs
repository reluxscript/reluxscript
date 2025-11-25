const t = require('@babel/types');
const { buildBindingPath } = require('./buildBindingPath.cjs');
const { extractConditionalTemplate } = require('./extractConditionalTemplate.cjs');
const { extractTemplateFromTemplateLiteral } = require('./extractTemplateFromTemplateLiteral.cjs');

/**
 * Extract prop templates from JSX attributes
 *
 * Handles:
 * - Simple bindings: checked={todo.done} → { template: "{0}", bindings: ["item.done"] }
 * - Conditionals: className={todo.done ? 'done' : 'pending'} → conditional template
 * - Template literals: className={`item-${todo.id}`} → template with placeholder
 */
function extractPropTemplates(attributes, itemVar, indexVar) {
  const templates = {};

  for (const attr of attributes) {
    // Skip non-JSXAttribute (spreads, etc.)
    if (!t.isJSXAttribute(attr)) continue;

    // Skip key attribute (handled separately)
    if (attr.name.name === 'key') continue;

    const propName = attr.name.name;
    const propValue = attr.value;

    // Static string: className="static"
    if (t.isStringLiteral(propValue)) {
      templates[propName] = {
        template: propValue.value,
        bindings: [],
        slots: [],
        type: 'static'
      };
      continue;
    }

    // Expression: {todo.done}, {todo.done ? 'yes' : 'no'}
    if (t.isJSXExpressionContainer(propValue)) {
      const expr = propValue.expression;

      // Conditional: {todo.done ? 'active' : 'inactive'}
      if (t.isConditionalExpression(expr)) {
        const conditionalTemplate = extractConditionalTemplate(expr, itemVar, indexVar);
        if (conditionalTemplate) {
          templates[propName] = conditionalTemplate;
          continue;
        }
      }

      // Template literal: {`item-${todo.id}`}
      if (t.isTemplateLiteral(expr)) {
        const template = extractTemplateFromTemplateLiteral(expr, itemVar, indexVar);
        if (template) {
          templates[propName] = template;
          continue;
        }
      }

      // Simple binding: {todo.text}, {todo.done}
      const binding = buildBindingPath(expr, itemVar, indexVar);
      if (binding) {
        templates[propName] = {
          template: '{0}',
          bindings: [binding],
          slots: [0],
          type: 'binding'
        };
      }
    }
  }

  return templates;
}

module.exports = { extractPropTemplates };
