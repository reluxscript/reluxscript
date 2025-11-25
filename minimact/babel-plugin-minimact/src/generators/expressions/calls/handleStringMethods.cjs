const t = require('@babel/types');

/**
 * Handle .toFixed(n) → .ToString("Fn")
 */
function handleToFixed(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'toFixed' })) {
    return null;
  }

  let object = generateCSharpExpression(node.callee.object);

  // Preserve parentheses for complex expressions
  if (t.isBinaryExpression(node.callee.object) ||
      t.isLogicalExpression(node.callee.object) ||
      t.isConditionalExpression(node.callee.object) ||
      t.isCallExpression(node.callee.object)) {
    object = `(${object})`;
  }

  const decimals = node.arguments.length > 0 && t.isNumericLiteral(node.arguments[0])
    ? node.arguments[0].value
    : 2;
  return `${object}.ToString("F${decimals}")`;
}

/**
 * Handle .toLocaleString() → .ToString("g")
 */
function handleToLocaleString(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'toLocaleString' })) {
    return null;
  }
  const object = generateCSharpExpression(node.callee.object);
  return `${object}.ToString("g")`;
}

/**
 * Handle .toLowerCase() → .ToLower()
 */
function handleToLowerCase(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'toLowerCase' })) {
    return null;
  }
  const object = generateCSharpExpression(node.callee.object);
  return `${object}.ToLower()`;
}

/**
 * Handle .toUpperCase() → .ToUpper()
 */
function handleToUpperCase(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'toUpperCase' })) {
    return null;
  }
  const object = generateCSharpExpression(node.callee.object);
  return `${object}.ToUpper()`;
}

/**
 * Handle .trim() → .Trim()
 */
function handleTrim(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'trim' })) {
    return null;
  }
  const object = generateCSharpExpression(node.callee.object);
  return `${object}.Trim()`;
}

/**
 * Handle .substring(start, end) → .Substring(start, end)
 */
function handleSubstring(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'substring' })) {
    return null;
  }
  const object = generateCSharpExpression(node.callee.object);
  const args = node.arguments.map(arg => generateCSharpExpression(arg)).join(', ');
  return `${object}.Substring(${args})`;
}

/**
 * Handle .padStart(length, char) → .PadLeft(length, char)
 */
function handlePadStart(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'padStart' })) {
    return null;
  }
  const object = generateCSharpExpression(node.callee.object);
  const length = node.arguments[0] ? generateCSharpExpression(node.arguments[0]) : '0';
  let padChar = node.arguments[1] ? generateCSharpExpression(node.arguments[1]) : '" "';

  // Convert string literal "0" to char literal '0'
  if (t.isStringLiteral(node.arguments[1]) && node.arguments[1].value.length === 1) {
    padChar = `'${node.arguments[1].value}'`;
  }

  return `${object}.PadLeft(${length}, ${padChar})`;
}

/**
 * Handle .padEnd(length, char) → .PadRight(length, char)
 */
function handlePadEnd(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'padEnd' })) {
    return null;
  }
  const object = generateCSharpExpression(node.callee.object);
  const length = node.arguments[0] ? generateCSharpExpression(node.arguments[0]) : '0';
  let padChar = node.arguments[1] ? generateCSharpExpression(node.arguments[1]) : '" "';

  // Convert string literal "0" to char literal '0'
  if (t.isStringLiteral(node.arguments[1]) && node.arguments[1].value.length === 1) {
    padChar = `'${node.arguments[1].value}'`;
  }

  return `${object}.PadRight(${length}, ${padChar})`;
}

/**
 * Handle response.json() → response.Content.ReadFromJsonAsync<dynamic>()
 */
function handleResponseJson(node, generateCSharpExpression) {
  if (!t.isMemberExpression(node.callee) ||
      !t.isIdentifier(node.callee.property, { name: 'json' })) {
    return null;
  }
  const object = generateCSharpExpression(node.callee.object);
  return `${object}.Content.ReadFromJsonAsync<dynamic>()`;
}

module.exports = {
  handleToFixed,
  handleToLocaleString,
  handleToLowerCase,
  handleToUpperCase,
  handleTrim,
  handleSubstring,
  handlePadStart,
  handlePadEnd,
  handleResponseJson
};
