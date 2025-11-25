const t = require('@babel/types');
const { buildBindingPath } = require('./buildBindingPath.cjs');

/**
 * Extract key binding from JSX element
 *
 * Example: <li key={todo.id}> → "item.id"
 */
function extractKeyBinding(jsxElement, itemVar, indexVar) {
  const keyAttr = jsxElement.openingElement.attributes.find(
    attr => t.isJSXAttribute(attr) &&
            t.isIdentifier(attr.name) &&
            attr.name.name === 'key'
  );

  if (!keyAttr) return null;

  const keyValue = keyAttr.value;
  if (t.isJSXExpressionContainer(keyValue)) {
    return buildBindingPath(keyValue.expression, itemVar, indexVar);
  } else if (t.isStringLiteral(keyValue)) {
    return null; // Static key (not based on item data)
  }

  return null;
}

module.exports = { extractKeyBinding };
