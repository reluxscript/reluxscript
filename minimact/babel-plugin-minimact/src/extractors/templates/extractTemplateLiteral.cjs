const { extractBindingShared } = require('./extractBindingShared.cjs');

/**
 * Extract template literal
 */
function extractTemplateLiteral(node, component) {
  let templateStr = '';
  const bindings = [];
  const slots = [];
  const transforms = [];
  const conditionals = [];

  for (let i = 0; i < node.quasis.length; i++) {
    const quasi = node.quasis[i];
    templateStr += quasi.value.raw;

    if (i < node.expressions.length) {
      const expr = node.expressions[i];
      slots.push(templateStr.length);
      templateStr += `{${i}}`;

      const binding = extractBindingShared(expr, component);

      if (binding && typeof binding === 'object' && binding.transform) {
        bindings.push(binding.binding);
        transforms.push({
          slotIndex: i,
          method: binding.transform,
          args: binding.args
        });
      } else if (binding) {
        bindings.push(binding);
      } else {
        bindings.push('__complex__');
      }
    }
  }

  const result = {
    template: templateStr,
    bindings,
    slots,
    type: 'attribute'
  };

  if (transforms.length > 0) {
    result.transforms = transforms;
  }
  if (conditionals.length > 0) {
    result.conditionals = conditionals;
  }

  return result;
}

module.exports = { extractTemplateLiteral };
