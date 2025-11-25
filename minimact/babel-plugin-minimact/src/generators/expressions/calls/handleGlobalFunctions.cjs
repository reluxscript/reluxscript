const t = require('@babel/types');

/**
 * Handle encodeURIComponent() → Uri.EscapeDataString()
 */
function handleEncodeURIComponent(node, generateCSharpExpression) {
  if (!t.isIdentifier(node.callee, { name: 'encodeURIComponent' })) {
    return null;
  }
  const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(', ');
  return `Uri.EscapeDataString(${args})`;
}

/**
 * Handle setState(key, value) → SetState(key, value)
 */
function handleSetState(node, generateCSharpExpression) {
  if (!t.isIdentifier(node.callee, { name: 'setState' })) {
    return null;
  }
  if (node.arguments.length >= 2) {
    const key = generateCSharpExpression(node.arguments[0]);
    const value = generateCSharpExpression(node.arguments[1]);
    return `SetState(${key}, ${value})`;
  } else {
    console.warn('[Babel Plugin] setState requires 2 arguments (key, value)');
    return `SetState("", null)`;
  }
}

/**
 * Handle fetch() → HttpClient call
 */
function handleFetch(node, generateCSharpExpression) {
  if (!t.isIdentifier(node.callee, { name: 'fetch' })) {
    return null;
  }
  const url = node.arguments.length > 0 ? generateCSharpExpression(node.arguments[0]) : '""';
  return `new HttpClient().GetAsync(${url})`;
}

/**
 * Handle alert() → Console.WriteLine()
 */
function handleAlert(node, generateCSharpExpression) {
  if (!t.isIdentifier(node.callee, { name: 'alert' })) {
    return null;
  }
  const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(' + ');
  return `Console.WriteLine(${args})`;
}

/**
 * Handle String(value) → value.ToString()
 */
function handleStringConstructor(node, generateCSharpExpression) {
  if (!t.isIdentifier(node.callee, { name: 'String' })) {
    return null;
  }
  if (node.arguments.length > 0) {
    const arg = generateCSharpExpression(node.arguments[0]);
    return `(${arg}).ToString()`;
  }
  return '""';
}

module.exports = {
  handleEncodeURIComponent,
  handleSetState,
  handleFetch,
  handleAlert,
  handleStringConstructor
};
