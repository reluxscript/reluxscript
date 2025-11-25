/**
 * Style Converter
 * Converts JavaScript style objects to CSS strings
 */

const t = require('@babel/types');

/**
 * Convert camelCase to kebab-case
 * Example: marginTop -> margin-top
 */
function camelToKebab(str) {
  return str.replace(/[A-Z]/g, letter => `-${letter.toLowerCase()}`);
}

/**
 * Convert a style value to CSS string
 */
function convertStyleValue(value) {
  if (t.isStringLiteral(value)) {
    return value.value;
  } else if (t.isNumericLiteral(value)) {
    // Add 'px' for numeric values (except certain properties)
    return `${value.value}px`;
  } else if (t.isIdentifier(value)) {
    return value.name;
  }
  return String(value);
}

/**
 * Convert a JavaScript style object expression to CSS string
 * Example: { marginTop: '12px', color: 'red' } -> "margin-top: 12px; color: red;"
 */
function convertStyleObjectToCss(objectExpression) {
  if (!t.isObjectExpression(objectExpression)) {
    throw new Error('Expected ObjectExpression for style');
  }

  const cssProperties = [];

  for (const prop of objectExpression.properties) {
    if (t.isObjectProperty(prop) && !prop.computed) {
      const key = t.isIdentifier(prop.key) ? prop.key.name : String(prop.key.value);
      const cssKey = camelToKebab(key);
      const cssValue = convertStyleValue(prop.value);
      cssProperties.push(`${cssKey}: ${cssValue}`);
    }
  }

  return cssProperties.join('; ');
}

module.exports = {
  convertStyleObjectToCss,
  camelToKebab
};
