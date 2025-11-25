/**
 * Object expression handlers
 */

const t = require('@babel/types');

/**
 * Generate object expression
 */
function generateObjectExpression(node, generateCSharpExpression) {
  // Convert JS object literal to C# anonymous object or Dictionary
  // Check if any key has hyphens (invalid for C# anonymous types)
  const hasHyphenatedKeys = node.properties.some(prop => {
    if (t.isObjectProperty(prop)) {
      const key = t.isIdentifier(prop.key) ? prop.key.name : prop.key.value;
      return typeof key === 'string' && key.includes('-');
    }
    return false;
  });

  const properties = node.properties.map(prop => {
    if (t.isObjectProperty(prop)) {
      const key = t.isIdentifier(prop.key) ? prop.key.name : prop.key.value;
      const value = generateCSharpExpression(prop.value);

      if (hasHyphenatedKeys) {
        // Use Dictionary syntax with quoted keys
        return `["${key}"] = ${value}`;
      } else {
        // Use anonymous object syntax
        return `${key} = ${value}`;
      }
    }
    return '';
  }).filter(p => p !== '');

  if (properties.length === 0) return 'null';

  if (hasHyphenatedKeys) {
    return `new Dictionary<string, object> { ${properties.join(', ')} }`;
  } else {
    return `new { ${properties.join(', ')} }`;
  }
}

module.exports = {
  generateObjectExpression
};
