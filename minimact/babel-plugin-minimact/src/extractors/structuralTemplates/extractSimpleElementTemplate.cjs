const t = require('@babel/types');

/**
 * Extract simple element template (without nested state dependencies)
 *
 * For structural templates, we extract a simplified version that captures:
 * - Tag name
 * - Static props
 * - Structure (not deeply nested templates)
 */
function extractSimpleElementTemplate(jsxElement, component) {
  const tagName = jsxElement.openingElement.name.name;
  const attributes = jsxElement.openingElement.attributes;

  // Extract static props only (complex props handled separately)
  const props = {};
  for (const attr of attributes) {
    if (t.isJSXAttribute(attr)) {
      const propName = attr.name.name;
      const propValue = attr.value;

      if (t.isStringLiteral(propValue)) {
        props[propName] = propValue.value;
      } else if (t.isJSXExpressionContainer(propValue)) {
        // Mark as dynamic (will be re-evaluated)
        const expr = propValue.expression;
        if (t.isIdentifier(expr)) {
          props[propName] = { binding: expr.name };
        } else {
          props[propName] = { expression: true };
        }
      }
    }
  }

  // Extract children (simplified)
  const children = jsxElement.children
    .filter(child => t.isJSXElement(child) || t.isJSXText(child))
    .map(child => {
      if (t.isJSXElement(child)) {
        return extractSimpleElementTemplate(child, component);
      } else if (t.isJSXText(child)) {
        const text = child.value.trim();
        return text ? { type: 'Text', content: text } : null;
      }
    })
    .filter(Boolean);

  return {
    type: 'Element',
    tag: tagName,
    props: Object.keys(props).length > 0 ? props : null,
    children: children.length > 0 ? children : null
  };
}

module.exports = { extractSimpleElementTemplate };
