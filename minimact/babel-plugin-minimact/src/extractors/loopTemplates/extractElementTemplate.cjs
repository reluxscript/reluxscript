const { extractPropTemplates } = require('./extractPropTemplates.cjs');
const { extractChildrenTemplates } = require('./extractChildrenTemplates.cjs');

/**
 * Extract element template from JSX element
 *
 * Returns template in format compatible with Rust LoopTemplate:
 * {
 *   type: "Element",
 *   tag: "li",
 *   propsTemplates: { className: { template: "{0}", bindings: ["item.done"], ... } },
 *   childrenTemplates: [ ... ],
 *   keyBinding: "item.id"
 * }
 */
function extractElementTemplate(jsxElement, itemVar, indexVar) {
  const tagName = jsxElement.openingElement.name.name;

  // Extract prop templates
  const propsTemplates = extractPropTemplates(
    jsxElement.openingElement.attributes,
    itemVar,
    indexVar
  );

  // Extract children templates
  const childrenTemplates = extractChildrenTemplates(
    jsxElement.children,
    itemVar,
    indexVar
  );

  return {
    type: 'Element',
    tag: tagName,
    propsTemplates: Object.keys(propsTemplates).length > 0 ? propsTemplates : null,
    childrenTemplates: childrenTemplates.length > 0 ? childrenTemplates : null
  };
}

module.exports = { extractElementTemplate };
