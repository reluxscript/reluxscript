const t = require('@babel/types');

/**
 * Handle Math.* method calls
 */
function handleMathCalls(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.object, { name: 'Math' })) {
    return null;
  }

  const methodName = node.callee.property.name;
  const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(', ');

  // Handle Math.max() → Math.Max()
  if (methodName === 'max') {
    return `Math.Max(${args})`;
  }

  // Handle Math.min() → Math.Min()
  if (methodName === 'min') {
    return `Math.Min(${args})`;
  }

  // Handle other Math methods (floor, ceil, round, pow, log, etc.) → Pascal case
  const pascalMethodName = methodName.charAt(0).toUpperCase() + methodName.slice(1);

  // Cast floor/ceil/round to int for array indexing compatibility
  if (methodName === 'floor' || methodName === 'ceil' || methodName === 'round') {
    return `(int)Math.${pascalMethodName}(${args})`;
  }

  return `Math.${pascalMethodName}(${args})`;
}

module.exports = { handleMathCalls };
