const t = require('@babel/types');
const { extractConditionalStructuralTemplate } = require('./extractConditionalStructuralTemplate.cjs');
const { extractLogicalAndTemplate } = require('./extractLogicalAndTemplate.cjs');

/**
 * Traverse JSX tree looking for conditional expressions that affect structure
 */
function traverseJSX(node, path, structuralTemplates, component) {
  if (t.isJSXElement(node)) {
    // Check children for conditional expressions
    for (let i = 0; i < node.children.length; i++) {
      const child = node.children[i];

      if (t.isJSXExpressionContainer(child)) {
        const expr = child.expression;

        // Ternary: {condition ? <A /> : <B />}
        if (t.isConditionalExpression(expr)) {
          const template = extractConditionalStructuralTemplate(expr, component, [...path, i]);
          if (template) {
            structuralTemplates.push(template);
          }
        }

        // Logical AND: {condition && <Component />}
        if (t.isLogicalExpression(expr) && expr.operator === '&&') {
          const template = extractLogicalAndTemplate(expr, component, [...path, i]);
          if (template) {
            structuralTemplates.push(template);
          }
        }
      } else if (t.isJSXElement(child)) {
        traverseJSX(child, [...path, i], structuralTemplates, component);
      }
    }
  } else if (t.isJSXFragment(node)) {
    for (let i = 0; i < node.children.length; i++) {
      const child = node.children[i];
      if (t.isJSXElement(child)) {
        traverseJSX(child, [...path, i], structuralTemplates, component);
      } else if (t.isJSXExpressionContainer(child)) {
        const expr = child.expression;

        if (t.isConditionalExpression(expr)) {
          const template = extractConditionalStructuralTemplate(expr, component, [...path, i]);
          if (template) {
            structuralTemplates.push(template);
          }
        }

        if (t.isLogicalExpression(expr) && expr.operator === '&&') {
          const template = extractLogicalAndTemplate(expr, component, [...path, i]);
          if (template) {
            structuralTemplates.push(template);
          }
        }
      }
    }
  }
}

module.exports = { traverseJSX };
