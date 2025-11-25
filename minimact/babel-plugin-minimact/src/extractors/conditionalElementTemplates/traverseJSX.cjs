const t = require('@babel/types');
const { extractLogicalAndElementTemplate } = require('./extractLogicalAndElementTemplate.cjs');
const { extractTernaryElementTemplate } = require('./extractTernaryElementTemplate.cjs');

/**
 * Traverse JSX tree to find conditional expressions
 * @param {*} node - JSX node to traverse
 * @param {string|null} parentPath - Hex path of parent conditional (for nesting)
 * @param {Object} conditionalTemplates - Object to store found templates
 * @param {Map} stateKeyMap - Map of variable names to state keys
 */
function traverseJSX(node, parentPath, conditionalTemplates, stateKeyMap) {
  if (t.isJSXElement(node)) {
    // Process children
    for (const child of node.children) {
      if (t.isJSXExpressionContainer(child)) {
        const expr = child.expression;

        // Logical AND: {condition && <Element />}
        if (t.isLogicalExpression(expr) && expr.operator === '&&') {
          const template = extractLogicalAndElementTemplate(expr, child, parentPath, stateKeyMap);
          if (template) {
            const path = child.__minimactPath;
            if (path) {
              conditionalTemplates[path] = template;
              // Recursively find nested conditionals inside this template
              traverseJSX(expr.right, path, conditionalTemplates, stateKeyMap);
            }
          }
        }

        // Ternary: {condition ? <A /> : <B />}
        if (t.isConditionalExpression(expr)) {
          const template = extractTernaryElementTemplate(expr, child, parentPath, stateKeyMap);
          if (template) {
            const path = child.__minimactPath;
            if (path) {
              conditionalTemplates[path] = template;
              // Recursively find nested conditionals in both branches
              if (expr.consequent) {
                traverseJSX(expr.consequent, path, conditionalTemplates, stateKeyMap);
              }
              if (expr.alternate) {
                traverseJSX(expr.alternate, path, conditionalTemplates, stateKeyMap);
              }
            }
          }
        }
      } else if (t.isJSXElement(child)) {
        traverseJSX(child, parentPath, conditionalTemplates, stateKeyMap);
      }
    }
  }
}

module.exports = { traverseJSX };
