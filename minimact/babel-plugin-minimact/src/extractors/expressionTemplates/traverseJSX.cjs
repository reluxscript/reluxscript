const t = require('@babel/types');
const { extractExpressionTemplate } = require('./extractExpressionTemplate.cjs');

/**
 * Traverse JSX tree looking for expression containers
 */
function traverseJSX(node, path, expressionTemplates, component) {
  if (t.isJSXElement(node)) {
    // Check children for expressions
    for (let i = 0; i < node.children.length; i++) {
      const child = node.children[i];

      if (t.isJSXExpressionContainer(child)) {
        const template = extractExpressionTemplate(child.expression, component, [...path, i]);
        if (template) {
          expressionTemplates.push(template);
        }
      } else if (t.isJSXElement(child)) {
        traverseJSX(child, [...path, i], expressionTemplates, component);
      }
    }

    // Check attributes for expressions
    for (const attr of node.openingElement.attributes) {
      if (t.isJSXAttribute(attr) && t.isJSXExpressionContainer(attr.value)) {
        const template = extractExpressionTemplate(attr.value.expression, component, path);
        if (template) {
          template.attribute = attr.name.name;
          expressionTemplates.push(template);
        }
      }
    }
  }
}

module.exports = { traverseJSX };
