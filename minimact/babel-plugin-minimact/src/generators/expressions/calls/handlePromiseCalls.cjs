const t = require('@babel/types');

/**
 * Handle Promise.resolve(value) → Task.FromResult(value)
 */
function handlePromiseResolve(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.object, { name: 'Promise' }) ||
      !t.isIdentifier(node.callee.property, { name: 'resolve' })) {
    return null;
  }
  if (node.arguments.length > 0) {
    const value = generateCSharpExpression(node.arguments[0]);
    return `Task.FromResult(${value})`;
  }
  return `Task.CompletedTask`;
}

/**
 * Handle Promise.reject(error) → Task.FromException(error)
 */
function handlePromiseReject(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.object, { name: 'Promise' }) ||
      !t.isIdentifier(node.callee.property, { name: 'reject' })) {
    return null;
  }
  if (node.arguments.length > 0) {
    const error = generateCSharpExpression(node.arguments[0]);
    return `Task.FromException(new Exception(${error}))`;
  }
  return null;
}

module.exports = {
  handlePromiseResolve,
  handlePromiseReject
};
