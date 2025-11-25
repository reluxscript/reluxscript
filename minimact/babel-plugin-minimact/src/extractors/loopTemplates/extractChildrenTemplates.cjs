const t = require('@babel/types');
const { extractTextTemplate } = require('./extractTextTemplate.cjs');

// Forward declaration - will be set by index.cjs to avoid circular dependency
let extractElementTemplate = null;

function setExtractElementTemplate(fn) {
  extractElementTemplate = fn;
}

/**
 * Extract children templates from JSX children
 *
 * Returns array of templates (text or element)
 */
function extractChildrenTemplates(children, itemVar, indexVar) {
  const templates = [];

  for (const child of children) {
    // Static text: <li>Static text</li>
    if (t.isJSXText(child)) {
      const text = child.value.trim();
      if (text) {
        templates.push({
          type: 'Text',
          template: text,
          bindings: [],
          slots: []
        });
      }
      continue;
    }

    // Expression: <li>{todo.text}</li>
    if (t.isJSXExpressionContainer(child)) {
      const template = extractTextTemplate(child.expression, itemVar, indexVar);
      if (template) {
        templates.push(template);
      }
      continue;
    }

    // Nested element: <li><span>{todo.text}</span></li>
    if (t.isJSXElement(child)) {
      if (extractElementTemplate) {
        const elementTemplate = extractElementTemplate(child, itemVar, indexVar);
        if (elementTemplate) {
          templates.push(elementTemplate);
        }
      }
      continue;
    }
  }

  return templates;
}

module.exports = {
  extractChildrenTemplates,
  setExtractElementTemplate
};
