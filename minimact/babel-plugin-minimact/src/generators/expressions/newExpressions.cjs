/**
 * New expression handlers
 */

const t = require('@babel/types');

/**
 * Generate new expression
 */
function generateNewExpression(node, generateCSharpExpression) {
  // Handle new Promise(resolve => setTimeout(resolve, ms)) → Task.Delay(ms)
  if (t.isIdentifier(node.callee, { name: 'Promise' }) && node.arguments.length > 0) {
    const callback = node.arguments[0];

    // Check if it's the setTimeout pattern
    if (t.isArrowFunctionExpression(callback) && callback.params.length === 1) {
      const resolveParam = callback.params[0].name;
      const body = callback.body;

      // Check if body is: setTimeout(resolve, ms)
      if (t.isCallExpression(body) &&
          t.isIdentifier(body.callee, { name: 'setTimeout' }) &&
          body.arguments.length === 2 &&
          t.isIdentifier(body.arguments[0], { name: resolveParam })) {
        const delay = generateCSharpExpression(body.arguments[1]);
        return `Task.Delay(${delay})`;
      }
    }

    // Generic Promise constructor - not directly supported in C#
    // Return Task.CompletedTask as a fallback
    return `Task.CompletedTask`;
  }

  // Handle new Date() → DateTime.Parse()
  if (t.isIdentifier(node.callee, { name: 'Date' })) {
    if (node.arguments.length === 0) {
      return 'DateTime.Now';
    } else if (node.arguments.length === 1) {
      const arg = generateCSharpExpression(node.arguments[0]);
      return `DateTime.Parse(${arg})`;
    }
  }

  // Handle new Error() → new Exception()
  if (t.isIdentifier(node.callee, { name: 'Error' })) {
    const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(', ');
    return `new Exception(${args})`;
  }

  // Handle other new expressions: new Foo() → new Foo()
  const callee = generateCSharpExpression(node.callee);
  const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(', ');
  return `new ${callee}(${args})`;
}

module.exports = {
  generateNewExpression
};
