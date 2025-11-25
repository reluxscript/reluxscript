const t = require('@babel/types');
const { extractSimpleElementTemplate } = require('./extractSimpleElementTemplate.cjs');

/**
 * Extract element or fragment template
 */
function extractElementOrFragmentTemplate(node, component) {
  if (t.isJSXElement(node)) {
    return extractSimpleElementTemplate(node, component);
  } else if (t.isJSXFragment(node)) {
    return {
      type: 'Fragment',
      children: node.children
        .filter(child => t.isJSXElement(child) || t.isJSXText(child))
        .map(child => {
          if (t.isJSXElement(child)) {
            return extractSimpleElementTemplate(child, component);
          } else if (t.isJSXText(child)) {
            const text = child.value.trim();
            return text ? { type: 'Text', content: text } : null;
          }
        })
        .filter(Boolean)
    };
  }
  return null;
}

module.exports = { extractElementOrFragmentTemplate };
