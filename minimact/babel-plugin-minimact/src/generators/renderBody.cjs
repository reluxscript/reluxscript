/**
 * Render Body Generator
 */

const t = require('@babel/types');
const { generateJSXElement } = require('./jsx.cjs');
const { generateConditional, generateShortCircuit, generateMapExpression } = require('./expressions.cjs');

/**
 * Generate C# code for render body
 */
function generateRenderBody(node, component, indent) {
  const indentStr = '    '.repeat(indent);

  if (!node) {
    return `${indentStr}return new VText("");`;
  }

  // Handle different node types
  if (t.isJSXElement(node) || t.isJSXFragment(node)) {
    return `${indentStr}return ${generateJSXElement(node, component, indent)};`;
  }

  if (t.isConditionalExpression(node)) {
    // Ternary: condition ? a : b
    return generateConditional(node, component, indent);
  }

  if (t.isLogicalExpression(node) && node.operator === '&&') {
    // Short-circuit: condition && <Element>
    return generateShortCircuit(node, component, indent);
  }

  if (t.isCallExpression(node) && t.isMemberExpression(node.callee) && node.callee.property.name === 'map') {
    // Array.map()
    return generateMapExpression(node, component, indent);
  }

  // Fallback
  return `${indentStr}return new VText("${node.type}");`;
}

module.exports = {
  generateRenderBody
};
