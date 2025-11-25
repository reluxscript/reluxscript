const t = require('@babel/types');

/**
 * Handle Object.keys() → dictionary.Keys
 */
function handleObjectKeys(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.object, { name: 'Object' }) ||
      !t.isIdentifier(node.callee.property, { name: 'keys' })) {
    return null;
  }
  if (node.arguments.length > 0) {
    const obj = generateCSharpExpression(node.arguments[0]);
    return `((IDictionary<string, object>)${obj}).Keys`;
  }
  return null;
}

/**
 * Handle Date.now() → DateTimeOffset.Now.ToUnixTimeMilliseconds()
 */
function handleDateNow(node) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.object, { name: 'Date' }) ||
      !t.isIdentifier(node.callee.property, { name: 'now' })) {
    return null;
  }
  return 'DateTimeOffset.Now.ToUnixTimeMilliseconds()';
}

/**
 * Handle console.log → Console.WriteLine
 */
function handleConsoleLog(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.object, { name: 'console' }) ||
      !t.isIdentifier(node.callee.property, { name: 'log' })) {
    return null;
  }
  const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(' + ');
  return `Console.WriteLine(${args})`;
}

module.exports = {
  handleObjectKeys,
  handleDateNow,
  handleConsoleLog
};
