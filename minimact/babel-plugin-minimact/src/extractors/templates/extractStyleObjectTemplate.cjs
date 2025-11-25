const t = require('@babel/types');
const { extractBindingShared } = require('./extractBindingShared.cjs');
const { camelToKebab, convertStyleValue } = require('./styleHelpers.cjs');

/**
 * Extract template from style object
 * Handles: { fontSize: '32px', opacity: isVisible ? 1 : 0.5 }
 */
function extractStyleObjectTemplate(objectExpr, tagName, elementIndex, parentPath, currentPath, component) {
  let hasBindings = false;
  const cssProperties = [];
  const bindings = [];
  const slots = [];
  let slotIndex = 0;

  // Check each property for dynamic values
  for (const prop of objectExpr.properties) {
    if (t.isObjectProperty(prop) && !prop.computed) {
      const key = t.isIdentifier(prop.key) ? prop.key.name : String(prop.key.value);
      const cssKey = camelToKebab(key);
      const value = prop.value;

      // Check if value is dynamic (expression, conditional, etc.)
      if (t.isConditionalExpression(value) || t.isIdentifier(value) || t.isMemberExpression(value)) {
        // Dynamic value - extract binding
        hasBindings = true;
        const binding = extractBindingShared(value, component);
        if (binding) {
          bindings.push(typeof binding === 'object' ? binding.binding || binding.conditional : binding);
          cssProperties.push(`${cssKey}: {${slotIndex}}`);
          slots.push(cssProperties.join('; ').lastIndexOf('{'));
          slotIndex++;
        } else {
          // Complex expression - fall back to static
          const cssValue = convertStyleValue(value);
          cssProperties.push(`${cssKey}: ${cssValue}`);
        }
      } else {
        // Static value
        const cssValue = convertStyleValue(value);
        cssProperties.push(`${cssKey}: ${cssValue}`);
      }
    }
  }

  const cssString = cssProperties.join('; ');

  return {
    template: cssString,
    bindings: bindings,
    slots: slots,
    path: currentPath,
    attribute: 'style',
    type: hasBindings ? 'attribute-dynamic' : 'attribute-static'
  };
}

module.exports = { extractStyleObjectTemplate };
