const { buildBindingPath } = require('./buildBindingPath.cjs');

/**
 * Extract template from template literal
 *
 * Example: `item-${todo.id}`
 * →
 * {
 *   template: "item-{0}",
 *   bindings: ["item.id"],
 *   slots: [5]
 * }
 */
function extractTemplateFromTemplateLiteral(templateLiteral, itemVar, indexVar) {
  let templateStr = '';
  const bindings = [];
  const slots = [];

  for (let i = 0; i < templateLiteral.quasis.length; i++) {
    const quasi = templateLiteral.quasis[i];
    templateStr += quasi.value.raw;

    if (i < templateLiteral.expressions.length) {
      const expr = templateLiteral.expressions[i];
      const binding = buildBindingPath(expr, itemVar, indexVar);

      if (binding) {
        slots.push(templateStr.length);
        templateStr += `{${bindings.length}}`;
        bindings.push(binding);
      } else {
        // Complex expression - can't template it
        return null;
      }
    }
  }

  return {
    template: templateStr,
    bindings,
    slots,
    type: 'template-literal'
  };
}

module.exports = { extractTemplateFromTemplateLiteral };
